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
    capability_frameworks, role_profiles, role_skill_requirements, skills, worker_skills, workers,
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

/// Query for the profile list.
#[derive(Debug, Deserialize)]
struct ProfileListQuery {
    /// Only profiles imported from this framework (e.g. `uk-gdad-pcf`).
    framework: Option<String>,
}

/// `GET /api/role-profiles?framework=` — every live profile with its
/// requirement count and, for an imported one, where it sits in its
/// framework (profession → role → level).
#[debug_handler]
async fn list_profiles(
    State(ctx): State<AppContext>,
    axum::extract::Query(query): axum::extract::Query<ProfileListQuery>,
) -> Result<Response> {
    let mut select =
        role_profiles::Entity::find().filter(role_profiles::Column::DeletedAt.is_null());
    if let Some(framework) = &query.framework {
        select = select.filter(role_profiles::Column::FrameworkSlug.eq(framework));
    }
    let profiles = select
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
                "framework": p.framework_slug,
                "profession": p.profession,
                "role_name": p.role_name,
                "level_name": p.level_name,
                "level_order": p.level_order,
                "management_track": p.management_track,
            })
        })
        .collect();
    format::json(out)
}

/// `GET /api/capability-frameworks` — the frameworks role profiles were
/// imported from, with their **attribution**, licence, proficiency scale,
/// and how many profiles each supplied.
#[debug_handler]
async fn list_frameworks(State(ctx): State<AppContext>) -> Result<Response> {
    let frameworks = capability_frameworks::Entity::find()
        .order_by_asc(capability_frameworks::Column::Name)
        .all(&ctx.db)
        .await?;
    let profiles = role_profiles::Entity::find()
        .filter(role_profiles::Column::DeletedAt.is_null())
        .filter(role_profiles::Column::FrameworkSlug.is_not_null())
        .all(&ctx.db)
        .await?;
    let out: Vec<serde_json::Value> = frameworks
        .iter()
        .map(|f| {
            let mine: Vec<&role_profiles::Model> = profiles
                .iter()
                .filter(|p| p.framework_slug.as_deref() == Some(f.slug.as_str()))
                .collect();
            let roles: std::collections::BTreeSet<(&str, &str)> = mine
                .iter()
                .filter_map(|p| Some((p.profession.as_deref()?, p.role_name.as_deref()?)))
                .collect();
            serde_json::json!({
                "slug": f.slug,
                "name": f.name,
                "source_url": f.source_url,
                "licence": f.licence,
                "attribution": f.attribution,
                "scale_max": f.scale_max,
                "scale_labels": f.scale_labels,
                "imported_on": f.imported_on,
                "note": f.note,
                "profiles": mine.len(),
                "roles": roles.len(),
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
                "source_level": r.source_level,
                "source_scale_max": r.source_scale_max,
            })
        })
        .collect();
    let framework = match &profile.framework_slug {
        Some(slug) => {
            capability_frameworks::Entity::find()
                .filter(capability_frameworks::Column::Slug.eq(slug))
                .one(&ctx.db)
                .await?
        }
        None => None,
    };
    format::json(serde_json::json!({
        "pid": profile.pid,
        "job_title": profile.job_title,
        "description": profile.description,
        "source_ref": profile.source_ref,
        "framework": framework.map(|f| serde_json::json!({
            "slug": f.slug, "name": f.name, "licence": f.licence,
            "attribution": f.attribution, "scale_max": f.scale_max,
            "scale_labels": f.scale_labels, "note": f.note,
        })),
        "profession": profile.profession,
        "role_name": profile.role_name,
        "level_name": profile.level_name,
        "level_order": profile.level_order,
        "management_track": profile.management_track,
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

/// The `(skill name, minimum)` requirements of a profile.
fn requirement_pairs(
    requirements: &[role_skill_requirements::Model],
    profile_pid: Uuid,
    names: &BTreeMap<Uuid, String>,
) -> Vec<(String, i32)> {
    requirements
        .iter()
        .filter(|r| r.role_profile_pid == profile_pid)
        .filter_map(|r| Some((names.get(&r.skill_pid)?.clone(), r.min_proficiency)))
        .collect()
}

/// `GET /api/role-profiles/{pid}/progression` — what changes going up a
/// level in the same role of the same framework: skills newly required,
/// skills required at a higher level, and skills unchanged. The next level
/// can be more than one profile (a technical and a management track). Empty
/// `next` at the top of a role, or for a profile not from a framework.
#[debug_handler]
async fn progression(State(ctx): State<AppContext>, Path(pid): Path<String>) -> Result<Response> {
    let profile = find_profile(&ctx, &pid).await?;
    let (Some(framework), Some(profession), Some(role), Some(order)) = (
        profile.framework_slug.clone(),
        profile.profession.clone(),
        profile.role_name.clone(),
        profile.level_order,
    ) else {
        return format::json(serde_json::json!({ "profile": profile.job_title, "next": [] }));
    };
    let siblings = role_profiles::Entity::find()
        .filter(role_profiles::Column::DeletedAt.is_null())
        .filter(role_profiles::Column::FrameworkSlug.eq(&framework))
        .filter(role_profiles::Column::Profession.eq(&profession))
        .filter(role_profiles::Column::RoleName.eq(&role))
        .all(&ctx.db)
        .await?;
    let next_order = siblings
        .iter()
        .filter_map(|p| p.level_order)
        .filter(|o| *o > order)
        .min();
    let requirements = role_skill_requirements::Entity::find().all(&ctx.db).await?;
    let names: BTreeMap<Uuid, String> = skills::Entity::find()
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|s| (s.pid, s.name))
        .collect();
    let current = requirement_pairs(&requirements, profile.pid, &names);
    let next: Vec<serde_json::Value> = siblings
        .iter()
        .filter(|p| next_order.is_some() && p.level_order == next_order)
        .map(|p| {
            let diff = rules::progression_diff(&current, &requirement_pairs(&requirements, p.pid, &names));
            serde_json::json!({
                "pid": p.pid,
                "job_title": p.job_title,
                "level_order": p.level_order,
                "management_track": p.management_track,
                "added": diff.added.iter().map(|(s, m)| serde_json::json!({ "skill": s, "min_proficiency": m })).collect::<Vec<_>>(),
                "raised": diff.raised.iter().map(|(s, from, to)| serde_json::json!({ "skill": s, "from": from, "to": to })).collect::<Vec<_>>(),
                "unchanged": diff.unchanged,
                "dropped": diff.dropped,
            })
        })
        .collect();
    format::json(serde_json::json!({
        "derivation": "compares required skills and minimum proficiency between this level and \
                       the next level of the same role in the same framework",
        "profile": { "pid": profile.pid, "job_title": profile.job_title, "level_order": order },
        "next": next,
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
        .add("/capability-frameworks", get(list_frameworks))
        .add("/role-profiles/{pid}/progression", get(progression))
        .add("/role-profiles/{pid}/gap", get(role_gap))
        .add("/workers/{pid}/role-gap", get(worker_role_gap))
        .add("/role-profiles/{pid}/requirements", put(set_requirement))
        .add(
            "/role-profiles/{pid}/requirements/{skill_pid}",
            delete(remove_requirement),
        )
}
