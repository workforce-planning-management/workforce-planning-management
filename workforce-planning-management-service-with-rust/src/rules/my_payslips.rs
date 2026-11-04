//! **My payslips** (WPM-R178, WPM-T324): which payslips a person sees, the year to date per tax year, and the rules of a
//! payslip query. Pure: no database and no clock.
//!
//! A person sees payslips of `approved` and `paid` runs only. The year to date is derived from those payslips and never
//! stored. A tax year starts on a configured month and day (`payroll.tax_year_start`); there is no default, because a
//! guessed start gives wrong totals that look right. Each tax year and currency has its own totals: an amount is never
//! added across currencies. A query flags a payslip for payroll; it changes no amount.

use std::collections::BTreeMap;

use chrono::{Datelike, NaiveDate};
use serde::Serialize;

use super::payroll::Deduction;

/// The run statuses a person's payslip is visible in. A draft or calculated run can still change.
pub const VISIBLE_RUN_STATUSES: [&str; 2] = ["approved", "paid"];

/// The longest payslip query comment, in characters.
pub const MAX_COMMENT_CHARS: usize = 500;

/// The month and day a tax year starts, read from `payroll.tax_year_start` written as `MMDD` (`406` is 6 April, `101` is
/// 1 January). `None` for anything that is not a date in every year, which includes 29 February.
#[must_use]
pub fn tax_year_start(mmdd: i64) -> Option<(u32, u32)> {
    let month = u32::try_from(mmdd / 100).ok()?;
    let day = u32::try_from(mmdd % 100).ok()?;
    // 2027 is not a leap year, so 29 February is refused.
    NaiveDate::from_ymd_opt(2027, month, day).map(|_| (month, day))
}

/// The first and last days of the tax year holding `date`, for a tax year starting on `start` (month, day). `None` if
/// `start` is not a date, or at the limits of the calendar.
#[must_use]
pub fn tax_year_of(date: NaiveDate, start: (u32, u32)) -> Option<(NaiveDate, NaiveDate)> {
    let (month, day) = start;
    let this_year = NaiveDate::from_ymd_opt(date.year(), month, day)?;
    let first = if date >= this_year {
        this_year
    } else {
        NaiveDate::from_ymd_opt(date.year() - 1, month, day)?
    };
    let next = NaiveDate::from_ymd_opt(first.year() + 1, month, day)?;
    Some((first, next.pred_opt()?))
}

/// What the year to date reads from one payslip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Slip {
    /// The last day of the run's period: this decides the tax year.
    pub period_end: NaiveDate,
    /// The payslip's currency.
    pub currency: String,
    /// Gross pay in minor units.
    pub gross_minor: i64,
    /// The deduction lines.
    pub deductions: Vec<Deduction>,
    /// Net pay in minor units.
    pub net_minor: i64,
}

/// The totals of one tax year in one currency.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct YearToDate {
    /// The first day of the tax year.
    pub starts_on: NaiveDate,
    /// The last day of the tax year.
    pub ends_on: NaiveDate,
    /// The currency of every amount here.
    pub currency: String,
    /// How many payslips the totals rest on.
    pub payslips: u32,
    /// Gross pay.
    pub gross_minor: i64,
    /// The total of each kind of deduction, sorted by label.
    pub deductions: Vec<Deduction>,
    /// Net pay.
    pub net_minor: i64,
}

/// The year to date per tax year and currency, newest tax year first, then by currency. An error if the start is not a
/// date, or a total does not fit: a wrong total is worse than none.
///
/// # Errors
///
/// A message if `start` is not a date, or if a total overflows.
pub fn year_to_date(slips: &[Slip], start: (u32, u32)) -> Result<Vec<YearToDate>, String> {
    #[derive(Default)]
    struct Totals {
        payslips: u32,
        gross: i64,
        net: i64,
        deductions: BTreeMap<String, i64>,
    }
    let overflow = || "a year-to-date total does not fit".to_string();
    let mut years: BTreeMap<(NaiveDate, NaiveDate, String), Totals> = BTreeMap::new();
    for slip in slips {
        let (first, last) =
            tax_year_of(slip.period_end, start).ok_or("the tax year start is not a date")?;
        let totals = years
            .entry((first, last, slip.currency.clone()))
            .or_default();
        totals.payslips = totals.payslips.checked_add(1).ok_or_else(overflow)?;
        totals.gross = totals
            .gross
            .checked_add(slip.gross_minor)
            .ok_or_else(overflow)?;
        totals.net = totals
            .net
            .checked_add(slip.net_minor)
            .ok_or_else(overflow)?;
        for line in &slip.deductions {
            let kind = totals.deductions.entry(line.label.clone()).or_insert(0);
            *kind = kind.checked_add(line.amount_minor).ok_or_else(overflow)?;
        }
    }
    let mut out: Vec<YearToDate> = years
        .into_iter()
        .map(|((starts_on, ends_on, currency), totals)| YearToDate {
            starts_on,
            ends_on,
            currency,
            payslips: totals.payslips,
            gross_minor: totals.gross,
            deductions: totals
                .deductions
                .into_iter()
                .map(|(label, amount_minor)| Deduction {
                    label,
                    amount_minor,
                })
                .collect(),
            net_minor: totals.net,
        })
        .collect();
    out.sort_by(|a, b| {
        b.starts_on
            .cmp(&a.starts_on)
            .then_with(|| a.currency.cmp(&b.currency))
    });
    Ok(out)
}

/// Whether a person may see a payslip of a run in `status`.
#[must_use]
pub fn visible(status: &str) -> bool {
    VISIBLE_RUN_STATUSES.contains(&status)
}

/// The comment of a query, trimmed: required, and at most [`MAX_COMMENT_CHARS`] characters.
///
/// # Errors
///
/// A message if the comment is empty or too long.
pub fn clean_comment(comment: &str) -> Result<String, String> {
    let trimmed = comment.trim();
    if trimmed.is_empty() {
        return Err("say what the query is about".to_string());
    }
    if trimmed.chars().count() > MAX_COMMENT_CHARS {
        return Err(format!(
            "a query comment is at most {MAX_COMMENT_CHARS} characters"
        ));
    }
    Ok(trimmed.to_string())
}

/// Whether a person may raise a query on a payslip that may already have an open one.
///
/// # Errors
///
/// A message if a query is already open.
pub fn may_raise(open_already: bool) -> Result<(), String> {
    if open_already {
        return Err("this payslip already has an open query".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).expect("a date")
    }

    fn slip(end: NaiveDate, currency: &str, gross: i64, lines: &[(&str, i64)], net: i64) -> Slip {
        Slip {
            period_end: end,
            currency: currency.to_string(),
            gross_minor: gross,
            deductions: lines
                .iter()
                .map(|(label, amount)| Deduction {
                    label: (*label).to_string(),
                    amount_minor: *amount,
                })
                .collect(),
            net_minor: net,
        }
    }

    #[test]
    fn a_start_is_a_month_and_day_of_every_year() {
        assert_eq!(tax_year_start(406), Some((4, 6)));
        assert_eq!(tax_year_start(101), Some((1, 1)));
        assert_eq!(tax_year_start(1231), Some((12, 31)));
        for bad in [0, 100, 132, 229, 431, 1301, -406] {
            assert_eq!(tax_year_start(bad), None, "{bad}");
        }
    }

    #[test]
    fn the_first_and_last_days_of_a_tax_year_fall_in_it() {
        let start = (4, 6);
        let y2026 = (day(2026, 4, 6), day(2027, 4, 5));
        assert_eq!(tax_year_of(day(2026, 4, 6), start), Some(y2026));
        assert_eq!(tax_year_of(day(2027, 4, 5), start), Some(y2026));
        assert_eq!(
            tax_year_of(day(2026, 4, 5), start),
            Some((day(2025, 4, 6), day(2026, 4, 5)))
        );
    }

    #[test]
    fn a_calendar_tax_year_ends_on_31_december() {
        assert_eq!(
            tax_year_of(day(2026, 6, 30), (1, 1)),
            Some((day(2026, 1, 1), day(2026, 12, 31)))
        );
    }

    #[test]
    fn a_leap_day_belongs_to_its_tax_year() {
        assert_eq!(
            tax_year_of(day(2028, 2, 29), (4, 6)),
            Some((day(2027, 4, 6), day(2028, 4, 5)))
        );
    }

    #[test]
    fn a_tax_year_start_that_is_not_a_date_gives_no_year() {
        assert_eq!(tax_year_of(day(2026, 6, 30), (2, 30)), None);
        assert!(year_to_date(&[slip(day(2026, 6, 30), "GBP", 1, &[], 1)], (2, 30)).is_err());
    }

    #[test]
    fn no_payslips_means_no_totals_and_not_totals_of_zero() {
        assert_eq!(year_to_date(&[], (4, 6)), Ok(vec![]));
    }

    #[test]
    fn totals_add_each_kind_of_deduction_within_a_tax_year() {
        let slips = [
            slip(
                day(2026, 5, 31),
                "GBP",
                300_000,
                &[("tax", 50_000), ("pension", 15_000)],
                235_000,
            ),
            slip(
                day(2026, 6, 30),
                "GBP",
                300_000,
                &[("tax", 50_000)],
                250_000,
            ),
        ];
        let years = year_to_date(&slips, (4, 6)).expect("totals");
        assert_eq!(years.len(), 1);
        let year = &years[0];
        assert_eq!(
            (year.payslips, year.gross_minor, year.net_minor),
            (2, 600_000, 485_000)
        );
        assert_eq!(
            year.deductions,
            vec![
                Deduction {
                    label: "pension".to_string(),
                    amount_minor: 15_000,
                },
                Deduction {
                    label: "tax".to_string(),
                    amount_minor: 100_000,
                },
            ]
        );
    }

    #[test]
    fn a_tax_year_boundary_divides_the_totals_and_the_newest_comes_first() {
        let slips = [
            slip(day(2026, 3, 31), "GBP", 100, &[], 100),
            slip(day(2026, 4, 30), "GBP", 200, &[], 200),
        ];
        let years = year_to_date(&slips, (4, 6)).expect("totals");
        assert_eq!(
            years
                .iter()
                .map(|y| (y.starts_on, y.gross_minor))
                .collect::<Vec<_>>(),
            vec![(day(2026, 4, 6), 200), (day(2025, 4, 6), 100)]
        );
    }

    #[test]
    fn the_period_end_decides_the_tax_year_of_a_period_across_the_boundary() {
        let slips = [slip(day(2026, 4, 30), "GBP", 100, &[], 100)];
        let years = year_to_date(&slips, (4, 6)).expect("totals");
        assert_eq!(years[0].starts_on, day(2026, 4, 6));
    }

    #[test]
    fn amounts_are_never_added_across_currencies() {
        let slips = [
            slip(day(2026, 6, 30), "GBP", 100, &[], 100),
            slip(day(2026, 6, 30), "EUR", 200, &[], 200),
        ];
        let years = year_to_date(&slips, (1, 1)).expect("totals");
        assert_eq!(
            years
                .iter()
                .map(|y| (y.currency.as_str(), y.gross_minor))
                .collect::<Vec<_>>(),
            vec![("EUR", 200), ("GBP", 100)]
        );
    }

    #[test]
    fn a_total_that_does_not_fit_is_refused() {
        let slips = [
            slip(day(2026, 6, 30), "GBP", i64::MAX, &[], 0),
            slip(day(2026, 6, 30), "GBP", 1, &[], 0),
        ];
        assert!(year_to_date(&slips, (1, 1)).is_err());
    }

    #[test]
    fn only_approved_and_paid_runs_are_visible() {
        assert!(visible("approved"));
        assert!(visible("paid"));
        assert!(!visible("draft"));
        assert!(!visible("calculated"));
        assert!(!visible(""));
    }

    #[test]
    fn a_comment_is_trimmed_required_and_limited() {
        assert_eq!(
            clean_comment("  The overtime is missing \n"),
            Ok("The overtime is missing".to_string())
        );
        assert!(clean_comment("   ").is_err());
        assert!(clean_comment(&"x".repeat(MAX_COMMENT_CHARS)).is_ok());
        assert!(clean_comment(&"x".repeat(MAX_COMMENT_CHARS + 1)).is_err());
        // Counted in characters, so a comment in Mandarin has the same room as one in English.
        assert!(clean_comment(&"字".repeat(MAX_COMMENT_CHARS)).is_ok());
    }

    #[test]
    fn one_query_is_open_at_a_time() {
        assert_eq!(may_raise(false), Ok(()));
        assert!(may_raise(true).is_err());
    }
}
