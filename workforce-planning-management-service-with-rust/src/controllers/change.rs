//! The **AI-driven change tracker** (WPM-D28): an operational record of an
//! automation or AI initiative and its impact on *roles* and *skills*, with
//! an aggregate readiness view of the workforce affected. Pure rules in
//! [`crate::rules::change`].
//!
//! It tracks positions and capabilities, never people: readiness is counts
//! and ratios over the **employed** workers whose job title matches an
//! affected role, and no individual is named or ranked.

use super::ratio_json;
use axa::extract::{Json, Path};
use axa::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

use super::{ensure_valid, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::db;
use crate::models::_entities::change_initiatives;
use crate::models::records;
use crate::rules::change as rules;
use crate::rules::metrics::is_employed_on;
use crate::rules::mobility::same_title;
use crate::rules::talent::ratio;
use crate::validation::Problems;

/// A `{pid}` reference response.
#[derive(Serialize)]
struct PidRef {
    pid: String,
}

/// `POST /api/change-initiatives` body.
#[derive(Debug, Deserialize)]
struct InitiativePayload {
    name: String,
    kind: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    starts_on: Option<chrono::NaiveDate>,
}

/// `POST /api/change-initiatives` — open a draft initiative.
#[debug_handler]
async fn create_initiative(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<InitiativePayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_text("name", &payload.name);
    problems.cap_text("name", &payload.name);
    problems.require_token("kind", rules::KINDS, &payload.kind);
    if let Some(text) = &payload.description {
        problems.cap_text("description", text);
    }
    ensure_valid(&problems.into_vec())?;
    let pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO change_initiatives (pid, name, description, kind, status, starts_on, deleted_at)
         VALUES ($1, $2, $3, $4, 'draft', $5, NULL)",
        pid,
        payload.name.trim(),
        payload.description,
        payload.kind,
        payload.starts_on,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "change_initiative",
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

/// One role an initiative touches.
struct RoleImpact {
    initiative_pid: Uuid,
    role_profile_pid: Uuid,
    impact: String,
    timeframe: String,
    note: Option<String>,
}

/// One skill whose demand an initiative moves.
struct SkillShift {
    initiative_pid: Uuid,
    skill_pid: Uuid,
    direction: String,
    note: Option<String>,
}

/// The role impacts of one initiative, or of all of them.
async fn role_impacts(ctx: &AppContext, initiative: Option<Uuid>) -> Result<Vec<RoleImpact>> {
    Ok(sqlx::query_as!(
        RoleImpact,
        "SELECT initiative_pid, role_profile_pid, impact, timeframe, note FROM initiative_role_impacts
          WHERE ($1::uuid IS NULL OR initiative_pid = $1) ORDER BY id",
        initiative,
    )
    .fetch_all(db::pool(ctx))
    .await?)
}

/// The skill shifts of one initiative, or of all of them.
async fn skill_shifts(ctx: &AppContext, initiative: Option<Uuid>) -> Result<Vec<SkillShift>> {
    Ok(sqlx::query_as!(
        SkillShift,
        "SELECT initiative_pid, skill_pid, direction, note FROM initiative_skill_shifts
          WHERE ($1::uuid IS NULL OR initiative_pid = $1) ORDER BY id",
        initiative,
    )
    .fetch_all(db::pool(ctx))
    .await?)
}

/// Every role profile's job title, by pid (soft-deleted ones too, as an impact may still name one).
async fn role_titles(ctx: &AppContext) -> Result<BTreeMap<Uuid, String>> {
    Ok(sqlx::query!("SELECT pid, job_title FROM role_profiles")
        .fetch_all(db::pool(ctx))
        .await?
        .into_iter()
        .map(|p| (p.pid, p.job_title))
        .collect())
}

/// Every skill's name, by pid (soft-deleted ones too, as a shift may still name one).
async fn skill_names(ctx: &AppContext) -> Result<BTreeMap<Uuid, String>> {
    Ok(sqlx::query!("SELECT pid, name FROM skills")
        .fetch_all(db::pool(ctx))
        .await?
        .into_iter()
        .map(|s| (s.pid, s.name))
        .collect())
}

/// `GET /api/change-initiatives` — live initiatives, newest first, with
/// how many roles and skills each touches.
#[debug_handler]
async fn list_initiatives(State(ctx): State<AppContext>) -> Result<Response> {
    let rows = sqlx::query_as!(
        change_initiatives::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, name, description, kind, status,
                  starts_on, deleted_at AS "deleted_at: _"
             FROM change_initiatives WHERE deleted_at IS NULL ORDER BY id DESC"#
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let impacts = role_impacts(&ctx, None).await?;
    let shifts = skill_shifts(&ctx, None).await?;
    let out: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "pid": r.pid,
                "name": r.name,
                "kind": r.kind,
                "status": r.status,
                "starts_on": r.starts_on,
                "roles_affected": impacts.iter().filter(|i| i.initiative_pid == r.pid).count(),
                "skills_shifting": shifts.iter().filter(|s| s.initiative_pid == r.pid).count(),
            })
        })
        .collect();
    format::json(out)
}

async fn find_initiative(ctx: &AppContext, pid: &str) -> Result<change_initiatives::Model> {
    sqlx::query_as!(
        change_initiatives::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, name, description, kind, status,
                  starts_on, deleted_at AS "deleted_at: _"
             FROM change_initiatives WHERE pid = $1 AND deleted_at IS NULL"#,
        records::parse_pid(pid)?,
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)
}

/// `GET /api/change-initiatives/{pid}` — an initiative with its role
/// impacts and skill shifts.
#[debug_handler]
async fn get_initiative(
    State(ctx): State<AppContext>,
    Path(pid): Path<String>,
) -> Result<Response> {
    let initiative = find_initiative(&ctx, &pid).await?;
    let titles = role_titles(&ctx).await?;
    let names = skill_names(&ctx).await?;
    let role_impacts: Vec<serde_json::Value> = role_impacts(&ctx, Some(initiative.pid))
        .await?
        .iter()
        .map(|i| {
            serde_json::json!({
                "role_profile_pid": i.role_profile_pid,
                "job_title": titles.get(&i.role_profile_pid),
                "impact": i.impact,
                "timeframe": i.timeframe,
                "note": i.note,
            })
        })
        .collect();
    let skill_shifts: Vec<serde_json::Value> = skill_shifts(&ctx, Some(initiative.pid))
        .await?
        .iter()
        .map(|s| {
            serde_json::json!({
                "skill_pid": s.skill_pid,
                "skill": names.get(&s.skill_pid),
                "direction": s.direction,
                "note": s.note,
            })
        })
        .collect();
    format::json(serde_json::json!({
        "pid": initiative.pid,
        "name": initiative.name,
        "description": initiative.description,
        "kind": initiative.kind,
        "status": initiative.status,
        "starts_on": initiative.starts_on,
        "role_impacts": role_impacts,
        "skill_shifts": skill_shifts,
    }))
}

/// `POST /api/change-initiatives/{pid}/status` body.
#[derive(Debug, Deserialize)]
struct StatusPayload {
    to: String,
}

/// `POST /api/change-initiatives/{pid}/status` — move an initiative
/// through its lifecycle (`draft → active → completed | cancelled`).
#[debug_handler]
async fn set_status(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<StatusPayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_token("to", rules::STATUSES, &payload.to);
    ensure_valid(&problems.into_vec())?;
    let initiative = find_initiative(&ctx, &pid).await?;
    rules::check_transition(&initiative.status, &payload.to).map_err(|e| unprocessable(&e))?;
    let from = initiative.status.clone();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "UPDATE change_initiatives SET status = $2, updated_at = now() WHERE id = $1",
        initiative.id,
        payload.to,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "change_initiative",
        initiative.pid,
        "status_changed",
        caller.actor(),
        Some(serde_json::json!({ "from": from, "to": payload.to })),
    )
    .await?;
    tx.commit().await?;
    format::json(PidRef {
        pid: initiative.pid.to_string(),
    })
}

/// Initiatives are editable until they reach a terminal status.
fn ensure_editable(initiative: &change_initiatives::Model) -> Result<()> {
    if matches!(initiative.status.as_str(), "completed" | "cancelled") {
        return Err(unprocessable("the initiative is closed"));
    }
    Ok(())
}

/// `PUT /api/change-initiatives/{pid}/role-impacts` body.
#[derive(Debug, Deserialize)]
struct RoleImpactPayload {
    role_profile_pid: Uuid,
    impact: String,
    timeframe: String,
    #[serde(default)]
    note: Option<String>,
}

/// `PUT /api/change-initiatives/{pid}/role-impacts` — record how a role is
/// affected (upsert per role).
#[debug_handler]
async fn set_role_impact(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<RoleImpactPayload>,
) -> Result<Response> {
    rules::validate_role_impact(&payload.impact, &payload.timeframe)
        .map_err(|e| unprocessable(&e))?;
    let mut problems = Problems::new();
    if let Some(note) = &payload.note {
        problems.cap_text("note", note);
    }
    ensure_valid(&problems.into_vec())?;
    let initiative = find_initiative(&ctx, &pid).await?;
    ensure_editable(&initiative)?;
    sqlx::query_scalar!(
        r#"SELECT 1 AS "one!" FROM role_profiles WHERE pid = $1 AND deleted_at IS NULL"#,
        payload.role_profile_pid,
    )
    .fetch_optional(db::pool(&ctx))
    .await?
    .ok_or(Error::NotFound)?;
    let mut tx = db::pool(&ctx).begin().await?;
    // One impact per role: a second one replaces the first.
    sqlx::query!(
        "INSERT INTO initiative_role_impacts (pid, initiative_pid, role_profile_pid, impact, timeframe, note)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (initiative_pid, role_profile_pid)
         DO UPDATE SET impact = EXCLUDED.impact, timeframe = EXCLUDED.timeframe, note = EXCLUDED.note, updated_at = now()",
        Uuid::new_v4(),
        initiative.pid,
        payload.role_profile_pid,
        payload.impact,
        payload.timeframe,
        payload.note,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "change_initiative",
        initiative.pid,
        "role_impact_set",
        caller.actor(),
        Some(serde_json::json!({ "role_profile_pid": payload.role_profile_pid, "impact": payload.impact })),
    )
    .await?;
    tx.commit().await?;
    format::empty_json()
}

/// `DELETE /api/change-initiatives/{pid}/role-impacts/{role_profile_pid}`.
#[debug_handler]
async fn remove_role_impact(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path((pid, role)): Path<(String, String)>,
) -> Result<Response> {
    let initiative = find_initiative(&ctx, &pid).await?;
    ensure_editable(&initiative)?;
    let role = records::parse_pid(&role)?;
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query_scalar!(
        "DELETE FROM initiative_role_impacts WHERE initiative_pid = $1 AND role_profile_pid = $2 RETURNING id",
        initiative.pid,
        role,
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(Error::NotFound)?;
    db::audit(
        &mut tx,
        "change_initiative",
        initiative.pid,
        "role_impact_removed",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::empty_json()
}

/// `PUT /api/change-initiatives/{pid}/skill-shifts` body.
#[derive(Debug, Deserialize)]
struct SkillShiftPayload {
    skill_pid: Uuid,
    direction: String,
    #[serde(default)]
    note: Option<String>,
}

/// `PUT /api/change-initiatives/{pid}/skill-shifts` — record a skill whose
/// demand the initiative raises or lowers (upsert per skill).
#[debug_handler]
async fn set_skill_shift(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<SkillShiftPayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_token("direction", rules::DIRECTIONS, &payload.direction);
    if let Some(note) = &payload.note {
        problems.cap_text("note", note);
    }
    ensure_valid(&problems.into_vec())?;
    let initiative = find_initiative(&ctx, &pid).await?;
    ensure_editable(&initiative)?;
    sqlx::query_scalar!(
        r#"SELECT 1 AS "one!" FROM skills WHERE pid = $1 AND deleted_at IS NULL"#,
        payload.skill_pid,
    )
    .fetch_optional(db::pool(&ctx))
    .await?
    .ok_or(Error::NotFound)?;
    let mut tx = db::pool(&ctx).begin().await?;
    // One shift per skill: a second one replaces the first.
    sqlx::query!(
        "INSERT INTO initiative_skill_shifts (pid, initiative_pid, skill_pid, direction, note)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (initiative_pid, skill_pid)
         DO UPDATE SET direction = EXCLUDED.direction, note = EXCLUDED.note, updated_at = now()",
        Uuid::new_v4(),
        initiative.pid,
        payload.skill_pid,
        payload.direction,
        payload.note,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "change_initiative",
        initiative.pid,
        "skill_shift_set",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::empty_json()
}

/// `DELETE /api/change-initiatives/{pid}/skill-shifts/{skill_pid}`.
#[debug_handler]
async fn remove_skill_shift(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path((pid, skill)): Path<(String, String)>,
) -> Result<Response> {
    let initiative = find_initiative(&ctx, &pid).await?;
    ensure_editable(&initiative)?;
    let skill = records::parse_pid(&skill)?;
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query_scalar!(
        "DELETE FROM initiative_skill_shifts WHERE initiative_pid = $1 AND skill_pid = $2 RETURNING id",
        initiative.pid,
        skill,
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(Error::NotFound)?;
    db::audit(
        &mut tx,
        "change_initiative",
        initiative.pid,
        "skill_shift_removed",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::empty_json()
}

/// What the readiness view reads of a worker: who, in which role, and whether employed.
struct Holder {
    pid: Uuid,
    job_title: String,
    hired_on: chrono::NaiveDate,
    terminated_on: Option<chrono::NaiveDate>,
}

/// Query for the readiness view.
#[derive(Debug, Deserialize)]
struct ReadinessQuery {
    /// The proficiency (1–5) counted as ready for a rising skill. Default 3.
    min_proficiency: Option<i32>,
}

/// `GET /api/change-initiatives/{pid}/readiness?min_proficiency=` —
/// **aggregate only**: for each affected role, the employed workers in it
/// and how many have an active reskill plan; and for each rising skill,
/// how many of the workers in displaced or reshaped roles already meet,
/// fall below, or have not declared it. No individual is named.
#[debug_handler]
#[allow(clippy::too_many_lines)] // roles, workers, plans, and skills in one aggregate pass
async fn readiness(
    State(ctx): State<AppContext>,
    Path(pid): Path<String>,
    axa::extract::Query(query): axa::extract::Query<ReadinessQuery>,
) -> Result<Response> {
    let bar = query
        .min_proficiency
        .unwrap_or(crate::rules::capability::DEFAULT_MIN_PROFICIENCY);
    if !crate::rules::learning::valid_proficiency(bar) {
        return Err(unprocessable("min_proficiency must be between 1 and 5"));
    }
    let initiative = find_initiative(&ctx, &pid).await?;
    let today = chrono::Utc::now().date_naive();
    let employed: Vec<Holder> = sqlx::query_as!(
        Holder,
        "SELECT pid, job_title, hired_on, terminated_on FROM workers WHERE deleted_at IS NULL ORDER BY id"
    )
    .fetch_all(db::pool(&ctx))
    .await?
    .into_iter()
    .filter(|w| is_employed_on(today, w.hired_on, w.terminated_on))
    .collect();
    let with_reskill_plan: BTreeSet<Uuid> = sqlx::query_scalar!(
        "SELECT worker_pid FROM development_plans
          WHERE deleted_at IS NULL AND kind = 'reskill' AND status = 'active'"
    )
    .fetch_all(db::pool(&ctx))
    .await?
    .into_iter()
    .collect();
    let titles = role_titles(&ctx).await?;
    let impacts = role_impacts(&ctx, Some(initiative.pid)).await?;

    let mut affected: BTreeSet<Uuid> = BTreeSet::new();
    let roles: Vec<serde_json::Value> = impacts
        .iter()
        .map(|i| {
            let title = titles.get(&i.role_profile_pid).cloned().unwrap_or_default();
            let holders: Vec<&Holder> = employed
                .iter()
                .filter(|w| same_title(&w.job_title, &title))
                .collect();
            let planned = holders
                .iter()
                .filter(|w| with_reskill_plan.contains(&w.pid))
                .count();
            if rules::affects_current_holders(&i.impact) {
                affected.extend(holders.iter().map(|w| w.pid));
            }
            serde_json::json!({
                "job_title": title,
                "impact": i.impact,
                "timeframe": i.timeframe,
                "employed_workers": holders.len(),
                "with_active_reskill_plan": planned,
                "reskill_plan_coverage": if rules::affects_current_holders(&i.impact) {
                    ratio_json(ratio(planned, holders.len()))
                } else {
                    serde_json::Value::Null
                },
            })
        })
        .collect();

    let shifts = skill_shifts(&ctx, Some(initiative.pid)).await?;
    let names = skill_names(&ctx).await?;
    // In id order, so a later declaration of the same skill wins, as it did when read in table order.
    let declared: BTreeMap<(Uuid, Uuid), i32> = sqlx::query!(
        "SELECT worker_pid, skill_pid, proficiency FROM worker_skills WHERE deleted_at IS NULL ORDER BY id"
    )
    .fetch_all(db::pool(&ctx))
    .await?
    .into_iter()
    .map(|d| ((d.worker_pid, d.skill_pid), d.proficiency))
    .collect();
    let rising: Vec<serde_json::Value> = shifts
        .iter()
        .filter(|s| s.direction == "rising")
        .map(|s| {
            let levels: Vec<Option<i32>> = affected
                .iter()
                .map(|w| declared.get(&(*w, s.skill_pid)).copied())
                .collect();
            let r = rules::skill_readiness(&levels, bar);
            serde_json::json!({
                "skill": names.get(&s.skill_pid),
                "meeting": r.meeting,
                "below": r.below,
                "undeclared": r.undeclared,
                "meeting_share": ratio_json(ratio(r.meeting, affected.len())),
            })
        })
        .collect();
    let declining: Vec<&String> = shifts
        .iter()
        .filter(|s| s.direction == "declining")
        .filter_map(|s| names.get(&s.skill_pid))
        .collect();
    let planned_total = affected
        .iter()
        .filter(|w| with_reskill_plan.contains(w))
        .count();

    format::json(serde_json::json!({
        "derivation": "counts EMPLOYED workers whose job title matches an affected role; \
                       `affected_workers` are those in displaced or reshaped roles. Skill \
                       readiness compares DECLARED proficiency with the bar (undeclared is \
                       unknown, not below). Aggregate only: no individual is named or ranked \
                       (WPM-D28).",
        "initiative": { "pid": initiative.pid, "name": initiative.name, "status": initiative.status },
        "bar": bar,
        "affected_workers": affected.len(),
        "with_active_reskill_plan": planned_total,
        "reskill_plan_coverage": ratio_json(ratio(planned_total, affected.len())),
        "roles": roles,
        "rising_skills": rising,
        "declining_skills": declining,
    }))
}

/// The change-tracker routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/change-initiatives", post(create_initiative))
        .add("/change-initiatives", get(list_initiatives))
        .add("/change-initiatives/{pid}", get(get_initiative))
        .add("/change-initiatives/{pid}/status", post(set_status))
        .add(
            "/change-initiatives/{pid}/role-impacts",
            put(set_role_impact),
        )
        .add(
            "/change-initiatives/{pid}/role-impacts/{role_profile_pid}",
            delete(remove_role_impact),
        )
        .add(
            "/change-initiatives/{pid}/skill-shifts",
            put(set_skill_shift),
        )
        .add(
            "/change-initiatives/{pid}/skill-shifts/{skill_pid}",
            delete(remove_skill_shift),
        )
        .add("/change-initiatives/{pid}/readiness", get(readiness))
}
