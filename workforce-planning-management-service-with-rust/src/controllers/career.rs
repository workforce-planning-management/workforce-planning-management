//! **Career history and aspirations** (`rules::career`): a person's *past*
//! roles and skill levels as intervals with a start and a stop, and their
//! *future* — aspirations, learning goals, growth ideas.
//!
//! - **Past:** changing a role or a skill level closes the previous interval
//!   automatically (see `framework_roles` and `models::skill_history`); past
//!   entries can also be added retrospectively with explicit dates.
//!   `skills-as-of` answers "what did they have on this date?".
//! - **Future:** aspirations are the person's own until they **share** them;
//!   anyone else sees only the shared ones.
//! - **On behalf of:** every row records who made it and whether that was
//!   someone other than the person (HR acting for a worker).
//!
//! Past entries and aspirations are statements, not assessments.

use super::writable_worker;
use axa::extract::{Json, Path};
use axa::prelude::*;
use chrono::{DateTime, FixedOffset, NaiveDate, TimeZone, Utc};
use serde::Deserialize;
use std::collections::BTreeMap;
use uuid::Uuid;

use super::unprocessable;
use crate::auth::{self, MaybeAuthUser};
use crate::db;
use crate::models::_entities::{
    worker_aspirations, worker_framework_roles, worker_skill_history, workers,
};
use crate::models::records;
use crate::rules::career::{self as rules, Interval};
use crate::tasks::import_esco::ESCO_SLUG;
use crate::tasks::import_framework::PCF_SLUG;

fn utc(t: DateTime<FixedOffset>) -> DateTime<Utc> {
    t.with_timezone(&Utc)
}

/// A live UK GDAD PCF role level: its pid and job title.
async fn pcf_profile(ctx: &AppContext, pid: Uuid) -> Result<Option<(Uuid, String)>> {
    Ok(sqlx::query!(
        "SELECT pid, job_title FROM role_profiles
          WHERE pid = $1 AND deleted_at IS NULL AND framework_slug = $2 LIMIT 1",
        pid,
        PCF_SLUG,
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .map(|r| (r.pid, r.job_title)))
}

/// Whether a live catalogue skill has this pid.
async fn live_skill(ctx: &AppContext, pid: Uuid) -> Result<bool> {
    Ok(sqlx::query_scalar!(
        r#"SELECT 1 AS "one!" FROM skills WHERE pid = $1 AND deleted_at IS NULL LIMIT 1"#,
        pid
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .is_some())
}

/// A worker's role intervals, in one framework or all, in the order they were recorded.
async fn roles_of(
    ctx: &AppContext,
    worker_pid: Uuid,
    framework: Option<&str>,
) -> Result<Vec<worker_framework_roles::Model>> {
    Ok(sqlx::query_as!(
        worker_framework_roles::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, framework_slug,
                  role_profile_pid, esco_occupation_uri, role_label, selected_on, started_at AS "started_at: _",
                  ended_at AS "ended_at: _", recorded_by, on_behalf
             FROM worker_framework_roles
            WHERE worker_pid = $1 AND ($2::text IS NULL OR framework_slug = $2)
            ORDER BY id"#,
        worker_pid,
        framework,
    )
    .fetch_all(db::pool(ctx))
    .await?)
}

/// A worker's skill-level intervals, for one skill or all, in the order they were recorded.
async fn skill_intervals_of(
    ctx: &AppContext,
    worker_pid: Uuid,
    skill_pid: Option<Uuid>,
) -> Result<Vec<worker_skill_history::Model>> {
    Ok(sqlx::query_as!(
        worker_skill_history::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, skill_pid,
                  proficiency, started_at AS "started_at: _", ended_at AS "ended_at: _", source, recorded_by,
                  on_behalf
             FROM worker_skill_history
            WHERE worker_pid = $1 AND ($2::uuid IS NULL OR skill_pid = $2)
            ORDER BY id"#,
        worker_pid,
        skill_pid,
    )
    .fetch_all(db::pool(ctx))
    .await?)
}

fn start_of(day: NaiveDate) -> DateTime<Utc> {
    Utc.from_utc_datetime(&day.and_hms_opt(0, 0, 0).unwrap_or_default())
}

// ─── Role history ───────────────────────────────────────────────────────────

/// Query naming a framework.
#[derive(Debug, Deserialize)]
struct FrameworkQuery {
    framework: Option<String>,
}

fn role_row(r: &worker_framework_roles::Model) -> serde_json::Value {
    serde_json::json!({
        "pid": r.pid,
        "framework": r.framework_slug,
        "role_label": r.role_label,
        "role_profile_pid": r.role_profile_pid,
        "occupation_uri": r.esco_occupation_uri,
        "started_at": r.started_at,
        "ended_at": r.ended_at,
        "current": r.ended_at.is_none(),
        "recorded_by": r.recorded_by,
        "on_behalf": r.on_behalf,
    })
}

/// `GET /api/workers/{pid}/role-history?framework=` — every role the person has
/// held in a framework (or all), newest first; the open one is `current`.
#[debug_handler]
async fn role_history(
    State(ctx): State<AppContext>,
    Path(pid): Path<String>,
    axa::extract::Query(query): axa::extract::Query<FrameworkQuery>,
) -> Result<Response> {
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    let rows = sqlx::query_as!(
        worker_framework_roles::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, framework_slug,
                  role_profile_pid, esco_occupation_uri, role_label, selected_on, started_at AS "started_at: _",
                  ended_at AS "ended_at: _", recorded_by, on_behalf
             FROM worker_framework_roles
            WHERE worker_pid = $1 AND ($2::text IS NULL OR framework_slug = $2)
            ORDER BY started_at DESC"#,
        worker.pid,
        query.framework,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    format::json(rows.iter().map(role_row).collect::<Vec<_>>())
}

/// `POST /api/workers/{pid}/framework-roles/{framework}/past` body.
#[derive(Debug, Deserialize)]
struct PastRolePayload {
    #[serde(default)]
    role_profile_pid: Option<Uuid>,
    #[serde(default)]
    occupation_uri: Option<String>,
    /// The first day in the role.
    started_on: NaiveDate,
    /// The last day in the role (inclusive).
    ended_on: NaiveDate,
}

/// `POST /api/workers/{pid}/framework-roles/{framework}/past` — record a role
/// held in the past, with its dates. It may not overlap another role in the
/// same framework (a person holds one at a time).
#[debug_handler]
async fn add_past_role(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path((pid, framework)): Path<(String, String)>,
    Json(payload): Json<PastRolePayload>,
) -> Result<Response> {
    if !crate::rules::framework_roles::valid_framework(&framework) {
        return Err(unprocessable("unknown framework"));
    }
    let worker = writable_worker(&ctx, &caller, &pid).await?;
    let interval = rules::validate_past(
        start_of(payload.started_on),
        start_of(payload.ended_on) + chrono::Duration::days(1),
        Utc::now(),
    )
    .map_err(|e| unprocessable(&e))?;
    let (profile_pid, occupation_uri, label) = if framework == PCF_SLUG {
        let id = payload
            .role_profile_pid
            .ok_or_else(|| unprocessable("role_profile_pid is required for the UK GDAD PCF"))?;
        let (profile_pid, job_title) = pcf_profile(&ctx, id)
            .await?
            .ok_or_else(|| unprocessable("that is not a UK GDAD PCF role level"))?;
        (Some(profile_pid), None, job_title)
    } else {
        let uri = payload
            .occupation_uri
            .ok_or_else(|| unprocessable("occupation_uri is required for ESCO"))?;
        let occupation = super::esco::find_occupation(&ctx, &uri)
            .await
            .map_err(|_| unprocessable("that is not an ESCO occupation in the pinned copy"))?;
        (None, Some(occupation.uri), occupation.label)
    };
    let existing = roles_of(&ctx, worker.pid, Some(&framework)).await?;
    let spans: Vec<Interval> = existing
        .iter()
        .map(|r| Interval {
            start: utc(r.started_at),
            end: r.ended_at.map(utc),
        })
        .collect();
    if let Some(i) = rules::first_overlap(&interval, &spans) {
        return Err(unprocessable(&format!(
            "that overlaps the role '{}' held from {}",
            existing[i].role_label,
            existing[i].started_at.date_naive()
        )));
    }
    let mut tx = db::pool(&ctx).begin().await?;
    let row = sqlx::query_as!(
        worker_framework_roles::Model,
        r#"INSERT INTO worker_framework_roles (pid, worker_pid, framework_slug, role_profile_pid, esco_occupation_uri,
                                             role_label, selected_on, started_at, ended_at, recorded_by, on_behalf)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
           RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, framework_slug,
                     role_profile_pid, esco_occupation_uri, role_label, selected_on, started_at AS "started_at: _",
                     ended_at AS "ended_at: _", recorded_by, on_behalf"#,
        Uuid::new_v4(),
        worker.pid,
        framework,
        profile_pid,
        occupation_uri,
        label,
        payload.started_on,
        interval.start,
        interval.end,
        caller.actor(),
        auth::acting_for_other(&caller, &worker.person_ref),
    )
    .fetch_one(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "worker",
        worker.pid,
        "past_role_added",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(role_row(&row))
}

// ─── Skill history ──────────────────────────────────────────────────────────

/// Query for the skill timeline.
#[derive(Debug, Deserialize)]
struct SkillQuery {
    skill_pid: Option<Uuid>,
}

async fn skill_names(ctx: &AppContext) -> Result<BTreeMap<Uuid, String>> {
    Ok(sqlx::query!("SELECT pid, name FROM skills")
        .fetch_all(db::pool(ctx))
        .await?
        .into_iter()
        .map(|s| (s.pid, s.name))
        .collect())
}

/// `GET /api/workers/{pid}/skill-history?skill_pid=` — the timeline of skill
/// levels: each interval a skill was held at a level, newest first.
#[debug_handler]
async fn skill_history(
    State(ctx): State<AppContext>,
    Path(pid): Path<String>,
    axa::extract::Query(query): axa::extract::Query<SkillQuery>,
) -> Result<Response> {
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    let rows = sqlx::query_as!(
        worker_skill_history::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, skill_pid,
                  proficiency, started_at AS "started_at: _", ended_at AS "ended_at: _", source, recorded_by,
                  on_behalf
             FROM worker_skill_history
            WHERE worker_pid = $1 AND ($2::uuid IS NULL OR skill_pid = $2)
            ORDER BY started_at DESC"#,
        worker.pid,
        query.skill_pid,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let names = skill_names(&ctx).await?;
    let out: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "pid": r.pid,
                "skill_pid": r.skill_pid,
                "skill": names.get(&r.skill_pid),
                "proficiency": r.proficiency,
                "started_at": r.started_at,
                "ended_at": r.ended_at,
                "current": r.ended_at.is_none(),
                "source": r.source,
                "recorded_by": r.recorded_by,
                "on_behalf": r.on_behalf,
            })
        })
        .collect();
    format::json(out)
}

/// `POST /api/workers/{pid}/skill-history/past` body.
#[derive(Debug, Deserialize)]
struct PastSkillPayload {
    skill_pid: Uuid,
    proficiency: i32,
    started_on: NaiveDate,
    ended_on: NaiveDate,
}

/// `POST /api/workers/{pid}/skill-history/past` — record a level held in the
/// past, with its dates (a retrospective entry). It may not overlap another
/// interval for the same skill.
#[debug_handler]
async fn add_past_skill(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<PastSkillPayload>,
) -> Result<Response> {
    if !crate::rules::learning::valid_proficiency(payload.proficiency) {
        return Err(unprocessable("proficiency must be between 1 and 5"));
    }
    let worker = writable_worker(&ctx, &caller, &pid).await?;
    if !live_skill(&ctx, payload.skill_pid).await? {
        return Err(Error::NotFound);
    }
    let interval = rules::validate_past(
        start_of(payload.started_on),
        start_of(payload.ended_on) + chrono::Duration::days(1),
        Utc::now(),
    )
    .map_err(|e| unprocessable(&e))?;
    let existing = skill_intervals_of(&ctx, worker.pid, Some(payload.skill_pid)).await?;
    let spans: Vec<Interval> = existing
        .iter()
        .map(|r| Interval {
            start: utc(r.started_at),
            end: r.ended_at.map(utc),
        })
        .collect();
    if rules::first_overlap(&interval, &spans).is_some() {
        return Err(unprocessable(
            "that overlaps another interval recorded for this skill",
        ));
    }
    let pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO worker_skill_history (pid, worker_pid, skill_pid, proficiency, started_at, ended_at, source,
                                           recorded_by, on_behalf)
         VALUES ($1, $2, $3, $4, $5, $6, 'retrospective', $7, $8)",
        pid,
        worker.pid,
        payload.skill_pid,
        payload.proficiency,
        interval.start,
        interval.end,
        caller.actor(),
        auth::acting_for_other(&caller, &worker.person_ref),
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "worker",
        worker.pid,
        "past_skill_added",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(serde_json::json!({ "pid": pid }))
}

/// Query for the as-of view.
#[derive(Debug, Deserialize)]
struct AsOfQuery {
    /// The date to read the state at (end of that day, UTC).
    at: NaiveDate,
}

/// `GET /api/workers/{pid}/skills-as-of?at=YYYY-MM-DD` — the skills and levels
/// the person held at the end of that day, from the history; also their roles.
#[debug_handler]
async fn skills_as_of(
    State(ctx): State<AppContext>,
    Path(pid): Path<String>,
    axa::extract::Query(query): axa::extract::Query<AsOfQuery>,
) -> Result<Response> {
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    let instant = start_of(query.at) + chrono::Duration::days(1) - chrono::Duration::seconds(1);
    let rows = skill_intervals_of(&ctx, worker.pid, None).await?;
    let names = skill_names(&ctx).await?;
    let mut by_skill: BTreeMap<Uuid, Vec<(Interval, i32)>> = BTreeMap::new();
    for r in &rows {
        by_skill.entry(r.skill_pid).or_default().push((
            Interval {
                start: utc(r.started_at),
                end: r.ended_at.map(utc),
            },
            r.proficiency,
        ));
    }
    let mut held: Vec<serde_json::Value> = by_skill
        .iter()
        .filter_map(|(skill, history)| {
            rules::level_at(history, instant).map(|level| {
                serde_json::json!({ "skill_pid": skill, "skill": names.get(skill), "proficiency": level })
            })
        })
        .collect();
    held.sort_by_key(|v| v["skill"].as_str().map(str::to_lowercase));
    let roles = roles_of(&ctx, worker.pid, None).await?;
    let held_roles: Vec<serde_json::Value> = roles
        .iter()
        .filter(|r| {
            Interval {
                start: utc(r.started_at),
                end: r.ended_at.map(utc),
            }
            .contains(instant)
        })
        .map(|r| serde_json::json!({ "framework": r.framework_slug, "role_label": r.role_label }))
        .collect();
    format::json(serde_json::json!({
        "at": query.at,
        "derivation": "from the recorded history: the interval each role or skill level was held; \
                       backfilled declarations start on the date they were last assessed",
        "roles": held_roles,
        "skills": held,
    }))
}

// ─── Aspirations ────────────────────────────────────────────────────────────

/// The role or skill an aspiration points at, checked against its catalogue:
/// `(framework, profile, occupation, role label, skill)`.
async fn aspiration_target(
    ctx: &AppContext,
    payload: &AspirationPayload,
) -> Result<(
    Option<String>,
    Option<Uuid>,
    Option<String>,
    Option<String>,
    Option<Uuid>,
)> {
    Ok(if payload.kind == "role" {
        let framework = payload
            .framework_slug
            .clone()
            .filter(|f| crate::rules::framework_roles::valid_framework(f))
            .ok_or_else(|| unprocessable("a role aspiration needs a framework_slug"))?;
        if framework == PCF_SLUG {
            let id = payload
                .role_profile_pid
                .ok_or_else(|| unprocessable("role_profile_pid is required for the UK GDAD PCF"))?;
            let (profile_pid, job_title) = pcf_profile(ctx, id)
                .await?
                .ok_or_else(|| unprocessable("that is not a UK GDAD PCF role level"))?;
            (
                Some(framework),
                Some(profile_pid),
                None,
                Some(job_title),
                None,
            )
        } else {
            let uri = payload
                .occupation_uri
                .clone()
                .ok_or_else(|| unprocessable("occupation_uri is required for ESCO"))?;
            let occupation = super::esco::find_occupation(ctx, &uri)
                .await
                .map_err(|_| unprocessable("that is not an ESCO occupation in the pinned copy"))?;
            (
                Some(framework),
                None,
                Some(occupation.uri),
                Some(occupation.label),
                None,
            )
        }
    } else {
        let id = payload
            .skill_pid
            .ok_or_else(|| unprocessable("a skill aspiration needs a skill_pid"))?;
        if !live_skill(ctx, id).await? {
            return Err(Error::NotFound);
        }
        (None, None, None, None, Some(id))
    })
}

/// `POST /api/workers/{pid}/aspirations` body.
#[derive(Debug, Deserialize)]
struct AspirationPayload {
    /// `role` or `skill`.
    kind: String,
    /// For a role: the framework (`uk-gdad-pcf` / `esco`).
    #[serde(default)]
    framework_slug: Option<String>,
    #[serde(default)]
    role_profile_pid: Option<Uuid>,
    #[serde(default)]
    occupation_uri: Option<String>,
    /// For a skill: the catalogue skill and the level aimed for (1–5).
    #[serde(default)]
    skill_pid: Option<Uuid>,
    #[serde(default)]
    target_level: Option<i32>,
    horizon: String,
    #[serde(default)]
    status: Option<String>,
    /// A growth idea or learning goal, in the person's own words.
    #[serde(default)]
    note: Option<String>,
    /// Who may see it: `private` (default), `manager` (their management
    /// chain), or `everyone` (anyone who can view the record).
    #[serde(default)]
    visibility: Option<String>,
}

/// `POST /api/workers/{pid}/aspirations` — record a future role or skill
/// target: an aspiration, learning goal, or growth idea.
#[debug_handler]
async fn add_aspiration(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<AspirationPayload>,
) -> Result<Response> {
    let worker = writable_worker(&ctx, &caller, &pid).await?;
    let status = payload.status.clone().unwrap_or_else(|| "idea".to_string());
    let visibility = payload
        .visibility
        .clone()
        .unwrap_or_else(|| "private".to_string());
    rules::validate_visibility(&visibility).map_err(|e| unprocessable(&e))?;
    rules::validate_aspiration(
        &payload.kind,
        payload.target_level,
        &payload.horizon,
        &status,
        payload.note.as_deref(),
    )
    .map_err(|e| unprocessable(&e))?;
    let (framework, profile, occupation, role_label, skill) =
        aspiration_target(&ctx, &payload).await?;
    let pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO worker_aspirations (pid, worker_pid, kind, framework_slug, role_profile_pid, esco_occupation_uri,
                                         role_label, skill_pid, target_level, horizon, status, note, visibility,
                                         recorded_by, on_behalf, deleted_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, NULL)",
        pid,
        worker.pid,
        payload.kind,
        framework,
        profile,
        occupation,
        role_label,
        skill,
        payload.target_level,
        payload.horizon,
        status,
        payload.note,
        visibility,
        caller.actor(),
        auth::acting_for_other(&caller, &worker.person_ref),
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "worker",
        worker.pid,
        "aspiration_added",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(serde_json::json!({ "pid": pid }))
}

/// How the worker stands against an aspiration today.
async fn progress_toward(
    ctx: &AppContext,
    worker_pid: Uuid,
    a: &worker_aspirations::Model,
    declared: &BTreeMap<Uuid, i32>,
) -> Result<serde_json::Value> {
    if a.kind == "skill" {
        let have = a.skill_pid.and_then(|s| declared.get(&s)).copied();
        let target = a.target_level.unwrap_or(0);
        return Ok(serde_json::json!({
            "current_level": have,
            "target_level": a.target_level,
            "gap": have.map_or(Some(target), |h| (h < target).then_some(target - h)),
            "achieved": have.is_some_and(|h| h >= target),
        }));
    }
    if let Some(profile) = a.role_profile_pid {
        let reqs = sqlx::query!(
            "SELECT skill_pid, min_proficiency FROM role_skill_requirements WHERE role_profile_pid = $1 ORDER BY id",
            profile
        )
        .fetch_all(db::pool(ctx))
        .await?;
        let met = reqs
            .iter()
            .filter(|r| {
                declared
                    .get(&r.skill_pid)
                    .is_some_and(|h| *h >= r.min_proficiency)
            })
            .count();
        return Ok(serde_json::json!({ "requirements": reqs.len(), "met": met }));
    }
    if let Some(uri) = &a.esco_occupation_uri {
        let essential: Vec<String> = sqlx::query_scalar!(
            "SELECT skill_uri FROM esco_occupation_skills
              WHERE occupation_uri = $1 AND relation = 'essential' ORDER BY id",
            uri
        )
        .fetch_all(db::pool(ctx))
        .await?;
        let links: BTreeMap<String, Uuid> = sqlx::query!(
            r#"SELECT "ref" AS reference, skill_pid FROM skill_external_refs WHERE framework_slug = $1 ORDER BY id"#,
            ESCO_SLUG
        )
        .fetch_all(db::pool(ctx))
        .await?
        .into_iter()
        .map(|r| (r.reference, r.skill_pid))
        .collect();
        let have = essential
            .iter()
            .filter(|u| links.get(*u).is_some_and(|p| declared.contains_key(p)))
            .count();
        return Ok(serde_json::json!({ "essential_skills": essential.len(), "have": have }));
    }
    let _ = worker_pid;
    Ok(serde_json::Value::Null)
}

/// How the caller relates to `worker`: the person themselves, someone above
/// them in the management chain, or anyone else. With enforcement off there is
/// no identity to compare, so the caller reads as the person (as in tests).
async fn viewer_of(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    worker: &workers::Model,
) -> Result<rules::Viewer> {
    if !auth::acting_for_other(caller, &worker.person_ref) {
        return Ok(rules::Viewer::Person);
    }
    let Some(claims) = caller.claims() else {
        return Ok(rules::Viewer::Other);
    };
    let all = sqlx::query!(
        "SELECT pid, manager_pid, person_ref FROM workers WHERE deleted_at IS NULL ORDER BY id"
    )
    .fetch_all(db::pool(ctx))
    .await?;
    let manager_of: BTreeMap<Uuid, Uuid> = all
        .iter()
        .filter_map(|w| w.manager_pid.map(|m| (w.pid, m)))
        .collect();
    let by_pid: BTreeMap<Uuid, _> = all.iter().map(|w| (w.pid, w)).collect();
    let is_manager = crate::rules::org::upline(worker.pid, &manager_of)
        .iter()
        .any(|m| {
            by_pid
                .get(m)
                .is_some_and(|boss| rules::is_self(&claims.sub, &boss.person_ref))
        });
    Ok(if is_manager {
        rules::Viewer::Manager
    } else {
        rules::Viewer::Other
    })
}

/// `GET /api/workers/{pid}/aspirations` — the person's aspirations and growth
/// ideas, each with how they stand against it today. **Private by default:**
/// the person sees all of theirs; anyone else sees only those shared.
#[debug_handler]
async fn list_aspirations(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    let viewer = viewer_of(&ctx, &caller, &worker).await?;
    let rows = aspirations_of(&ctx, &[worker.pid]).await?;
    let declared: BTreeMap<Uuid, i32> = sqlx::query!(
        "SELECT skill_pid, proficiency FROM worker_skills
          WHERE worker_pid = $1 AND deleted_at IS NULL ORDER BY id",
        worker.pid
    )
    .fetch_all(db::pool(&ctx))
    .await?
    .into_iter()
    .map(|d| (d.skill_pid, d.proficiency))
    .collect();
    let names = skill_names(&ctx).await?;
    let mut out = Vec::new();
    for a in &rows {
        if !rules::can_view(viewer, &a.visibility) {
            continue;
        }
        out.push(serde_json::json!({
            "pid": a.pid,
            "kind": a.kind,
            "framework": a.framework_slug,
            "role_label": a.role_label,
            "role_profile_pid": a.role_profile_pid,
            "occupation_uri": a.esco_occupation_uri,
            "skill_pid": a.skill_pid,
            "skill": a.skill_pid.and_then(|s| names.get(&s)),
            "target_level": a.target_level,
            "horizon": a.horizon,
            "status": a.status,
            "note": a.note,
            "visibility": a.visibility,
            "recorded_by": a.recorded_by,
            "on_behalf": a.on_behalf,
            "progress": progress_toward(&ctx, worker.pid, a, &declared).await?,
        }));
    }
    format::json(serde_json::json!({
        "viewer": match viewer {
            rules::Viewer::Person => "person",
            rules::Viewer::Manager => "manager",
            rules::Viewer::Other => "other",
        },
        "viewer_is_the_person": viewer == rules::Viewer::Person,
        "aspirations": out,
    }))
}

/// `GET /api/workers/{pid}/downline-aspirations` — for a manager: everyone below
/// them (direct and indirect reports) and the aspirations each has shared with
/// their managers or with everyone. **Private ones are never shown, and their
/// number is not either.** Read as the manager `{pid}`; if the caller is
/// someone else, only aspirations shared with everyone appear.
#[debug_handler]
async fn downline_aspirations(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let manager = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    let viewer = if auth::acting_for_other(&caller, &manager.person_ref) {
        rules::Viewer::Other
    } else {
        rules::Viewer::Manager
    };
    let all = sqlx::query!(
        "SELECT pid, manager_pid, display_name, job_title, department FROM workers
          WHERE deleted_at IS NULL ORDER BY id"
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let mut reports_of: BTreeMap<Uuid, Vec<Uuid>> = BTreeMap::new();
    for w in &all {
        if let Some(boss) = w.manager_pid {
            reports_of.entry(boss).or_default().push(w.pid);
        }
    }
    let below = crate::rules::org::downline(manager.pid, &reports_of);
    let by_pid: BTreeMap<Uuid, _> = all.iter().map(|w| (w.pid, w)).collect();
    let pids: Vec<Uuid> = below.iter().map(|(p, _)| *p).collect();
    let rows = aspirations_of(&ctx, &pids).await?;
    let names = skill_names(&ctx).await?;
    let declared: BTreeMap<(Uuid, Uuid), i32> = sqlx::query!(
        "SELECT worker_pid, skill_pid, proficiency FROM worker_skills WHERE deleted_at IS NULL ORDER BY id"
    )
    .fetch_all(db::pool(&ctx))
    .await?
    .into_iter()
    .map(|d| ((d.worker_pid, d.skill_pid), d.proficiency))
    .collect();
    let mut people = Vec::new();
    for (pid, depth) in &below {
        let Some(person) = by_pid.get(pid) else {
            continue;
        };
        let mine: Vec<&worker_aspirations::Model> = rows
            .iter()
            .filter(|a| a.worker_pid == *pid && rules::can_view(viewer, &a.visibility))
            .collect();
        let person_declared: BTreeMap<Uuid, i32> = declared
            .iter()
            .filter(|((w, _), _)| w == pid)
            .map(|((_, s), l)| (*s, *l))
            .collect();
        let mut items = Vec::new();
        for a in &mine {
            items.push(serde_json::json!({
                "pid": a.pid,
                "kind": a.kind,
                "framework": a.framework_slug,
                "role_label": a.role_label,
                "skill": a.skill_pid.and_then(|s| names.get(&s)),
                "target_level": a.target_level,
                "horizon": a.horizon,
                "status": a.status,
                "note": a.note,
                "visibility": a.visibility,
                "progress": progress_toward(&ctx, *pid, a, &person_declared).await?,
            }));
        }
        people.push(serde_json::json!({
            "worker_pid": pid,
            "display_name": person.display_name,
            "job_title": person.job_title,
            "department": person.department,
            "depth": depth,
            "direct_report": *depth == 1,
            "aspirations": items,
        }));
    }
    format::json(serde_json::json!({
        "manager": { "pid": manager.pid, "display_name": manager.display_name },
        "viewer": if viewer == rules::Viewer::Manager { "manager" } else { "other" },
        "derivation": "everyone below the manager in the management chain (direct and indirect \
                       reports); only aspirations shared with managers or everyone are shown — \
                       private ones, and how many there are, are never revealed",
        "team": people,
    }))
}

/// `PUT /api/aspirations/{pid}` body — any of these may change.
#[derive(Debug, Deserialize)]
struct AspirationUpdate {
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    horizon: Option<String>,
    #[serde(default)]
    target_level: Option<i32>,
    #[serde(default)]
    note: Option<String>,
    #[serde(default)]
    visibility: Option<String>,
}

/// The live aspirations of these workers, newest first.
async fn aspirations_of(
    ctx: &AppContext,
    worker_pids: &[Uuid],
) -> Result<Vec<worker_aspirations::Model>> {
    Ok(sqlx::query_as!(
        worker_aspirations::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, kind,
                  framework_slug, role_profile_pid, esco_occupation_uri, role_label, skill_pid, target_level, horizon,
                  status, note, visibility, recorded_by, on_behalf, deleted_at AS "deleted_at: _"
             FROM worker_aspirations
            WHERE worker_pid = ANY($1) AND deleted_at IS NULL
            ORDER BY id DESC"#,
        worker_pids,
    )
    .fetch_all(db::pool(ctx))
    .await?)
}

async fn find_aspiration(ctx: &AppContext, pid: &str) -> Result<worker_aspirations::Model> {
    sqlx::query_as!(
        worker_aspirations::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, kind,
                  framework_slug, role_profile_pid, esco_occupation_uri, role_label, skill_pid, target_level, horizon,
                  status, note, visibility, recorded_by, on_behalf, deleted_at AS "deleted_at: _"
             FROM worker_aspirations
            WHERE pid = $1 AND deleted_at IS NULL"#,
        records::parse_pid(pid)?,
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)
}

/// `PUT /api/aspirations/{pid}` — update status, horizon, target, note, or
/// whether it is shared.
#[debug_handler]
async fn update_aspiration(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<AspirationUpdate>,
) -> Result<Response> {
    let row = find_aspiration(&ctx, &pid).await?;
    let worker = writable_worker(&ctx, &caller, &row.worker_pid.to_string()).await?;
    rules::validate_aspiration(
        &row.kind,
        payload.target_level.or(row.target_level),
        payload.horizon.as_deref().unwrap_or(&row.horizon),
        payload.status.as_deref().unwrap_or(&row.status),
        payload.note.as_deref().or(row.note.as_deref()),
    )
    .map_err(|e| unprocessable(&e))?;
    if let Some(visibility) = &payload.visibility {
        rules::validate_visibility(visibility).map_err(|e| unprocessable(&e))?;
    }
    let mut tx = db::pool(&ctx).begin().await?;
    let updated = sqlx::query!(
        "UPDATE worker_aspirations
            SET status = COALESCE($2, status), horizon = COALESCE($3, horizon),
                target_level = COALESCE($4, target_level), note = COALESCE($5, note),
                visibility = COALESCE($6, visibility)
          WHERE id = $1
          RETURNING pid, status, visibility",
        row.id,
        payload.status,
        payload.horizon,
        payload.target_level,
        payload.note,
        payload.visibility,
    )
    .fetch_one(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "worker",
        worker.pid,
        "aspiration_updated",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(
        serde_json::json!({ "pid": updated.pid, "status": updated.status, "visibility": updated.visibility }),
    )
}

/// `DELETE /api/aspirations/{pid}` — drop an aspiration (soft-delete).
#[debug_handler]
async fn delete_aspiration(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let row = find_aspiration(&ctx, &pid).await?;
    let worker = writable_worker(&ctx, &caller, &row.worker_pid.to_string()).await?;
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "UPDATE worker_aspirations SET deleted_at = now() WHERE id = $1",
        row.id
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "worker",
        worker.pid,
        "aspiration_dropped",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::empty_json()
}

/// The career routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/workers/{pid}/role-history", get(role_history))
        .add(
            "/workers/{pid}/framework-roles/{framework}/past",
            post(add_past_role),
        )
        .add("/workers/{pid}/skill-history", get(skill_history))
        .add("/workers/{pid}/skill-history/past", post(add_past_skill))
        .add("/workers/{pid}/skills-as-of", get(skills_as_of))
        .add("/workers/{pid}/aspirations", post(add_aspiration))
        .add("/workers/{pid}/aspirations", get(list_aspirations))
        .add(
            "/workers/{pid}/downline-aspirations",
            get(downline_aspirations),
        )
        .add("/aspirations/{pid}", put(update_aspiration))
        .add("/aspirations/{pid}", delete(delete_aspiration))
}
