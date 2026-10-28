//! **SFIA** in role profiles and declared skills (WPM-R111, WPM-D70): search the
//! deployer's imported catalogue, see a skill's levels, start a role from SFIA
//! skills, and the helpers that keep a framework's **own level** next to WPM's
//! 1–5 figure. Nothing here ships SFIA wording: it is read from the tables
//! `import_sfia` filled, and only for a signed-in caller, so it does not leave
//! the organization (WPM-R113).

use loco_rs::prelude::*;
use sea_orm::{ActiveValue, ConnectionTrait, TransactionTrait};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

use super::{ensure_valid, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::models::_entities::{
    capability_frameworks, role_profiles, role_skill_requirements, sfia_skill_levels, sfia_skills,
    sfiaplus_resources, skill_external_refs, skills,
};
use crate::models::audit_logs::Model as Audit;
use crate::rules::esco::normalise_label;
use crate::rules::sfia as rules;
use crate::tasks::import_sfia::SFIA_SLUG;
use crate::validation::Problems;

/// A level stated on a framework's own scale, with WPM's 1–5 figure for it.
#[derive(Debug, Clone)]
pub(crate) struct Native {
    pub framework: String,
    pub level: i32,
    pub scale_max: i32,
    pub wpm: i32,
}

/// Resolve an optional `(framework, level)` pair a client sent.
///
/// # Errors
/// 422 when only one of the two is given, the framework is not recorded, or the level is outside its scale.
pub(crate) async fn resolve_native(
    db: &impl ConnectionTrait,
    framework: Option<&str>,
    level: Option<i32>,
) -> Result<Option<Native>> {
    let (framework, level) = match (framework.map(str::trim), level) {
        (None | Some(""), None) => return Ok(None),
        (Some(f), Some(l)) if !f.is_empty() => (f, l),
        _ => return Err(unprocessable("give framework and framework_level together")),
    };
    let row = capability_frameworks::Entity::find()
        .filter(capability_frameworks::Column::Slug.eq(framework))
        .one(db)
        .await?
        .ok_or_else(|| unprocessable(&format!("unknown framework {framework:?}")))?;
    let wpm = rules::to_wpm(level, row.scale_max).map_err(|e| unprocessable(&e))?;
    Ok(Some(Native {
        framework: row.slug,
        level,
        scale_max: row.scale_max,
        wpm,
    }))
}

/// The SFIA code a catalogue skill is known by, if any.
async fn sfia_code(db: &impl ConnectionTrait, skill_pid: Uuid) -> Result<Option<String>> {
    Ok(skill_external_refs::Entity::find()
        .filter(skill_external_refs::Column::SkillPid.eq(skill_pid))
        .filter(skill_external_refs::Column::FrameworkSlug.eq(SFIA_SLUG))
        .one(db)
        .await?
        .map(|r| r.reference))
}

/// A skill known in SFIA may only be stated at a level SFIA defines for it (a level is never invented).
///
/// # Errors
/// 422 when the skill has a SFIA code and that code has no current definition at this level.
pub(crate) async fn check_defined(
    db: &impl ConnectionTrait,
    skill_pid: Uuid,
    native: &Native,
) -> Result<()> {
    if native.framework != SFIA_SLUG {
        return Ok(());
    }
    let Some(code) = sfia_code(db, skill_pid).await? else {
        return Ok(());
    };
    let defined = sfia_skill_levels::Entity::find()
        .filter(sfia_skill_levels::Column::Code.eq(&code))
        .filter(sfia_skill_levels::Column::Level.eq(native.level))
        .filter(sfia_skill_levels::Column::RetiredIn.is_null())
        .one(db)
        .await?
        .is_some();
    if defined {
        Ok(())
    } else {
        Err(unprocessable(&format!(
            "SFIA {code} is not defined at level {}",
            native.level
        )))
    }
}

/// Each recorded framework's display name by slug, so a level stated on one is shown by name, not by its slug.
///
/// # Errors
/// A database error.
pub(crate) async fn framework_names(db: &impl ConnectionTrait) -> Result<BTreeMap<String, String>> {
    Ok(capability_frameworks::Entity::find()
        .all(db)
        .await?
        .into_iter()
        .map(|f| (f.slug, f.name))
        .collect())
}

/// The framework's own wording for each `(skill, level)` stated on SFIA, so a native level can be shown with what it says.
///
/// # Errors
/// A database error.
pub(crate) async fn native_descriptions(
    db: &impl ConnectionTrait,
    stated: &[(Uuid, Option<&str>, Option<i32>)],
) -> Result<BTreeMap<Uuid, String>> {
    let wanted: Vec<(Uuid, i32)> = stated
        .iter()
        .filter_map(|(p, f, l)| (*f == Some(SFIA_SLUG)).then_some((*p, (*l)?)))
        .collect();
    if wanted.is_empty() {
        return Ok(BTreeMap::new());
    }
    let refs: BTreeMap<Uuid, String> = skill_external_refs::Entity::find()
        .filter(skill_external_refs::Column::FrameworkSlug.eq(SFIA_SLUG))
        .filter(skill_external_refs::Column::SkillPid.is_in(wanted.iter().map(|(p, _)| *p)))
        .all(db)
        .await?
        .into_iter()
        .map(|r| (r.skill_pid, r.reference))
        .collect();
    let levels: BTreeMap<(String, i32), String> = sfia_skill_levels::Entity::find()
        .filter(sfia_skill_levels::Column::Code.is_in(refs.values().cloned()))
        .all(db)
        .await?
        .into_iter()
        .map(|l| ((l.code, l.level), l.description))
        .collect();
    Ok(wanted
        .into_iter()
        .filter_map(|(p, l)| Some((p, levels.get(&(refs.get(&p)?.clone(), l))?.clone())))
        .collect())
}

/// `SFIAplus` development **pointers** for skills a person needs at a WPM 1–5 level: the resources attached to the skill's SFIA
/// code at the SFIA levels that count as that level. Each names its source and SFIA level; nothing is booked or bought from
/// one. Empty when `SFIAplus` was never imported or the skill has no SFIA code.
///
/// # Errors
/// A database error.
pub(crate) async fn pointers_for(
    db: &impl ConnectionTrait,
    wanted: &[(Uuid, i32)],
) -> Result<BTreeMap<Uuid, Vec<serde_json::Value>>> {
    let Some(source) = capability_frameworks::Entity::find()
        .filter(
            capability_frameworks::Column::Slug.eq(crate::tasks::import_sfiaplus::SFIAPLUS_SLUG),
        )
        .one(db)
        .await?
    else {
        return Ok(BTreeMap::new());
    };
    let refs: BTreeMap<Uuid, String> = skill_external_refs::Entity::find()
        .filter(skill_external_refs::Column::FrameworkSlug.eq(SFIA_SLUG))
        .filter(skill_external_refs::Column::SkillPid.is_in(wanted.iter().map(|(p, _)| *p)))
        .all(db)
        .await?
        .into_iter()
        .map(|r| (r.skill_pid, r.reference))
        .collect();
    if refs.is_empty() {
        return Ok(BTreeMap::new());
    }
    let resources = sfiaplus_resources::Entity::find()
        .filter(sfiaplus_resources::Column::Code.is_in(refs.values().cloned()))
        .filter(sfiaplus_resources::Column::RetiredIn.is_null())
        .all(db)
        .await?;
    let mut out: BTreeMap<Uuid, Vec<serde_json::Value>> = BTreeMap::new();
    for (skill, required) in wanted {
        let Some(code) = refs.get(skill) else {
            continue;
        };
        let mine: Vec<serde_json::Value> = resources
            .iter()
            .filter(|r| {
                &r.code == code && rules::to_wpm(r.level, rules::SFIA_SCALE_MAX) == Ok(*required)
            })
            .map(|r| {
                serde_json::json!({
                    "source": source.name, "sfia_code": r.code, "sfia_level": r.level,
                    "kind": r.kind, "title": r.title, "provider": r.provider, "url": r.url,
                })
            })
            .collect();
        if !mine.is_empty() {
            out.insert(*skill, mine);
        }
    }
    Ok(out)
}

#[derive(Debug, Deserialize)]
struct SearchQuery {
    q: Option<String>,
}

/// `GET /api/sfia/skills?q=` — search the imported catalogue by code or name (literal, case-insensitive, at least 2 characters).
#[debug_handler]
async fn search(
    State(ctx): State<AppContext>,
    axum::extract::Query(query): axum::extract::Query<SearchQuery>,
) -> Result<Response> {
    let q = query.q.unwrap_or_default().trim().to_lowercase();
    if q.chars().count() < 2 {
        return Err(unprocessable("q must be at least 2 characters"));
    }
    let levels = sfia_skill_levels::Entity::find()
        .filter(sfia_skill_levels::Column::RetiredIn.is_null())
        .all(&ctx.db)
        .await?;
    let out: Vec<serde_json::Value> = sfia_skills::Entity::find()
        .all(&ctx.db)
        .await?
        .into_iter()
        .filter(|s| s.code.to_lowercase().contains(&q) || s.name.to_lowercase().contains(&q))
        .take(50)
        .map(|s| {
            let mine: Vec<i32> = levels
                .iter()
                .filter(|l| l.code == s.code)
                .map(|l| l.level)
                .collect();
            serde_json::json!({
                "code": s.code, "name": s.name, "category": s.category,
                "subcategory": s.subcategory, "retired_in": s.retired_in, "levels": mine,
            })
        })
        .collect();
    format::json(out)
}

/// `GET /api/sfia/skills/{code}` — one skill with each level's wording and the mapping to WPM's 1–5.
#[debug_handler]
async fn show(State(ctx): State<AppContext>, Path(code): Path<String>) -> Result<Response> {
    let skill = sfia_skills::Entity::find()
        .filter(sfia_skills::Column::Code.eq(code.trim()))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    let levels = sfia_skill_levels::Entity::find()
        .filter(sfia_skill_levels::Column::Code.eq(&skill.code))
        .all(&ctx.db)
        .await?;
    let resources = sfiaplus_resources::Entity::find()
        .filter(sfiaplus_resources::Column::Code.eq(&skill.code))
        .filter(sfiaplus_resources::Column::RetiredIn.is_null())
        .all(&ctx.db)
        .await?;
    let mut levels: Vec<serde_json::Value> = levels
        .into_iter()
        .map(|l| {
            let pointers: Vec<serde_json::Value> = resources
                .iter()
                .filter(|r| r.level == l.level)
                .map(|r| serde_json::json!({ "kind": r.kind, "title": r.title, "provider": r.provider, "url": r.url }))
                .collect();
            serde_json::json!({
                "level": l.level, "description": l.description, "attributes": l.attributes,
                "retired_in": l.retired_in, "wpm_level": rules::to_wpm(l.level, rules::SFIA_SCALE_MAX).ok(),
                "sfiaplus": pointers,
            })
        })
        .collect();
    levels.sort_by_key(|l| l["level"].as_i64());
    format::json(serde_json::json!({
        "code": skill.code, "name": skill.name, "category": skill.category,
        "subcategory": skill.subcategory, "description": skill.description,
        "retired_in": skill.retired_in, "scale_max": rules::SFIA_SCALE_MAX,
        "mapping": rules::MAPPING_NAME, "levels": levels,
    }))
}

/// One skill a role is to be started from.
#[derive(Debug, Deserialize)]
struct SfiaSkillPick {
    code: String,
    level: i32,
}

/// `POST /api/role-profiles/from-sfia` body.
#[derive(Debug, Deserialize)]
struct FromSfiaPayload {
    job_title: String,
    skills: Vec<SfiaSkillPick>,
    /// `critical`, `important` or `useful`; default `important`.
    #[serde(default)]
    importance: Option<String>,
    /// The category given to a catalogue skill this creates; default `technical`. A planner can change it later.
    #[serde(default)]
    category: Option<String>,
}

/// `POST /api/role-profiles/from-sfia` — draft a role profile from SFIA skills the planner picked, each at a SFIA level that
/// SFIA defines for it. The level is kept on SFIA's scale and the 1–5 figure comes from the stated mapping. A draft for a
/// human to edit; catalogue skills are matched by their SFIA code, then by exact name, else created and linked.
#[debug_handler]
#[allow(clippy::too_many_lines)] // validate → profile → each skill, linearly
async fn from_sfia(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<FromSfiaPayload>,
) -> Result<Response> {
    let importance = payload.importance.as_deref().unwrap_or("important");
    let category = payload.category.as_deref().unwrap_or("technical");
    if !crate::rules::learning::SKILL_CATEGORIES.contains(&category) {
        return Err(unprocessable(&format!(
            "category must be one of {}",
            crate::rules::learning::SKILL_CATEGORIES.join(", ")
        )));
    }
    crate::rules::roles::validate_requirement(1, importance).map_err(|e| unprocessable(&e))?;
    let mut problems = Problems::new();
    problems.cap_text("job_title", &payload.job_title);
    if payload.skills.is_empty() || payload.skills.len() > 50 {
        problems.push("skills must name between 1 and 50 SFIA skills");
    }
    ensure_valid(&problems.into_vec())?;
    let title = crate::rules::roles::normalize_job_title(&payload.job_title)
        .map_err(|e| unprocessable(&e))?;
    let mut seen = BTreeSet::new();
    for pick in &payload.skills {
        if !seen.insert(pick.code.trim().to_string()) {
            return Err(unprocessable(&format!(
                "SFIA {} is named twice",
                pick.code.trim()
            )));
        }
    }
    let known: BTreeMap<String, sfia_skills::Model> = sfia_skills::Entity::find()
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|s| (s.code.clone(), s))
        .collect();
    let defined: BTreeSet<(String, i32)> = sfia_skill_levels::Entity::find()
        .filter(sfia_skill_levels::Column::RetiredIn.is_null())
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|l| (l.code, l.level))
        .collect();
    let mut chosen: Vec<(&sfia_skills::Model, i32, i32)> = Vec::new();
    for pick in &payload.skills {
        let code = pick.code.trim();
        let skill = known
            .get(code)
            .filter(|s| s.retired_in.is_none())
            .ok_or_else(|| {
                unprocessable(&format!(
                    "SFIA skill {code:?} is not in the imported catalogue"
                ))
            })?;
        if !defined.contains(&(skill.code.clone(), pick.level)) {
            return Err(unprocessable(&format!(
                "SFIA {code} is not defined at level {}",
                pick.level
            )));
        }
        let wpm =
            rules::to_wpm(pick.level, rules::SFIA_SCALE_MAX).map_err(|e| unprocessable(&e))?;
        chosen.push((skill, pick.level, wpm));
    }
    let taken = role_profiles::Entity::find()
        .filter(role_profiles::Column::DeletedAt.is_null())
        .filter(role_profiles::Column::JobTitle.eq(&title))
        .one(&ctx.db)
        .await?
        .is_some();
    if taken {
        return Err(unprocessable(
            "a role profile with that job title already exists",
        ));
    }

    let txn = ctx.db.begin().await?;
    let profile = role_profiles::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        job_title: ActiveValue::set(title),
        source_ref: ActiveValue::set(Some(format!("{SFIA_SLUG}:picked-skills"))),
        management_track: ActiveValue::set(false),
        deleted_at: ActiveValue::set(None),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    let (mut created, mut linked, mut requirements) = (0usize, 0usize, 0usize);
    let version = capability_frameworks::Entity::find()
        .filter(capability_frameworks::Column::Slug.eq(SFIA_SLUG))
        .one(&txn)
        .await?
        .and_then(|f| f.version);
    for (sfia, level, wpm) in chosen {
        let by_ref = skill_external_refs::Entity::find()
            .filter(skill_external_refs::Column::FrameworkSlug.eq(SFIA_SLUG))
            .filter(skill_external_refs::Column::Reference.eq(&sfia.code))
            .one(&txn)
            .await?;
        let skill_pid = if let Some(r) = by_ref {
            r.skill_pid
        } else {
            let wanted = normalise_label(&sfia.name);
            let same_name: Vec<skills::Model> = skills::Entity::find()
                .filter(skills::Column::DeletedAt.is_null())
                .all(&txn)
                .await?
                .into_iter()
                .filter(|s| normalise_label(&s.name) == wanted)
                .collect();
            let reused = match same_name.as_slice() {
                [one] => {
                    let has_ref = skill_external_refs::Entity::find()
                        .filter(skill_external_refs::Column::SkillPid.eq(one.pid))
                        .filter(skill_external_refs::Column::FrameworkSlug.eq(SFIA_SLUG))
                        .one(&txn)
                        .await?
                        .is_some();
                    (!has_ref).then_some(one.pid)
                }
                _ => None,
            };
            let pid = if let Some(pid) = reused {
                linked += 1;
                pid
            } else {
                let taken_name = skills::Entity::find()
                    .filter(skills::Column::Name.eq(&sfia.name))
                    .one(&txn)
                    .await?
                    .is_some();
                let name = if taken_name {
                    format!("{} (SFIA)", sfia.name)
                } else {
                    sfia.name.clone()
                };
                created += 1;
                skills::ActiveModel {
                    pid: ActiveValue::set(Uuid::new_v4()),
                    name: ActiveValue::set(name),
                    category: ActiveValue::set(category.to_string()),
                    deleted_at: ActiveValue::set(None),
                    ..Default::default()
                }
                .insert(&txn)
                .await?
                .pid
            };
            skill_external_refs::ActiveModel {
                pid: ActiveValue::set(Uuid::new_v4()),
                skill_pid: ActiveValue::set(pid),
                framework_slug: ActiveValue::set(SFIA_SLUG.to_string()),
                reference: ActiveValue::set(sfia.code.clone()),
                label: ActiveValue::set(Some(sfia.name.clone())),
                version: ActiveValue::set(version.clone()),
                ..Default::default()
            }
            .insert(&txn)
            .await?;
            pid
        };
        let exists = role_skill_requirements::Entity::find()
            .filter(role_skill_requirements::Column::RoleProfilePid.eq(profile.pid))
            .filter(role_skill_requirements::Column::SkillPid.eq(skill_pid))
            .one(&txn)
            .await?
            .is_some();
        if exists {
            continue;
        }
        role_skill_requirements::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            role_profile_pid: ActiveValue::set(profile.pid),
            skill_pid: ActiveValue::set(skill_pid),
            min_proficiency: ActiveValue::set(wpm),
            importance: ActiveValue::set(importance.to_string()),
            source_framework: ActiveValue::set(Some(SFIA_SLUG.to_string())),
            source_level: ActiveValue::set(Some(level)),
            source_scale_max: ActiveValue::set(Some(rules::SFIA_SCALE_MAX)),
            ..Default::default()
        }
        .insert(&txn)
        .await?;
        requirements += 1;
    }
    Audit::record(
        &txn,
        "role_profile",
        profile.pid,
        "seeded_from_sfia",
        caller.actor(),
        Some(serde_json::json!({ "requirements": requirements })),
    )
    .await?;
    txn.commit().await?;
    format::json(serde_json::json!({
        "pid": profile.pid,
        "requirements_created": requirements,
        "skills_created": created,
        "skills_linked": linked,
    }))
}

/// The SFIA routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/sfia/skills", get(search))
        .add("/sfia/skills/{code}", get(show))
        .add("/role-profiles/from-sfia", post(from_sfia))
}
