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
use crate::models::_entities::{role_profiles, role_skill_requirements, skills};
use crate::models::audit_logs::Model as Audit;
use crate::models::records;
use crate::rules::roles as rules;
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
        .add("/role-profiles/{pid}/requirements", put(set_requirement))
        .add(
            "/role-profiles/{pid}/requirements/{skill_pid}",
            delete(remove_requirement),
        )
}
