//! Learning & development — the skills framework (catalog + declared
//! worker proficiency), learning paths (ordered course steps +
//! per-worker enrolment with honest progress), and mentorships
//! (lifecycle + session log). Declared/recorded data plus derived
//! views; the pure rules live in [`crate::rules::learning`], and the
//! progress derivation counts only real course completions.

use loco_rs::prelude::*;
use sea_orm::{ActiveValue, PaginatorTrait, QueryOrder, TransactionTrait};
use serde::Deserialize;
use uuid::Uuid;

use super::{ensure_valid, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::models::_entities::{
    development_plan_items, initiative_skill_shifts, learning_path_steps, learning_paths,
    mentorship_sessions, mentorships, path_enrollments, role_skill_requirements,
    skill_external_refs, skills, training_enrollments, worker_skills, workers,
};
use crate::models::audit_logs::Model as Audit;
use crate::models::records;
use crate::rules::learning as rules;
use crate::rules::skill_merge as merge_rules;
use crate::validation::Problems;

/// A `{pid}` reference response.
#[derive(serde::Serialize)]
struct PidRef {
    pid: String,
}

impl PidRef {
    fn of(pid: Uuid) -> Self {
        Self {
            pid: pid.to_string(),
        }
    }
}

// ─── Skills ─────────────────────────────────────────────────────────────────

/// `POST /api/skills` body.
#[derive(Debug, Deserialize)]
struct SkillPayload {
    name: String,
    category: String,
}

/// `POST /api/skills` — add a skill to the catalog.
#[debug_handler]
async fn create_skill(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<SkillPayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_text("name", &payload.name);
    problems.cap_text("name", &payload.name);
    problems.require_token("category", rules::SKILL_CATEGORIES, &payload.category);
    ensure_valid(&problems.into_vec())?;
    let row = skills::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        name: ActiveValue::set(payload.name.clone()),
        category: ActiveValue::set(payload.category.clone()),
        deleted_at: ActiveValue::set(None),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;
    Audit::record(&ctx.db, "skill", row.pid, "created", caller.actor(), None).await?;
    format::json(PidRef::of(row.pid))
}

/// `GET /api/skills` — the catalog, each skill with the external-framework
/// references it carries (PCF name, ESCO concept URI).
#[debug_handler]
async fn list_skills(State(ctx): State<AppContext>) -> Result<Response> {
    let rows = skills::Entity::find()
        .filter(skills::Column::DeletedAt.is_null())
        .order_by_asc(skills::Column::Name)
        .all(&ctx.db)
        .await?;
    let refs = skill_external_refs::Entity::find().all(&ctx.db).await?;
    let out: Vec<serde_json::Value> = rows
        .iter()
        .map(|s| {
            let mine: Vec<serde_json::Value> = refs
                .iter()
                .filter(|r| r.skill_pid == s.pid)
                .map(|r| {
                    serde_json::json!({
                        "framework": r.framework_slug,
                        "ref": r.reference,
                        "label": r.label,
                        "version": r.version,
                    })
                })
                .collect();
            serde_json::json!({
                "pid": s.pid,
                "name": s.name,
                "category": s.category,
                "external_refs": mine,
            })
        })
        .collect();
    format::json(out)
}

/// `PUT /api/skills/{pid}` body — rename and/or recategorise.
#[derive(Debug, Deserialize)]
struct SkillUpdate {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    category: Option<String>,
}

/// `PUT /api/skills/{pid}` — rename and/or recategorise a catalogue skill.
/// Names are unique; a rename never breaks a framework import, which
/// matches by reference first.
#[debug_handler]
async fn update_skill(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<SkillUpdate>,
) -> Result<Response> {
    let mut problems = Problems::new();
    if let Some(category) = &payload.category {
        problems.require_token("category", rules::SKILL_CATEGORIES, category);
    }
    ensure_valid(&problems.into_vec())?;
    let row = find_live_skill(&ctx, &pid).await?;
    let mut active: skills::ActiveModel = row.clone().into();
    if let Some(name) = &payload.name {
        let name = rules::normalize_skill_name(name).map_err(|e| unprocessable(&e))?;
        if name != row.name {
            let taken = skills::Entity::find()
                .filter(skills::Column::Name.eq(&name))
                .one(&ctx.db)
                .await?
                .is_some();
            if taken {
                return Err(unprocessable("a skill with that name already exists"));
            }
            active.name = ActiveValue::set(name);
        }
    }
    if let Some(category) = &payload.category {
        active.category = ActiveValue::set(category.clone());
    }
    let updated = active.update(&ctx.db).await?;
    Audit::record(
        &ctx.db,
        "skill",
        updated.pid,
        "updated",
        caller.actor(),
        Some(serde_json::json!({ "name": updated.name, "category": updated.category })),
    )
    .await?;
    format::json(PidRef::of(updated.pid))
}

async fn find_live_skill(ctx: &AppContext, pid: &str) -> Result<skills::Model> {
    skills::Entity::find()
        .filter(skills::Column::Pid.eq(records::parse_pid(pid)?))
        .filter(skills::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)
}

/// `GET /api/skills/category-suggestions` — skills still `other` for which a
/// keyword rule suggests a category, with the keyword that triggered it.
/// Suggestions only: nothing changes until a planner applies them.
#[debug_handler]
async fn category_suggestions(State(ctx): State<AppContext>) -> Result<Response> {
    let rows = skills::Entity::find()
        .filter(skills::Column::DeletedAt.is_null())
        .filter(skills::Column::Category.eq("other"))
        .order_by_asc(skills::Column::Name)
        .all(&ctx.db)
        .await?;
    let out: Vec<serde_json::Value> = rows
        .iter()
        .filter_map(|s| {
            rules::suggest_category(&s.name).map(|(category, keyword)| {
                serde_json::json!({
                    "pid": s.pid,
                    "name": s.name,
                    "suggested": category,
                    "keyword": keyword,
                })
            })
        })
        .collect();
    format::json(serde_json::json!({
        "derivation": "ordered keyword rules over the skill name (compliance, leadership, \
                       technical, domain); a suggestion for a planner to accept or correct, \
                       never applied automatically",
        "suggestions": out,
        "unsuggested": rows.len() - out.len(),
    }))
}

/// `POST /api/skills/category-suggestions/apply` body.
#[derive(Debug, Deserialize)]
struct ApplySuggestions {
    skill_pids: Vec<Uuid>,
}

/// `POST /api/skills/category-suggestions/apply` — apply the **current**
/// suggestion to the chosen skills (still `other`); skills with no
/// suggestion, or no longer `other`, are left alone and counted.
#[debug_handler]
async fn apply_suggestions(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<ApplySuggestions>,
) -> Result<Response> {
    let (mut applied, mut skipped) = (0usize, 0usize);
    for pid in &payload.skill_pids {
        let found = skills::Entity::find()
            .filter(skills::Column::Pid.eq(*pid))
            .filter(skills::Column::DeletedAt.is_null())
            .one(&ctx.db)
            .await?;
        let suggestion = found
            .as_ref()
            .filter(|s| s.category == "other")
            .and_then(|s| rules::suggest_category(&s.name));
        match (found, suggestion) {
            (Some(row), Some((category, _))) => {
                let mut active: skills::ActiveModel = row.into();
                active.category = ActiveValue::set(category.to_string());
                let updated = active.update(&ctx.db).await?;
                Audit::record(
                    &ctx.db,
                    "skill",
                    updated.pid,
                    "categorised",
                    caller.actor(),
                    Some(serde_json::json!({ "category": category })),
                )
                .await?;
                applied += 1;
            }
            _ => skipped += 1,
        }
    }
    format::json(serde_json::json!({ "applied": applied, "skipped": skipped }))
}

/// Count where a skill is used.
async fn skill_usage<C: ConnectionTrait>(db: &C, skill_pid: Uuid) -> Result<merge_rules::Usage> {
    let count = |n: u64| usize::try_from(n).unwrap_or(usize::MAX);
    Ok(merge_rules::Usage {
        declared_by: count(
            worker_skills::Entity::find()
                .filter(worker_skills::Column::SkillPid.eq(skill_pid))
                .filter(worker_skills::Column::DeletedAt.is_null())
                .count(db)
                .await?,
        ),
        required_by_profiles: count(
            role_skill_requirements::Entity::find()
                .filter(role_skill_requirements::Column::SkillPid.eq(skill_pid))
                .count(db)
                .await?,
        ),
        in_development_plans: count(
            development_plan_items::Entity::find()
                .filter(development_plan_items::Column::SkillPid.eq(skill_pid))
                .count(db)
                .await?,
        ),
        in_initiatives: count(
            initiative_skill_shifts::Entity::find()
                .filter(initiative_skill_shifts::Column::SkillPid.eq(skill_pid))
                .count(db)
                .await?,
        ),
    })
}

fn usage_json(u: &merge_rules::Usage) -> serde_json::Value {
    serde_json::json!({
        "declared_by": u.declared_by,
        "required_by_profiles": u.required_by_profiles,
        "in_development_plans": u.in_development_plans,
        "in_initiatives": u.in_initiatives,
        "total": u.total(),
        "deletable": u.deletable(),
    })
}

/// `GET /api/skills/{pid}/usage` — where a skill is used, and whether it can
/// be deleted (only when nothing uses it; otherwise merge it).
#[debug_handler]
async fn skill_usage_view(
    State(ctx): State<AppContext>,
    Path(pid): Path<String>,
) -> Result<Response> {
    let skill = find_live_skill(&ctx, &pid).await?;
    format::json(usage_json(&skill_usage(&ctx.db, skill.pid).await?))
}

/// `DELETE /api/skills/{pid}` — delete a skill **nothing uses** (soft-delete;
/// its external references go with it). A skill in use is refused with `422`:
/// merge it into another instead.
#[debug_handler]
async fn delete_skill(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let skill = find_live_skill(&ctx, &pid).await?;
    let usage = skill_usage(&ctx.db, skill.pid).await?;
    if !usage.deletable() {
        return Err(unprocessable(&format!(
            "the skill is in use ({} place(s)); merge it into another skill instead",
            usage.total()
        )));
    }
    let txn = ctx.db.begin().await?;
    skill_external_refs::Entity::delete_many()
        .filter(skill_external_refs::Column::SkillPid.eq(skill.pid))
        .exec(&txn)
        .await?;
    let mut active: skills::ActiveModel = skill.into();
    active.deleted_at = ActiveValue::set(Some(chrono::Utc::now().into()));
    let row = active.update(&txn).await?;
    Audit::record(&txn, "skill", row.pid, "deleted", caller.actor(), None).await?;
    txn.commit().await?;
    format::empty_json()
}

/// `POST /api/skills/{pid}/merge` body.
#[derive(Debug, Deserialize)]
struct MergePayload {
    into_pid: Uuid,
}

/// `POST /api/skills/{pid}/merge` — fold this skill into another and retire
/// it. Everything pointing at it is repointed at the target; where both
/// already appear (a worker who declared both, a profile requiring both, a
/// plan listing both) the stronger statement is kept. One transaction.
#[debug_handler]
#[allow(clippy::too_many_lines)] // one repointing pass per referencing table
async fn merge_skill(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<MergePayload>,
) -> Result<Response> {
    let source = find_live_skill(&ctx, &pid).await?;
    if source.pid == payload.into_pid {
        return Err(unprocessable("a skill cannot be merged into itself"));
    }
    let target = find_live_skill(&ctx, &payload.into_pid.to_string()).await?;
    let txn = ctx.db.begin().await?;
    let (mut merged, mut moved) = (0usize, 0usize);

    // Worker declarations.
    for row in worker_skills::Entity::find()
        .filter(worker_skills::Column::SkillPid.eq(source.pid))
        .all(&txn)
        .await?
    {
        let twin = worker_skills::Entity::find()
            .filter(worker_skills::Column::WorkerPid.eq(row.worker_pid))
            .filter(worker_skills::Column::SkillPid.eq(target.pid))
            .one(&txn)
            .await?;
        if let Some(twin) = twin {
            let mut active: worker_skills::ActiveModel = twin.clone().into();
            active.proficiency =
                ActiveValue::set(merge_rules::merge_level(twin.proficiency, row.proficiency));
            active.target = ActiveValue::set(merge_rules::merge_target(twin.target, row.target));
            active.assessed_on = ActiveValue::set(twin.assessed_on.max(row.assessed_on));
            if row.deleted_at.is_none() {
                active.deleted_at = ActiveValue::set(None);
            }
            active.update(&txn).await?;
            row.delete(&txn).await?;
            merged += 1;
        } else {
            let mut active: worker_skills::ActiveModel = row.into();
            active.skill_pid = ActiveValue::set(target.pid);
            active.update(&txn).await?;
            moved += 1;
        }
    }
    // Role requirements.
    for row in role_skill_requirements::Entity::find()
        .filter(role_skill_requirements::Column::SkillPid.eq(source.pid))
        .all(&txn)
        .await?
    {
        let twin = role_skill_requirements::Entity::find()
            .filter(role_skill_requirements::Column::RoleProfilePid.eq(row.role_profile_pid))
            .filter(role_skill_requirements::Column::SkillPid.eq(target.pid))
            .one(&txn)
            .await?;
        if let Some(twin) = twin {
            let mut active: role_skill_requirements::ActiveModel = twin.clone().into();
            active.min_proficiency = ActiveValue::set(merge_rules::merge_level(
                twin.min_proficiency,
                row.min_proficiency,
            ));
            active.importance = ActiveValue::set(
                merge_rules::stronger_importance(&twin.importance, &row.importance).to_string(),
            );
            if twin.note.is_none() {
                active.note = ActiveValue::set(row.note.clone());
            }
            if twin.source_level.is_none() {
                active.source_level = ActiveValue::set(row.source_level);
                active.source_scale_max = ActiveValue::set(row.source_scale_max);
            }
            active.update(&txn).await?;
            row.delete(&txn).await?;
            merged += 1;
        } else {
            let mut active: role_skill_requirements::ActiveModel = row.into();
            active.skill_pid = ActiveValue::set(target.pid);
            active.update(&txn).await?;
            moved += 1;
        }
    }
    // Initiative skill shifts: the target's statement wins a conflict.
    for row in initiative_skill_shifts::Entity::find()
        .filter(initiative_skill_shifts::Column::SkillPid.eq(source.pid))
        .all(&txn)
        .await?
    {
        let twin_exists = initiative_skill_shifts::Entity::find()
            .filter(initiative_skill_shifts::Column::InitiativePid.eq(row.initiative_pid))
            .filter(initiative_skill_shifts::Column::SkillPid.eq(target.pid))
            .one(&txn)
            .await?
            .is_some();
        if twin_exists {
            row.delete(&txn).await?;
            merged += 1;
        } else {
            let mut active: initiative_skill_shifts::ActiveModel = row.into();
            active.skill_pid = ActiveValue::set(target.pid);
            active.update(&txn).await?;
            moved += 1;
        }
    }
    // Development-plan items: the target's item wins a conflict.
    for row in development_plan_items::Entity::find()
        .filter(development_plan_items::Column::SkillPid.eq(source.pid))
        .all(&txn)
        .await?
    {
        let twin_exists = development_plan_items::Entity::find()
            .filter(development_plan_items::Column::PlanPid.eq(row.plan_pid))
            .filter(development_plan_items::Column::SkillPid.eq(target.pid))
            .one(&txn)
            .await?
            .is_some();
        if twin_exists {
            row.delete(&txn).await?;
            merged += 1;
        } else {
            let mut active: development_plan_items::ActiveModel = row.into();
            active.skill_pid = ActiveValue::set(target.pid);
            active.update(&txn).await?;
            moved += 1;
        }
    }
    // External references: keep the target's per framework, move the rest.
    for row in skill_external_refs::Entity::find()
        .filter(skill_external_refs::Column::SkillPid.eq(source.pid))
        .all(&txn)
        .await?
    {
        let has = skill_external_refs::Entity::find()
            .filter(skill_external_refs::Column::SkillPid.eq(target.pid))
            .filter(skill_external_refs::Column::FrameworkSlug.eq(&row.framework_slug))
            .one(&txn)
            .await?
            .is_some();
        if has {
            row.delete(&txn).await?;
        } else {
            let mut active: skill_external_refs::ActiveModel = row.into();
            active.skill_pid = ActiveValue::set(target.pid);
            active.update(&txn).await?;
        }
    }
    let source_name = source.name.clone();
    let mut retire: skills::ActiveModel = source.into();
    retire.deleted_at = ActiveValue::set(Some(chrono::Utc::now().into()));
    let retired = retire.update(&txn).await?;
    Audit::record(
        &txn,
        "skill",
        retired.pid,
        "merged",
        caller.actor(),
        Some(serde_json::json!({
            "into": target.pid, "from_name": source_name, "records_moved": moved, "records_merged": merged,
        })),
    )
    .await?;
    txn.commit().await?;
    format::json(serde_json::json!({
        "merged_into": target.pid,
        "records_moved": moved,
        "records_merged": merged,
    }))
}

/// `POST /api/skills/{pid}/refs` body.
#[derive(Debug, Deserialize)]
struct SkillRefPayload {
    framework_slug: String,
    #[serde(rename = "ref")]
    reference: String,
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    version: Option<String>,
}

/// `POST /api/skills/{pid}/refs` — record how this skill is known in an
/// external framework (e.g. an ESCO concept URI). One per skill per
/// framework; a reference names one skill.
#[debug_handler]
async fn add_skill_ref(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<SkillRefPayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_text("framework_slug", &payload.framework_slug);
    problems.require_text("ref", &payload.reference);
    problems.cap_text("ref", &payload.reference);
    ensure_valid(&problems.into_vec())?;
    let skill = find_live_skill(&ctx, &pid).await?;
    // A reference into ESCO must name a concept in the pinned local copy
    // (when one has been loaded).
    if payload.framework_slug.trim() == "esco" {
        let loaded = crate::models::_entities::esco_skills::Entity::find()
            .one(&ctx.db)
            .await?
            .is_some();
        let known = crate::models::_entities::esco_skills::Entity::find()
            .filter(crate::models::_entities::esco_skills::Column::Uri.eq(&payload.reference))
            .one(&ctx.db)
            .await?
            .is_some();
        if loaded && !known {
            return Err(unprocessable(
                "that URI is not an ESCO skill in the pinned copy",
            ));
        }
    }
    let clash = skill_external_refs::Entity::find()
        .filter(skill_external_refs::Column::FrameworkSlug.eq(&payload.framework_slug))
        .filter(
            sea_orm::Condition::any()
                .add(skill_external_refs::Column::SkillPid.eq(skill.pid))
                .add(skill_external_refs::Column::Reference.eq(&payload.reference)),
        )
        .one(&ctx.db)
        .await?
        .is_some();
    if clash {
        return Err(unprocessable(
            "this skill already has a reference in that framework, or the reference names another skill",
        ));
    }
    let row = skill_external_refs::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        skill_pid: ActiveValue::set(skill.pid),
        framework_slug: ActiveValue::set(payload.framework_slug.trim().to_string()),
        reference: ActiveValue::set(payload.reference.clone()),
        label: ActiveValue::set(payload.label.clone()),
        version: ActiveValue::set(payload.version.clone()),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;
    Audit::record(
        &ctx.db,
        "skill",
        skill.pid,
        "reference_added",
        caller.actor(),
        None,
    )
    .await?;
    format::json(PidRef::of(row.pid))
}

/// `DELETE /api/skills/{pid}/refs/{framework_slug}`.
#[debug_handler]
async fn remove_skill_ref(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path((pid, framework)): Path<(String, String)>,
) -> Result<Response> {
    let skill = find_live_skill(&ctx, &pid).await?;
    let row = skill_external_refs::Entity::find()
        .filter(skill_external_refs::Column::SkillPid.eq(skill.pid))
        .filter(skill_external_refs::Column::FrameworkSlug.eq(&framework))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    row.delete(&ctx.db).await?;
    Audit::record(
        &ctx.db,
        "skill",
        skill.pid,
        "reference_removed",
        caller.actor(),
        None,
    )
    .await?;
    format::empty_json()
}

/// `PUT /api/workers/{pid}/skills` body — declare (upsert) an
/// worker's proficiency in a skill (1–5, optional target).
#[derive(Debug, Deserialize)]
struct WorkerSkillPayload {
    skill_pid: Uuid,
    proficiency: i32,
    #[serde(default)]
    target: Option<i32>,
}

/// `PUT /api/workers/{pid}/skills` — declare or update the
/// proficiency (one row per worker+skill).
#[debug_handler]
async fn declare_skill(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<WorkerSkillPayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    if !rules::valid_proficiency(payload.proficiency) {
        problems.push("proficiency must be between 1 and 5");
    }
    if let Some(target) = payload.target
        && !rules::valid_proficiency(target)
    {
        problems.push("target must be between 1 and 5");
    }
    ensure_valid(&problems.into_vec())?;
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    let skill = skills::Entity::find()
        .filter(skills::Column::Pid.eq(payload.skill_pid))
        .filter(skills::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    let existing = worker_skills::Entity::find()
        .filter(worker_skills::Column::WorkerPid.eq(worker.pid))
        .filter(worker_skills::Column::SkillPid.eq(skill.pid))
        .one(&ctx.db)
        .await?;
    let today = chrono::Utc::now().date_naive();
    let row = match existing {
        Some(row) => {
            let mut active: worker_skills::ActiveModel = row.into();
            active.proficiency = ActiveValue::set(payload.proficiency);
            active.target = ActiveValue::set(payload.target);
            active.assessed_on = ActiveValue::set(today);
            active.deleted_at = ActiveValue::set(None);
            active.update(&ctx.db).await?
        }
        None => {
            worker_skills::ActiveModel {
                pid: ActiveValue::set(Uuid::new_v4()),
                worker_pid: ActiveValue::set(worker.pid),
                skill_pid: ActiveValue::set(skill.pid),
                proficiency: ActiveValue::set(payload.proficiency),
                target: ActiveValue::set(payload.target),
                assessed_on: ActiveValue::set(today),
                deleted_at: ActiveValue::set(None),
                ..Default::default()
            }
            .insert(&ctx.db)
            .await?
        }
    };
    Audit::record(
        &ctx.db,
        "worker_skill",
        row.pid,
        "declared",
        caller.actor(),
        None,
    )
    .await?;
    format::json(row)
}

/// `GET /api/workers/{pid}/skills` — one worker's declared skills.
#[debug_handler]
async fn list_worker_skills(
    State(ctx): State<AppContext>,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    let rows = worker_skills::Entity::find()
        .filter(worker_skills::Column::WorkerPid.eq(worker.pid))
        .filter(worker_skills::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?;
    format::json(rows)
}

// ─── Learning paths ─────────────────────────────────────────────────────────

/// `POST /api/learning-paths` body (steps declared inline, ordered).
#[derive(Debug, Deserialize)]
struct PathPayload {
    name: String,
    #[serde(default)]
    summary: Option<String>,
    #[serde(default)]
    steps: Vec<PathStepPayload>,
}

/// One step of a learning path.
#[derive(Debug, Deserialize)]
struct PathStepPayload {
    course_ref: String,
    title: String,
}

/// `POST /api/learning-paths` — create a path with its ordered steps.
#[debug_handler]
async fn create_path(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<PathPayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_text("name", &payload.name);
    problems.cap_text("name", &payload.name);
    for (index, step) in payload.steps.iter().enumerate() {
        problems.require_text(&format!("steps[{index}].course_ref"), &step.course_ref);
        problems.require_text(&format!("steps[{index}].title"), &step.title);
    }
    ensure_valid(&problems.into_vec())?;
    let txn = ctx.db.begin().await?;
    let path = learning_paths::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        name: ActiveValue::set(payload.name.clone()),
        summary: ActiveValue::set(payload.summary.clone()),
        deleted_at: ActiveValue::set(None),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    for (index, step) in payload.steps.iter().enumerate() {
        learning_path_steps::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            path_pid: ActiveValue::set(path.pid),
            course_ref: ActiveValue::set(step.course_ref.clone()),
            title: ActiveValue::set(step.title.clone()),
            position: ActiveValue::set(i32::try_from(index).unwrap_or(i32::MAX)),
            ..Default::default()
        }
        .insert(&txn)
        .await?;
    }
    Audit::record(
        &txn,
        "learning_path",
        path.pid,
        "created",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(PidRef::of(path.pid))
}

/// `GET /api/learning-paths` — all paths with their step counts.
#[debug_handler]
async fn list_paths(State(ctx): State<AppContext>) -> Result<Response> {
    let paths = learning_paths::Entity::find()
        .filter(learning_paths::Column::DeletedAt.is_null())
        .order_by_asc(learning_paths::Column::Id)
        .all(&ctx.db)
        .await?;
    let steps = learning_path_steps::Entity::find().all(&ctx.db).await?;
    let mut counts: std::collections::BTreeMap<Uuid, usize> = std::collections::BTreeMap::new();
    for step in &steps {
        *counts.entry(step.path_pid).or_default() += 1;
    }
    let view: Vec<serde_json::Value> = paths
        .iter()
        .map(|path| {
            serde_json::json!({
                "pid": path.pid, "name": path.name, "summary": path.summary,
                "steps": counts.get(&path.pid).copied().unwrap_or(0),
            })
        })
        .collect();
    format::json(view)
}

/// `POST /api/learning-paths/{pid}/enrollments` body.
#[derive(Debug, Deserialize)]
struct PathEnrollPayload {
    worker_pid: Uuid,
}

/// `POST /api/learning-paths/{pid}/enrollments` — enrol a worker
/// (idempotent per worker+path).
#[debug_handler]
async fn enroll_path(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<PathEnrollPayload>,
) -> Result<Response> {
    let path = find_path(&ctx, &pid).await?;
    let worker = records::find_worker(&ctx.db, payload.worker_pid).await?;
    let existing = path_enrollments::Entity::find()
        .filter(path_enrollments::Column::PathPid.eq(path.pid))
        .filter(path_enrollments::Column::WorkerPid.eq(worker.pid))
        .filter(path_enrollments::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?;
    if existing.is_some() {
        return Err(unprocessable("worker is already enrolled in this path"));
    }
    let row = path_enrollments::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        path_pid: ActiveValue::set(path.pid),
        worker_pid: ActiveValue::set(worker.pid),
        enrolled_on: ActiveValue::set(chrono::Utc::now().date_naive()),
        deleted_at: ActiveValue::set(None),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;
    Audit::record(
        &ctx.db,
        "path_enrollment",
        row.pid,
        "created",
        caller.actor(),
        None,
    )
    .await?;
    format::json(PidRef::of(row.pid))
}

/// `GET /api/learning-paths/{pid}/progress` — each enrolled worker's
/// honest progress: steps completed (a step counts only if the
/// worker has a **completed** training enrolment for that
/// `course_ref`) out of the path's step count.
#[debug_handler]
async fn path_progress(State(ctx): State<AppContext>, Path(pid): Path<String>) -> Result<Response> {
    let path = find_path(&ctx, &pid).await?;
    let mut steps = learning_path_steps::Entity::find()
        .filter(learning_path_steps::Column::PathPid.eq(path.pid))
        .all(&ctx.db)
        .await?;
    steps.sort_by_key(|step| step.position);
    let step_courses: Vec<String> = steps.iter().map(|s| s.course_ref.clone()).collect();
    let enrollments = path_enrollments::Entity::find()
        .filter(path_enrollments::Column::PathPid.eq(path.pid))
        .filter(path_enrollments::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?;
    let mut members = Vec::new();
    for enrollment in &enrollments {
        let completed: Vec<String> = training_enrollments::Entity::find()
            .filter(training_enrollments::Column::WorkerPid.eq(enrollment.worker_pid))
            .filter(training_enrollments::Column::DeletedAt.is_null())
            .filter(training_enrollments::Column::Status.eq("completed"))
            .all(&ctx.db)
            .await?
            .into_iter()
            .map(|t| t.course_ref)
            .collect();
        let (done, total) = rules::path_progress(&step_courses, &completed);
        let worker = workers::Entity::find()
            .filter(workers::Column::Pid.eq(enrollment.worker_pid))
            .one(&ctx.db)
            .await?;
        members.push(serde_json::json!({
            "worker_pid": enrollment.worker_pid,
            "display_name": worker.map(|e| e.display_name),
            "completed_steps": done,
            "total_steps": total,
        }));
    }
    format::json(serde_json::json!({
        "as_of": chrono::Utc::now(),
        "path": { "pid": path.pid, "name": path.name },
        "steps": steps.iter().map(|s| serde_json::json!({
            "course_ref": s.course_ref, "title": s.title, "position": s.position,
        })).collect::<Vec<_>>(),
        "derivation": "a step is complete iff the worker has a completed training \
                       enrolment for its course_ref; real completions only",
        "members": members,
    }))
}

// ─── Mentorships ────────────────────────────────────────────────────────────

/// `POST /api/mentorships` body.
#[derive(Debug, Deserialize)]
struct MentorshipPayload {
    mentor_pid: Uuid,
    mentee_pid: Uuid,
    focus: String,
}

/// `POST /api/mentorships` — propose a pairing (mentor ≠ mentee).
#[debug_handler]
async fn create_mentorship(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<MentorshipPayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_text("focus", &payload.focus);
    problems.cap_text("focus", &payload.focus);
    if payload.mentor_pid == payload.mentee_pid {
        problems.push("mentor and mentee must be different workers");
    }
    ensure_valid(&problems.into_vec())?;
    records::find_worker(&ctx.db, payload.mentor_pid).await?;
    records::find_worker(&ctx.db, payload.mentee_pid).await?;
    let row = mentorships::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        mentor_pid: ActiveValue::set(payload.mentor_pid),
        mentee_pid: ActiveValue::set(payload.mentee_pid),
        focus: ActiveValue::set(payload.focus.clone()),
        status: ActiveValue::set("proposed".to_string()),
        started_on: ActiveValue::set(None),
        ended_on: ActiveValue::set(None),
        deleted_at: ActiveValue::set(None),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;
    Audit::record(
        &ctx.db,
        "mentorship",
        row.pid,
        "created",
        caller.actor(),
        None,
    )
    .await?;
    format::json(PidRef::of(row.pid))
}

/// `POST /api/mentorships/{pid}/status` body.
#[derive(Debug, Deserialize)]
struct StatusPayload {
    to: String,
}

/// `POST /api/mentorships/{pid}/status` — the lifecycle move (the
/// pure machine refuses illegal transitions); activating stamps
/// `started_on`, closing stamps `ended_on`.
#[debug_handler]
async fn mentorship_status(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<StatusPayload>,
) -> Result<Response> {
    let row = mentorships::Entity::find()
        .filter(mentorships::Column::Pid.eq(records::parse_pid(&pid)?))
        .filter(mentorships::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    rules::mentorship_transition(&row.status, &payload.to)
        .map_err(|reason| unprocessable(&reason))?;
    let today = chrono::Utc::now().date_naive();
    let from = row.status.clone();
    let row_pid = row.pid;
    let mut active: mentorships::ActiveModel = row.into();
    active.status = ActiveValue::set(payload.to.clone());
    if payload.to == "active" {
        active.started_on = ActiveValue::set(Some(today));
    } else if matches!(payload.to.as_str(), "completed" | "ended") {
        active.ended_on = ActiveValue::set(Some(today));
    }
    let updated = active.update(&ctx.db).await?;
    Audit::record(
        &ctx.db,
        "mentorship",
        row_pid,
        "status_changed",
        caller.actor(),
        Some(serde_json::json!({ "from": from, "to": payload.to })),
    )
    .await?;
    format::json(updated)
}

/// `POST /api/mentorships/{pid}/sessions` body.
#[derive(Debug, Deserialize)]
struct SessionPayload {
    held_on: chrono::NaiveDate,
    notes: String,
}

/// `POST /api/mentorships/{pid}/sessions` — log a session (only on an
/// active mentorship).
#[debug_handler]
async fn log_session(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<SessionPayload>,
) -> Result<Response> {
    let mentorship = mentorships::Entity::find()
        .filter(mentorships::Column::Pid.eq(records::parse_pid(&pid)?))
        .filter(mentorships::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    if mentorship.status != "active" {
        return Err(unprocessable(
            "sessions can only be logged on an active mentorship",
        ));
    }
    let mut problems = Problems::new();
    problems.require_text("notes", &payload.notes);
    problems.cap_text("notes", &payload.notes);
    ensure_valid(&problems.into_vec())?;
    let row = mentorship_sessions::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        mentorship_pid: ActiveValue::set(mentorship.pid),
        held_on: ActiveValue::set(payload.held_on),
        notes: ActiveValue::set(payload.notes.clone()),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;
    Audit::record(
        &ctx.db,
        "mentorship_session",
        row.pid,
        "created",
        caller.actor(),
        None,
    )
    .await?;
    format::json(PidRef::of(row.pid))
}

/// `GET /api/mentorships/{pid}` — one mentorship + its session log.
#[debug_handler]
async fn get_mentorship(
    State(ctx): State<AppContext>,
    Path(pid): Path<String>,
) -> Result<Response> {
    let mentorship = mentorships::Entity::find()
        .filter(mentorships::Column::Pid.eq(records::parse_pid(&pid)?))
        .filter(mentorships::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    let sessions = mentorship_sessions::Entity::find()
        .filter(mentorship_sessions::Column::MentorshipPid.eq(mentorship.pid))
        .order_by_desc(mentorship_sessions::Column::HeldOn)
        .all(&ctx.db)
        .await?;
    format::json(serde_json::json!({ "mentorship": mentorship, "sessions": sessions }))
}

// ─── Derived views ──────────────────────────────────────────────────────────

/// `GET /api/learning/skills-matrix` — per-department skill coverage:
/// for each (department, skill) the count of workers at each
/// proficiency and the average, plus per-skill gap counts
/// (proficiency below a declared target).
#[debug_handler]
#[allow(clippy::too_many_lines)] // one pass over the declared skills
async fn skills_matrix(State(ctx): State<AppContext>) -> Result<Response> {
    let worker_rows = workers::Entity::find()
        .filter(workers::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?;
    let dept_of: std::collections::BTreeMap<Uuid, String> = worker_rows
        .iter()
        .map(|e| (e.pid, e.department.clone()))
        .collect();
    let skill_rows = skills::Entity::find()
        .filter(skills::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?;
    let skill_name: std::collections::BTreeMap<Uuid, &str> = skill_rows
        .iter()
        .map(|s| (s.pid, s.name.as_str()))
        .collect();
    let declared = worker_skills::Entity::find()
        .filter(worker_skills::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?;
    // (department, skill) → (count, sum, gap_count)
    let mut cells: std::collections::BTreeMap<(String, Uuid), (usize, i64, usize)> =
        std::collections::BTreeMap::new();
    let mut gaps: Vec<serde_json::Value> = Vec::new();
    for row in &declared {
        let Some(department) = dept_of.get(&row.worker_pid) else {
            continue;
        };
        let cell = cells
            .entry((department.clone(), row.skill_pid))
            .or_default();
        cell.0 += 1;
        cell.1 += i64::from(row.proficiency);
        if row.target.is_some_and(|t| row.proficiency < t) {
            cell.2 += 1;
            gaps.push(serde_json::json!({
                "worker_pid": row.worker_pid,
                "department": department,
                "skill": skill_name.get(&row.skill_pid),
                "proficiency": row.proficiency,
                "target": row.target,
            }));
        }
    }
    let matrix: Vec<serde_json::Value> = cells
        .iter()
        .map(|((department, skill_pid), (count, sum, gap))| {
            #[allow(clippy::cast_precision_loss)] // display average
            let average = *sum as f64 / *count as f64;
            serde_json::json!({
                "department": department,
                "skill": skill_name.get(skill_pid),
                "workers": count,
                "average_proficiency": average,
                "below_target": gap,
            })
        })
        .collect();
    format::json(serde_json::json!({
        "as_of": chrono::Utc::now(),
        "note": "coverage over declared proficiencies only; gaps = declared below a declared target",
        "matrix": matrix,
        "gaps": gaps,
    }))
}

/// Query for the training analytics window.
#[derive(Debug, Deserialize)]
struct AnalyticsQuery {
    /// Cert-expiry horizon in days (default 90, cap 3650).
    expiring_within_days: Option<i64>,
}

/// `GET /api/learning/training-analytics?expiring_within_days=` —
/// per-department training rollups: enrolments by status, completion
/// ratio (completed / non-failed, numerator+denominator), and
/// certificates expiring within the horizon.
#[debug_handler]
async fn training_analytics(
    axum::extract::Query(query): axum::extract::Query<AnalyticsQuery>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let horizon_days = query.expiring_within_days.unwrap_or(90).clamp(0, 3650);
    let today = chrono::Utc::now().date_naive();
    let horizon = today + chrono::Duration::days(horizon_days);
    let worker_rows = workers::Entity::find()
        .filter(workers::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?;
    let dept_of: std::collections::BTreeMap<Uuid, String> = worker_rows
        .iter()
        .map(|e| (e.pid, e.department.clone()))
        .collect();
    let enrollments = training_enrollments::Entity::find()
        .filter(training_enrollments::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?;
    // department → (by_status map, expiring count)
    let mut per_dept: std::collections::BTreeMap<
        String,
        (std::collections::BTreeMap<String, usize>, usize),
    > = std::collections::BTreeMap::new();
    for row in &enrollments {
        let Some(department) = dept_of.get(&row.worker_pid) else {
            continue;
        };
        let entry = per_dept.entry(department.clone()).or_default();
        *entry.0.entry(row.status.clone()).or_default() += 1;
        if row
            .certificate_expires_on
            .is_some_and(|d| d <= horizon && d >= today)
        {
            entry.1 += 1;
        }
    }
    let departments: Vec<serde_json::Value> = per_dept
        .iter()
        .map(|(department, (by_status, expiring))| {
            let completed = by_status.get("completed").copied().unwrap_or(0);
            let denominator: usize = by_status
                .iter()
                .filter(|(status, _)| status.as_str() != "failed")
                .map(|(_, count)| *count)
                .sum();
            #[allow(clippy::cast_precision_loss)] // display ratio
            let value = if denominator == 0 {
                serde_json::Value::Null
            } else {
                serde_json::json!(completed as f64 / denominator as f64)
            };
            serde_json::json!({
                "department": department,
                "by_status": by_status,
                "completion_rate": {
                    "numerator": completed, "denominator": denominator, "value": value,
                },
                "certs_expiring": expiring,
            })
        })
        .collect();
    format::json(serde_json::json!({
        "as_of": today,
        "horizon": horizon,
        "note": "completion rate = completed / (all non-failed enrolments)",
        "departments": departments,
    }))
}

/// `GET /api/learning/mentorship-overview?days=` — active pairings,
/// mentor load (active mentees per mentor), unmatched workers (no
/// active mentorship as mentor or mentee), and stale actives (no
/// session in the window).
#[debug_handler]
async fn mentorship_overview(
    axum::extract::Query(query): axum::extract::Query<MentorshipOverviewQuery>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let stale_days = query.days.unwrap_or(30).clamp(1, 365);
    let today = chrono::Utc::now().date_naive();
    let stale_before = today - chrono::Duration::days(stale_days);
    let worker_rows = workers::Entity::find()
        .filter(workers::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?;
    let name_of: std::collections::BTreeMap<Uuid, &str> = worker_rows
        .iter()
        .map(|e| (e.pid, e.display_name.as_str()))
        .collect();
    let mentorship_rows = mentorships::Entity::find()
        .filter(mentorships::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?;
    let active: Vec<&mentorships::Model> = mentorship_rows
        .iter()
        .filter(|m| m.status == "active")
        .collect();
    let sessions = mentorship_sessions::Entity::find().all(&ctx.db).await?;
    let mut last_session: std::collections::BTreeMap<Uuid, chrono::NaiveDate> =
        std::collections::BTreeMap::new();
    for session in &sessions {
        let entry = last_session
            .entry(session.mentorship_pid)
            .or_insert(session.held_on);
        if session.held_on > *entry {
            *entry = session.held_on;
        }
    }
    let mut mentor_load: std::collections::BTreeMap<Uuid, usize> =
        std::collections::BTreeMap::new();
    let mut engaged: std::collections::HashSet<Uuid> = std::collections::HashSet::new();
    let mut stale = Vec::new();
    for mentorship in &active {
        *mentor_load.entry(mentorship.mentor_pid).or_default() += 1;
        engaged.insert(mentorship.mentor_pid);
        engaged.insert(mentorship.mentee_pid);
        let last = last_session.get(&mentorship.pid).copied();
        if last.is_none_or(|d| d < stale_before) {
            stale.push(serde_json::json!({
                "pid": mentorship.pid,
                "mentor": name_of.get(&mentorship.mentor_pid),
                "mentee": name_of.get(&mentorship.mentee_pid),
                "last_session": last,
            }));
        }
    }
    let load_view: Vec<serde_json::Value> = mentor_load
        .iter()
        .map(|(mentor, count)| {
            serde_json::json!({ "mentor_pid": mentor, "mentor": name_of.get(mentor), "active_mentees": count })
        })
        .collect();
    let unmatched: Vec<serde_json::Value> = worker_rows
        .iter()
        .filter(|e| e.status == "active" && !engaged.contains(&e.pid))
        .map(|e| serde_json::json!({ "pid": e.pid, "display_name": e.display_name, "department": e.department }))
        .collect();
    format::json(serde_json::json!({
        "as_of": today,
        "active_pairings": active.len(),
        "mentor_load": load_view,
        "unmatched_workers": unmatched,
        "stale_days": stale_days,
        "stale_mentorships": stale,
    }))
}

/// Query for the mentorship overview.
#[derive(Debug, Deserialize)]
struct MentorshipOverviewQuery {
    days: Option<i64>,
}

/// Find one live learning path by pid, or 404.
async fn find_path(ctx: &AppContext, pid: &str) -> Result<learning_paths::Model> {
    learning_paths::Entity::find()
        .filter(learning_paths::Column::Pid.eq(records::parse_pid(pid)?))
        .filter(learning_paths::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)
}

/// The learning routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/skills", post(create_skill))
        .add("/skills", get(list_skills))
        .add("/skills/category-suggestions", get(category_suggestions))
        .add(
            "/skills/category-suggestions/apply",
            post(apply_suggestions),
        )
        .add("/skills/{pid}", put(update_skill))
        .add("/skills/{pid}", delete(delete_skill))
        .add("/skills/{pid}/usage", get(skill_usage_view))
        .add("/skills/{pid}/merge", post(merge_skill))
        .add("/skills/{pid}/refs", post(add_skill_ref))
        .add(
            "/skills/{pid}/refs/{framework_slug}",
            delete(remove_skill_ref),
        )
        .add("/workers/{pid}/skills", put(declare_skill))
        .add("/workers/{pid}/skills", get(list_worker_skills))
        .add("/learning-paths", post(create_path))
        .add("/learning-paths", get(list_paths))
        .add("/learning-paths/{pid}/enrollments", post(enroll_path))
        .add("/learning-paths/{pid}/progress", get(path_progress))
        .add("/mentorships", post(create_mentorship))
        .add("/mentorships/{pid}", get(get_mentorship))
        .add("/mentorships/{pid}/status", post(mentorship_status))
        .add("/mentorships/{pid}/sessions", post(log_session))
        .add("/learning/skills-matrix", get(skills_matrix))
        .add("/learning/training-analytics", get(training_analytics))
        .add("/learning/mentorship-overview", get(mentorship_overview))
}
