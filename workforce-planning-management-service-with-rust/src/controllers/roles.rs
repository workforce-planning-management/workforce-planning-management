//! **Role profiles** (WPM-R34) — what a role (keyed by job title)
//! requires: a set of catalogue skills, each with a minimum proficiency
//! (1–5) and an importance. The pure rules live in [`crate::rules::roles`].
//!
//! A profile describes a *role*, never a person; it is the yardstick that
//! competency-gap analysis measures declared proficiency against.

use super::ratio_json;
use axa::extract::{Json, Path};
use axa::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

use super::sfia::{check_defined, framework_names, native_descriptions, resolve_native};
use super::{ensure_valid, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::db;
use crate::models::_entities::{capability_frameworks, role_profiles, role_skill_requirements};
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
    /// WPM's 1–5. May be left out when a framework level is given: it is then worked out by the stated mapping.
    #[serde(default)]
    min_proficiency: Option<i32>,
    importance: String,
    #[serde(default)]
    note: Option<String>,
    /// The framework the level was stated on (e.g. `sfia`), with `framework_level` on its own scale.
    #[serde(default)]
    framework: Option<String>,
    #[serde(default)]
    framework_level: Option<i32>,
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
    let taken = sqlx::query_scalar!(
        r#"SELECT 1 AS "one!" FROM role_profiles WHERE deleted_at IS NULL AND job_title = $1 LIMIT 1"#,
        title,
    )
    .fetch_optional(db::pool(&ctx))
    .await?
    .is_some();
    if taken {
        return Err(unprocessable(
            "a role profile for this job title already exists",
        ));
    }
    let pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO role_profiles (pid, job_title, description, source_ref) VALUES ($1, $2, $3, $4)",
        pid,
        title,
        payload.description,
        payload.source_ref,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "role_profile",
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
    axa::extract::Query(query): axa::extract::Query<ProfileListQuery>,
) -> Result<Response> {
    let profiles = sqlx::query!(
        r#"SELECT p.pid, p.job_title, p.description, p.source_ref, p.framework_slug, p.profession, p.role_name,
                  p.level_name, p.level_order, p.management_track,
                  (SELECT COUNT(*) FROM role_skill_requirements r WHERE r.role_profile_pid = p.pid) AS "requirement_count!"
             FROM role_profiles p
            WHERE p.deleted_at IS NULL AND ($1::text IS NULL OR p.framework_slug = $1)
            ORDER BY p.job_title, p.id"#,
        query.framework,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let out: Vec<serde_json::Value> = profiles
        .iter()
        .map(|p| {
            serde_json::json!({
                "pid": p.pid,
                "job_title": p.job_title,
                "description": p.description,
                "source_ref": p.source_ref,
                "requirement_count": p.requirement_count,
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
    let frameworks = sqlx::query_as!(
        capability_frameworks::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, slug, name, source_url, licence,
                  attribution, scale_max, scale_labels, imported_on, note, version, redistribution, licence_holder,
                  licence_reference, licence_checked_on
             FROM capability_frameworks ORDER BY name, id"#,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let profiles = sqlx::query!(
        "SELECT framework_slug, profession, role_name FROM role_profiles
          WHERE deleted_at IS NULL AND framework_slug IS NOT NULL"
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let out: Vec<serde_json::Value> = frameworks
        .iter()
        .map(|f| {
            let mine: Vec<_> = profiles
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
                "version": f.version,
                "redistribution": f.redistribution,
                "licence_holder": f.licence_holder,
                "licence_checked_on": f.licence_checked_on,
                "profiles": mine.len(),
                "roles": roles.len(),
            })
        })
        .collect();
    format::json(out)
}

/// A profile's requirements, in the order they were recorded.
async fn requirements_of(
    ctx: &AppContext,
    profile_pid: Uuid,
) -> Result<Vec<role_skill_requirements::Model>> {
    Ok(sqlx::query_as!(
        role_skill_requirements::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, role_profile_pid, skill_pid,
                  min_proficiency, importance, note, source_framework, source_level, source_scale_max
             FROM role_skill_requirements WHERE role_profile_pid = $1 ORDER BY id"#,
        profile_pid,
    )
    .fetch_all(db::pool(ctx))
    .await?)
}

/// Every skill's name, retired ones too: a requirement or a declaration may still name one.
async fn skill_names(ctx: &AppContext) -> Result<BTreeMap<Uuid, String>> {
    Ok(sqlx::query!("SELECT pid, name FROM skills")
        .fetch_all(db::pool(ctx))
        .await?
        .into_iter()
        .map(|s| (s.pid, s.name))
        .collect())
}

/// `GET /api/role-profiles/{pid}` — a profile with its requirements,
/// critical first.
#[debug_handler]
async fn get_profile(State(ctx): State<AppContext>, Path(pid): Path<String>) -> Result<Response> {
    let profile = find_profile(&ctx, &pid).await?;
    let mut requirements = requirements_of(&ctx, profile.pid).await?;
    let skill_of: BTreeMap<Uuid, (String, String)> =
        sqlx::query!("SELECT pid, name, category FROM skills")
            .fetch_all(db::pool(&ctx))
            .await?
            .into_iter()
            .map(|s| (s.pid, (s.name, s.category)))
            .collect();
    let rank = |importance: &str| {
        rules::IMPORTANCES
            .iter()
            .position(|i| *i == importance)
            .unwrap_or(rules::IMPORTANCES.len())
    };
    requirements.sort_by_key(|r| {
        (
            rank(&r.importance),
            skill_of.get(&r.skill_pid).map(|s| s.0.clone()),
        )
    });
    let stated: Vec<_> = requirements
        .iter()
        .map(|r| (r.skill_pid, r.source_framework.as_deref(), r.source_level))
        .collect();
    let described = native_descriptions(&ctx, &stated).await?;
    let framework_names = framework_names(&ctx).await?;
    let out: Vec<serde_json::Value> = requirements
        .iter()
        .map(|r| {
            serde_json::json!({
                "skill_pid": r.skill_pid,
                "skill": skill_of.get(&r.skill_pid).map(|s| s.0.clone()),
                "category": skill_of.get(&r.skill_pid).map(|s| s.1.clone()),
                "min_proficiency": r.min_proficiency,
                "importance": r.importance,
                "note": r.note,
                "source_framework": r.source_framework,
                "source_framework_name": r.source_framework.as_ref().and_then(|f| framework_names.get(f)),
                "source_level": r.source_level,
                "source_scale_max": r.source_scale_max,
                "mapping": r.source_level.map(|_| crate::rules::sfia::MAPPING_NAME),
                "native_description": described.get(&r.skill_pid),
            })
        })
        .collect();
    let framework = match &profile.framework_slug {
        Some(slug) => {
            sqlx::query!(
                "SELECT slug, name, licence, attribution, scale_max, scale_labels, note
                   FROM capability_frameworks WHERE slug = $1",
                slug,
            )
            .fetch_optional(db::pool(&ctx))
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
    let native =
        resolve_native(&ctx, payload.framework.as_deref(), payload.framework_level).await?;
    let min_proficiency = payload
        .min_proficiency
        .or_else(|| native.as_ref().map(|n| n.wpm))
        .ok_or_else(|| unprocessable("give min_proficiency, or framework and framework_level"))?;
    rules::validate_requirement(min_proficiency, &payload.importance)
        .map_err(|e| unprocessable(&e))?;
    let mut problems = Problems::new();
    if let Some(text) = &payload.note {
        problems.cap_text("note", text);
    }
    ensure_valid(&problems.into_vec())?;
    let profile = find_profile(&ctx, &pid).await?;
    let skill = sqlx::query!(
        "SELECT pid, name FROM skills WHERE pid = $1 AND deleted_at IS NULL",
        payload.skill_pid,
    )
    .fetch_optional(db::pool(&ctx))
    .await?
    .ok_or(Error::NotFound)?;
    if let Some(n) = &native {
        check_defined(&ctx, skill.pid, n).await?;
    }
    let existing = sqlx::query_scalar!(
        "SELECT id FROM role_skill_requirements WHERE role_profile_pid = $1 AND skill_pid = $2",
        profile.pid,
        skill.pid,
    )
    .fetch_optional(db::pool(&ctx))
    .await?;
    let mut tx = db::pool(&ctx).begin().await?;
    match existing {
        Some(id) => {
            // The native level only when one is stated: an edit that gives none leaves the imported one as it was.
            sqlx::query!(
                "UPDATE role_skill_requirements
                    SET min_proficiency = $2, importance = $3, note = $4,
                        source_framework = CASE WHEN $5 THEN $6 ELSE source_framework END,
                        source_level = CASE WHEN $5 THEN $7 ELSE source_level END,
                        source_scale_max = CASE WHEN $5 THEN $8 ELSE source_scale_max END
                  WHERE id = $1",
                id,
                min_proficiency,
                payload.importance,
                payload.note,
                native.is_some(),
                native.as_ref().map(|n| n.framework.clone()),
                native.as_ref().map(|n| n.level),
                native.as_ref().map(|n| n.scale_max),
            )
            .execute(&mut *tx)
            .await?;
        }
        None => {
            sqlx::query!(
                "INSERT INTO role_skill_requirements
                        (pid, role_profile_pid, skill_pid, min_proficiency, importance, note, source_framework, source_level,
                         source_scale_max)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
                Uuid::new_v4(),
                profile.pid,
                skill.pid,
                min_proficiency,
                payload.importance,
                payload.note,
                native.as_ref().map(|n| n.framework.clone()),
                native.as_ref().map(|n| n.level),
                native.as_ref().map(|n| n.scale_max),
            )
            .execute(&mut *tx)
            .await?;
        }
    }
    db::audit(
        &mut tx,
        "role_profile",
        profile.pid,
        "requirement_set",
        caller.actor(),
        Some(serde_json::json!({
            "skill": skill.name,
            "min_proficiency": min_proficiency,
            "importance": payload.importance,
        })),
    )
    .await?;
    tx.commit().await?;
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
    let id = sqlx::query_scalar!(
        "SELECT id FROM role_skill_requirements WHERE role_profile_pid = $1 AND skill_pid = $2",
        profile.pid,
        skill_pid,
    )
    .fetch_optional(db::pool(&ctx))
    .await?
    .ok_or(Error::NotFound)?;
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!("DELETE FROM role_skill_requirements WHERE id = $1", id)
        .execute(&mut *tx)
        .await?;
    db::audit(
        &mut tx,
        "role_profile",
        profile.pid,
        "requirement_removed",
        caller.actor(),
        Some(serde_json::json!({ "skill_pid": skill_pid })),
    )
    .await?;
    tx.commit().await?;
    format::empty_json()
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
    axa::extract::Query(query): axa::extract::Query<RoleGapQuery>,
) -> Result<Response> {
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    let profile = find_profile(&ctx, &query.role_profile_pid.to_string()).await?;
    let requirements = requirements_of(&ctx, profile.pid).await?;
    let declared: BTreeMap<Uuid, i32> = sqlx::query!(
        "SELECT skill_pid, proficiency FROM worker_skills WHERE worker_pid = $1 AND deleted_at IS NULL ORDER BY id",
        worker.pid,
    )
    .fetch_all(db::pool(&ctx))
    .await?
    .into_iter()
    .map(|d| (d.skill_pid, d.proficiency))
    .collect();
    let names = skill_names(&ctx).await?;

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
    let employed: Vec<Uuid> =
        sqlx::query!("SELECT pid, hired_on, terminated_on FROM workers WHERE deleted_at IS NULL")
            .fetch_all(db::pool(&ctx))
            .await?
            .into_iter()
            .filter(|w| is_employed_on(today, w.hired_on, w.terminated_on))
            .map(|w| w.pid)
            .collect();
    let headcount = employed.len();
    let requirements = requirements_of(&ctx, profile.pid).await?;
    let declarations = sqlx::query!(
        "SELECT worker_pid, skill_pid, proficiency FROM worker_skills WHERE deleted_at IS NULL ORDER BY id"
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let names = skill_names(&ctx).await?;
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
    let siblings = sqlx::query!(
        "SELECT pid, job_title, level_order, management_track FROM role_profiles
          WHERE deleted_at IS NULL AND framework_slug = $1 AND profession = $2 AND role_name = $3
          ORDER BY id",
        framework,
        profession,
        role,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let next_order = siblings
        .iter()
        .filter_map(|p| p.level_order)
        .filter(|o| *o > order)
        .min();
    let wanted: Vec<Uuid> = siblings
        .iter()
        .map(|p| p.pid)
        .chain(std::iter::once(profile.pid))
        .collect();
    let requirements = sqlx::query_as!(
        role_skill_requirements::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, role_profile_pid, skill_pid,
                  min_proficiency, importance, note, source_framework, source_level, source_scale_max
             FROM role_skill_requirements WHERE role_profile_pid = ANY($1) ORDER BY id"#,
        &wanted,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let names = skill_names(&ctx).await?;
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
    sqlx::query_as!(
        role_profiles::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, job_title, description, source_ref,
                  framework_slug, external_ref, profession, role_name, level_name, level_order, management_track,
                  job_level_framework, job_level, pay_scale_id, pay_band, deleted_at AS "deleted_at: _"
             FROM role_profiles WHERE pid = $1 AND deleted_at IS NULL"#,
        records::parse_pid(pid)?,
    )
    .fetch_optional(db::pool(ctx))
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
