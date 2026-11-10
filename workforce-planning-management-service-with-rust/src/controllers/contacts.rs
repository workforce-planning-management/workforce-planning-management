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

// An emergency contact's number is not read in the worker's country (the contact may live elsewhere): only a number
// written with `+` has an E.164 form (WPM-D152).
use super::writable_worker;
use crate::rules::phone as phone_rules;
use axa::extract::{Json, Path};
use axa::prelude::*;
use chrono::{NaiveDate, Utc};
use serde::Deserialize;
use std::collections::HashMap;
use uuid::Uuid;

use super::{record_rejection, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::db;
use crate::models::_entities::{emergency_contacts, worker_backups, workers};
use crate::models::{memberships, records};
use crate::rules::cover::{self as cover_rules, Candidate};
use crate::rules::emergency as rules;
use crate::rules::metrics as metric_rules;

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
        "phone_digits": c.phone_digits,
        "phone_e164": c.phone_e164,
        "alt_phone_digits": c.alt_phone_digits,
        "alt_phone_e164": c.alt_phone_e164,
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
    list_contacts_for(&ctx, &worker).await
}

/// The worker's contacts, first-to-call first. Shared by the HR route and `/api/me`.
pub(crate) async fn list_contacts_for(
    ctx: &AppContext,
    worker: &workers::Model,
) -> Result<Response> {
    let rows = live_contacts(ctx, worker.pid).await?;
    format::json(rows.iter().map(contact_row).collect::<Vec<_>>())
}

/// The worker's live contacts, first-to-call first.
async fn live_contacts(ctx: &AppContext, worker: Uuid) -> Result<Vec<emergency_contacts::Model>> {
    Ok(sqlx::query_as!(
        emergency_contacts::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, name, relationship,
                  phone, alt_phone, email, priority, note, recorded_by, on_behalf, deleted_at AS "deleted_at: _",
                  phone_digits, phone_e164, alt_phone_digits, alt_phone_e164
             FROM emergency_contacts WHERE worker_pid = $1 AND deleted_at IS NULL ORDER BY priority, id"#,
        worker,
    )
    .fetch_all(db::pool(ctx))
    .await?)
}

/// `POST` body.
#[derive(Debug, Deserialize)]
pub(crate) struct ContactPayload {
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
    add_contact_for(&ctx, &caller, &worker, payload).await
}

/// Add a contact to this worker. Shared by the HR route and `/api/me`.
pub(crate) async fn add_contact_for(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    worker: &workers::Model,
    payload: ContactPayload,
) -> Result<Response> {
    let existing = live_contacts(ctx, worker.pid).await?;
    if existing.len() >= rules::MAX_CONTACTS {
        return Err(unprocessable(&format!(
            "at most {} emergency contacts",
            rules::MAX_CONTACTS
        )));
    }
    let next = i32::try_from(existing.len())
        .unwrap_or(i32::MAX)
        .saturating_add(1);
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
    let pid = Uuid::new_v4();
    let phone = payload.phone.trim();
    let mut tx = db::pool(ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO emergency_contacts (pid, worker_pid, name, relationship, phone, phone_digits, phone_e164,
                                         alt_phone_digits, alt_phone_e164, alt_phone, email, priority, note, recorded_by,
                                         on_behalf, deleted_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, NULL)",
        pid,
        worker.pid,
        payload.name.trim(),
        payload.relationship.trim(),
        phone,
        phone_rules::ascii_digits(phone),
        phone_rules::e164(phone, None),
        alt_phone.as_deref().and_then(phone_rules::ascii_digits),
        alt_phone
            .as_deref()
            .and_then(|p| phone_rules::e164(p, None)),
        alt_phone,
        email,
        priority,
        clean(payload.note),
        caller.actor(),
        auth::acting_for_other(caller, &worker.person_ref),
    )
    .execute(&mut *tx)
    .await?;
    // The audit entry names no contact detail: that is the point of keeping it
    // to the worker and HR.
    db::audit(
        &mut tx,
        "worker",
        worker.pid,
        "emergency_contact_added",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(serde_json::json!({ "pid": pid }))
}

/// `PUT` body — any of these may change.
#[derive(Debug, Deserialize)]
pub(crate) struct ContactUpdate {
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

pub(crate) async fn find_contact(ctx: &AppContext, pid: &str) -> Result<emergency_contacts::Model> {
    sqlx::query_as!(
        emergency_contacts::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, name, relationship,
                  phone, alt_phone, email, priority, note, recorded_by, on_behalf, deleted_at AS "deleted_at: _",
                  phone_digits, phone_e164, alt_phone_digits, alt_phone_e164
             FROM emergency_contacts WHERE pid = $1 AND deleted_at IS NULL"#,
        records::parse_pid(pid)?,
    )
    .fetch_optional(db::pool(ctx))
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
    update_contact_for(&ctx, &caller, &worker, row, payload).await
}

/// Change a contact of this worker. Shared by the HR route and `/api/me`.
pub(crate) async fn update_contact_for(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    worker: &workers::Model,
    row: emergency_contacts::Model,
    payload: ContactUpdate,
) -> Result<Response> {
    let name = payload.name.unwrap_or_else(|| row.name.clone());
    let relationship = payload
        .relationship
        .unwrap_or_else(|| row.relationship.clone());
    let phone = payload.phone.unwrap_or_else(|| row.phone.clone());
    let alt_phone = payload
        .alt_phone
        .map_or_else(|| row.alt_phone.clone(), |v| clean(Some(v)));
    let email = payload
        .email
        .map_or_else(|| row.email.clone(), |v| clean(Some(v)));
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
    let note = payload
        .note
        .map_or_else(|| row.note.clone(), |v| clean(Some(v)));
    let phone = phone.trim();
    let mut tx = db::pool(ctx).begin().await?;
    let updated = sqlx::query_as!(
        emergency_contacts::Model,
        r#"UPDATE emergency_contacts
              SET name = $2, relationship = $3, phone = $4, phone_digits = $5, phone_e164 = $6, alt_phone_digits = $7,
                  alt_phone_e164 = $8, alt_phone = $9, email = $10, priority = $11, note = $12
            WHERE id = $1
        RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, name, relationship,
                  phone, alt_phone, email, priority, note, recorded_by, on_behalf, deleted_at AS "deleted_at: _",
                  phone_digits, phone_e164, alt_phone_digits, alt_phone_e164"#,
        row.id,
        name.trim(),
        relationship.trim(),
        phone,
        phone_rules::ascii_digits(phone),
        phone_rules::e164(phone, None),
        alt_phone.as_deref().and_then(phone_rules::ascii_digits),
        alt_phone
            .as_deref()
            .and_then(|p| phone_rules::e164(p, None)),
        alt_phone,
        email,
        priority,
        note,
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(Error::NotFound)?;
    db::audit(
        &mut tx,
        "worker",
        worker.pid,
        "emergency_contact_updated",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
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
    delete_contact_for(&ctx, &caller, &worker, row).await
}

/// Remove a contact of this worker. Shared by the HR route and `/api/me`.
pub(crate) async fn delete_contact_for(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    worker: &workers::Model,
    row: emergency_contacts::Model,
) -> Result<Response> {
    let mut tx = db::pool(ctx).begin().await?;
    sqlx::query!(
        "UPDATE emergency_contacts SET deleted_at = now() WHERE id = $1",
        row.id
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "worker",
        worker.pid,
        "emergency_contact_removed",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::empty_json()
}

// ─── Backups ────────────────────────────────────────────────────────────────

/// A live worker by pid, or 422 (a backup that does not exist is a bad
/// request, not a missing page).
async fn live_worker(ctx: &AppContext, pid: Uuid) -> Result<workers::Model> {
    records::worker_by_pid(ctx, pid)
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
    Ok(sqlx::query_as!(
        workers::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, person_ref,
                  upstream_worker_ref, organization_ref, worker_number, display_name, status, employment_type,
                  fte_percent, department, job_title, location, work_jurisdiction, manager_pid, salary_minor,
                  salary_currency, hired_on, terminated_on, engagement_ends_on, deleted_at AS "deleted_at: _"
             FROM workers WHERE pid = ANY($1) AND deleted_at IS NULL"#,
        &ids,
    )
    .fetch_all(db::pool(ctx))
    .await?
    .into_iter()
    .map(|w| (w.pid, w))
    .collect())
}

async fn live_backups(ctx: &AppContext, worker: Uuid) -> Result<Vec<worker_backups::Model>> {
    Ok(sqlx::query_as!(
        worker_backups::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, backup_pid, priority,
                  starts_on, ends_on, note, recorded_by, on_behalf, deleted_at AS "deleted_at: _"
             FROM worker_backups WHERE worker_pid = $1 AND deleted_at IS NULL ORDER BY priority, id"#,
        worker,
    )
    .fetch_all(db::pool(ctx))
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
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
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
    let next = i32::try_from(existing.len())
        .unwrap_or(i32::MAX)
        .saturating_add(1);
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
    if let Some(refs) = memberships::scope_organization_refs(&ctx, caller.claims()).await?
        && !refs.contains(&backup.organization_ref)
    {
        return Err(unprocessable("that backup is outside your organizations"));
    }
    let pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO worker_backups (pid, worker_pid, backup_pid, priority, starts_on, ends_on, note, recorded_by, on_behalf,
                                     deleted_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NULL)",
        pid,
        worker.pid,
        backup.pid,
        priority,
        payload.starts_on,
        payload.ends_on,
        clean(payload.note),
        caller.actor(),
        auth::acting_for_other(&caller, &worker.person_ref),
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "worker",
        worker.pid,
        "backup_added",
        caller.actor(),
        Some(serde_json::json!({ "backup_pid": backup.pid })),
    )
    .await?;
    tx.commit().await?;
    format::json(serde_json::json!({ "pid": pid }))
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
    sqlx::query_as!(
        worker_backups::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, backup_pid, priority,
                  starts_on, ends_on, note, recorded_by, on_behalf, deleted_at AS "deleted_at: _"
             FROM worker_backups WHERE pid = $1 AND deleted_at IS NULL"#,
        records::parse_pid(pid)?,
    )
    .fetch_optional(db::pool(ctx))
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
    let note = payload
        .note
        .map_or_else(|| row.note.clone(), |v| clean(Some(v)));
    let mut tx = db::pool(&ctx).begin().await?;
    let updated = sqlx::query_as!(
        worker_backups::Model,
        r#"UPDATE worker_backups SET priority = $2, starts_on = $3, ends_on = $4, note = $5
            WHERE id = $1
        RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, backup_pid, priority,
                  starts_on, ends_on, note, recorded_by, on_behalf, deleted_at AS "deleted_at: _""#,
        row.id,
        priority,
        starts_on,
        ends_on,
        note,
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(Error::NotFound)?;
    db::audit(
        &mut tx,
        "worker",
        worker.pid,
        "backup_updated",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    let who = records::worker_by_pid(&ctx, updated.backup_pid).await.ok();
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
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "UPDATE worker_backups SET deleted_at = now() WHERE id = $1",
        row.id
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "worker",
        worker.pid,
        "backup_removed",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
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
    axa::extract::Query(query): axa::extract::Query<CoverQuery>,
) -> Result<Response> {
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
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
    let on_leave: Vec<Uuid> = sqlx::query_scalar!(
        "SELECT worker_pid FROM leave_requests
          WHERE worker_pid = ANY($1) AND status = 'approved' AND deleted_at IS NULL AND start_on <= $2 AND end_on >= $2",
        &ids,
        on,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let candidates: Vec<Candidate> = rows
        .iter()
        .map(|b| {
            let employed = who
                .get(&b.backup_pid)
                .is_some_and(|w| metric_rules::is_employed_on(on, w.hired_on, w.terminated_on));
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
