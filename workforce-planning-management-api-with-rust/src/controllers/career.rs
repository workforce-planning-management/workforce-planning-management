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

use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use loco_rs::prelude::*;
use sea_orm::{ActiveValue, QueryOrder};
use serde::Deserialize;
use std::collections::BTreeMap;
use uuid::Uuid;

use super::{record_rejection, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::models::_entities::{
    esco_occupation_skills, role_profiles, role_skill_requirements, skill_external_refs, skills,
    worker_aspirations, worker_framework_roles, worker_skill_history, worker_skills, workers,
};
use crate::models::audit_logs::Model as Audit;
use crate::models::records;
use crate::rules::career::{self as rules, Interval};
use crate::tasks::import_esco::ESCO_SLUG;
use crate::tasks::import_framework::PCF_SLUG;

fn utc(t: sea_orm::prelude::DateTimeWithTimeZone) -> DateTime<Utc> {
    t.with_timezone(&Utc)
}

fn start_of(day: NaiveDate) -> DateTime<Utc> {
    Utc.from_utc_datetime(&day.and_hms_opt(0, 0, 0).unwrap_or_default())
}

/// A worker, authorized for a write to their record.
async fn writable_worker(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: &str,
) -> Result<workers::Model> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(pid)?).await?;
    auth::authorize_record(
        caller,
        authentication_verifier::Action::Write,
        &auth::worker_resource_attrs(&worker),
    )
    .map_err(record_rejection)?;
    Ok(worker)
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
    axum::extract::Query(query): axum::extract::Query<FrameworkQuery>,
) -> Result<Response> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    let mut select = worker_framework_roles::Entity::find()
        .filter(worker_framework_roles::Column::WorkerPid.eq(worker.pid));
    if let Some(framework) = &query.framework {
        select = select.filter(worker_framework_roles::Column::FrameworkSlug.eq(framework));
    }
    let rows = select
        .order_by_desc(worker_framework_roles::Column::StartedAt)
        .all(&ctx.db)
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
        let profile = role_profiles::Entity::find()
            .filter(role_profiles::Column::Pid.eq(id))
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
    let existing = worker_framework_roles::Entity::find()
        .filter(worker_framework_roles::Column::WorkerPid.eq(worker.pid))
        .filter(worker_framework_roles::Column::FrameworkSlug.eq(&framework))
        .all(&ctx.db)
        .await?;
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
    let row = worker_framework_roles::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        worker_pid: ActiveValue::set(worker.pid),
        framework_slug: ActiveValue::set(framework.clone()),
        role_profile_pid: ActiveValue::set(profile_pid),
        esco_occupation_uri: ActiveValue::set(occupation_uri),
        role_label: ActiveValue::set(label),
        selected_on: ActiveValue::set(payload.started_on),
        started_at: ActiveValue::set(interval.start.into()),
        ended_at: ActiveValue::set(interval.end.map(Into::into)),
        recorded_by: ActiveValue::set(caller.actor().map(ToString::to_string)),
        on_behalf: ActiveValue::set(auth::acting_for_other(&caller, &worker.person_ref)),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;
    Audit::record(
        &ctx.db,
        "worker",
        worker.pid,
        "past_role_added",
        caller.actor(),
        None,
    )
    .await?;
    format::json(role_row(&row))
}

// ─── Skill history ──────────────────────────────────────────────────────────

/// Query for the skill timeline.
#[derive(Debug, Deserialize)]
struct SkillQuery {
    skill_pid: Option<Uuid>,
}

async fn skill_names(ctx: &AppContext) -> Result<BTreeMap<Uuid, String>> {
    Ok(skills::Entity::find()
        .all(&ctx.db)
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
    axum::extract::Query(query): axum::extract::Query<SkillQuery>,
) -> Result<Response> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    let mut select = worker_skill_history::Entity::find()
        .filter(worker_skill_history::Column::WorkerPid.eq(worker.pid));
    if let Some(skill) = query.skill_pid {
        select = select.filter(worker_skill_history::Column::SkillPid.eq(skill));
    }
    let rows = select
        .order_by_desc(worker_skill_history::Column::StartedAt)
        .all(&ctx.db)
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
    skills::Entity::find()
        .filter(skills::Column::Pid.eq(payload.skill_pid))
        .filter(skills::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    let interval = rules::validate_past(
        start_of(payload.started_on),
        start_of(payload.ended_on) + chrono::Duration::days(1),
        Utc::now(),
    )
    .map_err(|e| unprocessable(&e))?;
    let existing = worker_skill_history::Entity::find()
        .filter(worker_skill_history::Column::WorkerPid.eq(worker.pid))
        .filter(worker_skill_history::Column::SkillPid.eq(payload.skill_pid))
        .all(&ctx.db)
        .await?;
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
    let row = worker_skill_history::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        worker_pid: ActiveValue::set(worker.pid),
        skill_pid: ActiveValue::set(payload.skill_pid),
        proficiency: ActiveValue::set(payload.proficiency),
        started_at: ActiveValue::set(interval.start.into()),
        ended_at: ActiveValue::set(interval.end.map(Into::into)),
        source: ActiveValue::set("retrospective".to_string()),
        recorded_by: ActiveValue::set(caller.actor().map(ToString::to_string)),
        on_behalf: ActiveValue::set(auth::acting_for_other(&caller, &worker.person_ref)),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;
    Audit::record(
        &ctx.db,
        "worker",
        worker.pid,
        "past_skill_added",
        caller.actor(),
        None,
    )
    .await?;
    format::json(serde_json::json!({ "pid": row.pid }))
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
    axum::extract::Query(query): axum::extract::Query<AsOfQuery>,
) -> Result<Response> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    let instant = start_of(query.at) + chrono::Duration::days(1) - chrono::Duration::seconds(1);
    let rows = worker_skill_history::Entity::find()
        .filter(worker_skill_history::Column::WorkerPid.eq(worker.pid))
        .all(&ctx.db)
        .await?;
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
    let roles = worker_framework_roles::Entity::find()
        .filter(worker_framework_roles::Column::WorkerPid.eq(worker.pid))
        .all(&ctx.db)
        .await?;
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
    /// Whether others (a manager, HR) may see it. Default: private.
    #[serde(default)]
    shared: bool,
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
    rules::validate_aspiration(
        &payload.kind,
        payload.target_level,
        &payload.horizon,
        &status,
        payload.note.as_deref(),
    )
    .map_err(|e| unprocessable(&e))?;
    let (framework, profile, occupation, role_label, skill) = if payload.kind == "role" {
        let framework = payload
            .framework_slug
            .clone()
            .filter(|f| crate::rules::framework_roles::valid_framework(f))
            .ok_or_else(|| unprocessable("a role aspiration needs a framework_slug"))?;
        if framework == PCF_SLUG {
            let id = payload
                .role_profile_pid
                .ok_or_else(|| unprocessable("role_profile_pid is required for the UK GDAD PCF"))?;
            let profile = role_profiles::Entity::find()
                .filter(role_profiles::Column::Pid.eq(id))
                .filter(role_profiles::Column::DeletedAt.is_null())
                .filter(role_profiles::Column::FrameworkSlug.eq(PCF_SLUG))
                .one(&ctx.db)
                .await?
                .ok_or_else(|| unprocessable("that is not a UK GDAD PCF role level"))?;
            (
                Some(framework),
                Some(profile.pid),
                None,
                Some(profile.job_title),
                None,
            )
        } else {
            let uri = payload
                .occupation_uri
                .clone()
                .ok_or_else(|| unprocessable("occupation_uri is required for ESCO"))?;
            let occupation = super::esco::find_occupation(&ctx, &uri)
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
        skills::Entity::find()
            .filter(skills::Column::Pid.eq(id))
            .filter(skills::Column::DeletedAt.is_null())
            .one(&ctx.db)
            .await?
            .ok_or(Error::NotFound)?;
        (None, None, None, None, Some(id))
    };
    let row = worker_aspirations::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        worker_pid: ActiveValue::set(worker.pid),
        kind: ActiveValue::set(payload.kind.clone()),
        framework_slug: ActiveValue::set(framework),
        role_profile_pid: ActiveValue::set(profile),
        esco_occupation_uri: ActiveValue::set(occupation),
        role_label: ActiveValue::set(role_label),
        skill_pid: ActiveValue::set(skill),
        target_level: ActiveValue::set(payload.target_level),
        horizon: ActiveValue::set(payload.horizon.clone()),
        status: ActiveValue::set(status),
        note: ActiveValue::set(payload.note.clone()),
        shared: ActiveValue::set(payload.shared),
        recorded_by: ActiveValue::set(caller.actor().map(ToString::to_string)),
        on_behalf: ActiveValue::set(auth::acting_for_other(&caller, &worker.person_ref)),
        deleted_at: ActiveValue::set(None),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;
    Audit::record(
        &ctx.db,
        "worker",
        worker.pid,
        "aspiration_added",
        caller.actor(),
        None,
    )
    .await?;
    format::json(serde_json::json!({ "pid": row.pid }))
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
        let reqs = role_skill_requirements::Entity::find()
            .filter(role_skill_requirements::Column::RoleProfilePid.eq(profile))
            .all(&ctx.db)
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
        let essential: Vec<String> = esco_occupation_skills::Entity::find()
            .filter(esco_occupation_skills::Column::OccupationUri.eq(uri))
            .filter(esco_occupation_skills::Column::Relation.eq("essential"))
            .all(&ctx.db)
            .await?
            .into_iter()
            .map(|r| r.skill_uri)
            .collect();
        let links: BTreeMap<String, Uuid> = skill_external_refs::Entity::find()
            .filter(skill_external_refs::Column::FrameworkSlug.eq(ESCO_SLUG))
            .all(&ctx.db)
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

/// `GET /api/workers/{pid}/aspirations` — the person's aspirations and growth
/// ideas, each with how they stand against it today. **Private by default:**
/// the person sees all of theirs; anyone else sees only those shared.
#[debug_handler]
async fn list_aspirations(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    let is_self = !auth::acting_for_other(&caller, &worker.person_ref);
    let rows = worker_aspirations::Entity::find()
        .filter(worker_aspirations::Column::WorkerPid.eq(worker.pid))
        .filter(worker_aspirations::Column::DeletedAt.is_null())
        .order_by_desc(worker_aspirations::Column::Id)
        .all(&ctx.db)
        .await?;
    let declared: BTreeMap<Uuid, i32> = worker_skills::Entity::find()
        .filter(worker_skills::Column::WorkerPid.eq(worker.pid))
        .filter(worker_skills::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|d| (d.skill_pid, d.proficiency))
        .collect();
    let names = skill_names(&ctx).await?;
    let mut out = Vec::new();
    let mut hidden = 0usize;
    for a in &rows {
        if !rules::can_view(is_self, a.shared) {
            hidden += 1;
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
            "shared": a.shared,
            "recorded_by": a.recorded_by,
            "on_behalf": a.on_behalf,
            "progress": progress_toward(&ctx, worker.pid, a, &declared).await?,
        }));
    }
    format::json(serde_json::json!({
        "viewer_is_the_person": is_self,
        "aspirations": out,
        "private_hidden": hidden,
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
    shared: Option<bool>,
}

async fn find_aspiration(ctx: &AppContext, pid: &str) -> Result<worker_aspirations::Model> {
    worker_aspirations::Entity::find()
        .filter(worker_aspirations::Column::Pid.eq(records::parse_pid(pid)?))
        .filter(worker_aspirations::Column::DeletedAt.is_null())
        .one(&ctx.db)
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
    let mut active: worker_aspirations::ActiveModel = row.into();
    if let Some(status) = payload.status {
        active.status = ActiveValue::set(status);
    }
    if let Some(horizon) = payload.horizon {
        active.horizon = ActiveValue::set(horizon);
    }
    if payload.target_level.is_some() {
        active.target_level = ActiveValue::set(payload.target_level);
    }
    if payload.note.is_some() {
        active.note = ActiveValue::set(payload.note);
    }
    if let Some(shared) = payload.shared {
        active.shared = ActiveValue::set(shared);
    }
    let updated = active.update(&ctx.db).await?;
    Audit::record(
        &ctx.db,
        "worker",
        worker.pid,
        "aspiration_updated",
        caller.actor(),
        None,
    )
    .await?;
    format::json(
        serde_json::json!({ "pid": updated.pid, "status": updated.status, "shared": updated.shared }),
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
    let mut active: worker_aspirations::ActiveModel = row.into();
    active.deleted_at = ActiveValue::set(Some(Utc::now().into()));
    active.update(&ctx.db).await?;
    Audit::record(
        &ctx.db,
        "worker",
        worker.pid,
        "aspiration_dropped",
        caller.actor(),
        None,
    )
    .await?;
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
        .add("/aspirations/{pid}", put(update_aspiration))
        .add("/aspirations/{pid}", delete(delete_aspiration))
}
