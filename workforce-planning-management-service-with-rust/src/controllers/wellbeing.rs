//! Wellbeing — configurable health-entitlement rules (e.g. NHS
//! vaccination cohorts) and self-service prompts (WPM-R25).
//!
//! The rules are **configuration, not code** (cohorts change year to
//! year) and can predicate only on non-clinical facts — an age band
//! (birth date resolved best-effort via the upstream person client),
//! department, job title — per WPM-D17. WPM prompts and records the
//! worker's acknowledgement; it does not book appointments and
//! stores no vaccination status. Acknowledgements are worker-owned
//! (`$sub` ownership rules apply on the worker record); HR sees
//! **aggregate counts only** (WPM-D16 terms), and no manager view
//! exists at all.

use std::str::FromStr;

use super::row_json;
use axa::extract::{Json, Path};
use axa::prelude::*;
use serde::Deserialize;
use uuid::Uuid;

use super::{ensure_valid, record_rejection};
use crate::auth::{self, MaybeAuthUser};
use crate::db;
use crate::models::_entities::{entitlement_acknowledgements, wellbeing_entitlements};
use crate::models::records;
use crate::rules::pulse;
use crate::rules::wellbeing as rules;
use crate::validation::Problems;

/// A `{pid}` reference response.
#[derive(serde::Serialize)]
struct PidRef {
    pid: String,
}

impl PidRef {
    fn of(pid: Uuid) -> Self {
        Self {
            pid: pid.to_string(),
        }
    }
}

// ─── Entitlement rules (HR configuration) ───────────────────────────────────

/// `POST /api/wellbeing-entitlements` / `PUT …/{pid}` body. The only
/// predicates the shape can carry are non-clinical (WPM-D17): there is
/// deliberately no field a health-status cohort could be expressed in.
#[derive(Debug, Deserialize)]
struct EntitlementPayload {
    name: String,
    #[serde(default = "health_kind")]
    kind: String,
    #[serde(default)]
    benefit_plan_pid: Option<Uuid>,
    description: String,
    #[serde(default)]
    info_url: Option<String>,
    #[serde(default)]
    min_age: Option<i32>,
    #[serde(default)]
    max_age: Option<i32>,
    #[serde(default)]
    departments: Vec<String>,
    #[serde(default)]
    job_titles: Vec<String>,
    #[serde(default = "one_dose")]
    doses: i32,
    #[serde(default)]
    active_from: Option<chrono::NaiveDate>,
    #[serde(default)]
    active_until: Option<chrono::NaiveDate>,
}

fn one_dose() -> i32 {
    1
}

fn health_kind() -> String {
    "health".to_string()
}

impl EntitlementPayload {
    fn validate(&self) -> Vec<String> {
        let mut problems = Problems::new();
        problems.require_text("name", &self.name);
        problems.require_token("kind", rules::ENTITLEMENT_KINDS, &self.kind);
        if self.benefit_plan_pid.is_some() && self.kind != "benefit" {
            problems.push("benefit_plan_pid requires kind `benefit`");
        }
        problems.require_text("description", &self.description);
        problems.cap_opt("info_url", self.info_url.as_deref());
        problems.cap_list("departments", &self.departments);
        problems.cap_list("job_titles", &self.job_titles);
        if !rules::valid_doses(self.doses) {
            problems.push(format!("doses must be between 1 and {}", rules::MAX_DOSES));
        }
        if !rules::valid_age_band(self.min_age, self.max_age) {
            problems.push(format!(
                "age band must satisfy 0 <= min_age <= max_age <= {}",
                rules::MAX_AGE
            ));
        }
        if let (Some(from), Some(until)) = (self.active_from, self.active_until)
            && from > until
        {
            problems.push("active_from must not be after active_until");
        }
        problems.into_vec()
    }
}

/// `POST /api/wellbeing-entitlements` — add a rule (HR configuration).
#[debug_handler]
async fn create_entitlement(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<EntitlementPayload>,
) -> Result<Response> {
    ensure_valid(&payload.validate())?;
    if let Some(plan_pid) = payload.benefit_plan_pid {
        ensure_benefit_plan(&ctx, plan_pid).await?;
    }
    let pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO wellbeing_entitlements (pid, name, kind, benefit_plan_pid, description, info_url, min_age, max_age,
                                             departments, job_titles, doses, active_from, active_until, deleted_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, NULL)",
        pid,
        payload.name,
        payload.kind,
        payload.benefit_plan_pid,
        payload.description,
        payload.info_url,
        payload.min_age,
        payload.max_age,
        serde_json::json!(payload.departments),
        serde_json::json!(payload.job_titles),
        payload.doses,
        payload.active_from,
        payload.active_until,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "wellbeing_entitlement",
        pid,
        "created",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(PidRef::of(pid))
}

/// A `404` unless a live benefit plan has this pid.
async fn ensure_benefit_plan(ctx: &AppContext, pid: Uuid) -> Result<()> {
    sqlx::query_scalar!(
        r#"SELECT 1 AS "one!" FROM benefit_plans WHERE pid = $1 AND deleted_at IS NULL"#,
        pid
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)?;
    Ok(())
}

/// The live entitlement rules, of one kind or all, by name.
async fn live_entitlements(
    ctx: &AppContext,
    kind: Option<&str>,
) -> Result<Vec<wellbeing_entitlements::Model>> {
    Ok(sqlx::query_as!(
        wellbeing_entitlements::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, name, kind, benefit_plan_pid,
                  description, info_url, min_age, max_age, departments, job_titles, doses, active_from, active_until,
                  deleted_at AS "deleted_at: _"
             FROM wellbeing_entitlements
            WHERE deleted_at IS NULL AND ($1::text IS NULL OR kind = $1)
            ORDER BY name"#,
        kind,
    )
    .fetch_all(db::pool(ctx))
    .await?)
}

/// A survey that is not soft-deleted, or `404`.
async fn find_survey(ctx: &AppContext, pid: &str) -> Result<Survey> {
    sqlx::query_as!(
        Survey,
        "SELECT pid, name, question, active_from, active_until FROM pulse_surveys
          WHERE pid = $1 AND deleted_at IS NULL",
        records::parse_pid(pid)?,
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)
}

/// The columns of a pulse survey these handlers read.
struct Survey {
    pid: Uuid,
    name: String,
    question: String,
    active_from: Option<chrono::NaiveDate>,
    active_until: Option<chrono::NaiveDate>,
}

/// `GET /api/wellbeing-entitlements?kind=` query.
#[derive(Debug, Deserialize)]
struct ListQuery {
    kind: Option<String>,
}

/// `GET /api/wellbeing-entitlements` — the configured rules,
/// optionally filtered by kind (`health | benefit`).
#[debug_handler]
async fn list_entitlements(
    axa::extract::Query(query): axa::extract::Query<ListQuery>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.token_opt("kind", rules::ENTITLEMENT_KINDS, query.kind.as_deref());
    ensure_valid(&problems.into_vec())?;
    let rows = live_entitlements(&ctx, query.kind.as_deref()).await?;
    format::json(rows)
}

/// `PUT /api/wellbeing-entitlements/{pid}` — restate a rule (cohorts
/// change year to year; existing acknowledgements are untouched).
#[debug_handler]
async fn update_entitlement(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<EntitlementPayload>,
) -> Result<Response> {
    ensure_valid(&payload.validate())?;
    if let Some(plan_pid) = payload.benefit_plan_pid {
        ensure_benefit_plan(&ctx, plan_pid).await?;
    }
    let row = find_entitlement(&ctx, &pid).await?;
    let row_pid = row.pid;
    let mut tx = db::pool(&ctx).begin().await?;
    let updated = sqlx::query_as!(
        wellbeing_entitlements::Model,
        r#"UPDATE wellbeing_entitlements
              SET name = $2, kind = $3, benefit_plan_pid = $4, description = $5, info_url = $6, min_age = $7,
                  max_age = $8, departments = $9, job_titles = $10, doses = $11, active_from = $12,
                  active_until = $13
            WHERE id = $1
            RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, name, kind,
                      benefit_plan_pid, description, info_url, min_age, max_age, departments, job_titles, doses,
                      active_from, active_until, deleted_at AS "deleted_at: _""#,
        row.id,
        payload.name,
        payload.kind,
        payload.benefit_plan_pid,
        payload.description,
        payload.info_url,
        payload.min_age,
        payload.max_age,
        serde_json::json!(payload.departments),
        serde_json::json!(payload.job_titles),
        payload.doses,
        payload.active_from,
        payload.active_until,
    )
    .fetch_one(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "wellbeing_entitlement",
        row_pid,
        "updated",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(row_json(&updated)?)
}

/// `DELETE /api/wellbeing-entitlements/{pid}` — soft-close a rule
/// (acknowledgement history is kept).
#[debug_handler]
async fn delete_entitlement(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let row = find_entitlement(&ctx, &pid).await?;
    let row_pid = row.pid;
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "UPDATE wellbeing_entitlements SET deleted_at = now() WHERE id = $1",
        row.id
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "wellbeing_entitlement",
        row_pid,
        "deleted",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(serde_json::json!({ "deleted": row_pid }))
}

// ─── Self-service prompts ───────────────────────────────────────────────────

/// The department / job-title list off a stored rule's JSONB column.
fn string_list(value: &serde_json::Value) -> Vec<String> {
    serde_json::from_value(value.clone()).unwrap_or_default()
}

/// `GET /api/workers/{pid}/wellbeing-prompts` — the worker's live
/// prompts: every active rule they are eligible for and have not
/// acknowledged, plus at most **one** reminder per multi-dose course
/// they acknowledged `booked`/`done` (serving a reminder stamps it, so
/// it appears exactly once). Eligibility runs over age (resolved
/// best-effort from the person service — unknown age fails an
/// age-banded rule), department, and job title; the payload names that
/// derivation. Worker-owned: `$sub` ownership policies on the
/// worker record apply.
#[debug_handler]
async fn worker_prompts(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    auth::authorize_record(
        &caller,
        authentication_verifier::Action::Read,
        &auth::worker_resource_attrs(&worker),
    )
    .map_err(record_rejection)?;
    let today = chrono::Utc::now().date_naive();
    let age = match entity_ref::EntityRef::from_str(&worker.person_ref) {
        Ok(person) => crate::clients::birth_date(&person)
            .await
            .and_then(|born| rules::age_on(born, today)),
        Err(_) => None,
    };
    let entitlements = live_entitlements(&ctx, None).await?;
    let acks = sqlx::query!(
        "SELECT entitlement_pid, response, reminded_on FROM entitlement_acknowledgements
          WHERE worker_pid = $1 ORDER BY id",
        worker.pid
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    // The plans this worker is live-enrolled in: a plan-linked rule
    // goes quiet for them — derived here, never stored (WPM-D18).
    let enrolled_plans: Vec<Uuid> = sqlx::query!(
        "SELECT plan_pid, ends_on FROM benefit_enrollments
          WHERE worker_pid = $1 AND deleted_at IS NULL ORDER BY id",
        worker.pid
    )
    .fetch_all(db::pool(&ctx))
    .await?
    .into_iter()
    .filter(|enrollment| enrollment.ends_on.is_none_or(|end| end >= today))
    .map(|enrollment| enrollment.plan_pid)
    .collect();
    let mut prompts = Vec::new();
    for entitlement in &entitlements {
        if entitlement
            .benefit_plan_pid
            .is_some_and(|plan| enrolled_plans.contains(&plan))
        {
            continue; // already enrolled — signposting is done
        }
        let departments = string_list(&entitlement.departments);
        let job_titles = string_list(&entitlement.job_titles);
        let predicates = rules::Predicates {
            min_age: entitlement.min_age,
            max_age: entitlement.max_age,
            departments: &departments,
            job_titles: &job_titles,
            active_from: entitlement.active_from,
            active_until: entitlement.active_until,
        };
        if !rules::eligible(
            &predicates,
            age,
            &worker.department,
            &worker.job_title,
            today,
        ) {
            continue;
        }
        let ack = acks.iter().find(|a| a.entitlement_pid == entitlement.pid);
        let state = rules::prompt_state(
            entitlement.doses,
            ack.map(|a| a.response.as_str()),
            ack.is_some_and(|a| a.reminded_on.is_some()),
        );
        let kind = match state {
            rules::PromptState::Prompt => "prompt",
            // Reading changes nothing: the reminder stays until the worker answers it (WPM-T247).
            rules::PromptState::Reminder => "reminder",
            rules::PromptState::Quiet => continue,
        };
        prompts.push(serde_json::json!({
            "kind": kind,
            "entitlement_kind": entitlement.kind,
            "benefit_plan_pid": entitlement.benefit_plan_pid,
            "entitlement_pid": entitlement.pid,
            "name": entitlement.name,
            "description": entitlement.description,
            "info_url": entitlement.info_url,
            "doses": entitlement.doses,
            "response": ack.map(|a| a.response.clone()),
        }));
    }
    format::json(serde_json::json!({
        "as_of": today,
        "age_known": age.is_some(),
        "derivation": "eligibility over age band (unknown age fails a banded rule), \
                       department, and job title only; informational — no response \
                       is required and declining has no recorded consequence",
        "prompts": prompts,
    }))
}

/// `POST /api/workers/{pid}/wellbeing-acknowledgements` body.
#[derive(Debug, Deserialize)]
struct AcknowledgePayload {
    entitlement_pid: Uuid,
    response: String,
}

/// `POST /api/workers/{pid}/wellbeing-acknowledgements` — record (or
/// restate) the worker's response to a prompt: one row per
/// worker + entitlement, upserted. The stored fact is the
/// acknowledgement of a prompt — never a vaccination status (WPM-D17).
/// Worker-owned (`$sub` ownership policies apply); audited.
#[debug_handler]
async fn acknowledge(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<AcknowledgePayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_token("response", rules::RESPONSES, &payload.response);
    ensure_valid(&problems.into_vec())?;
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    auth::authorize_record(
        &caller,
        authentication_verifier::Action::Write,
        &auth::worker_resource_attrs(&worker),
    )
    .map_err(record_rejection)?;
    let entitlement = sqlx::query!(
        "SELECT pid, doses FROM wellbeing_entitlements WHERE pid = $1 AND deleted_at IS NULL",
        payload.entitlement_pid
    )
    .fetch_optional(db::pool(&ctx))
    .await?
    .ok_or(Error::NotFound)?;
    let today = chrono::Utc::now().date_naive();
    let existing = sqlx::query!(
        "SELECT id, response, reminded_on FROM entitlement_acknowledgements
          WHERE entitlement_pid = $1 AND worker_pid = $2 LIMIT 1",
        entitlement.pid,
        worker.pid,
    )
    .fetch_optional(db::pool(&ctx))
    .await?;
    let mut tx = db::pool(&ctx).begin().await?;
    let row = match existing {
        Some(row) => {
            // Answering the one reminder uses it up, whatever the answer.
            let answers_reminder = rules::prompt_state(
                entitlement.doses,
                Some(row.response.as_str()),
                row.reminded_on.is_some(),
            ) == rules::PromptState::Reminder;
            sqlx::query_as!(
                entitlement_acknowledgements::Model,
                r#"UPDATE entitlement_acknowledgements
                      SET response = $2, responded_on = $3,
                          reminded_on = CASE WHEN $4 THEN $3 ELSE reminded_on END
                    WHERE id = $1
                    RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid,
                              entitlement_pid, worker_pid, response, responded_on, reminded_on"#,
                row.id,
                payload.response,
                today,
                answers_reminder,
            )
            .fetch_one(&mut *tx)
            .await?
        }
        None => {
            sqlx::query_as!(
                entitlement_acknowledgements::Model,
                r#"INSERT INTO entitlement_acknowledgements (pid, entitlement_pid, worker_pid, response, responded_on,
                                                            reminded_on)
                   VALUES ($1, $2, $3, $4, $5, NULL)
                   RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid,
                             entitlement_pid, worker_pid, response, responded_on, reminded_on"#,
                Uuid::new_v4(),
                entitlement.pid,
                worker.pid,
                payload.response,
                today,
            )
            .fetch_one(&mut *tx)
            .await?
        }
    };
    db::audit(
        &mut tx,
        "entitlement_acknowledgement",
        row.pid,
        "acknowledged",
        caller.actor(),
        Some(serde_json::json!({ "response": payload.response })),
    )
    .await?;
    tx.commit().await?;
    format::json(row_json(&row)?)
}

// ─── HR aggregate view ──────────────────────────────────────────────────────

/// `GET /api/wellbeing/uptake` — HR's view is **aggregate counts
/// only** (WPM-R25): per entitlement, the acknowledgement counts by
/// response and the uptake rate with its terms (WPM-D16). A
/// plan-linked rule additionally reports **enrolment conversion** —
/// of the distinct workers who acknowledged, how many now hold a
/// live enrolment in the linked plan (derived per request, never
/// stored; WPM-D18). No worker appears in the payload, and there is
/// deliberately no per-manager or per-worker variant of this view.
#[debug_handler]
async fn uptake(State(ctx): State<AppContext>) -> Result<Response> {
    let entitlements = live_entitlements(&ctx, None).await?;
    let acks = sqlx::query!(
        "SELECT entitlement_pid, worker_pid, response FROM entitlement_acknowledgements ORDER BY id"
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let today = chrono::Utc::now().date_naive();
    // Live (plan, worker) enrolment pairs, for the conversion terms.
    let live_enrollments: std::collections::HashSet<(Uuid, Uuid)> = sqlx::query!(
        "SELECT plan_pid, worker_pid, ends_on FROM benefit_enrollments WHERE deleted_at IS NULL"
    )
    .fetch_all(db::pool(&ctx))
    .await?
    .into_iter()
    .filter(|enrollment| enrollment.ends_on.is_none_or(|end| end >= today))
    .map(|enrollment| (enrollment.plan_pid, enrollment.worker_pid))
    .collect();
    let view: Vec<serde_json::Value> = entitlements
        .iter()
        .map(|entitlement| {
            let mut by_response: std::collections::BTreeMap<&str, usize> =
                rules::RESPONSES.iter().map(|r| (*r, 0)).collect();
            let mut acknowledgers: std::collections::HashSet<Uuid> =
                std::collections::HashSet::new();
            for ack in acks.iter().filter(|a| a.entitlement_pid == entitlement.pid) {
                if let Some(count) = by_response.get_mut(ack.response.as_str()) {
                    *count += 1;
                }
                acknowledgers.insert(ack.worker_pid);
            }
            #[allow(clippy::cast_precision_loss)] // display ratio
            let conversion = entitlement.benefit_plan_pid.map(|plan| {
                let enrolled = acknowledgers
                    .iter()
                    .filter(|worker| live_enrollments.contains(&(plan, **worker)))
                    .count();
                let value = crate::rules::display::ratio(enrolled, acknowledgers.len())
                    .map_or(serde_json::Value::Null, |v| serde_json::json!(v));
                serde_json::json!({
                    "numerator": enrolled, "denominator": acknowledgers.len(), "value": value,
                })
            });
            let uptaken = by_response["booked"] + by_response["done"];
            let responded: usize = by_response.values().sum();
            let value = crate::rules::display::ratio(uptaken, responded)
                .map_or(serde_json::Value::Null, |v| serde_json::json!(v));
            serde_json::json!({
                "entitlement_pid": entitlement.pid,
                "name": entitlement.name,
                "kind": entitlement.kind,
                "by_response": by_response,
                "uptake_rate": {
                    "numerator": uptaken, "denominator": responded, "value": value,
                },
                "enrolment_conversion": conversion,
            })
        })
        .collect();
    format::json(serde_json::json!({
        "as_of": chrono::Utc::now(),
        "derivation": "uptake = (booked + done) / all acknowledgements; enrolment \
                       conversion (plan-linked rules) = acknowledgers now live-enrolled \
                       in the linked plan / distinct acknowledgers; counts only — \
                       responses are self-reported workflow facts, not clinical records, \
                       and no individual appears in this view",
        "entitlements": view,
    }))
}

// ─── Wellbeing pulse (anonymous, WPM-R28) ───────────────────────────────────

/// `POST /api/pulse-surveys` body.
#[derive(Debug, Deserialize)]
struct SurveyPayload {
    name: String,
    question: String,
    #[serde(default)]
    active_from: Option<chrono::NaiveDate>,
    #[serde(default)]
    active_until: Option<chrono::NaiveDate>,
}

/// `POST /api/pulse-surveys` — open a pulse survey (HR configuration;
/// audited normally — the survey is not sensitive, the answers are).
#[debug_handler]
async fn create_survey(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<SurveyPayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_text("name", &payload.name);
    problems.cap_text("name", &payload.name);
    problems.require_text("question", &payload.question);
    problems.cap_text("question", &payload.question);
    if let (Some(from), Some(until)) = (payload.active_from, payload.active_until)
        && from > until
    {
        problems.push("active_from must not be after active_until");
    }
    ensure_valid(&problems.into_vec())?;
    let pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO pulse_surveys (pid, name, question, active_from, active_until, deleted_at)
         VALUES ($1, $2, $3, $4, $5, NULL)",
        pid,
        payload.name,
        payload.question,
        payload.active_from,
        payload.active_until,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "pulse_survey",
        pid,
        "created",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(PidRef::of(pid))
}

/// `GET /api/pulse-surveys` — the surveys, with their open state.
#[debug_handler]
async fn list_surveys(State(ctx): State<AppContext>) -> Result<Response> {
    let today = chrono::Utc::now().date_naive();
    let rows = sqlx::query_as!(
        Survey,
        "SELECT pid, name, question, active_from, active_until FROM pulse_surveys
          WHERE deleted_at IS NULL ORDER BY name"
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let view: Vec<serde_json::Value> = rows
        .iter()
        .map(|survey| {
            serde_json::json!({
                "pid": survey.pid,
                "name": survey.name,
                "question": survey.question,
                "active_from": survey.active_from,
                "active_until": survey.active_until,
                "open": pulse::survey_open(survey.active_from, survey.active_until, today),
            })
        })
        .collect();
    format::json(view)
}

/// `POST /api/pulse-surveys/{pid}/responses` body. The worker names
/// themself so the department can be derived and ownership enforced —
/// then the identity is dropped: the stored row has no author.
#[derive(Debug, Deserialize)]
struct PulseResponsePayload {
    worker_pid: Uuid,
    score: i32,
}

/// `POST /api/pulse-surveys/{pid}/responses` — submit one anonymous
/// score. The row stores survey + department + score + date only
/// (WPM-D20); the audit row records that a submission happened with
/// **no actor**; the response body returns no handle to the row.
#[debug_handler]
async fn submit_response(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<PulseResponsePayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    if !pulse::valid_score(payload.score) {
        problems.push(format!(
            "score must be between {} and {}",
            pulse::SCORE_MIN,
            pulse::SCORE_MAX
        ));
    }
    ensure_valid(&problems.into_vec())?;
    let survey = find_survey(&ctx, &pid).await?;
    let today = chrono::Utc::now().date_naive();
    if !pulse::survey_open(survey.active_from, survey.active_until, today) {
        return Err(super::unprocessable("this survey is not open"));
    }
    let worker = records::worker_by_pid(&ctx, payload.worker_pid).await?;
    auth::authorize_record(
        &caller,
        authentication_verifier::Action::Write,
        &auth::worker_resource_attrs(&worker),
    )
    .map_err(record_rejection)?;
    let response_pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO pulse_responses (pid, survey_pid, department, score, submitted_on) VALUES ($1, $2, $3, $4, $5)",
        response_pid,
        survey.pid,
        worker.department,
        payload.score,
        today,
    )
    .execute(&mut *tx)
    .await?;
    // Deliberately actor-less (WPM-D20): the trail records that a
    // submission happened, never who made it.
    db::audit(
        &mut tx,
        "pulse_response",
        response_pid,
        "submitted",
        None,
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(serde_json::json!({ "submitted": true }))
}

/// `GET /api/pulse-surveys/{pid}/results` — the k-floored aggregate
/// (WPM-D20): per-department cells and the overall block, each either
/// suppressed (below k = 5; count withheld too) or disclosed
/// (count, 1–5 distribution, mean). Counts are *responses*, never
/// *respondents* — no author link exists to dedupe on.
#[debug_handler]
async fn survey_results(
    State(ctx): State<AppContext>,
    Path(pid): Path<String>,
) -> Result<Response> {
    let survey = find_survey(&ctx, &pid).await?;
    let responses = sqlx::query!(
        "SELECT department, score FROM pulse_responses WHERE survey_pid = $1 ORDER BY id",
        survey.pid
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let mut by_department: std::collections::BTreeMap<&str, Vec<i32>> =
        std::collections::BTreeMap::new();
    for response in &responses {
        by_department
            .entry(response.department.as_str())
            .or_default()
            .push(response.score);
    }
    let cell_json = |cell: &pulse::Cell| match cell {
        pulse::Cell::Suppressed => serde_json::json!({ "suppressed": true }),
        pulse::Cell::Disclosed {
            count,
            distribution,
            mean,
        } => serde_json::json!({
            "suppressed": false,
            "count": count,
            "distribution": distribution,
            "mean": mean,
        }),
    };
    let departments: Vec<serde_json::Value> = by_department
        .iter()
        .map(|(department, scores)| {
            let mut cell = cell_json(&pulse::aggregate_cell(scores));
            cell["department"] = serde_json::json!(department);
            cell
        })
        .collect();
    let all_scores: Vec<i32> = responses.iter().map(|r| r.score).collect();
    format::json(serde_json::json!({
        "as_of": chrono::Utc::now(),
        "survey": { "pid": survey.pid, "name": survey.name, "question": survey.question },
        "overall": cell_json(&pulse::aggregate_cell(&all_scores)),
        "departments": departments,
        "derivation": format!(
            "anonymous by construction — responses store no author, so counts are \
             responses, not respondents; any cell (department or overall) with fewer \
             than {} responses is suppressed, count withheld",
            pulse::K_ANONYMITY
        ),
    }))
}

/// Find one live entitlement rule by pid, or 404.
async fn find_entitlement(ctx: &AppContext, pid: &str) -> Result<wellbeing_entitlements::Model> {
    sqlx::query_as!(
        wellbeing_entitlements::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, name, kind, benefit_plan_pid,
                  description, info_url, min_age, max_age, departments, job_titles, doses, active_from, active_until,
                  deleted_at AS "deleted_at: _"
             FROM wellbeing_entitlements
            WHERE pid = $1 AND deleted_at IS NULL"#,
        records::parse_pid(pid)?,
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)
}

/// The wellbeing routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/wellbeing-entitlements", post(create_entitlement))
        .add("/wellbeing-entitlements", get(list_entitlements))
        .add("/wellbeing-entitlements/{pid}", put(update_entitlement))
        .add("/wellbeing-entitlements/{pid}", delete(delete_entitlement))
        .add("/workers/{pid}/wellbeing-prompts", get(worker_prompts))
        .add(
            "/workers/{pid}/wellbeing-acknowledgements",
            post(acknowledge),
        )
        .add("/wellbeing/uptake", get(uptake))
        .add("/pulse-surveys", post(create_survey))
        .add("/pulse-surveys", get(list_surveys))
        .add("/pulse-surveys/{pid}/responses", post(submit_response))
        .add("/pulse-surveys/{pid}/results", get(survey_results))
}
