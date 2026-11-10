//! **Joiners and leavers** (`rules::movements`): a movement record for
//! someone joining or leaving, with a dated checklist built from a standard
//! template around the effective day (start day, or a leaver's **last day**).
//!
//! The record and its checklist live here; what a leaver still holds — and
//! reassigning it, with an audit trail — is [`super::handover`]. A leaver
//! cannot be completed while checklist items are open or anything they hold
//! is unassigned.
//!
//! Who may open a record: whoever may write the worker's record (the person,
//! or HR). Reading and ticking items is limited to organizations the caller
//! can read.
//!
//! **Why someone left is not for everyone on the record** (WPM-D178). A manager
//! and a colleague with a seat see the dates and the checklist they work from.
//! The reason and the notes appear on one read only, for a caller who may write
//! the worker's record, and that read is audited with the pid and no content.

use super::ensure_org_in_scope;
use super::writable_worker;
use axa::extract::{Json, Path};
use axa::prelude::*;
use chrono::{NaiveDate, Utc};
use serde::Deserialize;
use std::collections::HashMap;
use uuid::Uuid;

use super::handover::unassigned_count;
use super::{record_rejection, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::db;
use crate::models::_entities::{movement_items, movements, workers};
use crate::models::{memberships, records};
use crate::rules::movements::{self as rules, ItemState};

/// A movement whose worker's organization the caller can read.
pub(super) async fn find_movement(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: &str,
) -> Result<(movements::Model, workers::Model)> {
    let movement = sqlx::query_as!(
        movements::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, kind,
                  effective_on, reason, status, notes, created_by, completed_at AS "completed_at: _"
             FROM movements WHERE pid = $1"#,
        records::parse_pid(pid)?,
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)?;
    let worker = records::worker_by_pid(ctx, movement.worker_pid).await?;
    ensure_org_in_scope(ctx, caller, &worker.organization_ref).await?;
    Ok((movement, worker))
}

async fn items_of(ctx: &AppContext, movement: Uuid) -> Result<Vec<movement_items::Model>> {
    Ok(sqlx::query_as!(
        movement_items::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, movement_pid, position,
                  title, category, due_on, assignee_pid, done_on, done_by, skipped_reason
             FROM movement_items WHERE movement_pid = $1
            ORDER BY position, id"#,
        movement,
    )
    .fetch_all(db::pool(ctx))
    .await?)
}

/// Each movement's item states, for the lists' progress.
async fn states_of(
    ctx: &AppContext,
    movements: &[Uuid],
    today: NaiveDate,
) -> Result<HashMap<Uuid, Vec<ItemState>>> {
    let mut states: HashMap<Uuid, Vec<ItemState>> = HashMap::new();
    for i in sqlx::query!(
        "SELECT movement_pid, due_on, done_on, skipped_reason FROM movement_items
          WHERE movement_pid = ANY($1)",
        movements,
    )
    .fetch_all(db::pool(ctx))
    .await?
    {
        states
            .entry(i.movement_pid)
            .or_default()
            .push(rules::item_state(
                i.due_on,
                i.done_on.is_some(),
                i.skipped_reason.is_some(),
                today,
            ));
    }
    Ok(states)
}

fn state_of(i: &movement_items::Model, today: NaiveDate) -> ItemState {
    rules::item_state(
        i.due_on,
        i.done_on.is_some(),
        i.skipped_reason.is_some(),
        today,
    )
}

fn item_json(
    i: &movement_items::Model,
    today: NaiveDate,
    names: &HashMap<Uuid, String>,
) -> serde_json::Value {
    serde_json::json!({
        "pid": i.pid,
        "position": i.position,
        "title": i.title,
        "category": i.category,
        "due_on": i.due_on,
        "assignee_pid": i.assignee_pid,
        "assignee_name": i.assignee_pid.and_then(|a| names.get(&a)),
        "done_on": i.done_on,
        "done_by": i.done_by,
        "skipped_reason": i.skipped_reason,
        "state": state_of(i, today),
    })
}

async fn names_for(ctx: &AppContext, pids: Vec<Uuid>) -> Result<HashMap<Uuid, String>> {
    Ok(sqlx::query!(
        "SELECT pid, display_name FROM workers WHERE pid = ANY($1)",
        &pids,
    )
    .fetch_all(db::pool(ctx))
    .await?
    .into_iter()
    .map(|w| (w.pid, w.display_name))
    .collect())
}

/// The worker fields a movement shows.
struct Shown<'a> {
    display_name: &'a str,
    department: &'a str,
    job_title: &'a str,
    organization_ref: &'a str,
}

impl<'a> Shown<'a> {
    fn of(w: &'a workers::Model) -> Self {
        Self {
            display_name: &w.display_name,
            department: &w.department,
            job_title: &w.job_title,
            organization_ref: &w.organization_ref,
        }
    }
}

fn movement_json(
    m: &movements::Model,
    w: &Shown<'_>,
    progress: rules::Progress,
) -> serde_json::Value {
    serde_json::json!({
        "pid": m.pid,
        "kind": m.kind,
        "worker_pid": m.worker_pid,
        "worker_name": w.display_name,
        "department": w.department,
        "job_title": w.job_title,
        "organization_ref": w.organization_ref,
        "effective_on": m.effective_on,
        "status": m.status,
        "completed_at": m.completed_at,
        "progress": progress,
    })
}

/// `POST /api/workers/{pid}/movements` body.
#[derive(Debug, Deserialize)]
pub(crate) struct MovementPayload {
    /// `joiner` or `leaver`.
    pub(crate) kind: String,
    /// The start day (default: the hire date) or the leaver's last day (required).
    #[serde(default)]
    pub(crate) effective_on: Option<NaiveDate>,
    /// A leaver's reason.
    #[serde(default)]
    pub(crate) reason: Option<String>,
    #[serde(default)]
    pub(crate) notes: Option<String>,
}

/// `POST /api/workers/{pid}/movements` — open a joiner or leaver record with
/// its dated checklist. A leaver must still be employed (not already
/// terminated or retired); there is at most one open record of each kind per
/// person. Items for the manager's conversations are assigned to the
/// manager; the rest are unassigned until someone takes them.
#[debug_handler]
async fn open_movement(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<MovementPayload>,
) -> Result<Response> {
    let worker = writable_worker(&ctx, &caller, &pid).await?;
    let mut tx = db::pool(&ctx).begin().await?;
    let movement = open_in(&mut tx, &caller, &worker, payload).await?;
    tx.commit().await?;
    format::json(serde_json::json!({ "pid": movement.pid }))
}

/// Open a movement inside the caller's transaction: validate, insert the record and its dated
/// checklist, audit. Shared by the HR route and the acceptance of a resignation (WPM-R127), which
/// opens the leaver process in the same transaction as the decision.
pub(crate) async fn open_in(
    connection: &mut sqlx::PgConnection,
    caller: &MaybeAuthUser,
    worker: &workers::Model,
    payload: MovementPayload,
) -> Result<movements::Model> {
    let effective = match (payload.kind.as_str(), payload.effective_on) {
        ("leaver", None) => return Err(unprocessable("a leaver needs a last day (effective_on)")),
        (_, Some(d)) => d,
        (_, None) => worker.hired_on,
    };
    rules::validate(
        &payload.kind,
        payload.reason.as_deref(),
        effective,
        worker.hired_on,
    )
    .map_err(|e| unprocessable(&e))?;
    if payload.kind == "leaver" && matches!(worker.status.as_str(), "terminated" | "retired") {
        return Err(unprocessable("that person has already left"));
    }
    let notes = payload
        .notes
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty());
    let movement = sqlx::query_as!(
        movements::Model,
        r#"INSERT INTO movements (pid, worker_pid, kind, effective_on, reason, status, notes, created_by)
           VALUES ($1, $2, $3, $4, $5, 'open', $6, $7)
           RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, kind,
                     effective_on, reason, status, notes, created_by, completed_at AS "completed_at: _""#,
        Uuid::new_v4(),
        worker.pid,
        payload.kind,
        effective,
        payload.reason,
        notes,
        caller.actor(),
    )
    .fetch_one(&mut *connection)
    .await
    .map_err(|e| {
        if e.as_database_error().and_then(|d| d.constraint()) == Some("movements_one_open") {
            unprocessable("that person already has an open record of that kind")
        } else {
            e.into()
        }
    })?;
    let plan = rules::plan(&payload.kind, effective);
    let assignees: Vec<Option<Uuid>> = plan
        .iter()
        .map(|item| {
            (item.category == "people")
                .then_some(worker.manager_pid)
                .flatten()
        })
        .collect();
    let pids: Vec<Uuid> = plan.iter().map(|_| Uuid::new_v4()).collect();
    let positions: Vec<i32> = plan.iter().map(|item| item.position).collect();
    let due: Vec<NaiveDate> = plan.iter().map(|item| item.due_on).collect();
    let (titles, categories): (Vec<String>, Vec<String>) = plan
        .into_iter()
        .map(|item| (item.title, item.category))
        .unzip();
    sqlx::query!(
        "INSERT INTO movement_items (pid, movement_pid, position, title, category, due_on, assignee_pid)
         SELECT p, $2, n, t, c, d, a
           FROM UNNEST($1::uuid[], $3::int4[], $4::text[], $5::text[], $6::date[], $7::uuid[]) AS i(p, n, t, c, d, a)",
        &pids,
        movement.pid,
        &positions,
        &titles,
        &categories,
        &due,
        &assignees as &[Option<Uuid>],
    )
    .execute(&mut *connection)
    .await?;
    let snapshot =
        Some(serde_json::json!({ "movement_pid": movement.pid, "effective_on": effective }));
    if movement.kind == "leaver" {
        db::audit(
            connection,
            "worker",
            worker.pid,
            "leaver_opened",
            caller.actor(),
            snapshot,
        )
        .await?;
    } else {
        db::audit(
            connection,
            "worker",
            worker.pid,
            "joiner_opened",
            caller.actor(),
            snapshot,
        )
        .await?;
    }
    Ok(movement)
}

/// Query for the list.
#[derive(Debug, Deserialize)]
struct ListQuery {
    /// `joiner` or `leaver`.
    kind: Option<String>,
    /// Default `open`.
    status: Option<String>,
}

/// `GET /api/movements?kind=&status=` — movements in the caller's
/// organizations (default open ones), soonest effective day first, each with
/// its checklist progress.
#[debug_handler]
async fn list_movements(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axa::extract::Query(q): axa::extract::Query<ListQuery>,
    page: super::Page,
) -> Result<Response> {
    let window = super::window(&page)?;
    let wanted = q.status.unwrap_or_else(|| "open".to_string());
    let scope = memberships::scope_organization_refs(&ctx, caller.claims()).await?;
    let rows = sqlx::query!(
        r#"SELECT m.created_at AS "created_at: chrono::DateTime<chrono::FixedOffset>",
                  m.updated_at AS "updated_at: chrono::DateTime<chrono::FixedOffset>", m.id, m.pid,
                  m.worker_pid, m.kind, m.effective_on, m.reason, m.status, m.notes, m.created_by,
                  m.completed_at AS "completed_at: chrono::DateTime<chrono::FixedOffset>",
                  w.display_name, w.department, w.job_title, w.organization_ref
             FROM movements m JOIN workers w ON w.pid = m.worker_pid AND w.deleted_at IS NULL
            WHERE m.status = $1 AND ($2::text IS NULL OR m.kind = $2)
              AND ($3::text[] IS NULL OR w.organization_ref = ANY($3))
            ORDER BY m.effective_on, m.id"#,
        wanted,
        q.kind,
        scope.as_deref(),
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let today = Utc::now().date_naive();
    let pids: Vec<Uuid> = rows.iter().map(|r| r.pid).collect();
    let states = states_of(&ctx, &pids, today).await?;
    let mut out = Vec::new();
    for r in rows {
        let progress = rules::progress(states.get(&r.pid).map_or(&[][..], Vec::as_slice));
        let shown = Shown {
            display_name: &r.display_name,
            department: &r.department,
            job_title: &r.job_title,
            organization_ref: &r.organization_ref,
        };
        let m = movements::Model {
            created_at: r.created_at,
            updated_at: r.updated_at,
            id: r.id,
            pid: r.pid,
            worker_pid: r.worker_pid,
            kind: r.kind,
            effective_on: r.effective_on,
            reason: r.reason,
            status: r.status,
            notes: r.notes,
            created_by: r.created_by,
            completed_at: r.completed_at,
        };
        out.push(movement_json(&m, &shown, progress));
    }
    Ok(super::paged_vec(out, &window))
}

/// Whether the caller may read why someone left: HR, the service and an administrator under the
/// reference policy. Never a line manager or payroll (WPM-R222, WPM-D178).
fn may_read_why(caller: &MaybeAuthUser, worker: &workers::Model) -> bool {
    auth::authorize_record(
        caller,
        authentication_verifier::Action::Write,
        &auth::worker_resource_attrs(worker),
    )
    .is_ok()
}

/// `GET /api/movements/{pid}` — a movement with its checklist, and for HR its reason and notes.
#[debug_handler]
async fn get_movement(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let (movement, worker) = find_movement(&ctx, &caller, &pid).await?;
    let today = Utc::now().date_naive();
    let items = items_of(&ctx, movement.pid).await?;
    let names = names_for(&ctx, items.iter().filter_map(|i| i.assignee_pid).collect()).await?;
    let states: Vec<ItemState> = items.iter().map(|i| state_of(i, today)).collect();
    let mut json = movement_json(&movement, &Shown::of(&worker), rules::progress(&states));
    json["items"] =
        serde_json::Value::Array(items.iter().map(|i| item_json(i, today, &names)).collect());
    if may_read_why(&caller, &worker) {
        json["reason"] = serde_json::json!(movement.reason);
        json["notes"] = serde_json::json!(movement.notes);
        if movement.reason.is_some() || movement.notes.is_some() {
            let mut connection = db::pool(&ctx).acquire().await?;
            db::audit(
                &mut connection,
                "movement",
                movement.pid,
                "movement_why_read",
                caller.actor(),
                None,
            )
            .await?;
        }
    }
    format::json(json)
}

/// `GET /api/workers/{pid}/movements` — a person's movements, newest first.
#[debug_handler]
async fn worker_movements(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    ensure_org_in_scope(&ctx, &caller, &worker.organization_ref).await?;
    let today = Utc::now().date_naive();
    let rows = sqlx::query_as!(
        movements::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, kind,
                  effective_on, reason, status, notes, created_by, completed_at AS "completed_at: _"
             FROM movements WHERE worker_pid = $1 ORDER BY id DESC"#,
        worker.pid,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let pids: Vec<Uuid> = rows.iter().map(|m| m.pid).collect();
    let states = states_of(&ctx, &pids, today).await?;
    let shown = Shown::of(&worker);
    let out: Vec<serde_json::Value> = rows
        .iter()
        .map(|m| {
            let progress = rules::progress(states.get(&m.pid).map_or(&[][..], Vec::as_slice));
            movement_json(m, &shown, progress)
        })
        .collect();
    format::json(out)
}

/// An open movement (items can only change while it is open).
fn require_open(m: &movements::Model) -> Result<()> {
    if m.status == "open" {
        Ok(())
    } else {
        Err(unprocessable(&format!(
            "that record is already {}",
            m.status
        )))
    }
}

/// `POST /api/movements/{pid}/items` body.
#[derive(Debug, Deserialize)]
struct ItemPayload {
    title: String,
    #[serde(default)]
    category: Option<String>,
    due_on: NaiveDate,
    #[serde(default)]
    assignee_pid: Option<Uuid>,
}

/// `POST /api/movements/{pid}/items` — add an ad-hoc dated item.
#[debug_handler]
async fn add_item(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<ItemPayload>,
) -> Result<Response> {
    let (movement, worker) = find_movement(&ctx, &caller, &pid).await?;
    require_open(&movement)?;
    if payload.title.trim().is_empty() || payload.title.chars().count() > 200 {
        return Err(unprocessable("title is required (up to 200 characters)"));
    }
    let next = i32::try_from(items_of(&ctx, movement.pid).await?.len()).unwrap_or(i32::MAX);
    let category = payload
        .category
        .filter(|c| !c.trim().is_empty())
        .unwrap_or_else(|| "other".to_string());
    let item = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO movement_items (pid, movement_pid, position, title, category, due_on, assignee_pid)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
        item,
        movement.pid,
        next,
        payload.title.trim(),
        category,
        payload.due_on,
        payload.assignee_pid,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "worker",
        worker.pid,
        "movement_item_added",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(serde_json::json!({ "pid": item }))
}

async fn find_item(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: &str,
) -> Result<(movement_items::Model, movements::Model, workers::Model)> {
    let item = sqlx::query_as!(
        movement_items::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, movement_pid, position,
                  title, category, due_on, assignee_pid, done_on, done_by, skipped_reason
             FROM movement_items WHERE pid = $1"#,
        records::parse_pid(pid)?,
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)?;
    let (movement, worker) = find_movement(ctx, caller, &item.movement_pid.to_string()).await?;
    require_open(&movement)?;
    Ok((item, movement, worker))
}

/// `POST /api/movement-items/{pid}/done` — tick an item off today.
#[debug_handler]
async fn item_done(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let (item, _, worker) = find_item(&ctx, &caller, &pid).await?;
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "UPDATE movement_items SET done_on = $2, done_by = $3, skipped_reason = NULL WHERE id = $1",
        item.id,
        Utc::now().date_naive(),
        caller.actor(),
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "worker",
        worker.pid,
        "movement_item_done",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(serde_json::json!({ "state": "done" }))
}

/// `POST /api/movement-items/{pid}/skip` body.
#[derive(Debug, Deserialize)]
struct SkipPayload {
    reason: String,
}

/// `POST /api/movement-items/{pid}/skip` — set an item aside, with a reason.
#[debug_handler]
async fn item_skip(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<SkipPayload>,
) -> Result<Response> {
    if payload.reason.trim().is_empty() {
        return Err(unprocessable("a skipped item needs a reason"));
    }
    let (item, _, worker) = find_item(&ctx, &caller, &pid).await?;
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "UPDATE movement_items SET skipped_reason = $2, done_on = NULL, done_by = $3 WHERE id = $1",
        item.id,
        payload.reason.trim(),
        caller.actor(),
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "worker",
        worker.pid,
        "movement_item_skipped",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(serde_json::json!({ "state": "skipped" }))
}

/// `POST /api/movement-items/{pid}/reopen` — undo a tick or a skip.
#[debug_handler]
async fn item_reopen(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let (item, _, worker) = find_item(&ctx, &caller, &pid).await?;
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "UPDATE movement_items SET done_on = NULL, done_by = NULL, skipped_reason = NULL WHERE id = $1",
        item.id,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "worker",
        worker.pid,
        "movement_item_reopened",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(serde_json::json!({ "state": "reopened" }))
}

/// `POST /api/movement-items/{pid}/assign` body.
#[derive(Debug, Deserialize)]
struct AssignPayload {
    /// Who is responsible; null = unassigned.
    assignee_pid: Option<Uuid>,
}

/// `POST /api/movement-items/{pid}/assign` — say who is responsible for an item.
#[debug_handler]
async fn item_assign(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<AssignPayload>,
) -> Result<Response> {
    let (item, _, worker) = find_item(&ctx, &caller, &pid).await?;
    if let Some(a) = payload.assignee_pid {
        records::worker_by_pid(&ctx, a)
            .await
            .map_err(|_| unprocessable("that assignee is not a worker here"))?;
    }
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "UPDATE movement_items SET assignee_pid = $2 WHERE id = $1",
        item.id,
        payload.assignee_pid,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "worker",
        worker.pid,
        "movement_item_assigned",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(serde_json::json!({ "assignee_pid": payload.assignee_pid }))
}

/// `POST /api/movements/{pid}/complete` — close the record. Refused while
/// checklist items are open (not done or skipped) or, for a leaver, anything
/// they hold is not yet reassigned or closed.
#[debug_handler]
async fn complete(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let (movement, worker) = find_movement(&ctx, &caller, &pid).await?;
    require_open(&movement)?;
    // Completing is HR's act on the person's record.
    auth::authorize_record(
        &caller,
        authentication_verifier::Action::Write,
        &auth::worker_resource_attrs(&worker),
    )
    .map_err(record_rejection)?;
    let today = Utc::now().date_naive();
    let items = items_of(&ctx, movement.pid).await?;
    let open = items
        .iter()
        .filter(|i| !matches!(state_of(i, today), ItemState::Done | ItemState::Skipped))
        .count();
    let held = if movement.kind == "leaver" {
        unassigned_count(&ctx, &movement, &worker).await?
    } else {
        0
    };
    let equipment = if movement.kind == "leaver" {
        super::equipment::out_count(&ctx, worker.pid).await?
    } else {
        0
    };
    rules::can_complete(open, held, equipment).map_err(|e| unprocessable(&e))?;
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "UPDATE movements SET status = 'completed', completed_at = now() WHERE id = $1",
        movement.id,
    )
    .execute(&mut *tx)
    .await?;
    if movement.kind == "leaver" {
        db::audit(
            &mut tx,
            "worker",
            worker.pid,
            "leaver_completed",
            caller.actor(),
            None,
        )
        .await?;
    } else {
        db::audit(
            &mut tx,
            "worker",
            worker.pid,
            "joiner_completed",
            caller.actor(),
            None,
        )
        .await?;
    }
    tx.commit().await?;
    format::json(serde_json::json!({ "status": "completed" }))
}

/// `POST /api/movements/{pid}/cancel` — the person is not joining / leaving after all.
#[debug_handler]
async fn cancel(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let (movement, worker) = find_movement(&ctx, &caller, &pid).await?;
    require_open(&movement)?;
    auth::authorize_record(
        &caller,
        authentication_verifier::Action::Write,
        &auth::worker_resource_attrs(&worker),
    )
    .map_err(record_rejection)?;
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "UPDATE movements SET status = 'cancelled' WHERE id = $1",
        movement.id,
    )
    .execute(&mut *tx)
    .await?;
    if movement.kind == "leaver" {
        db::audit(
            &mut tx,
            "worker",
            worker.pid,
            "leaver_cancelled",
            caller.actor(),
            None,
        )
        .await?;
    } else {
        db::audit(
            &mut tx,
            "worker",
            worker.pid,
            "joiner_cancelled",
            caller.actor(),
            None,
        )
        .await?;
    }
    tx.commit().await?;
    format::json(serde_json::json!({ "status": "cancelled" }))
}

/// The movement routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/workers/{pid}/movements", post(open_movement))
        .add("/workers/{pid}/movements", get(worker_movements))
        .add("/movements", get(list_movements))
        .add("/movements/{pid}", get(get_movement))
        .add("/movements/{pid}/items", post(add_item))
        .add("/movements/{pid}/complete", post(complete))
        .add("/movements/{pid}/cancel", post(cancel))
        .add("/movement-items/{pid}/done", post(item_done))
        .add("/movement-items/{pid}/skip", post(item_skip))
        .add("/movement-items/{pid}/reopen", post(item_reopen))
        .add("/movement-items/{pid}/assign", post(item_assign))
}
