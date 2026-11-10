//! **SFIA** in role profiles and declared skills (WPM-R111, WPM-D70): search the
//! deployer's imported catalogue, see a skill's levels, start a role from SFIA
//! skills, and the helpers that keep a framework's **own level** next to WPM's
//! 1–5 figure. Nothing here ships SFIA wording: it is read from the tables
//! `import_sfia` filled, and only for a signed-in caller, so it does not leave
//! the organization (WPM-R113).

use axa::extract::{Json, Path};
use axa::prelude::*;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

use super::{ensure_valid, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::db;
use crate::rules::esco::normalise_label;
use crate::rules::sfia as rules;
use crate::tasks::import_sfia::SFIA_SLUG;
use crate::validation::Problems;

/// Whatever the helpers below can read through: the context, the pool, or, while callers outside this module still hold one,
/// the `SeaORM` handle over the same pool (WPM-T466). The last goes when `SeaORM` does.
pub(crate) trait OnPool {
    /// The `sqlx` pool underneath.
    fn on_pool(&self) -> &sqlx::PgPool;
}

impl OnPool for sqlx::PgPool {
    fn on_pool(&self) -> &sqlx::PgPool {
        self
    }
}

impl OnPool for AppContext {
    fn on_pool(&self) -> &sqlx::PgPool {
        db::pool(self)
    }
}

impl OnPool for sea_orm::DatabaseConnection {
    fn on_pool(&self) -> &sqlx::PgPool {
        self.get_postgres_connection_pool()
    }
}

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
    db: &impl OnPool,
    framework: Option<&str>,
    level: Option<i32>,
) -> Result<Option<Native>> {
    let (framework, level) = match (framework.map(str::trim), level) {
        (None | Some(""), None) => return Ok(None),
        (Some(f), Some(l)) if !f.is_empty() => (f, l),
        _ => return Err(unprocessable("give framework and framework_level together")),
    };
    let row = sqlx::query!(
        "SELECT slug, scale_max FROM capability_frameworks WHERE slug = $1 LIMIT 1",
        framework,
    )
    .fetch_optional(db.on_pool())
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
async fn sfia_code(db: &impl OnPool, skill_pid: Uuid) -> Result<Option<String>> {
    Ok(sqlx::query_scalar!(
        r#"SELECT "ref" FROM skill_external_refs WHERE skill_pid = $1 AND framework_slug = $2 ORDER BY id LIMIT 1"#,
        skill_pid,
        SFIA_SLUG,
    )
    .fetch_optional(db.on_pool())
    .await?)
}

/// A skill known in SFIA may only be stated at a level SFIA defines for it (a level is never invented).
///
/// # Errors
/// 422 when the skill has a SFIA code and that code has no current definition at this level.
pub(crate) async fn check_defined(
    db: &impl OnPool,
    skill_pid: Uuid,
    native: &Native,
) -> Result<()> {
    if native.framework != SFIA_SLUG {
        return Ok(());
    }
    let Some(code) = sfia_code(db, skill_pid).await? else {
        return Ok(());
    };
    let defined = sqlx::query_scalar!(
        r#"SELECT 1 AS "one!" FROM sfia_skill_levels WHERE code = $1 AND level = $2 AND retired_in IS NULL LIMIT 1"#,
        code,
        native.level,
    )
    .fetch_optional(db.on_pool())
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
pub(crate) async fn framework_names(db: &impl OnPool) -> Result<BTreeMap<String, String>> {
    Ok(
        sqlx::query!("SELECT slug, name FROM capability_frameworks ORDER BY id")
            .fetch_all(db.on_pool())
            .await?
            .into_iter()
            .map(|f| (f.slug, f.name))
            .collect(),
    )
}

/// The SFIA codes of these catalogue skills, by skill.
async fn sfia_codes(db: &impl OnPool, skills: &[Uuid]) -> Result<BTreeMap<Uuid, String>> {
    Ok(sqlx::query!(
        r#"SELECT skill_pid, "ref" AS reference FROM skill_external_refs
            WHERE framework_slug = $1 AND skill_pid = ANY($2) ORDER BY id"#,
        SFIA_SLUG,
        skills,
    )
    .fetch_all(db.on_pool())
    .await?
    .into_iter()
    .map(|r| (r.skill_pid, r.reference))
    .collect())
}

/// The framework's own wording for each `(skill, level)` stated on SFIA, so a native level can be shown with what it says.
///
/// # Errors
/// A database error.
pub(crate) async fn native_descriptions(
    db: &impl OnPool,
    stated: &[(Uuid, Option<&str>, Option<i32>)],
) -> Result<BTreeMap<Uuid, String>> {
    let wanted: Vec<(Uuid, i32)> = stated
        .iter()
        .filter_map(|(p, f, l)| (*f == Some(SFIA_SLUG)).then_some((*p, (*l)?)))
        .collect();
    if wanted.is_empty() {
        return Ok(BTreeMap::new());
    }
    let skill_pids: Vec<Uuid> = wanted.iter().map(|(p, _)| *p).collect();
    let refs = sfia_codes(db, &skill_pids).await?;
    let codes: Vec<String> = refs.values().cloned().collect();
    let levels: BTreeMap<(String, i32), String> = sqlx::query!(
        "SELECT code, level, description FROM sfia_skill_levels WHERE code = ANY($1) ORDER BY id",
        &codes,
    )
    .fetch_all(db.on_pool())
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
    db: &impl OnPool,
    wanted: &[(Uuid, i32)],
) -> Result<BTreeMap<Uuid, Vec<serde_json::Value>>> {
    let Some(source) = sqlx::query_scalar!(
        "SELECT name FROM capability_frameworks WHERE slug = $1 LIMIT 1",
        crate::tasks::import_sfiaplus::SFIAPLUS_SLUG,
    )
    .fetch_optional(db.on_pool())
    .await?
    else {
        return Ok(BTreeMap::new());
    };
    let skill_pids: Vec<Uuid> = wanted.iter().map(|(p, _)| *p).collect();
    let refs = sfia_codes(db, &skill_pids).await?;
    if refs.is_empty() {
        return Ok(BTreeMap::new());
    }
    let codes: Vec<String> = refs.values().cloned().collect();
    let resources = sqlx::query!(
        "SELECT code, level, kind, title, provider, url FROM sfiaplus_resources
          WHERE code = ANY($1) AND retired_in IS NULL ORDER BY id",
        &codes,
    )
    .fetch_all(db.on_pool())
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
                    "source": source, "sfia_code": r.code, "sfia_level": r.level,
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
    axa::extract::Query(query): axa::extract::Query<SearchQuery>,
) -> Result<Response> {
    let q = query.q.unwrap_or_default().trim().to_lowercase();
    if q.chars().count() < 2 {
        return Err(unprocessable("q must be at least 2 characters"));
    }
    let levels = sqlx::query!(
        "SELECT code, level FROM sfia_skill_levels WHERE retired_in IS NULL ORDER BY id"
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let out: Vec<serde_json::Value> = sqlx::query!(
        "SELECT code, name, category, subcategory, retired_in FROM sfia_skills ORDER BY id"
    )
    .fetch_all(db::pool(&ctx))
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
    let skill = sfia_skill(&ctx, code.trim())
        .await?
        .ok_or(Error::NotFound)?;
    let levels = sqlx::query!(
        "SELECT level, description, attributes, retired_in FROM sfia_skill_levels WHERE code = $1 ORDER BY id",
        skill.code,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let resources = sqlx::query!(
        "SELECT level, kind, title, provider, url FROM sfiaplus_resources
          WHERE code = $1 AND retired_in IS NULL ORDER BY id",
        skill.code,
    )
    .fetch_all(db::pool(&ctx))
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

/// One skill of the imported catalogue.
struct SfiaSkill {
    code: String,
    name: String,
    category: String,
    subcategory: String,
    description: String,
    retired_in: Option<String>,
}

/// The imported catalogue's skill with this code.
async fn sfia_skill(ctx: &AppContext, code: &str) -> Result<Option<SfiaSkill>> {
    Ok(sqlx::query_as!(
        SfiaSkill,
        "SELECT code, name, category, subcategory, description, retired_in FROM sfia_skills WHERE code = $1 ORDER BY id LIMIT 1",
        code,
    )
    .fetch_optional(db::pool(ctx))
    .await?)
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
    // In id order, so a later row of the same code wins, as it did when read in table order.
    let known: BTreeMap<String, SfiaSkill> = sqlx::query_as!(
        SfiaSkill,
        "SELECT code, name, category, subcategory, description, retired_in FROM sfia_skills ORDER BY id"
    )
    .fetch_all(db::pool(&ctx))
    .await?
    .into_iter()
    .map(|s| (s.code.clone(), s))
    .collect();
    let defined: BTreeSet<(String, i32)> =
        sqlx::query!("SELECT code, level FROM sfia_skill_levels WHERE retired_in IS NULL")
            .fetch_all(db::pool(&ctx))
            .await?
            .into_iter()
            .map(|l| (l.code, l.level))
            .collect();
    let mut chosen: Vec<(&SfiaSkill, i32, i32)> = Vec::new();
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
    let taken = sqlx::query_scalar!(
        r#"SELECT 1 AS "one!" FROM role_profiles WHERE deleted_at IS NULL AND job_title = $1 LIMIT 1"#,
        title,
    )
    .fetch_optional(db::pool(&ctx))
    .await?
    .is_some();
    if taken {
        return Err(unprocessable(
            "a role profile with that job title already exists",
        ));
    }

    let mut tx = db::pool(&ctx).begin().await?;
    let profile_pid = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO role_profiles (pid, job_title, source_ref, management_track, deleted_at)
         VALUES ($1, $2, $3, false, NULL)",
        profile_pid,
        title,
        format!("{SFIA_SLUG}:picked-skills"),
    )
    .execute(&mut *tx)
    .await?;
    let (mut created, mut linked, mut requirements) = (0usize, 0usize, 0usize);
    let version = sqlx::query_scalar!(
        "SELECT version FROM capability_frameworks WHERE slug = $1 LIMIT 1",
        SFIA_SLUG,
    )
    .fetch_optional(&mut *tx)
    .await?
    .flatten();
    for (sfia, level, wpm) in chosen {
        let by_ref = sqlx::query_scalar!(
            r#"SELECT skill_pid FROM skill_external_refs WHERE framework_slug = $1 AND "ref" = $2 ORDER BY id LIMIT 1"#,
            SFIA_SLUG,
            sfia.code,
        )
        .fetch_optional(&mut *tx)
        .await?;
        let skill_pid = if let Some(skill_pid) = by_ref {
            skill_pid
        } else {
            let wanted = normalise_label(&sfia.name);
            let same_name: Vec<Uuid> =
                sqlx::query!("SELECT pid, name FROM skills WHERE deleted_at IS NULL ORDER BY id")
                    .fetch_all(&mut *tx)
                    .await?
                    .into_iter()
                    .filter(|s| normalise_label(&s.name) == wanted)
                    .map(|s| s.pid)
                    .collect();
            let reused = match same_name.as_slice() {
                [one] => {
                    let has_ref = sqlx::query_scalar!(
                        r#"SELECT 1 AS "one!" FROM skill_external_refs WHERE skill_pid = $1 AND framework_slug = $2 LIMIT 1"#,
                        *one,
                        SFIA_SLUG,
                    )
                    .fetch_optional(&mut *tx)
                    .await?
                    .is_some();
                    (!has_ref).then_some(*one)
                }
                _ => None,
            };
            let pid = if let Some(pid) = reused {
                linked += 1;
                pid
            } else {
                let taken_name = sqlx::query_scalar!(
                    r#"SELECT 1 AS "one!" FROM skills WHERE name = $1 LIMIT 1"#,
                    sfia.name,
                )
                .fetch_optional(&mut *tx)
                .await?
                .is_some();
                let name = if taken_name {
                    format!("{} (SFIA)", sfia.name)
                } else {
                    sfia.name.clone()
                };
                created += 1;
                let pid = Uuid::new_v4();
                sqlx::query!(
                    "INSERT INTO skills (pid, name, category, deleted_at) VALUES ($1, $2, $3, NULL)",
                    pid,
                    name,
                    category,
                )
                .execute(&mut *tx)
                .await?;
                pid
            };
            sqlx::query!(
                r#"INSERT INTO skill_external_refs (pid, skill_pid, framework_slug, "ref", label, version)
                   VALUES ($1, $2, $3, $4, $5, $6)"#,
                Uuid::new_v4(),
                pid,
                SFIA_SLUG,
                sfia.code,
                sfia.name,
                version,
            )
            .execute(&mut *tx)
            .await?;
            pid
        };
        let exists = sqlx::query_scalar!(
            r#"SELECT 1 AS "one!" FROM role_skill_requirements WHERE role_profile_pid = $1 AND skill_pid = $2 LIMIT 1"#,
            profile_pid,
            skill_pid,
        )
        .fetch_optional(&mut *tx)
        .await?
        .is_some();
        if exists {
            continue;
        }
        sqlx::query!(
            "INSERT INTO role_skill_requirements (pid, role_profile_pid, skill_pid, min_proficiency, importance,
                                                  source_framework, source_level, source_scale_max)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            Uuid::new_v4(),
            profile_pid,
            skill_pid,
            wpm,
            importance,
            SFIA_SLUG,
            level,
            rules::SFIA_SCALE_MAX,
        )
        .execute(&mut *tx)
        .await?;
        requirements += 1;
    }
    db::audit(
        &mut tx,
        "role_profile",
        profile_pid,
        "seeded_from_sfia",
        caller.actor(),
        Some(serde_json::json!({ "requirements": requirements })),
    )
    .await?;
    tx.commit().await?;
    format::json(serde_json::json!({
        "pid": profile_pid,
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
