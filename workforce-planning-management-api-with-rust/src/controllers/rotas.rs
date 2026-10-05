//! **On-call rotas** (`rules::rota`): a named rotation of workers in one
//! organization where the duty passes to the next member every
//! `period_days`; swaps are overrides; a scheduled member who is on approved
//! leave (or has left) is skipped to the next available, and a day with
//! nobody available is shown as unassigned, never guessed.
//!
//! Reads and writes are limited to organizations the caller can read; who
//! may *change* a rota is the route-level policy's write gate (HR / roster
//! owners), as for shifts. Members must be employed workers of the rota's
//! own organization.

use chrono::{Duration, NaiveDate, Utc};
use loco_rs::prelude::*;
use sea_orm::{ActiveValue, QueryOrder, TransactionTrait};
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use uuid::Uuid;

use super::{unprocessable, with_page_headers};
use crate::auth::MaybeAuthUser;
use crate::models::_entities::{leave_requests, rota_members, rota_overrides, rotas, workers};
use crate::models::audit_logs::Model as Audit;
use crate::models::{memberships, records};
use crate::rules::metrics as metric_rules;
use crate::rules::rota::{self as rules, Assignment, Override, Rota};

/// Days shown when the caller names no window.
const DEFAULT_WINDOW_DAYS: i64 = 28;

/// The rota, if it exists and its organization is one the caller can read.
async fn find_rota(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: &str,
) -> Result<rotas::Model> {
    let rota = rotas::Entity::find()
        .filter(rotas::Column::Pid.eq(records::parse_pid(pid)?))
        .filter(rotas::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    require_scope(ctx, caller, &rota.organization_ref).await?;
    Ok(rota)
}

/// A rota outside the caller's organizations does not exist, to them.
async fn require_scope(ctx: &AppContext, caller: &MaybeAuthUser, org: &str) -> Result<()> {
    if let Some(refs) = memberships::scope_organization_refs(&ctx.db, caller.claims()).await?
        && !refs.iter().any(|r| r == org)
    {
        return Err(Error::NotFound);
    }
    Ok(())
}

async fn member_pids(ctx: &AppContext, rota: Uuid) -> Result<Vec<Uuid>> {
    Ok(rota_members::Entity::find()
        .filter(rota_members::Column::RotaPid.eq(rota))
        .order_by_asc(rota_members::Column::Position)
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|m| m.worker_pid)
        .collect())
}

async fn override_rows(ctx: &AppContext, rota: Uuid) -> Result<Vec<rota_overrides::Model>> {
    Ok(rota_overrides::Entity::find()
        .filter(rota_overrides::Column::RotaPid.eq(rota))
        .order_by_asc(rota_overrides::Column::Id)
        .all(&ctx.db)
        .await?)
}

/// Live workers by pid, for names and availability.
async fn workers_by_pid(
    ctx: &AppContext,
    pids: Vec<Uuid>,
) -> Result<HashMap<Uuid, workers::Model>> {
    Ok(workers::Entity::find()
        .filter(workers::Column::Pid.is_in(pids))
        .filter(workers::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|w| (w.pid, w))
        .collect())
}

/// The day-by-day schedule for a rota over `from..=to`, with the workers
/// named in it. Availability = employed that day and not on approved leave.
async fn compute(
    ctx: &AppContext,
    rota: &rotas::Model,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<(Vec<Uuid>, Vec<rota_overrides::Model>, Vec<Assignment>, HashMap<Uuid, workers::Model>)>
{
    let members = member_pids(ctx, rota.pid).await?;
    let overrides = override_rows(ctx, rota.pid).await?;
    let mut ids = members.clone();
    ids.extend(overrides.iter().map(|o| o.worker_pid));
    let people = workers_by_pid(ctx, ids.clone()).await?;
    let mut away: HashMap<Uuid, Vec<(NaiveDate, NaiveDate)>> = HashMap::new();
    for l in leave_requests::Entity::find()
        .filter(leave_requests::Column::WorkerPid.is_in(ids))
        .filter(leave_requests::Column::Status.eq("approved"))
        .filter(leave_requests::Column::DeletedAt.is_null())
        .filter(leave_requests::Column::StartOn.lte(to))
        .filter(leave_requests::Column::EndOn.gte(from))
        .all(&ctx.db)
        .await?
    {
        away.entry(l.worker_pid).or_default().push((l.start_on, l.end_on));
    }
    let unavailable = |worker: Uuid, day: NaiveDate| -> bool {
        let Some(w) = people.get(&worker) else {
            return true;
        };
        !metric_rules::is_employed_on(day, w.hired_on, w.terminated_on)
            || away
                .get(&worker)
                .is_some_and(|spans| spans.iter().any(|(s, e)| *s <= day && day <= *e))
    };
    let model = Rota {
        period_days: i64::from(rota.period_days),
        starts_on: rota.starts_on,
        members: members.clone(),
    };
    let swaps: Vec<Override> = overrides
        .iter()
        .map(|o| Override { worker: o.worker_pid, starts_on: o.starts_on, ends_on: o.ends_on })
        .collect();
    let assignments = rules::schedule(&model, &swaps, &unavailable, from, to);
    Ok((members, overrides, assignments, people))
}

fn name_of(people: &HashMap<Uuid, workers::Model>, pid: Option<Uuid>) -> Option<String> {
    pid.and_then(|p| people.get(&p)).map(|w| w.display_name.clone())
}

/// The `from..=to` window from a query, defaulting to 28 days from today.
fn window(from: Option<NaiveDate>, to: Option<NaiveDate>) -> Result<(NaiveDate, NaiveDate)> {
    let from = from.unwrap_or_else(|| Utc::now().date_naive());
    let to = to.unwrap_or(from + Duration::days(DEFAULT_WINDOW_DAYS - 1));
    if to < from {
        return Err(unprocessable("to must not be before from"));
    }
    if (to - from).num_days() + 1 > rules::MAX_WINDOW_DAYS {
        return Err(unprocessable(&format!(
            "the window may cover at most {} days",
            rules::MAX_WINDOW_DAYS
        )));
    }
    Ok((from, to))
}

fn run_json(
    r: &rules::Run,
    people: &HashMap<Uuid, workers::Model>,
) -> serde_json::Value {
    serde_json::json!({
        "from": r.from,
        "to": r.to,
        "worker_pid": r.worker,
        "worker_name": name_of(people, r.worker),
        "source": r.source,
    })
}

// ─── Create / list / read ───────────────────────────────────────────────────

/// `POST /api/rotas` body.
#[derive(Debug, Deserialize)]
struct RotaPayload {
    organization_ref: String,
    name: String,
    #[serde(default)]
    description: Option<String>,
    period_days: i32,
    starts_on: NaiveDate,
    /// Members in rotation order.
    members: Vec<Uuid>,
}

/// Members must be employed workers of `org`, in the order given.
async fn check_members(ctx: &AppContext, org: &str, members: &[Uuid]) -> Result<()> {
    let people = workers_by_pid(ctx, members.to_vec()).await?;
    let today = Utc::now().date_naive();
    for m in members {
        let Some(w) = people.get(m) else {
            return Err(unprocessable("a member is not a worker here"));
        };
        if w.organization_ref != org {
            return Err(unprocessable("members must belong to the rota's organization"));
        }
        if !metric_rules::is_employed_on(today, w.hired_on, w.terminated_on) {
            return Err(unprocessable("a member is not currently employed"));
        }
    }
    Ok(())
}

async fn write_members(
    txn: &impl ConnectionTrait,
    rota: Uuid,
    members: &[Uuid],
) -> Result<()> {
    rota_members::Entity::delete_many()
        .filter(rota_members::Column::RotaPid.eq(rota))
        .exec(txn)
        .await?;
    for (i, worker) in members.iter().enumerate() {
        rota_members::ActiveModel {
            rota_pid: ActiveValue::set(rota),
            worker_pid: ActiveValue::set(*worker),
            position: ActiveValue::set(i32::try_from(i).unwrap_or(i32::MAX)),
            ..Default::default()
        }
        .insert(txn)
        .await?;
    }
    Ok(())
}

/// `POST /api/rotas` — create a rota.
#[debug_handler]
async fn create_rota(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<RotaPayload>,
) -> Result<Response> {
    require_scope(&ctx, &caller, &payload.organization_ref).await?;
    if payload.name.trim().is_empty() {
        return Err(unprocessable("name is required"));
    }
    rules::validate_rota(i64::from(payload.period_days), &payload.members)
        .map_err(|e| unprocessable(&e))?;
    check_members(&ctx, &payload.organization_ref, &payload.members).await?;
    let txn = ctx.db.begin().await?;
    let row = rotas::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        organization_ref: ActiveValue::set(payload.organization_ref.clone()),
        name: ActiveValue::set(payload.name.trim().to_string()),
        description: ActiveValue::set(
            payload.description.map(|d| d.trim().to_string()).filter(|d| !d.is_empty()),
        ),
        period_days: ActiveValue::set(payload.period_days),
        starts_on: ActiveValue::set(payload.starts_on),
        deleted_at: ActiveValue::set(None),
        ..Default::default()
    }
    .insert(&txn)
    .await
    .map_err(|e| {
        if e.to_string().contains("rotas_org_name") {
            unprocessable("a rota with that name already exists in this organization")
        } else {
            e.into()
        }
    })?;
    write_members(&txn, row.pid, &payload.members).await?;
    Audit::record(&txn, "rota", row.pid, "created", caller.actor(), None).await?;
    txn.commit().await?;
    format::json(serde_json::json!({ "pid": row.pid }))
}

/// `GET /api/rotas` — rotas in the caller's organizations, each with who is
/// on call today.
#[debug_handler]
async fn list_rotas(State(ctx): State<AppContext>, caller: MaybeAuthUser) -> Result<Response> {
    let mut query = rotas::Entity::find().filter(rotas::Column::DeletedAt.is_null());
    if let Some(refs) = memberships::scope_organization_refs(&ctx.db, caller.claims()).await? {
        query = query.filter(rotas::Column::OrganizationRef.is_in(refs));
    }
    let all = query
        .order_by_asc(rotas::Column::OrganizationRef)
        .order_by_asc(rotas::Column::Name)
        .all(&ctx.db)
        .await?;
    let today = Utc::now().date_naive();
    let mut out = Vec::with_capacity(all.len());
    for rota in &all {
        let (members, _, assignments, people) = compute(&ctx, rota, today, today).await?;
        let now = assignments.first().and_then(|a| a.worker);
        out.push(serde_json::json!({
            "pid": rota.pid,
            "organization_ref": rota.organization_ref,
            "name": rota.name,
            "description": rota.description,
            "period_days": rota.period_days,
            "starts_on": rota.starts_on,
            "members": members.len(),
            "on_call_today": now.map(|w| serde_json::json!({
                "worker_pid": w, "name": name_of(&people, Some(w)),
            })),
        }));
    }
    let total = out.len() as u64;
    Ok(with_page_headers(format::json(out)?, total, total, 0))
}

/// Query for a rota's schedule.
#[derive(Debug, Deserialize)]
struct WindowQuery {
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
}

/// `GET /api/rotas/{pid}?from=&to=` — the rota: members in order, swaps,
/// the schedule as runs (who is on call when, and why), days-on-call per
/// member, and who is on call today.
#[debug_handler]
async fn get_rota(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    axum::extract::Query(q): axum::extract::Query<WindowQuery>,
) -> Result<Response> {
    let rota = find_rota(&ctx, &caller, &pid).await?;
    let (from, to) = window(q.from, q.to)?;
    let (members, overrides, assignments, people) = compute(&ctx, &rota, from, to).await?;
    let runs: Vec<serde_json::Value> =
        rules::runs(&assignments).iter().map(|r| run_json(r, &people)).collect();
    let load: Vec<serde_json::Value> = rules::load(&assignments)
        .into_iter()
        .map(|(w, days)| {
            serde_json::json!({ "worker_pid": w, "name": name_of(&people, Some(w)), "days": days })
        })
        .collect();
    let today = Utc::now().date_naive();
    let on_call_today = assignments
        .iter()
        .find(|a| a.date == today)
        .and_then(|a| a.worker)
        .map(|w| serde_json::json!({ "worker_pid": w, "name": name_of(&people, Some(w)) }));
    format::json(serde_json::json!({
        "pid": rota.pid,
        "organization_ref": rota.organization_ref,
        "name": rota.name,
        "description": rota.description,
        "period_days": rota.period_days,
        "starts_on": rota.starts_on,
        "members": members.iter().enumerate().map(|(i, w)| serde_json::json!({
            "position": i + 1,
            "worker_pid": w,
            "name": name_of(&people, Some(*w)),
            "job_title": people.get(w).map(|p| p.job_title.clone()),
        })).collect::<Vec<_>>(),
        "overrides": overrides.iter().map(|o| serde_json::json!({
            "pid": o.pid,
            "worker_pid": o.worker_pid,
            "worker_name": name_of(&people, Some(o.worker_pid)),
            "starts_on": o.starts_on,
            "ends_on": o.ends_on,
            "note": o.note,
        })).collect::<Vec<_>>(),
        "window": { "from": from, "to": to },
        "runs": runs,
        "load": load,
        "on_call_today": on_call_today,
    }))
}

/// Query naming a day.
#[derive(Debug, Deserialize)]
struct OnQuery {
    on: Option<NaiveDate>,
}

/// `GET /api/rotas/{pid}/on-call?on=` — who is on call on a day (default
/// today) and why; `worker_pid` is null when nobody is available.
#[debug_handler]
async fn on_call(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    axum::extract::Query(q): axum::extract::Query<OnQuery>,
) -> Result<Response> {
    let rota = find_rota(&ctx, &caller, &pid).await?;
    let on = q.on.unwrap_or_else(|| Utc::now().date_naive());
    let (_, _, assignments, people) = compute(&ctx, &rota, on, on).await?;
    let a = assignments.first();
    format::json(serde_json::json!({
        "on": on,
        "worker_pid": a.and_then(|a| a.worker),
        "name": name_of(&people, a.and_then(|a| a.worker)),
        "source": a.and_then(|a| a.source),
    }))
}

// ─── Update / delete / swaps ────────────────────────────────────────────────

/// `PUT /api/rotas/{pid}` body — any of these may change; `members`
/// replaces the whole rotation order.
#[derive(Debug, Deserialize)]
struct RotaUpdate {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    period_days: Option<i32>,
    #[serde(default)]
    starts_on: Option<NaiveDate>,
    #[serde(default)]
    members: Option<Vec<Uuid>>,
}

/// `PUT /api/rotas/{pid}` — rename, re-time or re-order a rota.
#[debug_handler]
async fn update_rota(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<RotaUpdate>,
) -> Result<Response> {
    let rota = find_rota(&ctx, &caller, &pid).await?;
    let current = member_pids(&ctx, rota.pid).await?;
    let members = payload.members.clone().unwrap_or(current);
    let period = payload.period_days.unwrap_or(rota.period_days);
    rules::validate_rota(i64::from(period), &members).map_err(|e| unprocessable(&e))?;
    if payload.members.is_some() {
        check_members(&ctx, &rota.organization_ref, &members).await?;
    }
    if payload.name.as_deref().is_some_and(|n| n.trim().is_empty()) {
        return Err(unprocessable("name must not be blank"));
    }
    let txn = ctx.db.begin().await?;
    let mut active: rotas::ActiveModel = rota.clone().into();
    if let Some(name) = payload.name {
        active.name = ActiveValue::set(name.trim().to_string());
    }
    if let Some(description) = payload.description {
        let d = description.trim().to_string();
        active.description = ActiveValue::set(if d.is_empty() { None } else { Some(d) });
    }
    active.period_days = ActiveValue::set(period);
    if let Some(starts_on) = payload.starts_on {
        active.starts_on = ActiveValue::set(starts_on);
    }
    active.update(&txn).await?;
    if payload.members.is_some() {
        write_members(&txn, rota.pid, &members).await?;
    }
    Audit::record(&txn, "rota", rota.pid, "updated", caller.actor(), None).await?;
    txn.commit().await?;
    format::json(serde_json::json!({ "pid": rota.pid }))
}

/// `DELETE /api/rotas/{pid}` — retire a rota (soft-delete; swaps and
/// membership go with it).
#[debug_handler]
async fn delete_rota(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let rota = find_rota(&ctx, &caller, &pid).await?;
    let txn = ctx.db.begin().await?;
    rota_members::Entity::delete_many()
        .filter(rota_members::Column::RotaPid.eq(rota.pid))
        .exec(&txn)
        .await?;
    rota_overrides::Entity::delete_many()
        .filter(rota_overrides::Column::RotaPid.eq(rota.pid))
        .exec(&txn)
        .await?;
    let rota_pid = rota.pid;
    let mut active: rotas::ActiveModel = rota.into();
    active.deleted_at = ActiveValue::set(Some(Utc::now().into()));
    active.update(&txn).await?;
    Audit::record(&txn, "rota", rota_pid, "retired", caller.actor(), None).await?;
    txn.commit().await?;
    format::empty_json()
}

/// `POST /api/rotas/{pid}/overrides` body.
#[derive(Debug, Deserialize)]
struct OverridePayload {
    worker_pid: Uuid,
    starts_on: NaiveDate,
    ends_on: NaiveDate,
    #[serde(default)]
    note: Option<String>,
}

/// `POST /api/rotas/{pid}/overrides` — a swap: `worker_pid` is on call for
/// the window regardless of the rotation. A later swap wins over an earlier
/// one on the days they share.
#[debug_handler]
async fn add_override(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<OverridePayload>,
) -> Result<Response> {
    let rota = find_rota(&ctx, &caller, &pid).await?;
    if payload.ends_on < payload.starts_on {
        return Err(unprocessable("ends_on must not be before starts_on"));
    }
    check_members(&ctx, &rota.organization_ref, &[payload.worker_pid]).await?;
    let row = rota_overrides::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        rota_pid: ActiveValue::set(rota.pid),
        worker_pid: ActiveValue::set(payload.worker_pid),
        starts_on: ActiveValue::set(payload.starts_on),
        ends_on: ActiveValue::set(payload.ends_on),
        note: ActiveValue::set(
            payload.note.map(|n| n.trim().to_string()).filter(|n| !n.is_empty()),
        ),
        created_by: ActiveValue::set(caller.actor().map(ToString::to_string)),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;
    Audit::record(&ctx.db, "rota", rota.pid, "swap_added", caller.actor(), None).await?;
    format::json(serde_json::json!({ "pid": row.pid }))
}

/// `DELETE /api/rota-overrides/{pid}` — undo a swap.
#[debug_handler]
async fn delete_override(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let row = rota_overrides::Entity::find()
        .filter(rota_overrides::Column::Pid.eq(records::parse_pid(&pid)?))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    let rota = find_rota(&ctx, &caller, &row.rota_pid.to_string()).await?;
    rota_overrides::Entity::delete_by_id(row.id).exec(&ctx.db).await?;
    Audit::record(&ctx.db, "rota", rota.pid, "swap_removed", caller.actor(), None).await?;
    format::empty_json()
}

/// `GET /api/workers/{pid}/on-call?from=&to=` — a worker's on-call stretches
/// across the rotas the caller can read, in date order.
#[debug_handler]
async fn worker_on_call(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    axum::extract::Query(q): axum::extract::Query<WindowQuery>,
) -> Result<Response> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    let (from, to) = window(q.from, q.to)?;
    let mut query = rotas::Entity::find().filter(rotas::Column::DeletedAt.is_null());
    if let Some(refs) = memberships::scope_organization_refs(&ctx.db, caller.claims()).await? {
        query = query.filter(rotas::Column::OrganizationRef.is_in(refs));
    }
    let mut out: BTreeMap<(NaiveDate, String), serde_json::Value> = BTreeMap::new();
    for rota in query.all(&ctx.db).await? {
        let (_, _, assignments, _) = compute(&ctx, &rota, from, to).await?;
        for run in rules::runs(&assignments) {
            if run.worker == Some(worker.pid) {
                out.insert(
                    (run.from, rota.name.clone()),
                    serde_json::json!({
                        "rota_pid": rota.pid,
                        "rota_name": rota.name,
                        "from": run.from,
                        "to": run.to,
                        "source": run.source,
                    }),
                );
            }
        }
    }
    format::json(out.into_values().collect::<Vec<_>>())
}

/// Who is on call **today** in the rotas of the caller's organizations:
/// worker pid → the names of the rotas they are on call for. One entry per
/// rota-day, so the directory can show it without computing a schedule.
///
/// # Errors
///
/// Any query error.
pub(super) async fn on_call_today(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
) -> Result<HashMap<Uuid, Vec<String>>> {
    let mut query = rotas::Entity::find().filter(rotas::Column::DeletedAt.is_null());
    if let Some(refs) = memberships::scope_organization_refs(&ctx.db, caller.claims()).await? {
        query = query.filter(rotas::Column::OrganizationRef.is_in(refs));
    }
    let today = Utc::now().date_naive();
    let mut out: HashMap<Uuid, Vec<String>> = HashMap::new();
    for rota in query.order_by_asc(rotas::Column::Name).all(&ctx.db).await? {
        let (_, _, assignments, _) = compute(ctx, &rota, today, today).await?;
        if let Some(worker) = assignments.first().and_then(|a| a.worker) {
            out.entry(worker).or_default().push(rota.name.clone());
        }
    }
    Ok(out)
}

/// The on-call rota routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/rotas", post(create_rota))
        .add("/rotas", get(list_rotas))
        .add("/rotas/{pid}", get(get_rota))
        .add("/rotas/{pid}", put(update_rota))
        .add("/rotas/{pid}", delete(delete_rota))
        .add("/rotas/{pid}/on-call", get(on_call))
        .add("/rotas/{pid}/overrides", post(add_override))
        .add("/rota-overrides/{pid}", delete(delete_override))
        .add("/workers/{pid}/on-call", get(worker_on_call))
}
