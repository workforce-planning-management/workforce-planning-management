//! **Role profiles** (WPM-R34) — what a role (keyed by job title)
//! requires: a set of catalogue skills, each with a minimum proficiency
//! (1–5) and an importance. The pure rules live in [`crate::rules::roles`].
//!
//! A profile describes a *role*, never a person; it is the yardstick that
//! competency-gap analysis measures declared proficiency against.

use loco_rs::prelude::*;
use sea_orm::{ActiveValue, QueryOrder};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

use super::{ensure_valid, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::models::_entities::{
    role_profiles, role_skill_requirements, skills, worker_skills, workers,
};
use crate::models::audit_logs::Model as Audit;
use crate::models::records;
use crate::rules::gap as gap_rules;
use crate::rules::metrics::is_employed_on;
use crate::rules::roles as rules;
use crate::rules::talent::ratio;
use crate::validation::Problems;

/// A `{pid}` reference response.
#[derive(Serialize)]
struct PidRef {
    pid: String,
}

/// `POST /api/role-profiles` body.
#[derive(Debug, Deserialize)]
struct ProfilePayload {
    job_title: String,
    #[serde(default)]
    description: Option<String>,
    /// Where the profile came from (e.g. a framework role-level slug), for
    /// provenance; free text.
    #[serde(default)]
    source_ref: Option<String>,
}

/// `PUT /api/role-profiles/{pid}/requirements` body.
#[derive(Debug, Deserialize)]
struct RequirementPayload {
    skill_pid: Uuid,
    min_proficiency: i32,
    importance: String,
    #[serde(default)]
    note: Option<String>,
}

/// `POST /api/role-profiles` — create a profile for a job title.
#[debug_handler]
async fn create_profile(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<ProfilePayload>,
) -> Result<Response> {
    let title = rules::normalize_job_title(&payload.job_title).map_err(|e| unprocessable(&e))?;
    let mut problems = Problems::new();
    if let Some(text) = &payload.description {
        problems.cap_text("description", text);
    }
    if let Some(text) = &payload.source_ref {
        problems.cap_text("source_ref", text);
    }
    ensure_valid(&problems.into_vec())?;
    let taken = role_profiles::Entity::find()
        .filter(role_profiles::Column::DeletedAt.is_null())
        .filter(role_profiles::Column::JobTitle.eq(&title))
        .one(&ctx.db)
        .await?
        .is_some();
    if taken {
        return Err(unprocessable(
            "a role profile for this job title already exists",
        ));
    }
    let row = role_profiles::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        job_title: ActiveValue::set(title),
        description: ActiveValue::set(payload.description.clone()),
        source_ref: ActiveValue::set(payload.source_ref.clone()),
        deleted_at: ActiveValue::set(None),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;
    Audit::record(
        &ctx.db,
        "role_profile",
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

/// `GET /api/role-profiles` — every live profile with its requirement count.
#[debug_handler]
async fn list_profiles(State(ctx): State<AppContext>) -> Result<Response> {
    let profiles = role_profiles::Entity::find()
        .filter(role_profiles::Column::DeletedAt.is_null())
        .order_by_asc(role_profiles::Column::JobTitle)
        .all(&ctx.db)
        .await?;
    let requirements = role_skill_requirements::Entity::find().all(&ctx.db).await?;
    let mut counts: BTreeMap<Uuid, usize> = BTreeMap::new();
    for requirement in &requirements {
        *counts.entry(requirement.role_profile_pid).or_default() += 1;
    }
    let out: Vec<serde_json::Value> = profiles
        .iter()
        .map(|p| {
            serde_json::json!({
                "pid": p.pid,
                "job_title": p.job_title,
                "description": p.description,
                "source_ref": p.source_ref,
                "requirement_count": counts.get(&p.pid).copied().unwrap_or(0),
            })
        })
        .collect();
    format::json(out)
}

/// `GET /api/role-profiles/{pid}` — a profile with its requirements,
/// critical first.
#[debug_handler]
async fn get_profile(State(ctx): State<AppContext>, Path(pid): Path<String>) -> Result<Response> {
    let profile = find_profile(&ctx, &pid).await?;
    let mut requirements = role_skill_requirements::Entity::find()
        .filter(role_skill_requirements::Column::RoleProfilePid.eq(profile.pid))
        .all(&ctx.db)
        .await?;
    let skill_rows = skills::Entity::find().all(&ctx.db).await?;
    let skill_of: BTreeMap<Uuid, &skills::Model> = skill_rows.iter().map(|s| (s.pid, s)).collect();
    let rank = |importance: &str| {
        rules::IMPORTANCES
            .iter()
            .position(|i| *i == importance)
            .unwrap_or(rules::IMPORTANCES.len())
    };
    requirements.sort_by_key(|r| {
        (
            rank(&r.importance),
            skill_of.get(&r.skill_pid).map(|s| s.name.clone()),
        )
    });
    let out: Vec<serde_json::Value> = requirements
        .iter()
        .map(|r| {
            serde_json::json!({
                "skill_pid": r.skill_pid,
                "skill": skill_of.get(&r.skill_pid).map(|s| s.name.clone()),
                "category": skill_of.get(&r.skill_pid).map(|s| s.category.clone()),
                "min_proficiency": r.min_proficiency,
                "importance": r.importance,
                "note": r.note,
            })
        })
        .collect();
    format::json(serde_json::json!({
        "pid": profile.pid,
        "job_title": profile.job_title,
        "description": profile.description,
        "source_ref": profile.source_ref,
        "requirements": out,
    }))
}

/// `PUT /api/role-profiles/{pid}/requirements` — require a skill at a
/// minimum proficiency (upsert: one row per profile + skill).
#[debug_handler]
async fn set_requirement(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<RequirementPayload>,
) -> Result<Response> {
    rules::validate_requirement(payload.min_proficiency, &payload.importance)
        .map_err(|e| unprocessable(&e))?;
    let mut problems = Problems::new();
    if let Some(text) = &payload.note {
        problems.cap_text("note", text);
    }
    ensure_valid(&problems.into_vec())?;
    let profile = find_profile(&ctx, &pid).await?;
    let skill = skills::Entity::find()
        .filter(skills::Column::Pid.eq(payload.skill_pid))
        .filter(skills::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    let existing = role_skill_requirements::Entity::find()
        .filter(role_skill_requirements::Column::RoleProfilePid.eq(profile.pid))
        .filter(role_skill_requirements::Column::SkillPid.eq(skill.pid))
        .one(&ctx.db)
        .await?;
    match existing {
        Some(row) => {
            let mut active: role_skill_requirements::ActiveModel = row.into();
            active.min_proficiency = ActiveValue::set(payload.min_proficiency);
            active.importance = ActiveValue::set(payload.importance.clone());
            active.note = ActiveValue::set(payload.note.clone());
            active.update(&ctx.db).await?;
        }
        None => {
            role_skill_requirements::ActiveModel {
                pid: ActiveValue::set(Uuid::new_v4()),
                role_profile_pid: ActiveValue::set(profile.pid),
                skill_pid: ActiveValue::set(skill.pid),
                min_proficiency: ActiveValue::set(payload.min_proficiency),
                importance: ActiveValue::set(payload.importance.clone()),
                note: ActiveValue::set(payload.note.clone()),
                ..Default::default()
            }
            .insert(&ctx.db)
            .await?;
        }
    }
    Audit::record(
        &ctx.db,
        "role_profile",
        profile.pid,
        "requirement_set",
        caller.actor(),
        Some(serde_json::json!({
            "skill": skill.name,
            "min_proficiency": payload.min_proficiency,
            "importance": payload.importance,
        })),
    )
    .await?;
    format::empty_json()
}

/// `DELETE /api/role-profiles/{pid}/requirements/{skill_pid}` — drop a
/// requirement.
#[debug_handler]
async fn remove_requirement(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path((pid, skill_pid)): Path<(String, String)>,
) -> Result<Response> {
    let profile = find_profile(&ctx, &pid).await?;
    let skill_pid = records::parse_pid(&skill_pid)?;
    let row = role_skill_requirements::Entity::find()
        .filter(role_skill_requirements::Column::RoleProfilePid.eq(profile.pid))
        .filter(role_skill_requirements::Column::SkillPid.eq(skill_pid))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    row.delete(&ctx.db).await?;
    Audit::record(
        &ctx.db,
        "role_profile",
        profile.pid,
        "requirement_removed",
        caller.actor(),
        Some(serde_json::json!({ "skill_pid": skill_pid })),
    )
    .await?;
    format::empty_json()
}

/// Render a `(numerator, denominator, value)` triple as a terms-carrying
/// ratio object, or `null` when there is nothing to divide.
fn ratio_json(terms: Option<(usize, usize, f64)>) -> serde_json::Value {
    terms.map_or(serde_json::Value::Null, |(numerator, denominator, value)| {
        serde_json::json!({ "numerator": numerator, "denominator": denominator, "value": value })
    })
}

/// Query for the worker-vs-role gap.
#[derive(Debug, Deserialize)]
struct RoleGapQuery {
    role_profile_pid: Uuid,
}

/// `GET /api/workers/{pid}/role-gap?role_profile_pid=` — one worker's
/// declared proficiency against every requirement of a role profile:
/// `met` / `below` / `undeclared`, the shortfall where it is known, and
/// readiness (critical and overall, as terms-carrying ratios).
///
/// Compares **declarations** — see [`crate::rules::gap`]. The result is a
/// development conversation starter, never a selection decision.
#[debug_handler]
async fn worker_role_gap(
    State(ctx): State<AppContext>,
    Path(pid): Path<String>,
    axum::extract::Query(query): axum::extract::Query<RoleGapQuery>,
) -> Result<Response> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    let profile = find_profile(&ctx, &query.role_profile_pid.to_string()).await?;
    let requirements = role_skill_requirements::Entity::find()
        .filter(role_skill_requirements::Column::RoleProfilePid.eq(profile.pid))
        .all(&ctx.db)
        .await?;
    let declared: BTreeMap<Uuid, i32> = worker_skills::Entity::find()
        .filter(worker_skills::Column::WorkerPid.eq(worker.pid))
        .filter(worker_skills::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|d| (d.skill_pid, d.proficiency))
        .collect();
    let names: BTreeMap<Uuid, String> = skills::Entity::find()
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|s| (s.pid, s.name))
        .collect();

    let mut rows = Vec::with_capacity(requirements.len());
    let mut graded: Vec<(String, &'static str)> = Vec::with_capacity(requirements.len());
    for r in &requirements {
        let have = declared.get(&r.skill_pid).copied();
        let grade = gap_rules::grade(have, r.min_proficiency);
        graded.push((r.importance.clone(), grade));
        rows.push(serde_json::json!({
            "skill_pid": r.skill_pid,
            "skill": names.get(&r.skill_pid),
            "importance": r.importance,
            "min_proficiency": r.min_proficiency,
            "declared": have,
            "grade": grade,
            "shortfall": gap_rules::shortfall(have, r.min_proficiency),
        }));
    }
    let pairs: Vec<(&str, &str)> = graded.iter().map(|(i, g)| (i.as_str(), *g)).collect();
    let readiness = gap_rules::readiness(&pairs);
    format::json(serde_json::json!({
        "derivation": "compares the worker's DECLARED proficiency with the role profile's \
                       minimum; `undeclared` means no declaration (unknown, not below) and has \
                       no shortfall figure. A development conversation starter, not a \
                       selection decision.",
        "worker_pid": worker.pid,
        "role": { "pid": profile.pid, "job_title": profile.job_title },
        "requirements": rows,
        "critical_met": ratio_json(ratio(readiness.critical_met, readiness.critical_total)),
        "all_met": ratio_json(ratio(readiness.met, readiness.total)),
    }))
}

/// `GET /api/role-profiles/{pid}/gap` — can the current workforce staff
/// this role? Per requirement: how many **employed** workers meet it, are
/// below it, or have not declared the skill, with the coverage ratio over
/// headcount. Aggregate only — no individual is named.
#[debug_handler]
async fn role_gap(State(ctx): State<AppContext>, Path(pid): Path<String>) -> Result<Response> {
    let profile = find_profile(&ctx, &pid).await?;
    let today = chrono::Utc::now().date_naive();
    let employed: Vec<Uuid> = workers::Entity::find()
        .filter(workers::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?
        .into_iter()
        .filter(|w| is_employed_on(today, w.hired_on, w.terminated_on))
        .map(|w| w.pid)
        .collect();
    let headcount = employed.len();
    let requirements = role_skill_requirements::Entity::find()
        .filter(role_skill_requirements::Column::RoleProfilePid.eq(profile.pid))
        .all(&ctx.db)
        .await?;
    let declarations = worker_skills::Entity::find()
        .filter(worker_skills::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?;
    let names: BTreeMap<Uuid, String> = skills::Entity::find()
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|s| (s.pid, s.name))
        .collect();
    // (worker, skill) → proficiency, employed workers only.
    let employed_set: std::collections::HashSet<Uuid> = employed.into_iter().collect();
    let have: BTreeMap<(Uuid, Uuid), i32> = declarations
        .iter()
        .filter(|d| employed_set.contains(&d.worker_pid))
        .map(|d| ((d.worker_pid, d.skill_pid), d.proficiency))
        .collect();

    let rows: Vec<serde_json::Value> = requirements
        .iter()
        .map(|r| {
            let (mut met, mut below, mut declared_count) = (0usize, 0usize, 0usize);
            for worker in &employed_set {
                match gap_rules::grade(
                    have.get(&(*worker, r.skill_pid)).copied(),
                    r.min_proficiency,
                ) {
                    "met" => {
                        met += 1;
                        declared_count += 1;
                    }
                    "below" => {
                        below += 1;
                        declared_count += 1;
                    }
                    _ => {}
                }
            }
            serde_json::json!({
                "skill": names.get(&r.skill_pid),
                "importance": r.importance,
                "min_proficiency": r.min_proficiency,
                "meeting": met,
                "below": below,
                "undeclared": headcount - declared_count,
                "coverage": ratio_json(ratio(met, headcount)),
            })
        })
        .collect();
    format::json(serde_json::json!({
        "derivation": "counts EMPLOYED workers (the shared employed-on-date definition) whose \
                       DECLARED proficiency meets the profile's minimum; undeclared is unknown, \
                       not below. Aggregate: no individual is named.",
        "role": { "pid": profile.pid, "job_title": profile.job_title },
        "headcount": headcount,
        "requirements": rows,
    }))
}

/// Find a live profile by pid string.
async fn find_profile(ctx: &AppContext, pid: &str) -> Result<role_profiles::Model> {
    role_profiles::Entity::find()
        .filter(role_profiles::Column::Pid.eq(records::parse_pid(pid)?))
        .filter(role_profiles::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)
}

/// The role-profile routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/role-profiles", post(create_profile))
        .add("/role-profiles", get(list_profiles))
        .add("/role-profiles/{pid}", get(get_profile))
        .add("/role-profiles/{pid}/gap", get(role_gap))
        .add("/workers/{pid}/role-gap", get(worker_role_gap))
        .add("/role-profiles/{pid}/requirements", put(set_requirement))
        .add(
            "/role-profiles/{pid}/requirements/{skill_pid}",
            delete(remove_requirement),
        )
}
