//! **ESCO** reference data and seeding (`spec/esco/index.md`): search the
//! pinned local copy of ESCO's occupations and skills, see an occupation's
//! essential and optional skills, and **seed a role profile** from an
//! occupation. Pure rules in [`crate::rules::esco`].
//!
//! ESCO has no proficiency scale, so seeding takes the planner's chosen
//! `default_min_proficiency` as an input — a level is never invented — and
//! the result is a draft for a human to edit. Essential skills draft
//! importance `important`, optional ones `useful`.

use loco_rs::prelude::*;
use sea_orm::{ActiveValue, QueryOrder, QuerySelect, TransactionTrait};
use serde::Deserialize;
use std::collections::BTreeMap;
use uuid::Uuid;

use super::{ensure_valid, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::models::_entities::{
    capability_frameworks, esco_occupation_skills, esco_occupations, esco_skills, role_profiles,
    role_skill_requirements, skill_external_refs, skills,
};
use crate::models::audit_logs::Model as Audit;
use crate::rules::esco as rules;
use crate::rules::learning::valid_proficiency;
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
    axum::extract::Query(query): axum::extract::Query<SearchQuery>,
) -> Result<Response> {
    let term = query.q.unwrap_or_default();
    if term.trim().chars().count() < 2 {
        return Err(unprocessable("q must be at least 2 characters"));
    }
    let rows = esco_occupations::Entity::find()
        .filter(sea_orm::sea_query::Expr::cust_with_values(
            "lower(label) LIKE $1 ESCAPE '\\'",
            [like_pattern(&term)],
        ))
        .order_by_asc(esco_occupations::Column::Label)
        .limit(search_limit(query.limit))
        .all(&ctx.db)
        .await?;
    let uris: Vec<String> = rows.iter().map(|r| r.uri.clone()).collect();
    let relations = esco_occupation_skills::Entity::find()
        .filter(esco_occupation_skills::Column::OccupationUri.is_in(uris))
        .all(&ctx.db)
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
    format::json(out)
}

/// `GET /api/esco/skills?q=&limit=` — ESCO skills whose preferred label
/// contains `q`, with the catalogue skill each is linked to, if any.
#[debug_handler]
async fn search_skills(
    State(ctx): State<AppContext>,
    axum::extract::Query(query): axum::extract::Query<SearchQuery>,
) -> Result<Response> {
    let term = query.q.unwrap_or_default();
    if term.trim().chars().count() < 2 {
        return Err(unprocessable("q must be at least 2 characters"));
    }
    let rows = esco_skills::Entity::find()
        .filter(sea_orm::sea_query::Expr::cust_with_values(
            "lower(label) LIKE $1 ESCAPE '\\'",
            [like_pattern(&term)],
        ))
        .order_by_asc(esco_skills::Column::Label)
        .limit(search_limit(query.limit))
        .all(&ctx.db)
        .await?;
    let linked: BTreeMap<String, Uuid> = skill_external_refs::Entity::find()
        .filter(skill_external_refs::Column::FrameworkSlug.eq(ESCO_SLUG))
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|r| (r.reference, r.skill_pid))
        .collect();
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

/// Query naming an occupation by URI.
#[derive(Debug, Deserialize)]
struct UriQuery {
    uri: String,
}

async fn find_occupation(ctx: &AppContext, uri: &str) -> Result<esco_occupations::Model> {
    esco_occupations::Entity::find()
        .filter(esco_occupations::Column::Uri.eq(uri))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)
}

/// An occupation's skills, essential first.
async fn occupation_skills(
    ctx: &AppContext,
    occupation_uri: &str,
) -> Result<Vec<(esco_occupation_skills::Model, esco_skills::Model)>> {
    let relations = esco_occupation_skills::Entity::find()
        .filter(esco_occupation_skills::Column::OccupationUri.eq(occupation_uri))
        .all(&ctx.db)
        .await?;
    let uris: Vec<String> = relations.iter().map(|r| r.skill_uri.clone()).collect();
    let by_uri: BTreeMap<String, esco_skills::Model> = esco_skills::Entity::find()
        .filter(esco_skills::Column::Uri.is_in(uris))
        .all(&ctx.db)
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
    axum::extract::Query(query): axum::extract::Query<UriQuery>,
) -> Result<Response> {
    let occupation = find_occupation(&ctx, &query.uri).await?;
    let linked: BTreeMap<String, Uuid> = skill_external_refs::Entity::find()
        .filter(skill_external_refs::Column::FrameworkSlug.eq(ESCO_SLUG))
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|r| (r.reference, r.skill_pid))
        .collect();
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
    let already = role_profiles::Entity::find()
        .filter(role_profiles::Column::DeletedAt.is_null())
        .filter(
            sea_orm::Condition::any()
                .add(role_profiles::Column::JobTitle.eq(&title))
                .add(
                    sea_orm::Condition::all()
                        .add(role_profiles::Column::FrameworkSlug.eq(ESCO_SLUG))
                        .add(role_profiles::Column::ExternalRef.eq(&occupation.uri)),
                ),
        )
        .one(&ctx.db)
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
    let version = capability_frameworks::Entity::find()
        .filter(capability_frameworks::Column::Slug.eq(ESCO_SLUG))
        .one(&ctx.db)
        .await?
        .and_then(|f| f.note)
        .and_then(|n| {
            n.rsplit("Pinned version: ")
                .next()
                .map(|v| v.trim_end_matches('.').to_string())
        });

    let txn = ctx.db.begin().await?;
    let profile = role_profiles::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        job_title: ActiveValue::set(title),
        description: ActiveValue::set(occupation.description.clone()),
        source_ref: ActiveValue::set(Some(format!("{ESCO_SLUG}:{}", occupation.uri))),
        framework_slug: ActiveValue::set(Some(ESCO_SLUG.to_string())),
        external_ref: ActiveValue::set(Some(occupation.uri.clone())),
        profession: ActiveValue::set(
            occupation
                .isco_code
                .as_ref()
                .map(|c| format!("ISCO-08 {c}")),
        ),
        role_name: ActiveValue::set(Some(occupation.label.clone())),
        management_track: ActiveValue::set(false),
        deleted_at: ActiveValue::set(None),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    let (mut created, mut linked, mut requirements) = (0usize, 0usize, 0usize);
    for (relation, esco_skill) in &wanted {
        let relation_kind = if relation.relation == "essential" {
            rules::Relation::Essential
        } else {
            rules::Relation::Optional
        };
        // Match: ESCO reference, else exact label, else create.
        let by_ref = skill_external_refs::Entity::find()
            .filter(skill_external_refs::Column::FrameworkSlug.eq(ESCO_SLUG))
            .filter(skill_external_refs::Column::Reference.eq(&esco_skill.uri))
            .one(&txn)
            .await?;
        let skill_pid = if let Some(r) = by_ref {
            r.skill_pid
        } else {
            let wanted_label = rules::normalise_label(&esco_skill.label);
            let candidates: Vec<skills::Model> = skills::Entity::find()
                .filter(skills::Column::DeletedAt.is_null())
                .all(&txn)
                .await?
                .into_iter()
                .filter(|s| rules::normalise_label(&s.name) == wanted_label)
                .collect();
            let (pid, is_new) = match candidates.as_slice() {
                [one] if !has_esco_ref(&txn, one.pid).await? => (one.pid, false),
                _ => {
                    let row = skills::ActiveModel {
                        pid: ActiveValue::set(Uuid::new_v4()),
                        name: ActiveValue::set(unique_name(&txn, &esco_skill.label).await?),
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
                    .insert(&txn)
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
                version: ActiveValue::set(version.clone()),
                ..Default::default()
            }
            .insert(&txn)
            .await?;
            if is_new {
                created += 1;
            } else {
                linked += 1;
            }
            pid
        };
        // One requirement per (profile, skill); two ESCO skills mapping to one
        // catalogue skill collapse to the first.
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
            min_proficiency: ActiveValue::set(payload.default_min_proficiency),
            importance: ActiveValue::set(relation_kind.importance().to_string()),
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
        "seeded_from_esco",
        caller.actor(),
        Some(serde_json::json!({ "occupation": occupation.uri, "requirements": requirements })),
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
    Err(Error::string("cannot find a free skill name"))
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
