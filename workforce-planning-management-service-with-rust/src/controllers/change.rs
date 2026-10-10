//! The **AI-driven change tracker** (WPM-D28): an operational record of an
//! automation or AI initiative and its impact on *roles* and *skills*, with
//! an aggregate readiness view of the workforce affected. Pure rules in
//! [`crate::rules::change`].
//!
//! It tracks positions and capabilities, never people: readiness is counts
//! and ratios over the **employed** workers whose job title matches an
//! affected role, and no individual is named or ranked.

use loco_rs::prelude::*;
use sea_orm::{ActiveValue, QueryOrder};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

use super::{ensure_valid, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::models::_entities::{
    change_initiatives, development_plans, initiative_role_impacts, initiative_skill_shifts,
    role_profiles, skills, worker_skills, workers,
};
use crate::models::audit_logs::Model as Audit;
use crate::models::records;
use crate::rules::change as rules;
use crate::rules::metrics::is_employed_on;
use crate::rules::mobility::same_title;
use crate::rules::talent::ratio;
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

/// `POST /api/change-initiatives` body.
#[derive(Debug, Deserialize)]
struct InitiativePayload {
    name: String,
    kind: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    starts_on: Option<chrono::NaiveDate>,
}

/// `POST /api/change-initiatives` — open a draft initiative.
#[debug_handler]
async fn create_initiative(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<InitiativePayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_text("name", &payload.name);
    problems.cap_text("name", &payload.name);
    problems.require_token("kind", rules::KINDS, &payload.kind);
    if let Some(text) = &payload.description {
        problems.cap_text("description", text);
    }
    ensure_valid(&problems.into_vec())?;
    let row = change_initiatives::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        name: ActiveValue::set(payload.name.trim().to_string()),
        description: ActiveValue::set(payload.description.clone()),
        kind: ActiveValue::set(payload.kind.clone()),
        status: ActiveValue::set("draft".to_string()),
        starts_on: ActiveValue::set(payload.starts_on),
        deleted_at: ActiveValue::set(None),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;
    Audit::record(
        &ctx.db,
        "change_initiative",
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

/// `GET /api/change-initiatives` — live initiatives, newest first, with
/// how many roles and skills each touches.
#[debug_handler]
async fn list_initiatives(State(ctx): State<AppContext>) -> Result<Response> {
    let rows = change_initiatives::Entity::find()
        .filter(change_initiatives::Column::DeletedAt.is_null())
        .order_by_desc(change_initiatives::Column::Id)
        .all(&ctx.db)
        .await?;
    let impacts = initiative_role_impacts::Entity::find().all(&ctx.db).await?;
    let shifts = initiative_skill_shifts::Entity::find().all(&ctx.db).await?;
    let out: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "pid": r.pid,
                "name": r.name,
                "kind": r.kind,
                "status": r.status,
                "starts_on": r.starts_on,
                "roles_affected": impacts.iter().filter(|i| i.initiative_pid == r.pid).count(),
                "skills_shifting": shifts.iter().filter(|s| s.initiative_pid == r.pid).count(),
            })
        })
        .collect();
    format::json(out)
}

async fn find_initiative(ctx: &AppContext, pid: &str) -> Result<change_initiatives::Model> {
    change_initiatives::Entity::find()
        .filter(change_initiatives::Column::Pid.eq(records::parse_pid(pid)?))
        .filter(change_initiatives::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)
}

/// `GET /api/change-initiatives/{pid}` — an initiative with its role
/// impacts and skill shifts.
#[debug_handler]
async fn get_initiative(
    State(ctx): State<AppContext>,
    Path(pid): Path<String>,
) -> Result<Response> {
    let initiative = find_initiative(&ctx, &pid).await?;
    let titles: BTreeMap<Uuid, String> = role_profiles::Entity::find()
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|p| (p.pid, p.job_title))
        .collect();
    let names: BTreeMap<Uuid, String> = skills::Entity::find()
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|s| (s.pid, s.name))
        .collect();
    let role_impacts: Vec<serde_json::Value> = initiative_role_impacts::Entity::find()
        .filter(initiative_role_impacts::Column::InitiativePid.eq(initiative.pid))
        .all(&ctx.db)
        .await?
        .iter()
        .map(|i| {
            serde_json::json!({
                "role_profile_pid": i.role_profile_pid,
                "job_title": titles.get(&i.role_profile_pid),
                "impact": i.impact,
                "timeframe": i.timeframe,
                "note": i.note,
            })
        })
        .collect();
    let skill_shifts: Vec<serde_json::Value> = initiative_skill_shifts::Entity::find()
        .filter(initiative_skill_shifts::Column::InitiativePid.eq(initiative.pid))
        .all(&ctx.db)
        .await?
        .iter()
        .map(|s| {
            serde_json::json!({
                "skill_pid": s.skill_pid,
                "skill": names.get(&s.skill_pid),
                "direction": s.direction,
                "note": s.note,
            })
        })
        .collect();
    format::json(serde_json::json!({
        "pid": initiative.pid,
        "name": initiative.name,
        "description": initiative.description,
        "kind": initiative.kind,
        "status": initiative.status,
        "starts_on": initiative.starts_on,
        "role_impacts": role_impacts,
        "skill_shifts": skill_shifts,
    }))
}

/// `POST /api/change-initiatives/{pid}/status` body.
#[derive(Debug, Deserialize)]
struct StatusPayload {
    to: String,
}

/// `POST /api/change-initiatives/{pid}/status` — move an initiative
/// through its lifecycle (`draft → active → completed | cancelled`).
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
    let initiative = find_initiative(&ctx, &pid).await?;
    rules::check_transition(&initiative.status, &payload.to).map_err(|e| unprocessable(&e))?;
    let from = initiative.status.clone();
    let mut active: change_initiatives::ActiveModel = initiative.into();
    active.status = ActiveValue::set(payload.to.clone());
    let row = active.update(&ctx.db).await?;
    Audit::record(
        &ctx.db,
        "change_initiative",
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

/// Initiatives are editable until they reach a terminal status.
fn ensure_editable(initiative: &change_initiatives::Model) -> Result<()> {
    if matches!(initiative.status.as_str(), "completed" | "cancelled") {
        return Err(unprocessable("the initiative is closed"));
    }
    Ok(())
}

/// `PUT /api/change-initiatives/{pid}/role-impacts` body.
#[derive(Debug, Deserialize)]
struct RoleImpactPayload {
    role_profile_pid: Uuid,
    impact: String,
    timeframe: String,
    #[serde(default)]
    note: Option<String>,
}

/// `PUT /api/change-initiatives/{pid}/role-impacts` — record how a role is
/// affected (upsert per role).
#[debug_handler]
async fn set_role_impact(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<RoleImpactPayload>,
) -> Result<Response> {
    rules::validate_role_impact(&payload.impact, &payload.timeframe)
        .map_err(|e| unprocessable(&e))?;
    let mut problems = Problems::new();
    if let Some(note) = &payload.note {
        problems.cap_text("note", note);
    }
    ensure_valid(&problems.into_vec())?;
    let initiative = find_initiative(&ctx, &pid).await?;
    ensure_editable(&initiative)?;
    role_profiles::Entity::find()
        .filter(role_profiles::Column::Pid.eq(payload.role_profile_pid))
        .filter(role_profiles::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    let existing = initiative_role_impacts::Entity::find()
        .filter(initiative_role_impacts::Column::InitiativePid.eq(initiative.pid))
        .filter(initiative_role_impacts::Column::RoleProfilePid.eq(payload.role_profile_pid))
        .one(&ctx.db)
        .await?;
    match existing {
        Some(row) => {
            let mut active: initiative_role_impacts::ActiveModel = row.into();
            active.impact = ActiveValue::set(payload.impact.clone());
            active.timeframe = ActiveValue::set(payload.timeframe.clone());
            active.note = ActiveValue::set(payload.note.clone());
            active.update(&ctx.db).await?;
        }
        None => {
            initiative_role_impacts::ActiveModel {
                pid: ActiveValue::set(Uuid::new_v4()),
                initiative_pid: ActiveValue::set(initiative.pid),
                role_profile_pid: ActiveValue::set(payload.role_profile_pid),
                impact: ActiveValue::set(payload.impact.clone()),
                timeframe: ActiveValue::set(payload.timeframe.clone()),
                note: ActiveValue::set(payload.note.clone()),
                ..Default::default()
            }
            .insert(&ctx.db)
            .await?;
        }
    }
    Audit::record(
        &ctx.db,
        "change_initiative",
        initiative.pid,
        "role_impact_set",
        caller.actor(),
        Some(serde_json::json!({ "role_profile_pid": payload.role_profile_pid, "impact": payload.impact })),
    )
    .await?;
    format::empty_json()
}

/// `DELETE /api/change-initiatives/{pid}/role-impacts/{role_profile_pid}`.
#[debug_handler]
async fn remove_role_impact(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path((pid, role)): Path<(String, String)>,
) -> Result<Response> {
    let initiative = find_initiative(&ctx, &pid).await?;
    ensure_editable(&initiative)?;
    let role = records::parse_pid(&role)?;
    let row = initiative_role_impacts::Entity::find()
        .filter(initiative_role_impacts::Column::InitiativePid.eq(initiative.pid))
        .filter(initiative_role_impacts::Column::RoleProfilePid.eq(role))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    row.delete(&ctx.db).await?;
    Audit::record(
        &ctx.db,
        "change_initiative",
        initiative.pid,
        "role_impact_removed",
        caller.actor(),
        None,
    )
    .await?;
    format::empty_json()
}

/// `PUT /api/change-initiatives/{pid}/skill-shifts` body.
#[derive(Debug, Deserialize)]
struct SkillShiftPayload {
    skill_pid: Uuid,
    direction: String,
    #[serde(default)]
    note: Option<String>,
}

/// `PUT /api/change-initiatives/{pid}/skill-shifts` — record a skill whose
/// demand the initiative raises or lowers (upsert per skill).
#[debug_handler]
async fn set_skill_shift(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<SkillShiftPayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_token("direction", rules::DIRECTIONS, &payload.direction);
    if let Some(note) = &payload.note {
        problems.cap_text("note", note);
    }
    ensure_valid(&problems.into_vec())?;
    let initiative = find_initiative(&ctx, &pid).await?;
    ensure_editable(&initiative)?;
    skills::Entity::find()
        .filter(skills::Column::Pid.eq(payload.skill_pid))
        .filter(skills::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    let existing = initiative_skill_shifts::Entity::find()
        .filter(initiative_skill_shifts::Column::InitiativePid.eq(initiative.pid))
        .filter(initiative_skill_shifts::Column::SkillPid.eq(payload.skill_pid))
        .one(&ctx.db)
        .await?;
    match existing {
        Some(row) => {
            let mut active: initiative_skill_shifts::ActiveModel = row.into();
            active.direction = ActiveValue::set(payload.direction.clone());
            active.note = ActiveValue::set(payload.note.clone());
            active.update(&ctx.db).await?;
        }
        None => {
            initiative_skill_shifts::ActiveModel {
                pid: ActiveValue::set(Uuid::new_v4()),
                initiative_pid: ActiveValue::set(initiative.pid),
                skill_pid: ActiveValue::set(payload.skill_pid),
                direction: ActiveValue::set(payload.direction.clone()),
                note: ActiveValue::set(payload.note.clone()),
                ..Default::default()
            }
            .insert(&ctx.db)
            .await?;
        }
    }
    Audit::record(
        &ctx.db,
        "change_initiative",
        initiative.pid,
        "skill_shift_set",
        caller.actor(),
        None,
    )
    .await?;
    format::empty_json()
}

/// `DELETE /api/change-initiatives/{pid}/skill-shifts/{skill_pid}`.
#[debug_handler]
async fn remove_skill_shift(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path((pid, skill)): Path<(String, String)>,
) -> Result<Response> {
    let initiative = find_initiative(&ctx, &pid).await?;
    ensure_editable(&initiative)?;
    let skill = records::parse_pid(&skill)?;
    let row = initiative_skill_shifts::Entity::find()
        .filter(initiative_skill_shifts::Column::InitiativePid.eq(initiative.pid))
        .filter(initiative_skill_shifts::Column::SkillPid.eq(skill))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    row.delete(&ctx.db).await?;
    Audit::record(
        &ctx.db,
        "change_initiative",
        initiative.pid,
        "skill_shift_removed",
        caller.actor(),
        None,
    )
    .await?;
    format::empty_json()
}

/// Query for the readiness view.
#[derive(Debug, Deserialize)]
struct ReadinessQuery {
    /// The proficiency (1–5) counted as ready for a rising skill. Default 3.
    min_proficiency: Option<i32>,
}

/// `GET /api/change-initiatives/{pid}/readiness?min_proficiency=` —
/// **aggregate only**: for each affected role, the employed workers in it
/// and how many have an active reskill plan; and for each rising skill,
/// how many of the workers in displaced or reshaped roles already meet,
/// fall below, or have not declared it. No individual is named.
#[debug_handler]
#[allow(clippy::too_many_lines)] // roles, workers, plans, and skills in one aggregate pass
async fn readiness(
    State(ctx): State<AppContext>,
    Path(pid): Path<String>,
    axum::extract::Query(query): axum::extract::Query<ReadinessQuery>,
) -> Result<Response> {
    let bar = query
        .min_proficiency
        .unwrap_or(crate::rules::capability::DEFAULT_MIN_PROFICIENCY);
    if !crate::rules::learning::valid_proficiency(bar) {
        return Err(unprocessable("min_proficiency must be between 1 and 5"));
    }
    let initiative = find_initiative(&ctx, &pid).await?;
    let today = chrono::Utc::now().date_naive();
    let employed: Vec<workers::Model> = workers::Entity::find()
        .filter(workers::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?
        .into_iter()
        .filter(|w| is_employed_on(today, w.hired_on, w.terminated_on))
        .collect();
    let with_reskill_plan: BTreeSet<Uuid> = development_plans::Entity::find()
        .filter(development_plans::Column::DeletedAt.is_null())
        .filter(development_plans::Column::Kind.eq("reskill"))
        .filter(development_plans::Column::Status.eq("active"))
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|p| p.worker_pid)
        .collect();
    let titles: BTreeMap<Uuid, String> = role_profiles::Entity::find()
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|p| (p.pid, p.job_title))
        .collect();
    let impacts = initiative_role_impacts::Entity::find()
        .filter(initiative_role_impacts::Column::InitiativePid.eq(initiative.pid))
        .all(&ctx.db)
        .await?;

    let mut affected: BTreeSet<Uuid> = BTreeSet::new();
    let roles: Vec<serde_json::Value> = impacts
        .iter()
        .map(|i| {
            let title = titles.get(&i.role_profile_pid).cloned().unwrap_or_default();
            let holders: Vec<&workers::Model> = employed
                .iter()
                .filter(|w| same_title(&w.job_title, &title))
                .collect();
            let planned = holders
                .iter()
                .filter(|w| with_reskill_plan.contains(&w.pid))
                .count();
            if rules::affects_current_holders(&i.impact) {
                affected.extend(holders.iter().map(|w| w.pid));
            }
            serde_json::json!({
                "job_title": title,
                "impact": i.impact,
                "timeframe": i.timeframe,
                "employed_workers": holders.len(),
                "with_active_reskill_plan": planned,
                "reskill_plan_coverage": if rules::affects_current_holders(&i.impact) {
                    ratio_json(ratio(planned, holders.len()))
                } else {
                    serde_json::Value::Null
                },
            })
        })
        .collect();

    let shifts = initiative_skill_shifts::Entity::find()
        .filter(initiative_skill_shifts::Column::InitiativePid.eq(initiative.pid))
        .all(&ctx.db)
        .await?;
    let names: BTreeMap<Uuid, String> = skills::Entity::find()
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|s| (s.pid, s.name))
        .collect();
    let declarations = worker_skills::Entity::find()
        .filter(worker_skills::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?;
    let declared: BTreeMap<(Uuid, Uuid), i32> = declarations
        .iter()
        .map(|d| ((d.worker_pid, d.skill_pid), d.proficiency))
        .collect();
    let rising: Vec<serde_json::Value> = shifts
        .iter()
        .filter(|s| s.direction == "rising")
        .map(|s| {
            let levels: Vec<Option<i32>> = affected
                .iter()
                .map(|w| declared.get(&(*w, s.skill_pid)).copied())
                .collect();
            let r = rules::skill_readiness(&levels, bar);
            serde_json::json!({
                "skill": names.get(&s.skill_pid),
                "meeting": r.meeting,
                "below": r.below,
                "undeclared": r.undeclared,
                "meeting_share": ratio_json(ratio(r.meeting, affected.len())),
            })
        })
        .collect();
    let declining: Vec<&String> = shifts
        .iter()
        .filter(|s| s.direction == "declining")
        .filter_map(|s| names.get(&s.skill_pid))
        .collect();
    let planned_total = affected
        .iter()
        .filter(|w| with_reskill_plan.contains(w))
        .count();

    format::json(serde_json::json!({
        "derivation": "counts EMPLOYED workers whose job title matches an affected role; \
                       `affected_workers` are those in displaced or reshaped roles. Skill \
                       readiness compares DECLARED proficiency with the bar (undeclared is \
                       unknown, not below). Aggregate only: no individual is named or ranked \
                       (WPM-D28).",
        "initiative": { "pid": initiative.pid, "name": initiative.name, "status": initiative.status },
        "bar": bar,
        "affected_workers": affected.len(),
        "with_active_reskill_plan": planned_total,
        "reskill_plan_coverage": ratio_json(ratio(planned_total, affected.len())),
        "roles": roles,
        "rising_skills": rising,
        "declining_skills": declining,
    }))
}

/// The change-tracker routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/change-initiatives", post(create_initiative))
        .add("/change-initiatives", get(list_initiatives))
        .add("/change-initiatives/{pid}", get(get_initiative))
        .add("/change-initiatives/{pid}/status", post(set_status))
        .add(
            "/change-initiatives/{pid}/role-impacts",
            put(set_role_impact),
        )
        .add(
            "/change-initiatives/{pid}/role-impacts/{role_profile_pid}",
            delete(remove_role_impact),
        )
        .add(
            "/change-initiatives/{pid}/skill-shifts",
            put(set_skill_shift),
        )
        .add(
            "/change-initiatives/{pid}/skill-shifts/{skill_pid}",
            delete(remove_skill_shift),
        )
        .add("/change-initiatives/{pid}/readiness", get(readiness))
}
