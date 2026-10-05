//! **Training time recommendations** (`rules::training`): for each skill gap,
//! which catalogue courses would close it and how many hours that takes — and,
//! at a stated weekly pace, when a person would finish.
//!
//! - A skill's catalogue (`/api/skills/{pid}/courses`) lists courses that build
//!   it: hours and the levels each typically adds. A skill may also carry its
//!   own `hours_per_level` planning figure; otherwise the service default
//!   applies and the recommendation says it is an estimate.
//! - `GET /api/workers/{pid}/training-plan` — one person's plan, in priority
//!   order, built from their gaps ([`super::skill_gaps`]). **A skill they have
//!   not declared is not given hours** — it is listed under `assess_first`.
//! - `GET /api/workforce-intelligence/training-demand` — hours across the
//!   workforce by skill and department. Counts and hours only; nobody named.
//!
//! These are planning estimates, not promises.

use chrono::{NaiveDate, Utc};
use loco_rs::prelude::*;
use sea_orm::{ActiveValue, QueryOrder};
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap, HashSet};
use uuid::Uuid;

use super::skill_gaps::{Evidence, load_evidence};
use super::{record_rejection, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::models::_entities::{skill_courses, skills, training_enrollments, workers};
use crate::models::{memberships, records};
use crate::rules::metrics::is_employed_on;
use crate::rules::skill_gap::{self as gap_rules, Gap, Status};
use crate::rules::training::{self as rules, Course, DEFAULT_HOURS_PER_LEVEL, Recommendation};

// ─── The course catalogue ───────────────────────────────────────────────────

async fn find_skill(ctx: &AppContext, pid: &str) -> Result<skills::Model> {
    skills::Entity::find()
        .filter(skills::Column::Pid.eq(records::parse_pid(pid)?))
        .filter(skills::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)
}

fn course_json(c: &skill_courses::Model) -> serde_json::Value {
    serde_json::json!({
        "pid": c.pid,
        "course_ref": c.course_ref,
        "title": c.title,
        "hours": c.hours,
        "levels": c.levels,
    })
}

/// `GET /api/skills/{pid}/courses` — the courses that build a skill, and the
/// skill's own hours-per-level figure.
#[debug_handler]
async fn list_courses(State(ctx): State<AppContext>, Path(pid): Path<String>) -> Result<Response> {
    let skill = find_skill(&ctx, &pid).await?;
    let rows = skill_courses::Entity::find()
        .filter(skill_courses::Column::SkillPid.eq(skill.pid))
        .filter(skill_courses::Column::DeletedAt.is_null())
        .order_by_asc(skill_courses::Column::Id)
        .all(&ctx.db)
        .await?;
    format::json(serde_json::json!({
        "skill_pid": skill.pid,
        "hours_per_level": skill.hours_per_level,
        "default_hours_per_level": DEFAULT_HOURS_PER_LEVEL,
        "courses": rows.iter().map(course_json).collect::<Vec<_>>(),
    }))
}

/// `POST /api/skills/{pid}/courses` body.
#[derive(Debug, Deserialize)]
struct CoursePayload {
    course_ref: String,
    title: String,
    hours: i32,
    /// Levels it typically adds (default 1).
    #[serde(default)]
    levels: Option<i32>,
}

/// `POST /api/skills/{pid}/courses` — add a catalogue course for a skill.
#[debug_handler]
async fn add_course(
    State(ctx): State<AppContext>,
    Path(pid): Path<String>,
    Json(payload): Json<CoursePayload>,
) -> Result<Response> {
    let skill = find_skill(&ctx, &pid).await?;
    let levels = payload.levels.unwrap_or(1);
    if payload.course_ref.trim().is_empty() || payload.course_ref.len() > 200 {
        return Err(unprocessable("course_ref is required (up to 200 characters)"));
    }
    if payload.title.trim().is_empty() || payload.title.chars().count() > 200 {
        return Err(unprocessable("title is required (up to 200 characters)"));
    }
    if !(1..=1000).contains(&payload.hours) {
        return Err(unprocessable("hours must be between 1 and 1000"));
    }
    if !(1..=4).contains(&levels) {
        return Err(unprocessable("levels must be between 1 and 4"));
    }
    let taken = skill_courses::Entity::find()
        .filter(skill_courses::Column::SkillPid.eq(skill.pid))
        .filter(skill_courses::Column::CourseRef.eq(payload.course_ref.trim()))
        .filter(skill_courses::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .is_some();
    if taken {
        return Err(unprocessable("that course is already listed for this skill"));
    }
    let row = skill_courses::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        skill_pid: ActiveValue::set(skill.pid),
        course_ref: ActiveValue::set(payload.course_ref.trim().to_string()),
        title: ActiveValue::set(payload.title.trim().to_string()),
        hours: ActiveValue::set(payload.hours),
        levels: ActiveValue::set(levels),
        deleted_at: ActiveValue::set(None),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;
    format::json(course_json(&row))
}

/// `DELETE /api/skill-courses/{pid}` — take a course off a skill (soft-delete).
#[debug_handler]
async fn remove_course(State(ctx): State<AppContext>, Path(pid): Path<String>) -> Result<Response> {
    let row = skill_courses::Entity::find()
        .filter(skill_courses::Column::Pid.eq(records::parse_pid(&pid)?))
        .filter(skill_courses::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    let mut active: skill_courses::ActiveModel = row.into();
    active.deleted_at = ActiveValue::set(Some(Utc::now().into()));
    active.update(&ctx.db).await?;
    format::empty_json()
}

/// `PUT /api/skills/{pid}/training-hours` body.
#[derive(Debug, Deserialize)]
struct HoursPayload {
    /// Hours to gain one level; `null` returns to the service default.
    hours_per_level: Option<i32>,
}

/// `PUT /api/skills/{pid}/training-hours` — set (or clear) a skill's planning
/// figure for hours per proficiency level.
#[debug_handler]
async fn set_hours(
    State(ctx): State<AppContext>,
    Path(pid): Path<String>,
    Json(payload): Json<HoursPayload>,
) -> Result<Response> {
    let skill = find_skill(&ctx, &pid).await?;
    if payload.hours_per_level.is_some_and(|h| !(1..=500).contains(&h)) {
        return Err(unprocessable("hours_per_level must be between 1 and 500"));
    }
    let mut active: skills::ActiveModel = skill.into();
    active.hours_per_level = ActiveValue::set(payload.hours_per_level);
    let updated = active.update(&ctx.db).await?;
    format::json(serde_json::json!({
        "skill_pid": updated.pid,
        "hours_per_level": updated.hours_per_level,
    }))
}

// ─── Recommendations ────────────────────────────────────────────────────────

/// What recommendations need beyond the gaps themselves.
struct Catalogue {
    courses: HashMap<Uuid, Vec<Course>>,
    hours_per_level: HashMap<Uuid, u32>,
    names: HashMap<Uuid, String>,
    completed: HashMap<Uuid, HashSet<String>>,
}

async fn load_catalogue(ctx: &AppContext, worker_pids: &[Uuid]) -> Result<Catalogue> {
    let mut courses: HashMap<Uuid, Vec<Course>> = HashMap::new();
    for c in skill_courses::Entity::find()
        .filter(skill_courses::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?
    {
        courses.entry(c.skill_pid).or_default().push(Course {
            course_ref: c.course_ref,
            title: c.title,
            hours: u32::try_from(c.hours).unwrap_or(0),
            levels: u32::try_from(c.levels).unwrap_or(1),
        });
    }
    let mut hours_per_level = HashMap::new();
    let mut names = HashMap::new();
    for s in skills::Entity::find().all(&ctx.db).await? {
        if let Some(h) = s.hours_per_level.and_then(|h| u32::try_from(h).ok()) {
            hours_per_level.insert(s.pid, h);
        }
        names.insert(s.pid, s.name);
    }
    let mut completed: HashMap<Uuid, HashSet<String>> = HashMap::new();
    for t in training_enrollments::Entity::find()
        .filter(training_enrollments::Column::WorkerPid.is_in(worker_pids.to_vec()))
        .filter(training_enrollments::Column::DeletedAt.is_null())
        .filter(training_enrollments::Column::Status.eq("completed"))
        .all(&ctx.db)
        .await?
    {
        completed.entry(t.worker_pid).or_default().insert(t.course_ref);
    }
    Ok(Catalogue { courses, hours_per_level, names, completed })
}

impl Catalogue {
    /// The training that would close `gap` for `worker`.
    fn recommend(&self, worker: Uuid, gap: &Gap) -> Recommendation {
        let empty = HashSet::new();
        rules::recommend(
            u32::try_from(gap.shortfall.unwrap_or(0)).unwrap_or(0),
            self.courses.get(&gap.skill).map_or(&[][..], Vec::as_slice),
            self.completed.get(&worker).unwrap_or(&empty),
            self.hours_per_level.get(&gap.skill).copied().unwrap_or(DEFAULT_HOURS_PER_LEVEL),
        )
    }
}

/// Query for a plan.
#[derive(Debug, Deserialize)]
struct PlanQuery {
    /// Weekly training hours to assume (1–40); default from the person's FTE.
    weekly_hours: Option<i64>,
    /// First day of the plan (default today).
    start: Option<NaiveDate>,
}

/// `GET /api/workers/{pid}/training-plan?weekly_hours=&start=` — one person's
/// training plan: for each skill they are *below*, in priority order, the
/// courses and hours that would close it, scheduled one after another at the
/// weekly pace. Skills they have not declared are under `assess_first`, with
/// no hours.
#[debug_handler]
async fn worker_plan(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    axum::extract::Query(q): axum::extract::Query<PlanQuery>,
) -> Result<Response> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    auth::authorize_record(
        &caller,
        authentication_verifier::Action::Read,
        &auth::worker_resource_attrs(&worker),
    )
    .map_err(record_rejection)?;
    let own_view = auth::authorize_record(
        &caller,
        authentication_verifier::Action::Write,
        &auth::worker_resource_attrs(&worker),
    )
    .is_ok();
    let weekly = rules::weekly_hours(q.weekly_hours, worker.fte_percent)
        .map_err(|e| unprocessable(&e))?;
    let start = q.start.unwrap_or_else(|| Utc::now().date_naive());

    let evidence = load_evidence(&ctx, &[worker.pid], own_view).await?;
    let mut gaps = evidence.gaps_for(worker.pid);
    gap_rules::rank(&mut gaps);
    let catalogue = load_catalogue(&ctx, &[worker.pid]).await?;
    let below: Vec<&Gap> = gaps.iter().filter(|g| g.status == Status::Below).collect();
    let recs: Vec<Recommendation> = below.iter().map(|g| catalogue.recommend(worker.pid, g)).collect();
    let hours: Vec<u32> = recs.iter().map(|r| r.hours).collect();
    let slots = rules::schedule(&hours, weekly, start);

    let plan: Vec<serde_json::Value> = below
        .iter()
        .zip(&recs)
        .zip(&slots)
        .map(|((g, r), slot)| {
            serde_json::json!({
                "skill_pid": g.skill,
                "skill": catalogue.names.get(&g.skill),
                "importance": g.importance,
                "required": g.required,
                "declared": g.declared,
                "shortfall": g.shortfall,
                "priority": gap_rules::priority(g),
                "recommendation": r,
                "starts_on": slot.starts_on,
                "ends_on": slot.ends_on,
                "weeks": slot.weeks,
                "cumulative_hours": slot.cumulative_hours,
            })
        })
        .collect();
    let assess_first: Vec<serde_json::Value> = gaps
        .iter()
        .filter(|g| g.status == Status::Undeclared)
        .map(|g| {
            serde_json::json!({
                "skill_pid": g.skill,
                "skill": catalogue.names.get(&g.skill),
                "required": g.required,
                "importance": g.importance,
            })
        })
        .collect();
    format::json(serde_json::json!({
        "derivation": "For each skill the person is BELOW (declared, under the need), in priority \\
                       order: catalogue courses they have not completed, cheapest per level \\
                       first, and — for any levels the courses leave — the skill's hours-per-level \\
                       planning figure (or the service default). Scheduled one after another at \\
                       the weekly pace. A skill they have not declared gets no hours: it is listed \\
                       under assess_first. Planning estimates, not promises.",
        "worker_pid": worker.pid,
        "includes_aspirations": own_view,
        "weekly_hours": weekly,
        "start": start,
        "total_hours": hours.iter().sum::<u32>(),
        "total_weeks": slots.iter().map(|s| s.weeks).sum::<u32>(),
        "finish_on": slots.iter().filter_map(|s| s.ends_on).max(),
        "plan": plan,
        "assess_first": assess_first,
    }))
}

/// Query for the workforce view.
#[derive(Debug, Deserialize)]
struct DemandQuery {
    department: Option<String>,
}

/// `GET /api/workforce-intelligence/training-demand?department=` — the
/// training hours the workforce's skill gaps would take, by skill and by
/// department: employed workers in the caller's organizations, each person's
/// skills they are below, recommended as for their own plan. Hours and counts
/// only; nobody is named, and aspirations are never included.
#[debug_handler]
async fn training_demand(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(q): axum::extract::Query<DemandQuery>,
) -> Result<Response> {
    let mut query = workers::Entity::find().filter(workers::Column::DeletedAt.is_null());
    if let Some(refs) = memberships::scope_organization_refs(&ctx.db, caller.claims()).await? {
        query = query.filter(workers::Column::OrganizationRef.is_in(refs));
    }
    let today = Utc::now().date_naive();
    let staff: Vec<workers::Model> = query
        .order_by_asc(workers::Column::Id)
        .all(&ctx.db)
        .await?
        .into_iter()
        .filter(|w| is_employed_on(today, w.hired_on, w.terminated_on))
        .filter(|w| q.department.as_deref().is_none_or(|d| w.department.eq_ignore_ascii_case(d)))
        .collect();
    let pids: Vec<Uuid> = staff.iter().map(|w| w.pid).collect();
    let evidence: Evidence = load_evidence(&ctx, &pids, false).await?;
    let catalogue = load_catalogue(&ctx, &pids).await?;

    // skill → (people, hours, estimated-basis people); department → (people, hours)
    let mut by_skill: BTreeMap<Uuid, (usize, u32, usize)> = BTreeMap::new();
    let mut by_department: BTreeMap<String, (usize, u32)> = BTreeMap::new();
    let mut people_with_gaps = 0usize;
    let mut total_hours = 0u32;
    for w in &staff {
        let mut person_hours = 0u32;
        for g in evidence.gaps_for(w.pid).iter().filter(|g| g.status == Status::Below) {
            let rec = catalogue.recommend(w.pid, g);
            let e = by_skill.entry(g.skill).or_default();
            e.0 += 1;
            e.1 += rec.hours;
            if rec.basis != rules::Basis::Courses {
                e.2 += 1;
            }
            person_hours += rec.hours;
        }
        if person_hours > 0 {
            people_with_gaps += 1;
            total_hours += person_hours;
            let d = by_department.entry(w.department.clone()).or_default();
            d.0 += 1;
            d.1 += person_hours;
        }
    }
    let mut skills_out: Vec<(Uuid, (usize, u32, usize))> = by_skill.into_iter().collect();
    skills_out.sort_by(|a, b| b.1.1.cmp(&a.1.1).then(a.0.cmp(&b.0)));
    let mut depts_out: Vec<(String, (usize, u32))> = by_department.into_iter().collect();
    depts_out.sort_by(|a, b| b.1.1.cmp(&a.1.1).then(a.0.cmp(&b.0)));
    #[allow(clippy::cast_precision_loss)] // small counts
    let average = (people_with_gaps > 0).then(|| f64::from(total_hours) / people_with_gaps as f64);
    format::json(serde_json::json!({
        "derivation": "Hours to close each person's BELOW skill gaps (declared, under the need), \\
                       from catalogue courses they have not completed and the per-level planning \\
                       figure for what those leave; summed. Skills people have not declared are \\
                       unknown and add no hours. Planning estimates; counts and hours only — \\
                       nobody is named.",
        "as_of": today,
        "workers_considered": staff.len(),
        "people_with_gaps": people_with_gaps,
        "total_hours": total_hours,
        "average_hours_per_person": average,
        "skills": skills_out.iter().map(|(s, (people, hours, estimated))| serde_json::json!({
            "skill_pid": s,
            "skill": catalogue.names.get(s),
            "people": people,
            "hours": hours,
            "people_on_estimate": estimated,
        })).collect::<Vec<_>>(),
        "departments": depts_out.iter().map(|(d, (people, hours))| serde_json::json!({
            "department": d, "people": people, "hours": hours,
        })).collect::<Vec<_>>(),
    }))
}

/// The training-time routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/skills/{pid}/courses", get(list_courses))
        .add("/skills/{pid}/courses", post(add_course))
        .add("/skill-courses/{pid}", delete(remove_course))
        .add("/skills/{pid}/training-hours", put(set_hours))
        .add("/workers/{pid}/training-plan", get(worker_plan))
        .add("/workforce-intelligence/training-demand", get(training_demand))
}
