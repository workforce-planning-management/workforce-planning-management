//! Pay position analysis (WPM-R154, WPM-D108): `GET /api/reports/pay-range?organization=&as_of=&cutoff=`. Where pay
//! sits inside its band, as compa-ratio and range penetration per band, level, department and location, with the
//! groups that sit far from the rest. An aggregate: small groups are withheld, no worker is named, nothing is
//! stored and no pay change is proposed. It reads salaries, so it needs the same unmasked read as the pay gap report.
//! Spec: `spec/payroll-compensation.md`.

use std::collections::BTreeMap;

use chrono::{NaiveDate, Utc};
use loco_rs::prelude::*;
use sea_orm::QueryOrder;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use super::pay_transparency::organization_problems;
use super::{ensure_org_in_scope, ensure_valid, forbidden, record_rejection, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::models::_entities::{worker_job_levels, worker_pay_positions, workers};
use crate::models::audit_logs::Model as Audit;
use crate::rules::cost as cost_rules;
use crate::rules::equality_monitoring::DEFAULT_FLOOR;
use crate::rules::metrics::is_employed_on;
use crate::rules::pay_range::{self as rules, Entry, Range};
use crate::rules::pay_scale;

/// What the figures are made of, said in every answer.
const BASIS: &str = "Annual full-time-equivalent basic salary on the worker record, over the midpoint of the worker's recorded pay band \
(the middle of its entry and top steps). A worker with no pay, pay in another currency, or no band on a known scale is left out and counted.";

#[derive(Debug, Deserialize)]
pub(crate) struct Query {
    organization: String,
    as_of: Option<NaiveDate>,
    /// Outlier cut-off in whole percentage points of compa-ratio (default 10).
    cutoff: Option<i64>,
    currency: Option<String>,
}

fn most_common_currency(staff: &[&workers::Model]) -> Option<String> {
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for w in staff {
        if w.salary_minor.is_some()
            && let Some(code) = &w.salary_currency
        {
            *counts.entry(code.as_str()).or_insert(0) += 1;
        }
    }
    counts
        .into_iter()
        .max_by(|a, b| a.1.cmp(&b.1).then(b.0.cmp(a.0)))
        .map(|(code, _)| code.to_string())
}

/// `GET /api/reports/pay-range` — the analysis. Payroll or HR with an unmasked salary read; audited.
#[debug_handler]
#[allow(clippy::too_many_lines)]
async fn pay_range(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(query): axum::extract::Query<Query>,
) -> Result<Response> {
    ensure_valid(&organization_problems(&query.organization))?;
    ensure_org_in_scope(&ctx, &caller, &query.organization).await?;
    // The same rule as the plan cost view: a figure made of salaries is for a caller who may read salaries.
    let obligations = auth::authorize_record(
        &caller,
        authentication_verifier::Action::Read,
        &BTreeMap::new(),
    )
    .map_err(record_rejection)?;
    if obligations.iter().any(|o| o == "mask") {
        return Err(forbidden(
            "a pay position analysis is made of salaries and needs payroll read access",
        ));
    }
    let today = Utc::now().date_naive();
    let as_of = query.as_of.unwrap_or(today);
    if as_of > today {
        return Err(unprocessable("as_of must not be in the future"));
    }
    let cutoff_bp = match query.cutoff {
        None => rules::DEFAULT_CUTOFF_BP,
        Some(points) => {
            rules::check_cutoff(points.saturating_mul(100)).map_err(|e| unprocessable(&e))?
        }
    };
    let staff: Vec<workers::Model> = workers::Entity::find()
        .filter(workers::Column::OrganizationRef.eq(&query.organization))
        .filter(workers::Column::DeletedAt.is_null())
        .order_by_asc(workers::Column::Id)
        .all(&ctx.db)
        .await?
        .into_iter()
        .filter(|w| is_employed_on(as_of, w.hired_on, w.terminated_on))
        .collect();
    let positions: BTreeMap<Uuid, worker_pay_positions::Model> =
        worker_pay_positions::Entity::find()
            .all(&ctx.db)
            .await?
            .into_iter()
            .map(|p| (p.worker_pid, p))
            .collect();
    let levels: BTreeMap<Uuid, String> = worker_job_levels::Entity::find()
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|l| (l.worker_pid, format!("{}:{}", l.framework, l.level_number)))
        .collect();
    // Only workers with a recorded band can be placed in one; the others are counted as having none.
    let with_band: Vec<&workers::Model> = staff
        .iter()
        .filter(|w| positions.contains_key(&w.pid))
        .collect();
    let currency = match &query.currency {
        Some(code) if cost_rules::valid_currency(code) => Some(code.clone()),
        Some(_) => return Err(unprocessable("currency must be an ISO-4217 code, e.g. GBP")),
        None => most_common_currency(&with_band),
    };
    let entries: Vec<Entry> = staff
        .iter()
        .map(|w| {
            let position = positions.get(&w.pid);
            let scale = position.and_then(|p| pay_scale::find(&p.scale_id));
            let band = position.and_then(|p| scale.as_ref().and_then(|s| s.band(&p.band)));
            let comparable = scale
                .as_ref()
                .is_some_and(|s| currency.as_deref() == Some(s.currency));
            Entry {
                // Pay in another currency than the analysis counts as no pay recorded.
                salary_minor: w
                    .salary_minor
                    .filter(|_| currency.is_some() && w.salary_currency == currency),
                band: position.map(|p| p.band.clone()).unwrap_or_default(),
                range: band.filter(|_| comparable).and_then(|b| {
                    Some(Range {
                        min: b.steps.first()?.annual_minor,
                        max: b.steps.last()?.annual_minor,
                    })
                }),
                level: levels.get(&w.pid).cloned(),
                department: w.department.clone(),
                location: w.location.clone(),
            }
        })
        .collect();
    let report = rules::analyse(&entries, DEFAULT_FLOOR, cutoff_bp);
    Audit::record(
        &ctx.db,
        "pay_range_report",
        Uuid::nil(),
        "pay_range_report_read",
        caller.actor(),
        Some(json!({ "as_of": as_of })),
    )
    .await?;
    format::json(json!({
        "organization": query.organization,
        "as_of": as_of,
        "currency": currency,
        "basis": BASIS,
        "report": report,
        "derivation": "Compa-ratio is 10 000 at the band midpoint; range penetration is 0 at the entry step and 10 000 at the top, \
                       and is not clamped. A group shows only from the floor and no count is given below it; a total is not shown if \
                       it would let a withheld group be worked out. An outlier is a group, never a worker; this is evidence for a \
                       compensation review and proposes no pay change.",
    }))
}

/// The route.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/reports/pay-range", get(pay_range))
}
