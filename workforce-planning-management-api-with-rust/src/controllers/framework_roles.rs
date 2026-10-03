//! **My role and skills** in an external framework: a person selects their
//! *current* UK GDAD PCF role level or ESCO occupation, sees that role's
//! skills, and selects which they have and at what level. The skills they
//! select are ordinary skill declarations (`worker_skills`), so everything
//! built on declared skills — gap analysis, matching, capability — picks them
//! up. Pure rules in [`crate::rules::framework_roles`].
//!
//! Self-description, not assessment: the person chooses their own level on
//! WPM's 1–5 scale; the framework's wording informs, it does not decide.
//! Writes pass the worker-record authorization pass like any other change to
//! a worker's own record.

use loco_rs::prelude::*;
use sea_orm::{ActiveValue, TransactionTrait};
use serde::Deserialize;
use std::collections::BTreeMap;
use uuid::Uuid;

use super::esco::{Resolution, occupation_skills, pinned_version, resolve_catalogue_skill};
use super::{record_rejection, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::models::_entities::{
    esco_occupations, esco_skills, role_profiles, role_skill_requirements, skill_external_refs,
    skills, worker_framework_roles, worker_skills,
};
use crate::models::audit_logs::Model as Audit;
use crate::models::records;
use crate::rules::framework_roles::{self as rules, Choice};
use crate::rules::learning::valid_proficiency;
use crate::tasks::import_esco::ESCO_SLUG;
use crate::tasks::import_framework::PCF_SLUG;

/// A worker, authorized for a write to their own record.
async fn writable_worker(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: &str,
) -> Result<crate::models::_entities::workers::Model> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(pid)?).await?;
    auth::authorize_record(
        caller,
        authentication_verifier::Action::Write,
        &auth::worker_resource_attrs(&worker),
    )
    .map_err(record_rejection)?;
    Ok(worker)
}

fn check_framework(slug: &str) -> Result<()> {
    if rules::valid_framework(slug) {
        Ok(())
    } else {
        Err(unprocessable(&format!(
            "framework must be one of {}",
            rules::FRAMEWORKS.join(", ")
        )))
    }
}

fn selection_json(row: &worker_framework_roles::Model) -> serde_json::Value {
    serde_json::json!({
        "framework": row.framework_slug,
        "role_label": row.role_label,
        "role_profile_pid": row.role_profile_pid,
        "occupation_uri": row.esco_occupation_uri,
        "selected_on": row.selected_on,
    })
}

async fn current_selection(
    ctx: &AppContext,
    worker_pid: Uuid,
    framework: &str,
) -> Result<Option<worker_framework_roles::Model>> {
    Ok(worker_framework_roles::Entity::find()
        .filter(worker_framework_roles::Column::WorkerPid.eq(worker_pid))
        .filter(worker_framework_roles::Column::FrameworkSlug.eq(framework))
        .one(&ctx.db)
        .await?)
}

/// `GET /api/workers/{pid}/framework-roles` — the worker's current role in each
/// framework they have chosen one in.
#[debug_handler]
async fn list_selections(
    State(ctx): State<AppContext>,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    let rows = worker_framework_roles::Entity::find()
        .filter(worker_framework_roles::Column::WorkerPid.eq(worker.pid))
        .all(&ctx.db)
        .await?;
    format::json(rows.iter().map(selection_json).collect::<Vec<_>>())
}

/// `PUT /api/workers/{pid}/framework-roles/{framework}` body: a PCF role level
/// (`role_profile_pid`) or an ESCO occupation (`occupation_uri`).
#[derive(Debug, Deserialize)]
struct RolePayload {
    #[serde(default)]
    role_profile_pid: Option<Uuid>,
    #[serde(default)]
    occupation_uri: Option<String>,
}

/// `PUT /api/workers/{pid}/framework-roles/{framework}` — set the worker's
/// current role in a framework (replacing any previous one).
#[debug_handler]
async fn set_selection(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path((pid, framework)): Path<(String, String)>,
    Json(payload): Json<RolePayload>,
) -> Result<Response> {
    check_framework(&framework)?;
    let worker = writable_worker(&ctx, &caller, &pid).await?;
    let (profile_pid, occupation_uri, label) = if framework == PCF_SLUG {
        let profile_pid = payload
            .role_profile_pid
            .ok_or_else(|| unprocessable("role_profile_pid is required for the UK GDAD PCF"))?;
        let profile = role_profiles::Entity::find()
            .filter(role_profiles::Column::Pid.eq(profile_pid))
            .filter(role_profiles::Column::DeletedAt.is_null())
            .filter(role_profiles::Column::FrameworkSlug.eq(PCF_SLUG))
            .one(&ctx.db)
            .await?
            .ok_or_else(|| unprocessable("that is not a UK GDAD PCF role level"))?;
        (Some(profile.pid), None, profile.job_title)
    } else {
        let uri = payload
            .occupation_uri
            .ok_or_else(|| unprocessable("occupation_uri is required for ESCO"))?;
        let occupation = super::esco::find_occupation(&ctx, &uri)
            .await
            .map_err(|_| unprocessable("that is not an ESCO occupation in the pinned copy"))?;
        (None, Some(occupation.uri), occupation.label)
    };
    let txn = ctx.db.begin().await?;
    worker_framework_roles::Entity::delete_many()
        .filter(worker_framework_roles::Column::WorkerPid.eq(worker.pid))
        .filter(worker_framework_roles::Column::FrameworkSlug.eq(&framework))
        .exec(&txn)
        .await?;
    let row = worker_framework_roles::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        worker_pid: ActiveValue::set(worker.pid),
        framework_slug: ActiveValue::set(framework.clone()),
        role_profile_pid: ActiveValue::set(profile_pid),
        esco_occupation_uri: ActiveValue::set(occupation_uri),
        role_label: ActiveValue::set(label),
        selected_on: ActiveValue::set(chrono::Utc::now().date_naive()),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    Audit::record(
        &txn,
        "worker",
        worker.pid,
        "framework_role_selected",
        caller.actor(),
        Some(serde_json::json!({ "framework": framework, "role": row.role_label })),
    )
    .await?;
    txn.commit().await?;
    format::json(selection_json(&row))
}

/// `DELETE /api/workers/{pid}/framework-roles/{framework}` — clear the
/// selection. The skills the worker declared stay declared.
#[debug_handler]
async fn clear_selection(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path((pid, framework)): Path<(String, String)>,
) -> Result<Response> {
    check_framework(&framework)?;
    let worker = writable_worker(&ctx, &caller, &pid).await?;
    worker_framework_roles::Entity::delete_many()
        .filter(worker_framework_roles::Column::WorkerPid.eq(worker.pid))
        .filter(worker_framework_roles::Column::FrameworkSlug.eq(&framework))
        .exec(&ctx.db)
        .await?;
    Audit::record(
        &ctx.db,
        "worker",
        worker.pid,
        "framework_role_cleared",
        caller.actor(),
        None,
    )
    .await?;
    format::empty_json()
}

/// The worker's live declarations by catalogue skill.
async fn declared_levels(ctx: &AppContext, worker_pid: Uuid) -> Result<BTreeMap<Uuid, i32>> {
    Ok(worker_skills::Entity::find()
        .filter(worker_skills::Column::WorkerPid.eq(worker_pid))
        .filter(worker_skills::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|d| (d.skill_pid, d.proficiency))
        .collect())
}

/// `GET /api/workers/{pid}/framework-roles/{framework}/skills` — the selected
/// role's skills with what the worker has already declared:
///
/// - **PCF:** the role level's requirements, each with the framework's own
///   level and wording (a prompt, not a verdict).
/// - **ESCO:** the occupation's essential then optional skills.
///
/// `declared` is the worker's own level on WPM's 1–5 scale, or `null`.
#[debug_handler]
async fn role_skills(
    State(ctx): State<AppContext>,
    Path((pid, framework)): Path<(String, String)>,
) -> Result<Response> {
    check_framework(&framework)?;
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    let selection = current_selection(&ctx, worker.pid, &framework)
        .await?
        .ok_or_else(|| unprocessable("no role selected in this framework yet"))?;
    let declared = declared_levels(&ctx, worker.pid).await?;
    let out: Vec<serde_json::Value> = if framework == PCF_SLUG {
        let profile_pid = selection.role_profile_pid.ok_or(Error::NotFound)?;
        let requirements = role_skill_requirements::Entity::find()
            .filter(role_skill_requirements::Column::RoleProfilePid.eq(profile_pid))
            .all(&ctx.db)
            .await?;
        let names: BTreeMap<Uuid, String> = skills::Entity::find()
            .all(&ctx.db)
            .await?
            .into_iter()
            .map(|s| (s.pid, s.name))
            .collect();
        let mut rows: Vec<serde_json::Value> = requirements
            .iter()
            .map(|r| {
                serde_json::json!({
                    "ref": r.skill_pid,
                    "label": names.get(&r.skill_pid),
                    "role_expects": r.min_proficiency,
                    "framework_level": r.source_level,
                    "framework_scale_max": r.source_scale_max,
                    "importance": r.importance,
                    "wording": r.note,
                    "declared": declared.get(&r.skill_pid),
                })
            })
            .collect();
        rows.sort_by_key(|v| v["label"].as_str().map(str::to_lowercase));
        rows
    } else {
        let uri = selection
            .esco_occupation_uri
            .clone()
            .ok_or(Error::NotFound)?;
        let links: BTreeMap<String, Uuid> = skill_external_refs::Entity::find()
            .filter(skill_external_refs::Column::FrameworkSlug.eq(ESCO_SLUG))
            .all(&ctx.db)
            .await?
            .into_iter()
            .map(|r| (r.reference, r.skill_pid))
            .collect();
        occupation_skills(&ctx, &uri)
            .await?
            .iter()
            .map(|(relation, skill)| {
                serde_json::json!({
                    "ref": skill.uri,
                    "label": skill.label,
                    "relation": relation.relation,
                    "skill_type": skill.skill_type,
                    "declared": links.get(&skill.uri).and_then(|p| declared.get(p)),
                })
            })
            .collect()
    };
    format::json(serde_json::json!({
        "framework": framework,
        "role": selection_json(&selection),
        "skills": out,
    }))
}

/// One skill choice in the request body.
#[derive(Debug, Deserialize)]
struct ChoicePayload {
    #[serde(rename = "ref")]
    reference: String,
    /// WPM 1–5; `null` or absent removes the declaration.
    #[serde(default)]
    proficiency: Option<i32>,
}

/// `PUT /api/workers/{pid}/framework-roles/{framework}/skills` body.
#[derive(Debug, Deserialize)]
struct ChoicesPayload {
    selections: Vec<ChoicePayload>,
}

/// Declare (or clear) one worker declaration, the same upsert as
/// `PUT /api/workers/{pid}/skills`.
async fn apply_declaration(
    txn: &sea_orm::DatabaseTransaction,
    worker_pid: Uuid,
    skill_pid: Uuid,
    proficiency: Option<i32>,
) -> Result<bool> {
    let today = chrono::Utc::now().date_naive();
    let existing = worker_skills::Entity::find()
        .filter(worker_skills::Column::WorkerPid.eq(worker_pid))
        .filter(worker_skills::Column::SkillPid.eq(skill_pid))
        .one(txn)
        .await?;
    match (proficiency, existing) {
        (Some(level), Some(row)) => {
            let mut active: worker_skills::ActiveModel = row.into();
            active.proficiency = ActiveValue::set(level);
            active.assessed_on = ActiveValue::set(today);
            active.deleted_at = ActiveValue::set(None);
            active.update(txn).await?;
            Ok(true)
        }
        (Some(level), None) => {
            worker_skills::ActiveModel {
                pid: ActiveValue::set(Uuid::new_v4()),
                worker_pid: ActiveValue::set(worker_pid),
                skill_pid: ActiveValue::set(skill_pid),
                proficiency: ActiveValue::set(level),
                target: ActiveValue::set(None),
                assessed_on: ActiveValue::set(today),
                deleted_at: ActiveValue::set(None),
                ..Default::default()
            }
            .insert(txn)
            .await?;
            Ok(true)
        }
        (None, Some(row)) if row.deleted_at.is_none() => {
            let mut active: worker_skills::ActiveModel = row.into();
            active.deleted_at = ActiveValue::set(Some(chrono::Utc::now().into()));
            active.update(txn).await?;
            Ok(false)
        }
        (None, _) => Ok(false),
    }
}

/// `PUT /api/workers/{pid}/framework-roles/{framework}/skills` — select the
/// skills the worker has, at their own level; a `null` level deselects. All
/// or nothing. PCF refs are catalogue skill ids that carry a PCF reference;
/// ESCO refs are ESCO skill URIs, resolved to (or created and linked as)
/// catalogue skills.
#[debug_handler]
async fn set_skills(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path((pid, framework)): Path<(String, String)>,
    Json(payload): Json<ChoicesPayload>,
) -> Result<Response> {
    check_framework(&framework)?;
    let worker = writable_worker(&ctx, &caller, &pid).await?;
    if current_selection(&ctx, worker.pid, &framework)
        .await?
        .is_none()
    {
        return Err(unprocessable("select a role in this framework first"));
    }
    let choices: Vec<Choice> = payload
        .selections
        .iter()
        .map(|c| Choice {
            reference: c.reference.trim().to_string(),
            proficiency: c.proficiency,
        })
        .collect();
    rules::validate_choices(&choices).map_err(|e| unprocessable(&e))?;
    debug_assert!(
        choices
            .iter()
            .all(|c| c.proficiency.is_none_or(valid_proficiency))
    );
    let version = if framework == ESCO_SLUG {
        pinned_version(&ctx.db).await?
    } else {
        None
    };

    let txn = ctx.db.begin().await?;
    let (mut declared, mut removed, mut created, mut linked) = (0usize, 0usize, 0usize, 0usize);
    for choice in &choices {
        let skill_pid = if framework == PCF_SLUG {
            let pid = Uuid::parse_str(&choice.reference)
                .map_err(|_| unprocessable("a PCF ref must be a skill id"))?;
            let has_ref = skill_external_refs::Entity::find()
                .filter(skill_external_refs::Column::SkillPid.eq(pid))
                .filter(skill_external_refs::Column::FrameworkSlug.eq(PCF_SLUG))
                .one(&txn)
                .await?
                .is_some();
            if !has_ref {
                return Err(unprocessable("that skill is not a UK GDAD PCF skill"));
            }
            pid
        } else {
            let esco = esco_skills::Entity::find()
                .filter(esco_skills::Column::Uri.eq(&choice.reference))
                .one(&txn)
                .await?
                .ok_or_else(|| unprocessable("that is not an ESCO skill in the pinned copy"))?;
            if choice.proficiency.is_none() {
                // Deselecting: only a skill already linked can have a declaration.
                let link = skill_external_refs::Entity::find()
                    .filter(skill_external_refs::Column::FrameworkSlug.eq(ESCO_SLUG))
                    .filter(skill_external_refs::Column::Reference.eq(&esco.uri))
                    .one(&txn)
                    .await?;
                match link {
                    Some(l) => l.skill_pid,
                    None => continue,
                }
            } else {
                let (pid, how) = resolve_catalogue_skill(&txn, &esco, version.as_deref()).await?;
                match how {
                    Resolution::Created => created += 1,
                    Resolution::Linked => linked += 1,
                    Resolution::Existing => {}
                }
                pid
            }
        };
        if apply_declaration(&txn, worker.pid, skill_pid, choice.proficiency).await? {
            declared += 1;
        } else {
            removed += 1;
        }
    }
    Audit::record(
        &txn,
        "worker",
        worker.pid,
        "framework_skills_set",
        caller.actor(),
        Some(
            serde_json::json!({ "framework": framework, "declared": declared, "removed": removed }),
        ),
    )
    .await?;
    txn.commit().await?;
    format::json(serde_json::json!({
        "declared": declared,
        "removed": removed,
        "skills_created": created,
        "skills_linked": linked,
    }))
}

/// `GET /api/frameworks/selectable` — the frameworks a person can choose a
/// role in, with how much each has loaded.
#[debug_handler]
async fn selectable(State(ctx): State<AppContext>) -> Result<Response> {
    let pcf = role_profiles::Entity::find()
        .filter(role_profiles::Column::DeletedAt.is_null())
        .filter(role_profiles::Column::FrameworkSlug.eq(PCF_SLUG))
        .all(&ctx.db)
        .await?
        .len();
    let esco = esco_occupations::Entity::find().all(&ctx.db).await?.len();
    format::json(serde_json::json!([
        { "slug": PCF_SLUG, "name": "UK Government Digital and Data Profession Capability Framework", "roles": pcf, "available": pcf > 0 },
        { "slug": ESCO_SLUG, "name": "ESCO — European Skills, Competences, Qualifications and Occupations", "roles": esco, "available": esco > 0 },
    ]))
}

/// The framework-role routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/frameworks/selectable", get(selectable))
        .add("/workers/{pid}/framework-roles", get(list_selections))
        .add(
            "/workers/{pid}/framework-roles/{framework}",
            put(set_selection),
        )
        .add(
            "/workers/{pid}/framework-roles/{framework}",
            delete(clear_selection),
        )
        .add(
            "/workers/{pid}/framework-roles/{framework}/skills",
            get(role_skills),
        )
        .add(
            "/workers/{pid}/framework-roles/{framework}/skills",
            put(set_skills),
        )
}
