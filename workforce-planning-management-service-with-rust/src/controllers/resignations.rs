//! **Resignations** (WPM-R127, WPM-D77): a worker logs an intent to resign; a person accepts it.
//! Pure rules in [`crate::rules::resignation`].
//!
//! The worker's side (`/api/me/resignation`) is in [`super::me`]; this module holds the shared
//! logic and the side of whoever decides. Accepting records the agreed last day and opens the
//! leaver process in the same transaction. It never ends anyone's employment: a person does that.
//!
//! **The reason is the worker's own.** Their manager and HR are told *that* a resignation was
//! logged and when, never why; the reason appears to the worker and in aggregates that withhold
//! small groups.

use chrono::{NaiveDate, Utc};
use loco_rs::prelude::*;
use sea_orm::{ActiveValue, QueryOrder, TransactionTrait};
use serde::Deserialize;
use serde_json::json;

use super::movements::{MovementPayload, open_in};
use super::{record_rejection, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::models::_entities::{movements, resignations, workers};
use crate::models::audit_logs::Model as Audit;
use crate::models::notifications::Model as Notification;
use crate::models::{memberships, records};
use crate::rules::pulse::K_ANONYMITY;
use crate::rules::resignation::{self as rules, Action, Status};

/// The deployment's notice period in calendar days: `WPM_RESIGNATION_NOTICE_CALENDAR_DAYS`, else
/// 28. An invalid setting is ignored with a warning, never silently shortened.
#[must_use]
pub fn notice_days() -> i64 {
    match crate::compat::env_var("WPM_RESIGNATION_NOTICE_CALENDAR_DAYS") {
        None => rules::DEFAULT_NOTICE_CALENDAR_DAYS,
        Some(text) => match text.trim().parse::<i64>() {
            Ok(n) if rules::validate_notice(n).is_ok() => n,
            _ => {
                tracing::warn!(
                    "WPM_RESIGNATION_NOTICE_CALENDAR_DAYS is not 0 to 366; using the default"
                );
                rules::DEFAULT_NOTICE_CALENDAR_DAYS
            }
        },
    }
}

/// The worker's latest resignation of any status.
pub(crate) async fn latest(
    ctx: &AppContext,
    worker_pid: uuid::Uuid,
) -> Result<Option<resignations::Model>> {
    Ok(resignations::Entity::find()
        .filter(resignations::Column::WorkerPid.eq(worker_pid))
        .order_by_desc(resignations::Column::Id)
        .one(&ctx.db)
        .await?)
}

fn status_of(row: &resignations::Model) -> Result<Status> {
    Status::parse(&row.status).ok_or_else(|| Error::string("a resignation has an unknown status"))
}

/// What the worker themself sees, with their reason.
pub(crate) fn own_json(row: &resignations::Model) -> serde_json::Value {
    json!({
        "pid": row.pid,
        "status": row.status,
        "logged_on": row.logged_on,
        "proposed_last_day": row.proposed_last_day,
        "agreed_last_day": row.agreed_last_day,
        "reason": row.reason,
        "notice_calendar_days": notice_days(),
    })
}

/// What a manager or HR sees: that it was logged, when, and the dates. **Never the reason.**
fn shared_json(row: &resignations::Model) -> serde_json::Value {
    json!({
        "pid": row.pid,
        "status": row.status,
        "logged_on": row.logged_on,
        "proposed_last_day": row.proposed_last_day,
        "agreed_last_day": row.agreed_last_day,
    })
}

/// `POST /api/me/resignation` body.
#[derive(Debug, Deserialize)]
pub(crate) struct LogPayload {
    proposed_last_day: NaiveDate,
    /// Optional, from a closed list; never free text.
    #[serde(default)]
    reason: Option<String>,
}

/// Log a resignation for this worker. Shared with `/api/me/resignation`.
pub(crate) async fn log_for(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    worker: &workers::Model,
    payload: LogPayload,
) -> Result<Response> {
    if matches!(worker.status.as_str(), "terminated" | "retired") {
        return Err(unprocessable("your employment has already ended"));
    }
    rules::validate_reason(payload.reason.as_deref()).map_err(|e| unprocessable(&e))?;
    let today = Utc::now().date_naive();
    let notice = notice_days();
    rules::validate_proposal(today, payload.proposed_last_day, notice)
        .map_err(|e| unprocessable(&e))?;
    let txn = ctx.db.begin().await?;
    let row = resignations::ActiveModel {
        pid: ActiveValue::set(uuid::Uuid::new_v4()),
        worker_pid: ActiveValue::set(worker.pid),
        logged_on: ActiveValue::set(today),
        proposed_last_day: ActiveValue::set(payload.proposed_last_day),
        reason: ActiveValue::set(payload.reason.clone()),
        status: ActiveValue::set(Status::Logged.as_str().to_string()),
        ..Default::default()
    }
    .insert(&txn)
    .await
    .map_err(|e| {
        if e.to_string().contains("resignations_one_open") {
            unprocessable("you have already logged a resignation that is waiting to be accepted")
        } else {
            e.into()
        }
    })?;
    // The manager is told that it was logged. No reason, no date of leaving beyond what they need
    // to plan: just the fact.
    if let Some(manager) = worker.manager_pid {
        Notification::push(
            &txn,
            manager,
            "resignation_logged",
            &format!("{} has logged a resignation.", worker.display_name),
            json!({ "worker_pid": worker.pid, "resignation_pid": row.pid }),
        )
        .await?;
    }
    Audit::record(
        &txn,
        "worker",
        worker.pid,
        "resignation_logged",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(json!({
        "pid": row.pid,
        "earliest_last_day": rules::earliest_last_day(today, notice),
        "notice_calendar_days": notice,
    }))
}

/// Withdraw the worker's open resignation. Shared with `/api/me/resignation/withdraw`.
pub(crate) async fn withdraw_for(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    worker: &workers::Model,
) -> Result<Response> {
    let row = latest(ctx, worker.pid).await?.ok_or(Error::NotFound)?;
    let to = rules::next(status_of(&row)?, Action::Withdraw).map_err(|e| unprocessable(&e))?;
    let mut active: resignations::ActiveModel = row.into();
    active.status = ActiveValue::set(to.as_str().to_string());
    let saved = active.update(&ctx.db).await?;
    Audit::record(
        &ctx.db,
        "worker",
        worker.pid,
        "resignation_withdrawn",
        caller.actor(),
        None,
    )
    .await?;
    format::json(own_json(&saved))
}

async fn resignation_and_worker(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: &str,
) -> Result<(resignations::Model, workers::Model)> {
    let row = resignations::Entity::find()
        .filter(resignations::Column::Pid.eq(records::parse_pid(pid)?))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    let worker = records::find_worker(&ctx.db, row.worker_pid).await?;
    if let Some(refs) = memberships::scope_organization_refs(&ctx.db, caller.claims()).await?
        && !refs.iter().any(|r| r == &worker.organization_ref)
    {
        return Err(Error::NotFound);
    }
    auth::authorize_record(
        caller,
        authentication_verifier::Action::Write,
        &auth::worker_resource_attrs(&worker),
    )
    .map_err(record_rejection)?;
    Ok((row, worker))
}

/// `POST /api/resignations/{pid}/accept` body.
#[derive(Debug, Deserialize)]
struct AcceptPayload {
    /// The last day agreed; default the day the worker proposed.
    #[serde(default)]
    agreed_last_day: Option<NaiveDate>,
}

/// `POST /api/resignations/{pid}/accept` — a person accepts it: the agreed last day is recorded,
/// the leaver process is opened and the worker is told, in one transaction. Employment is not
/// ended by this.
#[debug_handler]
async fn accept(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<AcceptPayload>,
) -> Result<Response> {
    let (row, worker) = resignation_and_worker(&ctx, &caller, &pid).await?;
    let to = rules::next(status_of(&row)?, Action::Accept).map_err(|e| unprocessable(&e))?;
    let agreed = payload.agreed_last_day.unwrap_or(row.proposed_last_day);
    rules::validate_agreed(row.logged_on, agreed).map_err(|e| unprocessable(&e))?;
    let txn = ctx.db.begin().await?;
    let movement = open_in(
        &txn,
        &caller,
        &worker,
        MovementPayload {
            kind: "leaver".to_string(),
            effective_on: Some(agreed),
            reason: Some("resignation".to_string()),
            notes: None,
        },
    )
    .await?;
    let mut active: resignations::ActiveModel = row.clone().into();
    active.status = ActiveValue::set(to.as_str().to_string());
    active.agreed_last_day = ActiveValue::set(Some(agreed));
    active.movement_pid = ActiveValue::set(Some(movement.pid));
    active.decided_by = ActiveValue::set(caller.actor().map(ToString::to_string));
    active.decided_at = ActiveValue::set(Some(Utc::now().into()));
    let saved = active.update(&txn).await?;
    Notification::push(
        &txn,
        worker.pid,
        "resignation_decided",
        "Your resignation was accepted.",
        json!({ "resignation_pid": saved.pid }),
    )
    .await?;
    Audit::record(
        &txn,
        "worker",
        worker.pid,
        "resignation_accepted",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(json!({ "resignation": shared_json(&saved), "movement_pid": movement.pid }))
}

/// `POST /api/resignations/{pid}/rescind` — a person records that an accepted resignation was set
/// aside by agreement; the leaver record, if still open, is cancelled in the same transaction.
#[debug_handler]
async fn rescind(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let (row, worker) = resignation_and_worker(&ctx, &caller, &pid).await?;
    let to = rules::next(status_of(&row)?, Action::Rescind).map_err(|e| unprocessable(&e))?;
    let txn = ctx.db.begin().await?;
    if let Some(movement_pid) = row.movement_pid
        && let Some(movement) = movements::Entity::find()
            .filter(movements::Column::Pid.eq(movement_pid))
            .one(&txn)
            .await?
        && movement.status == "open"
    {
        let mut m: movements::ActiveModel = movement.into();
        m.status = ActiveValue::set("cancelled".to_string());
        m.update(&txn).await?;
    }
    let mut active: resignations::ActiveModel = row.into();
    active.status = ActiveValue::set(to.as_str().to_string());
    active.decided_by = ActiveValue::set(caller.actor().map(ToString::to_string));
    active.decided_at = ActiveValue::set(Some(Utc::now().into()));
    let saved = active.update(&txn).await?;
    Notification::push(
        &txn,
        worker.pid,
        "resignation_decided",
        "Your resignation was set aside.",
        json!({ "resignation_pid": saved.pid }),
    )
    .await?;
    Audit::record(
        &txn,
        "worker",
        worker.pid,
        "resignation_rescinded",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(shared_json(&saved))
}

/// Query for the list.
#[derive(Debug, Deserialize)]
struct ListQuery {
    /// Default `logged`.
    status: Option<String>,
}

/// `GET /api/resignations?status=` — resignations in the caller's organizations, soonest proposed
/// last day first. Privileged callers only. **No reason.**
#[debug_handler]
async fn list(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(query): axum::extract::Query<ListQuery>,
) -> Result<Response> {
    let status = query.status.unwrap_or_else(|| "logged".to_string());
    if Status::parse(&status).is_none() {
        return Err(unprocessable(
            "status must be logged, withdrawn, accepted or rescinded",
        ));
    }
    let rows = resignations::Entity::find()
        .filter(resignations::Column::Status.eq(&status))
        .order_by_asc(resignations::Column::ProposedLastDay)
        .all(&ctx.db)
        .await?;
    let scope = memberships::scope_organization_refs(&ctx.db, caller.claims()).await?;
    let mut out = Vec::new();
    for row in rows {
        let Some(worker) = workers::Entity::find()
            .filter(workers::Column::Pid.eq(row.worker_pid))
            .filter(workers::Column::DeletedAt.is_null())
            .one(&ctx.db)
            .await?
        else {
            continue;
        };
        if scope
            .as_ref()
            .is_some_and(|refs| !refs.iter().any(|r| r == &worker.organization_ref))
        {
            continue;
        }
        let mut item = shared_json(&row);
        item["worker_pid"] = json!(worker.pid);
        item["worker_number"] = json!(worker.worker_number);
        item["department"] = json!(worker.department);
        out.push(item);
    }
    format::json(out)
}

/// `GET /api/resignations/summary` — resignations by department and, where every cell is large
/// enough, by reason. Groups below the floor are withheld. Privileged callers only.
#[debug_handler]
async fn summary(State(ctx): State<AppContext>, caller: MaybeAuthUser) -> Result<Response> {
    let rows = resignations::Entity::find()
        .filter(resignations::Column::Status.is_in(["logged", "accepted"]))
        .all(&ctx.db)
        .await?;
    let scope = memberships::scope_organization_refs(&ctx.db, caller.claims()).await?;
    let mut counted: Vec<(String, Option<String>)> = Vec::new();
    for row in rows {
        let Some(worker) = workers::Entity::find()
            .filter(workers::Column::Pid.eq(row.worker_pid))
            .one(&ctx.db)
            .await?
        else {
            continue;
        };
        if scope
            .as_ref()
            .is_some_and(|refs| !refs.iter().any(|r| r == &worker.organization_ref))
        {
            continue;
        }
        counted.push((worker.department, row.reason));
    }
    let departments: Vec<serde_json::Value> = rules::summarise(&counted, K_ANONYMITY)
        .into_iter()
        .map(|d| json!({ "department": d.department, "count": d.count, "reasons": d.reasons }))
        .collect();
    format::json(json!({
        "floor": K_ANONYMITY,
        "counted": "logged and accepted resignations",
        "derivation": "A department's count shows only when it reaches the floor, and its split by reason only when every reason also reaches it, so no withheld figure can be worked out from the others.",
        "departments": departments,
    }))
}

/// `GET /api/workers/{pid}/resignation` — for the worker's line managers and HR: whether one is
/// open and the dates. **Never the reason.**
#[debug_handler]
async fn worker_resignation(
    State(ctx): State<AppContext>,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    let row = latest(&ctx, worker.pid).await?;
    format::json(json!({ "resignation": row.as_ref().map(shared_json) }))
}

/// The resignation routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/resignations", get(list))
        .add("/resignations/summary", get(summary))
        .add("/resignations/{pid}/accept", post(accept))
        .add("/resignations/{pid}/rescind", post(rescind))
        .add("/workers/{pid}/resignation", get(worker_resignation))
}
