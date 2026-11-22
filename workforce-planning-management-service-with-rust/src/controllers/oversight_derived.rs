//! The data side of the derived oversight measures (WPM-R67, WPM-T114): load what a derivation needs
//! for an organization and hand it to the pure rules in [`crate::rules::derived`], one reading per
//! month asked for. Reads only; nothing is stored, so a derived reading is never stale.

use crate::sql_time::SqlDate;
use axa::prelude::*;
use jiff::civil::Date;

use super::capacity;
use crate::db::pool;
use crate::rules::derived as rules;
use crate::rules::equality_monitoring::DEFAULT_FLOOR;
use crate::rules::funding::SplitRow;
use crate::rules::metrics::{Tenure, headcount_on, leavers};
use crate::rules::oversight::{Origin, Reading};

/// The first day of the month a period number stands for.
fn month_of(period: i64) -> Option<Date> {
    let year = i32::try_from(period.div_euclid(12)).ok()?;
    // The month of the year, 1 to 12: the position of the period in its year, counted from one.
    let month = (1..=12_u32).nth(usize::try_from(period.rem_euclid(12)).ok()?)?;
    crate::dates::ymd(year, month, 1)
}

fn last_day(first: Date) -> Option<Date> {
    first
        .checked_add(jiff::Span::new().months(1))
        .ok()
        .and_then(|d| d.yesterday().ok())
}

/// One derived reading per period in `periods`, for the derivation `key`. A month the data cannot
/// speak for is a reading with no value, never a zero.
///
/// # Errors
///
/// A database error.
pub(super) async fn readings(
    ctx: &AppContext,
    org: &str,
    key: &str,
    periods: &[i64],
) -> Result<Vec<Reading>> {
    let values = match key {
        "turnover_rate" => turnover(ctx, org, periods).await?,
        "contingent_share" => contingent(ctx, org, periods).await?,
        "overcommitted_pools" => overcommitted(ctx, org, periods).await?,
        "time_to_report" => time_to_report(ctx, org, periods).await?,
        _ => vec![None; periods.len()],
    };
    Ok(periods
        .iter()
        .zip(values)
        .map(|(period, value)| Reading {
            period: *period,
            value,
            origin: Origin::Derived,
        })
        .collect())
}

async fn turnover(ctx: &AppContext, org: &str, periods: &[i64]) -> Result<Vec<Option<i64>>> {
    let tenures: Vec<Tenure> = sqlx::query!(
        "SELECT hired_on, terminated_on FROM workers WHERE organization_ref = $1 AND deleted_at IS NULL",
        org,
    )
    .fetch_all(pool(ctx))
    .await?
    .into_iter()
    .map(|w| Tenure {
        hired_on: w.hired_on.get(),
        terminated_on: w.terminated_on.map(SqlDate::get),
    })
    .collect();
    Ok(periods
        .iter()
        .map(|p| {
            let first = month_of(*p)?;
            let last = last_day(first)?;
            let opening = headcount_on(first.yesterday().ok()?, &tenures);
            let closing = headcount_on(last, &tenures);
            rules::turnover_bp(
                leavers(first, last, &tenures),
                opening,
                closing,
                DEFAULT_FLOOR,
            )
        })
        .collect())
}

async fn contingent(ctx: &AppContext, org: &str, periods: &[i64]) -> Result<Vec<Option<i64>>> {
    let rows: Vec<SplitRow> = sqlx::query!(
        "SELECT as_of, headcount, fte_percent_total, contractor_fte_percent_total, fixed_term_fte_percent_total
         FROM headcount_snapshots WHERE organization_ref = $1 ORDER BY as_of",
        org,
    )
    .fetch_all(pool(ctx))
    .await?
    .into_iter()
    .map(|r| SplitRow {
        as_of: r.as_of.get(),
        headcount: i64::from(r.headcount),
        fte: r.fte_percent_total,
        contractor_fte: r.contractor_fte_percent_total,
        fixed_term_fte: r.fixed_term_fte_percent_total,
    })
    .collect();
    Ok(periods
        .iter()
        .map(|p| {
            let first = month_of(*p)?;
            let last = last_day(first)?;
            let newest = rows
                .iter()
                .map(|r| r.as_of)
                .filter(|d| *d >= first && *d <= last)
                .max()?;
            let day: Vec<SplitRow> = rows.iter().filter(|r| r.as_of == newest).copied().collect();
            rules::contingent_bp(&day, DEFAULT_FLOOR)
        })
        .collect())
}

/// How many months the periods run across, first to last inclusive; `None` when there are no periods or the count does not fit.
fn months_spanned(periods: &[i64]) -> Option<u32> {
    let (min, max) = (periods.iter().min()?, periods.iter().max()?);
    u32::try_from(max.checked_sub(*min)?.checked_add(1)?).ok()
}

async fn overcommitted(ctx: &AppContext, org: &str, periods: &[i64]) -> Result<Vec<Option<i64>>> {
    let Some(first) = periods.iter().min().and_then(|p| month_of(*p)) else {
        return Ok(Vec::new());
    };
    // Months past the supported horizon, or too many to count, are unknown, not guessed.
    let Some(span) =
        months_spanned(periods).filter(|s| *s <= crate::rules::capacity::MAX_HORIZON_MONTHS)
    else {
        return Ok(vec![None; periods.len()]);
    };
    let cap = capacity::load_org_capacity(ctx, org, first, span).await?;
    Ok(periods
        .iter()
        .map(|p| {
            let month = month_of(*p)?;
            let at = cap.months.iter().position(|m| *m == month)?;
            let cells: Vec<_> = cap
                .views
                .values()
                .filter_map(|v| v.cells.get(at).map(|c| c.status))
                .collect();
            rules::over_committed(&cells)
        })
        .collect())
}

async fn time_to_report(ctx: &AppContext, org: &str, periods: &[i64]) -> Result<Vec<Option<i64>>> {
    let rows: Vec<(Date, Option<Date>)> = sqlx::query!(
        "SELECT first_true_on, acknowledged_on FROM early_warnings WHERE organization_ref = $1",
        org,
    )
    .fetch_all(pool(ctx))
    .await?
    .into_iter()
    .map(|w| (w.first_true_on.get(), w.acknowledged_on.map(SqlDate::get)))
    .collect();
    Ok(periods
        .iter()
        .map(|p| {
            let first = month_of(*p)?;
            let last = last_day(first)?;
            let in_month: Vec<_> = rows
                .iter()
                .filter(|(f, _)| *f >= first && *f <= last)
                .copied()
                .collect();
            crate::rules::early_warning::time_to_report_days(&in_month)
        })
        .collect())
}

#[cfg(test)]
mod span_tests {
    use super::{month_of, months_spanned};

    #[test]
    fn months_run_first_to_last_inclusive_and_an_unreachable_count_is_none() {
        assert_eq!(months_spanned(&[]), None);
        assert_eq!(months_spanned(&[24_312]), Some(1));
        assert_eq!(months_spanned(&[24_314, 24_312, 24_313]), Some(3));
        assert_eq!(months_spanned(&[i64::MIN, i64::MAX]), None);
        assert_eq!(months_spanned(&[0, i64::from(u32::MAX)]), None);
    }

    #[test]
    fn a_period_names_the_first_of_its_month() {
        assert_eq!(
            month_of(24_312).map(|d| d.to_string()).as_deref(),
            Some("2026-01-01")
        );
        assert_eq!(
            month_of(24_323).map(|d| d.to_string()).as_deref(),
            Some("2026-12-01")
        );
        assert_eq!(month_of(i64::MAX), None);
    }
}
