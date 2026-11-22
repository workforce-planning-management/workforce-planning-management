//! Pure rules for the **workforce metrics** layer — one named, shared
//! definition for each standard count and rate (headcount, starters,
//! leavers, turnover, span of control), so "headcount" means the same
//! thing wherever it is shown.
//!
//! DB-free and clock-free (dates are always supplied). Nothing is
//! imputed: a rate with nothing to divide by is `None`, never `0`.

use jiff::civil::Date;

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
        "Whole calendar days from a requisition's opened_on to its filled_on, over requisitions filled \
         within the period. Requisitions lacking either date are left out, not guessed.",
    ),
];

/// A worker's employment dates, the only inputs the headcount metrics
/// need.
#[derive(Debug, Clone, Copy)]
pub struct Tenure {
    /// Hire date.
    pub hired_on: Date,
    /// Termination date, when there is one.
    pub terminated_on: Option<Date>,
}

/// Whether a worker is employed on `date`: hired on or before it, and
/// not terminated on or before it. **The** definition of "employed" —
/// every headcount-style figure in the service goes through it.
#[must_use]
pub fn is_employed_on(date: Date, hired_on: Date, terminated_on: Option<Date>) -> bool {
    hired_on <= date && terminated_on.is_none_or(|t| t > date)
}

/// Headcount on `date`: workers [`is_employed_on`] it.
#[must_use]
pub fn headcount_on(date: Date, workers: &[Tenure]) -> usize {
    workers
        .iter()
        .filter(|w| is_employed_on(date, w.hired_on, w.terminated_on))
        .count()
}

/// Workers hired within `[from, to]` (inclusive).
#[must_use]
pub fn starters(from: Date, to: Date, workers: &[Tenure]) -> usize {
    workers
        .iter()
        .filter(|w| w.hired_on >= from && w.hired_on <= to)
        .count()
}

/// Workers terminated within `[from, to]` (inclusive).
#[must_use]
pub fn leavers(from: Date, to: Date, workers: &[Tenure]) -> usize {
    workers
        .iter()
        .filter(|w| w.terminated_on.is_some_and(|t| t >= from && t <= to))
        .count()
}

/// Turnover rate: `leavers / mean(opening, closing)`; `None` when the
/// mean headcount is zero.
#[must_use]
pub fn turnover_rate(leavers: usize, opening: usize, closing: usize) -> Option<f64> {
    let mean_twice = opening.checked_add(closing)?;
    if mean_twice == 0 {
        return None;
    }
    Some(count_f64(leavers)? * 2.0 / count_f64(mean_twice)?)
}

/// The largest magnitude `f64` holds exactly for every integer up to it, 2^53 (RCA-CONV-4).
const MAX_EXACT_F64: u64 = 1 << 53;

/// An integer as an `f64` exactly, or `None` when its magnitude is past 2^53 and a float would round it (RCA-CONV-4).
/// A display figure that cannot be shown exactly is unknown, not rounded.
#[must_use]
pub(crate) fn exact_f64(n: i64) -> Option<f64> {
    let magnitude = n.unsigned_abs();
    if magnitude > MAX_EXACT_F64 {
        return None;
    }
    // Two 32-bit halves: each converts with `f64::from`, and `high * 2^32 + low` is exact below 2^53.
    let high = f64::from(u32::try_from(magnitude >> 32_i32).ok()?);
    let low = f64::from(u32::try_from(magnitude & u64::from(u32::MAX)).ok()?);
    let value = high * 4_294_967_296.0_f64 + low;
    Some(if n < 0 { -value } else { value })
}

/// `x` rounded to the nearest whole number as an `i64`, or `None` when it is not finite or outside the `i64` range (RCA-CONV-3).
#[must_use]
pub fn rounded_i64(x: f64) -> Option<i64> {
    // 2^63 is exactly representable as an `f64`; `i64::MAX` is not.
    const TWO_POW_63: f64 = 4_294_967_296.0_f64 * 2_147_483_648.0_f64;
    let whole = x.round();
    if !(whole.is_finite() && (-TWO_POW_63..TWO_POW_63).contains(&whole)) {
        return None;
    }
    #[expect(
        clippy::as_conversions,
        clippy::cast_possible_truncation,
        reason = "RCA-CONV-3: whole, finite and within the i64 range, checked above"
    )]
    let n = whole as i64;
    Some(n)
}

/// A count as an `f64` exactly, or `None` past 2^53 (RCA-CONV-4).
#[must_use]
pub(crate) fn count_f64(n: usize) -> Option<f64> {
    exact_f64(i64::try_from(n).ok()?)
}

/// What a snapshot needs to know about one worker.
#[derive(Debug, Clone)]
pub struct WorkerFacts {
    /// Owning organization URN.
    pub organization_ref: String,
    /// Department.
    pub department: String,
    /// Declared FTE, in percent (100 = one full-time person).
    pub fte_percent: i32,
    /// The engagement basis (`employment_type`).
    pub employment_type: String,
    /// Hire date.
    pub hired_on: Date,
    /// Termination date, when there is one.
    pub terminated_on: Option<Date>,
}

/// One organization × department row of a headcount snapshot.
#[derive(Debug, PartialEq, Eq)]
pub struct SnapshotRow {
    /// Owning organization URN.
    pub organization_ref: String,
    /// Department.
    pub department: String,
    /// Workers employed on the snapshot date.
    pub headcount: usize,
    /// Sum of declared `fte_percent` over those workers (hundredths of a
    /// person), so no float rounds a headcount.
    pub fte_percent_total: i64,
    /// Contractors among them (WPM-R84).
    pub contractor_headcount: usize,
    /// Their FTE, in hundredths of a person.
    pub contractor_fte_percent_total: i64,
    /// Fixed-term people among them.
    pub fixed_term_headcount: usize,
    /// Their FTE, in hundredths of a person.
    pub fixed_term_fte_percent_total: i64,
    /// Hires in `(since, as_of]`; `None` when there is no previous
    /// snapshot to measure from (unknown, not zero).
    pub starters: Option<usize>,
    /// Terminations in `(since, as_of]`; `None` likewise.
    pub leavers: Option<usize>,
}

/// Build the snapshot rows for `as_of`: one per organization × department
/// that has either employed workers or leavers in the window, sorted by
/// organization then department. `since` is the previous snapshot date
/// (exclusive); with `None`, starters and leavers are unknown. `None` when
/// a count or total does not fit its type (unknown, not a clamped figure).
#[must_use]
pub fn snapshot_rows(
    as_of: Date,
    since: Option<Date>,
    workers: &[WorkerFacts],
) -> Option<Vec<SnapshotRow>> {
    use std::collections::BTreeMap;
    let in_window = |d: Date| since.is_some_and(|s| d > s && d <= as_of);
    let mut rows: BTreeMap<(String, String), SnapshotRow> = BTreeMap::new();
    for w in workers {
        let key = (w.organization_ref.clone(), w.department.clone());
        let row = rows.entry(key).or_insert_with(|| SnapshotRow {
            organization_ref: w.organization_ref.clone(),
            department: w.department.clone(),
            headcount: 0,
            fte_percent_total: 0,
            contractor_headcount: 0,
            contractor_fte_percent_total: 0,
            fixed_term_headcount: 0,
            fixed_term_fte_percent_total: 0,
            starters: since.map(|_| 0),
            leavers: since.map(|_| 0),
        });
        if is_employed_on(as_of, w.hired_on, w.terminated_on) {
            let fte = i64::from(w.fte_percent);
            row.headcount = row.headcount.checked_add(1)?;
            row.fte_percent_total = row.fte_percent_total.checked_add(fte)?;
            match w.employment_type.as_str() {
                "contractor" => {
                    row.contractor_headcount = row.contractor_headcount.checked_add(1)?;
                    row.contractor_fte_percent_total =
                        row.contractor_fte_percent_total.checked_add(fte)?;
                }
                "fixed_term" => {
                    row.fixed_term_headcount = row.fixed_term_headcount.checked_add(1)?;
                    row.fixed_term_fte_percent_total =
                        row.fixed_term_fte_percent_total.checked_add(fte)?;
                }
                _ => {}
            }
        }
        if in_window(w.hired_on) {
            row.starters = match row.starters {
                Some(n) => Some(n.checked_add(1)?),
                None => None,
            };
        }
        if w.terminated_on.is_some_and(in_window) {
            row.leavers = match row.leavers {
                Some(n) => Some(n.checked_add(1)?),
                None => None,
            };
        }
    }
    Some(
        rows.into_values()
            .filter(|r| r.headcount > 0 || r.leavers.is_some_and(|n| n > 0))
            .collect(),
    )
}

/// Days a requisition took to fill: `filled_on - opened_on`. `None` when
/// either date is missing or the fill date precedes the opening (bad
/// data is left out, not reported as a negative duration).
#[must_use]
pub fn time_to_fill_days(opened_on: Option<Date>, filled_on: Option<Date>) -> Option<i64> {
    let days = crate::dates::days_between(opened_on?, filled_on?);
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
    // A total past i64, or a figure past 2^53, is unknown, not a rounded mean.
    let total = sorted
        .iter()
        .try_fold(0_i64, |acc, d| acc.checked_add(*d))?;
    let mean = exact_f64(total)? / count_f64(n)?;
    let median = if n % 2 == 1 {
        exact_f64(*sorted.get(n.div_euclid(2))?)?
    } else {
        // The middle two: the one `n.div_euclid(2)` from the end, and the one `n.div_euclid(2)` from the start.
        let lo = sorted.iter().rev().nth(n.div_euclid(2))?;
        f64::midpoint(exact_f64(*lo)?, exact_f64(*sorted.get(n.div_euclid(2))?)?)
    };
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
    let total = direct_reports
        .iter()
        .try_fold(0_usize, |acc, n| acc.checked_add(*n))?;
    let mean = count_f64(total)? / count_f64(managers)?;
    Some(Span {
        managers,
        mean,
        max: direct_reports.iter().copied().max().unwrap_or(0),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `snapshot_rows`, which holds for every workers these tests build.
    fn snapshot_rows_ok(
        as_of: Date,
        since: Option<Date>,
        workers: &[WorkerFacts],
    ) -> Vec<SnapshotRow> {
        snapshot_rows(as_of, since, workers).expect("totals that fit")
    }

    fn day(y: i32, m: u32, d: u32) -> Date {
        crate::dates::ymd(y, m, d).expect("valid date")
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

    fn facts(org: &str, dept: &str, fte: i32, hired: Date, left: Option<Date>) -> WorkerFacts {
        WorkerFacts {
            organization_ref: org.into(),
            department: dept.into(),
            fte_percent: fte,
            employment_type: "permanent".into(),
            hired_on: hired,
            terminated_on: left,
        }
    }

    /// Rows group by organization × department, count only the employed,
    /// sum FTE in hundredths, and measure the window `(since, as_of]`.
    #[test]
    fn snapshot_rows_group_and_measure() {
        let ws = vec![
            facts("o1", "eng", 100, day(2020, 1, 1), None),
            facts("o1", "eng", 50, day(2026, 6, 10), None),
            facts("o1", "eng", 100, day(2020, 1, 1), Some(day(2026, 6, 20))),
            facts("o1", "ops", 100, day(2019, 1, 1), None),
            facts("o2", "eng", 100, day(2021, 1, 1), None),
            facts("o1", "gone", 100, day(2018, 1, 1), Some(day(2026, 6, 15))),
        ];
        let rows = snapshot_rows_ok(day(2026, 7, 1), Some(day(2026, 6, 1)), &ws);
        let eng = &rows[0];
        assert_eq!(
            (eng.organization_ref.as_str(), eng.department.as_str()),
            ("o1", "eng")
        );
        assert_eq!(eng.headcount, 2, "the leaver is not employed on as_of");
        assert_eq!(eng.fte_percent_total, 150);
        assert_eq!(eng.starters, Some(1));
        assert_eq!(eng.leavers, Some(1));
        let names: Vec<_> = rows
            .iter()
            .map(|r| (r.organization_ref.as_str(), r.department.as_str()))
            .collect();
        assert_eq!(
            names,
            [("o1", "eng"), ("o1", "gone"), ("o1", "ops"), ("o2", "eng")],
            "a department that only lost people still appears"
        );
        assert_eq!(rows[1].headcount, 0);
        assert_eq!(rows[1].leavers, Some(1));
    }

    /// Without a previous snapshot, starters and leavers are unknown —
    /// `None`, not zero — and an empty department is omitted.
    #[test]
    fn first_snapshot_has_unknown_flows() {
        let ws = vec![
            facts("o1", "eng", 100, day(2020, 1, 1), None),
            facts("o1", "gone", 100, day(2018, 1, 1), Some(day(2019, 1, 1))),
        ];
        let rows = snapshot_rows_ok(day(2026, 7, 1), None, &ws);
        assert_eq!(
            rows.len(),
            1,
            "no employed workers and no known leavers ⇒ no row"
        );
        assert_eq!(rows[0].starters, None);
        assert_eq!(rows[0].leavers, None);
        assert_eq!(snapshot_rows_ok(day(2026, 7, 1), None, &[]), vec![]);
    }

    /// The snapshot splits headcount and FTE by basis; a person not employed on the date is in neither.
    #[test]
    fn snapshot_rows_split_by_basis() {
        let mut contractor = facts("o1", "eng", 50, day(2020, 1, 1), None);
        contractor.employment_type = "contractor".into();
        let mut fixed = facts("o1", "eng", 80, day(2020, 1, 1), None);
        fixed.employment_type = "fixed_term".into();
        let mut gone = facts("o1", "eng", 100, day(2020, 1, 1), Some(day(2026, 1, 1)));
        gone.employment_type = "contractor".into();
        let rows = snapshot_rows_ok(
            day(2026, 7, 1),
            None,
            &[
                facts("o1", "eng", 100, day(2020, 1, 1), None),
                contractor,
                fixed,
                gone,
            ],
        );
        let r = &rows[0];
        assert_eq!((r.headcount, r.fte_percent_total), (3, 230));
        assert_eq!(
            (r.contractor_headcount, r.contractor_fte_percent_total),
            (1, 50)
        );
        assert_eq!(
            (r.fixed_term_headcount, r.fixed_term_fte_percent_total),
            (1, 80)
        );
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

    #[test]
    fn turnover_over_headcounts_that_do_not_add_up_is_absent() {
        assert_eq!(turnover_rate(1, usize::MAX, 1), None);
    }
}
