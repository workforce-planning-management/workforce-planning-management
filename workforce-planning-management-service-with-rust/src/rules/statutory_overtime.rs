//! Statutory overtime as bands (WPM-R219, WPM-D157, WPM-D162). DB-free and clock-free.
//!
//! Contractual overtime (minutes beyond the contracted day, [`super::workforce::overtime_minutes`]) is a different thing and stays as it
//! is. Here a rule set is a list of **bands**: a period (day, week or month), a threshold in minutes and a multiplier in basis
//! points. A minute worked past a band's threshold in its period is paid at that band's multiplier. Where bands overlap, the rule set
//! says whether the highest multiplier applies ([`Stacking::Highest`], no pyramiding) or the premiums add ([`Stacking::Add`]).
//!
//! **Exemption is a recorded fact, never inferred** from a role's duties: an exempt post has nothing to derive, and an unrecorded status
//! is `Unknown`, never "no overtime". No rule set configured says so ([`Outcome::NoLimitConfigured`]). The figures here are
//! parameters a deployer enters; none is built in. Pay uses the base hourly rate as the regular rate, and says so.

use std::collections::BTreeMap;

/// Basis points in 100 percent: the regular rate.
pub const REGULAR_BP: u32 = 10_000;

/// The most minutes in a day.
const DAY: u32 = 1_440;

/// The period a band's threshold is counted over.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Period {
    /// Minutes worked that day.
    Day,
    /// Minutes worked that week.
    Week,
    /// Minutes worked that month (the minutes before the week are an argument).
    Month,
}

/// One band: minutes past `threshold_minutes` in `period` are paid at `multiplier_bp`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Band {
    /// What the threshold is counted over.
    pub period: Period,
    /// Minutes in the period before the band starts.
    pub threshold_minutes: u32,
    /// The pay multiplier in basis points (15000 is one and a half times).
    pub multiplier_bp: u32,
}

/// How overlapping bands combine for one minute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stacking {
    /// The highest multiplier of the bands that apply: no pyramiding.
    Highest,
    /// The premiums (each multiplier less the regular rate) add.
    Add,
}

/// A configured rule set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rules {
    /// The bands. Empty means no limit is configured.
    pub bands: Vec<Band>,
    /// How overlapping bands combine.
    pub stacking: Stacking,
}

/// Read a bands setting: each name is `period:threshold_minutes:multiplier_bp`, and the one name `none` is no bands. A name that does not
/// read is an error naming it, never skipped (a skipped band would pay too little).
///
/// # Errors
/// A name that is not `period:minutes:bp` with a known period, whole numbers, a threshold of at most a month and a multiplier above
/// the regular rate (and at most 1000 percent).
pub fn parse_premiums(names: &[String]) -> Result<Vec<Band>, String> {
    if names.len() == 1 && names[0] == "none" {
        return Ok(Vec::new());
    }
    names
        .iter()
        .map(|n| {
            let bad = || format!("{n:?} is not period:threshold_minutes:multiplier_bp");
            let mut parts = n.split(':');
            let (Some(p), Some(t), Some(m), None) =
                (parts.next(), parts.next(), parts.next(), parts.next())
            else {
                return Err(bad());
            };
            let period = match p {
                "day" => Period::Day,
                "week" => Period::Week,
                "month" => Period::Month,
                _ => return Err(bad()),
            };
            let threshold_minutes: u32 = t.parse().map_err(|_| bad())?;
            let multiplier_bp: u32 = m.parse().map_err(|_| bad())?;
            if threshold_minutes > 44_640 || multiplier_bp <= REGULAR_BP || multiplier_bp > 100_000
            {
                return Err(bad());
            }
            Ok(Band {
                period,
                threshold_minutes,
                multiplier_bp,
            })
        })
        .collect()
}

/// The recorded exemption of a post: a human's entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Recorded as exempt.
    Exempt,
    /// Recorded as not exempt.
    NonExempt,
    /// Nobody recorded it.
    Unknown,
}

impl Status {
    /// The stored name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Exempt => "exempt",
            Status::NonExempt => "non_exempt",
            Status::Unknown => "unknown",
        }
    }

    /// Read a stored or submitted name; anything else is `None`, never a guess.
    #[must_use]
    pub fn parse(name: &str) -> Option<Status> {
        [Status::Exempt, Status::NonExempt, Status::Unknown]
            .into_iter()
            .find(|s| s.as_str() == name)
    }
}

/// What the derivation comes to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// The post is recorded as exempt: nothing to derive.
    Exempt,
    /// The status is not recorded: not "no overtime".
    Unknown,
    /// No bands are configured: no limit to apply.
    NoLimitConfigured,
    /// Minutes by multiplier (basis points), the regular rate included.
    Derived(Derived),
}

/// Minutes by effective multiplier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Derived {
    /// Minutes at each effective multiplier in basis points, regular rate included; ascending.
    pub minutes_at: BTreeMap<u32, u32>,
}

impl Derived {
    /// Minutes paid above the regular rate.
    #[must_use]
    pub fn overtime_minutes(&self) -> u32 {
        self.minutes_at
            .iter()
            .filter(|(bp, _)| **bp > REGULAR_BP)
            .map(|(_, m)| *m)
            .sum()
    }

    /// The premium pay in minor units: the extra over the regular rate, on the base hourly rate, rounded half up once at the end.
    /// `None` when the arithmetic would overflow, never a wrong figure.
    #[must_use]
    pub fn premium_minor(&self, hourly_minor: i64) -> Option<i64> {
        let mut total: i128 = 0;
        for (bp, minutes) in &self.minutes_at {
            let extra = i128::from(*bp) - i128::from(REGULAR_BP);
            total += i128::from(*minutes) * i128::from(hourly_minor) * extra;
        }
        let unit = 60 * i128::from(REGULAR_BP);
        let rounded = (total * 2 + unit * total.signum()) / (2 * unit);
        i64::try_from(rounded).ok()
    }
}

fn multiplier(rules: &Rules, day_pos: u32, week_pos: u32, month_pos: u32) -> u32 {
    let hits = rules.bands.iter().filter(|b| {
        let pos = match b.period {
            Period::Day => day_pos,
            Period::Week => week_pos,
            Period::Month => month_pos,
        };
        pos > b.threshold_minutes
    });
    match rules.stacking {
        Stacking::Highest => hits
            .map(|b| b.multiplier_bp)
            .max()
            .unwrap_or(REGULAR_BP)
            .max(REGULAR_BP),
        Stacking::Add => {
            REGULAR_BP
                + hits
                    .map(|b| b.multiplier_bp.saturating_sub(REGULAR_BP))
                    .sum::<u32>()
        }
    }
}

/// Derive one week. `days` are the minutes worked each day, in order (a day is capped at 1440). `month_before_minutes` is what the
/// person had worked earlier in the same month before this week, for a monthly band.
#[must_use]
pub fn derive(status: Status, rules: &Rules, days: &[u32], month_before_minutes: u32) -> Outcome {
    match status {
        Status::Exempt => return Outcome::Exempt,
        Status::Unknown => return Outcome::Unknown,
        Status::NonExempt => {}
    }
    if rules.bands.is_empty() {
        return Outcome::NoLimitConfigured;
    }
    let mut minutes_at: BTreeMap<u32, u32> = BTreeMap::new();
    let mut week_pos: u32 = 0;
    for worked in days {
        for day_pos in 1..=(*worked).min(DAY) {
            week_pos += 1;
            let bp = multiplier(rules, day_pos, week_pos, month_before_minutes + week_pos);
            *minutes_at.entry(bp).or_insert(0) += 1;
        }
    }
    Outcome::Derived(Derived { minutes_at })
}

#[cfg(test)]
mod tests {
    use super::*;

    const H: u32 = 60;

    fn band(period: Period, hours: u32, bp: u32) -> Band {
        Band {
            period,
            threshold_minutes: hours * H,
            multiplier_bp: bp,
        }
    }

    fn weekly40() -> Rules {
        Rules {
            bands: vec![band(Period::Week, 40, 15_000)],
            stacking: Stacking::Highest,
        }
    }

    fn derived(o: Outcome) -> Derived {
        match o {
            Outcome::Derived(d) => d,
            other => panic!("expected a derivation, got {other:?}"),
        }
    }

    #[test]
    fn a_forty_hour_week_has_no_overtime_and_a_forty_five_hour_week_has_five_hours() {
        let d = derived(derive(Status::NonExempt, &weekly40(), &[8 * H; 5], 0));
        assert_eq!(d.overtime_minutes(), 0);
        let d = derived(derive(Status::NonExempt, &weekly40(), &[9 * H; 5], 0));
        assert_eq!(d.overtime_minutes(), 5 * H);
        assert_eq!(d.minutes_at[&REGULAR_BP], 40 * H);
        assert_eq!(d.minutes_at[&15_000], 5 * H);
    }

    #[test]
    fn a_daily_band_pays_the_long_day_and_a_double_time_band_pays_more() {
        let rules = Rules {
            bands: vec![band(Period::Day, 8, 15_000), band(Period::Day, 12, 20_000)],
            stacking: Stacking::Highest,
        };
        let eight = derived(derive(Status::NonExempt, &rules, &[8 * H], 0));
        assert_eq!(eight.overtime_minutes(), 0, "an 8-hour day is not overtime");
        let twelve = derived(derive(Status::NonExempt, &rules, &[12 * H], 0));
        assert_eq!(twelve.minutes_at[&15_000], 4 * H);
        assert!(
            !twelve.minutes_at.contains_key(&20_000),
            "12 hours is not past 12"
        );
        let fourteen = derived(derive(Status::NonExempt, &rules, &[14 * H], 0));
        assert_eq!(fourteen.minutes_at[&15_000], 4 * H);
        assert_eq!(
            fourteen.minutes_at[&20_000],
            2 * H,
            "the hours past twelve are double"
        );
    }

    #[test]
    fn overlapping_bands_do_not_pyramid_unless_the_rules_say_to_add() {
        let bands = vec![band(Period::Day, 8, 15_000), band(Period::Week, 40, 15_000)];
        let days = [10 * H; 5]; // 50 hours: 10 are past 40, 10 are past 8 in a day
        let highest = derived(derive(
            Status::NonExempt,
            &Rules {
                bands: bands.clone(),
                stacking: Stacking::Highest,
            },
            &days,
            0,
        ));
        assert_eq!(highest.minutes_at.get(&20_000), None, "no pyramiding");
        assert_eq!(
            highest.overtime_minutes(),
            18 * H,
            "two hours a day past eight on four days, and all ten of the last day past forty"
        );
        let add = derived(derive(
            Status::NonExempt,
            &Rules {
                bands,
                stacking: Stacking::Add,
            },
            &days,
            0,
        ));
        assert_eq!(
            add.minutes_at[&20_000],
            2 * H,
            "only the last day's two hours past eight are also past forty"
        );
    }

    #[test]
    fn a_monthly_band_counts_from_what_was_worked_earlier_in_the_month() {
        let rules = Rules {
            bands: vec![band(Period::Month, 100, 15_000)],
            stacking: Stacking::Highest,
        };
        let d = derived(derive(Status::NonExempt, &rules, &[8 * H; 5], 80 * H));
        assert_eq!(
            d.overtime_minutes(),
            20 * H,
            "20 hours past 100 in the month"
        );
    }

    #[test]
    fn an_exempt_post_an_unrecorded_status_and_no_rules_are_each_said_not_zero() {
        assert_eq!(
            derive(Status::Exempt, &weekly40(), &[12 * H; 5], 0),
            Outcome::Exempt
        );
        assert_eq!(
            derive(Status::Unknown, &weekly40(), &[12 * H; 5], 0),
            Outcome::Unknown
        );
        let none = Rules {
            bands: vec![],
            stacking: Stacking::Highest,
        };
        assert_eq!(
            derive(Status::NonExempt, &none, &[12 * H; 5], 0),
            Outcome::NoLimitConfigured
        );
    }

    #[test]
    fn a_band_below_the_regular_rate_never_lowers_pay() {
        let rules = Rules {
            bands: vec![band(Period::Week, 1, 5_000)],
            stacking: Stacking::Highest,
        };
        let d = derived(derive(Status::NonExempt, &rules, &[2 * H], 0));
        assert_eq!(
            d.minutes_at.keys().copied().collect::<Vec<_>>(),
            vec![REGULAR_BP]
        );
    }

    #[test]
    fn a_status_name_reads_back_and_a_stray_one_is_refused() {
        for s in [Status::Exempt, Status::NonExempt, Status::Unknown] {
            assert_eq!(Status::parse(s.as_str()), Some(s));
        }
        assert_eq!(Status::parse("Exempt"), None);
        assert_eq!(Status::parse(""), None);
    }

    #[test]
    fn bands_read_from_names_and_a_bad_name_is_an_error_not_a_skipped_band() {
        let ok = parse_premiums(&["day:480:15000".into(), "week:2400:15000".into()]).unwrap();
        assert_eq!(
            ok,
            vec![band(Period::Day, 8, 15_000), band(Period::Week, 40, 15_000)]
        );
        assert_eq!(parse_premiums(&["none".into()]).unwrap(), vec![]);
        for bad in [
            "day:480",
            "year:480:15000",
            "day:x:15000",
            "day:480:10000",
            "day:480:15000:1",
            "day:99999:15000",
            "day:480:100001",
            "",
        ] {
            assert!(parse_premiums(&[bad.to_string()]).is_err(), "{bad:?}");
        }
        assert!(
            parse_premiums(&["day:480:15000".into(), "oops".into()]).is_err(),
            "one bad name fails them all"
        );
    }

    #[test]
    fn a_day_is_capped_at_the_minutes_in_it() {
        let d = derived(derive(Status::NonExempt, &weekly40(), &[5_000], 0));
        assert_eq!(d.minutes_at.values().sum::<u32>(), 1_440);
    }

    #[test]
    fn the_premium_is_the_extra_over_the_regular_rate_rounded_once() {
        // 5 hours at one and a half times on 20.00 an hour: 5 * 20.00 * 0.5 = 50.00.
        let d = derived(derive(Status::NonExempt, &weekly40(), &[9 * H; 5], 0));
        assert_eq!(d.premium_minor(2_000), Some(5_000));
        // One minute at 150% on 20.01 an hour: 2001 * 0.5 / 60 = 16.675 -> 17.
        let one = Derived {
            minutes_at: BTreeMap::from([(15_000, 1)]),
        };
        assert_eq!(one.premium_minor(2_001), Some(17));
        let huge = Derived {
            minutes_at: BTreeMap::from([(15_000, u32::MAX)]),
        };
        assert_eq!(
            huge.premium_minor(i64::MAX),
            None,
            "overflow is None, not a wrong figure"
        );
    }
}
