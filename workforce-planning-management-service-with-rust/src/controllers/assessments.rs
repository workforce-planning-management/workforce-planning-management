//! Assessments — aptitude, personality, psychometric, and selection
//! tests across the hiring pipeline and talent development (WPM-R20).
//!
//! Three record kinds: an **instrument** catalog (the named test, its
//! category, and the scales it reports), an **assessment** (one
//! administration of an instrument to one candidate or worker,
//! optionally tied to an application), and per-scale **results**.
//!
//! The pure rules live in [`crate::rules::assessment`]: the
//! category↔scale mapping (with the deliberate psychometric overlap),
//! the lifecycle machine, the score bounds, the band split, currency
//! (completed and unexpired), and the gap list. This module only wires
//! them, persists, audits, and emits.
//!
//! **Results are sensitive personal data.** They profile cognition and
//! behaviour, so — like salary and payslips — every read path honours
//! the ABAC `mask` obligation ([`mask_result`]: the scale and the band
//! survive; raw scores, percentiles, and narratives do not) and every
//! unmasked read of scored results is audited.
//!
//! The derived views report **real scores only**: a mean is `null`
//! rather than zero when nothing carries a percentile, and every
//! payload names its derivation.

use super::row_json;
use axa::extract::{Json, Path};
use axa::prelude::*;
use serde::Deserialize;
use std::collections::BTreeMap;
use uuid::Uuid;

use super::{ensure_valid, record_rejection, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::db;
use crate::models::_entities::{assessment_instruments, assessment_results, assessments};
use crate::models::records;
use crate::rules::assessment as rules;
use crate::streaming;
use crate::validation::Problems;

/// The live instrument with this pid, else `404`.
async fn find_instrument(ctx: &AppContext, pid: Uuid) -> Result<assessment_instruments::Model> {
    sqlx::query_as!(
        assessment_instruments::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, name, category, provider, scales,
                  duration_minutes, validity_months, deleted_at AS "deleted_at: _"
             FROM assessment_instruments WHERE pid = $1 AND deleted_at IS NULL"#,
        pid,
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)
}

/// Every instrument, withdrawn ones too (a sitting keeps its instrument).
async fn all_instruments(ctx: &AppContext) -> Result<Vec<assessment_instruments::Model>> {
    Ok(sqlx::query_as!(
        assessment_instruments::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, name, category, provider, scales,
                  duration_minutes, validity_months, deleted_at AS "deleted_at: _"
             FROM assessment_instruments ORDER BY id"#,
    )
    .fetch_all(db::pool(ctx))
    .await?)
}

/// The live assessment with this pid, else `404`.
async fn find_assessment(ctx: &AppContext, pid: Uuid) -> Result<assessments::Model> {
    sqlx::query_as!(
        assessments::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, instrument_pid, subject_kind,
                  subject_pid, application_pid, status, scheduled_on, completed_on, expires_on, administered_by, notes,
                  deleted_at AS "deleted_at: _"
             FROM assessments WHERE pid = $1 AND deleted_at IS NULL"#,
        pid,
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)
}

/// A sitting's results, by scale.
async fn results_of(ctx: &AppContext, assessment: Uuid) -> Result<Vec<assessment_results::Model>> {
    Ok(sqlx::query_as!(
        assessment_results::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, assessment_pid, scale, raw_score,
                  max_score, percentile, band, narrative
             FROM assessment_results WHERE assessment_pid = $1 ORDER BY scale"#,
        assessment,
    )
    .fetch_all(db::pool(ctx))
    .await?)
}

/// What the assessment views read of a live application.
struct Application {
    pid: Uuid,
    candidate_pid: Uuid,
    stage: String,
}

/// The live application with this pid, else `404`.
async fn find_application(ctx: &AppContext, pid: Uuid) -> Result<Application> {
    sqlx::query_as!(
        Application,
        "SELECT pid, candidate_pid, stage FROM applications WHERE pid = $1 AND deleted_at IS NULL",
        pid,
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)
}

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

// ─── Instruments ────────────────────────────────────────────────────────────

/// `POST /api/assessment-instruments` body.
#[derive(Debug, Deserialize)]
struct InstrumentPayload {
    name: String,
    category: String,
    #[serde(default)]
    provider: Option<String>,
    /// The scales this instrument reports; each must suit the category.
    #[serde(default)]
    scales: Vec<String>,
    #[serde(default)]
    duration_minutes: Option<i32>,
    /// How long a sitting stays current, in months (drives `expires_on`
    /// when an assessment completes).
    #[serde(default)]
    validity_months: Option<i32>,
}

/// `POST /api/assessment-instruments` — add a test to the catalog.
///
/// Every declared scale must be permitted by the category
/// ([`rules::category_permits`]) — `psychometric` spans aptitude and
/// personality, everything else stays in its own lane.
#[debug_handler]
async fn create_instrument(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<InstrumentPayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_text("name", &payload.name);
    problems.require_token("category", rules::ASSESSMENT_CATEGORIES, &payload.category);
    problems.cap_opt("provider", payload.provider.as_deref());
    problems.cap_list("scales", &payload.scales);
    for (index, scale) in payload.scales.iter().enumerate() {
        problems.require_token(&format!("scales[{index}]"), rules::ASSESSMENT_SCALES, scale);
        if rules::ASSESSMENT_SCALES.contains(&scale.as_str())
            && !rules::category_permits(&payload.category, scale)
        {
            problems.push(format!(
                "scales[{index}]: `{scale}` is not measured by a `{}` instrument",
                payload.category
            ));
        }
    }
    if payload.duration_minutes.is_some_and(|m| m <= 0) {
        problems.push("duration_minutes must be positive");
    }
    if payload.validity_months.is_some_and(|m| m <= 0) {
        problems.push("validity_months must be positive");
    }
    ensure_valid(&problems.into_vec())?;

    let pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO assessment_instruments (pid, name, category, provider, scales, duration_minutes, validity_months)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
        pid,
        payload.name,
        payload.category,
        payload.provider,
        serde_json::json!(payload.scales),
        payload.duration_minutes,
        payload.validity_months,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "assessment_instrument",
        pid,
        "created",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(PidRef::of(pid))
}

/// Query for the instrument catalog.
#[derive(Debug, Deserialize)]
struct InstrumentQuery {
    category: Option<String>,
}

/// `GET /api/assessment-instruments?category=` — the catalog.
#[debug_handler]
async fn list_instruments(
    axa::extract::Query(query): axa::extract::Query<InstrumentQuery>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    if let Some(category) = &query.category
        && !rules::ASSESSMENT_CATEGORIES.contains(&category.as_str())
    {
        return Err(unprocessable(&format!(
            "unknown category `{category}` (categories: {:?})",
            rules::ASSESSMENT_CATEGORIES
        )));
    }
    let rows = sqlx::query_as!(
        assessment_instruments::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, name, category, provider, scales,
                  duration_minutes, validity_months, deleted_at AS "deleted_at: _"
             FROM assessment_instruments
            WHERE deleted_at IS NULL AND ($1::text IS NULL OR category = $1)
            ORDER BY name"#,
        query.category,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    format::json(rows)
}

// ─── Assessments ────────────────────────────────────────────────────────────

/// `POST /api/assessments` body — schedule (or record) a sitting.
#[derive(Debug, Deserialize)]
struct AssessmentPayload {
    instrument_pid: Uuid,
    /// `candidate` or `worker`.
    subject_kind: String,
    /// The candidate's or worker's pid (existence is checked).
    subject_pid: Uuid,
    /// The application this sitting belongs to, for a hiring process.
    #[serde(default)]
    application_pid: Option<Uuid>,
    #[serde(default)]
    scheduled_on: Option<chrono::NaiveDate>,
    #[serde(default)]
    administered_by: Option<String>,
    #[serde(default)]
    notes: Option<String>,
}

/// `POST /api/assessments` — schedule a sitting for a candidate or an
/// worker. The subject and (when given) the application must exist;
/// an application-linked sitting must belong to that application's
/// candidate, so a result can never be filed against the wrong hire.
#[debug_handler]
async fn create_assessment(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<AssessmentPayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_token(
        "subject_kind",
        rules::ASSESSMENT_SUBJECTS,
        &payload.subject_kind,
    );
    problems.cap_opt("administered_by", payload.administered_by.as_deref());
    problems.cap_opt("notes", payload.notes.as_deref());
    ensure_valid(&problems.into_vec())?;

    let instrument = find_instrument(&ctx, payload.instrument_pid).await?;
    let subject_name =
        subject_display_name(&ctx, &payload.subject_kind, payload.subject_pid, &caller).await?;

    if let Some(application_pid) = payload.application_pid {
        let application = find_application(&ctx, application_pid).await?;
        if payload.subject_kind != "candidate" {
            return Err(unprocessable(
                "only a candidate assessment can belong to an application",
            ));
        }
        if application.candidate_pid != payload.subject_pid {
            return Err(unprocessable(
                "the application belongs to a different candidate",
            ));
        }
    }

    let pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO assessments (pid, instrument_pid, subject_kind, subject_pid, application_pid, status, scheduled_on,
                                  administered_by, notes)
         VALUES ($1, $2, $3, $4, $5, 'scheduled', $6, $7, $8)",
        pid,
        instrument.pid,
        payload.subject_kind,
        payload.subject_pid,
        payload.application_pid,
        payload.scheduled_on,
        payload.administered_by,
        payload.notes,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "assessment",
        pid,
        "created",
        caller.actor(),
        Some(serde_json::json!({
            "instrument": instrument.name,
            "category": instrument.category,
            "subject_kind": payload.subject_kind,
        })),
    )
    .await?;
    streaming::emit_in(
        &mut tx,
        "assessment",
        "created",
        &pid.to_string(),
        &instrument.name,
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    tracing::debug!(subject = %subject_name, "assessment scheduled");
    format::json(PidRef::of(pid))
}

/// `POST /api/assessments/{pid}/status` body.
#[derive(Debug, Deserialize)]
struct StatusPayload {
    to: String,
}

/// `POST /api/assessments/{pid}/status` — the lifecycle move (the pure
/// machine refuses an illegal one). Completing stamps `completed_on`
/// and, when the instrument declares a `validity_months`, derives
/// `expires_on` from it — so currency is a recorded fact, not an
/// assumption at read time.
///
/// Completion requires at least one recorded result: "completed" must
/// not assert a scoring that never happened.
#[debug_handler]
async fn assessment_status(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<StatusPayload>,
) -> Result<Response> {
    let row = find_assessment(&ctx, records::parse_pid(&pid)?).await?;
    rules::assessment_transition(&row.status, &payload.to)
        .map_err(|reason| unprocessable(&reason))?;

    let today = chrono::Utc::now().date_naive();
    let instrument = find_instrument(&ctx, row.instrument_pid).await?;
    let from = row.status.clone();
    let row_pid = row.pid;

    if payload.to == "completed" {
        let scored = sqlx::query_scalar!(
            r#"SELECT 1 AS "one!" FROM assessment_results WHERE assessment_pid = $1 LIMIT 1"#,
            row_pid
        )
        .fetch_optional(db::pool(&ctx))
        .await?;
        if scored.is_none() {
            return Err(unprocessable(
                "record at least one result before completing the assessment",
            ));
        }
    }

    // Completing stamps the day and, when the instrument has a validity, the expiry; other moves change only the status.
    let completing = payload.to == "completed";
    let expires = instrument
        .validity_months
        .map(|months| expiry_from(today, months));
    let mut tx = db::pool(&ctx).begin().await?;
    let updated = sqlx::query_as!(
        assessments::Model,
        r#"UPDATE assessments
              SET status = $2,
                  completed_on = CASE WHEN $3 THEN $4::date ELSE completed_on END,
                  expires_on = CASE WHEN $3 AND $5 THEN $6::date ELSE expires_on END
            WHERE id = $1
        RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, instrument_pid, subject_kind,
                  subject_pid, application_pid, status, scheduled_on, completed_on, expires_on, administered_by, notes,
                  deleted_at AS "deleted_at: _""#,
        row.id,
        payload.to,
        completing,
        today,
        expires.is_some(),
        expires.flatten(),
    )
    .fetch_one(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "assessment",
        row_pid,
        "status_changed",
        caller.actor(),
        Some(serde_json::json!({ "from": from, "to": payload.to })),
    )
    .await?;
    streaming::emit_in(
        &mut tx,
        "assessment",
        &format!("assessment_{}", payload.to),
        &row_pid.to_string(),
        &instrument.name,
        caller.actor(),
        Some(serde_json::json!({ "from": from })),
    )
    .await?;
    tx.commit().await?;
    format::json(row_json(&updated)?)
}

/// The expiry date `months` after `from`, or `None` when the arithmetic
/// would overflow the calendar (never panics — security invariant 2).
fn expiry_from(from: chrono::NaiveDate, months: i32) -> Option<chrono::NaiveDate> {
    u32::try_from(months)
        .ok()
        .and_then(|m| from.checked_add_months(chrono::Months::new(m)))
}

/// `POST /api/assessments/{pid}/results` body — record (or replace) one
/// scale's outcome.
#[derive(Debug, Deserialize)]
struct ResultPayload {
    scale: String,
    #[serde(default)]
    raw_score: Option<i32>,
    #[serde(default)]
    max_score: Option<i32>,
    /// Norm-referenced percentile, 0–100. The band is derived from it
    /// unless one is given explicitly.
    #[serde(default)]
    percentile: Option<i32>,
    #[serde(default)]
    band: Option<String>,
    #[serde(default)]
    narrative: Option<String>,
}

/// `POST /api/assessments/{pid}/results` — record a scale's outcome
/// (upsert: one row per assessment+scale).
///
/// The scale must be permitted by the **instrument's** category, and —
/// when the instrument declares its scales — must be one it reports.
/// Scores are bounded by the pure rules; the band is derived from the
/// percentile when not given.
#[debug_handler]
async fn record_result(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<ResultPayload>,
) -> Result<Response> {
    let assessment = find_assessment(&ctx, records::parse_pid(&pid)?).await?;
    let instrument = find_instrument(&ctx, assessment.instrument_pid).await?;

    let mut problems = Problems::new();
    problems.require_token("scale", rules::ASSESSMENT_SCALES, &payload.scale);
    problems.token_opt("band", rules::SCORE_BANDS, payload.band.as_deref());
    problems.cap_opt("narrative", payload.narrative.as_deref());
    if rules::ASSESSMENT_SCALES.contains(&payload.scale.as_str())
        && !rules::category_permits(&instrument.category, &payload.scale)
    {
        problems.push(format!(
            "`{}` is not measured by a `{}` assessment",
            payload.scale, instrument.category
        ));
    }
    if let Some(declared) = declared_scales(&instrument)
        && !declared.iter().any(|s| s == &payload.scale)
    {
        problems.push(format!(
            "`{}` is not one of the scales `{}` reports",
            payload.scale, instrument.name
        ));
    }
    if payload
        .percentile
        .is_some_and(|p| !rules::valid_percentile(p))
    {
        problems.push("percentile must be between 0 and 100");
    }
    match (payload.raw_score, payload.max_score) {
        (Some(raw), Some(max)) if !rules::valid_raw_score(raw, max) => {
            problems.push("raw_score must be between 0 and max_score, and max_score positive");
        }
        (Some(_), None) => problems.push("raw_score requires max_score"),
        _ => {}
    }
    if assessment.status == "cancelled" || assessment.status == "expired" {
        problems.push(format!(
            "cannot record a result on a `{}` assessment",
            assessment.status
        ));
    }
    ensure_valid(&problems.into_vec())?;

    let band = payload.band.clone().or_else(|| {
        payload
            .percentile
            .map(|p| rules::band_for_percentile(p).to_string())
    });

    let existing = sqlx::query!(
        "SELECT id, pid FROM assessment_results WHERE assessment_pid = $1 AND scale = $2 LIMIT 1",
        assessment.pid,
        payload.scale,
    )
    .fetch_optional(db::pool(&ctx))
    .await?;
    let mut tx = db::pool(&ctx).begin().await?;
    let result_pid = if let Some(found) = existing {
        sqlx::query!(
            "UPDATE assessment_results SET raw_score = $2, max_score = $3, percentile = $4, band = $5, narrative = $6
              WHERE id = $1",
            found.id,
            payload.raw_score,
            payload.max_score,
            payload.percentile,
            band,
            payload.narrative,
        )
        .execute(&mut *tx)
        .await?;
        found.pid
    } else {
        let pid = Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO assessment_results (pid, assessment_pid, scale, raw_score, max_score, percentile, band, narrative)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            pid,
            assessment.pid,
            payload.scale,
            payload.raw_score,
            payload.max_score,
            payload.percentile,
            band,
            payload.narrative,
        )
        .execute(&mut *tx)
        .await?;
        pid
    };
    // The scale is recorded; the score itself never enters the trail.
    db::audit(
        &mut tx,
        "assessment_result",
        result_pid,
        "recorded",
        caller.actor(),
        Some(serde_json::json!({ "assessment": assessment.pid, "scale": payload.scale })),
    )
    .await?;
    tx.commit().await?;
    format::json(PidRef::of(result_pid))
}

/// `GET /api/assessments/{pid}` — one assessment with its instrument
/// and its results.
///
/// Results are sensitive: the `mask` obligation redacts the scores
/// ([`mask_result`]), and an **unmasked** read of scored results is
/// audited.
#[debug_handler]
async fn get_assessment(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let assessment = find_assessment(&ctx, records::parse_pid(&pid)?).await?;
    let instrument = find_instrument(&ctx, assessment.instrument_pid).await?;
    let obligations = auth::authorize_record(
        &caller,
        authentication_verifier::Action::Read,
        &assessment_resource_attrs(&assessment, &instrument),
    )
    .map_err(record_rejection)?;
    let masked = obligations.iter().any(|o| o == "mask");

    let rows = results_of(&ctx, assessment.pid).await?;
    if !masked && !rows.is_empty() {
        // A read, audited on its own connection: nothing changed, so there is no transaction to join.
        let mut connection = db::pool(&ctx).acquire().await?;
        db::audit(
            &mut connection,
            "assessment",
            assessment.pid,
            "results_read",
            caller.actor(),
            Some(serde_json::json!({ "category": instrument.category, "scales": rows.len() })),
        )
        .await?;
    }
    let results: Vec<assessment_results::Model> = if masked {
        rows.into_iter().map(mask_result).collect()
    } else {
        rows
    };
    format::json(serde_json::json!({
        "assessment": assessment,
        "instrument": {
            "pid": instrument.pid, "name": instrument.name,
            "category": instrument.category, "provider": instrument.provider,
        },
        "masked": masked,
        "results": results,
    }))
}

/// `DELETE /api/assessments/{pid}` — withdraw a sitting (soft delete),
/// keeping the audit trail intact.
#[debug_handler]
async fn delete_assessment(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let assessment = find_assessment(&ctx, records::parse_pid(&pid)?).await?;
    let instrument = find_instrument(&ctx, assessment.instrument_pid).await?;
    auth::authorize_record(
        &caller,
        authentication_verifier::Action::Delete,
        &assessment_resource_attrs(&assessment, &instrument),
    )
    .map_err(record_rejection)?;

    let row_pid = assessment.pid;
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "UPDATE assessments SET deleted_at = $2 WHERE id = $1",
        assessment.id,
        chrono::DateTime::<chrono::FixedOffset>::from(chrono::Utc::now()),
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "assessment",
        row_pid,
        "deleted",
        caller.actor(),
        None,
    )
    .await?;
    streaming::emit_in(
        &mut tx,
        "assessment",
        "deleted",
        &row_pid.to_string(),
        &instrument.name,
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(serde_json::json!({ "pid": row_pid, "deleted": true }))
}

// ─── Derived views ──────────────────────────────────────────────────────────

/// Query for the subject profile.
#[derive(Debug, Deserialize)]
struct ProfileQuery {
    /// Judge currency as of this date (default today).
    as_of: Option<chrono::NaiveDate>,
}

/// `GET /api/candidates/{pid}/assessment-profile` — a candidate's
/// profile (see [`subject_profile`]).
#[debug_handler]
async fn candidate_profile(
    axa::extract::Query(query): axa::extract::Query<ProfileQuery>,
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let candidate = live_candidate(&ctx, records::parse_pid(&pid)?).await?;
    subject_profile(&ctx, &caller, "candidate", candidate.pid, query.as_of).await
}

/// `GET /api/workers/{pid}/assessment-profile` — a worker's
/// profile. Authorized (and masked) at the **worker** level, so an
/// assessment profile is never a way around the worker's own
/// record-level policy.
#[debug_handler]
async fn worker_profile(
    axa::extract::Query(query): axa::extract::Query<ProfileQuery>,
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
    build_profile(&ctx, "worker", worker.pid, query.as_of, masked).await
}

/// Shared body of the candidate profile: authorize coarsely (there is
/// no per-candidate resource shape), then build.
async fn subject_profile(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    subject_kind: &str,
    subject_pid: Uuid,
    as_of: Option<chrono::NaiveDate>,
) -> Result<Response> {
    let obligations = auth::authorize_record(
        caller,
        authentication_verifier::Action::Read,
        &BTreeMap::from([
            ("subject_kind".to_string(), vec![subject_kind.to_string()]),
            ("record".to_string(), vec!["assessment".to_string()]),
        ]),
    )
    .map_err(record_rejection)?;
    let masked = obligations.iter().any(|o| o == "mask");
    build_profile(ctx, subject_kind, subject_pid, as_of, masked).await
}

/// Build one subject's assessment profile: per category, the sittings
/// recorded and current, the **current reading per scale** (the most
/// recently completed current assessment reporting it), the scales with
/// no current reading, and — for selection — the mean percentile with
/// its numerator and denominator.
///
/// Under the `mask` obligation the bands survive and every score
/// (including the mean) is withheld: an aggregate read must never
/// reveal more than the equivalent single read.
async fn build_profile(
    ctx: &AppContext,
    subject_kind: &str,
    subject_pid: Uuid,
    as_of: Option<chrono::NaiveDate>,
    masked: bool,
) -> Result<Response> {
    let as_of = as_of.unwrap_or_else(|| chrono::Utc::now().date_naive());
    let sittings = sqlx::query_as!(
        assessments::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, instrument_pid, subject_kind,
                  subject_pid, application_pid, status, scheduled_on, completed_on, expires_on, administered_by, notes,
                  deleted_at AS "deleted_at: _"
             FROM assessments WHERE subject_kind = $1 AND subject_pid = $2 AND deleted_at IS NULL ORDER BY id"#,
        subject_kind,
        subject_pid,
    )
    .fetch_all(db::pool(ctx))
    .await?;
    let instruments = all_instruments(ctx).await?;
    let instrument_of: BTreeMap<Uuid, &assessment_instruments::Model> =
        instruments.iter().map(|i| (i.pid, i)).collect();

    let mut categories = Vec::with_capacity(rules::ASSESSMENT_CATEGORIES.len());
    let mut selection_percentiles: Vec<i32> = Vec::new();

    for category in rules::ASSESSMENT_CATEGORIES {
        let in_category: Vec<&assessments::Model> = sittings
            .iter()
            .filter(|a| {
                instrument_of
                    .get(&a.instrument_pid)
                    .is_some_and(|i| i.category == *category)
            })
            .collect();
        let current: Vec<&&assessments::Model> = in_category
            .iter()
            .filter(|a| rules::is_current(&a.status, a.expires_on, as_of))
            .collect();

        // Latest current reading per scale; ties on the completion date
        // break on the later row id, so the reading is deterministic.
        let mut latest: BTreeMap<String, (&assessments::Model, assessment_results::Model)> =
            BTreeMap::new();
        for sitting in &current {
            for row in results_of(ctx, sitting.pid).await? {
                let replace = latest.get(&row.scale).is_none_or(|(held, _)| {
                    (sitting.completed_on, sitting.id) > (held.completed_on, held.id)
                });
                if replace {
                    latest.insert(row.scale.clone(), (sitting, row));
                }
            }
        }
        if *category == "selection" {
            selection_percentiles.extend(latest.values().filter_map(|(_, r)| r.percentile));
        }

        let scales: Vec<serde_json::Value> = latest
            .iter()
            .map(|(scale, (sitting, result))| {
                serde_json::json!({
                    "scale": scale,
                    "band": result.band,
                    "percentile": if masked { None } else { result.percentile },
                    "raw_score": if masked { None } else { result.raw_score },
                    "max_score": if masked { None } else { result.max_score },
                    "instrument": instrument_of.get(&sitting.instrument_pid).map(|i| i.name.clone()),
                    "completed_on": sitting.completed_on,
                    "assessment_pid": sitting.pid,
                })
            })
            .collect();
        let measured: Vec<String> = latest.keys().cloned().collect();

        categories.push(serde_json::json!({
            "category": category,
            "recorded": in_category.len(),
            "current": current.len(),
            "scales": scales,
            "scales_not_assessed": rules::scales_not_assessed(category, &measured),
        }));
    }

    let suitability = if masked {
        serde_json::Value::Null
    } else {
        rules::mean_percentile(&selection_percentiles).map_or(serde_json::Value::Null, |(sum, count, mean)| {
            serde_json::json!({ "numerator": sum, "denominator": count, "value": mean })
        })
    };

    format::json(serde_json::json!({
        "as_of": as_of,
        "subject": { "kind": subject_kind, "pid": subject_pid },
        "masked": masked,
        "derivation": "a scale's current reading = the most recently completed, unexpired \
                       assessment reporting it; selection suitability = the mean percentile of \
                       those current selection readings, real scores only (null when none carry \
                       a percentile)",
        "categories": categories,
        "selection_suitability": suitability,
    }))
}

/// `GET /api/applications/{pid}/assessments` — every sitting recorded
/// for one application's candidate, with the per-scale bands.
///
/// The hiring view: what the candidate has been asked to do, what is
/// outstanding, and how the completed sittings read. Scores obey the
/// `mask` obligation exactly as elsewhere.
#[debug_handler]
async fn application_assessments(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let application = find_application(&ctx, records::parse_pid(&pid)?).await?;
    let obligations = auth::authorize_record(
        &caller,
        authentication_verifier::Action::Read,
        &BTreeMap::from([
            ("record".to_string(), vec!["assessment".to_string()]),
            ("stage".to_string(), vec![application.stage.clone()]),
        ]),
    )
    .map_err(record_rejection)?;
    let masked = obligations.iter().any(|o| o == "mask");

    let sittings = sqlx::query!(
        "SELECT pid, instrument_pid, status, scheduled_on, completed_on FROM assessments
          WHERE application_pid = $1 AND deleted_at IS NULL ORDER BY id",
        application.pid,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let mut view = Vec::with_capacity(sittings.len());
    for sitting in &sittings {
        let instrument = find_instrument(&ctx, sitting.instrument_pid).await?;
        let rows = results_of(&ctx, sitting.pid).await?;
        let results: Vec<assessment_results::Model> = if masked {
            rows.into_iter().map(mask_result).collect()
        } else {
            rows
        };
        view.push(serde_json::json!({
            "pid": sitting.pid,
            "instrument": instrument.name,
            "category": instrument.category,
            "status": sitting.status,
            "scheduled_on": sitting.scheduled_on,
            "completed_on": sitting.completed_on,
            "results": results,
        }));
    }
    let outstanding = sittings
        .iter()
        .filter(|s| matches!(s.status.as_str(), "scheduled" | "in_progress"))
        .count();
    format::json(serde_json::json!({
        "application": { "pid": application.pid, "stage": application.stage },
        "candidate_pid": application.candidate_pid,
        "masked": masked,
        "outstanding": outstanding,
        "assessments": view,
    }))
}

/// `GET /api/assessments/analytics` — programme-level rollups: sittings
/// by category and status, completion counts, and the band distribution
/// per scale.
///
/// Aggregate counts only — no individual's score appears here, so it is
/// safe to serve without the per-record mask (a band distribution over
/// the whole population identifies no one).
#[debug_handler]
async fn analytics(State(ctx): State<AppContext>) -> Result<Response> {
    let sittings = sqlx::query!(
        "SELECT instrument_pid, status FROM assessments WHERE deleted_at IS NULL ORDER BY id"
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let instruments = all_instruments(&ctx).await?;
    let category_of: BTreeMap<Uuid, &str> = instruments
        .iter()
        .map(|i| (i.pid, i.category.as_str()))
        .collect();

    // category → status → count
    let mut by_category: BTreeMap<&str, BTreeMap<String, usize>> = BTreeMap::new();
    for sitting in &sittings {
        let Some(category) = category_of.get(&sitting.instrument_pid) else {
            continue;
        };
        *by_category
            .entry(category)
            .or_default()
            .entry(sitting.status.clone())
            .or_default() += 1;
    }

    // scale → band → count, over every recorded result.
    let results = sqlx::query!(
        r#"SELECT scale, band AS "band!" FROM assessment_results WHERE band IS NOT NULL ORDER BY id"#
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let mut bands: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
    for row in results {
        *bands
            .entry(row.scale)
            .or_default()
            .entry(row.band)
            .or_default() += 1;
    }

    let categories: Vec<serde_json::Value> = by_category
        .iter()
        .map(|(category, by_status)| {
            let completed = by_status.get("completed").copied().unwrap_or(0);
            let total: usize = by_status.values().sum();
            serde_json::json!({
                "category": category,
                "by_status": by_status,
                "completed": completed,
                "total": total,
            })
        })
        .collect();
    format::json(serde_json::json!({
        "as_of": chrono::Utc::now(),
        "note": "counts over live sittings; the band distribution is aggregate only — \
                 no individual score is reported here",
        "categories": categories,
        "band_distribution": bands,
    }))
}

// ─── Helpers ────────────────────────────────────────────────────────────────

/// Apply the `mask` obligation to a result: the scale and the
/// interpreted band survive; the raw score, its maximum, the
/// percentile, and the narrative are redacted. A masked caller learns
/// *that* a dimension was measured and roughly where it landed, never
/// the profile itself.
#[must_use]
fn mask_result(mut result: assessment_results::Model) -> assessment_results::Model {
    result.raw_score = None;
    result.max_score = None;
    result.percentile = None;
    result.narrative = None;
    result
}

/// Resource attributes for an assessment, for the `resource.*` policy
/// namespace: the instrument's category, the sitting's status, and the
/// subject kind. A deployment can then write e.g. "allow an unmasked
/// read of `resource.category=selection` only to recruiters, and a
/// masked read otherwise" — entirely as policy, no code change.
#[must_use]
fn assessment_resource_attrs(
    assessment: &assessments::Model,
    instrument: &assessment_instruments::Model,
) -> BTreeMap<String, Vec<String>> {
    BTreeMap::from([
        ("record".to_string(), vec!["assessment".to_string()]),
        ("category".to_string(), vec![instrument.category.clone()]),
        ("status".to_string(), vec![assessment.status.clone()]),
        (
            "subject_kind".to_string(),
            vec![assessment.subject_kind.clone()],
        ),
    ])
}

/// The scales an instrument declares, when it declares any (an empty
/// or absent list means "any scale its category permits").
fn declared_scales(instrument: &assessment_instruments::Model) -> Option<Vec<String>> {
    let declared: Vec<String> =
        serde_json::from_value(instrument.scales.clone()).unwrap_or_default();
    if declared.is_empty() {
        None
    } else {
        Some(declared)
    }
}

/// Resolve the subject (candidate or worker) and return its display
/// name, `404`-ing when it does not exist — so a sitting can never be
/// booked against a subject that is not there. A worker subject is
/// additionally authorized at the worker level.
async fn subject_display_name(
    ctx: &AppContext,
    subject_kind: &str,
    subject_pid: Uuid,
    caller: &MaybeAuthUser,
) -> Result<String> {
    if subject_kind == "worker" {
        let worker = records::worker_by_pid(ctx, subject_pid).await?;
        auth::authorize_record(
            caller,
            authentication_verifier::Action::Write,
            &auth::worker_resource_attrs(&worker),
        )
        .map_err(record_rejection)?;
        return Ok(worker.display_name);
    }
    Ok(live_candidate(ctx, subject_pid).await?.display_name)
}

/// What the assessment views read of a live candidate.
struct Candidate {
    pid: Uuid,
    display_name: String,
}

/// The live candidate with this pid, else `404`.
async fn live_candidate(ctx: &AppContext, pid: Uuid) -> Result<Candidate> {
    sqlx::query_as!(
        Candidate,
        "SELECT pid, display_name FROM candidates WHERE pid = $1 AND deleted_at IS NULL",
        pid,
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)
}

/// The assessment routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/assessment-instruments", post(create_instrument))
        .add("/assessment-instruments", get(list_instruments))
        .add("/assessments", post(create_assessment))
        .add("/assessments/analytics", get(analytics))
        .add("/assessments/{pid}", get(get_assessment))
        .add("/assessments/{pid}", delete(delete_assessment))
        .add("/assessments/{pid}/status", post(assessment_status))
        .add("/assessments/{pid}/results", post(record_result))
        .add(
            "/applications/{pid}/assessments",
            get(application_assessments),
        )
        .add(
            "/candidates/{pid}/assessment-profile",
            get(candidate_profile),
        )
        .add("/workers/{pid}/assessment-profile", get(worker_profile))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A stored result, as the DB would return it.
    fn a_result(scale: &str, percentile: Option<i32>) -> assessment_results::Model {
        assessment_results::Model {
            created_at: chrono::Utc::now().into(),
            updated_at: chrono::Utc::now().into(),
            id: 1,
            pid: Uuid::new_v4(),
            assessment_pid: Uuid::new_v4(),
            scale: scale.to_string(),
            raw_score: Some(41),
            max_score: Some(50),
            percentile,
            band: percentile.map(|p| rules::band_for_percentile(p).to_string()),
            narrative: Some("strong under time pressure".to_string()),
        }
    }

    /// An instrument row declaring `scales`.
    fn an_instrument(category: &str, scales: &[&str]) -> assessment_instruments::Model {
        assessment_instruments::Model {
            created_at: chrono::Utc::now().into(),
            updated_at: chrono::Utc::now().into(),
            id: 1,
            pid: Uuid::new_v4(),
            name: "Test".to_string(),
            category: category.to_string(),
            provider: None,
            scales: serde_json::json!(scales),
            duration_minutes: None,
            validity_months: None,
            deleted_at: None,
        }
    }

    /// Masking keeps the scale and band, drops every score and the
    /// narrative (the sensitive-read invariant).
    #[test]
    fn mask_keeps_the_band_and_drops_the_profile() {
        let masked = mask_result(a_result("judgement_test", Some(88)));
        assert_eq!(masked.scale, "judgement_test");
        assert_eq!(masked.band.as_deref(), Some("above_average"));
        assert!(masked.raw_score.is_none() && masked.max_score.is_none());
        assert!(masked.percentile.is_none(), "the percentile is redacted");
        assert!(masked.narrative.is_none(), "the narrative is redacted");
    }

    /// An instrument's declared scales gate which results it accepts;
    /// an empty declaration means "any its category permits".
    #[test]
    fn declared_scales_gate_results() {
        let declared = an_instrument("aptitude", &["numerical_reasoning"]);
        let scales = declared_scales(&declared).expect("declared");
        assert_eq!(scales, vec!["numerical_reasoning".to_string()]);
        assert!(!scales.iter().any(|s| s == "verbal_reasoning"));

        let open = an_instrument("aptitude", &[]);
        assert!(
            declared_scales(&open).is_none(),
            "no declaration ⇒ any permitted scale"
        );
    }

    /// The resource attributes a policy can key on.
    #[test]
    fn resource_attrs_expose_category_status_and_subject() {
        let instrument = an_instrument("selection", &["job_simulation"]);
        let sitting = assessments::Model {
            created_at: chrono::Utc::now().into(),
            updated_at: chrono::Utc::now().into(),
            id: 1,
            pid: Uuid::new_v4(),
            instrument_pid: instrument.pid,
            subject_kind: "candidate".to_string(),
            subject_pid: Uuid::new_v4(),
            application_pid: None,
            status: "completed".to_string(),
            scheduled_on: None,
            completed_on: None,
            expires_on: None,
            administered_by: None,
            notes: None,
            deleted_at: None,
        };
        let attrs = assessment_resource_attrs(&sitting, &instrument);
        assert_eq!(attrs["category"], vec!["selection".to_string()]);
        assert_eq!(attrs["status"], vec!["completed".to_string()]);
        assert_eq!(attrs["subject_kind"], vec!["candidate".to_string()]);
    }

    /// Expiry arithmetic adds the months and refuses to panic on a
    /// negative or absurd validity.
    #[test]
    fn expiry_is_derived_and_never_panics() {
        let day = |y, m, d| chrono::NaiveDate::from_ymd_opt(y, m, d).expect("valid date");
        assert_eq!(expiry_from(day(2026, 7, 23), 12), Some(day(2027, 7, 23)));
        assert_eq!(expiry_from(day(2026, 1, 31), 1), Some(day(2026, 2, 28)));
        assert_eq!(expiry_from(day(2026, 7, 23), -1), None, "negative months");
        assert_eq!(
            expiry_from(day(2026, 7, 23), i32::MAX),
            None,
            "overflowing the calendar yields None, not a panic"
        );
    }
}
