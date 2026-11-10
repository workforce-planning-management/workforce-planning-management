//! Escalations (part of [`super`]; WPM-R141, WPM-D94): a person takes a breached tolerance to someone
//! who can decide, it goes up at most three levels, and an administrator of the organization decides.
//!
//! The measure (status, deviation, limit) is stored at the moment of raising and of deciding, so the
//! record says what the decider was looking at. Nothing is raised automatically.

// A section of one controller: it shares the parent module's imports.
#[allow(clippy::wildcard_imports)]
use super::*;
use crate::db;
use crate::models::_entities::plan_escalations;
use crate::rules::escalation as esc;
use crate::rules::tolerance as tol;
use axa::extract::{Json, Path};

/// An escalation as the API shows it. `plan_name` is given by the inbox, which lists several plans.
pub(super) fn escalation_json(
    e: &plan_escalations::Model,
    today: chrono::NaiveDate,
    plan_name: Option<&str>,
) -> serde_json::Value {
    serde_json::json!({
        "pid": e.pid,
        "plan_pid": e.plan_pid,
        "plan_name": plan_name,
        "dimension": e.dimension,
        "status": e.status,
        "level": esc::Level::from_number(e.level),
        "level_number": e.level,
        "raised_by": e.raised_by,
        "raised_on": e.raised_on,
        "level_raised_on": e.level_raised_on,
        "days_open": (e.status == "open").then(|| esc::days_open(e.raised_on, today)),
        "note": e.note,
        "at_raise": { "rag": e.rag_at_raise, "deviation": e.deviation_at_raise, "limit": e.limit_at_raise },
        "decision": e.decision,
        "decided_by": e.decided_by,
        "decided_on": e.decided_on,
        "decision_note": e.decision_note,
        "at_decision": e.decided_on.map(|_| serde_json::json!({
            "rag": e.rag_at_decision, "deviation": e.deviation_at_decision,
        })),
    })
}

/// Worker pids of the organization's HR and organization administrators, to be told.
async fn administrators(ctx: &AppContext, org: &str) -> Result<Vec<Uuid>> {
    let mut out: Vec<Uuid> = Vec::new();
    for pid in sqlx::query_scalar!(
        r#"SELECT worker_pid AS "worker_pid!" FROM organization_memberships
            WHERE organization_ref = $1 AND role IN ('hr_admin', 'org_admin') AND deleted_at IS NULL
              AND worker_pid IS NOT NULL
            ORDER BY id"#,
        org,
    )
    .fetch_all(db::pool(ctx))
    .await?
    {
        if !out.contains(&pid) {
            out.push(pid);
        }
    }
    Ok(out)
}

/// Tell the administrators. The text names the plan, the dimension and the level; the note is never copied.
async fn tell(
    administrators: &[Uuid],
    tx: &mut sqlx::PgConnection,
    plan: &workforce_plans::Model,
    e: &plan_escalations::Model,
    what: &str,
) -> Result<()> {
    let level = esc::Level::from_number(e.level).map_or("plan_owner", |l| match l {
        esc::Level::PlanOwner => "plan_owner",
        esc::Level::Board => "board",
        esc::Level::Executive => "executive",
    });
    let text = format!(
        "{what}: plan \"{}\", {} tolerance, now at level {level}.",
        plan.name, e.dimension
    );
    for &to in administrators {
        db::notify(
            tx,
            to,
            "escalation",
            &text,
            serde_json::json!({ "escalation_pid": e.pid, "plan_pid": plan.pid, "dimension": e.dimension, "level": e.level }),
        )
        .await?;
    }
    Ok(())
}

/// `POST /api/workforce-plans/{pid}/escalations` body.
#[derive(Debug, Deserialize)]
pub(super) struct RaisePayload {
    dimension: String,
    #[serde(default)]
    note: Option<String>,
}

/// `POST /api/workforce-plans/{pid}/escalations` — take an amber or red dimension to level 1.
#[debug_handler]
pub(super) async fn raise_escalation(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<RaisePayload>,
) -> Result<Response> {
    let plan = scoped_plan(&ctx, &caller, &pid).await?;
    let dimension = tol::Dimension::parse(&payload.dimension)
        .ok_or_else(|| unprocessable("dimension must be one of time, cost, resources, risk"))?;
    let note = esc::clean_note(payload.note.as_deref(), false).map_err(|e| unprocessable(&e))?;
    let states = dimension_states(&ctx, &caller, &pid, &plan).await?;
    let state = states
        .iter()
        .find(|s| s.dimension == dimension)
        .ok_or(Error::NotFound)?;
    esc::may_raise(state.rag).map_err(|e| unprocessable(&e))?;
    let tolerance = state.tolerance.as_ref().ok_or(Error::NotFound)?;
    let already = sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM plan_escalations WHERE plan_pid = $1 AND dimension = $2 AND status = 'open')
             AS "exists!""#,
        plan.pid,
        dimension.as_str(),
    )
    .fetch_one(db::pool(&ctx))
    .await?;
    if already {
        return Err(unprocessable(
            "an escalation is already open for this dimension; raise its level or decide it",
        ));
    }
    let today = chrono::Utc::now().date_naive();
    let notify = administrators(&ctx, &plan.organization_ref).await?;
    let mut tx = db::pool(&ctx).begin().await?;
    let row = sqlx::query_as!(
        plan_escalations::Model,
        r#"INSERT INTO plan_escalations (pid, plan_pid, dimension, level, status, raised_by, raised_on, note, rag_at_raise,
                                         deviation_at_raise, limit_at_raise)
           VALUES ($1, $2, $3, $4, 'open', $5, $6, $7, $8, $9, $10)
           RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, plan_pid, dimension, level, status,
                  raised_by, raised_on, level_raised_on, note, rag_at_raise, deviation_at_raise, limit_at_raise, decision,
                  decided_by, decided_on, decision_note, rag_at_decision, deviation_at_decision"#,
        Uuid::new_v4(),
        plan.pid,
        dimension.as_str(),
        esc::Level::PlanOwner.number(),
        caller.actor(),
        today,
        note,
        if state.rag == tol::Rag::Red { "red" } else { "amber" },
        state.deviation,
        tolerance.limit_value,
    )
    .fetch_one(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "plan_escalation",
        row.pid,
        "escalation_raised",
        caller.actor(),
        Some(serde_json::json!({
            "plan_pid": plan.pid, "dimension": dimension, "level": row.level,
            "rag": row.rag_at_raise, "deviation": row.deviation_at_raise, "limit": row.limit_at_raise,
        })),
    )
    .await?;
    tell(&notify, &mut tx, &plan, &row, "An escalation was raised").await?;
    tx.commit().await?;
    format::json(escalation_json(&row, today, None))
}

/// The open escalation `pid`, and the plan it is about (which the caller must be able to read).
async fn open_escalation(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: &str,
) -> Result<(plan_escalations::Model, workforce_plans::Model)> {
    let e = sqlx::query_as!(
        plan_escalations::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, plan_pid, dimension, level, status,
                  raised_by, raised_on, level_raised_on, note, rag_at_raise, deviation_at_raise, limit_at_raise, decision,
                  decided_by, decided_on, decision_note, rag_at_decision, deviation_at_decision
             FROM plan_escalations WHERE pid = $1"#,
        records::parse_pid(pid)?,
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)?;
    let plan = scoped_plan(ctx, caller, &e.plan_pid.to_string()).await?;
    if e.status != "open" {
        return Err(unprocessable("this escalation has been decided"));
    }
    Ok((e, plan))
}

/// `POST /api/escalations/{pid}/raise-level` — one level up.
#[debug_handler]
pub(super) async fn raise_level(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let (e, plan) = open_escalation(&ctx, &caller, &pid).await?;
    let next = esc::Level::from_number(e.level)
        .and_then(esc::Level::next)
        .ok_or_else(|| unprocessable("this escalation is already at the top level"))?;
    let today = chrono::Utc::now().date_naive();
    let notify = administrators(&ctx, &plan.organization_ref).await?;
    let mut tx = db::pool(&ctx).begin().await?;
    let row = sqlx::query_as!(
        plan_escalations::Model,
        r#"UPDATE plan_escalations SET level = $2, level_raised_on = $3 WHERE id = $1
           RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, plan_pid, dimension, level, status,
                  raised_by, raised_on, level_raised_on, note, rag_at_raise, deviation_at_raise, limit_at_raise, decision,
                  decided_by, decided_on, decision_note, rag_at_decision, deviation_at_decision"#,
        e.id,
        next.number(),
        today,
    )
    .fetch_one(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "plan_escalation",
        row.pid,
        "escalation_level_raised",
        caller.actor(),
        Some(serde_json::json!({ "plan_pid": plan.pid, "dimension": row.dimension, "level": row.level })),
    )
    .await?;
    tell(
        &notify,
        &mut tx,
        &plan,
        &row,
        "An escalation went up a level",
    )
    .await?;
    tx.commit().await?;
    format::json(escalation_json(&row, today, None))
}

/// `POST /api/escalations/{pid}/decide` body.
#[derive(Debug, Deserialize)]
pub(super) struct DecidePayload {
    decision: String,
    note: String,
}

/// `POST /api/escalations/{pid}/decide` — an administrator of the organization closes it.
#[debug_handler]
pub(super) async fn decide_escalation(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<DecidePayload>,
) -> Result<Response> {
    let (e, plan) = open_escalation(&ctx, &caller, &pid).await?;
    super::super::oversight_packs::ensure_org_administrator(
        &ctx,
        &caller,
        &plan.organization_ref,
        "deciding an escalation",
    )
    .await?;
    let decision = esc::Decision::parse(&payload.decision).ok_or_else(|| {
        unprocessable("decision must be one of accept_deviation, change_plan, stop")
    })?;
    let note = esc::clean_note(Some(&payload.note), true)
        .map_err(|m| unprocessable(&m))?
        .unwrap_or_default();
    // what the plan looks like as it is decided, kept with the decision
    let states = dimension_states(&ctx, &caller, &plan.pid.to_string(), &plan).await?;
    let now = states.iter().find(|s| s.dimension.as_str() == e.dimension);
    let today = chrono::Utc::now().date_naive();
    let rag_at_decision = now
        .map(|s| serde_json::json!(s.rag))
        .and_then(|v| v.as_str().map(ToString::to_string));
    let mut tx = db::pool(&ctx).begin().await?;
    let row = sqlx::query_as!(
        plan_escalations::Model,
        r#"UPDATE plan_escalations SET status = 'decided', decision = $2, decided_by = $3, decided_on = $4,
                  decision_note = $5, rag_at_decision = $6, deviation_at_decision = $7
            WHERE id = $1
            RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, plan_pid, dimension, level, status,
                  raised_by, raised_on, level_raised_on, note, rag_at_raise, deviation_at_raise, limit_at_raise, decision,
                  decided_by, decided_on, decision_note, rag_at_decision, deviation_at_decision"#,
        e.id,
        decision.as_str(),
        caller.actor(),
        today,
        note,
        rag_at_decision,
        now.and_then(|s| s.deviation),
    )
    .fetch_one(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "plan_escalation",
        row.pid,
        "escalation_decided",
        caller.actor(),
        Some(serde_json::json!({
            "plan_pid": plan.pid, "dimension": row.dimension, "decision": decision,
            "rag": row.rag_at_decision, "deviation": row.deviation_at_decision,
        })),
    )
    .await?;
    tx.commit().await?;
    format::json(escalation_json(&row, today, None))
}

/// `GET /api/workforce-plans/{pid}/escalations` — open first, then decided, newest first.
#[debug_handler]
pub(super) async fn list_plan_escalations(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let plan = scoped_plan(&ctx, &caller, &pid).await?;
    let today = chrono::Utc::now().date_naive();
    // Open first, then decided; newest first within each.
    let rows = sqlx::query_as!(
        plan_escalations::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, plan_pid, dimension, level, status,
                  raised_by, raised_on, level_raised_on, note, rag_at_raise, deviation_at_raise, limit_at_raise, decision,
                  decided_by, decided_on, decision_note, rag_at_decision, deviation_at_decision
             FROM plan_escalations WHERE plan_pid = $1 ORDER BY status <> 'open', id DESC"#,
        plan.pid,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    format::json(
        rows.iter()
            .map(|e| escalation_json(e, today, None))
            .collect::<Vec<_>>(),
    )
}

/// `GET /api/escalations?organization=` query.
#[derive(Debug, Deserialize)]
pub(super) struct InboxQuery {
    organization: Option<String>,
}

/// `GET /api/escalations?organization=` — the open escalations of an organization, oldest first.
#[debug_handler]
pub(super) async fn list_open_escalations(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axa::extract::Query(query): axa::extract::Query<InboxQuery>,
) -> Result<Response> {
    let org = query
        .organization
        .ok_or_else(|| unprocessable("organization is required"))?;
    ensure_org_in_scope(&ctx, &caller, &org).await?;
    let plans: BTreeMap<Uuid, String> = sqlx::query!(
        "SELECT pid, name FROM workforce_plans WHERE organization_ref = $1 AND deleted_at IS NULL",
        org,
    )
    .fetch_all(db::pool(&ctx))
    .await?
    .into_iter()
    .map(|p| (p.pid, p.name))
    .collect();
    let today = chrono::Utc::now().date_naive();
    let plan_pids: Vec<Uuid> = plans.keys().copied().collect();
    let rows = sqlx::query_as!(
        plan_escalations::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, plan_pid, dimension, level, status,
                  raised_by, raised_on, level_raised_on, note, rag_at_raise, deviation_at_raise, limit_at_raise, decision,
                  decided_by, decided_on, decision_note, rag_at_decision, deviation_at_decision
             FROM plan_escalations WHERE status = 'open' AND plan_pid = ANY($1) ORDER BY raised_on, id"#,
        &plan_pids,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    format::json(
        rows.iter()
            .map(|e| escalation_json(e, today, plans.get(&e.plan_pid).map(String::as_str)))
            .collect::<Vec<_>>(),
    )
}
