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
use crate::models::_entities::positions;
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
                  requisition_pid, demand_line_pid, overtime_status, number, recorded_by
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
                                programme_ref, state, overtime_status, demand_line_pid, number, recorded_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'planned', $9, $10, $11, $12)",
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
    )
    .execute(&mut *tx)
    .await?;
    record_move(
        &mut tx,
        pid,
        &organization,
        None,
        "planned",
        None,
        caller.actor(),
    )
    .await?;
    db::audit(
        &mut tx,
        "position",
        pid,
        "position_created",
        caller.actor(),
        Some(json!({ "from_demand_line": payload.demand_line_pid.is_some() })),
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
                  requisition_pid, demand_line_pid, overtime_status, number, recorded_by
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
                recorded_by = $10
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
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "position",
        pid,
        "position_updated",
        caller.actor(),
        status_changed.then(|| json!({ "overtime_status": payload.overtime_status })),
    )
    .await?;
    tx.commit().await?;
    format::json(json!({ "pid": pid }))
}

/// Add one row to the state history of a position (WPM-R288): the date is today's, and the row is never changed after.
async fn record_move(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    position: Uuid,
    organization: &str,
    from: Option<&str>,
    to: &str,
    freeze: Option<&Freeze<'_>>,
    actor: Option<&str>,
) -> Result<()> {
    sqlx::query!(
        "INSERT INTO position_state_moves (position_pid, organization_ref, from_state, to_state, moved_on, freeze_reason,
                                           decided_by_role, review_on, recorded_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        position,
        organization,
        from,
        to,
        SqlDate(crate::dates::today()),
        freeze.and_then(|f| f.reason),
        freeze.and_then(|f| f.decided_by_role).map(str::trim),
        freeze.and_then(|f| f.review_on).map(SqlDate),
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
        pid,
        &row.organization_ref,
        Some(&from),
        &payload.to,
        freezing.then_some(&freeze),
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
    let seats: Vec<Seat> = sqlx::query!(
        r#"SELECT p.cost_centre, p.state, p.budget_fte_percent, p.holder_worker_pid, w.fte_percent AS "fte_percent?"
             FROM positions p
             LEFT JOIN workers w ON w.pid = p.holder_worker_pid AND w.deleted_at IS NULL
            WHERE p.organization_ref = $1"#,
        query.organization,
    )
    .fetch_all(db::pool(&ctx))
    .await?
    .into_iter()
    .map(|r| Seat {
        cost_centre: r.cost_centre,
        state: r.state,
        budget_fte_percent: r.budget_fte_percent,
        // A holder whose FTE we cannot say is held by someone, not vacant.
        holding: match (r.holder_worker_pid, r.fte_percent) {
            (None, _) => Holding::Vacant,
            (Some(_), None) => Holding::HeldUnknown,
            (Some(_), Some(f)) => Holding::Held(f),
        },
    })
    .collect();
    let floor = if sees_holders(&ctx, &caller, &query.organization).await? {
        None
    } else {
        Some(DEFAULT_FLOOR)
    };
    format::json(json!({
        "organization": query.organization,
        "small_group_floor": floor,
        "cost_centres": rules::establishment(&seats, floor),
        "derivation": "The establishment is the positions that are approved, open, filled or frozen. Held is the FTE of the workers linked as holders; \
                       vacant is budget not held; over is FTE held beyond budget. Over-establishment on one position is never netted against a \
                       vacancy on another. A holder whose FTE cannot be read makes held, vacant and over unknown for that cost centre.",
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
        r#"SELECT pid, cost_centre, state, (holder_worker_pid IS NOT NULL) AS "held!" FROM positions
            WHERE organization_ref = $1 ORDER BY cost_centre, id"#,
        query.organization,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let moves = sqlx::query!(
        r#"SELECT position_pid, to_state, moved_on, start_known, review_on FROM position_state_moves
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
        });
    }
    let posts: Vec<history::Post> = positions
        .into_iter()
        .map(|p| {
            let (moves, review_on) = by_post.remove(&p.pid).unwrap_or_else(|| (Vec::new(), None));
            history::Post {
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
    format::json(json!({
        "organization": query.organization,
        "small_group_floor": floor,
        "age_bands_days": history::AGE_BANDS.iter().map(|b| b.0).collect::<Vec<_>>(),
        "cost_centres": history::report(&posts, crate::dates::today(), floor),
        "derivation": "A vacancy is an approved, open or frozen position nobody holds. Its age is counted from the move that began its current run of \
                       those states (a position that was filled or planned before). A position whose state was already set when the history \
                       began has an unknown start, counted in start_unknown and in no age band. Abolished positions are shown beside the \
                       ages, because abolishing a post ends its vacancy. By cost centre only: a position carries no band or critical-role marker yet.",
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
        .add("/positions/{pid}", put(update))
        .add("/positions/{pid}/status", post(status))
        .add("/positions/{pid}/holder", put(holder))
        .add("/positions/{pid}/requisition", put(requisition))
}
