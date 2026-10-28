//! Annual service across all roles (WPM-R166, WPM-D113). DB-free and clock-free: the as-of date is an argument.
//!
//! Service is the **union** of a person's employment periods, so an overlap counts once. A gap no longer than a configured
//! tolerance does not break **continuous** service; no tolerance is shipped (employers and agreements differ), so with none set
//! only periods that touch or overlap join. A period with no recorded start, or an end before its start, makes the answer
//! `None` (unknown, never zero) and is named. Recognised prior service is counted only when recorded and is shown apart.
//! Dates are inclusive at both ends.

use chrono::{Days, Months, NaiveDate};

/// One employment period, as recorded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Period {
    /// How the period is named when it is unknown (a reference, never an employer or role).
    pub reference: String,
    /// The first day, where recorded.
    pub starts_on: Option<NaiveDate>,
    /// The last day, or none while the period continues.
    pub ends_on: Option<NaiveDate>,
}

/// A period that is known and clipped to the as-of date.
type Span = (NaiveDate, NaiveDate);

/// The answer for one person.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Service {
    /// Days in the union of the periods, or none when any period is unknown.
    pub total_days: Option<i64>,
    /// Whole years of the total, counted from the earliest start; none when unknown.
    pub total_years: Option<i32>,
    /// Days of continuous service up to the as-of date (0 when the latest chain ended before it); none when unknown.
    pub continuous_days: Option<i64>,
    /// Whole years of continuous service, from the chain's own start; none when unknown.
    pub continuous_years: Option<i32>,
    /// The periods the answer rests on, merged and clipped, oldest first.
    pub spans: Vec<(NaiveDate, NaiveDate)>,
    /// The references of the periods that make the answer unknown.
    pub unknown: Vec<String>,
}

fn days(span: Span) -> i64 {
    (span.1 - span.0).num_days() + 1
}

/// Whole years completed by service that began on `start` and has run for `total_days` inclusive days.
fn whole_years(start: NaiveDate, total_days: i64) -> i32 {
    let mut n = 0;
    while let Some(next) = start.checked_add_months(Months::new(12 * (n + 1))) {
        if total_days >= (next - start).num_days() {
            n += 1;
        } else {
            break;
        }
    }
    i32::try_from(n).unwrap_or(i32::MAX)
}

/// Union of spans with gaps up to `tolerance` days joined. A gap is the days not worked between two spans.
fn merge(mut spans: Vec<Span>, tolerance: i64) -> Vec<Span> {
    spans.sort();
    let mut out: Vec<Span> = Vec::new();
    for s in spans {
        match out.last_mut() {
            Some(last) if (s.0 - last.1).num_days() - 1 <= tolerance => last.1 = last.1.max(s.1),
            _ => out.push(s),
        }
    }
    out
}

/// Service on `as_of`. `tolerance_days` is the configured break that does not end continuous service (none: no break allowed).
#[must_use]
pub fn service(periods: &[Period], tolerance_days: Option<u32>, as_of: NaiveDate) -> Service {
    let mut unknown = Vec::new();
    let mut spans: Vec<Span> = Vec::new();
    for p in periods {
        match p.starts_on {
            None => unknown.push(p.reference.clone()),
            Some(s) if p.ends_on.is_some_and(|e| e < s) => unknown.push(p.reference.clone()),
            Some(s) if s > as_of => {}
            Some(s) => spans.push((s, p.ends_on.map_or(as_of, |e| e.min(as_of)))),
        }
    }
    let union = merge(spans.clone(), 0);
    if !unknown.is_empty() {
        return Service {
            total_days: None,
            total_years: None,
            continuous_days: None,
            continuous_years: None,
            spans: union,
            unknown,
        };
    }
    let total: i64 = union.iter().copied().map(days).sum();
    let total_years = union.first().map(|f| whole_years(f.0, total));
    let chains = merge(spans, i64::from(tolerance_days.unwrap_or(0)));
    let (continuous_days, continuous_years) = match chains.last() {
        Some(c) if c.1 >= as_of => (days(*c), whole_years(c.0, days(*c))),
        _ => (0, 0),
    };
    Service {
        total_days: Some(total),
        total_years: Some(total_years.unwrap_or(0)),
        continuous_days: Some(continuous_days),
        continuous_years: Some(continuous_years),
        spans: union,
        unknown,
    }
}

/// Days of recognised prior service, shown apart and never added to [`Service`]. Only periods HR recorded are passed in; one
/// with no end runs to `as_of`, and overlaps count once.
#[must_use]
pub fn prior_days(recorded: &[(NaiveDate, Option<NaiveDate>)], as_of: NaiveDate) -> i64 {
    let spans = recorded
        .iter()
        .filter(|(s, _)| *s <= as_of)
        .filter(|(s, e)| e.is_none_or(|e| e >= *s))
        .map(|(s, e)| (*s, e.map_or(as_of, |e| e.min(as_of))))
        .collect();
    merge(spans, 0).into_iter().map(days).sum()
}

/// The day a period of `n` years of service is reached from `start`: the anniversary, one day before the date n years on.
#[must_use]
pub fn anniversary(start: NaiveDate, years: u32) -> Option<NaiveDate> {
    start
        .checked_add_months(Months::new(12 * years))
        .and_then(|d| d.checked_sub_days(Days::new(1)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    fn p(r: &str, from: Option<NaiveDate>, to: Option<NaiveDate>) -> Period {
        Period {
            reference: r.to_string(),
            starts_on: from,
            ends_on: to,
        }
    }

    const NOW: fn() -> NaiveDate = || d(2026, 10, 28);

    #[test]
    fn one_period_counts_inclusive_days_and_whole_years() {
        let s = service(
            &[p("a", Some(d(2024, 3, 1)), Some(d(2025, 2, 28)))],
            None,
            NOW(),
        );
        assert_eq!(s.total_days, Some(365));
        assert_eq!(
            s.total_years,
            Some(1),
            "a full year ends the day before the anniversary"
        );
        let short = service(
            &[p("a", Some(d(2024, 3, 1)), Some(d(2025, 2, 27)))],
            None,
            NOW(),
        );
        assert_eq!(short.total_years, Some(0));
    }

    #[test]
    fn a_current_period_runs_to_the_as_of_date() {
        let s = service(&[p("a", Some(d(2026, 10, 1)), None)], None, NOW());
        assert_eq!(s.total_days, Some(28));
        assert_eq!(s.continuous_days, Some(28));
    }

    #[test]
    fn an_overlap_counts_once() {
        let s = service(
            &[
                p("a", Some(d(2020, 1, 1)), Some(d(2020, 12, 31))),
                p("b", Some(d(2020, 7, 1)), Some(d(2021, 6, 30))),
            ],
            None,
            NOW(),
        );
        assert_eq!(
            s.total_days,
            Some(366 + 181),
            "2020 is a leap year; Jan 2020 to Jun 2021 is 547 days"
        );
        assert_eq!(s.spans, vec![(d(2020, 1, 1), d(2021, 6, 30))]);
    }

    #[test]
    fn a_gap_within_the_tolerance_does_not_break_continuous_service() {
        let list = [
            p("a", Some(d(2020, 1, 1)), Some(d(2020, 12, 31))),
            p("b", Some(d(2021, 1, 8)), None),
        ];
        let gap = 7;
        let with = service(&list, Some(gap), NOW());
        let without = service(&list, Some(gap - 1), NOW());
        let none = service(&list, None, NOW());
        assert_eq!(
            with.continuous_days,
            Some((NOW() - d(2020, 1, 1)).num_days() + 1)
        );
        assert_eq!(
            without.continuous_days,
            Some((NOW() - d(2021, 1, 8)).num_days() + 1)
        );
        assert_eq!(
            none.continuous_days, without.continuous_days,
            "no tolerance shipped, none assumed"
        );
        assert_eq!(
            with.total_days, without.total_days,
            "the gap is never counted as service"
        );
    }

    #[test]
    fn adjacent_periods_join_with_no_tolerance() {
        let s = service(
            &[
                p("a", Some(d(2020, 1, 1)), Some(d(2020, 12, 31))),
                p("b", Some(d(2021, 1, 1)), None),
            ],
            None,
            NOW(),
        );
        assert_eq!(s.continuous_days, s.total_days);
    }

    #[test]
    fn someone_who_has_left_has_no_continuous_service_to_the_as_of_date() {
        let s = service(
            &[p("a", Some(d(2020, 1, 1)), Some(d(2022, 1, 1)))],
            Some(30),
            NOW(),
        );
        assert_eq!(s.continuous_days, Some(0));
        assert!(s.total_days.unwrap() > 0);
    }

    #[test]
    fn a_period_with_no_start_makes_the_answer_unknown_and_is_named() {
        let s = service(
            &[
                p("known", Some(d(2020, 1, 1)), None),
                p("old-job", None, Some(d(2019, 1, 1))),
            ],
            None,
            NOW(),
        );
        assert_eq!(s.total_days, None);
        assert_eq!(s.continuous_days, None);
        assert_eq!(s.total_years, None);
        assert_eq!(s.unknown, vec!["old-job".to_string()]);
    }

    #[test]
    fn an_end_before_its_start_is_unknown_not_negative() {
        let s = service(
            &[p("x", Some(d(2021, 1, 1)), Some(d(2020, 1, 1)))],
            None,
            NOW(),
        );
        assert_eq!(s.total_days, None);
        assert_eq!(s.unknown, vec!["x".to_string()]);
    }

    #[test]
    fn a_period_not_yet_started_counts_nothing_and_no_period_is_zero() {
        let s = service(&[p("future", Some(d(2027, 1, 1)), None)], None, NOW());
        assert_eq!(s.total_days, Some(0));
        assert_eq!(service(&[], None, NOW()).total_days, Some(0));
    }

    #[test]
    fn whole_years_run_from_the_earliest_start_over_all_periods() {
        let s = service(
            &[
                p("a", Some(d(2020, 1, 1)), Some(d(2020, 12, 31))),
                p("b", Some(d(2022, 1, 1)), Some(d(2022, 12, 31))),
            ],
            None,
            NOW(),
        );
        assert_eq!(s.total_days, Some(366 + 365));
        assert_eq!(s.total_years, Some(2));
    }

    #[test]
    fn prior_service_is_its_own_number() {
        let rec = [
            (d(2010, 1, 1), Some(d(2010, 12, 31))),
            (d(2010, 6, 1), Some(d(2011, 5, 31))),
        ];
        assert_eq!(prior_days(&rec, NOW()), 365 + 151);
        assert_eq!(prior_days(&[], NOW()), 0);
        assert_eq!(prior_days(&[(d(2030, 1, 1), None)], NOW()), 0);
    }

    #[test]
    fn the_anniversary_is_the_day_before() {
        assert_eq!(anniversary(d(2024, 3, 1), 1), Some(d(2025, 2, 28)));
        assert_eq!(anniversary(d(2024, 3, 1), 0), Some(d(2024, 2, 29)));
    }
}
