//! Skills: the catalogue, declared worker proficiency, merging, and external references.

// A section of one controller: it shares the parent module's imports.
#[allow(clippy::wildcard_imports)]
use super::*;

/// `POST /api/skills` body.
#[derive(Debug, Deserialize)]
pub(super) struct SkillPayload {
    name: String,
    category: String,
}

/// `POST /api/skills` — add a skill to the catalog.
#[debug_handler]
pub(super) async fn create_skill(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<SkillPayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_text("name", &payload.name);
    problems.cap_text("name", &payload.name);
    problems.require_token("category", rules::SKILL_CATEGORIES, &payload.category);
    ensure_valid(&problems.into_vec())?;
    let txn = ctx.db.begin().await?;
    let row = skills::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        name: ActiveValue::set(payload.name.clone()),
        category: ActiveValue::set(payload.category.clone()),
        deleted_at: ActiveValue::set(None),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    Audit::record(&txn, "skill", row.pid, "created", caller.actor(), None).await?;
    txn.commit().await?;
    format::json(PidRef::of(row.pid))
}

/// `GET /api/skills` — the catalog, each skill with the external-framework
/// references it carries (PCF name, ESCO concept URI).
#[debug_handler]
pub(super) async fn list_skills(State(ctx): State<AppContext>) -> Result<Response> {
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
pub(super) struct SkillUpdate {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    category: Option<String>,
}

/// `PUT /api/skills/{pid}` — rename and/or recategorise a catalogue skill.
/// Names are unique; a rename never breaks a framework import, which
/// matches by reference first.
#[debug_handler]
pub(super) async fn update_skill(
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
    let txn = ctx.db.begin().await?;
    let updated = active.update(&txn).await?;
    Audit::record(
        &txn,
        "skill",
        updated.pid,
        "updated",
        caller.actor(),
        Some(serde_json::json!({ "name": updated.name, "category": updated.category })),
    )
    .await?;
    txn.commit().await?;
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
pub(super) async fn category_suggestions(State(ctx): State<AppContext>) -> Result<Response> {
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
pub(super) struct ApplySuggestions {
    skill_pids: Vec<Uuid>,
}

/// `POST /api/skills/category-suggestions/apply` — apply the **current**
/// suggestion to the chosen skills (still `other`); skills with no
/// suggestion, or no longer `other`, are left alone and counted.
#[debug_handler]
pub(super) async fn apply_suggestions(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<ApplySuggestions>,
) -> Result<Response> {
    let (mut applied, mut skipped) = (0usize, 0usize);
    let txn = ctx.db.begin().await?;
    for pid in &payload.skill_pids {
        let found = skills::Entity::find()
            .filter(skills::Column::Pid.eq(*pid))
            .filter(skills::Column::DeletedAt.is_null())
            .one(&txn)
            .await?;
        let suggestion = found
            .as_ref()
            .filter(|s| s.category == "other")
            .and_then(|s| rules::suggest_category(&s.name));
        match (found, suggestion) {
            (Some(row), Some((category, _))) => {
                let mut active: skills::ActiveModel = row.into();
                active.category = ActiveValue::set(category.to_string());
                let updated = active.update(&txn).await?;
                Audit::record(
                    &txn,
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
    txn.commit().await?;
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
pub(super) async fn skill_usage_view(
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
pub(super) async fn delete_skill(
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
pub(super) struct MergePayload {
    into_pid: Uuid,
}

/// `POST /api/skills/{pid}/merge` — fold this skill into another and retire
/// it. Everything pointing at it is repointed at the target; where both
/// already appear (a worker who declared both, a profile requiring both, a
/// plan listing both) the stronger statement is kept. One transaction.
#[debug_handler]
#[allow(clippy::too_many_lines)] // one repointing pass per referencing table
pub(super) async fn merge_skill(
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
pub(super) struct SkillRefPayload {
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
pub(super) async fn add_skill_ref(
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
    let txn = ctx.db.begin().await?;
    let row = skill_external_refs::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        skill_pid: ActiveValue::set(skill.pid),
        framework_slug: ActiveValue::set(payload.framework_slug.trim().to_string()),
        reference: ActiveValue::set(payload.reference.clone()),
        label: ActiveValue::set(payload.label.clone()),
        version: ActiveValue::set(payload.version.clone()),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    Audit::record(
        &txn,
        "skill",
        skill.pid,
        "reference_added",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(PidRef::of(row.pid))
}

/// `DELETE /api/skills/{pid}/refs/{framework_slug}`.
#[debug_handler]
pub(super) async fn remove_skill_ref(
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
    let txn = ctx.db.begin().await?;
    row.delete(&txn).await?;
    Audit::record(
        &txn,
        "skill",
        skill.pid,
        "reference_removed",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::empty_json()
}

/// `PUT /api/workers/{pid}/skills` body — declare (upsert) an
/// worker's proficiency in a skill (1–5, optional target).
#[derive(Debug, Deserialize)]
pub(super) struct WorkerSkillPayload {
    skill_pid: Uuid,
    /// WPM's 1–5. May be left out when a framework level is given: it is then worked out by the stated mapping.
    #[serde(default)]
    proficiency: Option<i32>,
    #[serde(default)]
    target: Option<i32>,
    /// The framework the level was stated on (e.g. `sfia`), with `framework_level` on its own scale.
    #[serde(default)]
    framework: Option<String>,
    #[serde(default)]
    framework_level: Option<i32>,
}

/// `PUT /api/workers/{pid}/skills` — declare or update the
/// proficiency (one row per worker+skill).
#[debug_handler]
pub(super) async fn declare_skill(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<WorkerSkillPayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    if let Some(p) = payload.proficiency
        && !rules::valid_proficiency(p)
    {
        problems.push("proficiency must be between 1 and 5");
    }
    if payload.proficiency.is_none() && payload.framework_level.is_none() {
        problems.push("give proficiency, or framework and framework_level");
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
    let native = crate::controllers::sfia::resolve_native(
        &ctx.db,
        payload.framework.as_deref(),
        payload.framework_level,
    )
    .await?;
    if let Some(n) = &native {
        crate::controllers::sfia::check_defined(&ctx.db, skill.pid, n).await?;
    }
    // A stated 1–5 stands; otherwise the framework level is mapped, and the mapping is shown wherever it is read.
    let proficiency = payload
        .proficiency
        .or_else(|| native.as_ref().map(|n| n.wpm))
        .ok_or_else(|| unprocessable("give proficiency, or framework and framework_level"))?;
    let existing = worker_skills::Entity::find()
        .filter(worker_skills::Column::WorkerPid.eq(worker.pid))
        .filter(worker_skills::Column::SkillPid.eq(skill.pid))
        .one(&ctx.db)
        .await?;
    let today = chrono::Utc::now().date_naive();
    let txn = ctx.db.begin().await?;
    let (source_framework, source_level, source_scale_max) =
        native.as_ref().map_or((None, None, None), |n| {
            (Some(n.framework.clone()), Some(n.level), Some(n.scale_max))
        });
    let row = match existing {
        Some(row) => {
            let mut active: worker_skills::ActiveModel = row.into();
            active.proficiency = ActiveValue::set(proficiency);
            active.source_framework = ActiveValue::set(source_framework);
            active.source_level = ActiveValue::set(source_level);
            active.source_scale_max = ActiveValue::set(source_scale_max);
            active.target = ActiveValue::set(payload.target);
            active.assessed_on = ActiveValue::set(today);
            active.deleted_at = ActiveValue::set(None);
            active.update(&txn).await?
        }
        None => {
            worker_skills::ActiveModel {
                pid: ActiveValue::set(Uuid::new_v4()),
                worker_pid: ActiveValue::set(worker.pid),
                skill_pid: ActiveValue::set(skill.pid),
                proficiency: ActiveValue::set(proficiency),
                target: ActiveValue::set(payload.target),
                assessed_on: ActiveValue::set(today),
                source_framework: ActiveValue::set(source_framework),
                source_level: ActiveValue::set(source_level),
                source_scale_max: ActiveValue::set(source_scale_max),
                deleted_at: ActiveValue::set(None),
                ..Default::default()
            }
            .insert(&txn)
            .await?
        }
    };
    crate::models::skill_history::record(
        &txn,
        worker.pid,
        skill.pid,
        Some(proficiency),
        "declared",
        caller.actor(),
        auth::acting_for_other(&caller, &worker.person_ref),
    )
    .await?;
    Audit::record(
        &txn,
        "worker_skill",
        row.pid,
        "declared",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(row_json(&row)?)
}

/// `GET /api/workers/{pid}/skills` — one worker's declared skills.
#[debug_handler]
pub(super) async fn list_worker_skills(
    State(ctx): State<AppContext>,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    let rows = worker_skills::Entity::find()
        .filter(worker_skills::Column::WorkerPid.eq(worker.pid))
        .filter(worker_skills::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?;
    // Where a level was stated on a framework's own scale, say so, with the framework's wording and the mapping used.
    let stated: Vec<_> = rows
        .iter()
        .map(|r| (r.skill_pid, r.source_framework.as_deref(), r.source_level))
        .collect();
    let described = crate::controllers::sfia::native_descriptions(&ctx.db, &stated).await?;
    let names = crate::controllers::sfia::framework_names(&ctx.db).await?;
    let out: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            let mut v = serde_json::to_value(r).unwrap_or_default();
            if r.source_level.is_some() {
                v["mapping"] = serde_json::json!(crate::rules::sfia::MAPPING_NAME);
                v["native_description"] = serde_json::json!(described.get(&r.skill_pid));
                v["source_framework_name"] =
                    serde_json::json!(r.source_framework.as_ref().and_then(|f| names.get(f)));
            }
            v
        })
        .collect();
    format::json(out)
}
