//! HR core (WPM-R7–R9): worker CRUD + the status state machine +
//! the derived org chart, and benefits administration.
//!
//! Every mutation runs on one transaction: the row change, its audit
//! entry, and (under the `outbox` transport) its event share a commit
//! boundary (WPM-D9). Worker reads run the record-level ABAC pass
//! and honour the `mask` obligation (salary redaction, WPM-R15).

use axa::extract::{Json, Path, Query};
use axa::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::{Page, ensure_valid, record_rejection, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::db;
use crate::metrics::Metrics;
use crate::models::_entities::{benefit_enrollments, benefit_plans, workers};
use crate::models::{memberships, records};
use crate::rules::{engagement, lifecycle, org, talent, tokens, worker_fields};
use crate::streaming;
use crate::validation::Problems;

/// `POST /api/workers` body.
#[derive(Debug, Deserialize)]
struct WorkerPayload {
    person_ref: String,
    #[serde(default)]
    upstream_worker_ref: Option<String>,
    organization_ref: String,
    worker_number: String,
    display_name: String,
    employment_type: String,
    #[serde(default = "default_fte")]
    fte_percent: i32,
    department: String,
    job_title: String,
    #[serde(default)]
    location: Option<String>,
    #[serde(default)]
    manager_pid: Option<Uuid>,
    #[serde(default)]
    salary_minor: Option<i64>,
    #[serde(default)]
    salary_currency: Option<String>,
    hired_on: chrono::NaiveDate,
    /// The last day of the engagement: required for `fixed_term` and `contractor`, optional
    /// for `intern`, refused for `permanent` (WPM-R79).
    #[serde(default)]
    engagement_ends_on: Option<chrono::NaiveDate>,
}

/// `PUT /api/workers/{pid}` body — the mutable employment facts.
#[derive(Debug, Deserialize)]
struct WorkerUpdate {
    #[serde(default)]
    display_name: Option<String>,
    #[serde(default)]
    department: Option<String>,
    #[serde(default)]
    job_title: Option<String>,
    /// A blank string clears the location.
    #[serde(default)]
    location: Option<String>,
    /// Where the work is done, a jurisdiction code (WPM-D144); a blank string clears it. HR only.
    #[serde(default)]
    work_jurisdiction: Option<String>,
    #[serde(default)]
    fte_percent: Option<i32>,
    #[serde(default)]
    manager_pid: Option<Uuid>,
    #[serde(default)]
    clear_manager: bool,
    #[serde(default)]
    salary_minor: Option<i64>,
    #[serde(default)]
    salary_currency: Option<String>,
    #[serde(default)]
    upstream_worker_ref: Option<String>,
    /// Record a missing end date. A date already set moves only by an extension
    /// (`POST /api/workers/{pid}/engagement/extensions`), so the history is kept.
    #[serde(default)]
    engagement_ends_on: Option<chrono::NaiveDate>,
}

/// `POST /api/workers/{pid}/status` body.
#[derive(Debug, Deserialize)]
struct StatusPayload {
    to: String,
    #[serde(default)]
    reason: Option<String>,
}

/// `POST /api/benefit-plans` body.
#[derive(Debug, Deserialize)]
struct BenefitPlanPayload {
    name: String,
    kind: String,
    provider: String,
    worker_cost_minor: i64,
    employer_cost_minor: i64,
    currency: String,
}

/// `POST /api/workers/{pid}/benefit-enrollments` body.
#[derive(Debug, Deserialize)]
struct EnrollmentPayload {
    plan_pid: Uuid,
    starts_on: chrono::NaiveDate,
    #[serde(default)]
    ends_on: Option<chrono::NaiveDate>,
}

/// A `{pid}` reference response.
#[derive(Debug, Serialize)]
struct PidRef {
    pid: String,
}

const fn default_fte() -> i32 {
    100
}

fn validate_worker(p: &WorkerPayload) -> Vec<String> {
    let mut problems = Problems::new();
    problems.require_ref("person_ref", entity_ref::EntityType::Person, &p.person_ref);
    problems.ref_opt(
        "upstream_worker_ref",
        entity_ref::EntityType::Worker,
        p.upstream_worker_ref.as_deref(),
    );
    problems.require_ref(
        "organization_ref",
        entity_ref::EntityType::Organization,
        &p.organization_ref,
    );
    problems.require_text("worker_number", &p.worker_number);
    problems.require_text("display_name", &p.display_name);
    problems.require_token(
        "employment_type",
        tokens::EMPLOYMENT_TYPES,
        &p.employment_type,
    );
    problems.require_text("department", &p.department);
    problems.require_text("job_title", &p.job_title);
    if engagement::Basis::parse(&p.employment_type).is_some()
        && let Err(message) =
            engagement::validate_end_date(&p.employment_type, p.hired_on, p.engagement_ends_on)
    {
        problems.push(message);
    }
    if let Err(message) = org::normalize_location(p.location.as_deref()) {
        problems.push(message);
    }
    if !(1..=100).contains(&p.fte_percent) {
        problems.push(format!("fte_percent {} out of range 1-100", p.fte_percent));
    }
    if p.salary_minor.is_some_and(|s| s < 0) {
        problems.push("salary_minor must be non-negative".to_string());
    }
    if p.salary_minor.is_some() && p.salary_currency.as_deref().unwrap_or("").trim().is_empty() {
        problems.push("salary_currency is required with salary_minor".to_string());
    } else if let Some(code) = p
        .salary_currency
        .as_deref()
        .filter(|c| !c.trim().is_empty())
    {
        problems.require_currency("salary_currency", code);
    }
    problems.into_vec()
}

/// The live `manager_of` map for the cycle check (worker pid →
/// manager pid).
async fn manager_map(ctx: &AppContext) -> Result<HashMap<Uuid, Uuid>> {
    let rows = sqlx::query!(
        r#"SELECT pid, manager_pid AS "manager_pid!" FROM workers WHERE deleted_at IS NULL AND manager_pid IS NOT NULL"#
    )
    .fetch_all(db::pool(ctx))
    .await?;
    Ok(rows.into_iter().map(|e| (e.pid, e.manager_pid)).collect())
}

/// `POST /api/workers` — create in `onboarding` status.
#[debug_handler]
async fn create_worker(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<WorkerPayload>,
) -> Result<Response> {
    ensure_valid(&validate_worker(&payload))?;
    if let Some(manager) = payload.manager_pid {
        records::worker_by_pid(&ctx, manager).await?;
    }
    let mut tx = db::pool(&ctx).begin().await?;
    let row = sqlx::query!(
        "INSERT INTO workers (pid, person_ref, upstream_worker_ref, organization_ref, worker_number, display_name, status,
                              employment_type, fte_percent, department, job_title, location, manager_pid, salary_minor,
                              salary_currency, hired_on, engagement_ends_on)
         VALUES ($1, $2, $3, $4, $5, $6, 'onboarding', $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
         RETURNING pid, person_ref, organization_ref, worker_number, department, hired_on",
        Uuid::new_v4(),
        payload.person_ref,
        payload.upstream_worker_ref,
        payload.organization_ref,
        payload.worker_number,
        payload.display_name,
        payload.employment_type,
        payload.fte_percent,
        payload.department,
        payload.job_title,
        org::normalize_location(payload.location.as_deref()).unwrap_or(None),
        payload.manager_pid,
        payload.salary_minor,
        payload.salary_currency,
        payload.hired_on,
        payload.engagement_ends_on,
    )
    .fetch_one(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "worker",
        row.pid,
        "created",
        caller.actor(),
        Some(serde_json::json!({ "department": row.department })),
    )
    .await?;
    streaming::emit_in(
        &mut tx,
        "worker",
        "created",
        &row.pid.to_string(),
        &row.worker_number,
        caller.actor(),
        None,
    )
    .await?;
    // A worker made directly (not hired from an application) is still a person who became an employee: tell the other apps (WPM-R207).
    streaming::emit_in(
        &mut tx,
        "worker",
        "worker_hired",
        &row.pid.to_string(),
        &row.worker_number,
        caller.actor(),
        Some(crate::rules::bridge::handoff_data(
            &row.person_ref,
            &row.organization_ref,
            row.hired_on,
        )),
    )
    .await?;
    tx.commit().await?;
    format::json(PidRef {
        pid: row.pid.to_string(),
    })
}

/// Default page size for `GET /api/workers` — the cap this endpoint
/// applied before pagination existed (WPM-T40), so omitting `?limit=`
/// returns what it always did.
const WORKER_LIST_DEFAULT_LIMIT: u64 = 500;

/// `GET /api/workers` — active workers, filterable by
/// `?department=` and `?status=`, paged via `?limit=&cursor=`
/// (WPM-T40; `agents/share/restful.md`). Record-level masking applies
/// per row (a list must never reveal more than the single read).
#[derive(Debug, Deserialize)]
struct WorkerListParams {
    #[serde(default)]
    department: Option<String>,
    #[serde(default)]
    status: Option<String>,
    /// A comma-separated allowlist of fields to return (`rules::worker_fields::LIST_FIELDS`); absent ⇒ the whole record. An unknown
    /// name is a 422 (WPM-R216).
    #[serde(default)]
    fields: Option<String>,
}

#[debug_handler]
async fn list_workers(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Query(params): Query<WorkerListParams>,
    page: Page,
) -> Result<Response> {
    let window = super::resolve(&page, WORKER_LIST_DEFAULT_LIMIT)?;
    let fields = worker_fields::parse_fields(params.fields.as_deref())
        .map_err(|message| unprocessable(&message))?;
    let scope = memberships::scope_organization_refs(&ctx, caller.claims()).await?;
    let (limit, offset) = window.bounds();
    let rows = sqlx::query_as!(
        workers::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, person_ref,
                  upstream_worker_ref, organization_ref, worker_number, display_name, status, employment_type,
                  fte_percent, department, job_title, location, work_jurisdiction, manager_pid, salary_minor,
                  salary_currency, hired_on, terminated_on, engagement_ends_on, deleted_at AS "deleted_at: _"
             FROM workers
            WHERE deleted_at IS NULL AND ($1::text IS NULL OR department = $1) AND ($2::text IS NULL OR status = $2)
              AND ($3::text[] IS NULL OR organization_ref = ANY($3))
            ORDER BY id
            LIMIT $4 OFFSET $5"#,
        params.department,
        params.status,
        scope.as_deref(),
        limit,
        offset,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    // The row past the page only says that more follow. It is dropped before the salary audit, which records only
    // what is sent.
    let (rows, more) = window.split(rows);
    let discloses_salary = worker_fields::includes_salary(fields.as_deref());
    let mut out = Vec::with_capacity(rows.len());
    let mut salary_reads = Vec::new();
    for worker in rows {
        let obligations = auth::authorize_record(
            &caller,
            authentication_verifier::Action::Read,
            &auth::worker_resource_attrs(&worker),
        )
        .map_err(record_rejection)?;
        let masked = obligations.iter().any(|o| o == "mask");
        // A salary the answer carries is a salary read, audited as the single read audits it (WPM-D7, WPM-R216).
        if discloses_salary && worker.salary_minor.is_some() && !masked {
            salary_reads.push((worker.pid, worker.department.clone()));
        }
        let worker = if masked {
            auth::mask_worker(worker)
        } else {
            worker
        };
        out.push(match &fields {
            None => serde_json::to_value(worker)?,
            Some(names) => match serde_json::to_value(worker)? {
                serde_json::Value::Object(record) => {
                    serde_json::Value::Object(worker_fields::project(&record, names))
                }
                other => other,
            },
        });
    }
    // No audit, no answer (WPM-D107): every salary read is recorded, together, before the list is sent.
    if !salary_reads.is_empty() {
        let mut tx = db::pool(&ctx).begin().await?;
        for (pid, department) in salary_reads {
            db::audit(
                &mut tx,
                "worker",
                pid,
                "salary_read",
                caller.actor(),
                Some(serde_json::json!({ "department": department, "via": "list" })),
            )
            .await?;
        }
        tx.commit().await?;
    }
    Ok(super::with_next(
        axum::Json(out).into_response(),
        &window,
        more,
    ))
}

/// `GET /api/workers/{pid}` — one worker; a salary-bearing read
/// is audited (WPM-D7); the `mask` obligation redacts the salary.
#[debug_handler]
async fn get_worker(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    let obligations = auth::authorize_record(
        &caller,
        authentication_verifier::Action::Read,
        &auth::worker_resource_attrs(&worker),
    )
    .map_err(record_rejection)?;
    let masked = obligations.iter().any(|o| o == "mask");
    if worker.salary_minor.is_some() && !masked {
        db::audit(
            &mut *db::pool(&ctx).acquire().await?,
            "worker",
            worker.pid,
            "salary_read",
            caller.actor(),
            Some(serde_json::json!({ "department": worker.department })),
        )
        .await?;
    }
    format::json(if masked {
        auth::mask_worker(worker)
    } else {
        worker
    })
}

/// What is wrong with recording `end` on an existing worker, if anything: a date already set
/// moves only by an extension (so the history is kept), and a new one must fit the basis.
fn end_date_problem(worker: &workers::Model, end: Option<chrono::NaiveDate>) -> Option<String> {
    let end = end?;
    if worker.engagement_ends_on.is_some() {
        return Some(
            "the engagement already has an end date; move it with an extension so the history is kept"
                .to_string(),
        );
    }
    engagement::validate_end_date(&worker.employment_type, worker.hired_on, Some(end)).err()
}

/// `PUT /api/workers/{pid}` — update mutable employment facts.
/// A manager change runs the org-chart cycle check (WPM-R7).
#[debug_handler]
#[allow(clippy::too_many_lines)] // one optional field after another, validated then applied
async fn update_worker(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<WorkerUpdate>,
) -> Result<Response> {
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    auth::authorize_record(
        &caller,
        authentication_verifier::Action::Write,
        &auth::worker_resource_attrs(&worker),
    )
    .map_err(record_rejection)?;
    let mut problems = Problems::new();
    if let Some(name) = &payload.display_name {
        problems.require_text("display_name", name);
    }
    if let Some(dept) = &payload.department {
        problems.require_text("department", dept);
    }
    if let Some(fte) = payload.fte_percent
        && !(1..=100).contains(&fte)
    {
        problems.push(format!("fte_percent {fte} out of range 1-100"));
    }
    if let Err(message) = org::normalize_location(payload.location.as_deref()) {
        problems.push(message);
    }
    let work_jurisdiction = payload.work_jurisdiction.as_deref().map(|code| {
        Some(code.trim())
            .filter(|code| !code.is_empty())
            .map(str::to_string)
    });
    if let Some(Some(code)) = &work_jurisdiction
        && !crate::rules::jurisdiction::valid_code(code)
    {
        problems.push(
            "work_jurisdiction must be a jurisdiction code (2 or 3 capital letters or digits, optionally - and 1 to 3 more)"
                .to_string(),
        );
    }
    if payload.salary_minor.is_some_and(|s| s < 0) {
        problems.push("salary_minor must be non-negative".to_string());
    }
    problems.currency_opt("salary_currency", payload.salary_currency.as_deref());
    problems.ref_opt(
        "upstream_worker_ref",
        entity_ref::EntityType::Worker,
        payload.upstream_worker_ref.as_deref(),
    );
    if let Some(message) = end_date_problem(&worker, payload.engagement_ends_on) {
        problems.push(message);
    }
    ensure_valid(&problems.into_vec())?;
    // Where the work is done decides which rules apply to the worker, so it is HR's to set, not the person's (WPM-D144).
    if work_jurisdiction.is_some()
        && !super::policies::privileged(&ctx, &caller, &worker.organization_ref).await?
    {
        return Err(super::forbidden("where the work is done is set by HR"));
    }
    if let Some(manager) = payload.manager_pid {
        records::worker_by_pid(&ctx, manager).await?;
        let map = manager_map(&ctx).await?;
        if org::would_create_cycle(worker.pid, manager, &map) {
            return Err(unprocessable("manager assignment would create a cycle"));
        }
    }
    // A field left out keeps what is recorded. A location or a place of work that is given is set, blank clearing it;
    // the manager is cleared on request, else set when given.
    let set_location = payload.location.is_some();
    let location = payload
        .location
        .as_deref()
        .and_then(|v| org::normalize_location(Some(v)).unwrap_or(None));
    let set_jurisdiction = work_jurisdiction.is_some();
    let jurisdiction = work_jurisdiction.flatten();
    let set_manager = payload.clear_manager || payload.manager_pid.is_some();
    let manager = if payload.clear_manager {
        None
    } else {
        payload.manager_pid
    };
    let mut tx = db::pool(&ctx).begin().await?;
    let row = sqlx::query_as!(
        workers::Model,
        r#"UPDATE workers
              SET display_name = COALESCE($2, display_name),
                  department = COALESCE($3, department),
                  job_title = COALESCE($4, job_title),
                  location = CASE WHEN $5 THEN $6 ELSE location END,
                  work_jurisdiction = CASE WHEN $7 THEN $8 ELSE work_jurisdiction END,
                  fte_percent = COALESCE($9, fte_percent),
                  manager_pid = CASE WHEN $10 THEN $11 ELSE manager_pid END,
                  salary_minor = COALESCE($12, salary_minor),
                  salary_currency = COALESCE($13, salary_currency),
                  upstream_worker_ref = COALESCE($14, upstream_worker_ref),
                  engagement_ends_on = COALESCE($15, engagement_ends_on)
            WHERE id = $1
        RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, person_ref,
                  upstream_worker_ref, organization_ref, worker_number, display_name, status, employment_type,
                  fte_percent, department, job_title, location, work_jurisdiction, manager_pid, salary_minor,
                  salary_currency, hired_on, terminated_on, engagement_ends_on, deleted_at AS "deleted_at: _""#,
        worker.id,
        payload.display_name,
        payload.department,
        payload.job_title,
        set_location,
        location,
        set_jurisdiction,
        jurisdiction,
        payload.fte_percent,
        set_manager,
        manager,
        payload.salary_minor,
        payload.salary_currency,
        payload.upstream_worker_ref,
        payload.engagement_ends_on,
    )
    .fetch_one(&mut *tx)
    .await?;
    if row.work_jurisdiction != worker.work_jurisdiction {
        // A number written without + is read in the place of work's country where no address country is given.
        super::me::refresh_phone_forms(&mut tx, &row).await?;
    }
    db::audit(
        &mut tx,
        "worker",
        row.pid,
        "updated",
        caller.actor(),
        Some(serde_json::json!({ "department": row.department })),
    )
    .await?;
    streaming::emit_in(
        &mut tx,
        "worker",
        "updated",
        &row.pid.to_string(),
        &row.worker_number,
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(auth::mask_worker(row))
}

/// `POST /api/workers/{pid}/status` — one lifecycle transition.
/// `onboarding → active` requires every mandatory onboarding item
/// complete or waived (WPM-R3).
#[debug_handler]
#[allow(clippy::too_many_lines)] // the audit is written per kind, each with its literal action
async fn change_status(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<StatusPayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_token("to", tokens::WORKER_STATUSES, &payload.to);
    problems.cap_opt("reason", payload.reason.as_deref());
    ensure_valid(&problems.into_vec())?;
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    auth::authorize_record(
        &caller,
        authentication_verifier::Action::Write,
        &auth::worker_resource_attrs(&worker),
    )
    .map_err(record_rejection)?;
    lifecycle::check("worker", lifecycle::WORKER, &worker.status, &payload.to)
        .map_err(|e| unprocessable(&e))?;
    if worker.status == "onboarding" && payload.to == "active" {
        let open = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "open!" FROM onboarding_items
                WHERE worker_pid = $1 AND deleted_at IS NULL AND mandatory AND status = 'pending'"#,
            worker.pid,
        )
        .fetch_one(db::pool(&ctx))
        .await?;
        if open > 0 {
            return Err(unprocessable(&format!(
                "{open} mandatory onboarding item(s) still pending (complete or waive them first)"
            )));
        }
    }
    let from = worker.status.clone();
    let number = worker.worker_number.clone();
    let department = worker.department.clone();
    let ends = (payload.to == "terminated" || payload.to == "retired")
        .then(|| chrono::Utc::now().date_naive());
    let mut tx = db::pool(&ctx).begin().await?;
    let row = sqlx::query_as!(
        workers::Model,
        r#"UPDATE workers SET status = $2, terminated_on = COALESCE($3, terminated_on)
            WHERE id = $1
        RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, person_ref,
                  upstream_worker_ref, organization_ref, worker_number, display_name, status, employment_type,
                  fte_percent, department, job_title, location, work_jurisdiction, manager_pid, salary_minor,
                  salary_currency, hired_on, terminated_on, engagement_ends_on, deleted_at AS "deleted_at: _""#,
        worker.id,
        payload.to,
        ends,
    )
    .fetch_one(&mut *tx)
    .await?;
    let kind = match payload.to.as_str() {
        "active" if from == "onboarding" => "worker_activated",
        "terminated" => "worker_terminated",
        "retired" => "worker_retired",
        _ => "worker_status_changed",
    };
    let snapshot = Some(serde_json::json!({
        "from": from, "to": payload.to, "reason": payload.reason,
        "department": department,
    }));
    // One literal action per kind, so the audit guard can read each.
    match kind {
        "worker_activated" => {
            db::audit(
                &mut tx,
                "worker",
                row.pid,
                "worker_activated",
                caller.actor(),
                snapshot,
            )
            .await?;
        }
        "worker_terminated" => {
            db::audit(
                &mut tx,
                "worker",
                row.pid,
                "worker_terminated",
                caller.actor(),
                snapshot,
            )
            .await?;
        }
        "worker_retired" => {
            db::audit(
                &mut tx,
                "worker",
                row.pid,
                "worker_retired",
                caller.actor(),
                snapshot,
            )
            .await?;
        }
        _ => {
            db::audit(
                &mut tx,
                "worker",
                row.pid,
                "worker_status_changed",
                caller.actor(),
                snapshot,
            )
            .await?;
        }
    }
    streaming::emit_in(
        &mut tx,
        "worker",
        kind,
        &row.pid.to_string(),
        &number,
        caller.actor(),
        Some(status_event_data(&from, &payload.to, &row)),
    )
    .await?;
    tx.commit().await?;
    match kind {
        "worker_activated" => Metrics::global().worker_activated_total.inc(),
        "worker_terminated" => Metrics::global().worker_terminated_total.inc(),
        _ => {}
    }
    format::json(auth::mask_worker(row))
}

/// `DELETE /api/workers/{pid}` — soft delete.
#[debug_handler]
async fn delete_worker(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    auth::authorize_record(
        &caller,
        authentication_verifier::Action::Destructive,
        &auth::worker_resource_attrs(&worker),
    )
    .map_err(record_rejection)?;
    let pid = worker.pid;
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "UPDATE workers SET deleted_at = now() WHERE id = $1",
        worker.id
    )
    .execute(&mut *tx)
    .await?;
    db::audit(&mut tx, "worker", pid, "deleted", caller.actor(), None).await?;
    streaming::emit_in(
        &mut tx,
        "worker",
        "deleted",
        &pid.to_string(),
        &worker.worker_number,
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::empty_json()
}

/// One org-chart node.
#[derive(Debug, Serialize)]
struct OrgNode {
    pid: String,
    display_name: String,
    job_title: String,
    department: String,
    /// Whole-month tenure band at the time of the request
    /// ([`talent::tenure_bucket`]); `not_started` for a future hire date.
    tenure: &'static str,
    /// Free-text work location; `null` when not recorded.
    location: Option<String>,
    reports: Vec<OrgNode>,
}

/// What the org chart draws of a person.
struct Person {
    pid: Uuid,
    display_name: String,
    job_title: String,
    department: String,
    location: Option<String>,
    manager_pid: Option<Uuid>,
    hired_on: chrono::NaiveDate,
}

/// Build one org-chart node (bounded depth; the write path
/// prevents cycles).
fn build_org_node(
    node: &Person,
    children: &HashMap<Option<Uuid>, Vec<&Person>>,
    depth: usize,
    as_of: chrono::NaiveDate,
) -> OrgNode {
    let reports = if depth > 32 {
        Vec::new() // corrupt-cycle guard; the write path prevents cycles
    } else {
        children
            .get(&Some(node.pid))
            .map(|kids| {
                kids.iter()
                    .map(|k| build_org_node(k, children, depth + 1, as_of))
                    .collect()
            })
            .unwrap_or_default()
    };
    OrgNode {
        pid: node.pid.to_string(),
        display_name: node.display_name.clone(),
        job_title: node.job_title.clone(),
        department: node.department.clone(),
        tenure: talent::tenure_bucket(talent::months_of_service(node.hired_on, as_of)),
        location: node.location.clone(),
        reports,
    }
}

/// `GET /api/org-chart?organization=<ref>&limit=` — the manager forest for
/// one organization (roots = workers with no manager). At most `limit` people are drawn (default 5,000,
/// at most 20,000), the roots first and then each level in turn, so a person is drawn only when their
/// manager is; `GET /api/org-chart/size` says how many there are in all (WPM-R142).
#[derive(Debug, Deserialize)]
struct OrgChartParams {
    organization: String,
    limit: Option<usize>,
}

/// The default and the largest number of people one org-chart answer draws.
const ORG_CHART_DEFAULT_LIMIT: usize = 5_000;
const ORG_CHART_MAX_LIMIT: usize = 20_000;

#[debug_handler]
async fn org_chart(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Query(params): Query<OrgChartParams>,
) -> Result<Response> {
    let limit = params
        .limit
        .unwrap_or(ORG_CHART_DEFAULT_LIMIT)
        .clamp(1, ORG_CHART_MAX_LIMIT);
    // `organization` outside the caller's scope ANDs down to zero rows here — no separate 403 needed for this read.
    let scope = memberships::scope_organization_refs(&ctx, caller.claims()).await?;
    // Breadth first: the roots, then the reports of each level, until `limit` people are held.
    let mut rows: Vec<Person> = sqlx::query_as!(
        Person,
        "SELECT pid, display_name, job_title, department, location, manager_pid, hired_on FROM workers
          WHERE deleted_at IS NULL AND organization_ref = $1 AND ($2::text[] IS NULL OR organization_ref = ANY($2))
            AND manager_pid IS NULL
          ORDER BY id LIMIT $3",
        params.organization,
        scope.as_deref(),
        i64::try_from(limit).unwrap_or(i64::MAX),
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let mut frontier: Vec<Uuid> = rows.iter().map(|r| r.pid).collect();
    for _ in 0..33 {
        if frontier.is_empty() || rows.len() >= limit {
            break;
        }
        let mut next = sqlx::query_as!(
            Person,
            "SELECT pid, display_name, job_title, department, location, manager_pid, hired_on FROM workers
              WHERE deleted_at IS NULL AND organization_ref = $1 AND ($2::text[] IS NULL OR organization_ref = ANY($2))
                AND manager_pid = ANY($3)
              ORDER BY id",
            params.organization,
            scope.as_deref(),
            &frontier,
        )
        .fetch_all(db::pool(&ctx))
        .await?;
        next.truncate(limit - rows.len());
        frontier = next.iter().map(|r| r.pid).collect();
        rows.extend(next);
    }
    let mut children: HashMap<Option<Uuid>, Vec<&Person>> = HashMap::new();
    for e in &rows {
        children.entry(e.manager_pid).or_default().push(e);
    }
    let as_of = chrono::Utc::now().date_naive();
    let roots: Vec<OrgNode> = children
        .get(&None)
        .map(|top| {
            top.iter()
                .map(|n| build_org_node(n, &children, 0, as_of))
                .collect()
        })
        .unwrap_or_default();
    format::json(roots)
}

/// `GET /api/org-chart/size?organization=<ref>` — how many people the organization has in all, so a
/// client can say when the chart it was given is only part of it.
#[debug_handler]
async fn org_chart_size(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Query(params): Query<OrgChartParams>,
) -> Result<Response> {
    let scope = memberships::scope_organization_refs(&ctx, caller.claims()).await?;
    let people = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "people!" FROM workers
            WHERE deleted_at IS NULL AND organization_ref = $1 AND ($2::text[] IS NULL OR organization_ref = ANY($2))"#,
        params.organization,
        scope.as_deref(),
    )
    .fetch_one(db::pool(&ctx))
    .await?;
    format::json(serde_json::json!({ "people": people }))
}

/// `POST /api/benefit-plans`.
#[debug_handler]
async fn create_plan(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<BenefitPlanPayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_text("name", &payload.name);
    problems.require_token("kind", tokens::BENEFIT_KINDS, &payload.kind);
    problems.require_text("provider", &payload.provider);
    problems.require_currency("currency", &payload.currency);
    if payload.worker_cost_minor < 0 || payload.employer_cost_minor < 0 {
        problems.push("benefit costs must be non-negative".to_string());
    }
    ensure_valid(&problems.into_vec())?;
    let pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO benefit_plans (pid, name, kind, provider, worker_cost_minor, employer_cost_minor, currency)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
        pid,
        payload.name,
        payload.kind,
        payload.provider,
        payload.worker_cost_minor,
        payload.employer_cost_minor,
        payload.currency,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "benefit_plan",
        pid,
        "created",
        caller.actor(),
        None,
    )
    .await?;
    streaming::emit_in(
        &mut tx,
        "benefit_plan",
        "created",
        &pid.to_string(),
        &payload.name,
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(PidRef {
        pid: pid.to_string(),
    })
}

/// Default page size for `GET /api/benefit-plans` — the cap this
/// endpoint applied before pagination existed (WPM-T40).
const BENEFIT_PLAN_LIST_DEFAULT_LIMIT: u64 = 200;

/// `GET /api/benefit-plans`, paged via `?limit=&cursor=` (WPM-T40;
/// `agents/share/restful.md`).
#[debug_handler]
async fn list_plans(State(ctx): State<AppContext>, page: Page) -> Result<Response> {
    let window = super::resolve(&page, BENEFIT_PLAN_LIST_DEFAULT_LIMIT)?;
    let (limit, offset) = window.bounds();
    let rows = sqlx::query_as!(
        benefit_plans::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, name, kind, provider,
                  worker_cost_minor, employer_cost_minor, currency, deleted_at AS "deleted_at: _"
             FROM benefit_plans WHERE deleted_at IS NULL ORDER BY id LIMIT $1 OFFSET $2"#,
        limit,
        offset,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    Ok(window.respond(rows))
}

/// `POST /api/workers/{pid}/benefit-enrollments` — enrol; the
/// partial unique index refuses a double enrolment (WPM-R9).
#[debug_handler]
async fn enroll(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<EnrollmentPayload>,
) -> Result<Response> {
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    let plan = sqlx::query!(
        "SELECT pid, name FROM benefit_plans WHERE pid = $1 AND deleted_at IS NULL",
        payload.plan_pid,
    )
    .fetch_optional(db::pool(&ctx))
    .await?
    .ok_or(Error::NotFound)?;
    if payload.ends_on.is_some_and(|end| end < payload.starts_on) {
        return Err(unprocessable("ends_on is before starts_on"));
    }
    let existing = sqlx::query_scalar!(
        r#"SELECT 1 AS "one!" FROM benefit_enrollments
            WHERE plan_pid = $1 AND worker_pid = $2 AND deleted_at IS NULL LIMIT 1"#,
        plan.pid,
        worker.pid,
    )
    .fetch_optional(db::pool(&ctx))
    .await?;
    if existing.is_some() {
        return Err(unprocessable("worker is already enrolled in this plan"));
    }
    let new_pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO benefit_enrollments (pid, plan_pid, worker_pid, starts_on, ends_on) VALUES ($1, $2, $3, $4, $5)",
        new_pid,
        plan.pid,
        worker.pid,
        payload.starts_on,
        payload.ends_on,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "benefit_enrollment",
        new_pid,
        "created",
        caller.actor(),
        None,
    )
    .await?;
    streaming::emit_in(
        &mut tx,
        "benefit_enrollment",
        "benefit_enrolled",
        &new_pid.to_string(),
        &plan.name,
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(PidRef {
        pid: new_pid.to_string(),
    })
}

/// `GET /api/workers/{pid}/benefit-enrollments`.
#[debug_handler]
async fn list_enrollments(
    State(ctx): State<AppContext>,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    let rows = sqlx::query_as!(
        benefit_enrollments::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, plan_pid, worker_pid, starts_on,
                  ends_on, deleted_at AS "deleted_at: _"
             FROM benefit_enrollments WHERE worker_pid = $1 AND deleted_at IS NULL ORDER BY id"#,
        worker.pid,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    format::json(rows)
}

/// `DELETE /api/benefit-enrollments/{pid}` — soft delete (unenrol).
#[debug_handler]
async fn unenroll(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let pid = records::parse_pid(&pid)?;
    let id = sqlx::query_scalar!(
        "SELECT id FROM benefit_enrollments WHERE pid = $1 AND deleted_at IS NULL",
        pid,
    )
    .fetch_optional(db::pool(&ctx))
    .await?
    .ok_or(Error::NotFound)?;
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "UPDATE benefit_enrollments SET deleted_at = now() WHERE id = $1",
        id
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "benefit_enrollment",
        pid,
        "deleted",
        caller.actor(),
        None,
    )
    .await?;
    streaming::emit_in(
        &mut tx,
        "benefit_enrollment",
        "deleted",
        &pid.to_string(),
        "",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::empty_json()
}

/// What a status-change event carries: the change, and, when employment ends, who, where and when for the other apps (WPM-R207).
fn status_event_data(from: &str, to: &str, row: &workers::Model) -> serde_json::Value {
    let mut data = serde_json::json!({ "from": from, "to": to });
    if matches!(to, "terminated" | "retired") {
        let handoff = crate::rules::bridge::handoff_data(
            &row.person_ref,
            &row.organization_ref,
            row.terminated_on
                .unwrap_or_else(|| chrono::Utc::now().date_naive()),
        );
        if let (Some(d), Some(h)) = (data.as_object_mut(), handoff.as_object()) {
            d.extend(h.clone());
        }
    }
    data
}

/// The HR-core routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/workers", post(create_worker))
        .add("/workers", get(list_workers))
        .add("/workers/{pid}", get(get_worker))
        .add("/workers/{pid}", put(update_worker))
        .add("/workers/{pid}", delete(delete_worker))
        .add("/workers/{pid}/status", post(change_status))
        .add("/org-chart", get(org_chart))
        .add("/org-chart/size", get(org_chart_size))
        .add("/benefit-plans", post(create_plan))
        .add("/benefit-plans", get(list_plans))
        .add("/workers/{pid}/benefit-enrollments", post(enroll))
        .add("/workers/{pid}/benefit-enrollments", get(list_enrollments))
        .add("/benefit-enrollments/{pid}", delete(unenroll))
}
