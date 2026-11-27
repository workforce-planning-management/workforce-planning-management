//! Positions (WPM-R152, WPM-D104): budgeted seats, their moves, their links and the establishment per cost centre. A position
//! holds no pay. Who holds it is HR's to see: other callers get the state and the count, not the holder. Spec: `spec/positions.md`.

use crate::absent::AbsentIsEmpty as _;
use crate::sql_time::SqlDate;
use axa::extract::{Json, Path};
use axa::prelude::*;
use jiff::civil::Date;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use super::pay_transparency::organization_problems;
use super::{ensure_org_in_scope, ensure_valid, row_json, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::db;
use crate::models::_entities::{position_holders, positions};
use crate::rules::equality_monitoring::DEFAULT_FLOOR;
use crate::rules::position::{self as rules, Facts, Holding, Seat};
use crate::rules::position_history::{self as history, Freeze};

/// Whether the caller may see who holds a position: HR, payroll, service and administrators, or anyone when sign-in is off.
async fn sees_holders(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    organization: &str,
) -> Result<bool> {
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
    axa::extract::Query(query): axa::extract::Query<ListQuery>,
) -> Result<Response> {
    ensure_valid(&organization_problems(&query.organization))?;
    ensure_org_in_scope(&ctx, &caller, &query.organization).await?;
    if let Some(state) = &query.state
        && !rules::STATES.contains(&state.as_str())
    {
        return Err(unprocessable(&format!(
            "state is one of {}",
            rules::STATES.join(", ")
        )));
    }
    let rows = sqlx::query_as!(
        positions::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, organization_ref, cost_centre,
                  title, budget_fte_percent, funding_kind, funding_ends_on, programme_ref, state, holder_worker_pid,
                  requisition_pid, demand_line_pid, overtime_status, number, recorded_by, critical, grade_scale, grade_band,
                  grade_pay_point
             FROM positions
            WHERE organization_ref = $1 AND ($2::text IS NULL OR state = $2) AND ($3::text IS NULL OR cost_centre = $3)
            ORDER BY cost_centre, title, id"#,
        query.organization,
        query.state,
        query.cost_centre,
    )
    .fetch_all(db::pool(&ctx))
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
    funding_ends_on: Option<Date>,
    programme_ref: Option<String>,
    /// The plan demand line this position came from, if any. Recorded when a person chooses it, never silently.
    demand_line_pid: Option<Uuid>,
    /// `exempt`, `non_exempt` or `unknown`: a person's entry about statutory overtime, never inferred (WPM-D157). Left out, a new
    /// position is `unknown` and an update keeps what is recorded.
    overtime_status: Option<String>,
    /// The number the post is known by in an establishment list, unique in the organization (WPM-T434). Left out, a new position has
    /// none and an update keeps what is recorded; blank is the same as left out.
    number: Option<String>,
    /// Whether HR marks the post critical (WPM-D237). Left out, a new position is not and an update keeps what is recorded.
    critical: Option<bool>,
}

/// The number as it is kept: trimmed, and blank is none.
fn clean_number(number: Option<&str>) -> Option<String> {
    number
        .map(str::trim)
        .filter(|n| !n.is_empty())
        .map(ToString::to_string)
}

fn problems(p: &Payload) -> Vec<String> {
    let mut out = rules::validate(
        &p.cost_centre,
        p.budget_fte_percent,
        &p.funding_kind,
        p.funding_ends_on,
        p.programme_ref.as_deref(),
    )
    .err()
    .or_empty();
    if !(1..=120).contains(&p.title.trim().chars().count()) {
        out.push("title is 1 to 120 characters".to_string());
    }
    if p.overtime_status
        .as_deref()
        .is_some_and(|s| crate::rules::statutory_overtime::Status::parse(s).is_none())
    {
        out.push("overtime_status must be exempt, non_exempt or unknown".to_string());
    }
    if clean_number(p.number.as_deref()).is_some_and(|n| n.chars().count() > 40) {
        out.push("number is 1 to 40 characters".to_string());
    }
    out
}

/// A number names one position in an organization, so a second use is refused in words, not left to a constraint error.
async fn ensure_number_free(
    ctx: &AppContext,
    organization: &str,
    number: Option<&str>,
    except: Option<Uuid>,
) -> Result<()> {
    let Some(number) = number else {
        return Ok(());
    };
    let taken = sqlx::query_scalar!(
        "SELECT pid FROM positions WHERE organization_ref = $1 AND number = $2 LIMIT 1",
        organization,
        number,
    )
    .fetch_optional(db::pool(ctx))
    .await?;
    if taken.is_some_and(|t| Some(t) != except) {
        return Err(unprocessable(
            "number is already used by another position in this organization",
        ));
    }
    Ok(())
}

async fn ensure_demand_line(ctx: &AppContext, pid: Option<Uuid>) -> Result<()> {
    if let Some(pid) = pid
        && sqlx::query_scalar!(
            "SELECT 1 AS \"one!\" FROM plan_demand_lines WHERE pid = $1",
            pid
        )
        .fetch_optional(db::pool(ctx))
        .await?
        .is_none()
    {
        return Err(unprocessable("demand_line_pid names no demand line"));
    }
    Ok(())
}

/// `POST /api/positions` — plan a seat. It starts `planned`; a person approves it.
#[debug_handler]
async fn create(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<Payload>,
) -> Result<Response> {
    let organization = payload.organization_ref.clone().or_empty();
    ensure_valid(&organization_problems(&organization))?;
    ensure_org_in_scope(&ctx, &caller, &organization).await?;
    ensure_valid(&problems(&payload))?;
    ensure_demand_line(&ctx, payload.demand_line_pid).await?;
    let number = clean_number(payload.number.as_deref());
    ensure_number_free(&ctx, &organization, number.as_deref(), None).await?;
    let pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO positions (pid, organization_ref, cost_centre, title, budget_fte_percent, funding_kind, funding_ends_on,
                                programme_ref, state, overtime_status, demand_line_pid, number, recorded_by, critical)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'planned', $9, $10, $11, $12, $13)",
        pid,
        organization,
        payload.cost_centre.trim(),
        payload.title.trim(),
        payload.budget_fte_percent,
        payload.funding_kind,
        payload.funding_ends_on.map(SqlDate),
        payload.programme_ref,
        payload.overtime_status.as_deref().unwrap_or("unknown"),
        payload.demand_line_pid,
        number,
        caller.actor(),
        payload.critical.unwrap_or(false),
    )
    .execute(&mut *tx)
    .await?;
    record_move(
        &mut tx,
        NewMove {
            position: pid,
            organization: &organization,
            from: None,
            to: "planned",
            freeze: None,
            holder_left: false,
        },
        caller.actor(),
    )
    .await?;
    db::audit(
        &mut tx,
        "position",
        pid,
        "position_created",
        caller.actor(),
        Some(json!({ "from_demand_line": payload.demand_line_pid.is_some(), "critical": payload.critical.unwrap_or(false) })),
    )
    .await?;
    tx.commit().await?;
    format::json(json!({ "pid": pid }))
}

async fn find(ctx: &AppContext, caller: &MaybeAuthUser, pid: Uuid) -> Result<positions::Model> {
    let row = sqlx::query_as!(
        positions::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, organization_ref, cost_centre,
                  title, budget_fte_percent, funding_kind, funding_ends_on, programme_ref, state, holder_worker_pid,
                  requisition_pid, demand_line_pid, overtime_status, number, recorded_by, critical, grade_scale, grade_band,
                  grade_pay_point
             FROM positions WHERE pid = $1"#,
        pid,
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)?;
    ensure_org_in_scope(ctx, caller, &row.organization_ref).await?;
    Ok(row)
}

/// `PUT /api/positions/{pid}` — change what a position is made of. The organization, the state and the links do not change here.
#[debug_handler]
async fn update(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<Uuid>,
    Json(payload): Json<Payload>,
) -> Result<Response> {
    let row = find(&ctx, &caller, pid).await?;
    if payload
        .organization_ref
        .as_ref()
        .is_some_and(|o| *o != row.organization_ref)
    {
        return Err(unprocessable(
            "a position cannot move to another organization",
        ));
    }
    ensure_valid(&problems(&payload))?;
    if row.state == "abolished" {
        return Err(unprocessable(
            "position is \"abolished\"; an ended position is not changed",
        ));
    }
    let number = clean_number(payload.number.as_deref());
    ensure_number_free(&ctx, &row.organization_ref, number.as_deref(), Some(pid)).await?;
    let status_changed = payload
        .overtime_status
        .as_ref()
        .is_some_and(|s| *s != row.overtime_status);
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "UPDATE positions SET cost_centre = $2, title = $3, budget_fte_percent = $4, funding_kind = $5, funding_ends_on = $6,
                programme_ref = $7, overtime_status = COALESCE($8, overtime_status), number = COALESCE($9, number),
                recorded_by = $10, critical = COALESCE($11, critical)
          WHERE id = $1",
        row.id,
        payload.cost_centre.trim(),
        payload.title.trim(),
        payload.budget_fte_percent,
        payload.funding_kind,
        payload.funding_ends_on.map(SqlDate),
        payload.programme_ref,
        payload.overtime_status,
        number,
        caller.actor(),
        payload.critical,
    )
    .execute(&mut *tx)
    .await?;
    let critical_changed = payload.critical.is_some_and(|c| c != row.critical);
    let mut detail = serde_json::Map::new();
    if status_changed {
        detail.insert("overtime_status".into(), json!(payload.overtime_status));
    }
    if critical_changed {
        detail.insert("critical".into(), json!(payload.critical));
    }
    db::audit(
        &mut tx,
        "position",
        pid,
        "position_updated",
        caller.actor(),
        (!detail.is_empty()).then_some(serde_json::Value::Object(detail)),
    )
    .await?;
    tx.commit().await?;
    format::json(json!({ "pid": pid }))
}

/// One row of the state history of a position (WPM-R288), as it is added.
struct NewMove<'a> {
    position: Uuid,
    organization: &'a str,
    /// `None` for the first row of a new position.
    from: Option<&'a str>,
    to: &'a str,
    /// The three facts of a move to `frozen` (WPM-R289).
    freeze: Option<&'a Freeze<'a>>,
    /// The row records the holder being unlinked while the state stays vacant, so the vacancy starts there.
    holder_left: bool,
}

/// Add one row to the state history of a position: the date is today's, and the row is never changed after (a trigger refuses it).
async fn record_move(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    row: NewMove<'_>,
    actor: Option<&str>,
) -> Result<()> {
    let freeze = row.freeze;
    sqlx::query!(
        "INSERT INTO position_state_moves (position_pid, organization_ref, from_state, to_state, moved_on, freeze_reason,
                                           decided_by_role, review_on, holder_left, recorded_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        row.position,
        row.organization,
        row.from,
        row.to,
        SqlDate(crate::dates::today()),
        freeze.and_then(|f| f.reason),
        freeze.and_then(|f| f.decided_by_role).map(str::trim),
        freeze.and_then(|f| f.review_on).map(SqlDate),
        row.holder_left,
        actor,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

#[derive(Debug, Deserialize)]
struct MovePayload {
    to: String,
    /// A move to `frozen` needs these three (WPM-R289): a reason from the list, the role that decided, and a review date in the future.
    reason: Option<String>,
    decided_by_role: Option<String>,
    review_on: Option<Date>,
}

/// `POST /api/positions/{pid}/status` — move a position. The rules refuse a move the table does not allow, an `open` with no
/// requisition, a `filled` with no holder, and a held position being made vacant or abolished.
#[debug_handler]
async fn status(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<Uuid>,
    Json(payload): Json<MovePayload>,
) -> Result<Response> {
    let row = find(&ctx, &caller, pid).await?;
    let facts = Facts {
        has_requisition: row.requisition_pid.is_some(),
        has_holder: row.holder_worker_pid.is_some(),
    };
    rules::check_move(&row.state, &payload.to, facts).map_err(|e| unprocessable(&e))?;
    let freeze = Freeze {
        reason: payload.reason.as_deref(),
        decided_by_role: payload.decided_by_role.as_deref(),
        review_on: payload.review_on,
    };
    let freezing = payload.to == "frozen";
    if freezing {
        if let Err(problems) = history::check_freeze(&freeze, crate::dates::today()) {
            ensure_valid(&problems)?;
        }
    } else if payload.reason.is_some()
        || payload.decided_by_role.is_some()
        || payload.review_on.is_some()
    {
        return Err(unprocessable(
            "reason, decided_by_role and review_on are only for a move to frozen",
        ));
    }
    let from = row.state.clone();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "UPDATE positions SET state = $2 WHERE id = $1",
        row.id,
        payload.to
    )
    .execute(&mut *tx)
    .await?;
    record_move(
        &mut tx,
        NewMove {
            position: pid,
            organization: &row.organization_ref,
            from: Some(&from),
            to: &payload.to,
            freeze: freezing.then_some(&freeze),
            holder_left: false,
        },
        caller.actor(),
    )
    .await?;
    db::audit(
        &mut tx,
        "position",
        pid,
        "position_status_changed",
        caller.actor(),
        Some(json!({ "from": from, "to": payload.to, "freeze_reason": freezing.then_some(&payload.reason) })),
    )
    .await?;
    tx.commit().await?;
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
async fn holder(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<Uuid>,
    Json(payload): Json<HolderPayload>,
) -> Result<Response> {
    let row = find(&ctx, &caller, pid).await?;
    if row.state == "abolished" || row.state == "planned" {
        return Err(unprocessable(&format!(
            "position is {:?}; only an approved position can have a holder",
            row.state
        )));
    }
    if let Some(worker) = payload.worker_pid {
        let found = sqlx::query_scalar!(
            "SELECT organization_ref FROM workers WHERE pid = $1 AND deleted_at IS NULL",
            worker,
        )
        .fetch_optional(db::pool(&ctx))
        .await?;
        if found.is_none_or(|o| o != row.organization_ref) {
            return Err(unprocessable(
                "worker_pid names no worker of this organization",
            ));
        }
    }
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "UPDATE positions SET holder_worker_pid = $2, recorded_by = $3 WHERE id = $1",
        row.id,
        payload.worker_pid,
        caller.actor(),
    )
    .execute(&mut *tx)
    .await?;
    // The holder history (WPM-R290): a change of holder closes the open tenure and opens the new one; the same holder again is no change.
    if row.holder_worker_pid != payload.worker_pid {
        let today = SqlDate(crate::dates::today());
        sqlx::query!(
            "UPDATE position_holders SET ended_on = $2 WHERE position_pid = $1 AND ended_on IS NULL",
            pid,
            today,
        )
        .execute(&mut *tx)
        .await?;
        if let Some(worker) = payload.worker_pid {
            sqlx::query!(
                "INSERT INTO position_holders (position_pid, organization_ref, worker_pid, started_on, recorded_by)
                 VALUES ($1, $2, $3, $4, $5)",
                pid,
                row.organization_ref,
                worker,
                today,
                caller.actor(),
            )
            .execute(&mut *tx)
            .await?;
        }
    }
    // A holder leaving a post that stays approved, open or frozen begins its vacancy (WPM-R288).
    if row.holder_worker_pid.is_some()
        && payload.worker_pid.is_none()
        && history::is_vacant_state(&row.state)
    {
        record_move(
            &mut tx,
            NewMove {
                position: pid,
                organization: &row.organization_ref,
                from: Some(&row.state),
                to: &row.state,
                freeze: None,
                holder_left: true,
            },
            caller.actor(),
        )
        .await?;
    }
    db::audit(
        &mut tx,
        "position",
        pid,
        "position_holder_set",
        caller.actor(),
        Some(json!({ "linked": payload.worker_pid.is_some() })),
    )
    .await?;
    tx.commit().await?;
    format::json(json!({ "pid": pid }))
}

#[derive(Debug, Deserialize)]
struct GradePayload {
    scale: String,
    band: String,
    /// The step of the band, 1-based (1 is the entry step); optional.
    pay_point: Option<i32>,
}

/// `PUT /api/positions/{pid}/grade` — the band and pay point the post is graded at (WPM-R255): a reference to a published scale,
/// never an amount (WPM-D208). Replaces what is recorded; `DELETE` clears it, and an unrecorded grade is unknown.
#[debug_handler]
async fn set_grade(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<Uuid>,
    Json(payload): Json<GradePayload>,
) -> Result<Response> {
    let row = find(&ctx, &caller, pid).await?;
    if row.state == "abolished" {
        return Err(unprocessable(
            "position is \"abolished\"; an ended position is not changed",
        ));
    }
    let (scale, band, pay_point) =
        rules::resolve_grade(&payload.scale, &payload.band, payload.pay_point)
            .map_err(|e| unprocessable(&e))?;
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "UPDATE positions SET grade_scale = $2, grade_band = $3, grade_pay_point = $4, recorded_by = $5 WHERE id = $1",
        row.id,
        scale,
        band,
        pay_point,
        caller.actor(),
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "position",
        pid,
        "position_grade_set",
        caller.actor(),
        Some(json!({ "grade": format!("{scale}:{band}"), "pay_point": pay_point })),
    )
    .await?;
    tx.commit().await?;
    format::json(
        json!({ "pid": pid, "grade": { "scale": scale, "band": band, "pay_point": pay_point } }),
    )
}

/// `DELETE /api/positions/{pid}/grade` — clear the grade; a post with none is `404`.
#[debug_handler]
async fn clear_grade(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<Uuid>,
) -> Result<Response> {
    let row = find(&ctx, &caller, pid).await?;
    if row.grade_scale.is_none() {
        return Err(Error::NotFound);
    }
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "UPDATE positions SET grade_scale = NULL, grade_band = NULL, grade_pay_point = NULL, recorded_by = $2 WHERE id = $1",
        row.id,
        caller.actor(),
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "position",
        pid,
        "position_grade_cleared",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(json!({ "pid": pid, "grade": null }))
}

#[derive(Debug, Deserialize)]
struct RequisitionPayload {
    /// The requisition that fills the position, or `null` to unlink.
    requisition_pid: Option<Uuid>,
}

/// `PUT /api/positions/{pid}/requisition` — link the requisition that fills the position, or unlink. Same organization.
#[debug_handler]
async fn requisition(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<Uuid>,
    Json(payload): Json<RequisitionPayload>,
) -> Result<Response> {
    let row = find(&ctx, &caller, pid).await?;
    if row.state == "abolished" {
        return Err(unprocessable(
            "position is \"abolished\"; an ended position is not changed",
        ));
    }
    if row.state == "open" && payload.requisition_pid.is_none() {
        return Err(unprocessable(
            "position is \"open\"; move it back before unlinking its requisition",
        ));
    }
    if let Some(r) = payload.requisition_pid {
        let found = sqlx::query_scalar!(
            "SELECT organization_ref FROM requisitions WHERE pid = $1 AND deleted_at IS NULL",
            r,
        )
        .fetch_optional(db::pool(&ctx))
        .await?;
        if found.is_none_or(|o| o != row.organization_ref) {
            return Err(unprocessable(
                "requisition_pid names no requisition of this organization",
            ));
        }
    }
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "UPDATE positions SET requisition_pid = $2, recorded_by = $3 WHERE id = $1",
        row.id,
        payload.requisition_pid,
        caller.actor(),
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "position",
        pid,
        "position_requisition_set",
        caller.actor(),
        Some(json!({ "linked": payload.requisition_pid.is_some() })),
    )
    .await?;
    tx.commit().await?;
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
    axa::extract::Query(query): axa::extract::Query<EstablishmentQuery>,
) -> Result<Response> {
    ensure_valid(&organization_problems(&query.organization))?;
    ensure_org_in_scope(&ctx, &caller, &query.organization).await?;
    // Each position with its holder's FTE; a holder whose record is gone (erased or deleted) has none.
    let rows = sqlx::query!(
        r#"SELECT p.cost_centre, p.state, p.budget_fte_percent, p.holder_worker_pid, w.fte_percent AS "fte_percent?",
                  p.grade_scale, p.grade_band, p.grade_pay_point
             FROM positions p
             LEFT JOIN workers w ON w.pid = p.holder_worker_pid AND w.deleted_at IS NULL
            WHERE p.organization_ref = $1"#,
        query.organization,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    // Each position with its holder's FTE. A holder whose record is gone (erased or deleted) has none.
    let seats: Vec<Seat> = rows
        .iter()
        .map(|r| Seat {
            cost_centre: r.cost_centre.clone(),
            state: r.state.clone(),
            budget_fte_percent: r.budget_fte_percent,
            // A holder whose FTE we cannot say is held by someone, not vacant.
            holding: match (r.holder_worker_pid, r.fte_percent) {
                (None, _) => Holding::Vacant,
                (Some(_), None) => Holding::HeldUnknown,
                (Some(_), Some(f)) => Holding::Held(f),
            },
        })
        .collect();
    // The same seats under their band, and under their band and pay point; an ungraded post is unknown in both.
    let (band_seats, point_seats): (Vec<Seat>, Vec<Seat>) = rows
        .iter()
        .zip(&seats)
        .map(|(r, seat)| {
            let band = rules::band_label(r.grade_scale.as_deref(), r.grade_band.as_deref());
            let point = match (&band, r.grade_pay_point) {
                (Some(b), Some(p)) => format!("{b}:{p}"),
                (Some(b), None) => format!("{b}:{}", history::UNKNOWN_BAND),
                (None, _) => history::UNKNOWN_BAND.to_string(),
            };
            let under = |key: String| Seat {
                cost_centre: key,
                ..seat.clone()
            };
            (
                under(band.unwrap_or_else(|| history::UNKNOWN_BAND.to_string())),
                under(point),
            )
        })
        .unzip();
    let floor = if sees_holders(&ctx, &caller, &query.organization).await? {
        None
    } else {
        Some(DEFAULT_FLOOR)
    };
    format::json(json!({
        "organization": query.organization,
        "small_group_floor": floor,
        "cost_centres": rules::establishment(&seats, floor),
        "bands": grouped(rules::establishment(&band_seats, floor))?,
        "pay_points": grouped(rules::establishment(&point_seats, floor))?,
        "derivation": "The establishment is the positions that are approved, open, filled or frozen. Held is the FTE of the workers linked as holders; \
                       vacant is budget not held; over is FTE held beyond budget. Over-establishment on one position is never netted against a \
                       vacancy on another. A holder whose FTE cannot be read makes held, vacant and over unknown for that cost centre. \
                       The same figures are given by band (scale:band) and by band and pay point (scale:band:point); a post with no \
                       grade is in unknown, and a graded post with no pay point is in scale:band:unknown, never in a band or point.",
    }))
}

/// `GET /api/positions/{pid}/holders` — the holder history of one post (WPM-R290): each tenure with its worker, start and end, and
/// the post's continuity over the last 36 months (tenures, days held, days empty), counted from the day its history begins when that
/// is later, and saying so. HR and the other privileged callers only (`rules::access::READS`); a view of one post names people. No
/// route counts the posts a person has left.
#[debug_handler]
async fn holders(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<Uuid>,
) -> Result<Response> {
    let row = find(&ctx, &caller, pid).await?;
    let tenures = sqlx::query_as!(
        position_holders::Model,
        r#"SELECT created_at AS "created_at: _", id, position_pid, organization_ref, worker_pid, started_on AS "started_on: _",
                  start_known, ended_on AS "ended_on: _", recorded_by
             FROM position_holders WHERE position_pid = $1 ORDER BY started_on, id"#,
        pid,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    // The post's history begins at its first state row: on the day it was created when that was recorded, otherwise on the day the
    // history began (the backfill's own date).
    let known_from = sqlx::query!(
        r#"SELECT moved_on, start_known, (created_at AT TIME ZONE 'UTC')::date AS "created_on!" FROM position_state_moves
            WHERE position_pid = $1 ORDER BY id LIMIT 1"#,
        pid,
    )
    .fetch_optional(db::pool(&ctx))
    .await?
    .map_or_else(crate::dates::today, |m| {
        if m.start_known {
            m.moved_on.get()
        } else {
            m.created_on.get()
        }
    });
    let today = crate::dates::today();
    let continuity = history::continuity(
        &tenures
            .iter()
            .map(|t| history::Tenure {
                started_on: t.started_on.get(),
                ended_on: t.ended_on.map(SqlDate::get),
            })
            .collect::<Vec<_>>(),
        known_from,
        today,
        history::CONTINUITY_MONTHS,
    );
    format::json(json!({
        "pid": pid,
        "organization": row.organization_ref,
        "holders": tenures.iter().map(|t| json!({
            "worker_pid": t.worker_pid,
            "erased": t.worker_pid.is_none(),
            "started_on": t.started_on,
            "start_known": t.start_known,
            "ended_on": t.ended_on,
        })).collect::<Vec<_>>(),
        "continuity": continuity,
        "derivation": "Each tenure runs from the day a holder was linked up to the day they were unlinked or replaced. A tenure already open when \
                       the history began has an unknown start. Continuity counts the last 36 months, or from the day this post's history \
                       begins when that is later (complete is then false); a day is held when any tenure covers it. A tenure whose worker \
                       was erased keeps its dates and names no one.",
    }))
}

#[derive(Debug, Deserialize)]
struct VacancyQuery {
    organization: String,
}

/// `GET /api/positions/vacancy-age?organization=` — open vacancies by cost centre and age (WPM-R288, WPM-R289): how long each approved,
/// open or frozen post nobody holds has been vacant, from the history of its moves, and not only for the posts that were filled. A post
/// whose state predates the history is `start_unknown`, never zero. The frozen posts, those with no review date and those whose review
/// date has passed are counted, and the abolished posts stand beside the ages. No holder is named; a caller who may not see who holds
/// what gets a cost centre under the small-group floor withheld, and the figure says so.
#[debug_handler]
async fn vacancy_age(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axa::extract::Query(query): axa::extract::Query<VacancyQuery>,
) -> Result<Response> {
    ensure_valid(&organization_problems(&query.organization))?;
    ensure_org_in_scope(&ctx, &caller, &query.organization).await?;
    let positions = sqlx::query!(
        r#"SELECT pid, cost_centre, state, (holder_worker_pid IS NOT NULL) AS "held!", critical, grade_scale, grade_band
             FROM positions WHERE organization_ref = $1 ORDER BY cost_centre, id"#,
        query.organization,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let moves = sqlx::query!(
        r#"SELECT position_pid, to_state, moved_on, start_known, review_on, holder_left FROM position_state_moves
            WHERE organization_ref = $1 ORDER BY position_pid, id"#,
        query.organization,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let mut by_post: std::collections::HashMap<Uuid, (Vec<history::Move>, Option<Date>)> =
        std::collections::HashMap::new();
    for m in moves {
        let entry = by_post.entry(m.position_pid).or_default();
        // The review date in force is the one of the latest move; a move out of frozen leaves none.
        entry.1 = m.review_on.map(SqlDate::get);
        entry.0.push(history::Move {
            to: m.to_state,
            on: m.moved_on.get(),
            start_known: m.start_known,
            holder_left: m.holder_left,
        });
    }
    let posts: Vec<history::Post> = positions
        .into_iter()
        .map(|p| {
            let (moves, review_on) = by_post.remove(&p.pid).unwrap_or_else(|| (Vec::new(), None));
            history::Post {
                band: rules::band_label(p.grade_scale.as_deref(), p.grade_band.as_deref()),
                critical: p.critical,
                cost_centre: p.cost_centre,
                state: p.state,
                held: p.held,
                moves,
                review_on,
            }
        })
        .collect();
    let floor = if sees_holders(&ctx, &caller, &query.organization).await? {
        None
    } else {
        Some(DEFAULT_FLOOR)
    };
    let today = crate::dates::today();
    format::json(json!({
        "organization": query.organization,
        "small_group_floor": floor,
        "age_bands_days": history::AGE_BANDS.iter().map(|b| b.0).collect::<Vec<_>>(),
        "cost_centres": history::report(&posts, today, floor),
        "bands": history::report_by_band(&posts, today, floor),
        "critical": history::report_critical(&posts, today, floor),
        "derivation": "A vacancy is an approved, open or frozen position nobody holds. Its age is counted from the move that began its current run of \
                       those states (a position that was filled or planned before). A position whose state was already set when the history \
                       began has an unknown start, counted in start_unknown and in no age band. Abolished positions are shown beside the \
                       ages, because abolishing a post ends its vacancy. The same figures are given by band (a post with no recorded grade is in \
                       unknown, never in a band) and for the posts HR has marked critical, together (null when none is marked).",
    }))
}

/// The establishment's rows under a key that is not a cost centre, with the key named `group`.
fn grouped(centres: Vec<rules::CostCentre>) -> Result<Vec<serde_json::Value>> {
    centres
        .into_iter()
        .map(|c| {
            let mut v = serde_json::to_value(c)
                .map_err(|_| super::internal("an establishment row does not serialise"))?;
            if let Some(o) = v.as_object_mut()
                && let Some(key) = o.remove("cost_centre")
            {
                o.insert("group".into(), key);
            }
            Ok(v)
        })
        .collect()
}

/// `GET /api/positions/critical-continuity?organization=` — the continuity of the posts HR has marked critical (WPM-R290), by cost
/// centre, by band and all together: posts, tenures, days held and days empty in the last 36 months, posts held today, and posts whose
/// history is shorter than the window. A senior group is small, so a group with fewer posts than the floor is withheld for every
/// caller, with its complement, and says so. No holder is named and no person's count of posts left is in it.
#[debug_handler]
async fn critical_continuity(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axa::extract::Query(query): axa::extract::Query<VacancyQuery>,
) -> Result<Response> {
    ensure_valid(&organization_problems(&query.organization))?;
    ensure_org_in_scope(&ctx, &caller, &query.organization).await?;
    let today = crate::dates::today();
    let posts = sqlx::query!(
        r#"SELECT p.pid, p.cost_centre, p.grade_scale, p.grade_band, (p.holder_worker_pid IS NOT NULL) AS "held!",
                  m.start_known AS "start_known?", m.moved_on AS "moved_on?", (m.created_at AT TIME ZONE 'UTC')::date AS "created_on?"
             FROM positions p
             LEFT JOIN LATERAL (SELECT start_known, moved_on, created_at FROM position_state_moves s
                                 WHERE s.position_pid = p.pid ORDER BY s.id LIMIT 1) m ON TRUE
            WHERE p.organization_ref = $1 AND p.critical AND p.state <> 'abolished'
            ORDER BY p.cost_centre, p.id"#,
        query.organization,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let pids: Vec<Uuid> = posts.iter().map(|p| p.pid).collect();
    let mut tenures: std::collections::HashMap<Uuid, Vec<history::Tenure>> =
        std::collections::HashMap::new();
    for t in sqlx::query!(
        "SELECT position_pid, started_on, ended_on FROM position_holders WHERE position_pid = ANY($1) ORDER BY started_on, id",
        &pids,
    )
    .fetch_all(db::pool(&ctx))
    .await?
    {
        tenures.entry(t.position_pid).or_default().push(history::Tenure {
            started_on: t.started_on.get(),
            ended_on: t.ended_on.map(SqlDate::get),
        });
    }
    let mut by_centre = Vec::new();
    let mut by_band = Vec::new();
    for p in &posts {
        // The post's history begins at its first state row (as for one post's holders); a post with none begins today.
        let known_from = match (p.start_known, p.moved_on, p.created_on) {
            (Some(true), Some(on), _) => on.get(),
            (_, _, Some(created)) => created.get(),
            _ => today,
        };
        let post = history::CriticalPost {
            continuity: history::continuity(
                tenures.get(&p.pid).map_or(&[][..], Vec::as_slice),
                known_from,
                today,
                history::CONTINUITY_MONTHS,
            ),
            held_now: p.held,
        };
        by_centre.push((p.cost_centre.clone(), post));
        by_band.push((
            rules::band_label(p.grade_scale.as_deref(), p.grade_band.as_deref())
                .unwrap_or_else(|| history::UNKNOWN_BAND.to_string()),
            post,
        ));
    }
    format::json(json!({
        "organization": query.organization,
        "small_group_floor": DEFAULT_FLOOR,
        "window_months": history::CONTINUITY_MONTHS,
        "cost_centres": history::critical_report(&by_centre, DEFAULT_FLOOR),
        "bands": history::critical_report(&by_band, DEFAULT_FLOOR),
        "derivation": "The posts HR has marked critical and not abolished. Each post's tenures, days held and days empty are counted over \
                       the last 36 months, or from the day its history begins when that is later (counted_partly), then added up by \
                       group. A group with fewer posts than the floor is withheld for every caller, and when exactly one is withheld the \
                       smallest other is too, so it cannot be had from the total. The two tables overlap, which this does not defend.",
    }))
}

/// The routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/positions", get(list))
        .add("/positions", post(create))
        .add("/positions/establishment", get(establishment))
        .add("/positions/vacancy-age", get(vacancy_age))
        .add("/positions/critical-continuity", get(critical_continuity))
        .add("/positions/{pid}", put(update))
        .add("/positions/{pid}/status", post(status))
        .add("/positions/{pid}/holder", put(holder))
        .add("/positions/{pid}/holders", get(holders))
        .add("/positions/{pid}/grade", put(set_grade))
        .add("/positions/{pid}/grade", delete(clear_grade))
        .add("/positions/{pid}/requisition", put(requisition))
}
