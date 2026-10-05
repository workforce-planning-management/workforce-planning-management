//! **Emergency contacts** and **backups** — two things a person provides
//! about themselves.
//!
//! - **Emergency contacts** (`rules::emergency`): people to reach if
//!   something happens. Third-party personal data, so only the worker and
//!   whoever may write to their record (HR) can read or change them — a
//!   manager cannot, by default.
//! - **Backups** (`rules::cover`): the colleague(s) who cover when the
//!   worker is out sick or on leave, ranked, optionally for a dated window.
//!   Anyone who can see the worker can see their backups (colleagues need
//!   to know who to ask); the worker and HR edit them. `…/cover?on=` says
//!   who actually covers on a day — skipping a backup who is on approved
//!   leave or has left.
//!
//! Both follow the aspirations pattern: edits record who made them and
//! whether on the person's behalf, and write an audit entry.

use chrono::{NaiveDate, Utc};
use loco_rs::prelude::*;
use sea_orm::{ActiveValue, QueryOrder};
use serde::Deserialize;
use std::collections::HashMap;
use uuid::Uuid;

use super::{record_rejection, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::models::_entities::{emergency_contacts, leave_requests, worker_backups, workers};
use crate::models::audit_logs::Model as Audit;
use crate::models::{memberships, records};
use crate::rules::cover::{self as cover_rules, Candidate};
use crate::rules::emergency as rules;
use crate::rules::metrics as metric_rules;

/// A worker, authorized for a write to their record: the worker themself or HR.
async fn writable_worker(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: &str,
) -> Result<workers::Model> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(pid)?).await?;
    auth::authorize_record(
        caller,
        authentication_verifier::Action::Write,
        &auth::worker_resource_attrs(&worker),
    )
    .map_err(record_rejection)?;
    Ok(worker)
}

/// Trimmed, with a blank treated as absent.
fn clean(value: Option<String>) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

// ─── Emergency contacts ─────────────────────────────────────────────────────

fn contact_row(c: &emergency_contacts::Model) -> serde_json::Value {
    serde_json::json!({
        "pid": c.pid,
        "name": c.name,
        "relationship": c.relationship,
        "phone": c.phone,
        "alt_phone": c.alt_phone,
        "email": c.email,
        "priority": c.priority,
        "note": c.note,
        "on_behalf": c.on_behalf,
    })
}

/// `GET /api/workers/{pid}/emergency-contacts` — the worker's contacts,
/// first-to-call first. Only the worker and HR (anyone who may write the
/// record) can read them.
#[debug_handler]
async fn list_contacts(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = writable_worker(&ctx, &caller, &pid).await?;
    let rows = emergency_contacts::Entity::find()
        .filter(emergency_contacts::Column::WorkerPid.eq(worker.pid))
        .filter(emergency_contacts::Column::DeletedAt.is_null())
        .order_by_asc(emergency_contacts::Column::Priority)
        .order_by_asc(emergency_contacts::Column::Id)
        .all(&ctx.db)
        .await?;
    format::json(rows.iter().map(contact_row).collect::<Vec<_>>())
}

/// `POST` body.
#[derive(Debug, Deserialize)]
struct ContactPayload {
    name: String,
    relationship: String,
    phone: String,
    #[serde(default)]
    alt_phone: Option<String>,
    #[serde(default)]
    email: Option<String>,
    /// 1 is the first person to call; default: after the existing contacts.
    #[serde(default)]
    priority: Option<i32>,
    #[serde(default)]
    note: Option<String>,
}

/// `POST /api/workers/{pid}/emergency-contacts` — add a contact (up to
/// [`rules::MAX_CONTACTS`]).
#[debug_handler]
async fn add_contact(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<ContactPayload>,
) -> Result<Response> {
    let worker = writable_worker(&ctx, &caller, &pid).await?;
    let existing = emergency_contacts::Entity::find()
        .filter(emergency_contacts::Column::WorkerPid.eq(worker.pid))
        .filter(emergency_contacts::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?;
    if existing.len() >= rules::MAX_CONTACTS {
        return Err(unprocessable(&format!(
            "at most {} emergency contacts",
            rules::MAX_CONTACTS
        )));
    }
    let next = i32::try_from(existing.len()).unwrap_or(i32::MAX).saturating_add(1);
    let priority = payload.priority.unwrap_or(next);
    let alt_phone = clean(payload.alt_phone);
    let email = clean(payload.email);
    rules::validate_contact(
        payload.name.trim(),
        payload.relationship.trim(),
        payload.phone.trim(),
        alt_phone.as_deref(),
        email.as_deref(),
        priority,
    )
    .map_err(|e| unprocessable(&e))?;
    let row = emergency_contacts::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        worker_pid: ActiveValue::set(worker.pid),
        name: ActiveValue::set(payload.name.trim().to_string()),
        relationship: ActiveValue::set(payload.relationship.trim().to_string()),
        phone: ActiveValue::set(payload.phone.trim().to_string()),
        alt_phone: ActiveValue::set(alt_phone),
        email: ActiveValue::set(email),
        priority: ActiveValue::set(priority),
        note: ActiveValue::set(clean(payload.note)),
        recorded_by: ActiveValue::set(caller.actor().map(ToString::to_string)),
        on_behalf: ActiveValue::set(auth::acting_for_other(&caller, &worker.person_ref)),
        deleted_at: ActiveValue::set(None),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;
    // The audit entry names no contact detail: that is the point of keeping it
    // to the worker and HR.
    Audit::record(
        &ctx.db,
        "worker",
        worker.pid,
        "emergency_contact_added",
        caller.actor(),
        None,
    )
    .await?;
    format::json(serde_json::json!({ "pid": row.pid }))
}

/// `PUT` body — any of these may change.
#[derive(Debug, Deserialize)]
struct ContactUpdate {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    relationship: Option<String>,
    #[serde(default)]
    phone: Option<String>,
    #[serde(default)]
    alt_phone: Option<String>,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    priority: Option<i32>,
    #[serde(default)]
    note: Option<String>,
}

async fn find_contact(ctx: &AppContext, pid: &str) -> Result<emergency_contacts::Model> {
    emergency_contacts::Entity::find()
        .filter(emergency_contacts::Column::Pid.eq(records::parse_pid(pid)?))
        .filter(emergency_contacts::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)
}

/// `PUT /api/emergency-contacts/{pid}` — change a contact's details.
#[debug_handler]
async fn update_contact(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<ContactUpdate>,
) -> Result<Response> {
    let row = find_contact(&ctx, &pid).await?;
    let worker = writable_worker(&ctx, &caller, &row.worker_pid.to_string()).await?;
    let name = payload.name.unwrap_or_else(|| row.name.clone());
    let relationship = payload
        .relationship
        .unwrap_or_else(|| row.relationship.clone());
    let phone = payload.phone.unwrap_or_else(|| row.phone.clone());
    let alt_phone = payload.alt_phone.map_or_else(|| row.alt_phone.clone(), |v| clean(Some(v)));
    let email = payload.email.map_or_else(|| row.email.clone(), |v| clean(Some(v)));
    let priority = payload.priority.unwrap_or(row.priority);
    rules::validate_contact(
        name.trim(),
        relationship.trim(),
        phone.trim(),
        alt_phone.as_deref(),
        email.as_deref(),
        priority,
    )
    .map_err(|e| unprocessable(&e))?;
    let note = payload.note.map_or_else(|| row.note.clone(), |v| clean(Some(v)));
    let mut active: emergency_contacts::ActiveModel = row.into();
    active.name = ActiveValue::set(name.trim().to_string());
    active.relationship = ActiveValue::set(relationship.trim().to_string());
    active.phone = ActiveValue::set(phone.trim().to_string());
    active.alt_phone = ActiveValue::set(alt_phone);
    active.email = ActiveValue::set(email);
    active.priority = ActiveValue::set(priority);
    active.note = ActiveValue::set(note);
    let updated = active.update(&ctx.db).await?;
    Audit::record(
        &ctx.db,
        "worker",
        worker.pid,
        "emergency_contact_updated",
        caller.actor(),
        None,
    )
    .await?;
    format::json(contact_row(&updated))
}

/// `DELETE /api/emergency-contacts/{pid}` — remove a contact (soft-delete).
#[debug_handler]
async fn delete_contact(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let row = find_contact(&ctx, &pid).await?;
    let worker = writable_worker(&ctx, &caller, &row.worker_pid.to_string()).await?;
    let mut active: emergency_contacts::ActiveModel = row.into();
    active.deleted_at = ActiveValue::set(Some(Utc::now().into()));
    active.update(&ctx.db).await?;
    Audit::record(
        &ctx.db,
        "worker",
        worker.pid,
        "emergency_contact_removed",
        caller.actor(),
        None,
    )
    .await?;
    format::empty_json()
}

// ─── Backups ────────────────────────────────────────────────────────────────

/// A live worker by pid, or 422 (a backup that does not exist is a bad
/// request, not a missing page).
async fn live_worker(ctx: &AppContext, pid: Uuid) -> Result<workers::Model> {
    records::find_worker(&ctx.db, pid)
        .await
        .map_err(|_| unprocessable("that backup is not a worker here"))
}

fn backup_row(b: &worker_backups::Model, who: Option<&workers::Model>) -> serde_json::Value {
    serde_json::json!({
        "pid": b.pid,
        "backup_pid": b.backup_pid,
        "backup_name": who.map(|w| w.display_name.clone()),
        "backup_title": who.map(|w| w.job_title.clone()),
        "priority": b.priority,
        "starts_on": b.starts_on,
        "ends_on": b.ends_on,
        "note": b.note,
        "on_behalf": b.on_behalf,
    })
}

/// The workers behind a set of backup rows, by pid.
async fn backup_workers(
    ctx: &AppContext,
    rows: &[worker_backups::Model],
) -> Result<HashMap<Uuid, workers::Model>> {
    let ids: Vec<Uuid> = rows.iter().map(|b| b.backup_pid).collect();
    Ok(workers::Entity::find()
        .filter(workers::Column::Pid.is_in(ids))
        .filter(workers::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|w| (w.pid, w))
        .collect())
}

async fn live_backups(ctx: &AppContext, worker: Uuid) -> Result<Vec<worker_backups::Model>> {
    Ok(worker_backups::Entity::find()
        .filter(worker_backups::Column::WorkerPid.eq(worker))
        .filter(worker_backups::Column::DeletedAt.is_null())
        .order_by_asc(worker_backups::Column::Priority)
        .order_by_asc(worker_backups::Column::Id)
        .all(&ctx.db)
        .await?)
}

/// `GET /api/workers/{pid}/backups` — who covers for this worker, in the
/// order to ask. Readable by anyone who can read the worker.
#[debug_handler]
async fn list_backups(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    auth::authorize_record(
        &caller,
        authentication_verifier::Action::Read,
        &auth::worker_resource_attrs(&worker),
    )
    .map_err(record_rejection)?;
    let rows = live_backups(&ctx, worker.pid).await?;
    let who = backup_workers(&ctx, &rows).await?;
    format::json(
        rows.iter()
            .map(|b| backup_row(b, who.get(&b.backup_pid)))
            .collect::<Vec<_>>(),
    )
}

/// `POST` body.
#[derive(Debug, Deserialize)]
struct BackupPayload {
    backup_pid: Uuid,
    /// 1 is the first backup to ask; default: after the existing ones.
    #[serde(default)]
    priority: Option<i32>,
    #[serde(default)]
    starts_on: Option<NaiveDate>,
    #[serde(default)]
    ends_on: Option<NaiveDate>,
    #[serde(default)]
    note: Option<String>,
}

/// `POST /api/workers/{pid}/backups` — name a backup (up to
/// [`cover_rules::MAX_BACKUPS`]). The backup must be a colleague employed
/// today in an organization the caller can read, and not already named.
#[debug_handler]
async fn add_backup(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<BackupPayload>,
) -> Result<Response> {
    let worker = writable_worker(&ctx, &caller, &pid).await?;
    let existing = live_backups(&ctx, worker.pid).await?;
    if existing.len() >= cover_rules::MAX_BACKUPS {
        return Err(unprocessable(&format!(
            "at most {} backups",
            cover_rules::MAX_BACKUPS
        )));
    }
    if existing.iter().any(|b| b.backup_pid == payload.backup_pid) {
        return Err(unprocessable("that person is already a backup"));
    }
    let next = i32::try_from(existing.len()).unwrap_or(i32::MAX).saturating_add(1);
    let priority = payload.priority.unwrap_or(next);
    cover_rules::validate_backup(
        worker.pid,
        payload.backup_pid,
        priority,
        payload.starts_on,
        payload.ends_on,
    )
    .map_err(|e| unprocessable(&e))?;
    let backup = live_worker(&ctx, payload.backup_pid).await?;
    let today = Utc::now().date_naive();
    if !metric_rules::is_employed_on(today, backup.hired_on, backup.terminated_on) {
        return Err(unprocessable("that backup is not currently employed"));
    }
    if let Some(refs) = memberships::scope_organization_refs(&ctx.db, caller.claims()).await?
        && !refs.contains(&backup.organization_ref)
    {
        return Err(unprocessable("that backup is outside your organizations"));
    }
    let row = worker_backups::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        worker_pid: ActiveValue::set(worker.pid),
        backup_pid: ActiveValue::set(backup.pid),
        priority: ActiveValue::set(priority),
        starts_on: ActiveValue::set(payload.starts_on),
        ends_on: ActiveValue::set(payload.ends_on),
        note: ActiveValue::set(clean(payload.note)),
        recorded_by: ActiveValue::set(caller.actor().map(ToString::to_string)),
        on_behalf: ActiveValue::set(auth::acting_for_other(&caller, &worker.person_ref)),
        deleted_at: ActiveValue::set(None),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;
    Audit::record(
        &ctx.db,
        "worker",
        worker.pid,
        "backup_added",
        caller.actor(),
        Some(serde_json::json!({ "backup_pid": backup.pid })),
    )
    .await?;
    format::json(serde_json::json!({ "pid": row.pid }))
}

/// `PUT` body — rank, window or note.
#[derive(Debug, Deserialize)]
struct BackupUpdate {
    #[serde(default)]
    priority: Option<i32>,
    #[serde(default)]
    starts_on: Option<NaiveDate>,
    #[serde(default)]
    ends_on: Option<NaiveDate>,
    #[serde(default)]
    note: Option<String>,
}

async fn find_backup(ctx: &AppContext, pid: &str) -> Result<worker_backups::Model> {
    worker_backups::Entity::find()
        .filter(worker_backups::Column::Pid.eq(records::parse_pid(pid)?))
        .filter(worker_backups::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)
}

/// `PUT /api/backups/{pid}` — change a backup's rank, window or note.
#[debug_handler]
async fn update_backup(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<BackupUpdate>,
) -> Result<Response> {
    let row = find_backup(&ctx, &pid).await?;
    let worker = writable_worker(&ctx, &caller, &row.worker_pid.to_string()).await?;
    let priority = payload.priority.unwrap_or(row.priority);
    let starts_on = payload.starts_on.or(row.starts_on);
    let ends_on = payload.ends_on.or(row.ends_on);
    cover_rules::validate_backup(worker.pid, row.backup_pid, priority, starts_on, ends_on)
        .map_err(|e| unprocessable(&e))?;
    let note = payload.note.map_or_else(|| row.note.clone(), |v| clean(Some(v)));
    let mut active: worker_backups::ActiveModel = row.into();
    active.priority = ActiveValue::set(priority);
    active.starts_on = ActiveValue::set(starts_on);
    active.ends_on = ActiveValue::set(ends_on);
    active.note = ActiveValue::set(note);
    let updated = active.update(&ctx.db).await?;
    Audit::record(
        &ctx.db,
        "worker",
        worker.pid,
        "backup_updated",
        caller.actor(),
        None,
    )
    .await?;
    let who = records::find_worker(&ctx.db, updated.backup_pid).await.ok();
    format::json(backup_row(&updated, who.as_ref()))
}

/// `DELETE /api/backups/{pid}` — stop naming this backup (soft-delete).
#[debug_handler]
async fn delete_backup(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let row = find_backup(&ctx, &pid).await?;
    let worker = writable_worker(&ctx, &caller, &row.worker_pid.to_string()).await?;
    let mut active: worker_backups::ActiveModel = row.into();
    active.deleted_at = ActiveValue::set(Some(Utc::now().into()));
    active.update(&ctx.db).await?;
    Audit::record(
        &ctx.db,
        "worker",
        worker.pid,
        "backup_removed",
        caller.actor(),
        None,
    )
    .await?;
    format::empty_json()
}

/// Query for the cover view.
#[derive(Debug, Deserialize)]
struct CoverQuery {
    /// The day to resolve (default today).
    on: Option<NaiveDate>,
}

/// `GET /api/workers/{pid}/cover?on=` — who covers for this worker on a
/// day: the best-ranked backup whose window includes it, who is employed
/// and not on approved leave. `covered_by` is null when nobody can — the
/// view says so rather than guessing.
#[debug_handler]
async fn cover(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    axum::extract::Query(query): axum::extract::Query<CoverQuery>,
) -> Result<Response> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    auth::authorize_record(
        &caller,
        authentication_verifier::Action::Read,
        &auth::worker_resource_attrs(&worker),
    )
    .map_err(record_rejection)?;
    let on = query.on.unwrap_or_else(|| Utc::now().date_naive());
    let rows = live_backups(&ctx, worker.pid).await?;
    let who = backup_workers(&ctx, &rows).await?;
    let ids: Vec<Uuid> = rows.iter().map(|b| b.backup_pid).collect();
    let on_leave: Vec<Uuid> = leave_requests::Entity::find()
        .filter(leave_requests::Column::WorkerPid.is_in(ids))
        .filter(leave_requests::Column::Status.eq("approved"))
        .filter(leave_requests::Column::DeletedAt.is_null())
        .filter(leave_requests::Column::StartOn.lte(on))
        .filter(leave_requests::Column::EndOn.gte(on))
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|l| l.worker_pid)
        .collect();
    let candidates: Vec<Candidate> = rows
        .iter()
        .map(|b| {
            let employed = who.get(&b.backup_pid).is_some_and(|w| {
                metric_rules::is_employed_on(on, w.hired_on, w.terminated_on)
            });
            Candidate {
                backup: b.backup_pid,
                priority: b.priority,
                starts_on: b.starts_on,
                ends_on: b.ends_on,
                employed,
                on_leave: on_leave.contains(&b.backup_pid),
            }
        })
        .collect();
    let covering = cover_rules::resolve(&candidates, on);
    let person = covering.and_then(|id| who.get(&id));
    format::json(serde_json::json!({
        "on": on,
        "worker_pid": worker.pid,
        "covered_by": covering,
        "covered_by_name": person.map(|w| w.display_name.clone()),
        "covered_by_title": person.map(|w| w.job_title.clone()),
        "backups_named": rows.len(),
    }))
}

/// The emergency-contact and backup routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/workers/{pid}/emergency-contacts", get(list_contacts))
        .add("/workers/{pid}/emergency-contacts", post(add_contact))
        .add("/emergency-contacts/{pid}", put(update_contact))
        .add("/emergency-contacts/{pid}", delete(delete_contact))
        .add("/workers/{pid}/backups", get(list_backups))
        .add("/workers/{pid}/backups", post(add_backup))
        .add("/backups/{pid}", put(update_backup))
        .add("/backups/{pid}", delete(delete_backup))
        .add("/workers/{pid}/cover", get(cover))
}
