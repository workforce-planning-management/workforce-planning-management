//! Pure rules for the **early warning** (WPM-R68, WPM-T115): which conditions are newly true, which
//! have stopped, and the time to report (the calendar days from a condition first being true to a
//! person acknowledging it). DB-free and clock-free.

use std::collections::BTreeSet;

use chrono::NaiveDate;
use uuid::Uuid;

/// The kinds of condition.
pub const KINDS: &[&str] = &["pool_over_committed", "measure_breach"];

/// How many months ahead a pool is looked at, counting the current month. A pool short in any of
/// them is over-committed.
pub const HORIZON_MONTHS: u32 = 6;

/// A condition: its kind and the pool or measure it is about.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Key {
    /// One of [`KINDS`].
    pub kind: String,
    /// The pool or measure.
    pub subject: Uuid,
}

/// What a sweep changes.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Plan {
    /// True now and not yet recorded as open: a new warning, dated today.
    pub opened: Vec<Key>,
    /// Recorded as open and no longer true: stopped today.
    pub cleared: Vec<Key>,
}

/// Compare what is recorded as open with what is true now. A condition that is still true stays as
/// it is, so its first date never moves; one that stops and returns is a new warning.
#[must_use]
pub fn reconcile(open: &[Key], now_true: &BTreeSet<Key>) -> Plan {
    let recorded: BTreeSet<&Key> = open.iter().collect();
    Plan {
        opened: now_true
            .iter()
            .filter(|k| !recorded.contains(k))
            .cloned()
            .collect(),
        cleared: open
            .iter()
            .filter(|k| !now_true.contains(k))
            .cloned()
            .collect(),
    }
}

/// Calendar days from first true to acknowledged; `None` while unacknowledged.
#[must_use]
pub fn days_to_acknowledge(
    first_true_on: NaiveDate,
    acknowledged_on: Option<NaiveDate>,
) -> Option<i64> {
    acknowledged_on.map(|a| (a - first_true_on).num_days())
}

/// The time to report for a set of warnings, as the **longest** calendar days to acknowledge (the
/// worst case, which a quick one cannot hide). `None` for no warnings, and `None` when any is still
/// unacknowledged: the figure is not known yet, and a slow one must not read as a fast one.
#[must_use]
pub fn time_to_report_days(rows: &[(NaiveDate, Option<NaiveDate>)]) -> Option<i64> {
    let mut longest: Option<i64> = None;
    for (first, ack) in rows {
        let days = days_to_acknowledge(*first, *ack)?;
        longest = Some(longest.map_or(days, |l| l.max(days)));
    }
    longest
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(kind: &str, n: u128) -> Key {
        Key {
            kind: kind.into(),
            subject: Uuid::from_u128(n),
        }
    }
    fn day(d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 3, d).unwrap()
    }

    #[test]
    fn a_new_condition_opens_and_a_stopped_one_clears() {
        let open = [key("pool_over_committed", 1), key("measure_breach", 2)];
        let now: BTreeSet<Key> = [key("pool_over_committed", 1), key("measure_breach", 3)].into();
        let plan = reconcile(&open, &now);
        assert_eq!(plan.opened, [key("measure_breach", 3)]);
        assert_eq!(plan.cleared, [key("measure_breach", 2)]);
    }

    #[test]
    fn a_condition_still_true_changes_nothing() {
        let open = [key("pool_over_committed", 1)];
        let now: BTreeSet<Key> = [key("pool_over_committed", 1)].into();
        assert_eq!(reconcile(&open, &now), Plan::default());
    }

    #[test]
    fn the_same_subject_under_another_kind_is_another_condition() {
        let open = [key("pool_over_committed", 1)];
        let now: BTreeSet<Key> = [key("measure_breach", 1)].into();
        let plan = reconcile(&open, &now);
        assert_eq!((plan.opened.len(), plan.cleared.len()), (1, 1));
    }

    #[test]
    fn days_to_acknowledge_is_calendar_days_and_unknown_until_acknowledged() {
        assert_eq!(days_to_acknowledge(day(1), Some(day(1))), Some(0));
        assert_eq!(days_to_acknowledge(day(1), Some(day(8))), Some(7));
        assert_eq!(days_to_acknowledge(day(1), None), None);
    }

    #[test]
    fn time_to_report_is_the_longest_and_unknown_while_any_is_open() {
        assert_eq!(time_to_report_days(&[]), None);
        let done = [(day(1), Some(day(2))), (day(3), Some(day(13)))];
        assert_eq!(time_to_report_days(&done), Some(10));
        let one_open = [(day(1), Some(day(2))), (day(3), None)];
        assert_eq!(time_to_report_days(&one_open), None);
    }
}
