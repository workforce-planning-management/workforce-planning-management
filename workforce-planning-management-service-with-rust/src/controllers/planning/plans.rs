//! Plans (part of [`super`]).

// A section of one controller: it shares the parent module's imports.
#[allow(clippy::wildcard_imports)]
use super::*;
use axa::extract::{Json, Path};

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
    let pid = Uuid::new_v4();
    let mut tx = crate::db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO workforce_plans (pid, name, organization_ref, horizon_start, horizon_end, rationale, attrition_bp,
                                      budget_minor, budget_currency, on_cost_bp, status)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 'draft')",
        pid,
        payload.name.trim(),
        payload.organization_ref,
        payload.horizon_start,
        payload.horizon_end,
        payload.rationale,
        payload.attrition_bp,
        payload.budget_minor,
        payload.budget_currency,
        payload.on_cost_bp,
    )
    .execute(&mut *tx)
    .await?;
    crate::db::audit(
        &mut tx,
        "workforce_plan",
        pid,
        "created",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(PidRef {
        pid: pid.to_string(),
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
    axa::extract::Query(query): axa::extract::Query<PlanListQuery>,
) -> Result<Response> {
    let scope = memberships::scope_organization_refs(&ctx.db, caller.claims()).await?;
    // Each plan with its line count, counted by the database, where every line of every plan was read.
    let out: Vec<serde_json::Value> = sqlx::query!(
        r#"SELECT p.pid, p.name, p.organization_ref, p.horizon_start, p.horizon_end, p.status, p.attrition_bp,
                  (SELECT COUNT(*) FROM plan_demand_lines l WHERE l.plan_pid = p.pid) AS "demand_lines!"
             FROM workforce_plans p
            WHERE p.deleted_at IS NULL AND ($1::text IS NULL OR p.organization_ref = $1)
              AND ($2::text[] IS NULL OR p.organization_ref = ANY($2))
            ORDER BY p.id DESC"#,
        query.organization,
        scope.as_deref(),
    )
    .fetch_all(crate::db::pool(&ctx))
    .await?
    .into_iter()
    .map(|p| {
        serde_json::json!({
            "pid": p.pid,
            "name": p.name,
            "organization_ref": p.organization_ref,
            "horizon_start": p.horizon_start,
            "horizon_end": p.horizon_end,
            "status": p.status,
            "attrition_bp": p.attrition_bp,
            "demand_lines": p.demand_lines,
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
    let plan = sqlx::query_as!(
        workforce_plans::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, name, organization_ref, horizon_start,
                  horizon_end, rationale, attrition_bp, budget_minor, budget_currency, on_cost_bp, status,
                  deleted_at AS "deleted_at: _"
             FROM workforce_plans WHERE pid = $1 AND deleted_at IS NULL"#,
        records::parse_pid(pid)?,
    )
    .fetch_optional(crate::db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)?;
    ensure_org_in_scope(ctx, caller, &plan.organization_ref).await?;
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
        "target_fte": l.target_fte_percent.map(|f| crate::rules::display::from_hundredths(i64::from(f))),
        "basis": l.basis,
        "expected_ends_on": l.expected_ends_on,
        "programme_ref": l.programme_ref,
        "note": l.note,
    })
}

pub(super) async fn role_titles(ctx: &AppContext) -> Result<BTreeMap<Uuid, String>> {
    Ok(sqlx::query!("SELECT pid, job_title FROM role_profiles")
        .fetch_all(crate::db::pool(ctx))
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
    let objectives = sqlx::query!(
        "SELECT pid, title, owner FROM plan_objectives WHERE plan_pid = $1 ORDER BY id",
        plan.pid,
    )
    .fetch_all(crate::db::pool(&ctx))
    .await?;
    // Only the links of this plan's lines, where every link was read.
    let line_pids: Vec<Uuid> = demand.iter().map(|l| l.pid).collect();
    let link_rows = sqlx::query!(
        "SELECT line_pid, objective_pid FROM demand_line_objectives WHERE line_pid = ANY($1) ORDER BY id",
        &line_pids,
    )
    .fetch_all(crate::db::pool(&ctx))
    .await?;
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
    Ok(sqlx::query_as!(
        plan_demand_lines::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, plan_pid, department,
                  role_profile_pid, target_on, target_headcount, note, programme_ref, target_fte_percent, basis,
                  expected_ends_on
             FROM plan_demand_lines WHERE plan_pid = $1 ORDER BY target_on, department"#,
        plan_pid,
    )
    .fetch_all(crate::db::pool(ctx))
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
        let other_active = sqlx::query_scalar!(
            r#"SELECT EXISTS (SELECT 1 FROM workforce_plans
                               WHERE organization_ref = $1 AND status = 'active' AND deleted_at IS NULL) AS "exists!""#,
            plan.organization_ref,
        )
        .fetch_one(crate::db::pool(&ctx))
        .await?;
        if other_active {
            return Err(unprocessable(
                "this organization already has an active plan; archive it first",
            ));
        }
    }
    let mut tx = crate::db::pool(&ctx).begin().await?;
    sqlx::query!(
        "UPDATE workforce_plans SET status = $2 WHERE id = $1",
        plan.id,
        payload.to
    )
    .execute(&mut *tx)
    .await?;
    crate::db::audit(
        &mut tx,
        "workforce_plan",
        plan.pid,
        "status_changed",
        caller.actor(),
        Some(serde_json::json!({ "from": plan.status, "to": payload.to })),
    )
    .await?;
    tx.commit().await?;
    format::json(PidRef {
        pid: plan.pid.to_string(),
    })
}
