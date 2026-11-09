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

use super::ensure_org_in_scope;
use axa::extract::{Json, Path};
use axa::prelude::*;
use chrono::{Duration, NaiveDate, Utc};
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use uuid::Uuid;

use super::unprocessable;
use crate::auth::MaybeAuthUser;
use crate::db;
use crate::models::_entities::{rota_overrides, rotas};
use crate::models::{memberships, records};
use crate::rules::metrics as metric_rules;
use crate::rules::notify;
use crate::rules::rota::{self as rules, Assignment, Override, Rota};

/// Days shown when the caller names no window.
const DEFAULT_WINDOW_DAYS: i64 = 28;

/// The rota, if it exists and its organization is one the caller can read.
pub(super) async fn find_rota(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: &str,
) -> Result<rotas::Model> {
    let rota = sqlx::query_as!(
        rotas::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, organization_ref, name, description,
                  period_days, starts_on, deleted_at AS "deleted_at: _"
             FROM rotas WHERE pid = $1 AND deleted_at IS NULL"#,
        records::parse_pid(pid)?,
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)?;
    require_scope(ctx, caller, &rota.organization_ref).await?;
    Ok(rota)
}

/// A rota outside the caller's organizations does not exist, to them.
pub(super) async fn require_scope(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    org: &str,
) -> Result<()> {
    ensure_org_in_scope(ctx, caller, org).await?;
    Ok(())
}

/// The live rotas in `scope` (all when `None`), by organization and name.
async fn rotas_in(ctx: &AppContext, scope: Option<&[String]>) -> Result<Vec<rotas::Model>> {
    Ok(sqlx::query_as!(
        rotas::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, organization_ref, name, description,
                  period_days, starts_on, deleted_at AS "deleted_at: _"
             FROM rotas WHERE deleted_at IS NULL AND ($1::text[] IS NULL OR organization_ref = ANY($1))
            ORDER BY organization_ref, name, id"#,
        scope,
    )
    .fetch_all(db::pool(ctx))
    .await?)
}

async fn member_pids(ctx: &AppContext, rota: Uuid) -> Result<Vec<Uuid>> {
    Ok(sqlx::query_scalar!(
        "SELECT worker_pid FROM rota_members WHERE rota_pid = $1 ORDER BY position",
        rota
    )
    .fetch_all(db::pool(ctx))
    .await?)
}

async fn override_rows(ctx: &AppContext, rota: Uuid) -> Result<Vec<rota_overrides::Model>> {
    Ok(sqlx::query_as!(
        rota_overrides::Model,
        r#"SELECT created_at AS "created_at: _", id, pid, rota_pid, worker_pid, starts_on, ends_on, note, created_by
             FROM rota_overrides WHERE rota_pid = $1 ORDER BY id"#,
        rota,
    )
    .fetch_all(db::pool(ctx))
    .await?)
}

/// A person in a rota, as the schedule and its views read them.
pub(super) struct RotaPerson {
    pub(super) display_name: String,
    pub(super) job_title: String,
    pub(super) organization_ref: String,
    pub(super) hired_on: NaiveDate,
    pub(super) terminated_on: Option<NaiveDate>,
}

/// Live workers by pid, for names and availability.
pub(super) async fn workers_by_pid(
    ctx: &AppContext,
    pids: Vec<Uuid>,
) -> Result<HashMap<Uuid, RotaPerson>> {
    Ok(sqlx::query!(
        "SELECT pid, display_name, job_title, organization_ref, hired_on, terminated_on FROM workers
          WHERE pid = ANY($1) AND deleted_at IS NULL",
        &pids,
    )
    .fetch_all(db::pool(ctx))
    .await?
    .into_iter()
    .map(|w| {
        (
            w.pid,
            RotaPerson {
                display_name: w.display_name,
                job_title: w.job_title,
                organization_ref: w.organization_ref,
                hired_on: w.hired_on,
                terminated_on: w.terminated_on,
            },
        )
    })
    .collect())
}

/// The day-by-day schedule for a rota over `from..=to`, with the workers
/// named in it. Availability = employed that day and not on approved leave.
pub(super) async fn compute(
    ctx: &AppContext,
    rota: &rotas::Model,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<(
    Vec<Uuid>,
    Vec<rota_overrides::Model>,
    Vec<Assignment>,
    HashMap<Uuid, RotaPerson>,
)> {
    let members = member_pids(ctx, rota.pid).await?;
    let overrides = override_rows(ctx, rota.pid).await?;
    let mut ids = members.clone();
    ids.extend(overrides.iter().map(|o| o.worker_pid));
    let people = workers_by_pid(ctx, ids.clone()).await?;
    let mut away: HashMap<Uuid, Vec<(NaiveDate, NaiveDate)>> = HashMap::new();
    for l in sqlx::query!(
        "SELECT worker_pid, start_on, end_on FROM leave_requests
          WHERE worker_pid = ANY($1) AND status = 'approved' AND deleted_at IS NULL AND start_on <= $2 AND end_on >= $3",
        &ids,
        to,
        from,
    )
    .fetch_all(db::pool(ctx))
    .await?
    {
        away.entry(l.worker_pid)
            .or_default()
            .push((l.start_on, l.end_on));
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
        .map(|o| Override {
            worker: o.worker_pid,
            starts_on: o.starts_on,
            ends_on: o.ends_on,
        })
        .collect();
    let assignments = rules::schedule(&model, &swaps, &unavailable, from, to);
    Ok((members, overrides, assignments, people))
}

pub(super) fn name_of(people: &HashMap<Uuid, RotaPerson>, pid: Option<Uuid>) -> Option<String> {
    pid.and_then(|p| people.get(&p))
        .map(|w| w.display_name.clone())
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
            "the window may cover at most {} calendar days",
            rules::MAX_WINDOW_DAYS
        )));
    }
    Ok((from, to))
}

fn run_json(r: &rules::Run, people: &HashMap<Uuid, RotaPerson>) -> serde_json::Value {
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
pub(super) async fn check_members(ctx: &AppContext, org: &str, members: &[Uuid]) -> Result<()> {
    let people = workers_by_pid(ctx, members.to_vec()).await?;
    let today = Utc::now().date_naive();
    for m in members {
        let Some(w) = people.get(m) else {
            return Err(unprocessable("a member is not a worker here"));
        };
        if w.organization_ref != org {
            return Err(unprocessable(
                "members must belong to the rota's organization",
            ));
        }
        if !metric_rules::is_employed_on(today, w.hired_on, w.terminated_on) {
            return Err(unprocessable("a member is not currently employed"));
        }
    }
    Ok(())
}

async fn write_members(tx: &mut sqlx::PgConnection, rota: Uuid, members: &[Uuid]) -> Result<()> {
    sqlx::query!("DELETE FROM rota_members WHERE rota_pid = $1", rota)
        .execute(&mut *tx)
        .await?;
    // The rotation order in one statement, each at its place from 0.
    sqlx::query!(
        "INSERT INTO rota_members (rota_pid, worker_pid, position)
         SELECT $1, member.worker_pid, (member.n - 1)::int FROM unnest($2::uuid[]) WITH ORDINALITY AS member(worker_pid, n)",
        rota,
        members,
    )
    .execute(&mut *tx)
    .await?;
    Ok(())
}

/// Tell newly named members they are in the rota (reference-only: the rota's
/// name, nothing about anyone's availability or leave).
async fn notify_added(
    tx: &mut sqlx::PgConnection,
    rota: &rotas::Model,
    recipients: &[Uuid],
) -> Result<()> {
    for worker in recipients {
        db::notify(
            tx,
            *worker,
            "rota_added",
            &format!("You were added to the on-call rota {}.", rota.name),
            serde_json::json!({ "rota_pid": rota.pid }),
        )
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
    let mut tx = db::pool(&ctx).begin().await?;
    let row = sqlx::query_as!(
        rotas::Model,
        r#"INSERT INTO rotas (pid, organization_ref, name, description, period_days, starts_on)
           VALUES ($1, $2, $3, $4, $5, $6)
           RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, organization_ref, name, description,
                  period_days, starts_on, deleted_at AS "deleted_at: _""#,
        Uuid::new_v4(),
        payload.organization_ref,
        payload.name.trim(),
        payload
            .description
            .map(|d| d.trim().to_string())
            .filter(|d| !d.is_empty()),
        payload.period_days,
        payload.starts_on,
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        if e.as_database_error().and_then(|d| d.constraint()) == Some("rotas_org_name") {
            unprocessable("a rota with that name already exists in this organization")
        } else {
            e.into()
        }
    })?;
    write_members(&mut tx, row.pid, &payload.members).await?;
    notify_added(
        &mut tx,
        &row,
        &notify::rota_added_recipients(&[], &payload.members),
    )
    .await?;
    db::audit(&mut tx, "rota", row.pid, "created", caller.actor(), None).await?;
    tx.commit().await?;
    format::json(serde_json::json!({ "pid": row.pid }))
}

/// `GET /api/rotas` — rotas in the caller's organizations, each with who is
/// on call today.
#[debug_handler]
async fn list_rotas(State(ctx): State<AppContext>, caller: MaybeAuthUser) -> Result<Response> {
    let scope = memberships::scope_organization_refs(&ctx.db, caller.claims()).await?;
    let all = rotas_in(&ctx, scope.as_deref()).await?;
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
    // Not paged: everything, with no `Link`, so a client that pages sees it is whole.
    Ok(axum::Json(out).into_response())
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
    axa::extract::Query(q): axa::extract::Query<WindowQuery>,
) -> Result<Response> {
    let rota = find_rota(&ctx, &caller, &pid).await?;
    let (from, to) = window(q.from, q.to)?;
    let (members, overrides, assignments, people) = compute(&ctx, &rota, from, to).await?;
    let runs: Vec<serde_json::Value> = rules::runs(&assignments)
        .iter()
        .map(|r| run_json(r, &people))
        .collect();
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
    axa::extract::Query(q): axa::extract::Query<OnQuery>,
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
    let members = payload.members.clone().unwrap_or_else(|| current.clone());
    let period = payload.period_days.unwrap_or(rota.period_days);
    rules::validate_rota(i64::from(period), &members).map_err(|e| unprocessable(&e))?;
    if payload.members.is_some() {
        check_members(&ctx, &rota.organization_ref, &members).await?;
    }
    if payload.name.as_deref().is_some_and(|n| n.trim().is_empty()) {
        return Err(unprocessable("name must not be blank"));
    }
    let name = payload
        .name
        .map_or_else(|| rota.name.clone(), |n| n.trim().to_string());
    let description = match payload.description {
        Some(d) => Some(d.trim().to_string()).filter(|d| !d.is_empty()),
        None => rota.description.clone(),
    };
    let starts_on = payload.starts_on.unwrap_or(rota.starts_on);
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "UPDATE rotas SET name = $2, description = $3, period_days = $4, starts_on = $5 WHERE id = $1",
        rota.id,
        name,
        description,
        period,
        starts_on,
    )
    .execute(&mut *tx)
    .await?;
    if payload.members.is_some() {
        write_members(&mut tx, rota.pid, &members).await?;
        // `current` was read before the transaction opened: reading again here
        // would wait on a second pooled connection while this one is held.
        notify_added(
            &mut tx,
            &rota,
            &notify::rota_added_recipients(&current, &members),
        )
        .await?;
    }
    db::audit(&mut tx, "rota", rota.pid, "updated", caller.actor(), None).await?;
    tx.commit().await?;
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
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!("DELETE FROM rota_members WHERE rota_pid = $1", rota.pid)
        .execute(&mut *tx)
        .await?;
    sqlx::query!("DELETE FROM rota_overrides WHERE rota_pid = $1", rota.pid)
        .execute(&mut *tx)
        .await?;
    sqlx::query!(
        "UPDATE rotas SET deleted_at = $2 WHERE id = $1",
        rota.id,
        Utc::now()
    )
    .execute(&mut *tx)
    .await?;
    db::audit(&mut tx, "rota", rota.pid, "retired", caller.actor(), None).await?;
    tx.commit().await?;
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
    let swap = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO rota_overrides (pid, rota_pid, worker_pid, starts_on, ends_on, note, created_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
        swap,
        rota.pid,
        payload.worker_pid,
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
        payload.worker_pid,
        "on_call_swap",
        &format!(
            "You are on call for {} from {} to {}.",
            rota.name, payload.starts_on, payload.ends_on
        ),
        serde_json::json!({
            "rota_pid": rota.pid,
            "from": payload.starts_on,
            "to": payload.ends_on,
        }),
    )
    .await?;
    db::audit(
        &mut tx,
        "rota",
        rota.pid,
        "swap_added",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(serde_json::json!({ "pid": swap }))
}

/// `DELETE /api/rota-overrides/{pid}` — undo a swap.
#[debug_handler]
async fn delete_override(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let row = sqlx::query!(
        "SELECT id, rota_pid FROM rota_overrides WHERE pid = $1",
        records::parse_pid(&pid)?,
    )
    .fetch_optional(db::pool(&ctx))
    .await?
    .ok_or(Error::NotFound)?;
    let rota = find_rota(&ctx, &caller, &row.rota_pid.to_string()).await?;
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!("DELETE FROM rota_overrides WHERE id = $1", row.id)
        .execute(&mut *tx)
        .await?;
    db::audit(
        &mut tx,
        "rota",
        rota.pid,
        "swap_removed",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::empty_json()
}

/// `GET /api/workers/{pid}/on-call?from=&to=` — a worker's on-call stretches
/// across the rotas the caller can read, in date order.
#[debug_handler]
async fn worker_on_call(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    axa::extract::Query(q): axa::extract::Query<WindowQuery>,
) -> Result<Response> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    let (from, to) = window(q.from, q.to)?;
    let scope = memberships::scope_organization_refs(&ctx.db, caller.claims()).await?;
    let mut out: BTreeMap<(NaiveDate, String), serde_json::Value> = BTreeMap::new();
    for rota in rotas_in(&ctx, scope.as_deref()).await? {
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
    let scope = memberships::scope_organization_refs(&ctx.db, caller.claims()).await?;
    let today = Utc::now().date_naive();
    let mut out: HashMap<Uuid, Vec<String>> = HashMap::new();
    // By name, as before: a worker on call in several rotas lists them in name order.
    let mut rotas = rotas_in(ctx, scope.as_deref()).await?;
    rotas.sort_by(|a, b| a.name.cmp(&b.name));
    for rota in rotas {
        let (_, _, assignments, _) = compute(ctx, &rota, today, today).await?;
        if let Some(worker) = assignments.first().and_then(|a| a.worker) {
            out.entry(worker).or_default().push(rota.name.clone());
        }
    }
    Ok(out)
}

/// The on-call turns that **start** on `date` across every live rota (the
/// reminder task's input): `(rota, worker)` where the worker is on call on
/// `date` but was not the day before. Not scoped to a caller — the task runs
/// as the system.
///
/// # Errors
///
/// Any query error.
pub(crate) async fn turns_starting_on(
    ctx: &AppContext,
    date: NaiveDate,
) -> Result<Vec<(rotas::Model, Uuid)>> {
    let mut out = Vec::new();
    let mut all = rotas_in(ctx, None).await?;
    all.sort_by_key(|r| r.id);
    for rota in all {
        let (_, _, assignments, _) = compute(ctx, &rota, date - Duration::days(1), date).await?;
        if let [yesterday, today] = assignments.as_slice()
            && let Some(worker) = rules::turn_starts(yesterday, today)
        {
            out.push((rota, worker));
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
