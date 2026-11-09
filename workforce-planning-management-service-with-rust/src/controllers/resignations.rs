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

use super::ensure_org_in_scope;
use axa::extract::{Json, Path};
use axa::prelude::*;
use chrono::{NaiveDate, Utc};
// `accept` opens the leaver record through `movements::open_in`, a SeaORM transaction, so it stays on SeaORM until
// `movements` moves (WPM-T466).
use sea_orm::{ActiveValue, TransactionTrait};
use serde::Deserialize;
use serde_json::json;

use super::movements::{MovementPayload, open_in};
use super::{record_rejection, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::db;
use crate::models::_entities::{resignations, workers};
use crate::models::audit_logs::Model as Audit;
use crate::models::notifications::Model as Notification;
use crate::models::{memberships, records};
use crate::rules::pulse::K_ANONYMITY;
use crate::rules::resignation::{self as rules, Action, Status};

use super::jurisdictions::Scope;
use super::statutory_defaults::{self as statutory, InForce};

/// The notice period in calendar days: the worker's profile (`resignation.notice_calendar_days`),
/// else `WPM_RESIGNATION_NOTICE_CALENDAR_DAYS`, else 28. An invalid setting is ignored with a warning,
/// never silently shortened.
#[must_use]
pub fn notice_days(scope: Option<&Scope>, as_of: NaiveDate) -> InForce<i64> {
    let name = "WPM_RESIGNATION_NOTICE_CALENDAR_DAYS";
    let configured =
        crate::compat::env_var(name).and_then(|text| match text.trim().parse::<i64>() {
            Ok(n) if rules::validate_notice(n).is_ok() => Some(n),
            _ => {
                tracing::warn!("{name} is not 0 to 366; ignored");
                None
            }
        });
    statutory::integer(
        scope,
        name,
        as_of,
        configured,
        rules::DEFAULT_NOTICE_CALENDAR_DAYS,
    )
}

/// [`notice_days`] for `worker`, over their own chain.
pub(crate) async fn notice_for(ctx: &AppContext, worker: &workers::Model) -> Result<InForce<i64>> {
    let scope = Scope::for_worker(crate::db::pool(ctx), worker).await?;
    Ok(notice_days(Some(&scope), Utc::now().date_naive()))
}

/// The worker's latest resignation of any status.
pub(crate) async fn latest(
    ctx: &AppContext,
    worker_pid: uuid::Uuid,
) -> Result<Option<resignations::Model>> {
    Ok(sqlx::query_as!(
        resignations::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, logged_on,
                  proposed_last_day, reason, status, agreed_last_day, movement_pid, decided_by, decided_at AS "decided_at: _"
             FROM resignations WHERE worker_pid = $1 ORDER BY id DESC LIMIT 1"#,
        worker_pid,
    )
    .fetch_optional(db::pool(ctx))
    .await?)
}

fn status_of(row: &resignations::Model) -> Result<Status> {
    Status::parse(&row.status).ok_or_else(|| super::internal("a resignation has an unknown status"))
}

/// What the worker themself sees, with their reason and the notice that applies to them.
pub(crate) fn own_json(row: &resignations::Model, notice: &InForce<i64>) -> serde_json::Value {
    json!({
        "pid": row.pid,
        "status": row.status,
        "logged_on": row.logged_on,
        "proposed_last_day": row.proposed_last_day,
        "agreed_last_day": row.agreed_last_day,
        "reason": row.reason,
        "notice_calendar_days": notice.value,
        // Where the notice came from: the worker's profile, the setting or the default (WPM-D153).
        "notice_basis": notice.came_from,
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
    let notice = notice_for(ctx, worker).await?;
    rules::validate_proposal(today, payload.proposed_last_day, notice.value)
        .map_err(|e| unprocessable(&e))?;
    let pid = uuid::Uuid::new_v4();
    let mut tx = db::pool(ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO resignations (pid, worker_pid, logged_on, proposed_last_day, reason, status) VALUES ($1, $2, $3, $4, $5, $6)",
        pid,
        worker.pid,
        today,
        payload.proposed_last_day,
        payload.reason,
        Status::Logged.as_str(),
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        if e.as_database_error().and_then(|d| d.constraint()) == Some("resignations_one_open") {
            unprocessable("you have already logged a resignation that is waiting to be accepted")
        } else {
            e.into()
        }
    })?;
    // The manager is told that it was logged. No reason, no date of leaving beyond what they need
    // to plan: just the fact.
    if let Some(manager) = worker.manager_pid {
        db::notify(
            &mut tx,
            manager,
            "resignation_logged",
            &format!("{} has logged a resignation.", worker.display_name),
            json!({ "worker_pid": worker.pid, "resignation_pid": pid }),
        )
        .await?;
    }
    db::audit(
        &mut tx,
        "worker",
        worker.pid,
        "resignation_logged",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(json!({
        "pid": pid,
        "earliest_last_day": rules::earliest_last_day(today, notice.value),
        "notice_calendar_days": notice.value,
        "notice_basis": notice.came_from,
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
    let mut tx = db::pool(ctx).begin().await?;
    let saved = sqlx::query_as!(
        resignations::Model,
        r#"UPDATE resignations SET status = $2 WHERE id = $1
           RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, logged_on,
                     proposed_last_day, reason, status, agreed_last_day, movement_pid, decided_by, decided_at AS "decided_at: _""#,
        row.id,
        to.as_str(),
    )
    .fetch_one(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "worker",
        worker.pid,
        "resignation_withdrawn",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(own_json(&saved, &notice_for(ctx, worker).await?))
}

async fn resignation_and_worker(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: &str,
) -> Result<(resignations::Model, workers::Model)> {
    let row = sqlx::query_as!(
        resignations::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, logged_on,
                  proposed_last_day, reason, status, agreed_last_day, movement_pid, decided_by, decided_at AS "decided_at: _"
             FROM resignations WHERE pid = $1"#,
        records::parse_pid(pid)?,
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)?;
    let worker = records::find_worker(&ctx.db, row.worker_pid).await?;
    ensure_org_in_scope(ctx, caller, &worker.organization_ref).await?;
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
    let mut tx = db::pool(&ctx).begin().await?;
    // The leaver record, if still open, is cancelled; a closed one is left as it is.
    if let Some(movement_pid) = row.movement_pid {
        sqlx::query!(
            "UPDATE movements SET status = 'cancelled' WHERE pid = $1 AND status = 'open'",
            movement_pid
        )
        .execute(&mut *tx)
        .await?;
    }
    let saved = sqlx::query_as!(
        resignations::Model,
        r#"UPDATE resignations SET status = $2, decided_by = $3, decided_at = $4 WHERE id = $1
           RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, logged_on,
                     proposed_last_day, reason, status, agreed_last_day, movement_pid, decided_by, decided_at AS "decided_at: _""#,
        row.id,
        to.as_str(),
        caller.actor(),
        Utc::now(),
    )
    .fetch_one(&mut *tx)
    .await?;
    db::notify(
        &mut tx,
        worker.pid,
        "resignation_decided",
        "Your resignation was set aside.",
        json!({ "resignation_pid": saved.pid }),
    )
    .await?;
    db::audit(
        &mut tx,
        "worker",
        worker.pid,
        "resignation_rescinded",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
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
    axa::extract::Query(query): axa::extract::Query<ListQuery>,
    page: super::Page,
) -> Result<Response> {
    let window = super::window(&page)?;
    let status = query.status.unwrap_or_else(|| "logged".to_string());
    if Status::parse(&status).is_none() {
        return Err(unprocessable(
            "status must be logged, withdrawn, accepted or rescinded",
        ));
    }
    let scope = memberships::scope_organization_refs(&ctx.db, caller.claims()).await?;
    // One read with its workers, in the caller's organizations; a removed worker's is left out.
    let out: Vec<serde_json::Value> = sqlx::query!(
        "SELECT r.pid, r.status, r.logged_on, r.proposed_last_day, r.agreed_last_day,
                w.pid AS worker_pid, w.worker_number, w.department
           FROM resignations r JOIN workers w ON w.pid = r.worker_pid AND w.deleted_at IS NULL
          WHERE r.status = $1 AND ($2::text[] IS NULL OR w.organization_ref = ANY($2))
          ORDER BY r.proposed_last_day, r.id",
        status,
        scope.as_deref(),
    )
    .fetch_all(db::pool(&ctx))
    .await?
    .into_iter()
    .map(|r| {
        json!({
            "pid": r.pid,
            "status": r.status,
            "logged_on": r.logged_on,
            "proposed_last_day": r.proposed_last_day,
            "agreed_last_day": r.agreed_last_day,
            "worker_pid": r.worker_pid,
            "worker_number": r.worker_number,
            "department": r.department,
        })
    })
    .collect();
    Ok(super::paged_vec(out, &window))
}

/// `GET /api/resignations/summary` — resignations by department and, where every cell is large
/// enough, by reason. Groups below the floor are withheld. Privileged callers only.
#[debug_handler]
async fn summary(State(ctx): State<AppContext>, caller: MaybeAuthUser) -> Result<Response> {
    let scope = memberships::scope_organization_refs(&ctx.db, caller.claims()).await?;
    let counted: Vec<(String, Option<String>)> = sqlx::query!(
        "SELECT w.department, r.reason
           FROM resignations r JOIN workers w ON w.pid = r.worker_pid
          WHERE r.status IN ('logged', 'accepted') AND ($1::text[] IS NULL OR w.organization_ref = ANY($1))",
        scope.as_deref(),
    )
    .fetch_all(db::pool(&ctx))
    .await?
    .into_iter()
    .map(|r| (r.department, r.reason))
    .collect();
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
