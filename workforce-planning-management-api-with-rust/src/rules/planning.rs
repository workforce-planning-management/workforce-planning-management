//! Pure rules for **strategic workforce planning** (WPM-R36, WPM-R37,
//! WPM-D26–D28): workforce plans, demand lines, and the transparent
//! arithmetic behind the supply projection and the headcount gap.
//!
//! DB-free and clock-free (dates are always supplied). Forecasting here is
//! **stated assumptions plus arithmetic** — no model, nothing imputed: with
//! too little history the answer is `None` (`insufficient_history`), never
//! an extrapolation (WPM-D27).

use chrono::NaiveDate;

use crate::rules::lifecycle;

/// Plan lifecycle statuses.
pub const STATUSES: &[&str] = &["draft", "active", "archived"];

/// The legal plan transitions; `archived` is terminal.
pub const MACHINE: &[(&str, &str)] = &[
    ("draft", "active"),
    ("draft", "archived"),
    ("active", "archived"),
];

/// The most headcount one demand line may ask for.
pub const MAX_LINE_HEADCOUNT: i32 = 100_000;

/// Snapshots needed before an observed attrition rate is offered (the first
/// has no leavers figure, so this is `MIN_SNAPSHOTS - 1` measured windows).
pub const MIN_SNAPSHOTS: usize = 3;

/// The shortest span of history (days) an observed rate may rest on.
pub const MIN_WINDOW_DAYS: i64 = 60;

/// Check a plan status transition.
///
/// # Errors
/// A message naming the current state.
pub fn check_transition(from: &str, to: &str) -> Result<(), String> {
    lifecycle::check("workforce plan", MACHINE, from, to)
}

/// Validate a plan: a name, an ordered horizon, and an attrition
/// assumption (basis points per year, 0–10000) when given.
///
/// # Errors
/// A message naming the offending field.
pub fn validate_plan(
    name: &str,
    horizon_start: NaiveDate,
    horizon_end: NaiveDate,
    attrition_bp: Option<i32>,
) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("name is required".to_string());
    }
    if horizon_end < horizon_start {
        return Err("horizon_end must not be before horizon_start".to_string());
    }
    if attrition_bp.is_some_and(|bp| !(0..=10_000).contains(&bp)) {
        return Err("attrition_bp must be between 0 and 10000".to_string());
    }
    Ok(())
}

/// Validate a demand line: a department, a target date inside the plan's
/// horizon, and a sane headcount.
///
/// # Errors
/// A message naming the offending field.
pub fn validate_line(
    department: &str,
    target_on: NaiveDate,
    horizon_start: NaiveDate,
    horizon_end: NaiveDate,
    target_headcount: i32,
) -> Result<(), String> {
    if department.trim().is_empty() {
        return Err("department is required".to_string());
    }
    if target_on < horizon_start || target_on > horizon_end {
        return Err("target_on must fall within the plan's horizon".to_string());
    }
    if !(0..=MAX_LINE_HEADCOUNT).contains(&target_headcount) {
        return Err(format!(
            "target_headcount must be between 0 and {MAX_LINE_HEADCOUNT}"
        ));
    }
    Ok(())
}

/// One recorded headcount snapshot, aggregated over the series being rated.
#[derive(Debug, Clone, Copy)]
pub struct Snap {
    /// Snapshot date.
    pub as_of: NaiveDate,
    /// Employed headcount on that date.
    pub headcount: i64,
    /// Leavers since the previous snapshot; `None` for the first.
    pub leavers: Option<i64>,
}

/// Observed annual attrition in basis points from a series of snapshots:
/// leavers over the window, annualised, divided by the mean headcount.
/// `None` — *insufficient history* — when there are fewer than
/// [`MIN_SNAPSHOTS`] snapshots with known leavers, the span is shorter than
/// [`MIN_WINDOW_DAYS`], or the mean headcount is zero.
#[must_use]
pub fn observed_attrition_bp(snaps: &[Snap]) -> Option<i32> {
    let mut sorted = snaps.to_vec();
    sorted.sort_by_key(|s| s.as_of);
    if sorted.len() < MIN_SNAPSHOTS {
        return None;
    }
    // Every snapshot after the first must say how many people left.
    let known: Vec<&Snap> = sorted.iter().skip(1).collect();
    if known.iter().any(|s| s.leavers.is_none()) {
        return None;
    }
    let first = sorted.first()?;
    let last = sorted.last()?;
    let days = (last.as_of - first.as_of).num_days();
    if days < MIN_WINDOW_DAYS {
        return None;
    }
    let leavers: i64 = known.iter().filter_map(|s| s.leavers).sum();
    let mean_headcount_x = sorted.iter().map(|s| s.headcount).sum::<i64>();
    let n = i64::try_from(sorted.len()).ok()?;
    if mean_headcount_x == 0 {
        return None;
    }
    // bp = leavers * (365 / days) / (sum / n) * 10000, in integer arithmetic.
    let numerator = i128::from(leavers) * 365 * i128::from(n) * 10_000;
    let denominator = i128::from(days) * i128::from(mean_headcount_x);
    let bp = (numerator + denominator / 2) / denominator;
    i32::try_from(bp).ok()
}

/// Projected supply after `days` at an annual attrition of `attrition_bp`
/// basis points, from `opening` employed workers; **no hires are assumed**.
/// Rounded to the nearest whole person and never negative.
#[must_use]
pub fn project_supply(opening: usize, attrition_bp: i32, days: i64) -> i64 {
    let opening = i128::try_from(opening).unwrap_or(i128::MAX);
    let days = i128::from(days.max(0));
    let bp = i128::from(attrition_bp.max(0));
    let denominator = 10_000 * 365;
    let leavers = (opening * bp * days + denominator / 2) / denominator;
    i64::try_from((opening - leavers).max(0)).unwrap_or(i64::MAX)
}

/// Headcount gap: demand minus projected supply. Positive is a shortfall,
/// negative a surplus.
#[must_use]
pub fn headcount_gap(demand: i64, supply: i64) -> i64 {
    demand - supply
}

/// Suggested levers for a headcount gap — suggestions with evidence, never
/// decisions (WPM-D28).
#[must_use]
pub fn suggested_levers(gap: i64) -> Vec<&'static str> {
    match gap.cmp(&0) {
        std::cmp::Ordering::Greater => vec!["hire", "reskill", "promote"],
        std::cmp::Ordering::Less => vec!["redeploy"],
        std::cmp::Ordering::Equal => Vec::new(),
    }
}

/// People still needed with a skill: the line's headcount less those
/// already proficient (never negative).
#[must_use]
pub fn competency_shortfall(needed: usize, proficient: usize) -> usize {
    needed.saturating_sub(proficient)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).expect("valid date")
    }

    /// Plans are one-way: draft → active → archived.
    #[test]
    fn lifecycle_machine() {
        assert!(check_transition("draft", "active").is_ok());
        assert!(check_transition("active", "archived").is_ok());
        assert!(check_transition("draft", "archived").is_ok());
        assert!(check_transition("archived", "active").is_err());
        assert!(check_transition("active", "draft").is_err());
        for (from, to) in MACHINE {
            assert!(STATUSES.contains(from) && STATUSES.contains(to));
        }
    }

    /// A plan needs a name, an ordered horizon, and an in-range assumption.
    #[test]
    fn plans_are_validated() {
        let (a, b) = (day(2027, 1, 1), day(2027, 12, 31));
        assert!(validate_plan("FY27", a, b, Some(1200)).is_ok());
        assert!(validate_plan("FY27", a, b, None).is_ok());
        assert!(validate_plan(" ", a, b, None).is_err());
        assert!(validate_plan("x", b, a, None).is_err());
        assert!(validate_plan("x", a, b, Some(10_001)).is_err());
        assert!(validate_plan("x", a, b, Some(-1)).is_err());
    }

    /// A demand line must sit inside the horizon with a sane headcount.
    #[test]
    fn lines_are_validated() {
        let (a, b) = (day(2027, 1, 1), day(2027, 12, 31));
        assert!(validate_line("eng", day(2027, 6, 30), a, b, 12).is_ok());
        assert!(
            validate_line("eng", a, a, b, 0).is_ok(),
            "zero is a real target"
        );
        assert!(validate_line("", day(2027, 6, 30), a, b, 1).is_err());
        assert!(validate_line("eng", day(2028, 1, 1), a, b, 1).is_err());
        assert!(validate_line("eng", day(2027, 6, 30), a, b, -1).is_err());
        assert!(validate_line("eng", day(2027, 6, 30), a, b, 100_001).is_err());
    }

    fn snap(d: NaiveDate, headcount: i64, leavers: Option<i64>) -> Snap {
        Snap {
            as_of: d,
            headcount,
            leavers,
        }
    }

    /// An observed rate needs enough snapshots over enough time; otherwise
    /// it is `None`, not a guess.
    #[test]
    fn observed_attrition_needs_history() {
        assert_eq!(observed_attrition_bp(&[]), None);
        let two = [
            snap(day(2026, 1, 1), 100, None),
            snap(day(2026, 7, 1), 100, Some(5)),
        ];
        assert_eq!(
            observed_attrition_bp(&two),
            None,
            "two snapshots are not enough"
        );
        let gap = [
            snap(day(2026, 1, 1), 100, None),
            snap(day(2026, 7, 1), 100, None),
            snap(day(2027, 1, 1), 100, Some(5)),
        ];
        assert_eq!(
            observed_attrition_bp(&gap),
            None,
            "a snapshot without a leavers figure breaks the window"
        );
        let short = [
            snap(day(2026, 1, 1), 100, None),
            snap(day(2026, 1, 8), 100, Some(1)),
            snap(day(2026, 1, 15), 100, Some(1)),
            snap(day(2026, 1, 22), 100, Some(1)),
        ];
        assert_eq!(
            observed_attrition_bp(&short),
            None,
            "three weeks is too short"
        );
    }

    /// 10 leavers over 365 days at a mean headcount of 100 is 1000 bp.
    #[test]
    fn observed_attrition_is_annualised() {
        let series = [
            snap(day(2026, 1, 1), 100, None),
            snap(day(2026, 7, 1), 100, Some(5)),
            snap(day(2027, 1, 1), 100, Some(5)),
        ];
        // 365 days: 10 leavers / mean 100 = 10% = 1000 bp.
        assert_eq!(observed_attrition_bp(&series), Some(1000));
        // Order of input does not matter.
        let shuffled = [series[2], series[0], series[1]];
        assert_eq!(observed_attrition_bp(&shuffled), Some(1000));
        // No leavers is a real 0, not absent.
        let none = [
            snap(day(2026, 1, 1), 50, None),
            snap(day(2026, 7, 1), 50, Some(0)),
            snap(day(2027, 1, 1), 50, Some(0)),
        ];
        assert_eq!(observed_attrition_bp(&none), Some(0));
        let empty = [
            snap(day(2026, 1, 1), 0, None),
            snap(day(2026, 7, 1), 0, Some(0)),
            snap(day(2027, 1, 1), 0, Some(0)),
        ];
        assert_eq!(observed_attrition_bp(&empty), None, "no base ⇒ no rate");
    }

    /// Supply falls by expected leavers, rounds to a person, never goes
    /// below zero, and assumes no hires.
    #[test]
    fn supply_projection() {
        assert_eq!(project_supply(100, 1000, 365), 90);
        assert_eq!(project_supply(100, 1000, 0), 100, "today is today");
        assert_eq!(project_supply(100, 0, 365), 100);
        assert_eq!(
            project_supply(100, 1000, 182),
            95,
            "half a year ≈ 5 leavers"
        );
        assert_eq!(project_supply(3, 10_000, 365 * 5), 0, "never negative");
        assert_eq!(project_supply(0, 1000, 365), 0);
        assert_eq!(
            project_supply(100, 1000, -5),
            100,
            "past dates do not grow supply"
        );
    }

    /// Gap sign and lever suggestions.
    #[test]
    fn gap_and_levers() {
        assert_eq!(headcount_gap(12, 9), 3);
        assert_eq!(headcount_gap(8, 9), -1);
        assert_eq!(suggested_levers(3), ["hire", "reskill", "promote"]);
        assert_eq!(suggested_levers(-1), ["redeploy"]);
        assert_eq!(suggested_levers(0), Vec::<&str>::new());
        assert_eq!(competency_shortfall(10, 4), 6);
        assert_eq!(competency_shortfall(3, 8), 0);
    }
}
