//! Employer-side statutory cost lines (WPM-R223, WPM-D159, WPM-D170). Pure: no database and no clock.
//!
//! A line is a cost to the employer that is **never deducted from the worker's net** and is never remitted or filed by WPM. It is a
//! stub: a rate in basis points of gross pay, up to an optional yearly wage base (a cap on the pay the rate applies to). A rate may be
//! one rate, or a table keyed by a **class** the deployer sets on the organization (an industry, a risk tier, an activity); WPM never
//! picks a class from what an organization is called, so a class-keyed line with no class set is `unresolved` and costs nothing, and
//! the answer says so (an unknown is never a zero).
//!
//! A line is written `name:rate_bp:wage_base_minor` or, for a class-keyed rate, `name@class:rate_bp:wage_base_minor`. A wage base of 0
//! is no cap. The name and the class are lowercase tokens.

/// A rate is at most 100 percent of pay.
pub const MAX_RATE_BP: u32 = 10_000;

/// One configured line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    /// The line's name, for example `unemployment_tax`.
    pub name: String,
    /// The class this rate is for; `None` for the rate that applies without one.
    pub class: Option<String>,
    /// The rate, in basis points of the pay it applies to.
    pub rate_bp: u32,
    /// The most pay in a calendar year the rate applies to, in minor units; 0 is no cap.
    pub wage_base_minor: i64,
}

/// A token: 1 to 32 lowercase letters, digits or underscores.
#[must_use]
pub fn is_token(text: &str) -> bool {
    (1..=32).contains(&text.chars().count())
        && text
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// Read a setting: each name is a line. The one name `none` is no lines.
///
/// # Errors
///
/// A name that does not read, a rate above [`MAX_RATE_BP`], a negative wage base, or the same name and class twice. A line that does
/// not read is an error naming it, never skipped (a skipped line would understate cost).
pub fn parse_lines(names: &[String]) -> Result<Vec<Line>, String> {
    if names.len() == 1 && names[0] == "none" {
        return Ok(Vec::new());
    }
    let mut out: Vec<Line> = Vec::new();
    for n in names {
        let bad = || format!("{n:?} is not name:rate_bp:wage_base_minor (or name@class:...)");
        let mut parts = n.split(':');
        let (Some(who), Some(rate), Some(base), None) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        else {
            return Err(bad());
        };
        let (name, class) = match who.split_once('@') {
            Some((name, class)) => (name, Some(class)),
            None => (who, None),
        };
        if !is_token(name) || class.is_some_and(|c| !is_token(c)) {
            return Err(bad());
        }
        let rate_bp: u32 = rate.parse().map_err(|_| bad())?;
        let wage_base_minor: i64 = base.parse().map_err(|_| bad())?;
        if rate_bp > MAX_RATE_BP || wage_base_minor < 0 {
            return Err(bad());
        }
        let class = class.map(ToString::to_string);
        if out.iter().any(|l| l.name == name && l.class == class) {
            return Err(format!("{n:?} repeats a name and class"));
        }
        out.push(Line {
            name: name.to_string(),
            class,
            rate_bp,
            wage_base_minor,
        });
    }
    Ok(out)
}

/// One cost line on a payslip.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Cost {
    /// The line's name.
    pub name: String,
    /// The class the rate was taken for, if it was class-keyed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class: Option<String>,
    /// The rate used, in basis points.
    pub rate_bp: u32,
    /// The pay the rate was applied to (gross, or what is left under the wage base).
    pub base_minor: i64,
    /// The cost to the employer, rounded half up. Never deducted from net.
    pub amount_minor: i64,
}

/// What the lines come to for one payslip.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Outcome {
    /// The costs, one per name that could be resolved.
    pub costs: Vec<Cost>,
    /// Names that have only class-keyed rates, with no class set or none that fits: costed at nothing, and said.
    pub unresolved: Vec<String>,
}

impl Outcome {
    /// The total cost, in minor units.
    ///
    /// # Errors
    ///
    /// On overflow.
    pub fn total_minor(&self) -> Result<i64, String> {
        self.costs
            .iter()
            .try_fold(0_i64, |acc, c| acc.checked_add(c.amount_minor))
            .ok_or_else(|| "employer cost overflows".to_string())
    }
}

/// The employer cost lines for a payslip. `class` is the one the deployer set on the organization (`None` if none), `gross_minor` the
/// payslip's gross, and `earlier_gross_minor` the worker's gross already paid this calendar year (for the wage base).
///
/// # Errors
///
/// A negative gross or year-to-date figure, or overflow.
pub fn cost_lines(
    lines: &[Line],
    class: Option<&str>,
    gross_minor: i64,
    earlier_gross_minor: i64,
) -> Result<Outcome, String> {
    if gross_minor < 0 || earlier_gross_minor < 0 {
        return Err("pay must not be negative".to_string());
    }
    let mut names: Vec<&str> = lines.iter().map(|l| l.name.as_str()).collect();
    names.sort_unstable();
    names.dedup();
    let mut out = Outcome::default();
    for name in names {
        let chosen = class
            .and_then(|c| {
                lines
                    .iter()
                    .find(|l| l.name == name && l.class.as_deref() == Some(c))
            })
            .or_else(|| lines.iter().find(|l| l.name == name && l.class.is_none()));
        let Some(line) = chosen else {
            out.unresolved.push(name.to_string());
            continue;
        };
        let base = if line.wage_base_minor == 0 {
            gross_minor
        } else {
            gross_minor.min((line.wage_base_minor - earlier_gross_minor).max(0))
        };
        let amount = (i128::from(base) * i128::from(line.rate_bp) + 5_000) / 10_000;
        out.costs.push(Cost {
            name: line.name.clone(),
            class: line.class.clone(),
            rate_bp: line.rate_bp,
            base_minor: base,
            amount_minor: i64::try_from(amount).map_err(|_| "employer cost overflows")?,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(items: &[&str]) -> Vec<Line> {
        parse_lines(&items.iter().map(ToString::to_string).collect::<Vec<_>>()).unwrap()
    }

    #[test]
    fn a_line_is_a_rate_on_pay_up_to_a_wage_base() {
        let l = lines(&["unemployment:250:700000"]);
        // 2.5 percent of 300 000 (under the base of 700 000 with nothing earlier).
        let o = cost_lines(&l, None, 300_000, 0).unwrap();
        assert_eq!(o.costs[0].amount_minor, 7_500);
        assert_eq!(o.total_minor().unwrap(), 7_500);
        // Only 100 000 of base is left, so only that much is costed.
        let o = cost_lines(&l, None, 300_000, 600_000).unwrap();
        assert_eq!(
            (o.costs[0].base_minor, o.costs[0].amount_minor),
            (100_000, 2_500)
        );
        // Past the base, nothing more is due, and the line is still said.
        let o = cost_lines(&l, None, 300_000, 900_000).unwrap();
        assert_eq!((o.costs[0].base_minor, o.costs[0].amount_minor), (0, 0));
    }

    #[test]
    fn no_wage_base_is_no_cap_and_rounding_is_half_up() {
        let l = lines(&["levy:333:0"]);
        // 3.33 percent of 1 001 is 33.3333: 33.
        assert_eq!(
            cost_lines(&l, None, 1_001, 5_000_000).unwrap().costs[0].amount_minor,
            33
        );
        // 3.33 percent of 1 050 is 34.965: 35.
        assert_eq!(
            cost_lines(&l, None, 1_050, 0).unwrap().costs[0].amount_minor,
            35
        );
    }

    #[test]
    fn a_class_keyed_rate_follows_the_class_the_deployer_set_and_never_a_guess() {
        let l = lines(&["injury@low:50:0", "injury@high:400:0", "pension:300:0"]);
        let low = cost_lines(&l, Some("low"), 100_000, 0).unwrap();
        assert_eq!(
            low.costs
                .iter()
                .find(|c| c.name == "injury")
                .unwrap()
                .amount_minor,
            500
        );
        let high = cost_lines(&l, Some("high"), 100_000, 0).unwrap();
        assert_eq!(
            high.costs
                .iter()
                .find(|c| c.name == "injury")
                .unwrap()
                .amount_minor,
            4_000
        );
        // No class set, or a class the table has no rate for: said, and costed at nothing, never the nearest rate.
        for class in [None, Some("medium")] {
            let o = cost_lines(&l, class, 100_000, 0).unwrap();
            assert_eq!(o.unresolved, vec!["injury".to_string()]);
            assert_eq!(o.costs.len(), 1, "the unclassed pension line still applies");
        }
    }

    #[test]
    fn an_unclassed_line_is_the_rate_when_no_class_fits() {
        let l = lines(&["injury:100:0", "injury@high:400:0"]);
        assert_eq!(
            cost_lines(&l, None, 10_000, 0).unwrap().costs[0].amount_minor,
            100
        );
        assert_eq!(
            cost_lines(&l, Some("high"), 10_000, 0).unwrap().costs[0].amount_minor,
            400
        );
        assert_eq!(
            cost_lines(&l, Some("other"), 10_000, 0).unwrap().costs[0].amount_minor,
            100
        );
    }

    #[test]
    fn a_setting_that_does_not_read_is_refused_not_skipped() {
        let bad = |s: &str| parse_lines(&[s.to_string()]).is_err();
        assert!(bad("tax"));
        assert!(bad("tax:abc:0"));
        assert!(bad("tax:10001:0"));
        assert!(bad("tax:100:-1"));
        assert!(bad("Tax:100:0"));
        assert!(bad("tax@:100:0"));
        assert!(bad("tax:100:0:1"));
        assert!(parse_lines(&["a:1:0".into(), "a:2:0".into()]).is_err());
        assert_eq!(parse_lines(&["none".to_string()]).unwrap(), vec![]);
    }

    #[test]
    fn negative_pay_is_refused() {
        assert!(cost_lines(&[], None, -1, 0).is_err());
        assert!(cost_lines(&[], None, 0, -1).is_err());
    }
}
