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

use super::writable_worker;
use axa::extract::{Json, Path};
use axa::prelude::*;
use serde::Deserialize;
use std::collections::BTreeMap;
use uuid::Uuid;

use super::esco::{Resolution, occupation_skills};
use super::unprocessable;
use crate::auth::{self, MaybeAuthUser};
use crate::db;
use crate::models::_entities::{esco_skills, worker_framework_roles};
use crate::models::records;
use crate::rules::esco as esco_rules;
use crate::rules::framework_roles::{self as rules, Choice};
use crate::rules::learning::valid_proficiency;
use crate::tasks::import_esco::ESCO_SLUG;
use crate::tasks::import_framework::PCF_SLUG;

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
        "started_at": row.started_at,
        "ended_at": row.ended_at,
        "recorded_by": row.recorded_by,
        "on_behalf": row.on_behalf,
    })
}

async fn current_selection(
    ctx: &AppContext,
    worker_pid: Uuid,
    framework: &str,
) -> Result<Option<worker_framework_roles::Model>> {
    Ok(sqlx::query_as!(
        worker_framework_roles::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, framework_slug,
                  role_profile_pid, esco_occupation_uri, role_label, selected_on, started_at AS "started_at: _",
                  ended_at AS "ended_at: _", recorded_by, on_behalf
             FROM worker_framework_roles
            WHERE worker_pid = $1 AND framework_slug = $2 AND ended_at IS NULL
            LIMIT 1"#,
        worker_pid,
        framework,
    )
    .fetch_optional(db::pool(ctx))
    .await?)
}

/// `GET /api/workers/{pid}/framework-roles` — the worker's current role in each
/// framework they have chosen one in.
#[debug_handler]
async fn list_selections(
    State(ctx): State<AppContext>,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    let rows = sqlx::query_as!(
        worker_framework_roles::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, framework_slug,
                  role_profile_pid, esco_occupation_uri, role_label, selected_on, started_at AS "started_at: _",
                  ended_at AS "ended_at: _", recorded_by, on_behalf
             FROM worker_framework_roles
            WHERE worker_pid = $1 AND ended_at IS NULL
            ORDER BY id"#,
        worker.pid,
    )
    .fetch_all(db::pool(&ctx))
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
        let profile = sqlx::query!(
            "SELECT pid, job_title FROM role_profiles WHERE pid = $1 AND deleted_at IS NULL AND framework_slug = $2 LIMIT 1",
            profile_pid,
            PCF_SLUG,
        )
        .fetch_optional(db::pool(&ctx))
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
    let on_behalf = auth::acting_for_other(&caller, &worker.person_ref);
    // Choosing the role they already hold changes nothing.
    if let Some(current) = current_selection(&ctx, worker.pid, &framework).await?
        && current.role_profile_pid == profile_pid
        && current.esco_occupation_uri == occupation_uri
    {
        return format::json(selection_json(&current));
    }
    let mut tx = db::pool(&ctx).begin().await?;
    // The previous role ends now — that is what makes it a *past* role.
    close_current(&mut tx, worker.pid, &framework).await?;
    let now: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();
    let row = sqlx::query_as!(
        worker_framework_roles::Model,
        r#"INSERT INTO worker_framework_roles (pid, worker_pid, framework_slug, role_profile_pid, esco_occupation_uri, role_label,
                                             selected_on, started_at, recorded_by, on_behalf)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
           RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, framework_slug,
                     role_profile_pid, esco_occupation_uri, role_label, selected_on, started_at AS "started_at: _",
                     ended_at AS "ended_at: _", recorded_by, on_behalf"#,
        Uuid::new_v4(),
        worker.pid,
        framework,
        profile_pid,
        occupation_uri,
        label,
        chrono::Utc::now().date_naive(),
        now,
        caller.actor(),
        on_behalf,
    )
    .fetch_one(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "worker",
        worker.pid,
        "framework_role_selected",
        caller.actor(),
        Some(serde_json::json!({ "framework": framework, "role": row.role_label, "on_behalf": on_behalf })),
    )
    .await?;
    tx.commit().await?;
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
    let mut tx = db::pool(&ctx).begin().await?;
    close_current(&mut tx, worker.pid, &framework).await?;
    db::audit(
        &mut tx,
        "worker",
        worker.pid,
        "framework_role_cleared",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::empty_json()
}

/// End the worker's current role in a framework, if they have one.
async fn close_current(
    tx: &mut sqlx::PgConnection,
    worker_pid: Uuid,
    framework: &str,
) -> Result<()> {
    if let Some(id) = sqlx::query_scalar!(
        "SELECT id FROM worker_framework_roles WHERE worker_pid = $1 AND framework_slug = $2 AND ended_at IS NULL LIMIT 1",
        worker_pid,
        framework,
    )
    .fetch_optional(&mut *tx)
    .await?
    {
        sqlx::query!(
            "UPDATE worker_framework_roles SET ended_at = $2 WHERE id = $1",
            id,
            chrono::DateTime::<chrono::FixedOffset>::from(chrono::Utc::now()),
        )
        .execute(&mut *tx)
        .await?;
    }
    Ok(())
}

/// The worker's live declarations by catalogue skill.
async fn declared_levels(ctx: &AppContext, worker_pid: Uuid) -> Result<BTreeMap<Uuid, i32>> {
    Ok(sqlx::query!(
        "SELECT skill_pid, proficiency FROM worker_skills WHERE worker_pid = $1 AND deleted_at IS NULL ORDER BY id",
        worker_pid,
    )
    .fetch_all(db::pool(ctx))
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
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    let selection = current_selection(&ctx, worker.pid, &framework)
        .await?
        .ok_or_else(|| unprocessable("no role selected in this framework yet"))?;
    let declared = declared_levels(&ctx, worker.pid).await?;
    let out: Vec<serde_json::Value> = if framework == PCF_SLUG {
        let profile_pid = selection.role_profile_pid.ok_or(Error::NotFound)?;
        // Each requirement with its skill's name (any skill row, as the catalogue names it).
        let requirements = sqlx::query!(
            r#"SELECT r.skill_pid, s.name AS "name?", r.min_proficiency, r.source_level, r.source_scale_max, r.importance,
                      r.note
                 FROM role_skill_requirements r
                 LEFT JOIN skills s ON s.pid = r.skill_pid
                WHERE r.role_profile_pid = $1
                ORDER BY r.id"#,
            profile_pid,
        )
        .fetch_all(db::pool(&ctx))
        .await?;
        let mut rows: Vec<serde_json::Value> = requirements
            .iter()
            .map(|r| {
                serde_json::json!({
                    "ref": r.skill_pid,
                    "label": r.name,
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
        let links: BTreeMap<String, Uuid> = sqlx::query!(
            r#"SELECT "ref" AS reference, skill_pid FROM skill_external_refs WHERE framework_slug = $1 ORDER BY id"#,
            ESCO_SLUG,
        )
        .fetch_all(db::pool(&ctx))
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
    tx: &mut sqlx::PgConnection,
    worker_pid: Uuid,
    skill_pid: Uuid,
    proficiency: Option<i32>,
    actor: Option<&str>,
    on_behalf: bool,
) -> Result<bool> {
    crate::models::skill_history::record_in(
        &mut *tx,
        worker_pid,
        skill_pid,
        proficiency,
        "framework",
        actor,
        on_behalf,
    )
    .await?;
    let today = chrono::Utc::now().date_naive();
    let existing = sqlx::query!(
        r#"SELECT id, deleted_at AS "deleted_at: chrono::DateTime<chrono::FixedOffset>"
             FROM worker_skills WHERE worker_pid = $1 AND skill_pid = $2 LIMIT 1"#,
        worker_pid,
        skill_pid,
    )
    .fetch_optional(&mut *tx)
    .await?;
    match (proficiency, existing) {
        (Some(level), Some(row)) => {
            sqlx::query!(
                "UPDATE worker_skills SET proficiency = $2, assessed_on = $3, deleted_at = NULL WHERE id = $1",
                row.id,
                level,
                today,
            )
            .execute(&mut *tx)
            .await?;
            Ok(true)
        }
        (Some(level), None) => {
            sqlx::query!(
                "INSERT INTO worker_skills (pid, worker_pid, skill_pid, proficiency, assessed_on) VALUES ($1, $2, $3, $4, $5)",
                Uuid::new_v4(),
                worker_pid,
                skill_pid,
                level,
                today,
            )
            .execute(&mut *tx)
            .await?;
            Ok(true)
        }
        (None, Some(row)) if row.deleted_at.is_none() => {
            sqlx::query!(
                "UPDATE worker_skills SET deleted_at = $2 WHERE id = $1",
                row.id,
                chrono::DateTime::<chrono::FixedOffset>::from(chrono::Utc::now()),
            )
            .execute(&mut *tx)
            .await?;
            Ok(false)
        }
        (None, _) => Ok(false),
    }
}

/// The ESCO version pinned by the last import, from the framework row's note: the sqlx form of
/// [`super::esco::pinned_version`], kept here while that file is on `SeaORM` (WPM-T466).
async fn pinned_esco_version(ctx: &AppContext) -> Result<Option<String>> {
    Ok(sqlx::query_scalar!(
        "SELECT note FROM capability_frameworks WHERE slug = $1 LIMIT 1",
        ESCO_SLUG,
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .flatten()
    .and_then(|n| {
        n.rsplit("Pinned version: ")
            .next()
            .map(|v| v.trim_end_matches('.').to_string())
    }))
}

/// Resolve an ESCO skill to a catalogue skill on `tx`: by its ESCO reference, else by exact normalised label (when
/// unambiguous and not already linked to another ESCO skill), else create one with a draft category — and link it. The
/// sqlx form of [`super::esco::resolve_catalogue_skill`], kept here while that file is on `SeaORM` (WPM-T466).
async fn resolve_catalogue_skill(
    tx: &mut sqlx::PgConnection,
    esco_skill: &esco_skills::Model,
    version: Option<&str>,
) -> Result<(Uuid, Resolution)> {
    if let Some(skill_pid) = sqlx::query_scalar!(
        r#"SELECT skill_pid FROM skill_external_refs WHERE framework_slug = $1 AND "ref" = $2 LIMIT 1"#,
        ESCO_SLUG,
        esco_skill.uri,
    )
    .fetch_optional(&mut *tx)
    .await?
    {
        return Ok((skill_pid, Resolution::Existing));
    }
    let wanted_label = esco_rules::normalise_label(&esco_skill.label);
    let candidates: Vec<Uuid> =
        sqlx::query!("SELECT pid, name FROM skills WHERE deleted_at IS NULL ORDER BY id")
            .fetch_all(&mut *tx)
            .await?
            .into_iter()
            .filter(|s| esco_rules::normalise_label(&s.name) == wanted_label)
            .map(|s| s.pid)
            .collect();
    let one_unlinked = match candidates.as_slice() {
        [one] => {
            let linked = sqlx::query_scalar!(
                r#"SELECT 1 AS "one!" FROM skill_external_refs WHERE skill_pid = $1 AND framework_slug = $2 LIMIT 1"#,
                one,
                ESCO_SLUG,
            )
            .fetch_optional(&mut *tx)
            .await?
            .is_some();
            (!linked).then_some(*one)
        }
        _ => None,
    };
    let (pid, created) = if let Some(one) = one_unlinked {
        (one, false)
    } else {
        let name = unique_skill_name(tx, &esco_skill.label).await?;
        let pid = Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO skills (pid, name, category) VALUES ($1, $2, $3)",
            pid,
            name,
            esco_rules::category_for(
                esco_skill.skill_type.as_deref(),
                esco_skill.reuse_level.as_deref(),
            ),
        )
        .execute(&mut *tx)
        .await?;
        (pid, true)
    };
    sqlx::query!(
        r#"INSERT INTO skill_external_refs (pid, skill_pid, framework_slug, "ref", label, version) VALUES ($1, $2, $3, $4, $5, $6)"#,
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

/// A catalogue name not yet taken: the label, else the label with ` (ESCO)`.
async fn unique_skill_name(tx: &mut sqlx::PgConnection, label: &str) -> Result<String> {
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

/// `PUT /api/workers/{pid}/framework-roles/{framework}/skills` — select the
/// skills the worker has, at their own level; a `null` level deselects. All
/// or nothing. PCF refs are catalogue skill ids that carry a PCF reference;
/// ESCO refs are ESCO skill URIs, resolved to (or created and linked as)
/// catalogue skills.
#[debug_handler]
#[allow(clippy::too_many_lines)] // validate → resolve → declare, per framework
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
    let on_behalf = auth::acting_for_other(&caller, &worker.person_ref);
    let version = if framework == ESCO_SLUG {
        pinned_esco_version(&ctx).await?
    } else {
        None
    };

    let mut tx = db::pool(&ctx).begin().await?;
    let (mut declared, mut removed, mut created, mut linked) = (0usize, 0usize, 0usize, 0usize);
    for choice in &choices {
        let skill_pid = if framework == PCF_SLUG {
            let pid = Uuid::parse_str(&choice.reference)
                .map_err(|_| unprocessable("a PCF ref must be a skill id"))?;
            let has_ref = sqlx::query_scalar!(
                r#"SELECT 1 AS "one!" FROM skill_external_refs WHERE skill_pid = $1 AND framework_slug = $2 LIMIT 1"#,
                pid,
                PCF_SLUG,
            )
            .fetch_optional(&mut *tx)
            .await?
            .is_some();
            if !has_ref {
                return Err(unprocessable("that skill is not a UK GDAD PCF skill"));
            }
            pid
        } else {
            let esco = sqlx::query_as!(
                esco_skills::Model,
                r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, uri, label, skill_type, reuse_level
                     FROM esco_skills WHERE uri = $1 LIMIT 1"#,
                choice.reference,
            )
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| unprocessable("that is not an ESCO skill in the pinned copy"))?;
            if choice.proficiency.is_none() {
                // Deselecting: only a skill already linked can have a declaration.
                let link = sqlx::query_scalar!(
                    r#"SELECT skill_pid FROM skill_external_refs WHERE framework_slug = $1 AND "ref" = $2 LIMIT 1"#,
                    ESCO_SLUG,
                    esco.uri,
                )
                .fetch_optional(&mut *tx)
                .await?;
                match link {
                    Some(skill_pid) => skill_pid,
                    None => continue,
                }
            } else {
                let (pid, how) =
                    resolve_catalogue_skill(&mut tx, &esco, version.as_deref()).await?;
                match how {
                    Resolution::Created => created += 1,
                    Resolution::Linked => linked += 1,
                    Resolution::Existing => {}
                }
                pid
            }
        };
        if apply_declaration(
            &mut tx,
            worker.pid,
            skill_pid,
            choice.proficiency,
            caller.actor(),
            on_behalf,
        )
        .await?
        {
            declared += 1;
        } else {
            removed += 1;
        }
    }
    db::audit(
        &mut tx,
        "worker",
        worker.pid,
        "framework_skills_set",
        caller.actor(),
        Some(
            serde_json::json!({ "framework": framework, "declared": declared, "removed": removed }),
        ),
    )
    .await?;
    tx.commit().await?;
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
    let pcf = sqlx::query_scalar!(
        r#"SELECT count(*) AS "n!" FROM role_profiles WHERE deleted_at IS NULL AND framework_slug = $1"#,
        PCF_SLUG,
    )
    .fetch_one(db::pool(&ctx))
    .await?;
    let esco = sqlx::query_scalar!(r#"SELECT count(*) AS "n!" FROM esco_occupations"#)
        .fetch_one(db::pool(&ctx))
        .await?;
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
