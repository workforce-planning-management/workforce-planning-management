//! **Strategic workforce planning** (WPM-R36–R38, WPM-D26–D28): workforce
//! plans (draft worlds of aggregate hypothetical headcount), demand lines,
//! strategic objectives, the supply forecast and gap analysis, and the
//! alignment view. Pure rules in [`crate::rules::planning`].
//!
//! A plan never references a worker row (WPM-D26). The forecast is stated
//! assumptions plus arithmetic — with too little history it answers
//! `insufficient_history`, not an extrapolation (WPM-D27). Gaps are
//! aggregate and levers are suggestions (WPM-D28).

use loco_rs::prelude::*;
use sea_orm::{ActiveValue, QueryOrder};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

use super::{ensure_valid, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::models::_entities::{
    demand_line_objectives, headcount_snapshots, plan_demand_lines, plan_objectives, role_profiles,
    role_skill_requirements, succession_candidates, succession_plans, worker_skills, workers,
    workforce_plans,
};
use crate::models::audit_logs::Model as Audit;
use crate::models::{memberships, records};
use crate::rules::metrics::is_employed_on;
use crate::rules::planning as rules;
use crate::rules::talent as talent_rules;
use crate::validation::Problems;

/// A `{pid}` reference response.
#[derive(Serialize)]
struct PidRef {
    pid: String,
}

fn ratio_json(terms: Option<(usize, usize, f64)>) -> serde_json::Value {
    terms.map_or(serde_json::Value::Null, |(numerator, denominator, value)| {
        serde_json::json!({ "numerator": numerator, "denominator": denominator, "value": value })
    })
}

// ─── Plans ──────────────────────────────────────────────────────────────────

/// `POST /api/workforce-plans` body.
#[derive(Debug, Deserialize)]
struct PlanPayload {
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
}

/// `POST /api/workforce-plans` — open a draft plan.
#[debug_handler]
async fn create_plan(
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
    let row = workforce_plans::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        name: ActiveValue::set(payload.name.trim().to_string()),
        organization_ref: ActiveValue::set(payload.organization_ref.clone()),
        horizon_start: ActiveValue::set(payload.horizon_start),
        horizon_end: ActiveValue::set(payload.horizon_end),
        rationale: ActiveValue::set(payload.rationale.clone()),
        attrition_bp: ActiveValue::set(payload.attrition_bp),
        status: ActiveValue::set("draft".to_string()),
        deleted_at: ActiveValue::set(None),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;
    Audit::record(
        &ctx.db,
        "workforce_plan",
        row.pid,
        "created",
        caller.actor(),
        None,
    )
    .await?;
    format::json(PidRef {
        pid: row.pid.to_string(),
    })
}

/// Query for the plan list.
#[derive(Debug, Deserialize)]
struct PlanListQuery {
    organization: Option<String>,
}

/// `GET /api/workforce-plans?organization=` — live plans within the
/// caller's organizations, newest first.
#[debug_handler]
async fn list_plans(
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
async fn scoped_plan(
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

fn ensure_editable(plan: &workforce_plans::Model) -> Result<()> {
    if plan.status == "archived" {
        return Err(unprocessable("the plan is archived"));
    }
    Ok(())
}

fn line_json(l: &plan_demand_lines::Model, titles: &BTreeMap<Uuid, String>) -> serde_json::Value {
    serde_json::json!({
        "pid": l.pid,
        "department": l.department,
        "role_profile_pid": l.role_profile_pid,
        "job_title": l.role_profile_pid.and_then(|r| titles.get(&r)),
        "target_on": l.target_on,
        "target_headcount": l.target_headcount,
        "note": l.note,
    })
}

async fn role_titles(ctx: &AppContext) -> Result<BTreeMap<Uuid, String>> {
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
async fn get_plan(
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
        "status": plan.status,
        "demand_lines": lines_out,
        "objectives": objectives.iter().map(|o| serde_json::json!({
            "pid": o.pid, "title": o.title, "owner": o.owner,
        })).collect::<Vec<_>>(),
    }))
}

async fn plan_lines(ctx: &AppContext, plan_pid: Uuid) -> Result<Vec<plan_demand_lines::Model>> {
    Ok(plan_demand_lines::Entity::find()
        .filter(plan_demand_lines::Column::PlanPid.eq(plan_pid))
        .order_by_asc(plan_demand_lines::Column::TargetOn)
        .order_by_asc(plan_demand_lines::Column::Department)
        .all(&ctx.db)
        .await?)
}

/// `POST /api/workforce-plans/{pid}/status` body.
#[derive(Debug, Deserialize)]
struct StatusPayload {
    to: String,
}

/// `POST /api/workforce-plans/{pid}/status` — `draft → active → archived`.
/// Activating refuses when the organization already has an active plan.
#[debug_handler]
async fn set_status(
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
    let row = active.update(&ctx.db).await?;
    Audit::record(
        &ctx.db,
        "workforce_plan",
        row.pid,
        "status_changed",
        caller.actor(),
        Some(serde_json::json!({ "from": from, "to": payload.to })),
    )
    .await?;
    format::json(PidRef {
        pid: row.pid.to_string(),
    })
}

// ─── Demand lines ───────────────────────────────────────────────────────────

/// `PUT /api/workforce-plans/{pid}/demand-lines` body.
#[derive(Debug, Deserialize)]
struct LinePayload {
    department: String,
    #[serde(default)]
    role_profile_pid: Option<Uuid>,
    target_on: chrono::NaiveDate,
    target_headcount: i32,
    #[serde(default)]
    note: Option<String>,
}

/// `PUT /api/workforce-plans/{pid}/demand-lines` — set planned headcount
/// for a department (and optionally a role) at a date; upsert on that key.
#[debug_handler]
async fn set_line(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<LinePayload>,
) -> Result<Response> {
    let plan = scoped_plan(&ctx, &caller, &pid).await?;
    ensure_editable(&plan)?;
    let mut problems = Problems::new();
    problems.cap_text("department", &payload.department);
    if let Some(note) = &payload.note {
        problems.cap_text("note", note);
    }
    ensure_valid(&problems.into_vec())?;
    rules::validate_line(
        &payload.department,
        payload.target_on,
        plan.horizon_start,
        plan.horizon_end,
        payload.target_headcount,
    )
    .map_err(|e| unprocessable(&e))?;
    if let Some(role) = payload.role_profile_pid {
        role_profiles::Entity::find()
            .filter(role_profiles::Column::Pid.eq(role))
            .filter(role_profiles::Column::DeletedAt.is_null())
            .one(&ctx.db)
            .await?
            .ok_or(Error::NotFound)?;
    }
    let department = payload.department.trim().to_string();
    let mut existing = plan_demand_lines::Entity::find()
        .filter(plan_demand_lines::Column::PlanPid.eq(plan.pid))
        .filter(plan_demand_lines::Column::Department.eq(&department))
        .filter(plan_demand_lines::Column::TargetOn.eq(payload.target_on));
    existing = match payload.role_profile_pid {
        Some(role) => existing.filter(plan_demand_lines::Column::RoleProfilePid.eq(role)),
        None => existing.filter(plan_demand_lines::Column::RoleProfilePid.is_null()),
    };
    let row = match existing.one(&ctx.db).await? {
        Some(row) => {
            let mut active: plan_demand_lines::ActiveModel = row.into();
            active.target_headcount = ActiveValue::set(payload.target_headcount);
            active.note = ActiveValue::set(payload.note.clone());
            active.update(&ctx.db).await?
        }
        None => {
            plan_demand_lines::ActiveModel {
                pid: ActiveValue::set(Uuid::new_v4()),
                plan_pid: ActiveValue::set(plan.pid),
                department: ActiveValue::set(department),
                role_profile_pid: ActiveValue::set(payload.role_profile_pid),
                target_on: ActiveValue::set(payload.target_on),
                target_headcount: ActiveValue::set(payload.target_headcount),
                note: ActiveValue::set(payload.note.clone()),
                ..Default::default()
            }
            .insert(&ctx.db)
            .await?
        }
    };
    Audit::record(
        &ctx.db,
        "workforce_plan",
        plan.pid,
        "demand_line_set",
        caller.actor(),
        None,
    )
    .await?;
    format::json(PidRef {
        pid: row.pid.to_string(),
    })
}

/// `DELETE /api/workforce-plans/{pid}/demand-lines/{line_pid}`.
#[debug_handler]
async fn remove_line(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path((pid, line_pid)): Path<(String, String)>,
) -> Result<Response> {
    let plan = scoped_plan(&ctx, &caller, &pid).await?;
    ensure_editable(&plan)?;
    let line_pid = records::parse_pid(&line_pid)?;
    let row = plan_demand_lines::Entity::find()
        .filter(plan_demand_lines::Column::PlanPid.eq(plan.pid))
        .filter(plan_demand_lines::Column::Pid.eq(line_pid))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    demand_line_objectives::Entity::delete_many()
        .filter(demand_line_objectives::Column::LinePid.eq(line_pid))
        .exec(&ctx.db)
        .await?;
    row.delete(&ctx.db).await?;
    Audit::record(
        &ctx.db,
        "workforce_plan",
        plan.pid,
        "demand_line_removed",
        caller.actor(),
        None,
    )
    .await?;
    format::empty_json()
}

// ─── Objectives ─────────────────────────────────────────────────────────────

/// `POST /api/workforce-plans/{pid}/objectives` body.
#[derive(Debug, Deserialize)]
struct ObjectivePayload {
    title: String,
    #[serde(default)]
    owner: Option<String>,
}

/// `POST /api/workforce-plans/{pid}/objectives` — add a strategic
/// objective the plan serves.
#[debug_handler]
async fn create_objective(
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
struct LinkPayload {
    objective_pids: Vec<Uuid>,
}

/// `PUT /api/workforce-plans/{pid}/demand-lines/{line_pid}/objectives` —
/// replace the set of objectives a demand line serves.
#[debug_handler]
async fn set_line_objectives(
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

// ─── Forecast + gap analysis ────────────────────────────────────────────────

/// Employed workers of the plan's organization, today.
async fn employed_in_org(ctx: &AppContext, org: &str) -> Result<Vec<workers::Model>> {
    let today = chrono::Utc::now().date_naive();
    Ok(workers::Entity::find()
        .filter(workers::Column::DeletedAt.is_null())
        .filter(workers::Column::OrganizationRef.eq(org))
        .all(&ctx.db)
        .await?
        .into_iter()
        .filter(|w| is_employed_on(today, w.hired_on, w.terminated_on))
        .collect())
}

/// The attrition assumption in force: the plan's own, else the observed
/// rate from the organization's snapshots, else none.
async fn resolve_attrition(
    ctx: &AppContext,
    plan: &workforce_plans::Model,
) -> Result<(Option<i32>, &'static str)> {
    if let Some(bp) = plan.attrition_bp {
        return Ok((Some(bp), "plan_assumption"));
    }
    let rows = headcount_snapshots::Entity::find()
        .filter(headcount_snapshots::Column::OrganizationRef.eq(&plan.organization_ref))
        .all(&ctx.db)
        .await?;
    let mut by_date: BTreeMap<chrono::NaiveDate, (i64, Option<i64>)> = BTreeMap::new();
    for row in &rows {
        let entry = by_date.entry(row.as_of).or_insert((0, Some(0)));
        entry.0 += i64::from(row.headcount);
        entry.1 = match (entry.1, row.leavers) {
            (Some(total), Some(n)) => Some(total + i64::from(n)),
            _ => None,
        };
    }
    let snaps: Vec<rules::Snap> = by_date
        .into_iter()
        .map(|(as_of, (headcount, leavers))| rules::Snap {
            as_of,
            headcount,
            leavers,
        })
        .collect();
    Ok(match rules::observed_attrition_bp(&snaps) {
        Some(bp) => (Some(bp), "observed_snapshots"),
        None => (None, "insufficient_history"),
    })
}

/// `GET /api/workforce-plans/{pid}/forecast` — supply projection and gaps
/// per department per target date, with the assumptions and derivation.
///
/// Headcount: opening employed workers, projected after expected leavers
/// (no hires assumed), against the planned demand. Competency: for each
/// demand line with a role profile, how many employed workers in that
/// department are already proficient in each required skill. Aggregate
/// only; levers are suggestions.
#[debug_handler]
#[allow(clippy::too_many_lines)] // headcount gap + competency gap in one pass
async fn forecast(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let plan = scoped_plan(&ctx, &caller, &pid).await?;
    let today = chrono::Utc::now().date_naive();
    let employed = employed_in_org(&ctx, &plan.organization_ref).await?;
    let (attrition, source) = resolve_attrition(&ctx, &plan).await?;
    let lines = plan_lines(&ctx, plan.pid).await?;
    let titles = role_titles(&ctx).await?;
    let requirements = role_skill_requirements::Entity::find().all(&ctx.db).await?;
    let skill_names: BTreeMap<Uuid, String> = crate::models::_entities::skills::Entity::find()
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|s| (s.pid, s.name))
        .collect();
    let declared: BTreeMap<(Uuid, Uuid), i32> = worker_skills::Entity::find()
        .filter(worker_skills::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|d| ((d.worker_pid, d.skill_pid), d.proficiency))
        .collect();

    // Group lines by (department, target date).
    let mut groups: BTreeMap<(String, chrono::NaiveDate), Vec<&plan_demand_lines::Model>> =
        BTreeMap::new();
    for line in &lines {
        groups
            .entry((line.department.clone(), line.target_on))
            .or_default()
            .push(line);
    }
    let departments: Vec<serde_json::Value> = groups
        .iter()
        .map(|((department, target_on), group)| {
            let in_dept: Vec<&workers::Model> = employed
                .iter()
                .filter(|w| &w.department == department)
                .collect();
            let opening = in_dept.len();
            let demand: i64 = group.iter().map(|l| i64::from(l.target_headcount)).sum();
            let days = (*target_on - today).num_days().max(0);
            let supply = attrition.map(|bp| rules::project_supply(opening, bp, days));
            let gap = supply.map(|s| rules::headcount_gap(demand, s));
            let competency: Vec<serde_json::Value> = group
                .iter()
                .filter_map(|line| line.role_profile_pid.map(|role| (line, role)))
                .flat_map(|(line, role)| {
                    let needed = usize::try_from(line.target_headcount).unwrap_or(0);
                    requirements
                        .iter()
                        .filter(move |r| r.role_profile_pid == role)
                        .map(|r| {
                            let (mut proficient, mut reskill_pool) = (0usize, 0usize);
                            for worker in &in_dept {
                                match declared.get(&(worker.pid, r.skill_pid)) {
                                    Some(level) if *level >= r.min_proficiency => proficient += 1,
                                    Some(_) => reskill_pool += 1,
                                    None => {}
                                }
                            }
                            serde_json::json!({
                                "job_title": titles.get(&role),
                                "skill": skill_names.get(&r.skill_pid),
                                "importance": r.importance,
                                "min_proficiency": r.min_proficiency,
                                "needed": needed,
                                "proficient_now": proficient,
                                "shortfall": rules::competency_shortfall(needed, proficient),
                                "reskill_pool": reskill_pool,
                            })
                        })
                        .collect::<Vec<_>>()
                })
                .collect();
            serde_json::json!({
                "department": department,
                "target_on": target_on,
                "days_ahead": days,
                "opening_headcount": opening,
                "planned_demand": demand,
                "projected_supply": supply,
                "headcount_gap": gap,
                "levers": gap.map(rules::suggested_levers),
                "competency_gaps": competency,
            })
        })
        .collect();

    format::json(serde_json::json!({
        "derivation": "opening = employed workers in the department today; projected supply = \
                       opening less expected leavers at the stated attrition (no hires assumed); \
                       gap = planned demand − projected supply (positive = shortfall). Lines for \
                       the same department and date add up. Competency counts EMPLOYED workers in \
                       the department whose DECLARED proficiency meets the requirement; the \
                       reskill pool is those who declared it below the bar. Aggregate only; \
                       levers are suggestions, not decisions.",
        "plan": { "pid": plan.pid, "name": plan.name, "status": plan.status },
        "as_of": today,
        "assumptions": {
            "attrition_bp": attrition,
            "attrition_source": source,
            "hires_assumed": 0,
        },
        "departments": departments,
    }))
}

// ─── Alignment ──────────────────────────────────────────────────────────────

/// `GET /api/workforce-plans/{pid}/alignment` — does the planned headcount
/// serve the strategy? Share of planned headcount tied to an objective,
/// objectives with no demand behind them, demand lines serving no
/// objective, and critical roles with no ready successor.
#[debug_handler]
async fn alignment(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let plan = scoped_plan(&ctx, &caller, &pid).await?;
    let lines = plan_lines(&ctx, plan.pid).await?;
    let objectives = plan_objectives::Entity::find()
        .filter(plan_objectives::Column::PlanPid.eq(plan.pid))
        .all(&ctx.db)
        .await?;
    let line_pids: BTreeSet<Uuid> = lines.iter().map(|l| l.pid).collect();
    let link_rows: Vec<demand_line_objectives::Model> = demand_line_objectives::Entity::find()
        .all(&ctx.db)
        .await?
        .into_iter()
        .filter(|k| line_pids.contains(&k.line_pid))
        .collect();
    let linked_lines: BTreeSet<Uuid> = link_rows.iter().map(|k| k.line_pid).collect();
    let served_objectives: BTreeSet<Uuid> = link_rows.iter().map(|k| k.objective_pid).collect();

    let total_planned: usize = lines
        .iter()
        .map(|l| usize::try_from(l.target_headcount).unwrap_or(0))
        .sum();
    let aligned_planned: usize = lines
        .iter()
        .filter(|l| linked_lines.contains(&l.pid))
        .map(|l| usize::try_from(l.target_headcount).unwrap_or(0))
        .sum();
    let unresourced: Vec<&str> = objectives
        .iter()
        .filter(|o| !served_objectives.contains(&o.pid))
        .map(|o| o.title.as_str())
        .collect();
    let unaligned: Vec<serde_json::Value> = lines
        .iter()
        .filter(|l| !linked_lines.contains(&l.pid))
        .map(|l| serde_json::json!({ "department": l.department, "target_on": l.target_on, "target_headcount": l.target_headcount }))
        .collect();

    // Critical roles with no ready successor (the shared single-point rule).
    let plan_departments: BTreeSet<&str> = lines.iter().map(|l| l.department.as_str()).collect();
    let succession = succession_plans::Entity::find()
        .filter(succession_plans::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?;
    let candidates = succession_candidates::Entity::find()
        .filter(succession_candidates::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?;
    let (mut spof_total, mut spof_in_plan) = (0usize, 0usize);
    for s in &succession {
        let readiness: Vec<String> = candidates
            .iter()
            .filter(|c| c.plan_pid == s.pid)
            .map(|c| c.readiness.clone())
            .collect();
        let coverage = talent_rules::bench_coverage(&readiness);
        if talent_rules::is_single_point_of_failure(
            s.criticality,
            coverage,
            s.risk_of_loss.as_deref(),
        ) {
            spof_total += 1;
            if plan_departments.contains(s.department.as_str()) {
                spof_in_plan += 1;
            }
        }
    }

    format::json(serde_json::json!({
        "derivation": "planned headcount sums the plan's demand lines; a line is aligned when it \
                       serves at least one objective. Critical roles with no ready successor use \
                       the shared single-point-of-failure rule. Terms-carrying ratios; null when \
                       there is nothing to divide.",
        "plan": { "pid": plan.pid, "name": plan.name, "status": plan.status },
        "planned_headcount": total_planned,
        "aligned_share": ratio_json(talent_rules::ratio(aligned_planned, total_planned)),
        "objectives": objectives.len(),
        "unresourced_objectives": unresourced,
        "unaligned_demand_lines": unaligned,
        "critical_roles_without_bench": {
            "in_plan_departments": spof_in_plan,
            "all_departments": spof_total,
        },
    }))
}

/// The workforce-planning routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/workforce-plans", post(create_plan))
        .add("/workforce-plans", get(list_plans))
        .add("/workforce-plans/{pid}", get(get_plan))
        .add("/workforce-plans/{pid}/status", post(set_status))
        .add("/workforce-plans/{pid}/demand-lines", put(set_line))
        .add(
            "/workforce-plans/{pid}/demand-lines/{line_pid}",
            delete(remove_line),
        )
        .add(
            "/workforce-plans/{pid}/demand-lines/{line_pid}/objectives",
            put(set_line_objectives),
        )
        .add("/workforce-plans/{pid}/objectives", post(create_objective))
        .add("/workforce-plans/{pid}/forecast", get(forecast))
        .add("/workforce-plans/{pid}/alignment", get(alignment))
}
