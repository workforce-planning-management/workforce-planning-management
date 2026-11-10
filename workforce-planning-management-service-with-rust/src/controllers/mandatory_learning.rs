//! Statutory and mandatory learning: records, API and views (WPM-R170, WPM-D117, WPM-T315). HR writes requirements and records the
//! completions of training done elsewhere; a completed training enrolment for one of a requirement's courses counts as well. The person
//! sees their own status; HR sees the organization's roll-up (counts and a rate, with the floor). A person whose assignments do not record
//! the category an audience is defined by is `unknown`, counted apart. The status arithmetic is `rules::mandatory_learning`.
//! Spec: `spec/staff-records-and-pay.md`.

use std::collections::HashMap;

use axa::extract::{Json, Path, Query};
use axa::prelude::*;
use chrono::{NaiveDate, Utc};
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use super::{ensure_org_in_scope, ensure_valid, ensure_within_ceiling, forbidden, writable_worker};
use crate::auth::MaybeAuthUser;
use crate::db;
use crate::models::_entities::{learning_requirements, workers};
use crate::models::records;
use crate::rules::mandatory_learning as rules;

/// HR of the organization, or anyone when sign-in is off.
pub(crate) async fn ensure_hr(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    organization: &str,
) -> Result<()> {
    ensure_org_in_scope(ctx, caller, organization).await?;
    if !super::policies::privileged(ctx, caller, organization).await? {
        return Err(forbidden("this is done by HR"));
    }
    Ok(())
}

fn strings(value: &Value, key: &str) -> Vec<String> {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(ToString::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn audience_of(value: &Value) -> rules::Audience {
    rules::Audience {
        staff_groups: strings(value, "staff_groups"),
        job_roles: strings(value, "job_roles"),
        areas_of_work: strings(value, "areas_of_work"),
        positions: strings(value, "positions"),
    }
}

fn requirement_json(r: &learning_requirements::Model) -> Value {
    json!({
        "pid": r.pid, "organization_ref": r.organization_ref, "subject": r.subject, "basis": r.basis,
        "source": r.source, "refresher_months": r.refresher_months, "courses": r.courses,
        "audience": r.audience, "accepts_transferred": r.accepts_transferred, "retired_on": r.retired_on,
        "due_soon_days": r.due_soon_days,
    })
}

/// What one person's evidence for one requirement comes to on `today`.
struct Evaluation {
    status: rules::Status,
    last_completed_on: Option<NaiveDate>,
    expires_on: Option<NaiveDate>,
}

/// The deployment's due-soon window in calendar days: `WPM_LEARNING_DUE_SOON_DAYS`, 1 to 366, else 90 (WPM-T434).
fn due_soon_days() -> i64 {
    rules::due_soon_window(std::env::var("WPM_LEARNING_DUE_SOON_DAYS").ok().as_deref())
}

fn evaluate(
    req: &learning_requirements::Model,
    categories: &[rules::Categories],
    completions: &[rules::Completion],
    today: NaiveDate,
) -> Evaluation {
    let applies = rules::applies(&audience_of(&req.audience), categories);
    let refresher = req.refresher_months.and_then(|m| u32::try_from(m).ok());
    let status = rules::status(
        applies,
        completions,
        req.accepts_transferred,
        refresher,
        today,
        rules::window_for(req.due_soon_days, due_soon_days()),
    );
    let last = completions
        .iter()
        .filter(|c| c.on <= today && (req.accepts_transferred || !c.transferred))
        .map(|c| c.on)
        .max();
    Evaluation {
        status,
        last_completed_on: last,
        expires_on: last.and_then(|d| rules::expires_on(d, refresher)),
    }
}

fn status_name(s: rules::Status) -> &'static str {
    match s {
        rules::Status::Compliant => "compliant",
        rules::Status::DueSoon => "due_soon",
        rules::Status::Expired => "expired",
        rules::Status::NeverCompleted => "never_completed",
        rules::Status::NotApplicable => "not_applicable",
        rules::Status::Unknown => "unknown",
    }
}

/// The number of the one numbered position each worker holds. A person who holds none, one with no number, or more than one numbered
/// position is left out, so an audience that names positions reads `unknown` for them, never a guess (WPM-T434). An abolished
/// position is not held.
async fn position_numbers(ctx: &AppContext, pids: &[Uuid]) -> Result<HashMap<Uuid, String>> {
    let mut held: HashMap<Uuid, Vec<String>> = HashMap::new();
    for p in sqlx::query!(
        "SELECT holder_worker_pid, number FROM positions
          WHERE holder_worker_pid = ANY($1) AND state <> 'abolished' AND number IS NOT NULL
          ORDER BY id",
        pids,
    )
    .fetch_all(db::pool(ctx))
    .await?
    {
        if let (Some(holder), Some(number)) = (p.holder_worker_pid, p.number) {
            held.entry(holder).or_default().push(number);
        }
    }
    Ok(held
        .into_iter()
        .filter_map(|(w, mut n)| (n.len() == 1).then(|| (w, n.remove(0))))
        .collect())
}

/// The categories of each worker's assignments in force on `today`.
async fn categories_of(
    ctx: &AppContext,
    pids: &[Uuid],
    today: NaiveDate,
) -> Result<HashMap<Uuid, Vec<rules::Categories>>> {
    let mut out: HashMap<Uuid, Vec<rules::Categories>> = HashMap::new();
    let numbers = position_numbers(ctx, pids).await?;
    for a in sqlx::query!(
        "SELECT worker_pid, staff_group, job_role, area_of_work, starts_on, ends_on
           FROM assignments WHERE worker_pid = ANY($1) ORDER BY id",
        pids,
    )
    .fetch_all(db::pool(ctx))
    .await?
    {
        if a.starts_on <= today && a.ends_on.is_none_or(|e| e >= today) {
            out.entry(a.worker_pid)
                .or_default()
                .push(rules::Categories {
                    staff_group: a.staff_group,
                    job_role: a.job_role,
                    area_of_work: a.area_of_work,
                    position: numbers.get(&a.worker_pid).cloned(),
                });
        }
    }
    Ok(out)
}

/// The completions that count for each (worker, requirement): HR-recorded ones, and completed enrolments in one of the courses.
async fn completions_of(
    ctx: &AppContext,
    workers_in: &[workers::Model],
    reqs: &[learning_requirements::Model],
) -> Result<HashMap<(Uuid, Uuid), Vec<rules::Completion>>> {
    let pids: Vec<Uuid> = workers_in.iter().map(|w| w.pid).collect();
    let home: HashMap<Uuid, &str> = workers_in
        .iter()
        .map(|w| (w.pid, w.organization_ref.as_str()))
        .collect();
    let mut out: HashMap<(Uuid, Uuid), Vec<rules::Completion>> = HashMap::new();
    for c in sqlx::query!(
        "SELECT worker_pid, requirement_pid, completed_on, source_organization
           FROM learning_completions WHERE worker_pid = ANY($1) ORDER BY id",
        &pids,
    )
    .fetch_all(db::pool(ctx))
    .await?
    {
        let transferred = c
            .source_organization
            .as_deref()
            .is_some_and(|o| home.get(&c.worker_pid).is_some_and(|h| *h != o));
        out.entry((c.worker_pid, c.requirement_pid))
            .or_default()
            .push(rules::Completion {
                on: c.completed_on,
                transferred,
            });
    }
    let enrolments = sqlx::query!(
        "SELECT worker_pid, course_ref, completed_on FROM training_enrollments
          WHERE worker_pid = ANY($1) AND status = 'completed' AND deleted_at IS NULL
          ORDER BY id",
        &pids,
    )
    .fetch_all(db::pool(ctx))
    .await?;
    for e in enrolments {
        let Some(on) = e.completed_on else { continue };
        for r in reqs {
            if strings(&json!({ "c": r.courses }), "c").contains(&e.course_ref) {
                out.entry((e.worker_pid, r.pid))
                    .or_default()
                    .push(rules::Completion {
                        on,
                        transferred: false,
                    });
            }
        }
    }
    Ok(out)
}

async fn live_requirements(
    ctx: &AppContext,
    organization: &str,
    today: NaiveDate,
) -> Result<Vec<learning_requirements::Model>> {
    Ok(sqlx::query_as!(
        learning_requirements::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, organization_ref, subject, basis, source,
                  refresher_months, courses, audience, accepts_transferred, retired_on, due_soon_days, recorded_by
             FROM learning_requirements
            WHERE organization_ref = $1
            ORDER BY subject"#,
        organization,
    )
    .fetch_all(db::pool(ctx))
    .await?
    .into_iter()
        .filter(|r| r.retired_on.is_none_or(|d| d > today))
        .collect())
}

/// The requirement with this pid.
async fn requirement(ctx: &AppContext, pid: Uuid) -> Result<learning_requirements::Model> {
    sqlx::query_as!(
        learning_requirements::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, organization_ref, subject, basis, source,
                  refresher_months, courses, audience, accepts_transferred, retired_on, due_soon_days, recorded_by
             FROM learning_requirements WHERE pid = $1"#,
        pid,
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)
}

/// The live, not terminated workers of an organization, refused past the deployment's ceiling.
async fn current_staff(ctx: &AppContext, organization: &str) -> Result<Vec<workers::Model>> {
    let count = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "n!" FROM workers
            WHERE organization_ref = $1 AND deleted_at IS NULL AND terminated_on IS NULL"#,
        organization,
    )
    .fetch_one(db::pool(ctx))
    .await?;
    ensure_within_ceiling(count.unsigned_abs())?;
    Ok(sqlx::query_as!(
        workers::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, person_ref,
                  upstream_worker_ref, organization_ref, worker_number, display_name, status, employment_type,
                  fte_percent, department, job_title, location, work_jurisdiction, manager_pid, salary_minor,
                  salary_currency, hired_on, terminated_on, engagement_ends_on, deleted_at AS "deleted_at: _"
             FROM workers
            WHERE organization_ref = $1 AND deleted_at IS NULL AND terminated_on IS NULL
            ORDER BY id"#,
        organization,
    )
    .fetch_all(db::pool(ctx))
    .await?)
}

#[derive(Debug, Deserialize)]
struct OrgQuery {
    organization: String,
}

/// `GET /api/learning-requirements?organization=` — the requirements in force. Anyone in the organization's scope may read them (a
/// requirement is policy, not a person's record).
#[debug_handler]
async fn list(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Query(q): Query<OrgQuery>,
) -> Result<Response> {
    ensure_org_in_scope(&ctx, &caller, &q.organization).await?;
    let rows = live_requirements(&ctx, &q.organization, Utc::now().date_naive()).await?;
    format::json(rows.iter().map(requirement_json).collect::<Vec<_>>())
}

#[derive(Debug, Deserialize)]
struct CreatePayload {
    organization: String,
    subject: String,
    basis: String,
    source: Option<String>,
    refresher_months: Option<u32>,
    /// This requirement's own due-soon window in calendar days, 1 to 366; absent means the deployment's window.
    due_soon_days: Option<u32>,
    #[serde(default)]
    courses: Vec<String>,
    #[serde(default)]
    staff_groups: Vec<String>,
    #[serde(default)]
    job_roles: Vec<String>,
    #[serde(default)]
    areas_of_work: Vec<String>,
    #[serde(default)]
    positions: Vec<String>,
    accepts_transferred: Option<bool>,
}

fn clean_list(items: &[String], what: &str, max: usize, problems: &mut Vec<String>) -> Vec<String> {
    let out: Vec<String> = items
        .iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if out.len() > 50 {
        problems.push(format!("{what}: at most 50 entries"));
    }
    if out.iter().any(|s| s.chars().count() > max) {
        problems.push(format!("{what}: each entry is at most {max} characters"));
    }
    out
}

/// `POST /api/learning-requirements` — HR adds a requirement. A statutory one names its source; a refresher is at least a month.
#[debug_handler]
async fn create(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(p): Json<CreatePayload>,
) -> Result<Response> {
    ensure_hr(&ctx, &caller, &p.organization).await?;
    let subject = p.subject.trim().to_string();
    let source = p
        .source
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string);
    let mut problems = rules::requirement_problems(&p.basis, source.as_deref(), p.refresher_months);
    if subject.is_empty() || subject.chars().count() > 120 {
        problems.push("subject must be 1 to 120 characters".to_string());
    }
    if source.as_ref().is_some_and(|s| s.chars().count() > 200) {
        problems.push("source must be at most 200 characters".to_string());
    }
    if p.refresher_months.is_some_and(|m| m > 600) {
        problems.push("refresher_months must be at most 600".to_string());
    }
    if p.due_soon_days.is_some_and(|d| !(1..=366).contains(&d)) {
        problems.push("due_soon_days must be 1 to 366".to_string());
    }
    let courses = clean_list(&p.courses, "courses", 255, &mut problems);
    let audience = json!({
        "staff_groups": clean_list(&p.staff_groups, "staff_groups", 60, &mut problems),
        "job_roles": clean_list(&p.job_roles, "job_roles", 60, &mut problems),
        "areas_of_work": clean_list(&p.areas_of_work, "areas_of_work", 60, &mut problems),
        "positions": clean_list(&p.positions, "positions", 60, &mut problems),
    });
    ensure_valid(&problems)?;
    let pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO learning_requirements (pid, organization_ref, subject, basis, source, refresher_months, courses, audience,
                                            accepts_transferred, due_soon_days, recorded_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
        pid,
        p.organization,
        subject,
        p.basis,
        source,
        p.refresher_months.and_then(|m| i32::try_from(m).ok()),
        json!(courses),
        audience,
        p.accepts_transferred.unwrap_or(true),
        p.due_soon_days.and_then(|d| i32::try_from(d).ok()),
        caller.actor(),
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "learning_requirement",
        pid,
        "learning_requirement_added",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(json!({ "pid": pid }))
}

#[derive(Debug, Deserialize)]
struct CompletionPayload {
    worker_pid: Uuid,
    completed_on: NaiveDate,
    source: Option<String>,
    source_organization: Option<String>,
}

/// `POST /api/learning-requirements/{pid}/completions` — HR records training done elsewhere. The date is not in the future. A
/// `source_organization` other than the worker's own marks it transferred (WPM-R167), counted only where the requirement accepts it.
#[debug_handler]
async fn record_completion(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<Uuid>,
    Json(p): Json<CompletionPayload>,
) -> Result<Response> {
    let req = requirement(&ctx, pid).await?;
    ensure_hr(&ctx, &caller, &req.organization_ref).await?;
    let worker = records::worker_by_pid(&ctx, p.worker_pid).await?;
    if worker.organization_ref != req.organization_ref {
        return Err(Error::NotFound);
    }
    let mut problems = Vec::new();
    if p.completed_on > Utc::now().date_naive() {
        problems.push("completed_on must not be in the future".to_string());
    }
    let source = p
        .source
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string);
    if source.as_ref().is_some_and(|s| s.chars().count() > 120) {
        problems.push("source must be at most 120 characters".to_string());
    }
    ensure_valid(&problems)?;
    let id = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO learning_completions (pid, requirement_pid, worker_pid, completed_on, source, source_organization, recorded_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
        id,
        req.pid,
        worker.pid,
        p.completed_on,
        source,
        p.source_organization
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty()),
        caller.actor(),
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "learning_completion",
        id,
        "learning_completion_recorded",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(json!({ "pid": id }))
}

/// `GET /api/workers/{pid}/mandatory-learning` — one person's status against each requirement in force: the person themself and HR.
#[debug_handler]
async fn of_worker(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = writable_worker(&ctx, &caller, &pid).await?;
    let today = Utc::now().date_naive();
    let reqs = live_requirements(&ctx, &worker.organization_ref, today).await?;
    let cats = categories_of(&ctx, &[worker.pid], today).await?;
    let comps = completions_of(&ctx, std::slice::from_ref(&worker), &reqs).await?;
    let mine = cats.get(&worker.pid).cloned().unwrap_or_default();
    let rows: Vec<Value> = reqs
        .iter()
        .map(|r| {
            let done = comps
                .get(&(worker.pid, r.pid))
                .cloned()
                .unwrap_or_default();
            let e = evaluate(r, &mine, &done, today);
            json!({
                "requirement_pid": r.pid, "subject": r.subject, "basis": r.basis, "source": r.source,
                "status": status_name(e.status), "last_completed_on": e.last_completed_on, "expires_on": e.expires_on,
            })
        })
        .collect();
    format::json(json!({ "worker_pid": worker.pid, "as_of": today, "requirements": rows }))
}

#[derive(Debug, Deserialize)]
struct RollupQuery {
    organization: String,
    /// `department` or `staff_group`: roll up per group as well, each group meeting the floor on its own.
    by: Option<String>,
}

/// The counts of one roll-up as JSON fields (hidden says nothing else).
fn rollup_fields(r: &rules::Rollup) -> Value {
    match r {
        rules::Rollup::Hidden => json!({ "hidden": true }),
        rules::Rollup::Shown {
            compliant,
            due_soon,
            expired,
            never_completed,
            not_applicable,
            unknown,
            rate,
        } => json!({
            "hidden": false, "compliant": compliant, "due_soon": due_soon, "expired": expired,
            "never_completed": never_completed, "not_applicable": not_applicable,
            "unknown": unknown, "rate": rate,
        }),
    }
}

/// The groups each person is in on `today`, from the assignments in force: one entry per distinct department or staff group, so a
/// person with two contracts in two departments counts in both; `None` when the group is not recorded (or there is no assignment).
async fn groups_of(
    ctx: &AppContext,
    pids: &[Uuid],
    today: NaiveDate,
    by: &str,
) -> Result<HashMap<Uuid, Vec<Option<String>>>> {
    let mut out: HashMap<Uuid, Vec<Option<String>>> = HashMap::new();
    for a in sqlx::query!(
        "SELECT worker_pid, department, staff_group, starts_on, ends_on
           FROM assignments WHERE worker_pid = ANY($1) ORDER BY id",
        pids,
    )
    .fetch_all(db::pool(ctx))
    .await?
    {
        if a.starts_on <= today && a.ends_on.is_none_or(|e| e >= today) {
            let g = if by == "department" {
                Some(a.department).filter(|d| !d.trim().is_empty())
            } else {
                a.staff_group
            };
            let v = out.entry(a.worker_pid).or_default();
            if !v.contains(&g) {
                v.push(g);
            }
        }
    }
    Ok(out)
}

/// `GET /api/learning-requirements/rollup?organization=[&by=department|staff_group]` — HR: per requirement, counts per status and the
/// rate in date among those counted, with the floor. `unknown` is counted apart; a group under the floor shows nothing; a rate with
/// nothing to divide by is null. With `by`, the same per department or staff group, each group meeting the floor on its own (WPM-T434);
/// a group not recorded is its own row, and no one is named.
#[debug_handler]
async fn rollup(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Query(q): Query<RollupQuery>,
) -> Result<Response> {
    ensure_hr(&ctx, &caller, &q.organization).await?;
    let by = match q.by.as_deref() {
        None => None,
        Some(b @ ("department" | "staff_group")) => Some(b),
        Some(_) => {
            return Err(Error::BadRequest(
                "by must be department or staff_group".into(),
            ));
        }
    };
    let today = Utc::now().date_naive();
    let people = current_staff(&ctx, &q.organization).await?;
    let pids: Vec<Uuid> = people.iter().map(|w| w.pid).collect();
    let reqs = live_requirements(&ctx, &q.organization, today).await?;
    let cats = categories_of(&ctx, &pids, today).await?;
    let comps = completions_of(&ctx, &people, &reqs).await?;
    let groups = match by {
        Some(b) => groups_of(&ctx, &pids, today, b).await?,
        None => HashMap::new(),
    };
    let floor = crate::rules::equality_monitoring::DEFAULT_FLOOR;
    let none: Vec<rules::Categories> = Vec::new();
    let cells: Vec<Value> = reqs
        .iter()
        .map(|r| {
            let statuses: Vec<(Uuid, rules::Status)> = people
                .iter()
                .map(|w| {
                    let done = comps.get(&(w.pid, r.pid)).cloned().unwrap_or_default();
                    (
                        w.pid,
                        evaluate(r, cats.get(&w.pid).unwrap_or(&none), &done, today).status,
                    )
                })
                .collect();
            let all: Vec<rules::Status> = statuses.iter().map(|(_, s)| *s).collect();
            let mut cell =
                json!({ "requirement_pid": r.pid, "subject": r.subject, "basis": r.basis });
            if let (Some(obj), Value::Object(f)) = (
                cell.as_object_mut(),
                rollup_fields(&rules::rollup(&all, floor)),
            ) {
                obj.extend(f);
            }
            if by.is_some() {
                let items: Vec<(Option<String>, rules::Status)> = statuses
                    .iter()
                    .flat_map(|(pid, s)| {
                        groups
                            .get(pid)
                            .cloned()
                            .unwrap_or_else(|| vec![None])
                            .into_iter()
                            .map(move |g| (g, *s))
                    })
                    .collect();
                let rows: Vec<Value> = rules::rollup_by(&items, floor)
                    .into_iter()
                    .map(|(g, r)| {
                        let mut row = json!({ "group": g });
                        if let (Some(obj), Value::Object(f)) =
                            (row.as_object_mut(), rollup_fields(&r))
                        {
                            obj.extend(f);
                        }
                        row
                    })
                    .collect();
                if let Some(obj) = cell.as_object_mut() {
                    obj.insert("groups".into(), Value::Array(rows));
                }
            }
            cell
        })
        .collect();
    format::json(
        json!({ "organization": q.organization, "as_of": today, "floor": floor, "by": by, "requirements": cells }),
    )
}

/// `GET /api/workers/{pid}/team-mandatory-learning` — for a manager: each person below them, by name, with their status against every
/// requirement that applies. Read as the manager `{pid}`; anyone else needs HR's access (WPM-R170: a work record, not a sensitive one,
/// so a manager sees their reports by name). Only people in the manager's own organization; unknown stays unknown.
#[debug_handler]
async fn team(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let manager = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    ensure_org_in_scope(&ctx, &caller, &manager.organization_ref).await?;
    if crate::auth::acting_for_other(&caller, &manager.person_ref)
        && !super::policies::privileged(&ctx, &caller, &manager.organization_ref).await?
    {
        return Err(forbidden("a team's learning is for that team's manager"));
    }
    let today = Utc::now().date_naive();
    let staff = current_staff(&ctx, &manager.organization_ref).await?;
    let mut reports_of: std::collections::BTreeMap<Uuid, Vec<Uuid>> =
        std::collections::BTreeMap::new();
    for w in &staff {
        if let Some(boss) = w.manager_pid {
            reports_of.entry(boss).or_default().push(w.pid);
        }
    }
    let below = crate::rules::org::downline(manager.pid, &reports_of);
    let by_pid: HashMap<Uuid, &workers::Model> = staff.iter().map(|w| (w.pid, w)).collect();
    let team: Vec<workers::Model> = below
        .iter()
        .filter_map(|(p, _)| by_pid.get(p).map(|w| (*w).clone()))
        .collect();
    let pids: Vec<Uuid> = team.iter().map(|w| w.pid).collect();
    let reqs = live_requirements(&ctx, &manager.organization_ref, today).await?;
    let cats = categories_of(&ctx, &pids, today).await?;
    let comps = completions_of(&ctx, &team, &reqs).await?;
    let none: Vec<rules::Categories> = Vec::new();
    let people: Vec<Value> = below
        .iter()
        .filter_map(|(p, depth)| by_pid.get(p).map(|w| (*w, *depth)))
        .map(|(w, depth)| {
            let items: Vec<Value> = reqs
                .iter()
                .filter_map(|r| {
                    let done = comps.get(&(w.pid, r.pid)).cloned().unwrap_or_default();
                    let e = evaluate(r, cats.get(&w.pid).unwrap_or(&none), &done, today);
                    (e.status != rules::Status::NotApplicable).then(|| {
                        json!({
                            "requirement_pid": r.pid, "subject": r.subject, "basis": r.basis,
                            "status": status_name(e.status), "expires_on": e.expires_on,
                        })
                    })
                })
                .collect();
            json!({
                "worker_pid": w.pid, "display_name": w.display_name,
                "report_kind": crate::rules::org::report_kind(depth).map(crate::rules::org::ReportKind::as_str),
                "requirements": items,
            })
        })
        .collect();
    format::json(json!({ "manager_pid": manager.pid, "as_of": today, "people": people }))
}

/// `POST /api/learning-requirements/{pid}/retire` — HR retires a requirement from today: it is no longer listed or counted, and the
/// completions recorded against it are kept. Retiring twice keeps the first date.
#[debug_handler]
async fn retire(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<Uuid>,
) -> Result<Response> {
    let req = requirement(&ctx, pid).await?;
    ensure_hr(&ctx, &caller, &req.organization_ref).await?;
    let on = req.retired_on.unwrap_or_else(|| Utc::now().date_naive());
    let mut tx = db::pool(&ctx).begin().await?;
    if req.retired_on.is_none() {
        sqlx::query!(
            "UPDATE learning_requirements SET retired_on = $2 WHERE id = $1",
            req.id,
            on,
        )
        .execute(&mut *tx)
        .await?;
        db::audit(
            &mut tx,
            "learning_requirement",
            req.pid,
            "learning_requirement_retired",
            caller.actor(),
            None,
        )
        .await?;
    }
    tx.commit().await?;
    format::json(json!({ "pid": req.pid, "retired_on": on }))
}

/// The routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/learning-requirements", get(list))
        .add("/learning-requirements", post(create))
        .add("/learning-requirements/rollup", get(rollup))
        .add("/learning-requirements/{pid}/retire", post(retire))
        .add(
            "/learning-requirements/{pid}/completions",
            post(record_completion),
        )
        .add("/workers/{pid}/mandatory-learning", get(of_worker))
        .add("/workers/{pid}/team-mandatory-learning", get(team))
}
