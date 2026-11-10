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

use axa::extract::{Json, Path};
use axa::prelude::*;
use chrono::{NaiveDate, Utc};
use serde::Deserialize;
use std::collections::HashMap;
use uuid::Uuid;

use super::rotas::{
    RotaPerson, check_members, compute, find_rota, name_of, require_scope, workers_by_pid,
};
use super::{record_rejection, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::db;
use crate::models::_entities::{rota_swap_requests, rotas, workers};
use crate::models::records;
use crate::rules::rota as rules;

/// A worker, authorized for a write to their record: themself, or HR.
async fn writable_worker(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: Uuid,
) -> Result<workers::Model> {
    let worker = records::worker_by_pid(ctx, pid).await?;
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
    people: &HashMap<Uuid, RotaPerson>,
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
    sqlx::query_as!(
        rota_swap_requests::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, rota_pid, requester_pid, taker_pid,
                  starts_on, ends_on, note, status, decided_at AS "decided_at: _", created_by
             FROM rota_swap_requests WHERE pid = $1"#,
        records::parse_pid(pid)?,
    )
    .fetch_optional(db::pool(ctx))
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
    let open = sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM rota_swap_requests
                           WHERE rota_pid = $1 AND requester_pid = $2 AND taker_pid = $3 AND starts_on = $4
                             AND ends_on = $5 AND status = 'requested') AS "open!""#,
        rota.pid,
        requester.pid,
        payload.taker_pid,
        payload.starts_on,
        payload.ends_on,
    )
    .fetch_one(db::pool(&ctx))
    .await?;
    if open {
        return Err(unprocessable("that swap has already been requested"));
    }
    let request = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO rota_swap_requests (pid, rota_pid, requester_pid, taker_pid, starts_on, ends_on, note, status, created_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7, 'requested', $8)",
        request,
        rota.pid,
        requester.pid,
        payload.taker_pid,
        payload.starts_on,
        payload.ends_on,
        payload
            .note
            .map(|n| n.trim().to_string())
            .filter(|n| !n.is_empty()),
        caller.actor(),
    )
    .execute(&mut *tx)
    .await?;
    db::notify(
        &mut tx,
        payload.taker_pid,
        "swap_requested",
        &format!(
            "{} asks you to take their on-call for {} from {} to {}.",
            requester.display_name, rota.name, payload.starts_on, payload.ends_on
        ),
        serde_json::json!({ "rota_pid": rota.pid, "swap_request_pid": request }),
    )
    .await?;
    db::audit(
        &mut tx,
        "rota",
        rota.pid,
        "swap_requested",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(serde_json::json!({ "pid": request }))
}

/// Tell the other party how a request ended.
async fn tell(
    tx: &mut sqlx::PgConnection,
    to: Uuid,
    rota: &rotas::Model,
    request: Uuid,
    message: String,
) -> Result<()> {
    db::notify(
        tx,
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
    tx: &mut sqlx::PgConnection,
    request: &rota_swap_requests::Model,
    to: &str,
) -> Result<()> {
    if !rules::swap_can_move(&request.status, to) {
        return Err(unprocessable(&format!(
            "that request is already {}",
            request.status
        )));
    }
    sqlx::query!(
        "UPDATE rota_swap_requests SET status = $2, decided_at = $3 WHERE id = $1",
        request.id,
        to,
        Utc::now(),
    )
    .execute(tx)
    .await?;
    Ok(())
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
    let note = format!(
        "Swap with {}",
        name_of(&people, Some(request.requester_pid)).unwrap_or_default()
    );
    let froms: Vec<NaiveDate> = stretches.iter().map(|run| run.from).collect();
    let tos: Vec<NaiveDate> = stretches.iter().map(|run| run.to).collect();
    let mut tx = db::pool(&ctx).begin().await?;
    // Every stretch becomes the taker's override, in one statement.
    sqlx::query!(
        "INSERT INTO rota_overrides (pid, rota_pid, worker_pid, starts_on, ends_on, note, created_by)
         SELECT gen_random_uuid(), $1, $2, run.starts_on, run.ends_on, $5, $6
           FROM unnest($3::date[], $4::date[]) AS run(starts_on, ends_on)",
        rota.pid,
        taker.pid,
        &froms,
        &tos,
        note,
        caller.actor(),
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "UPDATE rota_swap_requests SET status = 'accepted', decided_at = $2 WHERE id = $1",
        request.id,
        Utc::now(),
    )
    .execute(&mut *tx)
    .await?;
    tell(
        &mut tx,
        request.requester_pid,
        &rota,
        request.pid,
        format!(
            "{} accepted your on-call swap for {}.",
            taker.display_name, rota.name
        ),
    )
    .await?;
    db::audit(
        &mut tx,
        "rota",
        rota.pid,
        "swap_accepted",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
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
    let mut tx = db::pool(&ctx).begin().await?;
    decide(&mut tx, &request, "declined").await?;
    tell(
        &mut tx,
        request.requester_pid,
        &rota,
        request.pid,
        format!(
            "{} declined your on-call swap for {}.",
            taker.display_name, rota.name
        ),
    )
    .await?;
    db::audit(
        &mut tx,
        "rota",
        rota.pid,
        "swap_declined",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
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
    let mut tx = db::pool(&ctx).begin().await?;
    decide(&mut tx, &request, "cancelled").await?;
    tell(
        &mut tx,
        request.taker_pid,
        &rota,
        request.pid,
        format!(
            "{} withdrew their on-call swap request for {}.",
            requester.display_name, rota.name
        ),
    )
    .await?;
    db::audit(
        &mut tx,
        "rota",
        rota.pid,
        "swap_cancelled",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
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
    let rows = sqlx::query_as!(
        rota_swap_requests::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, rota_pid, requester_pid, taker_pid,
                  starts_on, ends_on, note, status, decided_at AS "decided_at: _", created_by
             FROM rota_swap_requests WHERE rota_pid = $1 ORDER BY id DESC"#,
        rota.pid,
    )
    .fetch_all(db::pool(&ctx))
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
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    let rows = sqlx::query_as!(
        rota_swap_requests::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, rota_pid, requester_pid, taker_pid,
                  starts_on, ends_on, note, status, decided_at AS "decided_at: _", created_by
             FROM rota_swap_requests
            WHERE status = 'requested' AND (requester_pid = $1 OR taker_pid = $1)
            ORDER BY starts_on, id"#,
        worker.pid,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let mut rota_names: HashMap<Uuid, String> = HashMap::new();
    let mut visible = Vec::new();
    for r in rows {
        if !rota_names.contains_key(&r.rota_pid)
            && let Some(rota) = sqlx::query!(
                "SELECT name, organization_ref FROM rotas WHERE pid = $1 AND deleted_at IS NULL",
                r.rota_pid,
            )
            .fetch_optional(db::pool(&ctx))
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
