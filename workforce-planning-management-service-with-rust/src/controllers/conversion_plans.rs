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

use super::ensure_org_in_scope;
use axa::extract::{Json, Path};
use axa::prelude::*;
use chrono::{NaiveDate, Utc};
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

use super::unprocessable;
use crate::auth::MaybeAuthUser;
use crate::db;
use crate::models::_entities::{conversion_plans, workers};
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
    ensure_org_in_scope(ctx, caller, &worker.organization_ref).await?;
    Ok(worker)
}

async fn open_plan(ctx: &AppContext, worker_pid: Uuid) -> Result<Option<conversion_plans::Model>> {
    Ok(sqlx::query_as!(
        conversion_plans::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, intent, target_on,
                  department, role_profile_ref, post_funding_kind, post_funding_ends_on, reason, proposed_by, approved_by,
                  status, review_on, settled_on
             FROM conversion_plans WHERE worker_pid = $1 AND status IN ('proposed', 'approved') LIMIT 1"#,
        worker_pid,
    )
    .fetch_optional(db::pool(ctx))
    .await?)
}

/// `GET /api/workers/{pid}/conversion-plans` — the worker's plans, oldest first. HR only.
#[debug_handler]
async fn list_for_worker(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    let worker = scoped_worker(&ctx, &caller, worker).await?;
    let as_of = Utc::now().date_naive();
    let rows = sqlx::query_as!(
        conversion_plans::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, intent, target_on,
                  department, role_profile_ref, post_funding_kind, post_funding_ends_on, reason, proposed_by, approved_by,
                  status, review_on, settled_on
             FROM conversion_plans WHERE worker_pid = $1 ORDER BY id"#,
        worker.pid,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    db::audit(
        &mut *db::pool(&ctx).acquire().await?,
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

/// What the list of open plans needs of a contingent worker.
struct Contingent {
    pid: Uuid,
    organization_ref: String,
    worker_number: String,
    department: String,
    employment_type: String,
    engagement_ends_on: Option<NaiveDate>,
}

/// `GET /api/conversion-plans` — every open plan in the caller's organizations with its flags, and
/// the contingent engagements past their end that no approved plan covers. HR only; names workers.
#[debug_handler]
async fn list_open(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axa::extract::Query(query): axa::extract::Query<ListQuery>,
    page: super::Page,
) -> Result<Response> {
    let window = super::window(&page)?;
    let as_of = Utc::now().date_naive();
    let scope = memberships::scope_organization_refs(&ctx, caller.claims()).await?;
    let staff: Vec<Contingent> = sqlx::query_as!(
        Contingent,
        "SELECT pid, organization_ref, worker_number, department, employment_type, engagement_ends_on
           FROM workers
          WHERE deleted_at IS NULL AND terminated_on IS NULL AND engagement_ends_on IS NOT NULL
          ORDER BY worker_number, id",
    )
    .fetch_all(db::pool(&ctx))
    .await?
    .into_iter()
    .filter(|w| {
        query
            .organization
            .as_ref()
            .is_none_or(|o| o == &w.organization_ref)
            && scope
                .as_ref()
                .is_none_or(|refs| refs.iter().any(|r| r == &w.organization_ref))
    })
    .collect();
    let pids: Vec<Uuid> = staff.iter().map(|w| w.pid).collect();
    // At most one open plan per worker: a partial unique index holds it.
    let mut open: HashMap<Uuid, conversion_plans::Model> = sqlx::query_as!(
        conversion_plans::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, intent, target_on,
                  department, role_profile_ref, post_funding_kind, post_funding_ends_on, reason, proposed_by, approved_by,
                  status, review_on, settled_on
             FROM conversion_plans WHERE worker_pid = ANY($1) AND status IN ('proposed', 'approved')"#,
        &pids,
    )
    .fetch_all(db::pool(&ctx))
    .await?
    .into_iter()
    .map(|p| (p.worker_pid, p))
    .collect();
    let mut out = Vec::new();
    for w in &staff {
        let plan = open.remove(&w.pid);
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
    db::audit(
        &mut *db::pool(&ctx).acquire().await?,
        "conversion_plan",
        Uuid::nil(),
        "conversion_plans_listed",
        caller.actor(),
        None,
    )
    .await?;
    Ok(super::paged_vec(out, &window))
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
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
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
    let mut tx = db::pool(&ctx).begin().await?;
    let inserted = sqlx::query_as!(
        conversion_plans::Model,
        r#"INSERT INTO conversion_plans (pid, worker_pid, intent, target_on, department, role_profile_ref, post_funding_kind,
                                         post_funding_ends_on, reason, proposed_by, status, review_on)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 'proposed', $11)
           RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, intent, target_on,
                     department, role_profile_ref, post_funding_kind, post_funding_ends_on, reason, proposed_by, approved_by,
                     status, review_on, settled_on"#,
        Uuid::new_v4(),
        worker.pid,
        payload.intent,
        payload.target_on,
        payload.department,
        payload.role_profile_ref,
        payload.post_funding_kind,
        payload.post_funding_ends_on,
        payload.reason,
        caller.actor(),
        payload.review_on,
    )
    .fetch_one(&mut *tx)
    .await;
    // Two proposals at once: the partial unique index lets one in and this one is told, not failed.
    let row = match inserted {
        Ok(row) => row,
        Err(e) => {
            tx.rollback().await?;
            return if open_plan(&ctx, worker.pid).await?.is_some() {
                Err(unprocessable(
                    "the worker already has an open plan; approve, carry out or abandon it first",
                ))
            } else {
                Err(e.into())
            };
        }
    };
    db::audit(
        &mut tx,
        "worker",
        worker.pid,
        "conversion_plan_proposed",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(plan_json(&row, worker.engagement_ends_on, today))
}

async fn plan_and_worker(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: &str,
) -> Result<(conversion_plans::Model, workers::Model)> {
    let row = sqlx::query_as!(
        conversion_plans::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, intent, target_on,
                  department, role_profile_ref, post_funding_kind, post_funding_ends_on, reason, proposed_by, approved_by,
                  status, review_on, settled_on
             FROM conversion_plans WHERE pid = $1"#,
        records::parse_pid(pid)?,
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)?;
    let worker = records::worker_by_pid(ctx, row.worker_pid).await?;
    let worker = scoped_worker(ctx, caller, worker).await?;
    super::flexible_working::ensure_decider(ctx, caller, &worker).await?;
    Ok((row, worker))
}

/// Re-read a plan inside the transaction with a row lock, so two callers acting on the same plan at
/// once are taken one after the other and the second sees what the first did.
async fn locked(connection: &mut sqlx::PgConnection, id: i32) -> Result<conversion_plans::Model> {
    sqlx::query_as!(
        conversion_plans::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, intent, target_on,
                  department, role_profile_ref, post_funding_kind, post_funding_ends_on, reason, proposed_by, approved_by,
                  status, review_on, settled_on
             FROM conversion_plans WHERE id = $1 FOR UPDATE"#,
        id,
    )
    .fetch_optional(connection)
    .await?
    .ok_or(Error::NotFound)
}

/// What a plan's step needs of its worker, re-read under a row lock.
struct LockedWorker {
    pid: Uuid,
    engagement_ends_on: Option<NaiveDate>,
    terminated_on: Option<NaiveDate>,
}

async fn locked_worker(connection: &mut sqlx::PgConnection, id: i32) -> Result<LockedWorker> {
    sqlx::query_as!(
        LockedWorker,
        "SELECT pid, engagement_ends_on, terminated_on FROM workers WHERE id = $1 FOR UPDATE",
        id,
    )
    .fetch_optional(connection)
    .await?
    .ok_or(Error::NotFound)
}

/// Set a plan's status, and its approver or settlement day where given; the row as it now stands.
async fn set_status(
    connection: &mut sqlx::PgConnection,
    id: i32,
    status: &str,
    approved_by: Option<&str>,
    settled_on: Option<NaiveDate>,
) -> Result<conversion_plans::Model> {
    Ok(sqlx::query_as!(
        conversion_plans::Model,
        r#"UPDATE conversion_plans
              SET status = $2, approved_by = COALESCE($3, approved_by), settled_on = COALESCE($4, settled_on)
            WHERE id = $1
        RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, intent, target_on,
                  department, role_profile_ref, post_funding_kind, post_funding_ends_on, reason, proposed_by, approved_by,
                  status, review_on, settled_on"#,
        id,
        status,
        approved_by,
        settled_on,
    )
    .fetch_one(connection)
    .await?)
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
    let today = Utc::now().date_naive();
    let mut tx = db::pool(&ctx).begin().await?;
    let row = locked(&mut tx, row.id).await?;
    rules::validate_approval(
        &row.intent,
        &row.status,
        row.proposed_by.as_deref(),
        actor.as_deref(),
    )
    .map_err(|e| unprocessable(&e))?;
    let worker = locked_worker(&mut tx, worker.id).await?;
    let updated = set_status(&mut tx, row.id, "approved", actor.as_deref(), None).await?;
    if let Some(ends_on) = worker.engagement_ends_on {
        let settled = sqlx::query_scalar!(
            r#"SELECT 1 AS "one!" FROM engagement_decisions WHERE worker_pid = $1 AND ends_on = $2 LIMIT 1"#,
            worker.pid,
            ends_on,
        )
        .fetch_optional(&mut *tx)
        .await?
        .is_some();
        if !settled {
            sqlx::query!(
                "INSERT INTO engagement_decisions (pid, worker_pid, ends_on, decision, decided_by, decided_on)
                 VALUES ($1, $2, $3, $4, $5, $6)",
                Uuid::new_v4(),
                worker.pid,
                ends_on,
                row.intent,
                actor,
                today,
            )
            .execute(&mut *tx)
            .await?;
        }
    }
    db::audit(
        &mut tx,
        "worker",
        worker.pid,
        "conversion_plan_approved",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
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
    let mut tx = db::pool(&ctx).begin().await?;
    let updated = set_status(&mut tx, row.id, "abandoned", None, Some(today)).await?;
    db::audit(
        &mut tx,
        "worker",
        worker.pid,
        "conversion_plan_abandoned",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
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
    let today = Utc::now().date_naive();
    let mut tx = db::pool(&ctx).begin().await?;
    let row = locked(&mut tx, row.id).await?;
    rules::validate_done(&row.intent, &row.status).map_err(|e| unprocessable(&e))?;
    let worker = locked_worker(&mut tx, worker.id).await?;
    if worker.terminated_on.is_some() {
        return Err(unprocessable("the worker's employment has ended"));
    }
    sqlx::query!(
        "UPDATE workers
            SET employment_type = 'permanent', engagement_ends_on = NULL, department = COALESCE($2, department)
          WHERE pid = $1",
        worker.pid,
        row.department,
    )
    .execute(&mut *tx)
    .await?;
    let updated = set_status(&mut tx, row.id, "done", None, Some(today)).await?;
    db::audit(
        &mut tx,
        "worker",
        worker.pid,
        "conversion_plan_done",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
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
