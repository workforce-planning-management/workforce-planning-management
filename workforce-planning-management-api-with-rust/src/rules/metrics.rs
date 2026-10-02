//! Pure rules for the **workforce metrics** layer — one named, shared
//! definition for each standard count and rate (headcount, starters,
//! leavers, turnover, span of control), so "headcount" means the same
//! thing wherever it is shown.
//!
//! DB-free and clock-free (dates are always supplied). Nothing is
//! imputed: a rate with nothing to divide by is `None`, never `0`.

use chrono::NaiveDate;

/// The shared vocabulary: `(metric, definition)`. The metrics endpoint
/// returns this verbatim so a consumer reads the definition beside the
/// number.
pub const DEFINITIONS: &[(&str, &str)] = &[
    (
        "headcount",
        "Workers employed on the date: hired on or before it, and not terminated on or before \
         it. Soft-deleted records are excluded.",
    ),
    (
        "starters",
        "Workers whose hire date falls within the period (inclusive).",
    ),
    (
        "leavers",
        "Workers whose termination date falls within the period (inclusive).",
    ),
    (
        "turnover_rate",
        "Leavers divided by the mean of opening and closing headcount. Absent when the mean \
         is zero.",
    ),
    (
        "span_of_control",
        "Direct reports per manager, over managers with at least one report.",
    ),
    (
        "time_to_fill",
        "Whole days from a requisition's opened_on to its filled_on, over requisitions filled \
         within the period. Requisitions lacking either date are left out, not guessed.",
    ),
];

/// A worker's employment dates, the only inputs the headcount metrics
/// need.
#[derive(Debug, Clone, Copy)]
pub struct Tenure {
    /// Hire date.
    pub hired_on: NaiveDate,
    /// Termination date, when there is one.
    pub terminated_on: Option<NaiveDate>,
}

/// Whether a worker is employed on `date`: hired on or before it, and
/// not terminated on or before it. **The** definition of "employed" —
/// every headcount-style figure in the service goes through it.
#[must_use]
pub fn is_employed_on(
    date: NaiveDate,
    hired_on: NaiveDate,
    terminated_on: Option<NaiveDate>,
) -> bool {
    hired_on <= date && terminated_on.is_none_or(|t| t > date)
}

/// Headcount on `date`: workers [`is_employed_on`] it.
#[must_use]
pub fn headcount_on(date: NaiveDate, workers: &[Tenure]) -> usize {
    workers
        .iter()
        .filter(|w| is_employed_on(date, w.hired_on, w.terminated_on))
        .count()
}

/// Workers hired within `[from, to]` (inclusive).
#[must_use]
pub fn starters(from: NaiveDate, to: NaiveDate, workers: &[Tenure]) -> usize {
    workers
        .iter()
        .filter(|w| w.hired_on >= from && w.hired_on <= to)
        .count()
}

/// Workers terminated within `[from, to]` (inclusive).
#[must_use]
pub fn leavers(from: NaiveDate, to: NaiveDate, workers: &[Tenure]) -> usize {
    workers
        .iter()
        .filter(|w| w.terminated_on.is_some_and(|t| t >= from && t <= to))
        .count()
}

/// Turnover rate: `leavers / mean(opening, closing)`; `None` when the
/// mean headcount is zero.
#[must_use]
pub fn turnover_rate(leavers: usize, opening: usize, closing: usize) -> Option<f64> {
    let mean_twice = opening + closing;
    if mean_twice == 0 {
        return None;
    }
    #[allow(clippy::cast_precision_loss)] // display ratio over small counts
    Some(leavers as f64 * 2.0 / mean_twice as f64)
}

/// Days a requisition took to fill: `filled_on - opened_on`. `None` when
/// either date is missing or the fill date precedes the opening (bad
/// data is left out, not reported as a negative duration).
#[must_use]
pub fn time_to_fill_days(
    opened_on: Option<NaiveDate>,
    filled_on: Option<NaiveDate>,
) -> Option<i64> {
    let days = (filled_on? - opened_on?).num_days();
    (days >= 0).then_some(days)
}

/// Summary of time-to-fill durations (days).
#[derive(Debug, PartialEq)]
pub struct FillTime {
    /// Requisitions with a usable duration.
    pub count: usize,
    /// Mean days.
    pub mean: f64,
    /// Median days (mean of the middle two for an even count).
    pub median: f64,
}

/// Summarise durations; `None` when there are none.
#[must_use]
pub fn fill_time_summary(days: &[i64]) -> Option<FillTime> {
    if days.is_empty() {
        return None;
    }
    let mut sorted = days.to_vec();
    sorted.sort_unstable();
    let n = sorted.len();
    #[allow(clippy::cast_precision_loss)] // display figures over small counts
    let (mean, median) = (
        sorted.iter().sum::<i64>() as f64 / n as f64,
        if n % 2 == 1 {
            sorted[n / 2] as f64
        } else {
            (sorted[n / 2 - 1] + sorted[n / 2]) as f64 / 2.0
        },
    );
    Some(FillTime {
        count: n,
        mean,
        median,
    })
}

/// Span-of-control summary over per-manager direct-report counts.
#[derive(Debug, PartialEq)]
pub struct Span {
    /// Managers with at least one report.
    pub managers: usize,
    /// Mean direct reports per such manager.
    pub mean: f64,
    /// The largest team.
    pub max: usize,
}

/// Summarise direct-report counts; `None` when nobody manages anyone.
#[must_use]
pub fn span_of_control(direct_reports: &[usize]) -> Option<Span> {
    let managers = direct_reports.iter().filter(|n| **n > 0).count();
    if managers == 0 {
        return None;
    }
    let total: usize = direct_reports.iter().sum();
    #[allow(clippy::cast_precision_loss)] // display mean over small counts
    let mean = total as f64 / managers as f64;
    Some(Span {
        managers,
        mean,
        max: direct_reports.iter().copied().max().unwrap_or(0),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).expect("valid date")
    }

    fn staff() -> Vec<Tenure> {
        vec![
            Tenure {
                hired_on: day(2020, 1, 1),
                terminated_on: None,
            },
            Tenure {
                hired_on: day(2020, 1, 1),
                terminated_on: Some(day(2026, 3, 1)),
            },
            Tenure {
                hired_on: day(2026, 2, 1),
                terminated_on: None,
            },
            Tenure {
                hired_on: day(2027, 1, 1),
                terminated_on: None,
            },
        ]
    }

    /// Headcount counts from the hire date and stops on the termination
    /// date (the leaving day is no longer employed); future hires are not
    /// counted.
    #[test]
    fn headcount_respects_both_dates() {
        let s = staff();
        assert_eq!(headcount_on(day(2019, 12, 31), &s), 0);
        assert_eq!(headcount_on(day(2026, 1, 1), &s), 2);
        assert_eq!(
            headcount_on(day(2026, 2, 1), &s),
            3,
            "starter counted from the hire date"
        );
        assert_eq!(
            headcount_on(day(2026, 3, 1), &s),
            2,
            "leaver gone on the termination date"
        );
        assert_eq!(
            headcount_on(day(2026, 12, 31), &s),
            2,
            "a 2027 hire is not yet counted"
        );
    }

    /// Period counts are inclusive at both ends.
    #[test]
    fn starters_and_leavers_are_inclusive() {
        let s = staff();
        assert_eq!(starters(day(2026, 2, 1), day(2026, 2, 1), &s), 1);
        assert_eq!(starters(day(2026, 1, 1), day(2026, 12, 31), &s), 1);
        assert_eq!(leavers(day(2026, 3, 1), day(2026, 3, 1), &s), 1);
        assert_eq!(leavers(day(2026, 3, 2), day(2026, 12, 31), &s), 0);
    }

    /// Turnover divides by the mean headcount; nothing to divide ⇒ `None`
    /// (not 0); a real zero stays 0.
    #[test]
    fn turnover_is_none_not_zero_without_a_base() {
        assert_eq!(turnover_rate(0, 0, 0), None);
        assert_eq!(turnover_rate(0, 10, 10), Some(0.0));
        let rate = turnover_rate(2, 8, 12).expect("rate");
        assert!((rate - 0.2).abs() < 1e-9, "2 leavers / mean 10");
    }

    /// Span of control ignores people who manage nobody.
    #[test]
    fn span_of_control_summary() {
        assert_eq!(span_of_control(&[]), None);
        assert_eq!(span_of_control(&[0, 0]), None);
        let span = span_of_control(&[0, 2, 4]).expect("span");
        assert_eq!(span.managers, 2);
        assert!((span.mean - 3.0).abs() < 1e-9);
        assert_eq!(span.max, 4);
    }

    /// Time-to-fill needs both dates, in order; summaries handle odd and
    /// even counts and an empty set.
    #[test]
    fn time_to_fill() {
        let d = day(2026, 3, 1);
        assert_eq!(time_to_fill_days(Some(day(2026, 1, 1)), Some(d)), Some(59));
        assert_eq!(time_to_fill_days(None, Some(d)), None);
        assert_eq!(time_to_fill_days(Some(d), None), None);
        assert_eq!(
            time_to_fill_days(Some(d), Some(day(2026, 1, 1))),
            None,
            "filled before opened"
        );
        assert_eq!(
            time_to_fill_days(Some(d), Some(d)),
            Some(0),
            "same-day fill is a real 0"
        );
        assert_eq!(fill_time_summary(&[]), None);
        let odd = fill_time_summary(&[30, 10, 20]).expect("odd");
        assert_eq!((odd.count, odd.median), (3, 20.0));
        assert!((odd.mean - 20.0).abs() < 1e-9);
        let even = fill_time_summary(&[10, 20, 30, 50]).expect("even");
        assert!((even.median - 25.0).abs() < 1e-9);
    }

    /// Every metric the endpoint reports has a definition.
    #[test]
    fn definitions_cover_the_reported_metrics() {
        for name in [
            "headcount",
            "starters",
            "leavers",
            "turnover_rate",
            "span_of_control",
            "time_to_fill",
        ] {
            assert!(
                DEFINITIONS.iter().any(|(n, _)| *n == name),
                "{name} undefined"
            );
        }
    }
}
