//! Pure rules for working-time guardrails (WPM-R27): the average of recorded minutes a week over a
//! trailing reference window, and the rest gap between shift assignments. Advisory only — these derive
//! flags, they gate nothing (WPM-D19). No I/O, no clock; arithmetic never panics.
//!
//! The limits are **jurisdiction parameters** (WPM-D144, WPM-T380), not constants: the window in weeks,
//! the ceiling in minutes a week and the rest floor in minutes come from the worker's profile through
//! `rules::jurisdiction::resolve`. With none configured a check is "no limit configured", never a pass.

use chrono::{DateTime, Utc};

/// The registry key of the reference window, in weeks of seven calendar days.
pub const REFERENCE_WEEKS_KEY: &str = "working_time.reference_weeks";
/// The registry key of the ceiling on average minutes a week.
pub const MAX_AVERAGE_WEEKLY_MINUTES_KEY: &str = "working_time.max_average_weekly_minutes";
/// The registry key of the least rest between shifts, in minutes.
pub const MIN_DAILY_REST_MINUTES_KEY: &str = "working_time.min_daily_rest_minutes";

/// The least recorded minutes in one rollup week that a worker over `ceiling` (minutes a week, averaged
/// over `weeks`) must have in at least one week. A trailing window of `weeks` × 7 days touches at most
/// `weeks + 1` rollup weeks whatever day the rollup's week starts on, so a total over the ceiling has a
/// week of at least this many: the rollup is only a superset filter, and its week start cannot change an
/// answer. `None` when either limit is not positive.
#[must_use]
pub fn heavy_week_minutes(ceiling: i64, weeks: i64) -> Option<i64> {
    (ceiling > 0 && weeks > 0).then(|| ceiling.saturating_mul(weeks) / (weeks + 1))
}

/// Average weekly minutes over `weeks`, or `None` when `weeks` is not
/// positive (a zero-width window has no average — `null`, never `0`).
#[must_use]
#[allow(clippy::cast_precision_loss)] // display average
pub fn weekly_average(total_minutes: i64, weeks: i64) -> Option<f64> {
    (weeks > 0).then(|| total_minutes as f64 / weeks as f64)
}

/// Whether the average weekly minutes over `weeks` exceed `ceiling` (minutes a week).
/// Integer comparison (`total > ceiling × weeks`) — no float edge.
#[must_use]
pub fn over_average(total_minutes: i64, weeks: i64, ceiling: i64) -> bool {
    weeks > 0 && total_minutes > ceiling.saturating_mul(weeks)
}

/// One rest-gap breach between two consecutive shift assignments.
#[derive(Debug, PartialEq, Eq)]
pub struct RestBreach {
    /// When the earlier shift ends.
    pub prev_end: DateTime<Utc>,
    /// When the next shift starts.
    pub next_start: DateTime<Utc>,
    /// The gap between them, in minutes (clamped at 0 for overlaps).
    pub gap_minutes: i64,
}

/// The rest-gap breaches in a set of shift `intervals`
/// (`(starts, ends)`; any order): sorted by start, each consecutive
/// pair with less than `min_rest` minutes between the earlier
/// end and the later start is a breach. Malformed intervals
/// (`end <= start`) are skipped rather than trusted.
#[must_use]
pub fn rest_breaches(
    intervals: &[(DateTime<Utc>, DateTime<Utc>)],
    min_rest: i64,
) -> Vec<RestBreach> {
    let mut sorted: Vec<&(DateTime<Utc>, DateTime<Utc>)> = intervals
        .iter()
        .filter(|(start, end)| end > start)
        .collect();
    sorted.sort_by_key(|(start, _)| *start);
    sorted
        .windows(2)
        .filter_map(|pair| {
            let (_, prev_end) = *pair[0];
            let (next_start, _) = *pair[1];
            let gap_minutes = (next_start - prev_end).num_minutes().max(0);
            (gap_minutes < min_rest).then_some(RestBreach {
                prev_end,
                next_start,
                gap_minutes,
            })
        })
        .collect()
}

/// One shift's `(starts, ends)`.
pub type Shift = (DateTime<Utc>, DateTime<Utc>);

/// Rest breaches per worker from `(worker, (starts, ends))` assignments, each against that worker's own
/// floor (`rest_of`); a worker with no floor configured (`None`) or no breach is left out.
#[must_use]
pub fn breaches_by_worker<K: Ord + Copy>(
    assignments: &[(K, Shift)],
    rest_of: impl Fn(K) -> Option<i64>,
) -> std::collections::BTreeMap<K, Vec<RestBreach>> {
    let mut intervals: std::collections::BTreeMap<K, Vec<Shift>> =
        std::collections::BTreeMap::new();
    for (worker, interval) in assignments {
        intervals.entry(*worker).or_default().push(*interval);
    }
    intervals
        .into_iter()
        .filter_map(|(worker, list)| {
            rest_of(worker).map(|rest| (worker, rest_breaches(&list, rest)))
        })
        .filter(|(_, breaches)| !breaches.is_empty())
        .collect()
}

/// Whether a worker is flagged at all: over their configured average (`over`), or with a rest breach.
#[must_use]
pub const fn is_flagged(over: bool, rest_breach_count: usize) -> bool {
    over || rest_breach_count > 0
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The example limits the tests use (the shape of a 17-week, 48-hour, 11-hour profile) and a second,
    /// differently shaped one (4 weeks, 40 hours, 12 hours).
    const EXAMPLE: (i64, i64, i64) = (17, 48 * 60, 11 * 60);
    const SECOND: (i64, i64, i64) = (4, 40 * 60, 12 * 60);

    /// Intervals are grouped per worker, so one worker's shifts never breach another's rest; a worker
    /// with no floor configured is not checked.
    #[test]
    fn breaches_are_found_within_a_worker_not_across_workers() {
        let assignments = [
            (1_u8, (at(1, 8), at(1, 20))),
            (2, (at(1, 23), at(1, 23) + chrono::Duration::hours(8))),
            (1, (at(2, 5), at(2, 13))),
            (3, (at(1, 8), at(1, 16))),
            (3, (at(3, 8), at(3, 16))),
            (4, (at(1, 8), at(1, 20))),
            (4, (at(2, 5), at(2, 13))),
        ];
        let found = breaches_by_worker(&assignments, |w| (w != 4).then_some(EXAMPLE.2));
        assert_eq!(
            found.keys().copied().collect::<Vec<_>>(),
            [1],
            "4 has no floor configured"
        );
        assert_eq!(found[&1][0].gap_minutes, 9 * 60);
        assert!(breaches_by_worker::<u8>(&[], |_| Some(1)).is_empty());
        // A higher floor elsewhere: worker 3's 40-hour gap still passes, 1's 9 hours does not.
        let strict = breaches_by_worker(&assignments, |_| Some(SECOND.2));
        assert!(strict.contains_key(&1) && !strict.contains_key(&3));
    }

    /// Exactly the ceiling on average is not over; a rest breach alone flags.
    #[test]
    fn a_worker_is_flagged_for_hours_or_for_rest() {
        let (weeks, ceiling, _) = EXAMPLE;
        assert!(!is_flagged(
            over_average(ceiling * weeks, weeks, ceiling),
            0
        ));
        assert!(is_flagged(
            over_average(ceiling * weeks + 1, weeks, ceiling),
            0
        ));
        assert!(is_flagged(false, 1));
    }

    /// The heavy-week floor: for the example profile 2,720 minutes (the old fixed index); for any
    /// profile, a total over the ceiling spread over `weeks + 1` rollup weeks has a week at the floor,
    /// and the floor never exceeds what a total at the ceiling allows.
    #[test]
    fn the_heavy_week_floor_follows_the_profile() {
        assert_eq!(heavy_week_minutes(EXAMPLE.1, EXAMPLE.0), Some(2_720));
        assert_eq!(heavy_week_minutes(SECOND.1, SECOND.0), Some(1_920));
        for (weeks, ceiling) in [(1, 1), (4, 2_400), (17, 2_880), (26, 2_700), (52, 10_080)] {
            let heavy = heavy_week_minutes(ceiling, weeks).unwrap();
            let rollup_weeks = weeks + 1;
            assert!((ceiling * weeks + 1 + rollup_weeks - 1) / rollup_weeks >= heavy);
            assert!(heavy * rollup_weeks <= ceiling * weeks);
        }
        assert_eq!(heavy_week_minutes(0, 17), None);
        assert_eq!(heavy_week_minutes(2_880, 0), None);
    }

    /// A trailing window of whole weeks touches at most `weeks + 1` rollup weeks whichever weekday the
    /// rollup's week starts on, so the rollup's own week start cannot change an answer.
    #[test]
    fn the_window_touches_at_most_weeks_plus_one_rollup_weeks_for_any_week_start() {
        use chrono::Datelike;
        for weeks in [1_i64, 4, 17] {
            for end_offset in 0..7 {
                let end = chrono::NaiveDate::from_ymd_opt(2026, 7, 10).unwrap()
                    + chrono::Duration::days(end_offset);
                let start = end - chrono::Duration::weeks(weeks) + chrono::Duration::days(1);
                for week_start in 0..7_u32 {
                    let bucket = |d: chrono::NaiveDate| {
                        let back = (d.weekday().num_days_from_monday() + 7 - week_start) % 7;
                        d - chrono::Duration::days(i64::from(back))
                    };
                    let touched: std::collections::BTreeSet<_> = start
                        .iter_days()
                        .take_while(|d| *d <= end)
                        .map(bucket)
                        .collect();
                    assert!(i64::try_from(touched.len()).unwrap() <= weeks + 1);
                }
            }
        }
    }

    fn at(day: u32, hour: u32) -> DateTime<Utc> {
        chrono::NaiveDate::from_ymd_opt(2026, 7, day)
            .unwrap()
            .and_hms_opt(hour, 0, 0)
            .unwrap()
            .and_utc()
    }

    /// The average carries its terms: a zero-week window is `None`
    /// (null, never 0), and the value is minutes per week.
    #[test]
    fn weekly_average_terms() {
        assert_eq!(weekly_average(1700, 17), Some(100.0));
        assert_eq!(weekly_average(0, 17), Some(0.0));
        assert_eq!(weekly_average(1700, 0), None, "no window, no average");
    }

    /// The flag is an integer comparison at the exact boundary, for each profile's own limits.
    #[test]
    fn over_average_boundary() {
        for (weeks, ceiling, _) in [EXAMPLE, SECOND] {
            assert!(
                !over_average(ceiling * weeks, weeks, ceiling),
                "exactly the ceiling is not over"
            );
            assert!(over_average(ceiling * weeks + 1, weeks, ceiling));
        }
        assert!(!over_average(i64::MAX, 0, 2_880), "no window, no flag");
        // The same total is over one profile and not the other.
        let total = 4 * 45 * 60;
        assert!(over_average(total, SECOND.0, SECOND.1));
        assert!(!over_average(total, 4, EXAMPLE.1));
    }

    /// The rest-gap check: under the floor is a breach, exactly the floor is not, unordered input is
    /// sorted first, an overlap clamps to 0, and a malformed interval is skipped.
    #[test]
    fn rest_breaches_matrix() {
        let rest = EXAMPLE.2;
        // 22:00 end (day 1) → 08:00 start (day 2) = 10 h ⇒ breach.
        let tight = vec![(at(1, 14), at(1, 22)), (at(2, 8), at(2, 16))];
        let breaches = rest_breaches(&tight, rest);
        assert_eq!(breaches.len(), 1);
        assert_eq!(breaches[0].gap_minutes, 600);
        // 22:00 → 09:00 = 11 h exactly ⇒ no breach (floor is inclusive), but under a 12-hour floor it is.
        let ok = vec![(at(1, 14), at(1, 22)), (at(2, 9), at(2, 17))];
        assert_eq!(rest_breaches(&ok, rest).len(), 0);
        assert_eq!(rest_breaches(&ok, SECOND.2).len(), 1);
        let unordered = vec![(at(2, 8), at(2, 16)), (at(1, 14), at(1, 22))];
        assert_eq!(rest_breaches(&unordered, rest).len(), 1);
        let overlap = vec![(at(1, 8), at(1, 18)), (at(1, 16), at(1, 23))];
        assert_eq!(rest_breaches(&overlap, rest)[0].gap_minutes, 0);
        let malformed = vec![(at(1, 22), at(1, 14)), (at(2, 8), at(2, 16))];
        assert_eq!(rest_breaches(&malformed, rest).len(), 0);
    }

    /// The keys are registry keys.
    #[test]
    fn the_keys_are_in_the_registry() {
        for key in [
            REFERENCE_WEEKS_KEY,
            MAX_AVERAGE_WEEKLY_MINUTES_KEY,
            MIN_DAILY_REST_MINUTES_KEY,
        ] {
            assert!(
                crate::rules::jurisdiction::parameter(key).is_some(),
                "{key}"
            );
        }
    }
}
