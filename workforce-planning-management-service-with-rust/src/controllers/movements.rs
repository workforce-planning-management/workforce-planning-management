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

use chrono::{NaiveDate, Utc};
use loco_rs::prelude::*;
use sea_orm::{ActiveValue, QueryOrder, TransactionTrait};
use serde::Deserialize;
use std::collections::HashMap;
use uuid::Uuid;

use super::handover::unassigned_count;
use super::{record_rejection, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::models::_entities::{movement_items, movements, workers};
use crate::models::audit_logs::Model as Audit;
use crate::models::{memberships, records};
use crate::rules::movements::{self as rules, ItemState};

/// A worker, authorized for a write to their record: themself, or HR.
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

/// A movement whose worker's organization the caller can read.
pub(super) async fn find_movement(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: &str,
) -> Result<(movements::Model, workers::Model)> {
    let movement = movements::Entity::find()
        .filter(movements::Column::Pid.eq(records::parse_pid(pid)?))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    let worker = records::find_worker(&ctx.db, movement.worker_pid).await?;
    if let Some(refs) = memberships::scope_organization_refs(&ctx.db, caller.claims()).await?
        && !refs.iter().any(|r| r == &worker.organization_ref)
    {
        return Err(Error::NotFound);
    }
    Ok((movement, worker))
}

async fn items_of(ctx: &AppContext, movement: Uuid) -> Result<Vec<movement_items::Model>> {
    Ok(movement_items::Entity::find()
        .filter(movement_items::Column::MovementPid.eq(movement))
        .order_by_asc(movement_items::Column::Position)
        .order_by_asc(movement_items::Column::Id)
        .all(&ctx.db)
        .await?)
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
    Ok(workers::Entity::find()
        .filter(workers::Column::Pid.is_in(pids))
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|w| (w.pid, w.display_name))
        .collect())
}

fn movement_json(
    m: &movements::Model,
    w: &workers::Model,
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
        "reason": m.reason,
        "status": m.status,
        "notes": m.notes,
        "completed_at": m.completed_at,
        "progress": progress,
    })
}

/// `POST /api/workers/{pid}/movements` body.
#[derive(Debug, Deserialize)]
struct MovementPayload {
    /// `joiner` or `leaver`.
    kind: String,
    /// The start day (default: the hire date) or the leaver's last day (required).
    #[serde(default)]
    effective_on: Option<NaiveDate>,
    /// A leaver's reason.
    #[serde(default)]
    reason: Option<String>,
    #[serde(default)]
    notes: Option<String>,
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
    let txn = ctx.db.begin().await?;
    let movement = movements::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        worker_pid: ActiveValue::set(worker.pid),
        kind: ActiveValue::set(payload.kind.clone()),
        effective_on: ActiveValue::set(effective),
        reason: ActiveValue::set(payload.reason.clone()),
        status: ActiveValue::set("open".to_string()),
        notes: ActiveValue::set(
            payload
                .notes
                .map(|n| n.trim().to_string())
                .filter(|n| !n.is_empty()),
        ),
        created_by: ActiveValue::set(caller.actor().map(ToString::to_string)),
        completed_at: ActiveValue::set(None),
        ..Default::default()
    }
    .insert(&txn)
    .await
    .map_err(|e| {
        if e.to_string().contains("movements_one_open") {
            unprocessable("that person already has an open record of that kind")
        } else {
            e.into()
        }
    })?;
    for item in rules::plan(&payload.kind, effective) {
        movement_items::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            movement_pid: ActiveValue::set(movement.pid),
            position: ActiveValue::set(item.position),
            title: ActiveValue::set(item.title),
            assignee_pid: ActiveValue::set(if item.category == "people" {
                worker.manager_pid
            } else {
                None
            }),
            category: ActiveValue::set(item.category),
            due_on: ActiveValue::set(item.due_on),
            ..Default::default()
        }
        .insert(&txn)
        .await?;
    }
    Audit::record(
        &txn,
        "worker",
        worker.pid,
        &format!("{}_opened", payload.kind),
        caller.actor(),
        Some(serde_json::json!({ "movement_pid": movement.pid, "effective_on": effective })),
    )
    .await?;
    txn.commit().await?;
    format::json(serde_json::json!({ "pid": movement.pid }))
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
    axum::extract::Query(q): axum::extract::Query<ListQuery>,
) -> Result<Response> {
    let wanted = q.status.unwrap_or_else(|| "open".to_string());
    let mut query = movements::Entity::find().filter(movements::Column::Status.eq(wanted));
    if let Some(kind) = &q.kind {
        query = query.filter(movements::Column::Kind.eq(kind));
    }
    let rows = query
        .order_by_asc(movements::Column::EffectiveOn)
        .order_by_asc(movements::Column::Id)
        .all(&ctx.db)
        .await?;
    let scope = memberships::scope_organization_refs(&ctx.db, caller.claims()).await?;
    let today = Utc::now().date_naive();
    let mut out = Vec::new();
    for m in rows {
        let Ok(worker) = records::find_worker(&ctx.db, m.worker_pid).await else {
            continue;
        };
        if scope
            .as_ref()
            .is_some_and(|refs| !refs.contains(&worker.organization_ref))
        {
            continue;
        }
        let states: Vec<ItemState> = items_of(&ctx, m.pid)
            .await?
            .iter()
            .map(|i| state_of(i, today))
            .collect();
        out.push(movement_json(&m, &worker, rules::progress(&states)));
    }
    format::json(out)
}

/// `GET /api/movements/{pid}` — a movement with its checklist.
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
    let mut json = movement_json(&movement, &worker, rules::progress(&states));
    json["items"] =
        serde_json::Value::Array(items.iter().map(|i| item_json(i, today, &names)).collect());
    format::json(json)
}

/// `GET /api/workers/{pid}/movements` — a person's movements, newest first.
#[debug_handler]
async fn worker_movements(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    if let Some(refs) = memberships::scope_organization_refs(&ctx.db, caller.claims()).await?
        && !refs.contains(&worker.organization_ref)
    {
        return Err(Error::NotFound);
    }
    let today = Utc::now().date_naive();
    let rows = movements::Entity::find()
        .filter(movements::Column::WorkerPid.eq(worker.pid))
        .order_by_desc(movements::Column::Id)
        .all(&ctx.db)
        .await?;
    let mut out = Vec::new();
    for m in rows {
        let states: Vec<ItemState> = items_of(&ctx, m.pid)
            .await?
            .iter()
            .map(|i| state_of(i, today))
            .collect();
        out.push(movement_json(&m, &worker, rules::progress(&states)));
    }
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
    let item = movement_items::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        movement_pid: ActiveValue::set(movement.pid),
        position: ActiveValue::set(next),
        title: ActiveValue::set(payload.title.trim().to_string()),
        category: ActiveValue::set(
            payload
                .category
                .filter(|c| !c.trim().is_empty())
                .unwrap_or_else(|| "other".to_string()),
        ),
        due_on: ActiveValue::set(payload.due_on),
        assignee_pid: ActiveValue::set(payload.assignee_pid),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;
    Audit::record(
        &ctx.db,
        "worker",
        worker.pid,
        "movement_item_added",
        caller.actor(),
        None,
    )
    .await?;
    format::json(serde_json::json!({ "pid": item.pid }))
}

async fn find_item(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: &str,
) -> Result<(movement_items::Model, movements::Model, workers::Model)> {
    let item = movement_items::Entity::find()
        .filter(movement_items::Column::Pid.eq(records::parse_pid(pid)?))
        .one(&ctx.db)
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
    let mut active: movement_items::ActiveModel = item.into();
    active.done_on = ActiveValue::set(Some(Utc::now().date_naive()));
    active.done_by = ActiveValue::set(caller.actor().map(ToString::to_string));
    active.skipped_reason = ActiveValue::set(None);
    active.update(&ctx.db).await?;
    Audit::record(
        &ctx.db,
        "worker",
        worker.pid,
        "movement_item_done",
        caller.actor(),
        None,
    )
    .await?;
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
    let mut active: movement_items::ActiveModel = item.into();
    active.skipped_reason = ActiveValue::set(Some(payload.reason.trim().to_string()));
    active.done_on = ActiveValue::set(None);
    active.done_by = ActiveValue::set(caller.actor().map(ToString::to_string));
    active.update(&ctx.db).await?;
    Audit::record(
        &ctx.db,
        "worker",
        worker.pid,
        "movement_item_skipped",
        caller.actor(),
        None,
    )
    .await?;
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
    let mut active: movement_items::ActiveModel = item.into();
    active.done_on = ActiveValue::set(None);
    active.done_by = ActiveValue::set(None);
    active.skipped_reason = ActiveValue::set(None);
    active.update(&ctx.db).await?;
    Audit::record(
        &ctx.db,
        "worker",
        worker.pid,
        "movement_item_reopened",
        caller.actor(),
        None,
    )
    .await?;
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
        records::find_worker(&ctx.db, a)
            .await
            .map_err(|_| unprocessable("that assignee is not a worker here"))?;
    }
    let mut active: movement_items::ActiveModel = item.into();
    active.assignee_pid = ActiveValue::set(payload.assignee_pid);
    active.update(&ctx.db).await?;
    Audit::record(
        &ctx.db,
        "worker",
        worker.pid,
        "movement_item_assigned",
        caller.actor(),
        None,
    )
    .await?;
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
    rules::can_complete(open, held).map_err(|e| unprocessable(&e))?;
    let mut active: movements::ActiveModel = movement.clone().into();
    active.status = ActiveValue::set("completed".to_string());
    active.completed_at = ActiveValue::set(Some(Utc::now().into()));
    active.update(&ctx.db).await?;
    Audit::record(
        &ctx.db,
        "worker",
        worker.pid,
        &format!("{}_completed", movement.kind),
        caller.actor(),
        None,
    )
    .await?;
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
    let mut active: movements::ActiveModel = movement.clone().into();
    active.status = ActiveValue::set("cancelled".to_string());
    active.update(&ctx.db).await?;
    Audit::record(
        &ctx.db,
        "worker",
        worker.pid,
        &format!("{}_cancelled", movement.kind),
        caller.actor(),
        None,
    )
    .await?;
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
