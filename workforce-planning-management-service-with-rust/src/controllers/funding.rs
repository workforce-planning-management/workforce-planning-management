//! **Post funding** (WPM-R62, WPM-R98): where a worker's post is funded from and when that ends,
//! shown beside the contract end so a contract that outlasts its money, or money that outlasts the
//! contract, is flagged. Pure rules in [`crate::rules::funding`].
//!
//! A kind and a date only, never an amount. HR writes and reads it; it is exported with the worker and
//! erased with them. Nothing here changes an engagement.

use loco_rs::prelude::*;
use sea_orm::ActiveValue;
use serde::Deserialize;
use serde_json::json;

use super::{ensure_valid, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::models::_entities::{post_fundings, workers};
use crate::models::audit_logs::Model as Audit;
use crate::models::{memberships, records};
use crate::need_to_know::facts_for;
use crate::rules::equality_monitoring::DEFAULT_FLOOR;
use crate::rules::funding as rules;
use crate::validation::Problems;

/// How far ahead the funding horizon looks.
const HORIZON_MONTHS: u32 = 12;

async fn find(ctx: &AppContext, worker_pid: uuid::Uuid) -> Result<Option<post_fundings::Model>> {
    Ok(post_fundings::Entity::find()
        .filter(post_fundings::Column::WorkerPid.eq(worker_pid))
        .one(&ctx.db)
        .await?)
}

fn view(worker: &workers::Model, row: Option<&post_fundings::Model>) -> serde_json::Value {
    let alignment = rules::align(worker.engagement_ends_on, row.map(|r| r.ends_on));
    json!({
        "funding": row.map(|r| json!({
            "kind": r.kind, "programme_ref": r.programme_ref, "ends_on": r.ends_on,
        })),
        "contract_ends_on": worker.engagement_ends_on,
        "alignment": alignment.as_str(),
        "needs_attention": alignment.needs_attention(),
    })
}

async fn scoped(ctx: &AppContext, caller: &MaybeAuthUser, pid: &str) -> Result<workers::Model> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(pid)?).await?;
    if let Some(refs) = memberships::scope_organization_refs(&ctx.db, caller.claims()).await?
        && !refs.iter().any(|r| r == &worker.organization_ref)
    {
        return Err(Error::NotFound);
    }
    Ok(worker)
}

/// `GET /api/workers/{pid}/funding` — the funding beside the contract end, and how they compare.
/// HR only.
#[debug_handler]
async fn get_funding(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = scoped(&ctx, &caller, &pid).await?;
    let row = find(&ctx, worker.pid).await?;
    format::json(view(&worker, row.as_ref()))
}

/// Funding is HR's. The policy decides by action only, so a deployment that lets managers write
/// would otherwise let them write this; the handler asks for itself.
async fn ensure_privileged(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    worker: &workers::Model,
) -> Result<()> {
    if !crate::auth::require_auth() {
        return Ok(());
    }
    let claims = caller
        .claims()
        .ok_or(Error::Unauthorized("sign in".to_string()))?;
    if facts_for(ctx, claims, worker).await?.privileged {
        Ok(())
    } else {
        Err(Error::CustomError(
            axum::http::StatusCode::FORBIDDEN,
            loco_rs::controller::ErrorDetail::new("forbidden", "only HR records a post's funding"),
        ))
    }
}

/// `PUT /api/workers/{pid}/funding` body.
#[derive(Debug, Deserialize)]
struct FundingPayload {
    kind: String,
    programme_ref: Option<String>,
    ends_on: Option<chrono::NaiveDate>,
}

/// `PUT /api/workers/{pid}/funding` — set or change it.
#[debug_handler]
async fn set_funding(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<FundingPayload>,
) -> Result<Response> {
    let worker = super::engagements::writable_worker(&ctx, &caller, &pid).await?;
    ensure_privileged(&ctx, &caller, &worker).await?;
    let mut problems = Problems::new();
    problems.ref_opt(
        "programme_ref",
        entity_ref::EntityType::Thing,
        payload.programme_ref.as_deref(),
    );
    ensure_valid(&problems.into_vec())?;
    rules::validate(&payload.kind, payload.ends_on, worker.hired_on)
        .map_err(|e| unprocessable(&e))?;
    let by = caller.actor().map(ToString::to_string);
    let saved = if let Some(row) = find(&ctx, worker.pid).await? {
        let mut active: post_fundings::ActiveModel = row.into();
        active.kind = ActiveValue::set(payload.kind);
        active.programme_ref = ActiveValue::set(payload.programme_ref);
        active.ends_on = ActiveValue::set(payload.ends_on);
        active.recorded_by = ActiveValue::set(by);
        active.update(&ctx.db).await?
    } else {
        post_fundings::ActiveModel {
            worker_pid: ActiveValue::set(worker.pid),
            kind: ActiveValue::set(payload.kind),
            programme_ref: ActiveValue::set(payload.programme_ref),
            ends_on: ActiveValue::set(payload.ends_on),
            recorded_by: ActiveValue::set(by),
            ..Default::default()
        }
        .insert(&ctx.db)
        .await?
    };
    Audit::record(
        &ctx.db,
        "worker",
        worker.pid,
        "post_funding_set",
        caller.actor(),
        None,
    )
    .await?;
    format::json(view(&worker, Some(&saved)))
}

/// `DELETE /api/workers/{pid}/funding` — clear it (none recorded is a `404`).
#[debug_handler]
async fn clear_funding(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = super::engagements::writable_worker(&ctx, &caller, &pid).await?;
    ensure_privileged(&ctx, &caller, &worker).await?;
    let removed = post_fundings::Entity::delete_many()
        .filter(post_fundings::Column::WorkerPid.eq(worker.pid))
        .exec(&ctx.db)
        .await?
        .rows_affected;
    if removed == 0 {
        return Err(Error::NotFound);
    }
    Audit::record(
        &ctx.db,
        "worker",
        worker.pid,
        "post_funding_cleared",
        caller.actor(),
        None,
    )
    .await?;
    format::json(view(&worker, None))
}

/// `GET /api/workforce-intelligence/contingent` — the share of FTE that is contractor and that is
/// fixed-term (each apart), and the funding horizon: who is left when each month's money runs out
/// (WPM-R62). Aggregate only, names no one, small groups withheld. HR only.
#[debug_handler]
async fn contingent(State(ctx): State<AppContext>, caller: MaybeAuthUser) -> Result<Response> {
    let today = chrono::Utc::now().date_naive();
    let scope = memberships::scope_organization_refs(&ctx.db, caller.claims()).await?;
    let staff: Vec<workers::Model> = workers::Entity::find()
        .filter(workers::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?
        .into_iter()
        .filter(|w| crate::rules::metrics::is_employed_on(today, w.hired_on, w.terminated_on))
        .filter(|w| {
            scope
                .as_ref()
                .is_none_or(|refs| refs.iter().any(|r| r == &w.organization_ref))
        })
        .collect();
    let rows: Vec<(String, String, i32)> = staff
        .iter()
        .map(|w| {
            (
                w.department.clone(),
                w.employment_type.clone(),
                w.fte_percent,
            )
        })
        .collect();
    let floor = DEFAULT_FLOOR;
    let share = rules::contingent_share(&rows, floor);
    let pids: Vec<uuid::Uuid> = staff.iter().map(|w| w.pid).collect();
    let fte_of: std::collections::BTreeMap<uuid::Uuid, i32> =
        staff.iter().map(|w| (w.pid, w.fte_percent)).collect();
    let ends: Vec<(chrono::NaiveDate, i32)> = post_fundings::Entity::find()
        .filter(post_fundings::Column::WorkerPid.is_in(pids))
        .filter(post_fundings::Column::EndsOn.is_not_null())
        .all(&ctx.db)
        .await?
        .into_iter()
        .filter_map(|f| Some((f.ends_on?, *fte_of.get(&f.worker_pid)?)))
        .collect();
    let horizon = rules::funding_horizon(&ends, today, HORIZON_MONTHS, floor);
    Audit::record(
        &ctx.db,
        "contingent_workforce",
        uuid::Uuid::nil(),
        "contingent_workforce_read",
        caller.actor(),
        None,
    )
    .await?;
    format::json(json!({
        "small_group_floor": floor,
        "headcount": share.headcount,
        "fte": share.fte,
        "contractor_percent_of_fte": share.contractor_percent,
        "fixed_term_percent_of_fte": share.fixed_term_percent,
        "departments_shown": share.departments_shown,
        "by_department": share.departments.iter().map(|d| json!({
            "department": d.department,
            "headcount": d.headcount,
            "contractor_percent_of_fte": d.contractor_percent,
            "fixed_term_percent_of_fte": d.fixed_term_percent,
        })).collect::<Vec<_>>(),
        "funding_ends": horizon.iter().map(|m| json!({
            "month": m.month, "people": m.people, "fte": m.fte,
        })).collect::<Vec<_>>(),
        "derivation": "Contractor and fixed-term are each a share of the FTE of the people employed today, so \
                       part-time work counts for what it is. The department breakdown shows only when every \
                       department reaches the floor, so a withheld one cannot be worked out from the others. \
                       Funding ends counts people employed today whose recorded funding ends in that month, \
                       or already has; a month with some but fewer than the floor is withheld, and no total is \
                       given, so it cannot be worked out. A worker with no funding recorded is not counted. \
                       Trend over time needs the snapshots split by basis (WPM-T139), which are not built.",
    }))
}

/// The funding routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/workers/{pid}/funding", get(get_funding))
        .add("/workers/{pid}/funding", put(set_funding))
        .add("/workers/{pid}/funding", delete(clear_funding))
        .add("/workforce-intelligence/contingent", get(contingent))
}
