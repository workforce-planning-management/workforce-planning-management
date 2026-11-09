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

use axa::extract::{Json, Path};
use axa::prelude::*;
use chrono::{NaiveDate, Utc};
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap, HashSet};
use uuid::Uuid;

use super::skill_gaps::{Evidence, load_evidence};
use super::{record_rejection, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::db;
use crate::models::_entities::skill_courses;
use crate::models::{memberships, records};
use crate::rules::skill_gap::{self as gap_rules, Gap, Status};
use crate::rules::training::{self as rules, Course, DEFAULT_HOURS_PER_LEVEL, Recommendation};
use crate::rules::training_economics::valid_fee;

// ─── The course catalogue ───────────────────────────────────────────────────

/// A skill, as the catalogue reads it.
struct Skill {
    pid: Uuid,
    hours_per_level: Option<i32>,
}

async fn find_skill(ctx: &AppContext, pid: &str) -> Result<Skill> {
    sqlx::query_as!(
        Skill,
        "SELECT pid, hours_per_level FROM skills WHERE pid = $1 AND deleted_at IS NULL",
        records::parse_pid(pid)?,
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)
}

/// A live course by pid.
async fn find_course(ctx: &AppContext, pid: &str) -> Result<skill_courses::Model> {
    sqlx::query_as!(
        skill_courses::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, skill_pid, course_ref, title, hours, levels,
                  deleted_at AS "deleted_at: _", fee_minor, fee_currency
             FROM skill_courses WHERE pid = $1 AND deleted_at IS NULL"#,
        records::parse_pid(pid)?,
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)
}

fn course_json(c: &skill_courses::Model) -> serde_json::Value {
    serde_json::json!({
        "pid": c.pid,
        "course_ref": c.course_ref,
        "lms_url": super::lms::course_url(&c.course_ref),
        "title": c.title,
        "hours": c.hours,
        "levels": c.levels,
        "fee_minor": c.fee_minor,
        "fee_currency": c.fee_currency,
    })
}

/// `GET /api/skills/{pid}/courses` — the courses that build a skill, and the
/// skill's own hours-per-level figure.
#[debug_handler]
async fn list_courses(State(ctx): State<AppContext>, Path(pid): Path<String>) -> Result<Response> {
    let skill = find_skill(&ctx, &pid).await?;
    let rows = sqlx::query_as!(
        skill_courses::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, skill_pid, course_ref, title, hours, levels,
                  deleted_at AS "deleted_at: _", fee_minor, fee_currency
             FROM skill_courses WHERE skill_pid = $1 AND deleted_at IS NULL ORDER BY id"#,
        skill.pid,
    )
    .fetch_all(db::pool(&ctx))
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
    /// The fee in minor units. Absent is unknown, never free; 0 is a recorded free course.
    #[serde(default)]
    fee_minor: Option<i64>,
    #[serde(default)]
    fee_currency: Option<String>,
}

/// `POST /api/skills/{pid}/courses` — add a catalogue course for a skill.
#[debug_handler]
async fn add_course(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<CoursePayload>,
) -> Result<Response> {
    let skill = find_skill(&ctx, &pid).await?;
    let levels = payload.levels.unwrap_or(1);
    if payload.course_ref.trim().is_empty() || payload.course_ref.len() > 200 {
        return Err(unprocessable(
            "course_ref is required (up to 200 characters)",
        ));
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
    valid_fee(payload.fee_minor, payload.fee_currency.as_deref()).map_err(|e| unprocessable(&e))?;
    let taken = sqlx::query_scalar!(
        r#"SELECT EXISTS (SELECT 1 FROM skill_courses WHERE skill_pid = $1 AND course_ref = $2 AND deleted_at IS NULL)
             AS "taken!""#,
        skill.pid,
        payload.course_ref.trim(),
    )
    .fetch_one(db::pool(&ctx))
    .await?;
    if taken {
        return Err(unprocessable(
            "that course is already listed for this skill",
        ));
    }
    let mut tx = db::pool(&ctx).begin().await?;
    let row = sqlx::query_as!(
        skill_courses::Model,
        r#"INSERT INTO skill_courses (pid, skill_pid, course_ref, title, hours, levels, fee_minor, fee_currency)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
           RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, skill_pid, course_ref, title, hours, levels,
                  deleted_at AS "deleted_at: _", fee_minor, fee_currency"#,
        Uuid::new_v4(),
        skill.pid,
        payload.course_ref.trim(),
        payload.title.trim(),
        payload.hours,
        levels,
        payload.fee_minor,
        payload.fee_currency,
    )
    .fetch_one(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "skill_course",
        row.pid,
        "created",
        caller.actor(),
        Some(course_json(&row)),
    )
    .await?;
    tx.commit().await?;
    format::json(course_json(&row))
}

/// `PUT /api/skill-courses/{pid}/fee` body. Both absent clears the fee back to unknown.
#[derive(Debug, Deserialize)]
struct FeePayload {
    #[serde(default)]
    fee_minor: Option<i64>,
    #[serde(default)]
    fee_currency: Option<String>,
}

/// `PUT /api/skill-courses/{pid}/fee` — record, change or clear a course's fee (WPM-R227). Both parts or neither.
#[debug_handler]
async fn set_fee(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<FeePayload>,
) -> Result<Response> {
    valid_fee(payload.fee_minor, payload.fee_currency.as_deref()).map_err(|e| unprocessable(&e))?;
    let row = find_course(&ctx, &pid).await?;
    let mut tx = db::pool(&ctx).begin().await?;
    let updated = sqlx::query_as!(
        skill_courses::Model,
        r#"UPDATE skill_courses SET fee_minor = $2, fee_currency = $3 WHERE id = $1
           RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, skill_pid, course_ref, title, hours, levels,
                  deleted_at AS "deleted_at: _", fee_minor, fee_currency"#,
        row.id,
        payload.fee_minor,
        payload.fee_currency,
    )
    .fetch_one(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "skill_course",
        updated.pid,
        "fee_set",
        caller.actor(),
        Some(serde_json::json!({
            "fee_minor": updated.fee_minor,
            "fee_currency": updated.fee_currency,
        })),
    )
    .await?;
    tx.commit().await?;
    format::json(course_json(&updated))
}

/// `DELETE /api/skill-courses/{pid}` — take a course off a skill (soft-delete).
#[debug_handler]
async fn remove_course(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let row = find_course(&ctx, &pid).await?;
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "UPDATE skill_courses SET deleted_at = $2 WHERE id = $1",
        row.id,
        Utc::now()
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "skill_course",
        row.pid,
        "deleted",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
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
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<HoursPayload>,
) -> Result<Response> {
    let skill = find_skill(&ctx, &pid).await?;
    if payload
        .hours_per_level
        .is_some_and(|h| !(1..=500).contains(&h))
    {
        return Err(unprocessable("hours_per_level must be between 1 and 500"));
    }
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "UPDATE skills SET hours_per_level = $2 WHERE pid = $1",
        skill.pid,
        payload.hours_per_level,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "skill",
        skill.pid,
        "updated",
        caller.actor(),
        Some(serde_json::json!({ "hours_per_level": payload.hours_per_level })),
    )
    .await?;
    tx.commit().await?;
    format::json(serde_json::json!({
        "skill_pid": skill.pid,
        "hours_per_level": payload.hours_per_level,
    }))
}

// ─── Recommendations ────────────────────────────────────────────────────────

/// A person below a need, as the demand view reads them.
struct Staff {
    pid: Uuid,
    department: String,
}

/// What recommendations need beyond the gaps themselves.
pub(crate) struct Catalogue {
    courses: HashMap<Uuid, Vec<Course>>,
    hours_per_level: HashMap<Uuid, u32>,
    names: HashMap<Uuid, String>,
    completed: HashMap<Uuid, HashSet<String>>,
}

pub(crate) async fn load_catalogue(ctx: &AppContext, worker_pids: &[Uuid]) -> Result<Catalogue> {
    let mut courses: HashMap<Uuid, Vec<Course>> = HashMap::new();
    let db = db::pool(ctx);
    for c in sqlx::query!(
        "SELECT skill_pid, course_ref, title, hours, levels FROM skill_courses WHERE deleted_at IS NULL ORDER BY id"
    )
    .fetch_all(db)
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
    for s in sqlx::query!("SELECT pid, name, hours_per_level FROM skills")
        .fetch_all(db)
        .await?
    {
        if let Some(h) = s.hours_per_level.and_then(|h| u32::try_from(h).ok()) {
            hours_per_level.insert(s.pid, h);
        }
        names.insert(s.pid, s.name);
    }
    let mut completed: HashMap<Uuid, HashSet<String>> = HashMap::new();
    for t in sqlx::query!(
        "SELECT worker_pid, course_ref FROM training_enrollments
          WHERE deleted_at IS NULL AND status = 'completed' AND worker_pid = ANY($1)",
        worker_pids,
    )
    .fetch_all(db)
    .await?
    {
        completed
            .entry(t.worker_pid)
            .or_default()
            .insert(t.course_ref);
    }
    Ok(Catalogue {
        courses,
        hours_per_level,
        names,
        completed,
    })
}

impl Catalogue {
    /// The training that would close `gap` for `worker`.
    pub(crate) fn recommend(&self, worker: Uuid, gap: &Gap) -> Recommendation {
        let empty = HashSet::new();
        rules::recommend(
            u32::try_from(gap.shortfall.unwrap_or(0)).unwrap_or(0),
            self.courses.get(&gap.skill).map_or(&[][..], Vec::as_slice),
            self.completed.get(&worker).unwrap_or(&empty),
            self.hours_per_level
                .get(&gap.skill)
                .copied()
                .unwrap_or(DEFAULT_HOURS_PER_LEVEL),
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
    axa::extract::Query(q): axa::extract::Query<PlanQuery>,
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
    let weekly =
        rules::weekly_hours(q.weekly_hours, worker.fte_percent).map_err(|e| unprocessable(&e))?;
    let start = q.start.unwrap_or_else(|| Utc::now().date_naive());

    let evidence = load_evidence(&ctx, &[worker.pid], own_view).await?;
    let mut gaps = evidence.gaps_for(worker.pid);
    gap_rules::rank(&mut gaps);
    let catalogue = load_catalogue(&ctx, &[worker.pid]).await?;
    let below: Vec<&Gap> = gaps.iter().filter(|g| g.status == Status::Below).collect();
    let recs: Vec<Recommendation> = below
        .iter()
        .map(|g| catalogue.recommend(worker.pid, g))
        .collect();
    let hours: Vec<u32> = recs.iter().map(|r| r.hours).collect();
    let slots = rules::schedule(&hours, weekly, start);
    // `SFIAplus` pointers (WPM-R112): resources the deployer is licensed for, for the level each skill is needed at. Not booked.
    let pointers = crate::controllers::sfia::pointers_for(
        &ctx.db,
        &below
            .iter()
            .map(|g| (g.skill, g.required))
            .collect::<Vec<_>>(),
    )
    .await?;

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
                "sfiaplus_pointers": pointers.get(&g.skill).cloned().unwrap_or_default(),
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
        "derivation": "For each skill the person is BELOW (declared, under the need), in priority \
                       order: catalogue courses they have not completed, cheapest per level \
                       first, and — for any levels the courses leave — the skill's hours-per-level \
                       planning figure (or the service default). Scheduled one after another at \
                       the weekly pace. A skill they have not declared gets no hours: it is listed \
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
    axa::extract::Query(q): axa::extract::Query<DemandQuery>,
) -> Result<Response> {
    let scope = memberships::scope_organization_refs(&ctx.db, caller.claims()).await?;
    let today = Utc::now().date_naive();
    let department = q.department.as_deref();
    // Only people below a need can add hours, so only they are read whole; everyone else is counted.
    let (considered, pids) =
        super::gap_sql::workers_below_a_need(&ctx, scope.as_deref(), department, today).await?;
    let by_pid: HashMap<Uuid, Staff> = sqlx::query_as!(
        Staff,
        "SELECT pid, department FROM workers WHERE pid = ANY($1)",
        &pids,
    )
    .fetch_all(db::pool(&ctx))
    .await?
    .into_iter()
    .map(|w| (w.pid, w))
    .collect();
    let staff: Vec<&Staff> = pids.iter().filter_map(|p| by_pid.get(p)).collect();
    let evidence: Evidence = load_evidence(&ctx, &pids, false).await?;
    let catalogue = load_catalogue(&ctx, &pids).await?;

    // skill → (people, hours, estimated-basis people); department → (people, hours)
    let mut by_skill: BTreeMap<Uuid, (usize, u32, usize)> = BTreeMap::new();
    let mut by_department: BTreeMap<String, (usize, u32)> = BTreeMap::new();
    let mut people_with_gaps = 0usize;
    let mut total_hours = 0u32;
    for w in staff {
        let mut person_hours = 0u32;
        for g in evidence
            .gaps_for(w.pid)
            .iter()
            .filter(|g| g.status == Status::Below)
        {
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
        "derivation": "Hours to close each person's BELOW skill gaps (declared, under the need), \
                       from catalogue courses they have not completed and the per-level planning \
                       figure for what those leave; summed. Skills people have not declared are \
                       unknown and add no hours. Planning estimates; counts and hours only — \
                       nobody is named.",
        "as_of": today,
        "workers_considered": considered,
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
        .add("/skill-courses/{pid}/fee", put(set_fee))
        .add("/skills/{pid}/training-hours", put(set_hours))
        .add("/workers/{pid}/training-plan", get(worker_plan))
        .add(
            "/workforce-intelligence/training-demand",
            get(training_demand),
        )
}
