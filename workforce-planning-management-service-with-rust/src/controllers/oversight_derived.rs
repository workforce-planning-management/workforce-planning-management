//! The data side of the derived oversight measures (WPM-R67, WPM-T114): load what a derivation needs
//! for an organization and hand it to the pure rules in [`crate::rules::derived`], one reading per
//! month asked for. Reads only; nothing is stored, so a derived reading is never stale.

use chrono::NaiveDate;
use loco_rs::prelude::*;
use sea_orm::QueryOrder;

use super::capacity;
use crate::models::_entities::{early_warnings, headcount_snapshots, workers};
use crate::rules::derived as rules;
use crate::rules::equality_monitoring::DEFAULT_FLOOR;
use crate::rules::funding::SplitRow;
use crate::rules::metrics::{Tenure, headcount_on, leavers};
use crate::rules::oversight::{Origin, Reading};

/// The first day of the month a period number stands for.
fn month_of(period: i64) -> Option<NaiveDate> {
    let year = i32::try_from(period.div_euclid(12)).ok()?;
    let month = u32::try_from(period.rem_euclid(12) + 1).ok()?;
    NaiveDate::from_ymd_opt(year, month, 1)
}

fn last_day(first: NaiveDate) -> Option<NaiveDate> {
    first
        .checked_add_months(chrono::Months::new(1))
        .and_then(|d| d.pred_opt())
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
    let tenures: Vec<Tenure> = workers::Entity::find()
        .filter(workers::Column::OrganizationRef.eq(org))
        .filter(workers::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|w| Tenure {
            hired_on: w.hired_on,
            terminated_on: w.terminated_on,
        })
        .collect();
    Ok(periods
        .iter()
        .map(|p| {
            let first = month_of(*p)?;
            let last = last_day(first)?;
            let opening = headcount_on(first.pred_opt()?, &tenures);
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
    let rows: Vec<SplitRow> = headcount_snapshots::Entity::find()
        .filter(headcount_snapshots::Column::OrganizationRef.eq(org))
        .order_by_asc(headcount_snapshots::Column::AsOf)
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|r| SplitRow {
            as_of: r.as_of,
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

async fn overcommitted(ctx: &AppContext, org: &str, periods: &[i64]) -> Result<Vec<Option<i64>>> {
    let Some(first) = periods.iter().min().and_then(|p| month_of(*p)) else {
        return Ok(Vec::new());
    };
    let span = periods.iter().max().map_or(1, |m| {
        u32::try_from(m - periods.iter().min().unwrap_or(m) + 1).unwrap_or(1)
    });
    // Months past the supported horizon are unknown, not guessed.
    if span > crate::rules::capacity::MAX_HORIZON_MONTHS {
        return Ok(vec![None; periods.len()]);
    }
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
    let rows: Vec<(NaiveDate, Option<NaiveDate>)> = early_warnings::Entity::find()
        .filter(early_warnings::Column::OrganizationRef.eq(org))
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|w| (w.first_true_on, w.acknowledged_on))
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
