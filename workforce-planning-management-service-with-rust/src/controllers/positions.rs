//! Positions (WPM-R152, WPM-D104): budgeted seats, their moves, their links and the establishment per cost centre. A position
//! holds no pay. Who holds it is HR's to see: other callers get the state and the count, not the holder. Spec: `spec/positions.md`.

use chrono::NaiveDate;
use loco_rs::prelude::*;
use sea_orm::{ActiveValue, QueryOrder, TransactionTrait};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use super::pay_transparency::organization_problems;
use super::{ensure_org_in_scope, ensure_valid, row_json, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::models::_entities::{plan_demand_lines, positions, requisitions, workers};
use crate::models::audit_logs::Model as Audit;
use crate::rules::equality_monitoring::DEFAULT_FLOOR;
use crate::rules::position::{self as rules, Facts, Holding, Seat};

/// Whether the caller may see who holds a position: HR, payroll, service and administrators, or anyone when sign-in is off.
async fn sees_holders(ctx: &AppContext, caller: &MaybeAuthUser, organization: &str) -> Result<bool> {
    if !crate::auth::require_auth() {
        return Ok(true);
    }
    let Some(claims) = caller.claims() else {
        return Ok(false);
    };
    Ok(crate::need_to_know::has_privileged_attrs(claims)
        || crate::need_to_know::privileged_by_membership(ctx, claims, Some(organization)).await?)
}

fn shown(row: &positions::Model, with_holder: bool) -> Result<serde_json::Value> {
    let mut out = row_json(row)?;
    let facts_holder = row.holder_worker_pid.is_some();
    if let Some(object) = out.as_object_mut() {
        object.remove("recorded_by");
        if !with_holder {
            // A manager sees that a seat is held, not by whom.
            object.remove("holder_worker_pid");
            object.insert("held".into(), json!(facts_holder));
        }
    }
    Ok(out)
}

#[derive(Debug, Deserialize)]
pub(crate) struct ListQuery {
    organization: String,
    state: Option<String>,
    cost_centre: Option<String>,
}

/// `GET /api/positions?organization=&state=&cost_centre=` — the positions, by cost centre and title.
#[debug_handler]
async fn list(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(query): axum::extract::Query<ListQuery>,
) -> Result<Response> {
    ensure_valid(&organization_problems(&query.organization))?;
    ensure_org_in_scope(&ctx, &caller, &query.organization).await?;
    let mut find = positions::Entity::find().filter(positions::Column::OrganizationRef.eq(&query.organization));
    if let Some(state) = &query.state {
        if !rules::STATES.contains(&state.as_str()) {
            return Err(unprocessable(&format!("state is one of {}", rules::STATES.join(", "))));
        }
        find = find.filter(positions::Column::State.eq(state));
    }
    if let Some(cc) = &query.cost_centre {
        find = find.filter(positions::Column::CostCentre.eq(cc));
    }
    let rows = find
        .order_by_asc(positions::Column::CostCentre)
        .order_by_asc(positions::Column::Title)
        .order_by_asc(positions::Column::Id)
        .all(&ctx.db)
        .await?;
    let with_holder = sees_holders(&ctx, &caller, &query.organization).await?;
    format::json(json!({
        "states": rules::STATES,
        "positions": rows.iter().map(|r| shown(r, with_holder)).collect::<Result<Vec<_>>>()?,
    }))
}

#[derive(Debug, Deserialize)]
pub(crate) struct Payload {
    organization_ref: Option<String>,
    cost_centre: String,
    title: String,
    budget_fte_percent: i32,
    funding_kind: String,
    funding_ends_on: Option<NaiveDate>,
    programme_ref: Option<String>,
    /// The plan demand line this position came from, if any. Recorded when a person chooses it, never silently.
    demand_line_pid: Option<Uuid>,
}

fn problems(p: &Payload) -> Vec<String> {
    let mut out = rules::validate(&p.cost_centre, p.budget_fte_percent, &p.funding_kind, p.funding_ends_on, p.programme_ref.as_deref())
        .err()
        .unwrap_or_default();
    if !(1..=120).contains(&p.title.trim().chars().count()) {
        out.push("title is 1 to 120 characters".to_string());
    }
    out
}

async fn ensure_demand_line(ctx: &AppContext, pid: Option<Uuid>) -> Result<()> {
    if let Some(pid) = pid
        && plan_demand_lines::Entity::find()
            .filter(plan_demand_lines::Column::Pid.eq(pid))
            .one(&ctx.db)
            .await?
            .is_none()
    {
        return Err(unprocessable("demand_line_pid names no demand line"));
    }
    Ok(())
}

/// `POST /api/positions` — plan a seat. It starts `planned`; a person approves it.
#[debug_handler]
async fn create(State(ctx): State<AppContext>, caller: MaybeAuthUser, Json(payload): Json<Payload>) -> Result<Response> {
    let organization = payload.organization_ref.clone().unwrap_or_default();
    ensure_valid(&organization_problems(&organization))?;
    ensure_org_in_scope(&ctx, &caller, &organization).await?;
    ensure_valid(&problems(&payload))?;
    ensure_demand_line(&ctx, payload.demand_line_pid).await?;
    let pid = Uuid::new_v4();
    let txn = ctx.db.begin().await?;
    positions::ActiveModel {
        pid: ActiveValue::set(pid),
        organization_ref: ActiveValue::set(organization),
        cost_centre: ActiveValue::set(payload.cost_centre.trim().to_string()),
        title: ActiveValue::set(payload.title.trim().to_string()),
        budget_fte_percent: ActiveValue::set(payload.budget_fte_percent),
        funding_kind: ActiveValue::set(payload.funding_kind.clone()),
        funding_ends_on: ActiveValue::set(payload.funding_ends_on),
        programme_ref: ActiveValue::set(payload.programme_ref.clone()),
        state: ActiveValue::set("planned".to_string()),
        demand_line_pid: ActiveValue::set(payload.demand_line_pid),
        recorded_by: ActiveValue::set(caller.actor().map(ToString::to_string)),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    Audit::record(&txn, "position", pid, "position_created", caller.actor(), Some(json!({ "from_demand_line": payload.demand_line_pid.is_some() }))).await?;
    txn.commit().await?;
    format::json(json!({ "pid": pid }))
}

async fn find(ctx: &AppContext, caller: &MaybeAuthUser, pid: Uuid) -> Result<positions::Model> {
    let row = positions::Entity::find()
        .filter(positions::Column::Pid.eq(pid))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    ensure_org_in_scope(ctx, caller, &row.organization_ref).await?;
    Ok(row)
}

/// `PUT /api/positions/{pid}` — change what a position is made of. The organization, the state and the links do not change here.
#[debug_handler]
async fn update(State(ctx): State<AppContext>, caller: MaybeAuthUser, Path(pid): Path<Uuid>, Json(payload): Json<Payload>) -> Result<Response> {
    let row = find(&ctx, &caller, pid).await?;
    if payload.organization_ref.as_ref().is_some_and(|o| *o != row.organization_ref) {
        return Err(unprocessable("a position cannot move to another organization"));
    }
    ensure_valid(&problems(&payload))?;
    if row.state == "abolished" {
        return Err(unprocessable("position is \"abolished\"; an ended position is not changed"));
    }
    let txn = ctx.db.begin().await?;
    let mut active: positions::ActiveModel = row.into();
    active.cost_centre = ActiveValue::set(payload.cost_centre.trim().to_string());
    active.title = ActiveValue::set(payload.title.trim().to_string());
    active.budget_fte_percent = ActiveValue::set(payload.budget_fte_percent);
    active.funding_kind = ActiveValue::set(payload.funding_kind.clone());
    active.funding_ends_on = ActiveValue::set(payload.funding_ends_on);
    active.programme_ref = ActiveValue::set(payload.programme_ref.clone());
    active.recorded_by = ActiveValue::set(caller.actor().map(ToString::to_string));
    active.update(&txn).await?;
    Audit::record(&txn, "position", pid, "position_updated", caller.actor(), None).await?;
    txn.commit().await?;
    format::json(json!({ "pid": pid }))
}

#[derive(Debug, Deserialize)]
struct MovePayload {
    to: String,
}

/// `POST /api/positions/{pid}/status` — move a position. The rules refuse a move the table does not allow, an `open` with no
/// requisition, a `filled` with no holder, and a held position being made vacant or abolished.
#[debug_handler]
async fn status(State(ctx): State<AppContext>, caller: MaybeAuthUser, Path(pid): Path<Uuid>, Json(payload): Json<MovePayload>) -> Result<Response> {
    let row = find(&ctx, &caller, pid).await?;
    let facts = Facts { has_requisition: row.requisition_pid.is_some(), has_holder: row.holder_worker_pid.is_some() };
    rules::check_move(&row.state, &payload.to, facts).map_err(|e| unprocessable(&e))?;
    let from = row.state.clone();
    let txn = ctx.db.begin().await?;
    let mut active: positions::ActiveModel = row.into();
    active.state = ActiveValue::set(payload.to.clone());
    active.update(&txn).await?;
    Audit::record(&txn, "position", pid, "position_status_changed", caller.actor(), Some(json!({ "from": from, "to": payload.to }))).await?;
    txn.commit().await?;
    format::json(json!({ "pid": pid, "state": payload.to }))
}

#[derive(Debug, Deserialize)]
struct HolderPayload {
    /// The worker who holds the position, or `null` to unlink.
    worker_pid: Option<Uuid>,
}

/// `PUT /api/positions/{pid}/holder` — link the worker who holds the position, or unlink. The worker must work in the same
/// organization. HR only to read back; the link is the only place a person appears.
#[debug_handler]
async fn holder(State(ctx): State<AppContext>, caller: MaybeAuthUser, Path(pid): Path<Uuid>, Json(payload): Json<HolderPayload>) -> Result<Response> {
    let row = find(&ctx, &caller, pid).await?;
    if row.state == "abolished" || row.state == "planned" {
        return Err(unprocessable(&format!("position is {:?}; only an approved position can have a holder", row.state)));
    }
    if let Some(worker) = payload.worker_pid {
        let found = workers::Entity::find()
            .filter(workers::Column::Pid.eq(worker))
            .filter(workers::Column::DeletedAt.is_null())
            .one(&ctx.db)
            .await?;
        if found.is_none_or(|w| w.organization_ref != row.organization_ref) {
            return Err(unprocessable("worker_pid names no worker of this organization"));
        }
    }
    let txn = ctx.db.begin().await?;
    let mut active: positions::ActiveModel = row.into();
    active.holder_worker_pid = ActiveValue::set(payload.worker_pid);
    active.recorded_by = ActiveValue::set(caller.actor().map(ToString::to_string));
    active.update(&txn).await?;
    Audit::record(&txn, "position", pid, "position_holder_set", caller.actor(), Some(json!({ "linked": payload.worker_pid.is_some() }))).await?;
    txn.commit().await?;
    format::json(json!({ "pid": pid }))
}

#[derive(Debug, Deserialize)]
struct RequisitionPayload {
    /// The requisition that fills the position, or `null` to unlink.
    requisition_pid: Option<Uuid>,
}

/// `PUT /api/positions/{pid}/requisition` — link the requisition that fills the position, or unlink. Same organization.
#[debug_handler]
async fn requisition(State(ctx): State<AppContext>, caller: MaybeAuthUser, Path(pid): Path<Uuid>, Json(payload): Json<RequisitionPayload>) -> Result<Response> {
    let row = find(&ctx, &caller, pid).await?;
    if row.state == "abolished" {
        return Err(unprocessable("position is \"abolished\"; an ended position is not changed"));
    }
    if row.state == "open" && payload.requisition_pid.is_none() {
        return Err(unprocessable("position is \"open\"; move it back before unlinking its requisition"));
    }
    if let Some(r) = payload.requisition_pid {
        let found = requisitions::Entity::find()
            .filter(requisitions::Column::Pid.eq(r))
            .filter(requisitions::Column::DeletedAt.is_null())
            .one(&ctx.db)
            .await?;
        if found.is_none_or(|q| q.organization_ref != row.organization_ref) {
            return Err(unprocessable("requisition_pid names no requisition of this organization"));
        }
    }
    let txn = ctx.db.begin().await?;
    let mut active: positions::ActiveModel = row.into();
    active.requisition_pid = ActiveValue::set(payload.requisition_pid);
    active.recorded_by = ActiveValue::set(caller.actor().map(ToString::to_string));
    active.update(&txn).await?;
    Audit::record(&txn, "position", pid, "position_requisition_set", caller.actor(), Some(json!({ "linked": payload.requisition_pid.is_some() }))).await?;
    txn.commit().await?;
    format::json(json!({ "pid": pid }))
}

#[derive(Debug, Deserialize)]
struct EstablishmentQuery {
    organization: String,
}

/// `GET /api/positions/establishment?organization=` — per cost centre: positions, budget, held, vacant, over, the vacancy in frozen
/// positions and positions by state. No holder is in it. A caller who may not see who holds what gets held, vacant and over
/// withheld for a centre under the small-group floor.
#[debug_handler]
async fn establishment(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(query): axum::extract::Query<EstablishmentQuery>,
) -> Result<Response> {
    ensure_valid(&organization_problems(&query.organization))?;
    ensure_org_in_scope(&ctx, &caller, &query.organization).await?;
    let rows = positions::Entity::find()
        .filter(positions::Column::OrganizationRef.eq(&query.organization))
        .all(&ctx.db)
        .await?;
    let held: Vec<Uuid> = rows.iter().filter_map(|r| r.holder_worker_pid).collect();
    let fte: std::collections::HashMap<Uuid, i32> = super::find_in(&ctx, workers::Entity::find().filter(workers::Column::DeletedAt.is_null()), workers::Column::Pid, &held)
        .await?
        .into_iter()
        .map(|w| (w.pid, w.fte_percent))
        .collect();
    let seats: Vec<Seat> = rows
        .iter()
        .map(|r| Seat {
            cost_centre: r.cost_centre.clone(),
            state: r.state.clone(),
            budget_fte_percent: r.budget_fte_percent,
            // A holder whose record is gone (erased or deleted) is held by someone whose FTE we cannot say.
            holding: match r.holder_worker_pid {
                None => Holding::Vacant,
                Some(w) => fte.get(&w).map_or(Holding::HeldUnknown, |f| Holding::Held(*f)),
            },
        })
        .collect();
    let floor = if sees_holders(&ctx, &caller, &query.organization).await? { None } else { Some(DEFAULT_FLOOR) };
    format::json(json!({
        "organization": query.organization,
        "small_group_floor": floor,
        "cost_centres": rules::establishment(&seats, floor),
        "derivation": "The establishment is the positions that are approved, open, filled or frozen. Held is the FTE of the workers linked as holders; \\
                       vacant is budget not held; over is FTE held beyond budget. Over-establishment on one position is never netted against a \\
                       vacancy on another. A holder whose FTE cannot be read makes held, vacant and over unknown for that cost centre.",
    }))
}

/// The routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/positions", get(list))
        .add("/positions", post(create))
        .add("/positions/establishment", get(establishment))
        .add("/positions/{pid}", put(update))
        .add("/positions/{pid}/status", post(status))
        .add("/positions/{pid}/holder", put(holder))
        .add("/positions/{pid}/requisition", put(requisition))
}
