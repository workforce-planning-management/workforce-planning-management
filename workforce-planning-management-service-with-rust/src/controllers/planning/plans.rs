//! Plans (part of [`super`]).

// A section of one controller: it shares the parent module's imports.
#[allow(clippy::wildcard_imports)]
use super::*;

/// `POST /api/workforce-plans` body.
#[derive(Debug, Deserialize)]
pub(super) struct PlanPayload {
    name: String,
    organization_ref: String,
    horizon_start: chrono::NaiveDate,
    horizon_end: chrono::NaiveDate,
    #[serde(default)]
    rationale: Option<String>,
    /// Assumed annual attrition, in basis points (1200 = 12%). Absent ⇒
    /// the observed rate from headcount snapshots is used, if there is
    /// enough history.
    #[serde(default)]
    attrition_bp: Option<i32>,
    /// Annual budget for the plan's hiring, in minor units; needs
    /// `budget_currency`.
    #[serde(default)]
    budget_minor: Option<i64>,
    #[serde(default)]
    budget_currency: Option<String>,
    /// Employer on-cost on top of salary, in basis points (2500 = +25%).
    #[serde(default)]
    on_cost_bp: Option<i32>,
}

/// Validate the plan's optional financial assumptions.
pub(super) fn validate_finance(payload: &PlanPayload) -> std::result::Result<(), String> {
    match (&payload.budget_minor, &payload.budget_currency) {
        (None, None) => {}
        (Some(minor), Some(code)) => {
            if *minor < 0 {
                return Err("budget_minor must not be negative".to_string());
            }
            if !cost_rules::valid_currency(code) {
                return Err("budget_currency must be an ISO-4217 code, e.g. GBP".to_string());
            }
        }
        _ => return Err("give budget_minor and budget_currency together".to_string()),
    }
    if payload
        .on_cost_bp
        .is_some_and(|bp| !(0..=10_000).contains(&bp))
    {
        return Err("on_cost_bp must be between 0 and 10000".to_string());
    }
    Ok(())
}

/// `POST /api/workforce-plans` — open a draft plan.
#[debug_handler]
pub(super) async fn create_plan(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<PlanPayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_ref(
        "organization_ref",
        entity_ref::EntityType::Organization,
        &payload.organization_ref,
    );
    problems.cap_text("name", &payload.name);
    if let Some(text) = &payload.rationale {
        problems.cap_text("rationale", text);
    }
    ensure_valid(&problems.into_vec())?;
    rules::validate_plan(
        &payload.name,
        payload.horizon_start,
        payload.horizon_end,
        payload.attrition_bp,
    )
    .map_err(|e| unprocessable(&e))?;
    validate_finance(&payload).map_err(|e| unprocessable(&e))?;
    let txn = ctx.db.begin().await?;
    let row = workforce_plans::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        name: ActiveValue::set(payload.name.trim().to_string()),
        organization_ref: ActiveValue::set(payload.organization_ref.clone()),
        horizon_start: ActiveValue::set(payload.horizon_start),
        horizon_end: ActiveValue::set(payload.horizon_end),
        rationale: ActiveValue::set(payload.rationale.clone()),
        attrition_bp: ActiveValue::set(payload.attrition_bp),
        budget_minor: ActiveValue::set(payload.budget_minor),
        budget_currency: ActiveValue::set(payload.budget_currency.clone()),
        on_cost_bp: ActiveValue::set(payload.on_cost_bp),
        status: ActiveValue::set("draft".to_string()),
        deleted_at: ActiveValue::set(None),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    Audit::record(
        &txn,
        "workforce_plan",
        row.pid,
        "created",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(PidRef {
        pid: row.pid.to_string(),
    })
}

/// Query for the plan list.
#[derive(Debug, Deserialize)]
pub(super) struct PlanListQuery {
    organization: Option<String>,
}

/// `GET /api/workforce-plans?organization=` — live plans within the
/// caller's organizations, newest first.
#[debug_handler]
pub(super) async fn list_plans(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(query): axum::extract::Query<PlanListQuery>,
) -> Result<Response> {
    let mut select =
        workforce_plans::Entity::find().filter(workforce_plans::Column::DeletedAt.is_null());
    if let Some(org) = &query.organization {
        select = select.filter(workforce_plans::Column::OrganizationRef.eq(org));
    }
    if let Some(refs) = memberships::scope_organization_refs(&ctx.db, caller.claims()).await? {
        select = select.filter(workforce_plans::Column::OrganizationRef.is_in(refs));
    }
    let plans = select
        .order_by_desc(workforce_plans::Column::Id)
        .all(&ctx.db)
        .await?;
    let lines = plan_demand_lines::Entity::find().all(&ctx.db).await?;
    let out: Vec<serde_json::Value> = plans
        .iter()
        .map(|p| {
            serde_json::json!({
                "pid": p.pid,
                "name": p.name,
                "organization_ref": p.organization_ref,
                "horizon_start": p.horizon_start,
                "horizon_end": p.horizon_end,
                "status": p.status,
                "attrition_bp": p.attrition_bp,
                "demand_lines": lines.iter().filter(|l| l.plan_pid == p.pid).count(),
            })
        })
        .collect();
    format::json(out)
}

/// Find a live plan within the caller's organization scope; a plan outside
/// it is `404`, as for every other scoped read.
pub(super) async fn scoped_plan(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: &str,
) -> Result<workforce_plans::Model> {
    let plan = workforce_plans::Entity::find()
        .filter(workforce_plans::Column::Pid.eq(records::parse_pid(pid)?))
        .filter(workforce_plans::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    if let Some(refs) = memberships::scope_organization_refs(&ctx.db, caller.claims()).await?
        && !refs.contains(&plan.organization_ref)
    {
        return Err(Error::NotFound);
    }
    Ok(plan)
}

pub(super) fn ensure_editable(plan: &workforce_plans::Model) -> Result<()> {
    if plan.status == "archived" {
        return Err(unprocessable("the plan is archived"));
    }
    Ok(())
}

pub(super) fn line_json(
    l: &plan_demand_lines::Model,
    titles: &BTreeMap<Uuid, String>,
) -> serde_json::Value {
    serde_json::json!({
        "pid": l.pid,
        "department": l.department,
        "role_profile_pid": l.role_profile_pid,
        "job_title": l.role_profile_pid.and_then(|r| titles.get(&r)),
        "target_on": l.target_on,
        "target_headcount": l.target_headcount,
        "target_fte": l.target_fte_percent.map(|f| f64::from(f) / 100.0),
        "basis": l.basis,
        "expected_ends_on": l.expected_ends_on,
        "programme_ref": l.programme_ref,
        "note": l.note,
    })
}

pub(super) async fn role_titles(ctx: &AppContext) -> Result<BTreeMap<Uuid, String>> {
    Ok(role_profiles::Entity::find()
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|p| (p.pid, p.job_title))
        .collect())
}

/// `GET /api/workforce-plans/{pid}` — a plan with its demand lines and
/// objectives (and which objectives each line serves).
#[debug_handler]
pub(super) async fn get_plan(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let plan = scoped_plan(&ctx, &caller, &pid).await?;
    let titles = role_titles(&ctx).await?;
    let demand = plan_lines(&ctx, plan.pid).await?;
    let objectives = plan_objectives::Entity::find()
        .filter(plan_objectives::Column::PlanPid.eq(plan.pid))
        .order_by_asc(plan_objectives::Column::Id)
        .all(&ctx.db)
        .await?;
    let link_rows = demand_line_objectives::Entity::find().all(&ctx.db).await?;
    let lines_out: Vec<serde_json::Value> = demand
        .iter()
        .map(|l| {
            let mut json = line_json(l, &titles);
            json["objective_pids"] = serde_json::json!(
                link_rows
                    .iter()
                    .filter(|k| k.line_pid == l.pid)
                    .map(|k| k.objective_pid)
                    .collect::<Vec<_>>()
            );
            json
        })
        .collect();
    format::json(serde_json::json!({
        "pid": plan.pid,
        "name": plan.name,
        "organization_ref": plan.organization_ref,
        "horizon_start": plan.horizon_start,
        "horizon_end": plan.horizon_end,
        "rationale": plan.rationale,
        "attrition_bp": plan.attrition_bp,
        "budget_minor": plan.budget_minor,
        "budget_currency": plan.budget_currency,
        "on_cost_bp": plan.on_cost_bp,
        "status": plan.status,
        "demand_lines": lines_out,
        "objectives": objectives.iter().map(|o| serde_json::json!({
            "pid": o.pid, "title": o.title, "owner": o.owner,
        })).collect::<Vec<_>>(),
    }))
}

pub(super) async fn plan_lines(
    ctx: &AppContext,
    plan_pid: Uuid,
) -> Result<Vec<plan_demand_lines::Model>> {
    Ok(plan_demand_lines::Entity::find()
        .filter(plan_demand_lines::Column::PlanPid.eq(plan_pid))
        .order_by_asc(plan_demand_lines::Column::TargetOn)
        .order_by_asc(plan_demand_lines::Column::Department)
        .all(&ctx.db)
        .await?)
}

/// `POST /api/workforce-plans/{pid}/status` body.
#[derive(Debug, Deserialize)]
pub(super) struct StatusPayload {
    to: String,
}

/// `POST /api/workforce-plans/{pid}/status` — `draft → active → archived`.
/// Activating refuses when the organization already has an active plan.
#[debug_handler]
pub(super) async fn set_status(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<StatusPayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_token("to", rules::STATUSES, &payload.to);
    ensure_valid(&problems.into_vec())?;
    let plan = scoped_plan(&ctx, &caller, &pid).await?;
    rules::check_transition(&plan.status, &payload.to).map_err(|e| unprocessable(&e))?;
    if payload.to == "active" {
        let other_active = workforce_plans::Entity::find()
            .filter(workforce_plans::Column::OrganizationRef.eq(&plan.organization_ref))
            .filter(workforce_plans::Column::Status.eq("active"))
            .filter(workforce_plans::Column::DeletedAt.is_null())
            .one(&ctx.db)
            .await?
            .is_some();
        if other_active {
            return Err(unprocessable(
                "this organization already has an active plan; archive it first",
            ));
        }
    }
    let from = plan.status.clone();
    let mut active: workforce_plans::ActiveModel = plan.into();
    active.status = ActiveValue::set(payload.to.clone());
    let txn = ctx.db.begin().await?;
    let row = active.update(&txn).await?;
    Audit::record(
        &txn,
        "workforce_plan",
        row.pid,
        "status_changed",
        caller.actor(),
        Some(serde_json::json!({ "from": from, "to": payload.to })),
    )
    .await?;
    txn.commit().await?;
    format::json(PidRef {
        pid: row.pid.to_string(),
    })
}
