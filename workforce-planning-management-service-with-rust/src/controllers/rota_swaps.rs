//! **Rota swap requests**: a person on call asks a colleague to take their
//! on-call days in a window. The colleague accepts or declines; the requester
//! can cancel while it is open. On acceptance the requester's *own* on-call
//! stretches inside the window become overrides for the colleague
//! ([`rules::stretches_for`]) — a swap never moves anyone else's days.
//!
//! Each side acts for themself (or HR on their behalf): the request is
//! authorized as a write to the requester's record, a decision as a write to
//! the colleague's. Both are told the outcome (`swap_requested`,
//! `swap_decided`; reference-only).

use chrono::{NaiveDate, Utc};
use loco_rs::prelude::*;
use sea_orm::{ActiveValue, QueryOrder, TransactionTrait};
use serde::Deserialize;
use std::collections::HashMap;
use uuid::Uuid;

use super::rotas::{check_members, compute, find_rota, name_of, require_scope, workers_by_pid};
use super::{record_rejection, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::models::_entities::{rota_overrides, rota_swap_requests, rotas, workers};
use crate::models::audit_logs::Model as Audit;
use crate::models::notifications::Model as Notification;
use crate::models::records;
use crate::rules::rota as rules;

/// A worker, authorized for a write to their record: themself, or HR.
async fn writable_worker(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: Uuid,
) -> Result<workers::Model> {
    let worker = records::find_worker(&ctx.db, pid).await?;
    auth::authorize_record(
        caller,
        authentication_verifier::Action::Write,
        &auth::worker_resource_attrs(&worker),
    )
    .map_err(record_rejection)?;
    Ok(worker)
}

fn request_json(
    r: &rota_swap_requests::Model,
    rota_name: Option<&str>,
    people: &HashMap<Uuid, workers::Model>,
) -> serde_json::Value {
    serde_json::json!({
        "pid": r.pid,
        "rota_pid": r.rota_pid,
        "rota_name": rota_name,
        "requester_pid": r.requester_pid,
        "requester_name": name_of(people, Some(r.requester_pid)),
        "taker_pid": r.taker_pid,
        "taker_name": name_of(people, Some(r.taker_pid)),
        "starts_on": r.starts_on,
        "ends_on": r.ends_on,
        "note": r.note,
        "status": r.status,
    })
}

async fn find_request(ctx: &AppContext, pid: &str) -> Result<rota_swap_requests::Model> {
    rota_swap_requests::Entity::find()
        .filter(rota_swap_requests::Column::Pid.eq(records::parse_pid(pid)?))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)
}

/// `POST /api/rotas/{pid}/swap-requests` body.
#[derive(Debug, Deserialize)]
struct SwapPayload {
    requester_pid: Uuid,
    taker_pid: Uuid,
    starts_on: NaiveDate,
    ends_on: NaiveDate,
    #[serde(default)]
    note: Option<String>,
}

/// `POST /api/rotas/{pid}/swap-requests` — ask a colleague to take the
/// requester's on-call days in a window. The requester must actually be on
/// call at some point in the window, and the colleague must be an employed
/// worker of the rota's organization.
#[debug_handler]
async fn request_swap(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<SwapPayload>,
) -> Result<Response> {
    let rota = find_rota(&ctx, &caller, &pid).await?;
    let requester = writable_worker(&ctx, &caller, payload.requester_pid).await?;
    let today = Utc::now().date_naive();
    rules::validate_swap(
        payload.requester_pid,
        payload.taker_pid,
        payload.starts_on,
        payload.ends_on,
        today,
    )
    .map_err(|e| unprocessable(&e))?;
    check_members(&ctx, &rota.organization_ref, &[payload.taker_pid]).await?;
    let (_, _, assignments, _) = compute(&ctx, &rota, payload.starts_on, payload.ends_on).await?;
    if rules::stretches_for(&assignments, requester.pid).is_empty() {
        return Err(unprocessable("the requester is not on call in that window"));
    }
    let open = rota_swap_requests::Entity::find()
        .filter(rota_swap_requests::Column::RotaPid.eq(rota.pid))
        .filter(rota_swap_requests::Column::RequesterPid.eq(requester.pid))
        .filter(rota_swap_requests::Column::TakerPid.eq(payload.taker_pid))
        .filter(rota_swap_requests::Column::StartsOn.eq(payload.starts_on))
        .filter(rota_swap_requests::Column::EndsOn.eq(payload.ends_on))
        .filter(rota_swap_requests::Column::Status.eq("requested"))
        .one(&ctx.db)
        .await?;
    if open.is_some() {
        return Err(unprocessable("that swap has already been requested"));
    }
    let txn = ctx.db.begin().await?;
    let row = rota_swap_requests::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        rota_pid: ActiveValue::set(rota.pid),
        requester_pid: ActiveValue::set(requester.pid),
        taker_pid: ActiveValue::set(payload.taker_pid),
        starts_on: ActiveValue::set(payload.starts_on),
        ends_on: ActiveValue::set(payload.ends_on),
        note: ActiveValue::set(
            payload
                .note
                .map(|n| n.trim().to_string())
                .filter(|n| !n.is_empty()),
        ),
        status: ActiveValue::set("requested".to_string()),
        created_by: ActiveValue::set(caller.actor().map(ToString::to_string)),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    Notification::push(
        &txn,
        payload.taker_pid,
        "swap_requested",
        &format!(
            "{} asks you to take their on-call for {} from {} to {}.",
            requester.display_name, rota.name, payload.starts_on, payload.ends_on
        ),
        serde_json::json!({ "rota_pid": rota.pid, "swap_request_pid": row.pid }),
    )
    .await?;
    Audit::record(
        &txn,
        "rota",
        rota.pid,
        "swap_requested",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(serde_json::json!({ "pid": row.pid }))
}

/// Tell the other party how a request ended.
async fn tell(
    db: &impl ConnectionTrait,
    to: Uuid,
    rota: &rotas::Model,
    request: Uuid,
    message: String,
) -> Result<()> {
    Notification::push(
        db,
        to,
        "swap_decided",
        &message,
        serde_json::json!({ "rota_pid": rota.pid, "swap_request_pid": request }),
    )
    .await?;
    Ok(())
}

/// Move a requested swap to `to`, once. Returns the request and its rota.
async fn decide(
    ctx: &AppContext,
    request: &rota_swap_requests::Model,
    to: &str,
) -> Result<rota_swap_requests::Model> {
    if !rules::swap_can_move(&request.status, to) {
        return Err(unprocessable(&format!(
            "that request is already {}",
            request.status
        )));
    }
    let mut active: rota_swap_requests::ActiveModel = request.clone().into();
    active.status = ActiveValue::set(to.to_string());
    active.decided_at = ActiveValue::set(Some(Utc::now().into()));
    Ok(active.update(&ctx.db).await?)
}

/// `POST /api/rota-swap-requests/{pid}/accept` — the colleague takes it: the
/// requester's on-call stretches inside the window become the colleague's.
#[debug_handler]
async fn accept(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let request = find_request(&ctx, &pid).await?;
    let taker = writable_worker(&ctx, &caller, request.taker_pid).await?;
    let rota = find_rota(&ctx, &caller, &request.rota_pid.to_string()).await?;
    if !rules::swap_can_move(&request.status, "accepted") {
        return Err(unprocessable(&format!(
            "that request is already {}",
            request.status
        )));
    }
    check_members(&ctx, &rota.organization_ref, &[taker.pid]).await?;
    let (_, _, assignments, people) =
        compute(&ctx, &rota, request.starts_on, request.ends_on).await?;
    let stretches = rules::stretches_for(&assignments, request.requester_pid);
    if stretches.is_empty() {
        return Err(unprocessable(
            "the requester is no longer on call in that window",
        ));
    }
    let txn = ctx.db.begin().await?;
    for run in &stretches {
        rota_overrides::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            rota_pid: ActiveValue::set(rota.pid),
            worker_pid: ActiveValue::set(taker.pid),
            starts_on: ActiveValue::set(run.from),
            ends_on: ActiveValue::set(run.to),
            note: ActiveValue::set(Some(format!(
                "Swap with {}",
                name_of(&people, Some(request.requester_pid)).unwrap_or_default()
            ))),
            created_by: ActiveValue::set(caller.actor().map(ToString::to_string)),
            ..Default::default()
        }
        .insert(&txn)
        .await?;
    }
    let mut active: rota_swap_requests::ActiveModel = request.clone().into();
    active.status = ActiveValue::set("accepted".to_string());
    active.decided_at = ActiveValue::set(Some(Utc::now().into()));
    active.update(&txn).await?;
    tell(
        &txn,
        request.requester_pid,
        &rota,
        request.pid,
        format!(
            "{} accepted your on-call swap for {}.",
            taker.display_name, rota.name
        ),
    )
    .await?;
    Audit::record(
        &txn,
        "rota",
        rota.pid,
        "swap_accepted",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(serde_json::json!({ "status": "accepted", "days_moved": stretches.len() }))
}

/// `POST /api/rota-swap-requests/{pid}/decline` — the colleague says no.
#[debug_handler]
async fn decline(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let request = find_request(&ctx, &pid).await?;
    let taker = writable_worker(&ctx, &caller, request.taker_pid).await?;
    let rota = find_rota(&ctx, &caller, &request.rota_pid.to_string()).await?;
    decide(&ctx, &request, "declined").await?;
    tell(
        &ctx.db,
        request.requester_pid,
        &rota,
        request.pid,
        format!(
            "{} declined your on-call swap for {}.",
            taker.display_name, rota.name
        ),
    )
    .await?;
    Audit::record(
        &ctx.db,
        "rota",
        rota.pid,
        "swap_declined",
        caller.actor(),
        None,
    )
    .await?;
    format::json(serde_json::json!({ "status": "declined" }))
}

/// `POST /api/rota-swap-requests/{pid}/cancel` — the requester withdraws it.
#[debug_handler]
async fn cancel(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let request = find_request(&ctx, &pid).await?;
    let requester = writable_worker(&ctx, &caller, request.requester_pid).await?;
    let rota = find_rota(&ctx, &caller, &request.rota_pid.to_string()).await?;
    decide(&ctx, &request, "cancelled").await?;
    tell(
        &ctx.db,
        request.taker_pid,
        &rota,
        request.pid,
        format!(
            "{} withdrew their on-call swap request for {}.",
            requester.display_name, rota.name
        ),
    )
    .await?;
    Audit::record(
        &ctx.db,
        "rota",
        rota.pid,
        "swap_cancelled",
        caller.actor(),
        None,
    )
    .await?;
    format::json(serde_json::json!({ "status": "cancelled" }))
}

/// `GET /api/rotas/{pid}/swap-requests` — a rota's requests, newest first.
#[debug_handler]
async fn list_for_rota(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let rota = find_rota(&ctx, &caller, &pid).await?;
    let rows = rota_swap_requests::Entity::find()
        .filter(rota_swap_requests::Column::RotaPid.eq(rota.pid))
        .order_by_desc(rota_swap_requests::Column::Id)
        .all(&ctx.db)
        .await?;
    let ids: Vec<Uuid> = rows
        .iter()
        .flat_map(|r| [r.requester_pid, r.taker_pid])
        .collect();
    let people = workers_by_pid(&ctx, ids).await?;
    format::json(
        rows.iter()
            .map(|r| request_json(r, Some(&rota.name), &people))
            .collect::<Vec<_>>(),
    )
}

/// `GET /api/workers/{pid}/swap-requests` — a person's open requests across
/// the rotas the caller can read: `incoming` (asking them to take on-call)
/// and `outgoing` (what they asked others).
#[debug_handler]
async fn list_for_worker(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    let rows = rota_swap_requests::Entity::find()
        .filter(rota_swap_requests::Column::Status.eq("requested"))
        .filter(
            sea_orm::Condition::any()
                .add(rota_swap_requests::Column::RequesterPid.eq(worker.pid))
                .add(rota_swap_requests::Column::TakerPid.eq(worker.pid)),
        )
        .order_by_asc(rota_swap_requests::Column::StartsOn)
        .all(&ctx.db)
        .await?;
    let mut rota_names: HashMap<Uuid, String> = HashMap::new();
    let mut visible = Vec::new();
    for r in rows {
        if !rota_names.contains_key(&r.rota_pid)
            && let Some(rota) = rotas::Entity::find()
                .filter(rotas::Column::Pid.eq(r.rota_pid))
                .filter(rotas::Column::DeletedAt.is_null())
                .one(&ctx.db)
                .await?
            && require_scope(&ctx, &caller, &rota.organization_ref)
                .await
                .is_ok()
        {
            rota_names.insert(r.rota_pid, rota.name);
        }
        if rota_names.contains_key(&r.rota_pid) {
            visible.push(r);
        }
    }
    let ids: Vec<Uuid> = visible
        .iter()
        .flat_map(|r| [r.requester_pid, r.taker_pid])
        .collect();
    let people = workers_by_pid(&ctx, ids).await?;
    let json = |r: &rota_swap_requests::Model| {
        request_json(r, rota_names.get(&r.rota_pid).map(String::as_str), &people)
    };
    format::json(serde_json::json!({
        "incoming": visible.iter().filter(|r| r.taker_pid == worker.pid).map(&json).collect::<Vec<_>>(),
        "outgoing": visible.iter().filter(|r| r.requester_pid == worker.pid).map(&json).collect::<Vec<_>>(),
    }))
}

/// The swap-request routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/rotas/{pid}/swap-requests", post(request_swap))
        .add("/rotas/{pid}/swap-requests", get(list_for_rota))
        .add("/rota-swap-requests/{pid}/accept", post(accept))
        .add("/rota-swap-requests/{pid}/decline", post(decline))
        .add("/rota-swap-requests/{pid}/cancel", post(cancel))
        .add("/workers/{pid}/swap-requests", get(list_for_worker))
}
