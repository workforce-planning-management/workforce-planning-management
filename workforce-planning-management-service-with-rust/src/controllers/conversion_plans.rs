//! **Conversion plans** (WPM-R98, WPM-D66): what a person intends for a fixed-term or contractor
//! engagement, recorded with who proposed it and who approved it. Pure rules in
//! [`crate::rules::conversion`].
//!
//! A plan is a decision a person makes and records, never an outcome of length of service. Reads are
//! for HR. The worker's line manager or a privileged caller proposes, approves and carries out a
//! plan, never the worker, and nobody approves their own. Marking a conversion done is the only
//! step that changes the worker: it makes them permanent and clears the contract end in one
//! transaction, and the plan, the extensions and the decisions stay as history. No audit entry
//! carries the reason.

use chrono::{NaiveDate, Utc};
use loco_rs::prelude::*;
use sea_orm::{ActiveValue, QueryOrder, TransactionTrait};
use serde::Deserialize;
use serde_json::json;

use super::unprocessable;
use crate::auth::MaybeAuthUser;
use crate::models::_entities::{conversion_plans, engagement_decisions, workers};
use crate::models::audit_logs::Model as Audit;
use crate::models::{memberships, records};
use crate::rules::conversion::{self as rules, Proposal};

fn plan_json(
    row: &conversion_plans::Model,
    contract_ends_on: Option<NaiveDate>,
    as_of: NaiveDate,
) -> serde_json::Value {
    let flags: Vec<&str> = rules::flags(
        contract_ends_on,
        Some(row.status.as_str()),
        row.review_on,
        as_of,
    )
    .into_iter()
    .map(rules::Flag::as_str)
    .collect();
    json!({
        "pid": row.pid, "worker_pid": row.worker_pid, "intent": row.intent,
        "target_on": row.target_on, "department": row.department,
        "role_profile_ref": row.role_profile_ref,
        "post_funding_kind": row.post_funding_kind, "post_funding_ends_on": row.post_funding_ends_on,
        "reason": row.reason, "proposed_by": row.proposed_by, "approved_by": row.approved_by,
        "status": row.status, "review_on": row.review_on, "settled_on": row.settled_on,
        "flags": flags,
    })
}

/// A worker inside the caller's organizations, or `404`.
async fn scoped_worker(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    worker: workers::Model,
) -> Result<workers::Model> {
    if let Some(refs) = memberships::scope_organization_refs(&ctx.db, caller.claims()).await?
        && !refs.iter().any(|r| r == &worker.organization_ref)
    {
        return Err(Error::NotFound);
    }
    Ok(worker)
}

async fn open_plan(
    ctx: &AppContext,
    worker_pid: uuid::Uuid,
) -> Result<Option<conversion_plans::Model>> {
    Ok(conversion_plans::Entity::find()
        .filter(conversion_plans::Column::WorkerPid.eq(worker_pid))
        .filter(conversion_plans::Column::Status.is_in(["proposed", "approved"]))
        .one(&ctx.db)
        .await?)
}

/// `GET /api/workers/{pid}/conversion-plans` — the worker's plans, oldest first. HR only.
#[debug_handler]
async fn list_for_worker(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    let worker = scoped_worker(&ctx, &caller, worker).await?;
    let as_of = Utc::now().date_naive();
    let rows = conversion_plans::Entity::find()
        .filter(conversion_plans::Column::WorkerPid.eq(worker.pid))
        .order_by_asc(conversion_plans::Column::Id)
        .all(&ctx.db)
        .await?;
    Audit::record(
        &ctx.db,
        "worker",
        worker.pid,
        "conversion_plans_read",
        caller.actor(),
        None,
    )
    .await?;
    format::json(
        rows.iter()
            .map(|r| plan_json(r, worker.engagement_ends_on, as_of))
            .collect::<Vec<_>>(),
    )
}

/// Query for the list of open plans.
#[derive(Debug, Deserialize)]
struct ListQuery {
    organization: Option<String>,
}

/// `GET /api/conversion-plans` — every open plan in the caller's organizations with its flags, and
/// the contingent engagements past their end that no approved plan covers. HR only; names workers.
#[debug_handler]
async fn list_open(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(query): axum::extract::Query<ListQuery>,
) -> Result<Response> {
    let as_of = Utc::now().date_naive();
    let scope = memberships::scope_organization_refs(&ctx.db, caller.claims()).await?;
    let in_scope = |w: &workers::Model| {
        query
            .organization
            .as_ref()
            .is_none_or(|o| o == &w.organization_ref)
            && scope
                .as_ref()
                .is_none_or(|refs| refs.iter().any(|r| r == &w.organization_ref))
    };
    let staff: Vec<workers::Model> = workers::Entity::find()
        .filter(workers::Column::DeletedAt.is_null())
        .filter(workers::Column::TerminatedOn.is_null())
        .filter(workers::Column::EngagementEndsOn.is_not_null())
        .order_by_asc(workers::Column::WorkerNumber)
        .all(&ctx.db)
        .await?
        .into_iter()
        .filter(in_scope)
        .collect();
    let mut out = Vec::new();
    for w in &staff {
        let plan = open_plan(&ctx, w.pid).await?;
        let status = plan.as_ref().map(|p| p.status.as_str());
        let flags = rules::flags(
            w.engagement_ends_on,
            status,
            plan.as_ref().and_then(|p| p.review_on),
            as_of,
        );
        if plan.is_none() && flags.is_empty() {
            continue;
        }
        out.push(json!({
            "worker_pid": w.pid, "worker_number": w.worker_number, "department": w.department,
            "employment_type": w.employment_type, "engagement_ends_on": w.engagement_ends_on,
            "plan": plan.as_ref().map(|p| plan_json(p, w.engagement_ends_on, as_of)),
            "flags": flags.into_iter().map(rules::Flag::as_str).collect::<Vec<_>>(),
        }));
    }
    Audit::record(
        &ctx.db,
        "conversion_plan",
        uuid::Uuid::nil(),
        "conversion_plans_listed",
        caller.actor(),
        None,
    )
    .await?;
    format::json(out)
}

/// `POST …/conversion-plans` body.
#[derive(Debug, Deserialize)]
struct ProposePayload {
    intent: String,
    target_on: NaiveDate,
    department: Option<String>,
    role_profile_ref: Option<String>,
    post_funding_kind: Option<String>,
    post_funding_ends_on: Option<NaiveDate>,
    reason: Option<String>,
    review_on: Option<NaiveDate>,
}

/// `POST /api/workers/{pid}/conversion-plans` — propose a plan for a fixed-term or contractor
/// engagement. The worker's line manager or a privileged caller, never the worker. One plan is open
/// at a time.
#[debug_handler]
async fn propose(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<ProposePayload>,
) -> Result<Response> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    super::flexible_working::ensure_decider(&ctx, &caller, &worker).await?;
    if worker.terminated_on.is_some() {
        return Err(unprocessable("the worker's employment has ended"));
    }
    if worker.employment_type == "permanent" {
        return Err(unprocessable(
            "a conversion plan is for a fixed-term, contractor or intern engagement",
        ));
    }
    if open_plan(&ctx, worker.pid).await?.is_some() {
        return Err(unprocessable(
            "the worker already has an open plan; approve, carry out or abandon it first",
        ));
    }
    let today = Utc::now().date_naive();
    rules::validate_proposal(
        &Proposal {
            intent: &payload.intent,
            target_on: payload.target_on,
            post_funding_kind: payload.post_funding_kind.as_deref(),
            post_funding_ends_on: payload.post_funding_ends_on,
            reason: payload.reason.as_deref(),
            review_on: payload.review_on,
            contract_ends_on: worker.engagement_ends_on,
        },
        today,
    )
    .map_err(|e| unprocessable(&e))?;
    let row = conversion_plans::ActiveModel {
        pid: ActiveValue::set(uuid::Uuid::new_v4()),
        worker_pid: ActiveValue::set(worker.pid),
        intent: ActiveValue::set(payload.intent),
        target_on: ActiveValue::set(payload.target_on),
        department: ActiveValue::set(payload.department),
        role_profile_ref: ActiveValue::set(payload.role_profile_ref),
        post_funding_kind: ActiveValue::set(payload.post_funding_kind),
        post_funding_ends_on: ActiveValue::set(payload.post_funding_ends_on),
        reason: ActiveValue::set(payload.reason),
        proposed_by: ActiveValue::set(caller.actor().map(ToString::to_string)),
        status: ActiveValue::set("proposed".to_string()),
        review_on: ActiveValue::set(payload.review_on),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;
    Audit::record(
        &ctx.db,
        "worker",
        worker.pid,
        "conversion_plan_proposed",
        caller.actor(),
        None,
    )
    .await?;
    format::json(plan_json(&row, worker.engagement_ends_on, today))
}

async fn plan_and_worker(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: &str,
) -> Result<(conversion_plans::Model, workers::Model)> {
    let row = conversion_plans::Entity::find()
        .filter(conversion_plans::Column::Pid.eq(records::parse_pid(pid)?))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    let worker = records::find_worker(&ctx.db, row.worker_pid).await?;
    let worker = scoped_worker(ctx, caller, worker).await?;
    super::flexible_working::ensure_decider(ctx, caller, &worker).await?;
    Ok((row, worker))
}

/// `POST /api/conversion-plans/{pid}/approve` — approve a proposed plan. Never the proposer.
/// Approving records the matching engagement decision against the current end date, in the same
/// transaction, so the end-of-engagement reminder is settled by a person's decision.
#[debug_handler]
async fn approve(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let (row, worker) = plan_and_worker(&ctx, &caller, &pid).await?;
    let actor = caller.actor().map(ToString::to_string);
    rules::validate_approval(
        &row.intent,
        &row.status,
        row.proposed_by.as_deref(),
        actor.as_deref(),
    )
    .map_err(|e| unprocessable(&e))?;
    let today = Utc::now().date_naive();
    let txn = ctx.db.begin().await?;
    let mut active: conversion_plans::ActiveModel = row.clone().into();
    active.status = ActiveValue::set("approved".to_string());
    active.approved_by = ActiveValue::set(actor.clone());
    let updated = active.update(&txn).await?;
    if let Some(ends_on) = worker.engagement_ends_on {
        let settled = engagement_decisions::Entity::find()
            .filter(engagement_decisions::Column::WorkerPid.eq(worker.pid))
            .filter(engagement_decisions::Column::EndsOn.eq(ends_on))
            .one(&txn)
            .await?
            .is_some();
        if !settled {
            engagement_decisions::ActiveModel {
                pid: ActiveValue::set(uuid::Uuid::new_v4()),
                worker_pid: ActiveValue::set(worker.pid),
                ends_on: ActiveValue::set(ends_on),
                decision: ActiveValue::set(row.intent.clone()),
                decided_by: ActiveValue::set(actor),
                decided_on: ActiveValue::set(today),
                ..Default::default()
            }
            .insert(&txn)
            .await?;
        }
    }
    txn.commit().await?;
    Audit::record(
        &ctx.db,
        "worker",
        worker.pid,
        "conversion_plan_approved",
        caller.actor(),
        None,
    )
    .await?;
    format::json(plan_json(&updated, worker.engagement_ends_on, today))
}

/// `POST /api/conversion-plans/{pid}/abandon` — drop an open plan. The plan stays as history.
#[debug_handler]
async fn abandon(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let (row, worker) = plan_and_worker(&ctx, &caller, &pid).await?;
    if !rules::may_move(&row.status, "abandoned") {
        return Err(unprocessable(&format!(
            "a plan that is {} cannot be abandoned",
            row.status
        )));
    }
    let today = Utc::now().date_naive();
    let mut active: conversion_plans::ActiveModel = row.into();
    active.status = ActiveValue::set("abandoned".to_string());
    active.settled_on = ActiveValue::set(Some(today));
    let updated = active.update(&ctx.db).await?;
    Audit::record(
        &ctx.db,
        "worker",
        worker.pid,
        "conversion_plan_abandoned",
        caller.actor(),
        None,
    )
    .await?;
    format::json(plan_json(&updated, worker.engagement_ends_on, today))
}

/// `POST /api/conversion-plans/{pid}/done` — carry out an approved conversion: the worker becomes
/// permanent and the contract end is cleared, in one transaction. The plan, the extensions and the
/// decisions are kept as the history. The department the plan names, if any, replaces the worker's.
/// Payroll, the contractor's supplier details and the rest of the permanent record are for the
/// processes that own them.
#[debug_handler]
async fn done(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let (row, worker) = plan_and_worker(&ctx, &caller, &pid).await?;
    rules::validate_done(&row.intent, &row.status).map_err(|e| unprocessable(&e))?;
    if worker.terminated_on.is_some() {
        return Err(unprocessable("the worker's employment has ended"));
    }
    let today = Utc::now().date_naive();
    let txn = ctx.db.begin().await?;
    let mut w: workers::ActiveModel = worker.clone().into();
    w.employment_type = ActiveValue::set("permanent".to_string());
    w.engagement_ends_on = ActiveValue::set(None);
    if let Some(department) = &row.department {
        w.department = ActiveValue::set(department.clone());
    }
    w.update(&txn).await?;
    let mut active: conversion_plans::ActiveModel = row.into();
    active.status = ActiveValue::set("done".to_string());
    active.settled_on = ActiveValue::set(Some(today));
    let updated = active.update(&txn).await?;
    txn.commit().await?;
    Audit::record(
        &ctx.db,
        "worker",
        worker.pid,
        "conversion_plan_done",
        caller.actor(),
        None,
    )
    .await?;
    format::json(plan_json(&updated, None, today))
}

/// The conversion-plan routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/conversion-plans", get(list_open))
        .add("/workers/{pid}/conversion-plans", get(list_for_worker))
        .add("/workers/{pid}/conversion-plans", post(propose))
        .add("/conversion-plans/{pid}/approve", post(approve))
        .add("/conversion-plans/{pid}/abandon", post(abandon))
        .add("/conversion-plans/{pid}/done", post(done))
}
