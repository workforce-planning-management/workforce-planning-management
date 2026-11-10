//! Skills: the catalogue, declared worker proficiency, merging, and external references.

// A section of one controller: it shares the parent module's imports.
#[allow(clippy::wildcard_imports)]
use super::*;
use axa::extract::{Json, Path};

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
    let pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO skills (pid, name, category) VALUES ($1, $2, $3)",
        pid,
        payload.name,
        payload.category,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(&mut tx, "skill", pid, "created", caller.actor(), None).await?;
    tx.commit().await?;
    format::json(PidRef::of(pid))
}

/// `GET /api/skills` — the catalog, each skill with the external-framework
/// references it carries (PCF name, ESCO concept URI).
#[debug_handler]
pub(super) async fn list_skills(State(ctx): State<AppContext>) -> Result<Response> {
    let rows = sqlx::query!(
        "SELECT pid, name, category FROM skills WHERE deleted_at IS NULL ORDER BY name, id"
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let refs = sqlx::query!(
        r#"SELECT skill_pid, framework_slug, "ref" AS reference, label, version FROM skill_external_refs ORDER BY id"#
    )
    .fetch_all(db::pool(&ctx))
    .await?;
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
    let mut new_name = None;
    if let Some(name) = &payload.name {
        let name = rules::normalize_skill_name(name).map_err(|e| unprocessable(&e))?;
        if name != row.name {
            let taken = sqlx::query_scalar!(
                r#"SELECT 1 AS "one!" FROM skills WHERE name = $1 LIMIT 1"#,
                name
            )
            .fetch_optional(db::pool(&ctx))
            .await?
            .is_some();
            if taken {
                return Err(unprocessable("a skill with that name already exists"));
            }
            new_name = Some(name);
        }
    }
    let mut tx = db::pool(&ctx).begin().await?;
    let updated = sqlx::query!(
        "UPDATE skills SET name = COALESCE($2, name), category = COALESCE($3, category), updated_at = now()
          WHERE id = $1
        RETURNING pid, name, category",
        row.id,
        new_name,
        payload.category,
    )
    .fetch_one(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "skill",
        updated.pid,
        "updated",
        caller.actor(),
        Some(serde_json::json!({ "name": updated.name, "category": updated.category })),
    )
    .await?;
    tx.commit().await?;
    format::json(PidRef::of(updated.pid))
}

/// A live catalogue skill: what the handlers here need of it.
struct LiveSkill {
    id: i32,
    pid: Uuid,
    name: String,
}

async fn find_live_skill(ctx: &AppContext, pid: &str) -> Result<LiveSkill> {
    find_live_skill_by_pid(ctx, records::parse_pid(pid)?).await
}

async fn find_live_skill_by_pid(ctx: &AppContext, pid: Uuid) -> Result<LiveSkill> {
    sqlx::query_as!(
        LiveSkill,
        "SELECT id, pid, name FROM skills WHERE pid = $1 AND deleted_at IS NULL",
        pid,
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)
}

/// `GET /api/skills/category-suggestions` — skills still `other` for which a
/// keyword rule suggests a category, with the keyword that triggered it.
/// Suggestions only: nothing changes until a planner applies them.
#[debug_handler]
pub(super) async fn category_suggestions(State(ctx): State<AppContext>) -> Result<Response> {
    let rows = sqlx::query!(
        "SELECT pid, name FROM skills WHERE deleted_at IS NULL AND category = 'other' ORDER BY name, id"
    )
    .fetch_all(db::pool(&ctx))
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
    let mut tx = db::pool(&ctx).begin().await?;
    for pid in &payload.skill_pids {
        let found = sqlx::query!(
            "SELECT id, pid, name, category FROM skills WHERE pid = $1 AND deleted_at IS NULL",
            pid,
        )
        .fetch_optional(&mut *tx)
        .await?;
        let suggestion = found
            .as_ref()
            .filter(|s| s.category == "other")
            .and_then(|s| rules::suggest_category(&s.name));
        match (found, suggestion) {
            (Some(row), Some((category, _))) => {
                sqlx::query!(
                    "UPDATE skills SET category = $2, updated_at = now() WHERE id = $1",
                    row.id,
                    category,
                )
                .execute(&mut *tx)
                .await?;
                db::audit(
                    &mut tx,
                    "skill",
                    row.pid,
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
    tx.commit().await?;
    format::json(serde_json::json!({ "applied": applied, "skipped": skipped }))
}

/// Count where a skill is used.
async fn skill_usage(ctx: &AppContext, skill_pid: Uuid) -> Result<merge_rules::Usage> {
    let count = |n: i64| usize::try_from(n).unwrap_or(usize::MAX);
    let row = sqlx::query!(
        r#"SELECT
             (SELECT COUNT(*) FROM worker_skills WHERE skill_pid = $1 AND deleted_at IS NULL) AS "declared_by!",
             (SELECT COUNT(*) FROM role_skill_requirements WHERE skill_pid = $1) AS "required_by_profiles!",
             (SELECT COUNT(*) FROM development_plan_items WHERE skill_pid = $1) AS "in_development_plans!",
             (SELECT COUNT(*) FROM initiative_skill_shifts WHERE skill_pid = $1) AS "in_initiatives!""#,
        skill_pid,
    )
    .fetch_one(db::pool(ctx))
    .await?;
    Ok(merge_rules::Usage {
        declared_by: count(row.declared_by),
        required_by_profiles: count(row.required_by_profiles),
        in_development_plans: count(row.in_development_plans),
        in_initiatives: count(row.in_initiatives),
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
    format::json(usage_json(&skill_usage(&ctx, skill.pid).await?))
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
    let usage = skill_usage(&ctx, skill.pid).await?;
    if !usage.deletable() {
        return Err(unprocessable(&format!(
            "the skill is in use ({} place(s)); merge it into another skill instead",
            usage.total()
        )));
    }
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "DELETE FROM skill_external_refs WHERE skill_pid = $1",
        skill.pid
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        "UPDATE skills SET deleted_at = now(), updated_at = now() WHERE id = $1",
        skill.id
    )
    .execute(&mut *tx)
    .await?;
    db::audit(&mut tx, "skill", skill.pid, "deleted", caller.actor(), None).await?;
    tx.commit().await?;
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
    let target = find_live_skill_by_pid(&ctx, payload.into_pid).await?;
    let mut tx = db::pool(&ctx).begin().await?;
    let (mut merged, mut moved) = (0usize, 0usize);

    // Worker declarations.
    for row in sqlx::query!(
        r#"SELECT id, worker_pid, proficiency, target, assessed_on, deleted_at AS "deleted_at: chrono::DateTime<chrono::FixedOffset>"
             FROM worker_skills WHERE skill_pid = $1 ORDER BY id"#,
        source.pid,
    )
    .fetch_all(&mut *tx)
    .await?
    {
        let twin = sqlx::query!(
            "SELECT id, proficiency, target, assessed_on FROM worker_skills WHERE worker_pid = $1 AND skill_pid = $2 LIMIT 1",
            row.worker_pid,
            target.pid,
        )
        .fetch_optional(&mut *tx)
        .await?;
        if let Some(twin) = twin {
            // The stronger statement is kept; a live declaration revives a retired twin.
            sqlx::query!(
                "UPDATE worker_skills
                    SET proficiency = $2, target = $3, assessed_on = $4,
                        deleted_at = CASE WHEN $5 THEN NULL ELSE deleted_at END, updated_at = now()
                  WHERE id = $1",
                twin.id,
                merge_rules::merge_level(twin.proficiency, row.proficiency),
                merge_rules::merge_target(twin.target, row.target),
                twin.assessed_on.max(row.assessed_on),
                row.deleted_at.is_none(),
            )
            .execute(&mut *tx)
            .await?;
            sqlx::query!("DELETE FROM worker_skills WHERE id = $1", row.id)
                .execute(&mut *tx)
                .await?;
            merged += 1;
        } else {
            sqlx::query!(
                "UPDATE worker_skills SET skill_pid = $2, updated_at = now() WHERE id = $1",
                row.id,
                target.pid,
            )
            .execute(&mut *tx)
            .await?;
            moved += 1;
        }
    }
    // Role requirements.
    for row in sqlx::query!(
        "SELECT id, role_profile_pid, min_proficiency, importance, note, source_level, source_scale_max
           FROM role_skill_requirements WHERE skill_pid = $1 ORDER BY id",
        source.pid,
    )
    .fetch_all(&mut *tx)
    .await?
    {
        let twin = sqlx::query!(
            "SELECT id, min_proficiency, importance, note, source_level FROM role_skill_requirements
              WHERE role_profile_pid = $1 AND skill_pid = $2 LIMIT 1",
            row.role_profile_pid,
            target.pid,
        )
        .fetch_optional(&mut *tx)
        .await?;
        if let Some(twin) = twin {
            let take_level = twin.source_level.is_none();
            sqlx::query!(
                "UPDATE role_skill_requirements
                    SET min_proficiency = $2, importance = $3, note = COALESCE(note, $4),
                        source_level = CASE WHEN $5 THEN $6 ELSE source_level END,
                        source_scale_max = CASE WHEN $5 THEN $7 ELSE source_scale_max END,
                        updated_at = now()
                  WHERE id = $1",
                twin.id,
                merge_rules::merge_level(twin.min_proficiency, row.min_proficiency),
                merge_rules::stronger_importance(&twin.importance, &row.importance),
                row.note,
                take_level,
                row.source_level,
                row.source_scale_max,
            )
            .execute(&mut *tx)
            .await?;
            sqlx::query!("DELETE FROM role_skill_requirements WHERE id = $1", row.id)
                .execute(&mut *tx)
                .await?;
            merged += 1;
        } else {
            sqlx::query!(
                "UPDATE role_skill_requirements SET skill_pid = $2, updated_at = now() WHERE id = $1",
                row.id,
                target.pid,
            )
            .execute(&mut *tx)
            .await?;
            moved += 1;
        }
    }
    // Initiative skill shifts: the target's statement wins a conflict.
    for row in sqlx::query!(
        "SELECT id, initiative_pid FROM initiative_skill_shifts WHERE skill_pid = $1 ORDER BY id",
        source.pid,
    )
    .fetch_all(&mut *tx)
    .await?
    {
        let twin_exists = sqlx::query_scalar!(
            r#"SELECT 1 AS "one!" FROM initiative_skill_shifts WHERE initiative_pid = $1 AND skill_pid = $2 LIMIT 1"#,
            row.initiative_pid,
            target.pid,
        )
        .fetch_optional(&mut *tx)
        .await?
        .is_some();
        if twin_exists {
            sqlx::query!("DELETE FROM initiative_skill_shifts WHERE id = $1", row.id)
                .execute(&mut *tx)
                .await?;
            merged += 1;
        } else {
            sqlx::query!(
                "UPDATE initiative_skill_shifts SET skill_pid = $2, updated_at = now() WHERE id = $1",
                row.id,
                target.pid,
            )
            .execute(&mut *tx)
            .await?;
            moved += 1;
        }
    }
    // Development-plan items: the target's item wins a conflict.
    for row in sqlx::query!(
        "SELECT id, plan_pid FROM development_plan_items WHERE skill_pid = $1 ORDER BY id",
        source.pid,
    )
    .fetch_all(&mut *tx)
    .await?
    {
        let twin_exists = sqlx::query_scalar!(
            r#"SELECT 1 AS "one!" FROM development_plan_items WHERE plan_pid = $1 AND skill_pid = $2 LIMIT 1"#,
            row.plan_pid,
            target.pid,
        )
        .fetch_optional(&mut *tx)
        .await?
        .is_some();
        if twin_exists {
            sqlx::query!("DELETE FROM development_plan_items WHERE id = $1", row.id)
                .execute(&mut *tx)
                .await?;
            merged += 1;
        } else {
            sqlx::query!(
                "UPDATE development_plan_items SET skill_pid = $2, updated_at = now() WHERE id = $1",
                row.id,
                target.pid,
            )
            .execute(&mut *tx)
            .await?;
            moved += 1;
        }
    }
    // External references: keep the target's per framework, move the rest.
    for row in sqlx::query!(
        "SELECT id, framework_slug FROM skill_external_refs WHERE skill_pid = $1 ORDER BY id",
        source.pid,
    )
    .fetch_all(&mut *tx)
    .await?
    {
        let has = sqlx::query_scalar!(
            r#"SELECT 1 AS "one!" FROM skill_external_refs WHERE skill_pid = $1 AND framework_slug = $2 LIMIT 1"#,
            target.pid,
            row.framework_slug,
        )
        .fetch_optional(&mut *tx)
        .await?
        .is_some();
        if has {
            sqlx::query!("DELETE FROM skill_external_refs WHERE id = $1", row.id)
                .execute(&mut *tx)
                .await?;
        } else {
            sqlx::query!(
                "UPDATE skill_external_refs SET skill_pid = $2, updated_at = now() WHERE id = $1",
                row.id,
                target.pid,
            )
            .execute(&mut *tx)
            .await?;
        }
    }
    sqlx::query!(
        "UPDATE skills SET deleted_at = now(), updated_at = now() WHERE id = $1",
        source.id
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "skill",
        source.pid,
        "merged",
        caller.actor(),
        Some(serde_json::json!({
            "into": target.pid, "from_name": source.name, "records_moved": moved, "records_merged": merged,
        })),
    )
    .await?;
    tx.commit().await?;
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
        let loaded = sqlx::query_scalar!(r#"SELECT 1 AS "one!" FROM esco_skills LIMIT 1"#)
            .fetch_optional(db::pool(&ctx))
            .await?
            .is_some();
        let known = sqlx::query_scalar!(
            r#"SELECT 1 AS "one!" FROM esco_skills WHERE uri = $1 LIMIT 1"#,
            payload.reference,
        )
        .fetch_optional(db::pool(&ctx))
        .await?
        .is_some();
        if loaded && !known {
            return Err(unprocessable(
                "that URI is not an ESCO skill in the pinned copy",
            ));
        }
    }
    let clash = sqlx::query_scalar!(
        r#"SELECT 1 AS "one!" FROM skill_external_refs
            WHERE framework_slug = $1 AND (skill_pid = $2 OR "ref" = $3) LIMIT 1"#,
        payload.framework_slug,
        skill.pid,
        payload.reference,
    )
    .fetch_optional(db::pool(&ctx))
    .await?
    .is_some();
    if clash {
        return Err(unprocessable(
            "this skill already has a reference in that framework, or the reference names another skill",
        ));
    }
    let ref_pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        r#"INSERT INTO skill_external_refs (pid, skill_pid, framework_slug, "ref", label, version)
           VALUES ($1, $2, $3, $4, $5, $6)"#,
        ref_pid,
        skill.pid,
        payload.framework_slug.trim(),
        payload.reference,
        payload.label,
        payload.version,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "skill",
        skill.pid,
        "reference_added",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(PidRef::of(ref_pid))
}

/// `DELETE /api/skills/{pid}/refs/{framework_slug}`.
#[debug_handler]
pub(super) async fn remove_skill_ref(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path((pid, framework)): Path<(String, String)>,
) -> Result<Response> {
    let skill = find_live_skill(&ctx, &pid).await?;
    let id = sqlx::query_scalar!(
        "SELECT id FROM skill_external_refs WHERE skill_pid = $1 AND framework_slug = $2 LIMIT 1",
        skill.pid,
        framework,
    )
    .fetch_optional(db::pool(&ctx))
    .await?
    .ok_or(Error::NotFound)?;
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!("DELETE FROM skill_external_refs WHERE id = $1", id)
        .execute(&mut *tx)
        .await?;
    db::audit(
        &mut tx,
        "skill",
        skill.pid,
        "reference_removed",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
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
#[allow(clippy::too_many_lines)] // validate, map the framework level, upsert, history and audit, linearly
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
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    let skill = find_live_skill_by_pid(&ctx, payload.skill_pid).await?;
    let native = crate::controllers::roles::resolve_native(
        &ctx,
        payload.framework.as_deref(),
        payload.framework_level,
    )
    .await?;
    if let Some(n) = &native {
        crate::controllers::roles::check_defined(&ctx, skill.pid, n).await?;
    }
    // A stated 1–5 stands; otherwise the framework level is mapped, and the mapping is shown wherever it is read.
    let proficiency = payload
        .proficiency
        .or_else(|| native.as_ref().map(|n| n.wpm))
        .ok_or_else(|| unprocessable("give proficiency, or framework and framework_level"))?;
    let existing = sqlx::query_scalar!(
        "SELECT id FROM worker_skills WHERE worker_pid = $1 AND skill_pid = $2 LIMIT 1",
        worker.pid,
        skill.pid,
    )
    .fetch_optional(db::pool(&ctx))
    .await?;
    let today = chrono::Utc::now().date_naive();
    let (source_framework, source_level, source_scale_max) =
        native.as_ref().map_or((None, None, None), |n| {
            (Some(n.framework.clone()), Some(n.level), Some(n.scale_max))
        });
    let mut tx = db::pool(&ctx).begin().await?;
    let row = match existing {
        Some(id) => {
            sqlx::query_as!(
                worker_skills::Model,
                r#"UPDATE worker_skills
                      SET proficiency = $2, source_framework = $3, source_level = $4, source_scale_max = $5, target = $6,
                          assessed_on = $7, deleted_at = NULL, updated_at = now()
                    WHERE id = $1
                RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, skill_pid,
                          proficiency, target, assessed_on, source_framework, source_level, source_scale_max,
                          deleted_at AS "deleted_at: _""#,
                id,
                proficiency,
                source_framework,
                source_level,
                source_scale_max,
                payload.target,
                today,
            )
            .fetch_one(&mut *tx)
            .await?
        }
        None => {
            sqlx::query_as!(
                worker_skills::Model,
                r#"INSERT INTO worker_skills
                          (pid, worker_pid, skill_pid, proficiency, target, assessed_on, source_framework, source_level,
                           source_scale_max)
                   VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, skill_pid,
                          proficiency, target, assessed_on, source_framework, source_level, source_scale_max,
                          deleted_at AS "deleted_at: _""#,
                Uuid::new_v4(),
                worker.pid,
                skill.pid,
                proficiency,
                payload.target,
                today,
                source_framework,
                source_level,
                source_scale_max,
            )
            .fetch_one(&mut *tx)
            .await?
        }
    };
    crate::models::skill_history::record_in(
        &mut tx,
        worker.pid,
        skill.pid,
        Some(proficiency),
        "declared",
        caller.actor(),
        auth::acting_for_other(&caller, &worker.person_ref),
    )
    .await?;
    db::audit(
        &mut tx,
        "worker_skill",
        row.pid,
        "declared",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(row_json(&row)?)
}

/// `GET /api/workers/{pid}/skills` — one worker's declared skills.
#[debug_handler]
pub(super) async fn list_worker_skills(
    State(ctx): State<AppContext>,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    let rows = sqlx::query_as!(
        worker_skills::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, skill_pid,
                  proficiency, target, assessed_on, source_framework, source_level, source_scale_max,
                  deleted_at AS "deleted_at: _"
             FROM worker_skills WHERE worker_pid = $1 AND deleted_at IS NULL ORDER BY id"#,
        worker.pid,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    // Where a level was stated on a framework's own scale, say so, with the framework's wording and the mapping used.
    let stated: Vec<_> = rows
        .iter()
        .map(|r| (r.skill_pid, r.source_framework.as_deref(), r.source_level))
        .collect();
    let described = crate::controllers::roles::native_descriptions(&ctx, &stated).await?;
    let names = crate::controllers::roles::framework_names(&ctx).await?;
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
