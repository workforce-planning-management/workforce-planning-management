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

use chrono::{Datelike, Duration, NaiveDate};

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

/// What kind of day a minute was worked on. A public holiday outranks a rest day that falls on the same date.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DayKind {
    /// An ordinary working day.
    #[default]
    Ordinary,
    /// The worker's weekly rest day (the jurisdiction's weekend).
    Rest,
    /// A public holiday of the worker's place of work.
    Holiday,
}

/// The multipliers for work on a rest day or a holiday, each in basis points; `None` is not configured (the day pays as any other).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Kinds {
    /// Every minute worked on a rest day.
    pub rest_day_bp: Option<u32>,
    /// Every minute worked on a public holiday.
    pub holiday_bp: Option<u32>,
}

impl Kinds {
    fn bp(self, kind: DayKind) -> Option<u32> {
        match kind {
            DayKind::Ordinary => None,
            DayKind::Rest => self.rest_day_bp,
            DayKind::Holiday => self.holiday_bp,
        }
    }
}

/// How overlapping bands combine for one minute.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Stacking {
    /// The highest multiplier of the bands that apply: no pyramiding.
    #[default]
    Highest,
    /// The premiums (each multiplier less the regular rate) add.
    Add,
}

/// A configured rule set.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Rules {
    /// The bands. Empty means no limit is configured.
    pub bands: Vec<Band>,
    /// How overlapping bands combine.
    pub stacking: Stacking,
    /// The multipliers for a rest day and a holiday. They combine with the bands as another band would.
    pub kinds: Kinds,
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

/// Group worked minutes by week, each week's days in date order, with a week starting on ISO weekday `week_start` (1 Monday to 7
/// Sunday; anything else is read as Monday). Days with no entry are left out, which does not change a derivation.
#[must_use]
pub fn weeks(days: &BTreeMap<NaiveDate, u32>, week_start: u32) -> Vec<Vec<u32>> {
    weeks_with(days, week_start, |_| DayKind::Ordinary)
        .into_iter()
        .map(|week| week.into_iter().map(|(m, _)| m).collect())
        .collect()
}

/// [`weeks`] with each day's kind, given by `kind_of`.
#[must_use]
pub fn weeks_with(
    days: &BTreeMap<NaiveDate, u32>,
    week_start: u32,
    kind_of: impl Fn(NaiveDate) -> DayKind,
) -> Vec<Vec<(u32, DayKind)>> {
    let start = i64::from(if (1..=7).contains(&week_start) {
        week_start
    } else {
        1
    });
    let mut by_week: BTreeMap<NaiveDate, Vec<(u32, DayKind)>> = BTreeMap::new();
    for (date, minutes) in days {
        let back = (i64::from(date.weekday().number_from_monday()) - start).rem_euclid(7);
        let first = *date - Duration::days(back);
        by_week
            .entry(first)
            .or_default()
            .push((*minutes, kind_of(*date)));
    }
    by_week.into_values().collect()
}

/// Derive a run's period: each week on its own, the minutes at each multiplier added up. A monthly band is not derived across weeks
/// here (it needs the month's earlier minutes), so it says so rather than leaving it out.
#[must_use]
pub fn derive_period(status: Status, rules: &Rules, weeks: &[Vec<u32>]) -> Outcome {
    let weeks: Vec<Vec<(u32, DayKind)>> = weeks
        .iter()
        .map(|w| w.iter().map(|m| (*m, DayKind::Ordinary)).collect())
        .collect();
    derive_period_kinds(status, rules, &weeks)
}

/// [`derive_period`] with the kind of each day.
#[must_use]
pub fn derive_period_kinds(
    status: Status,
    rules: &Rules,
    weeks: &[Vec<(u32, DayKind)>],
) -> Outcome {
    if status == Status::NonExempt && rules.bands.iter().any(|b| b.period == Period::Month) {
        return Outcome::NotDerived("monthly_band");
    }
    // The status and the rules decide first; only a derivable post has weeks to add up.
    if !matches!(derive_kinds(status, rules, &[], 0), Outcome::Derived(_)) {
        return derive_kinds(status, rules, &[], 0);
    }
    let mut total: BTreeMap<u32, u32> = BTreeMap::new();
    for week in weeks {
        if let Outcome::Derived(d) = derive_kinds(status, rules, week, 0) {
            for (bp, m) in d.minutes_at {
                *total.entry(bp).or_insert(0) += m;
            }
        }
    }
    Outcome::Derived(Derived { minutes_at: total })
}

/// Whether an exempt post's weekly salary is below the configured weekly floor: `Some(true)` below, `Some(false)` at or above, `None`
/// with no floor configured (not "fine"). Advisory; it never classifies a role from its duties (WPM-D157).
#[must_use]
pub fn exempt_below_floor(
    annual_salary_minor: i64,
    fte_percent: i32,
    floor_weekly_minor: Option<i64>,
) -> Option<bool> {
    let floor = floor_weekly_minor?;
    let weekly = i128::from(annual_salary_minor) * i128::from(fte_percent.clamp(0, 100)) / 100 / 52;
    Some(weekly < i128::from(floor))
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
    /// A rule this derivation does not do yet, with the reason; never read as no overtime.
    NotDerived(&'static str),
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

    /// The premium pay in minor units: the extra over the regular rate, where `rate_minor` is the base pay for `per_minutes` minutes
    /// (an hourly rate is 60; a monthly base is the minutes in a month), rounded half up once at the end. `None` when the
    /// arithmetic would overflow or `per_minutes` is not positive, never a wrong figure.
    #[must_use]
    pub fn premium_minor(&self, rate_minor: i64, per_minutes: i64) -> Option<i64> {
        if per_minutes <= 0 {
            return None;
        }
        let mut total: i128 = 0;
        for (bp, minutes) in &self.minutes_at {
            let extra = i128::from(*bp) - i128::from(REGULAR_BP);
            total += i128::from(*minutes) * i128::from(rate_minor) * extra;
        }
        let unit = i128::from(per_minutes) * i128::from(REGULAR_BP);
        let rounded = (total * 2 + unit * total.signum()) / (2 * unit);
        i64::try_from(rounded).ok()
    }
}

fn multiplier(rules: &Rules, day_pos: u32, week_pos: u32, month_pos: u32, kind: DayKind) -> u32 {
    let hits = rules
        .bands
        .iter()
        .filter(|b| {
            let pos = match b.period {
                Period::Day => day_pos,
                Period::Week => week_pos,
                Period::Month => month_pos,
            };
            pos > b.threshold_minutes
        })
        .map(|b| b.multiplier_bp)
        .chain(rules.kinds.bp(kind));
    match rules.stacking {
        Stacking::Highest => hits.max().unwrap_or(REGULAR_BP).max(REGULAR_BP),
        Stacking::Add => REGULAR_BP + hits.map(|bp| bp.saturating_sub(REGULAR_BP)).sum::<u32>(),
    }
}

/// Derive one week. `days` are the minutes worked each day, in order (a day is capped at 1440). `month_before_minutes` is what the
/// person had worked earlier in the same month before this week, for a monthly band. Every day is read as an ordinary one; see
/// [`derive_kinds`].
#[must_use]
pub fn derive(status: Status, rules: &Rules, days: &[u32], month_before_minutes: u32) -> Outcome {
    let days: Vec<(u32, DayKind)> = days.iter().map(|m| (*m, DayKind::Ordinary)).collect();
    derive_kinds(status, rules, &days, month_before_minutes)
}

/// [`derive`] with the kind of each day, so a rest day or a holiday is paid at its own multiplier where the rules set one.
#[must_use]
pub fn derive_kinds(
    status: Status,
    rules: &Rules,
    days: &[(u32, DayKind)],
    month_before_minutes: u32,
) -> Outcome {
    match status {
        Status::Exempt => return Outcome::Exempt,
        Status::Unknown => return Outcome::Unknown,
        Status::NonExempt => {}
    }
    if rules.bands.is_empty() && rules.kinds == Kinds::default() {
        return Outcome::NoLimitConfigured;
    }
    let mut minutes_at: BTreeMap<u32, u32> = BTreeMap::new();
    let mut week_pos: u32 = 0;
    for (worked, kind) in days {
        for day_pos in 1..=(*worked).min(DAY) {
            week_pos += 1;
            let bp = multiplier(
                rules,
                day_pos,
                week_pos,
                month_before_minutes + week_pos,
                *kind,
            );
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
            kinds: Kinds::default(),
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
            kinds: Kinds::default(),
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
                kinds: Kinds::default(),
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
                kinds: Kinds::default(),
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
            kinds: Kinds::default(),
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
            kinds: Kinds::default(),
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
            kinds: Kinds::default(),
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
    fn days_group_into_weeks_that_start_where_the_calendar_says() {
        let d = |day| NaiveDate::from_ymd_opt(2026, 11, day).unwrap();
        // 2026-11-01 is a Sunday. Monday weeks: {1} then {2..4}; Sunday weeks: {1..4}.
        let days = BTreeMap::from([(d(1), 60), (d(2), 120), (d(4), 180)]);
        assert_eq!(weeks(&days, 1), vec![vec![60], vec![120, 180]]);
        assert_eq!(weeks(&days, 7), vec![vec![60, 120, 180]]);
        assert_eq!(weeks(&days, 0), weeks(&days, 1), "a stray start is Monday");
    }

    #[test]
    fn a_period_adds_its_weeks_and_a_monthly_band_says_it_is_not_derived() {
        let two = vec![vec![9 * H; 5], vec![9 * H; 5]];
        let d = derived(derive_period(Status::NonExempt, &weekly40(), &two));
        assert_eq!(
            d.overtime_minutes(),
            10 * H,
            "five hours past forty in each week"
        );
        let monthly = Rules {
            bands: vec![band(Period::Month, 100, 15_000)],
            stacking: Stacking::Highest,
            kinds: Kinds::default(),
        };
        assert_eq!(
            derive_period(Status::NonExempt, &monthly, &two),
            Outcome::NotDerived("monthly_band")
        );
        assert_eq!(
            derive_period(Status::Exempt, &monthly, &two),
            Outcome::Exempt
        );
        assert_eq!(
            derive_period(Status::Unknown, &weekly40(), &two),
            Outcome::Unknown
        );
        let nothing = derived(derive_period(Status::NonExempt, &weekly40(), &[]));
        assert_eq!(nothing.overtime_minutes(), 0);
    }

    #[test]
    fn an_exempt_salary_is_compared_with_the_weekly_floor_and_no_floor_is_not_fine() {
        // 52,000.00 a year is 1,000.00 a week.
        assert_eq!(
            exempt_below_floor(5_200_000, 100, Some(100_000)),
            Some(false),
            "at the floor"
        );
        assert_eq!(
            exempt_below_floor(5_200_000, 100, Some(100_001)),
            Some(true)
        );
        assert_eq!(
            exempt_below_floor(5_200_000, 50, Some(100_000)),
            Some(true),
            "half time halves the week"
        );
        assert_eq!(exempt_below_floor(5_200_000, 100, None), None);
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
        assert_eq!(d.premium_minor(2_000, 60), Some(5_000));
        // One minute at 150% on 20.01 an hour: 2001 * 0.5 / 60 = 16.675 -> 17.
        let one = Derived {
            minutes_at: BTreeMap::from([(15_000, 1)]),
        };
        assert_eq!(one.premium_minor(2_001, 60), Some(17));
        let huge = Derived {
            minutes_at: BTreeMap::from([(15_000, u32::MAX)]),
        };
        assert_eq!(
            huge.premium_minor(i64::MAX, 60),
            None,
            "overflow is None, not a wrong figure"
        );
    }

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn a_rest_day_and_a_holiday_pay_their_own_multiplier_and_an_ordinary_day_is_untouched() {
        let rules = Rules {
            bands: vec![band(Period::Week, 40, 15_000)],
            stacking: Stacking::Highest,
            kinds: Kinds {
                rest_day_bp: Some(20_000),
                holiday_bp: Some(30_000),
            },
        };
        let days = [
            (8 * H, DayKind::Ordinary),
            (8 * H, DayKind::Rest),
            (8 * H, DayKind::Holiday),
        ];
        let d = derived(derive_kinds(Status::NonExempt, &rules, &days, 0));
        assert_eq!(d.minutes_at[&REGULAR_BP], 8 * H);
        assert_eq!(d.minutes_at[&20_000], 8 * H);
        assert_eq!(d.minutes_at[&30_000], 8 * H);
    }

    #[test]
    fn a_kind_with_no_multiplier_pays_as_an_ordinary_day() {
        let rules = Rules {
            bands: vec![band(Period::Day, 8, 15_000)],
            stacking: Stacking::Highest,
            kinds: Kinds {
                rest_day_bp: None,
                holiday_bp: Some(20_000),
            },
        };
        let d = derived(derive_kinds(
            Status::NonExempt,
            &rules,
            &[(9 * H, DayKind::Rest)],
            0,
        ));
        assert_eq!(d.minutes_at[&REGULAR_BP], 8 * H);
        assert_eq!(d.minutes_at[&15_000], H);
    }

    #[test]
    fn where_premiums_add_a_holiday_adds_to_the_long_day_and_otherwise_the_highest_wins() {
        let kinds = Kinds {
            rest_day_bp: None,
            holiday_bp: Some(20_000),
        };
        let add = Rules {
            bands: vec![band(Period::Day, 8, 15_000)],
            stacking: Stacking::Add,
            kinds,
        };
        let day = [(9 * H, DayKind::Holiday)];
        let got = derived(derive_kinds(Status::NonExempt, &add, &day, 0));
        assert_eq!(got.minutes_at[&20_000], 8 * H);
        assert_eq!(got.minutes_at[&25_000], H);
        let highest = Rules {
            stacking: Stacking::Highest,
            ..add
        };
        let got = derived(derive_kinds(Status::NonExempt, &highest, &day, 0));
        assert_eq!(got.minutes_at[&20_000], 9 * H);
    }

    #[test]
    fn a_holiday_multiplier_alone_is_a_rule_set_but_nothing_at_all_is_not() {
        let only = Rules {
            kinds: Kinds {
                rest_day_bp: None,
                holiday_bp: Some(20_000),
            },
            ..Rules::default()
        };
        assert!(matches!(
            derive_kinds(Status::NonExempt, &only, &[(H, DayKind::Holiday)], 0),
            Outcome::Derived(_)
        ));
        assert_eq!(
            derive_kinds(
                Status::NonExempt,
                &Rules::default(),
                &[(H, DayKind::Holiday)],
                0
            ),
            Outcome::NoLimitConfigured
        );
    }

    #[test]
    fn weeks_with_labels_each_day_by_its_date() {
        let mut days = BTreeMap::new();
        days.insert(d(2026, 11, 2), 60); // Monday
        days.insert(d(2026, 11, 7), 120); // Saturday
        let weeks = weeks_with(&days, 1, |date| {
            if date.weekday().number_from_monday() == 6 {
                DayKind::Rest
            } else {
                DayKind::Ordinary
            }
        });
        assert_eq!(
            weeks,
            vec![vec![(60, DayKind::Ordinary), (120, DayKind::Rest)]]
        );
    }
}
