//! **ESCO** reference data and seeding (`spec/esco/index.md`): search the
//! pinned local copy of ESCO's occupations and skills, see an occupation's
//! essential and optional skills, and **seed a role profile** from an
//! occupation. Pure rules in [`crate::rules::esco`].
//!
//! ESCO has no proficiency scale, so seeding takes the planner's chosen
//! `default_min_proficiency` as an input — a level is never invented — and
//! the result is a draft for a human to edit. Essential skills draft
//! importance `important`, optional ones `useful`.

use axa::extract::Json;
use axa::prelude::*;
use sea_orm::ActiveValue;
use serde::Deserialize;
use std::collections::BTreeMap;
use uuid::Uuid;

use super::{ensure_valid, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::db;
use crate::models::_entities::{
    capability_frameworks, esco_occupation_skills, esco_occupations, esco_skills,
    skill_external_refs, skills,
};
use crate::rules::esco as rules;
use crate::rules::learning::valid_proficiency;
use crate::shared_cache::Namespace;
use crate::tasks::import_esco::ESCO_SLUG;
use crate::validation::Problems;

/// Query for a label search.
#[derive(Debug, Deserialize)]
struct SearchQuery {
    q: Option<String>,
    limit: Option<u64>,
}

fn search_limit(limit: Option<u64>) -> u64 {
    limit.unwrap_or(25).clamp(1, 100)
}

/// Escape `%`, `_` and `\` so a search term is matched literally.
fn like_pattern(term: &str) -> String {
    let escaped = term
        .trim()
        .to_lowercase()
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    format!("%{escaped}%")
}

/// `GET /api/esco/occupations?q=&limit=` — occupations whose preferred label
/// contains `q` (case-insensitive; at most `limit`, default 25, max 100),
/// each with how many essential and optional skills it lists.
#[debug_handler]
async fn search_occupations(
    State(ctx): State<AppContext>,
    axa::extract::Query(query): axa::extract::Query<SearchQuery>,
) -> Result<Response> {
    let term = query.q.unwrap_or_default();
    if term.trim().chars().count() < 2 {
        return Err(unprocessable("q must be at least 2 characters"));
    }
    let cache_key = format!(
        "{}|{}",
        term.trim().to_lowercase(),
        search_limit(query.limit)
    );
    if let Some(hit) = crate::shared_cache::get(Namespace::EscoOccupations, &cache_key).await
        && let Ok(value) = serde_json::from_str::<serde_json::Value>(&hit)
    {
        return format::json(value);
    }
    let rows = sqlx::query!(
        r"SELECT uri, label, isco_code FROM esco_occupations
           WHERE lower(label) LIKE $1 ESCAPE '\'
           ORDER BY label LIMIT $2",
        like_pattern(&term),
        i64::try_from(search_limit(query.limit)).unwrap_or(100),
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let uris: Vec<String> = rows.iter().map(|r| r.uri.clone()).collect();
    let relations = sqlx::query!(
        "SELECT occupation_uri, relation FROM esco_occupation_skills WHERE occupation_uri = ANY($1)",
        &uris,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let out: Vec<serde_json::Value> = rows
        .iter()
        .map(|o| {
            let count = |kind: &str| {
                relations
                    .iter()
                    .filter(|r| r.occupation_uri == o.uri && r.relation == kind)
                    .count()
            };
            serde_json::json!({
                "uri": o.uri,
                "label": o.label,
                "isco_code": o.isco_code,
                "essential_skills": count("essential"),
                "optional_skills": count("optional"),
            })
        })
        .collect();
    let out = serde_json::Value::Array(out);
    if let Ok(text) = serde_json::to_string(&out) {
        crate::shared_cache::put(Namespace::EscoOccupations, &cache_key, &text).await;
    }
    format::json(out)
}

/// `GET /api/esco/skills?q=&limit=` — ESCO skills whose preferred label
/// contains `q`, with the catalogue skill each is linked to, if any.
#[debug_handler]
async fn search_skills(
    State(ctx): State<AppContext>,
    axa::extract::Query(query): axa::extract::Query<SearchQuery>,
) -> Result<Response> {
    let term = query.q.unwrap_or_default();
    if term.trim().chars().count() < 2 {
        return Err(unprocessable("q must be at least 2 characters"));
    }
    let rows = sqlx::query!(
        r"SELECT uri, label, skill_type, reuse_level FROM esco_skills
           WHERE lower(label) LIKE $1 ESCAPE '\'
           ORDER BY label LIMIT $2",
        like_pattern(&term),
        i64::try_from(search_limit(query.limit)).unwrap_or(100),
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let linked = esco_links(&ctx).await?;
    let out: Vec<serde_json::Value> = rows
        .iter()
        .map(|s| {
            serde_json::json!({
                "uri": s.uri,
                "label": s.label,
                "skill_type": s.skill_type,
                "reuse_level": s.reuse_level,
                "catalogue_skill_pid": linked.get(&s.uri),
            })
        })
        .collect();
    format::json(out)
}

/// Every catalogue skill linked to an ESCO skill, by the ESCO skill's URI.
async fn esco_links(ctx: &AppContext) -> Result<BTreeMap<String, Uuid>> {
    Ok(sqlx::query!(
        "SELECT ref AS reference, skill_pid FROM skill_external_refs WHERE framework_slug = $1",
        ESCO_SLUG,
    )
    .fetch_all(db::pool(ctx))
    .await?
    .into_iter()
    .map(|r| (r.reference, r.skill_pid))
    .collect())
}

/// Query naming an occupation by URI.
#[derive(Debug, Deserialize)]
struct UriQuery {
    uri: String,
}

pub(crate) async fn find_occupation(
    ctx: &AppContext,
    uri: &str,
) -> Result<esco_occupations::Model> {
    sqlx::query_as!(
        esco_occupations::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, uri, label, isco_code, description
             FROM esco_occupations WHERE uri = $1"#,
        uri,
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)
}

/// An occupation's skills, essential first.
pub(crate) async fn occupation_skills(
    ctx: &AppContext,
    occupation_uri: &str,
) -> Result<Vec<(esco_occupation_skills::Model, esco_skills::Model)>> {
    let relations = sqlx::query_as!(
        esco_occupation_skills::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, occupation_uri, skill_uri, relation
             FROM esco_occupation_skills WHERE occupation_uri = $1"#,
        occupation_uri,
    )
    .fetch_all(db::pool(ctx))
    .await?;
    let uris: Vec<String> = relations.iter().map(|r| r.skill_uri.clone()).collect();
    let by_uri: BTreeMap<String, esco_skills::Model> = sqlx::query_as!(
        esco_skills::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, uri, label, skill_type, reuse_level
             FROM esco_skills WHERE uri = ANY($1)"#,
        &uris,
    )
    .fetch_all(db::pool(ctx))
    .await?
    .into_iter()
        .map(|s| (s.uri.clone(), s))
        .collect();
    let mut out: Vec<(esco_occupation_skills::Model, esco_skills::Model)> = relations
        .into_iter()
        .filter_map(|r| by_uri.get(&r.skill_uri).cloned().map(|s| (r, s)))
        .collect();
    out.sort_by(|a, b| {
        (a.0.relation != "essential", &a.1.label).cmp(&(b.0.relation != "essential", &b.1.label))
    });
    Ok(out)
}

/// `GET /api/esco/occupation?uri=` — one occupation with its essential and
/// optional skills and the catalogue skill each is already linked to.
#[debug_handler]
async fn get_occupation(
    State(ctx): State<AppContext>,
    axa::extract::Query(query): axa::extract::Query<UriQuery>,
) -> Result<Response> {
    let occupation = find_occupation(&ctx, &query.uri).await?;
    let linked = esco_links(&ctx).await?;
    let skills_out: Vec<serde_json::Value> = occupation_skills(&ctx, &occupation.uri)
        .await?
        .iter()
        .map(|(relation, skill)| {
            serde_json::json!({
                "uri": skill.uri,
                "label": skill.label,
                "relation": relation.relation,
                "skill_type": skill.skill_type,
                "reuse_level": skill.reuse_level,
                "draft_category": rules::category_for(skill.skill_type.as_deref(), skill.reuse_level.as_deref()),
                "catalogue_skill_pid": linked.get(&skill.uri),
            })
        })
        .collect();
    format::json(serde_json::json!({
        "uri": occupation.uri,
        "label": occupation.label,
        "isco_code": occupation.isco_code,
        "description": occupation.description,
        "skills": skills_out,
    }))
}

/// `POST /api/role-profiles/from-esco` body.
#[derive(Debug, Deserialize)]
struct FromEscoPayload {
    occupation_uri: String,
    /// The job title for the new profile; default the occupation's label.
    #[serde(default)]
    job_title: Option<String>,
    /// The minimum proficiency (1–5) every drafted requirement starts at —
    /// the planner's call, since ESCO states no levels.
    default_min_proficiency: i32,
    /// Also draft the occupation's optional skills (as `useful`).
    #[serde(default)]
    include_optional: bool,
}

/// `POST /api/role-profiles/from-esco` — draft a role profile from an ESCO
/// occupation: essential skills (and optionally optional ones) become
/// requirements at the planner's chosen level, importance `important` /
/// `useful`. Catalogue skills are matched by their ESCO reference, then by
/// exact label, else created (draft category from ESCO's skill type and
/// reuse level) and linked. A draft for a human to edit.
#[debug_handler]
#[allow(clippy::too_many_lines)] // resolve occupation → profile → each skill, linearly
async fn from_esco(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<FromEscoPayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    if !valid_proficiency(payload.default_min_proficiency) {
        problems.push("default_min_proficiency must be between 1 and 5");
    }
    if let Some(title) = &payload.job_title {
        problems.cap_text("job_title", title);
    }
    ensure_valid(&problems.into_vec())?;
    let occupation = find_occupation(&ctx, &payload.occupation_uri).await?;
    let title = crate::rules::roles::normalize_job_title(
        payload.job_title.as_deref().unwrap_or(&occupation.label),
    )
    .map_err(|e| unprocessable(&e))?;
    let already = sqlx::query_scalar!(
        r#"SELECT 1 AS "one!" FROM role_profiles
            WHERE deleted_at IS NULL AND (job_title = $1 OR (framework_slug = $2 AND external_ref = $3))
            LIMIT 1"#,
        title,
        ESCO_SLUG,
        occupation.uri,
    )
    .fetch_optional(db::pool(&ctx))
    .await?
    .is_some();
    if already {
        return Err(unprocessable(
            "a role profile with that job title already exists, or a profile was already seeded from this occupation",
        ));
    }
    let wanted: Vec<(esco_occupation_skills::Model, esco_skills::Model)> =
        occupation_skills(&ctx, &occupation.uri)
            .await?
            .into_iter()
            .filter(|(r, _)| payload.include_optional || r.relation == "essential")
            .collect();
    let version = pinned_version_in(&ctx).await?;

    let profile_pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO role_profiles (pid, job_title, description, source_ref, framework_slug, external_ref, profession,
                                    role_name, management_track)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, false)",
        profile_pid,
        title,
        occupation.description,
        format!("{ESCO_SLUG}:{}", occupation.uri),
        ESCO_SLUG,
        occupation.uri,
        occupation
            .isco_code
            .as_ref()
            .map(|c| format!("ISCO-08 {c}")),
        occupation.label,
    )
    .execute(&mut *tx)
    .await?;
    let (mut created, mut linked, mut requirements) = (0usize, 0usize, 0usize);
    for (relation, esco_skill) in &wanted {
        let relation_kind = if relation.relation == "essential" {
            rules::Relation::Essential
        } else {
            rules::Relation::Optional
        };
        let (skill_pid, how) =
            resolve_catalogue_skill_in(&mut tx, esco_skill, version.as_deref()).await?;
        match how {
            Resolution::Created => created += 1,
            Resolution::Linked => linked += 1,
            Resolution::Existing => {}
        }
        // One requirement per (profile, skill); two ESCO skills mapping to one
        // catalogue skill collapse to the first.
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
            "INSERT INTO role_skill_requirements (pid, role_profile_pid, skill_pid, min_proficiency, importance)
             VALUES ($1, $2, $3, $4, $5)",
            Uuid::new_v4(),
            profile_pid,
            skill_pid,
            payload.default_min_proficiency,
            relation_kind.importance(),
        )
        .execute(&mut *tx)
        .await?;
        requirements += 1;
    }
    db::audit(
        &mut tx,
        "role_profile",
        profile_pid,
        "seeded_from_esco",
        caller.actor(),
        Some(serde_json::json!({ "occupation": occupation.uri, "requirements": requirements })),
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

/// The ESCO version pinned by the last import, from the framework row's note.
fn version_from_note(note: Option<String>) -> Option<String> {
    note.and_then(|n| {
        n.rsplit("Pinned version: ")
            .next()
            .map(|v| v.trim_end_matches('.').to_string())
    })
}

/// [`pinned_version`] through `sqlx`.
///
/// # Errors
/// A database failure.
pub(crate) async fn pinned_version_in(ctx: &AppContext) -> Result<Option<String>> {
    let note = sqlx::query_scalar!(
        "SELECT note FROM capability_frameworks WHERE slug = $1 LIMIT 1",
        ESCO_SLUG
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .flatten();
    Ok(version_from_note(note))
}

/// [`resolve_catalogue_skill`] inside a `sqlx` transaction: the same resolution, in the same order.
///
/// # Errors
/// A database failure, or no free catalogue name.
pub(crate) async fn resolve_catalogue_skill_in(
    tx: &mut sqlx::PgConnection,
    esco_skill: &esco_skills::Model,
    version: Option<&str>,
) -> Result<(Uuid, Resolution)> {
    let by_ref = sqlx::query_scalar!(
        "SELECT skill_pid FROM skill_external_refs WHERE framework_slug = $1 AND ref = $2 LIMIT 1",
        ESCO_SLUG,
        esco_skill.uri,
    )
    .fetch_optional(&mut *tx)
    .await?;
    if let Some(skill_pid) = by_ref {
        return Ok((skill_pid, Resolution::Existing));
    }
    let wanted_label = rules::normalise_label(&esco_skill.label);
    let candidates: Vec<Uuid> =
        sqlx::query!("SELECT pid, name FROM skills WHERE deleted_at IS NULL ORDER BY id")
            .fetch_all(&mut *tx)
            .await?
            .into_iter()
            .filter(|s| rules::normalise_label(&s.name) == wanted_label)
            .map(|s| s.pid)
            .collect();
    let (pid, created) = match candidates.as_slice() {
        [one] if !has_esco_ref_in(tx, *one).await? => (*one, false),
        _ => {
            let pid = Uuid::new_v4();
            let name = unique_name_in(tx, &esco_skill.label).await?;
            sqlx::query!(
                "INSERT INTO skills (pid, name, category) VALUES ($1, $2, $3)",
                pid,
                name,
                rules::category_for(
                    esco_skill.skill_type.as_deref(),
                    esco_skill.reuse_level.as_deref(),
                ),
            )
            .execute(&mut *tx)
            .await?;
            (pid, true)
        }
    };
    sqlx::query!(
        "INSERT INTO skill_external_refs (pid, skill_pid, framework_slug, ref, label, version)
         VALUES ($1, $2, $3, $4, $5, $6)",
        Uuid::new_v4(),
        pid,
        ESCO_SLUG,
        esco_skill.uri,
        esco_skill.label,
        version,
    )
    .execute(&mut *tx)
    .await?;
    Ok((
        pid,
        if created {
            Resolution::Created
        } else {
            Resolution::Linked
        },
    ))
}

/// Whether a catalogue skill already carries an ESCO reference.
async fn has_esco_ref_in(tx: &mut sqlx::PgConnection, skill_pid: Uuid) -> Result<bool> {
    Ok(sqlx::query_scalar!(
        r#"SELECT 1 AS "one!" FROM skill_external_refs WHERE skill_pid = $1 AND framework_slug = $2 LIMIT 1"#,
        skill_pid,
        ESCO_SLUG,
    )
    .fetch_optional(&mut *tx)
    .await?
    .is_some())
}

/// A catalogue name not yet taken: the label, else the label with ` (ESCO)`.
async fn unique_name_in(tx: &mut sqlx::PgConnection, label: &str) -> Result<String> {
    for candidate in [label.to_string(), format!("{label} (ESCO)")] {
        let taken = sqlx::query_scalar!(
            r#"SELECT 1 AS "one!" FROM skills WHERE name = $1 LIMIT 1"#,
            candidate
        )
        .fetch_optional(&mut *tx)
        .await?
        .is_some();
        if !taken {
            return Ok(candidate);
        }
    }
    Err(super::internal("cannot find a free skill name"))
}

// The SeaORM forms below serve `framework_roles.rs`, whose declaration handler still runs one SeaORM transaction
// (WPM-T466). They go when it moves to the `sqlx` forms above.

/// The ESCO version pinned by the last import, from the framework row's note.
pub(crate) async fn pinned_version(db: &sea_orm::DatabaseConnection) -> Result<Option<String>> {
    Ok(capability_frameworks::Entity::find()
        .filter(capability_frameworks::Column::Slug.eq(ESCO_SLUG))
        .one(db)
        .await?
        .and_then(|f| version_from_note(f.note)))
}

/// How an ESCO skill was resolved to a catalogue skill.
pub(crate) enum Resolution {
    /// Already linked to this ESCO skill.
    Existing,
    /// An existing catalogue skill matched by exact label, now linked.
    Linked,
    /// A new catalogue skill was created (draft category) and linked.
    Created,
}

/// Resolve an ESCO skill to a catalogue skill: by its ESCO reference, else by
/// exact normalised label (when unambiguous and not already linked to another
/// ESCO skill), else create one with a draft category — and link it. Shared by
/// role seeding and by a user selecting their own ESCO skills.
pub(crate) async fn resolve_catalogue_skill(
    txn: &sea_orm::DatabaseTransaction,
    esco_skill: &esco_skills::Model,
    version: Option<&str>,
) -> Result<(Uuid, Resolution)> {
    let by_ref = skill_external_refs::Entity::find()
        .filter(skill_external_refs::Column::FrameworkSlug.eq(ESCO_SLUG))
        .filter(skill_external_refs::Column::Reference.eq(&esco_skill.uri))
        .one(txn)
        .await?;
    if let Some(r) = by_ref {
        return Ok((r.skill_pid, Resolution::Existing));
    }
    let wanted_label = rules::normalise_label(&esco_skill.label);
    let candidates: Vec<skills::Model> = skills::Entity::find()
        .filter(skills::Column::DeletedAt.is_null())
        .all(txn)
        .await?
        .into_iter()
        .filter(|s| rules::normalise_label(&s.name) == wanted_label)
        .collect();
    let (pid, created) = match candidates.as_slice() {
        [one] if !has_esco_ref(txn, one.pid).await? => (one.pid, false),
        _ => {
            let row = skills::ActiveModel {
                pid: ActiveValue::set(Uuid::new_v4()),
                name: ActiveValue::set(unique_name(txn, &esco_skill.label).await?),
                category: ActiveValue::set(
                    rules::category_for(
                        esco_skill.skill_type.as_deref(),
                        esco_skill.reuse_level.as_deref(),
                    )
                    .to_string(),
                ),
                deleted_at: ActiveValue::set(None),
                ..Default::default()
            }
            .insert(txn)
            .await?;
            (row.pid, true)
        }
    };
    skill_external_refs::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        skill_pid: ActiveValue::set(pid),
        framework_slug: ActiveValue::set(ESCO_SLUG.to_string()),
        reference: ActiveValue::set(esco_skill.uri.clone()),
        label: ActiveValue::set(Some(esco_skill.label.clone())),
        version: ActiveValue::set(version.map(ToString::to_string)),
        ..Default::default()
    }
    .insert(txn)
    .await?;
    Ok((
        pid,
        if created {
            Resolution::Created
        } else {
            Resolution::Linked
        },
    ))
}

/// Whether a catalogue skill already carries an ESCO reference.
async fn has_esco_ref(txn: &sea_orm::DatabaseTransaction, skill_pid: Uuid) -> Result<bool> {
    Ok(skill_external_refs::Entity::find()
        .filter(skill_external_refs::Column::SkillPid.eq(skill_pid))
        .filter(skill_external_refs::Column::FrameworkSlug.eq(ESCO_SLUG))
        .one(txn)
        .await?
        .is_some())
}

/// A catalogue name not yet taken: the label, else the label with ` (ESCO)`.
async fn unique_name(txn: &sea_orm::DatabaseTransaction, label: &str) -> Result<String> {
    for candidate in [label.to_string(), format!("{label} (ESCO)")] {
        let taken = skills::Entity::find()
            .filter(skills::Column::Name.eq(&candidate))
            .one(txn)
            .await?
            .is_some();
        if !taken {
            return Ok(candidate);
        }
    }
    Err(super::internal("cannot find a free skill name"))
}

/// The ESCO routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/esco/occupations", get(search_occupations))
        .add("/esco/occupation", get(get_occupation))
        .add("/esco/skills", get(search_skills))
        .add("/role-profiles/from-esco", post(from_esco))
}
