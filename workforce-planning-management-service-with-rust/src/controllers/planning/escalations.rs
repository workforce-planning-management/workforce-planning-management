//! Escalations (part of [`super`]; WPM-R141, WPM-D94): a person takes a breached tolerance to someone
//! who can decide, it goes up at most three levels, and an administrator of the organization decides.
//!
//! The measure (status, deviation, limit) is stored at the moment of raising and of deciding, so the
//! record says what the decider was looking at. Nothing is raised automatically.

// A section of one controller: it shares the parent module's imports.
#[allow(clippy::wildcard_imports)]
use super::*;
use crate::models::_entities::{organization_memberships, plan_escalations};
use crate::models::notifications::Model as Notification;
use crate::rules::escalation as esc;
use crate::rules::tolerance as tol;

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
    for m in organization_memberships::Entity::find()
        .filter(organization_memberships::Column::OrganizationRef.eq(org))
        .filter(organization_memberships::Column::Role.is_in(["hr_admin", "org_admin"]))
        .filter(organization_memberships::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?
    {
        if let Some(pid) = m.worker_pid
            && !out.contains(&pid)
        {
            out.push(pid);
        }
    }
    Ok(out)
}

/// Tell the administrators. The text names the plan, the dimension and the level; the note is never copied.
async fn tell<C: ConnectionTrait>(
    administrators: &[Uuid],
    db: &C,
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
        Notification::push(
            db,
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
    let already = plan_escalations::Entity::find()
        .filter(plan_escalations::Column::PlanPid.eq(plan.pid))
        .filter(plan_escalations::Column::Dimension.eq(dimension.as_str()))
        .filter(plan_escalations::Column::Status.eq("open"))
        .one(&ctx.db)
        .await?;
    if already.is_some() {
        return Err(unprocessable(
            "an escalation is already open for this dimension; raise its level or decide it",
        ));
    }
    let today = chrono::Utc::now().date_naive();
    let notify = administrators(&ctx, &plan.organization_ref).await?;
    let txn = ctx.db.begin().await?;
    let row = plan_escalations::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        plan_pid: ActiveValue::set(plan.pid),
        dimension: ActiveValue::set(dimension.as_str().to_string()),
        level: ActiveValue::set(esc::Level::PlanOwner.number()),
        status: ActiveValue::set("open".to_string()),
        raised_by: ActiveValue::set(caller.actor().map(ToString::to_string)),
        raised_on: ActiveValue::set(today),
        note: ActiveValue::set(note),
        rag_at_raise: ActiveValue::set(
            if state.rag == tol::Rag::Red {
                "red"
            } else {
                "amber"
            }
            .to_string(),
        ),
        deviation_at_raise: ActiveValue::set(state.deviation),
        limit_at_raise: ActiveValue::set(tolerance.limit_value),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    Audit::record(
        &txn,
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
    tell(&notify, &txn, &plan, &row, "An escalation was raised").await?;
    txn.commit().await?;
    format::json(escalation_json(&row, today, None))
}

/// The open escalation `pid`, and the plan it is about (which the caller must be able to read).
async fn open_escalation(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: &str,
) -> Result<(plan_escalations::Model, workforce_plans::Model)> {
    let e = plan_escalations::Entity::find()
        .filter(plan_escalations::Column::Pid.eq(records::parse_pid(pid)?))
        .one(&ctx.db)
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
    let mut active: plan_escalations::ActiveModel = e.into();
    active.level = ActiveValue::set(next.number());
    active.level_raised_on = ActiveValue::set(Some(today));
    let notify = administrators(&ctx, &plan.organization_ref).await?;
    let txn = ctx.db.begin().await?;
    let row = active.update(&txn).await?;
    Audit::record(
        &txn,
        "plan_escalation",
        row.pid,
        "escalation_level_raised",
        caller.actor(),
        Some(serde_json::json!({ "plan_pid": plan.pid, "dimension": row.dimension, "level": row.level })),
    )
    .await?;
    tell(&notify, &txn, &plan, &row, "An escalation went up a level").await?;
    txn.commit().await?;
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
    let mut active: plan_escalations::ActiveModel = e.into();
    active.status = ActiveValue::set("decided".to_string());
    active.decision = ActiveValue::set(Some(decision.as_str().to_string()));
    active.decided_by = ActiveValue::set(caller.actor().map(ToString::to_string));
    active.decided_on = ActiveValue::set(Some(today));
    active.decision_note = ActiveValue::set(Some(note));
    active.rag_at_decision = ActiveValue::set(
        now.map(|s| serde_json::json!(s.rag))
            .and_then(|v| v.as_str().map(ToString::to_string)),
    );
    active.deviation_at_decision = ActiveValue::set(now.and_then(|s| s.deviation));
    let txn = ctx.db.begin().await?;
    let row = active.update(&txn).await?;
    Audit::record(
        &txn,
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
    txn.commit().await?;
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
    let mut rows = plan_escalations::Entity::find()
        .filter(plan_escalations::Column::PlanPid.eq(plan.pid))
        .order_by_desc(plan_escalations::Column::Id)
        .all(&ctx.db)
        .await?;
    rows.sort_by_key(|e| e.status != "open"); // stable: newest first within each group
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
    axum::extract::Query(query): axum::extract::Query<InboxQuery>,
) -> Result<Response> {
    let org = query
        .organization
        .ok_or_else(|| unprocessable("organization is required"))?;
    if let Some(refs) = memberships::scope_organization_refs(&ctx.db, caller.claims()).await?
        && !refs.contains(&org)
    {
        return Err(Error::NotFound);
    }
    let plans: BTreeMap<Uuid, String> = workforce_plans::Entity::find()
        .filter(workforce_plans::Column::OrganizationRef.eq(&org))
        .filter(workforce_plans::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|p| (p.pid, p.name))
        .collect();
    let today = chrono::Utc::now().date_naive();
    let rows = plan_escalations::Entity::find()
        .filter(plan_escalations::Column::Status.eq("open"))
        .filter(plan_escalations::Column::PlanPid.is_in(plans.keys().copied()))
        .order_by_asc(plan_escalations::Column::RaisedOn)
        .order_by_asc(plan_escalations::Column::Id)
        .all(&ctx.db)
        .await?;
    format::json(
        rows.iter()
            .map(|e| escalation_json(e, today, plans.get(&e.plan_pid).map(String::as_str)))
            .collect::<Vec<_>>(),
    )
}
