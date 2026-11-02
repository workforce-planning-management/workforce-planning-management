//! Statutory and mandatory learning: records, API and views (WPM-R170, WPM-D117, WPM-T315). HR writes requirements and records the
//! completions of training done elsewhere; a completed training enrolment for one of a requirement's courses counts as well. The person
//! sees their own status; HR sees the organization's roll-up (counts and a rate, with the floor). A person whose assignments do not record
//! the category an audience is defined by is `unknown`, counted apart. The status arithmetic is `rules::mandatory_learning`.
//! Spec: `spec/staff-records-and-pay.md`.

use std::collections::HashMap;

use chrono::{NaiveDate, Utc};
use loco_rs::prelude::*;
use sea_orm::{ActiveValue, QueryOrder, TransactionTrait};
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use super::{ensure_org_in_scope, ensure_scale, ensure_valid, forbidden, writable_worker};
use crate::auth::MaybeAuthUser;
use crate::models::_entities::{
    assignments, learning_completions, learning_requirements, training_enrollments, workers,
};
use crate::models::audit_logs::Model as Audit;
use crate::models::records;
use crate::rules::mandatory_learning as rules;

/// HR of the organization, or anyone when sign-in is off.
async fn ensure_hr(ctx: &AppContext, caller: &MaybeAuthUser, organization: &str) -> Result<()> {
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
    })
}

/// What one person's evidence for one requirement comes to on `today`.
struct Evaluation {
    status: rules::Status,
    last_completed_on: Option<NaiveDate>,
    expires_on: Option<NaiveDate>,
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

/// The categories of each worker's assignments in force on `today`.
async fn categories_of(
    ctx: &AppContext,
    pids: &[Uuid],
    today: NaiveDate,
) -> Result<HashMap<Uuid, Vec<rules::Categories>>> {
    let mut out: HashMap<Uuid, Vec<rules::Categories>> = HashMap::new();
    for a in assignments::Entity::find()
        .filter(assignments::Column::WorkerPid.is_in(pids.to_vec()))
        .all(&ctx.db)
        .await?
    {
        if a.starts_on <= today && a.ends_on.is_none_or(|e| e >= today) {
            out.entry(a.worker_pid)
                .or_default()
                .push(rules::Categories {
                    staff_group: a.staff_group,
                    job_role: a.job_role,
                    area_of_work: a.area_of_work,
                    // Position numbers are not a column yet, so an audience that names positions is `unknown`, never a guess.
                    position: None,
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
    for c in learning_completions::Entity::find()
        .filter(learning_completions::Column::WorkerPid.is_in(pids.clone()))
        .all(&ctx.db)
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
    let enrolments = training_enrollments::Entity::find()
        .filter(training_enrollments::Column::WorkerPid.is_in(pids))
        .filter(training_enrollments::Column::Status.eq("completed"))
        .filter(training_enrollments::Column::DeletedAt.is_null())
        .all(&ctx.db)
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
    Ok(learning_requirements::Entity::find()
        .filter(learning_requirements::Column::OrganizationRef.eq(organization))
        .order_by_asc(learning_requirements::Column::Subject)
        .all(&ctx.db)
        .await?
        .into_iter()
        .filter(|r| r.retired_on.is_none_or(|d| d > today))
        .collect())
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
    let courses = clean_list(&p.courses, "courses", 255, &mut problems);
    let audience = json!({
        "staff_groups": clean_list(&p.staff_groups, "staff_groups", 60, &mut problems),
        "job_roles": clean_list(&p.job_roles, "job_roles", 60, &mut problems),
        "areas_of_work": clean_list(&p.areas_of_work, "areas_of_work", 60, &mut problems),
        "positions": clean_list(&p.positions, "positions", 60, &mut problems),
    });
    ensure_valid(&problems)?;
    let pid = Uuid::new_v4();
    let txn = ctx.db.begin().await?;
    learning_requirements::ActiveModel {
        pid: ActiveValue::set(pid),
        organization_ref: ActiveValue::set(p.organization.clone()),
        subject: ActiveValue::set(subject),
        basis: ActiveValue::set(p.basis.clone()),
        source: ActiveValue::set(source),
        refresher_months: ActiveValue::set(p.refresher_months.and_then(|m| i32::try_from(m).ok())),
        courses: ActiveValue::set(json!(courses)),
        audience: ActiveValue::set(audience),
        accepts_transferred: ActiveValue::set(p.accepts_transferred.unwrap_or(true)),
        recorded_by: ActiveValue::set(caller.actor().map(ToString::to_string)),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    Audit::record(
        &txn,
        "learning_requirement",
        pid,
        "learning_requirement_added",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
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
    let req = learning_requirements::Entity::find()
        .filter(learning_requirements::Column::Pid.eq(pid))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    ensure_hr(&ctx, &caller, &req.organization_ref).await?;
    let worker = records::find_worker(&ctx.db, p.worker_pid).await?;
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
    let txn = ctx.db.begin().await?;
    learning_completions::ActiveModel {
        pid: ActiveValue::set(id),
        requirement_pid: ActiveValue::set(req.pid),
        worker_pid: ActiveValue::set(worker.pid),
        completed_on: ActiveValue::set(p.completed_on),
        source: ActiveValue::set(source),
        source_organization: ActiveValue::set(
            p.source_organization
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty()),
        ),
        recorded_by: ActiveValue::set(caller.actor().map(ToString::to_string)),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    Audit::record(
        &txn,
        "learning_completion",
        id,
        "learning_completion_recorded",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
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

/// `GET /api/learning-requirements/rollup?organization=` — HR: per requirement, counts per status and the rate in date among those
/// counted, with the floor. `unknown` is counted apart; a group under the floor shows nothing; a rate with nothing to divide by is null.
#[debug_handler]
async fn rollup(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Query(q): Query<OrgQuery>,
) -> Result<Response> {
    ensure_hr(&ctx, &caller, &q.organization).await?;
    let today = Utc::now().date_naive();
    let in_org = workers::Entity::find()
        .filter(workers::Column::OrganizationRef.eq(&q.organization))
        .filter(workers::Column::DeletedAt.is_null())
        .filter(workers::Column::TerminatedOn.is_null());
    ensure_scale(&ctx, in_org.clone()).await?;
    let people = in_org.all(&ctx.db).await?;
    let pids: Vec<Uuid> = people.iter().map(|w| w.pid).collect();
    let reqs = live_requirements(&ctx, &q.organization, today).await?;
    let cats = categories_of(&ctx, &pids, today).await?;
    let comps = completions_of(&ctx, &people, &reqs).await?;
    let floor = crate::rules::equality_monitoring::DEFAULT_FLOOR;
    let none: Vec<rules::Categories> = Vec::new();
    let cells: Vec<Value> = reqs
        .iter()
        .map(|r| {
            let statuses: Vec<rules::Status> = people
                .iter()
                .map(|w| {
                    let done = comps.get(&(w.pid, r.pid)).cloned().unwrap_or_default();
                    evaluate(r, cats.get(&w.pid).unwrap_or(&none), &done, today).status
                })
                .collect();
            match rules::rollup(&statuses, floor) {
                rules::Rollup::Hidden => json!({
                    "requirement_pid": r.pid, "subject": r.subject, "basis": r.basis, "hidden": true,
                }),
                rules::Rollup::Shown {
                    compliant,
                    due_soon,
                    expired,
                    never_completed,
                    not_applicable,
                    unknown,
                    rate,
                } => json!({
                    "requirement_pid": r.pid, "subject": r.subject, "basis": r.basis, "hidden": false,
                    "compliant": compliant, "due_soon": due_soon, "expired": expired,
                    "never_completed": never_completed, "not_applicable": not_applicable,
                    "unknown": unknown, "rate": rate,
                }),
            }
        })
        .collect();
    format::json(
        json!({ "organization": q.organization, "as_of": today, "floor": floor, "requirements": cells }),
    )
}

/// The routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/learning-requirements", get(list))
        .add("/learning-requirements", post(create))
        .add("/learning-requirements/rollup", get(rollup))
        .add(
            "/learning-requirements/{pid}/completions",
            post(record_completion),
        )
        .add("/workers/{pid}/mandatory-learning", get(of_worker))
}
