//! Objectives (part of [`super`]).

// A section of one controller: it shares the parent module's imports.
#[allow(clippy::wildcard_imports)]
use super::*;

/// `POST /api/workforce-plans/{pid}/objectives` body.
#[derive(Debug, Deserialize)]
pub(super) struct ObjectivePayload {
    title: String,
    #[serde(default)]
    owner: Option<String>,
}

/// `POST /api/workforce-plans/{pid}/objectives` — add a strategic
/// objective the plan serves.
#[debug_handler]
pub(super) async fn create_objective(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<ObjectivePayload>,
) -> Result<Response> {
    let plan = scoped_plan(&ctx, &caller, &pid).await?;
    ensure_editable(&plan)?;
    let mut problems = Problems::new();
    problems.require_text("title", &payload.title);
    problems.cap_text("title", &payload.title);
    if let Some(owner) = &payload.owner {
        problems.cap_text("owner", owner);
    }
    ensure_valid(&problems.into_vec())?;
    let row = plan_objectives::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        plan_pid: ActiveValue::set(plan.pid),
        title: ActiveValue::set(payload.title.trim().to_string()),
        owner: ActiveValue::set(payload.owner.clone()),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;
    Audit::record(
        &ctx.db,
        "workforce_plan",
        plan.pid,
        "objective_added",
        caller.actor(),
        None,
    )
    .await?;
    format::json(PidRef {
        pid: row.pid.to_string(),
    })
}

/// `PUT /api/workforce-plans/{pid}/demand-lines/{line_pid}/objectives` body.
#[derive(Debug, Deserialize)]
pub(super) struct LinkPayload {
    objective_pids: Vec<Uuid>,
}

/// `PUT /api/workforce-plans/{pid}/demand-lines/{line_pid}/objectives` —
/// replace the set of objectives a demand line serves.
#[debug_handler]
pub(super) async fn set_line_objectives(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path((pid, line_pid)): Path<(String, String)>,
    Json(payload): Json<LinkPayload>,
) -> Result<Response> {
    let plan = scoped_plan(&ctx, &caller, &pid).await?;
    ensure_editable(&plan)?;
    let line = plan_demand_lines::Entity::find()
        .filter(plan_demand_lines::Column::PlanPid.eq(plan.pid))
        .filter(plan_demand_lines::Column::Pid.eq(records::parse_pid(&line_pid)?))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    let known: BTreeSet<Uuid> = plan_objectives::Entity::find()
        .filter(plan_objectives::Column::PlanPid.eq(plan.pid))
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|o| o.pid)
        .collect();
    let wanted: BTreeSet<Uuid> = payload.objective_pids.iter().copied().collect();
    if !wanted.is_subset(&known) {
        return Err(unprocessable("objective_pids must belong to this plan"));
    }
    demand_line_objectives::Entity::delete_many()
        .filter(demand_line_objectives::Column::LinePid.eq(line.pid))
        .exec(&ctx.db)
        .await?;
    for objective in &wanted {
        demand_line_objectives::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            line_pid: ActiveValue::set(line.pid),
            objective_pid: ActiveValue::set(*objective),
            ..Default::default()
        }
        .insert(&ctx.db)
        .await?;
    }
    Audit::record(
        &ctx.db,
        "workforce_plan",
        plan.pid,
        "line_objectives_set",
        caller.actor(),
        None,
    )
    .await?;
    format::empty_json()
}
