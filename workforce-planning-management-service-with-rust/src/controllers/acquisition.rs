//! Talent acquisition (WPM-R1–R3): requisitions, candidates,
//! applications + interviews, and onboarding checklists. Hiring an
//! application creates the Worker in one transaction (WPM-D9).

use super::row_json;
use axa::extract::{Json, Path, Query};
use axa::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{ensure_valid, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::db;
use crate::metrics::Metrics;
use crate::models::_entities::{
    applications, candidates, interviews, onboarding_items, requisitions,
};
use crate::models::{memberships, records};
use crate::rules::{engagement, lifecycle, planning, tokens};
use crate::streaming;
use crate::validation::Problems;

/// The live requisition with this pid, else `404`.
async fn find_requisition(ctx: &AppContext, pid: Uuid) -> Result<requisitions::Model> {
    sqlx::query_as!(
        requisitions::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, organization_ref, department,
                  job_title, headcount, salary_min_minor, salary_max_minor, salary_currency, status, opened_on, filled_on,
                  deleted_at AS "deleted_at: _", programme_ref, basis, expected_duration_months
             FROM requisitions WHERE pid = $1 AND deleted_at IS NULL"#,
        pid,
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)
}

/// The live candidate with this pid, else `404`.
async fn find_candidate(ctx: &AppContext, pid: Uuid) -> Result<candidates::Model> {
    sqlx::query_as!(
        candidates::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, person_ref, display_name, email,
                  source, consent_until, deleted_at AS "deleted_at: _"
             FROM candidates WHERE pid = $1 AND deleted_at IS NULL"#,
        pid,
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)
}

/// The live application with this pid, else `404`.
async fn find_application(ctx: &AppContext, pid: Uuid) -> Result<applications::Model> {
    sqlx::query_as!(
        applications::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, requisition_pid, candidate_pid,
                  stage, notes, deleted_at AS "deleted_at: _"
             FROM applications WHERE pid = $1 AND deleted_at IS NULL"#,
        pid,
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)
}

/// `POST /api/requisitions` body.
#[derive(Debug, Deserialize)]
struct RequisitionPayload {
    organization_ref: String,
    department: String,
    job_title: String,
    #[serde(default = "default_headcount")]
    headcount: i32,
    #[serde(default)]
    salary_min_minor: Option<i64>,
    #[serde(default)]
    salary_max_minor: Option<i64>,
    #[serde(default)]
    salary_currency: Option<String>,
    /// The programme the post serves (`thing:` URN).
    #[serde(default)]
    programme_ref: Option<String>,
    /// `permanent` (default), `fixed_term`, `contractor` or `intern` (WPM-R82).
    #[serde(default = "default_basis")]
    basis: String,
    /// How long a basis that ends is expected to run, in calendar months.
    #[serde(default)]
    expected_duration_months: Option<i32>,
}

fn default_basis() -> String {
    "permanent".to_string()
}

/// `POST /api/candidates` body.
#[derive(Debug, Deserialize)]
struct CandidatePayload {
    display_name: String,
    email: String,
    source: String,
    #[serde(default)]
    person_ref: Option<String>,
    #[serde(default)]
    consent_until: Option<chrono::NaiveDate>,
}

/// `POST /api/requisitions/{pid}/applications` body.
#[derive(Debug, Deserialize)]
struct ApplicationPayload {
    candidate_pid: Uuid,
    #[serde(default)]
    notes: Option<String>,
}

/// `POST /api/applications/{pid}/stage` body. Moving to `hired`
/// requires the worker fields.
#[derive(Debug, Deserialize)]
struct StagePayload {
    to: String,
    /// Required on `hired`: the person URN for the new worker.
    #[serde(default)]
    person_ref: Option<String>,
    /// Required on `hired`: the worker number.
    #[serde(default)]
    worker_number: Option<String>,
    #[serde(default)]
    employment_type: Option<String>,
    #[serde(default)]
    fte_percent: Option<i32>,
    #[serde(default)]
    salary_minor: Option<i64>,
    #[serde(default)]
    salary_currency: Option<String>,
    #[serde(default)]
    hired_on: Option<chrono::NaiveDate>,
    /// The last day of the engagement (WPM-R79): required for `fixed_term` and
    /// `contractor`, refused for `permanent`.
    #[serde(default)]
    engagement_ends_on: Option<chrono::NaiveDate>,
}

/// `POST /api/applications/{pid}/interviews` body.
#[derive(Debug, Deserialize)]
struct InterviewPayload {
    scheduled_at: chrono::DateTime<chrono::FixedOffset>,
    interviewer_ref: String,
    #[serde(default)]
    notes: Option<String>,
}

/// `PUT /api/interviews/{pid}` body (outcome).
#[derive(Debug, Deserialize)]
struct OutcomePayload {
    outcome: String,
    #[serde(default)]
    notes: Option<String>,
}

/// `POST /api/workers/{pid}/onboarding` body — add checklist items.
#[derive(Debug, Deserialize)]
struct OnboardingPayload {
    items: Vec<OnboardingItemPayload>,
}

/// One checklist item.
#[derive(Debug, Deserialize)]
struct OnboardingItemPayload {
    name: String,
    #[serde(default = "default_true")]
    mandatory: bool,
}

/// `POST /api/onboarding-items/{pid}/waive` body.
#[derive(Debug, Deserialize)]
struct WaivePayload {
    reason: String,
}

/// A `{pid}` reference response.
#[derive(Debug, Serialize)]
struct PidRef {
    pid: String,
}

const fn default_headcount() -> i32 {
    1
}
const fn default_true() -> bool {
    true
}

/// `POST /api/requisitions` — create in `draft`.
#[debug_handler]
async fn create_requisition(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<RequisitionPayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_ref(
        "organization_ref",
        entity_ref::EntityType::Organization,
        &payload.organization_ref,
    );
    problems.require_text("department", &payload.department);
    problems.require_text("job_title", &payload.job_title);
    if payload.headcount < 1 {
        problems.push("headcount must be at least 1".to_string());
    }
    if let (Some(min), Some(max)) = (payload.salary_min_minor, payload.salary_max_minor)
        && min > max
    {
        problems.push("salary_min_minor exceeds salary_max_minor".to_string());
    }
    problems.currency_opt("salary_currency", payload.salary_currency.as_deref());
    problems.ref_opt(
        "programme_ref",
        entity_ref::EntityType::Thing,
        payload.programme_ref.as_deref(),
    );
    if let Err(message) =
        planning::validate_requisition_basis(&payload.basis, payload.expected_duration_months)
    {
        problems.push(message);
    }
    ensure_valid(&problems.into_vec())?;
    let pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO requisitions (pid, organization_ref, department, job_title, headcount, salary_min_minor, salary_max_minor,
                                   salary_currency, status, programme_ref, basis, expected_duration_months)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'draft', $9, $10, $11)",
        pid,
        payload.organization_ref,
        payload.department,
        payload.job_title,
        payload.headcount,
        payload.salary_min_minor,
        payload.salary_max_minor,
        payload.salary_currency,
        payload.programme_ref,
        payload.basis,
        payload.expected_duration_months,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(&mut tx, "requisition", pid, "created", caller.actor(), None).await?;
    streaming::emit_in(
        &mut tx,
        "requisition",
        "created",
        &pid.to_string(),
        &payload.job_title,
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(PidRef {
        pid: pid.to_string(),
    })
}

/// `GET /api/requisitions` — active, filterable by `?status=`.
#[derive(Debug, Deserialize)]
struct RequisitionListParams {
    #[serde(default)]
    status: Option<String>,
}

#[debug_handler]
async fn list_requisitions(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Query(params): Query<RequisitionListParams>,
    page: super::Page,
) -> Result<Response> {
    let window = super::window(&page)?;
    let refs = memberships::scope_organization_refs(&ctx, caller.claims()).await?;
    let (limit, offset) = window.bounds();
    let rows = sqlx::query_as!(
        requisitions::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, organization_ref, department,
                  job_title, headcount, salary_min_minor, salary_max_minor, salary_currency, status, opened_on, filled_on,
                  deleted_at AS "deleted_at: _", programme_ref, basis, expected_duration_months
             FROM requisitions
            WHERE deleted_at IS NULL AND ($1::text IS NULL OR status = $1)
              AND ($2::text[] IS NULL OR organization_ref = ANY($2))
            ORDER BY id
            LIMIT $3 OFFSET $4"#,
        params.status,
        refs.as_deref(),
        limit,
        offset,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let shown = rows.iter().map(row_json).collect::<Result<Vec<_>>>()?;
    Ok(window.respond(shown))
}

/// `GET /api/requisitions/{pid}`.
#[debug_handler]
async fn get_requisition(
    State(ctx): State<AppContext>,
    Path(pid): Path<String>,
) -> Result<Response> {
    format::json(find_requisition(&ctx, records::parse_pid(&pid)?).await?)
}

/// `POST /api/requisitions/{pid}/status` — one pipeline transition.
/// `filled` requires hired applications ≥ headcount (WPM-R1).
#[derive(Debug, Deserialize)]
struct RequisitionStatusPayload {
    to: String,
}

#[allow(clippy::too_many_lines)] // the audit is written out once per kind of move
#[debug_handler]
async fn requisition_status(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<RequisitionStatusPayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_token("to", tokens::REQUISITION_STATUSES, &payload.to);
    ensure_valid(&problems.into_vec())?;
    let requisition = find_requisition(&ctx, records::parse_pid(&pid)?).await?;
    lifecycle::check(
        "requisition",
        lifecycle::REQUISITION,
        &requisition.status,
        &payload.to,
    )
    .map_err(|e| unprocessable(&e))?;
    // Applicants are owed the pay range before the interview (Article 5 of the Pay Transparency Directive), so a
    // vacancy states it before it is opened.
    // Over the organization's home jurisdiction's chain (WPM-D153), else the setting, else on.
    let pay_range_required = if payload.to == "open" {
        let scope = super::jurisdictions::Scope::for_organization(
            crate::db::pool(&ctx),
            &requisition.organization_ref,
        )
        .await?;
        super::pay_information::pay_range_required(Some(&scope), chrono::Utc::now().date_naive())
            .value
    } else {
        false
    };
    if pay_range_required {
        crate::rules::pay_information::require_range(
            requisition.salary_min_minor,
            requisition.salary_max_minor,
        )
        .map_err(|e| unprocessable(&e))?;
    }
    if payload.to == "filled" {
        let hired = sqlx::query_scalar!(
            r#"SELECT count(*) AS "n!" FROM applications
                WHERE requisition_pid = $1 AND deleted_at IS NULL AND stage = 'hired'"#,
            requisition.pid,
        )
        .fetch_one(db::pool(&ctx))
        .await?;
        if hired < i64::from(requisition.headcount) {
            return Err(unprocessable(&format!(
                "cannot fill: {hired} hired of headcount {}",
                requisition.headcount
            )));
        }
    }
    let from = requisition.status.clone();
    let title = requisition.job_title.clone();
    let today = chrono::Utc::now().date_naive();
    let mut tx = db::pool(&ctx).begin().await?;
    let row = sqlx::query_as!(
        requisitions::Model,
        r#"UPDATE requisitions
              SET status = $2,
                  opened_on = CASE WHEN $4 THEN $3::date ELSE opened_on END,
                  filled_on = CASE WHEN $5 THEN $3::date ELSE filled_on END
            WHERE id = $1
        RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, organization_ref, department,
                  job_title, headcount, salary_min_minor, salary_max_minor, salary_currency, status, opened_on, filled_on,
                  deleted_at AS "deleted_at: _", programme_ref, basis, expected_duration_months"#,
        requisition.id,
        payload.to,
        today,
        payload.to == "open",
        payload.to == "filled",
    )
    .fetch_one(&mut *tx)
    .await?;
    let snapshot = Some(serde_json::json!({ "from": from, "to": payload.to }));
    // One literal action per kind of move.
    let kind = match payload.to.as_str() {
        "open" => {
            db::audit(
                &mut tx,
                "requisition",
                row.pid,
                "requisition_opened",
                caller.actor(),
                snapshot,
            )
            .await?;
            "requisition_opened"
        }
        "filled" => {
            db::audit(
                &mut tx,
                "requisition",
                row.pid,
                "requisition_filled",
                caller.actor(),
                snapshot,
            )
            .await?;
            "requisition_filled"
        }
        _ => {
            db::audit(
                &mut tx,
                "requisition",
                row.pid,
                "requisition_status_changed",
                caller.actor(),
                snapshot,
            )
            .await?;
            "requisition_status_changed"
        }
    };
    streaming::emit_in(
        &mut tx,
        "requisition",
        kind,
        &row.pid.to_string(),
        &title,
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(row_json(&row)?)
}

/// `POST /api/candidates`.
#[debug_handler]
async fn create_candidate(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<CandidatePayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_text("display_name", &payload.display_name);
    problems.require_text("email", &payload.email);
    if !payload.email.contains('@') {
        problems.push("email must contain @".to_string());
    }
    problems.require_token("source", tokens::CANDIDATE_SOURCES, &payload.source);
    problems.ref_opt(
        "person_ref",
        entity_ref::EntityType::Person,
        payload.person_ref.as_deref(),
    );
    ensure_valid(&problems.into_vec())?;
    let pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO candidates (pid, person_ref, display_name, email, source, consent_until) VALUES ($1, $2, $3, $4, $5, $6)",
        pid,
        payload.person_ref,
        payload.display_name,
        payload.email,
        payload.source,
        payload.consent_until,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(&mut tx, "candidate", pid, "created", caller.actor(), None).await?;
    streaming::emit_in(
        &mut tx,
        "candidate",
        "created",
        &pid.to_string(),
        &payload.display_name,
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(PidRef {
        pid: pid.to_string(),
    })
}

/// `GET /api/candidates` — the pool. Consent-expired candidates are
/// **excluded** (WPM-D8); `?expired=1` lists only the purge queue.
#[derive(Debug, Deserialize)]
struct CandidateListParams {
    #[serde(default)]
    expired: Option<String>,
}

#[debug_handler]
async fn list_candidates(
    State(ctx): State<AppContext>,
    Query(params): Query<CandidateListParams>,
) -> Result<Response> {
    let today = chrono::Utc::now().date_naive();
    let rows = sqlx::query_as!(
        candidates::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, person_ref, display_name, email,
                  source, consent_until, deleted_at AS "deleted_at: _"
             FROM candidates WHERE deleted_at IS NULL ORDER BY id LIMIT 1000"#,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let expired_only = params
        .expired
        .as_deref()
        .is_some_and(|v| v == "1" || v == "true");
    let rows: Vec<_> = rows
        .into_iter()
        .filter(|c| {
            let expired = c.consent_until.is_some_and(|until| until < today);
            if expired_only { expired } else { !expired }
        })
        .collect();
    format::json(rows)
}

/// `POST /api/requisitions/{pid}/applications` — apply a candidate.
#[debug_handler]
async fn create_application(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<ApplicationPayload>,
) -> Result<Response> {
    let requisition = find_requisition(&ctx, records::parse_pid(&pid)?).await?;
    let candidate = find_candidate(&ctx, payload.candidate_pid).await?;
    if requisition.status == "draft"
        || requisition.status == "cancelled"
        || requisition.status == "filled"
    {
        return Err(unprocessable(&format!(
            "requisition is {} and not accepting applications",
            requisition.status
        )));
    }
    let today = chrono::Utc::now().date_naive();
    if candidate.consent_until.is_some_and(|until| until < today) {
        return Err(unprocessable("candidate consent has expired"));
    }
    let mut problems = Problems::new();
    problems.cap_opt("notes", payload.notes.as_deref());
    ensure_valid(&problems.into_vec())?;
    let app_pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO applications (pid, requisition_pid, candidate_pid, stage, notes) VALUES ($1, $2, $3, 'received', $4)",
        app_pid,
        requisition.pid,
        candidate.pid,
        payload.notes,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "application",
        app_pid,
        "created",
        caller.actor(),
        None,
    )
    .await?;
    streaming::emit_in(
        &mut tx,
        "application",
        "created",
        &app_pid.to_string(),
        &candidate.display_name,
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(PidRef {
        pid: app_pid.to_string(),
    })
}

/// `GET /api/requisitions/{pid}/applications`.
#[debug_handler]
async fn list_applications(
    State(ctx): State<AppContext>,
    Path(pid): Path<String>,
) -> Result<Response> {
    let requisition = find_requisition(&ctx, records::parse_pid(&pid)?).await?;
    let rows = sqlx::query_as!(
        applications::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, requisition_pid, candidate_pid,
                  stage, notes, deleted_at AS "deleted_at: _"
             FROM applications WHERE requisition_pid = $1 AND deleted_at IS NULL ORDER BY id"#,
        requisition.pid,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    format::json(rows)
}

/// `POST /api/applications/{pid}/stage` — one stage transition.
/// `hired` creates the Worker (onboarding status) **in the same
/// transaction** (WPM-R2, WPM-D9) and emits `worker_hired`.
#[allow(clippy::too_many_lines)] // one linear stage walk incl. the in-tx hire
#[debug_handler]
async fn application_stage(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<StagePayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_token("to", tokens::APPLICATION_STAGES, &payload.to);
    ensure_valid(&problems.into_vec())?;
    let application = find_application(&ctx, records::parse_pid(&pid)?).await?;
    lifecycle::check(
        "application",
        lifecycle::APPLICATION,
        &application.stage,
        &payload.to,
    )
    .map_err(|e| unprocessable(&e))?;
    let requisition = find_requisition(&ctx, application.requisition_pid).await?;
    let candidate = find_candidate(&ctx, application.candidate_pid).await?;

    let mut tx = db::pool(&ctx).begin().await?;
    let from = application.stage.clone();
    let stage = sqlx::query_scalar!(
        "UPDATE applications SET stage = $2 WHERE id = $1 RETURNING stage",
        application.id,
        payload.to,
    )
    .fetch_one(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "application",
        application.pid,
        "application_staged",
        caller.actor(),
        Some(serde_json::json!({ "from": from, "to": payload.to })),
    )
    .await?;
    streaming::emit_in(
        &mut tx,
        "application",
        "application_staged",
        &application.pid.to_string(),
        &candidate.display_name,
        caller.actor(),
        Some(serde_json::json!({ "from": from, "to": payload.to })),
    )
    .await?;

    let mut worker_pid = None;
    if payload.to == "hired" {
        let person_ref = payload
            .person_ref
            .clone()
            .or_else(|| candidate.person_ref.clone())
            .ok_or_else(|| {
                unprocessable("hiring requires person_ref (on the payload or the candidate)")
            })?;
        let worker_number = payload
            .worker_number
            .clone()
            .ok_or_else(|| unprocessable("hiring requires worker_number"))?;
        let mut problems = Problems::new();
        problems.require_ref("person_ref", entity_ref::EntityType::Person, &person_ref);
        // The hire carries the requisition's basis, and the end it implies, unless told otherwise.
        let employment_type = payload
            .employment_type
            .clone()
            .unwrap_or_else(|| requisition.basis.clone());
        problems.require_token(
            "employment_type",
            tokens::EMPLOYMENT_TYPES,
            &employment_type,
        );
        let hired_on = payload
            .hired_on
            .unwrap_or_else(|| chrono::Utc::now().date_naive());
        let engagement_ends_on = payload.engagement_ends_on.or_else(|| {
            (employment_type == requisition.basis)
                .then_some(requisition.expected_duration_months)
                .flatten()
                .and_then(|months| planning::end_from_duration(hired_on, months))
        });
        if engagement::Basis::parse(&employment_type).is_some()
            && let Err(message) =
                engagement::validate_end_date(&employment_type, hired_on, engagement_ends_on)
        {
            problems.push(message);
        }
        problems.currency_opt("salary_currency", payload.salary_currency.as_deref());
        ensure_valid(&problems.into_vec())?;
        let hire = Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO workers (pid, person_ref, organization_ref, worker_number, display_name, status, employment_type,
                                  fte_percent, department, job_title, salary_minor, salary_currency, hired_on, engagement_ends_on)
             VALUES ($1, $2, $3, $4, $5, 'onboarding', $6, $7, $8, $9, $10, $11, $12, $13)",
            hire,
            person_ref,
            requisition.organization_ref,
            worker_number,
            candidate.display_name,
            employment_type,
            payload.fte_percent.unwrap_or(100),
            requisition.department,
            requisition.job_title,
            payload.salary_minor,
            payload.salary_currency,
            hired_on,
            engagement_ends_on,
        )
        .execute(&mut *tx)
        .await?;
        db::audit(
            &mut tx,
            "worker",
            hire,
            "worker_hired",
            caller.actor(),
            Some(serde_json::json!({
                "application_pid": application.pid, "department": requisition.department,
            })),
        )
        .await?;
        streaming::emit_in(
            &mut tx,
            "worker",
            "worker_hired",
            &hire.to_string(),
            &worker_number,
            caller.actor(),
            Some(crate::rules::bridge::handoff_data(
                &person_ref,
                &requisition.organization_ref,
                hired_on,
            )),
        )
        .await?;
        worker_pid = Some(hire);
    }
    tx.commit().await?;
    if worker_pid.is_some() {
        Metrics::global().worker_hired_total.inc();
    }
    format::json(serde_json::json!({
        "pid": application.pid, "stage": stage, "worker_pid": worker_pid,
    }))
}

/// `POST /api/applications/{pid}/interviews`.
#[debug_handler]
async fn create_interview(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<InterviewPayload>,
) -> Result<Response> {
    let application = find_application(&ctx, records::parse_pid(&pid)?).await?;
    let mut problems = Problems::new();
    problems.require_ref(
        "interviewer_ref",
        entity_ref::EntityType::Worker,
        &payload.interviewer_ref,
    );
    problems.cap_opt("notes", payload.notes.as_deref());
    ensure_valid(&problems.into_vec())?;
    let interview = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO interviews (pid, application_pid, scheduled_at, interviewer_ref, outcome, notes)
         VALUES ($1, $2, $3, $4, 'pending', $5)",
        interview,
        application.pid,
        payload.scheduled_at,
        payload.interviewer_ref,
        payload.notes,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "interview",
        interview,
        "created",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(PidRef {
        pid: interview.to_string(),
    })
}

/// `GET /api/applications/{pid}/interviews`.
#[debug_handler]
async fn list_interviews(
    State(ctx): State<AppContext>,
    Path(pid): Path<String>,
) -> Result<Response> {
    let application = find_application(&ctx, records::parse_pid(&pid)?).await?;
    let rows = sqlx::query_as!(
        interviews::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, application_pid,
                  scheduled_at AS "scheduled_at: _", interviewer_ref, outcome, notes, deleted_at AS "deleted_at: _"
             FROM interviews WHERE application_pid = $1 AND deleted_at IS NULL ORDER BY id"#,
        application.pid,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    format::json(rows)
}

/// `PUT /api/interviews/{pid}` — record the outcome.
#[debug_handler]
async fn interview_outcome(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<OutcomePayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_token("outcome", tokens::INTERVIEW_OUTCOMES, &payload.outcome);
    problems.cap_opt("notes", payload.notes.as_deref());
    ensure_valid(&problems.into_vec())?;
    let pid = records::parse_pid(&pid)?;
    let id = sqlx::query_scalar!(
        "SELECT id FROM interviews WHERE pid = $1 AND deleted_at IS NULL",
        pid
    )
    .fetch_optional(db::pool(&ctx))
    .await?
    .ok_or(Error::NotFound)?;
    let mut tx = db::pool(&ctx).begin().await?;
    // Notes left out stay as recorded.
    let row = sqlx::query_as!(
        interviews::Model,
        r#"UPDATE interviews SET outcome = $2, notes = COALESCE($3, notes) WHERE id = $1
           RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, application_pid,
                     scheduled_at AS "scheduled_at: _", interviewer_ref, outcome, notes, deleted_at AS "deleted_at: _""#,
        id,
        payload.outcome,
        payload.notes,
    )
    .fetch_one(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "interview",
        row.pid,
        "updated",
        caller.actor(),
        Some(serde_json::json!({ "outcome": row.outcome })),
    )
    .await?;
    tx.commit().await?;
    format::json(row_json(&row)?)
}

/// `POST /api/workers/{pid}/onboarding` — add checklist items.
#[debug_handler]
async fn add_onboarding(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<OnboardingPayload>,
) -> Result<Response> {
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    let mut problems = Problems::new();
    if payload.items.is_empty() {
        problems.push("items must be non-empty".to_string());
    }
    if payload.items.len() > 64 {
        problems.push("items exceeds 64 entries".to_string());
    }
    for item in &payload.items {
        problems.require_text("items[].name", &item.name);
    }
    ensure_valid(&problems.into_vec())?;
    let mut tx = db::pool(&ctx).begin().await?;
    let mut pids = Vec::with_capacity(payload.items.len());
    for item in &payload.items {
        let pid = Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO onboarding_items (pid, worker_pid, name, mandatory, status) VALUES ($1, $2, $3, $4, 'pending')",
            pid,
            worker.pid,
            item.name,
            item.mandatory,
        )
        .execute(&mut *tx)
        .await?;
        pids.push(pid.to_string());
    }
    db::audit(
        &mut tx,
        "worker",
        worker.pid,
        "onboarding_items_added",
        caller.actor(),
        Some(serde_json::json!({ "count": pids.len() })),
    )
    .await?;
    tx.commit().await?;
    format::json(serde_json::json!({ "pids": pids }))
}

/// Every column of an onboarding item, for loading into its model.
async fn onboarding_of(ctx: &AppContext, worker: Uuid) -> Result<Vec<onboarding_items::Model>> {
    Ok(sqlx::query_as!(
        onboarding_items::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, name, mandatory, status,
                  waived_reason, completed_at AS "completed_at: _", deleted_at AS "deleted_at: _"
             FROM onboarding_items WHERE worker_pid = $1 AND deleted_at IS NULL ORDER BY id"#,
        worker,
    )
    .fetch_all(db::pool(ctx))
    .await?)
}

/// `GET /api/workers/{pid}/onboarding`.
#[debug_handler]
async fn list_onboarding(
    State(ctx): State<AppContext>,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    format::json(onboarding_of(&ctx, worker.pid).await?)
}

/// Shared item-transition body for complete/waive.
async fn finish_item(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: &str,
    status: &str,
    reason: Option<String>,
) -> Result<Response> {
    let item = sqlx::query!(
        "SELECT id, status FROM onboarding_items WHERE pid = $1 AND deleted_at IS NULL",
        records::parse_pid(pid)?,
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)?;
    if item.status != "pending" {
        return Err(unprocessable(&format!("item is already {}", item.status)));
    }
    let mut tx = db::pool(ctx).begin().await?;
    let row = sqlx::query_as!(
        onboarding_items::Model,
        r#"UPDATE onboarding_items SET status = $2, waived_reason = $3, completed_at = $4 WHERE id = $1
           RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, name, mandatory,
                     status, waived_reason, completed_at AS "completed_at: _", deleted_at AS "deleted_at: _""#,
        item.id,
        status,
        reason,
        chrono::DateTime::<chrono::FixedOffset>::from(chrono::Utc::now()),
    )
    .fetch_one(&mut *tx)
    .await?;
    let snapshot = reason.map(|r| serde_json::json!({ "reason": r }));
    // One literal action per way an item is finished.
    if status == "waived" {
        db::audit(
            &mut tx,
            "onboarding_item",
            row.pid,
            "onboarding_item_waived",
            caller.actor(),
            snapshot,
        )
        .await?;
    } else {
        db::audit(
            &mut tx,
            "onboarding_item",
            row.pid,
            "onboarding_item_complete",
            caller.actor(),
            snapshot,
        )
        .await?;
    }
    streaming::emit_in(
        &mut tx,
        "onboarding_item",
        "onboarding_item_completed",
        &row.pid.to_string(),
        &row.name,
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(row_json(&row)?)
}

/// `POST /api/onboarding-items/{pid}/complete`.
#[debug_handler]
async fn complete_item(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    finish_item(&ctx, &caller, &pid, "complete", None).await
}

/// `POST /api/onboarding-items/{pid}/waive` — waive with a recorded
/// reason (WPM-R3).
#[debug_handler]
async fn waive_item(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<WaivePayload>,
) -> Result<Response> {
    if payload.reason.trim().is_empty() {
        return Err(unprocessable("a waive requires a reason"));
    }
    finish_item(&ctx, &caller, &pid, "waived", Some(payload.reason)).await
}

/// The acquisition routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/requisitions", post(create_requisition))
        .add("/requisitions", get(list_requisitions))
        .add("/requisitions/{pid}", get(get_requisition))
        .add("/requisitions/{pid}/status", post(requisition_status))
        .add("/requisitions/{pid}/applications", post(create_application))
        .add("/requisitions/{pid}/applications", get(list_applications))
        .add("/candidates", post(create_candidate))
        .add("/candidates", get(list_candidates))
        .add("/applications/{pid}/stage", post(application_stage))
        .add("/applications/{pid}/interviews", post(create_interview))
        .add("/applications/{pid}/interviews", get(list_interviews))
        .add("/interviews/{pid}", put(interview_outcome))
        .add("/workers/{pid}/onboarding", post(add_onboarding))
        .add("/workers/{pid}/onboarding", get(list_onboarding))
        .add("/onboarding-items/{pid}/complete", post(complete_item))
        .add("/onboarding-items/{pid}/waive", post(waive_item))
}
