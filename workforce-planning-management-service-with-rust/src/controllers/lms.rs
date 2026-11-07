//! **LMS completion sync** (WPM-T54): the learning-management system owns
//! courses and learners (WPM-D10) and pushes completions here. One batch
//! endpoint, idempotent per event (`external_ref`), reporting a per-item
//! outcome so a bad event never poisons the rest.
//!
//! A completion updates (or creates) the worker's **training enrollment**
//! for the course — which already feeds certificate-expiry reporting and
//! learning-path progress — and, when it carries hours or points, lands a
//! **CPD entry** with `source = "lms"`. Pure rules: [`crate::rules::lms`].
//!
//! Intended caller: the LMS's service account (the `svc` ABAC attribute);
//! the blanket write guard applies like any other mutation.

use axa::extract::Json;
use axa::prelude::*;
use serde::Deserialize;
use std::str::FromStr;
use uuid::Uuid;

use super::unprocessable;
use crate::auth::MaybeAuthUser;
use crate::db::{self, pool};
use crate::rules::lms as rules;

/// The link to a course in the learning system, from the deployer's `WPM_LMS_COURSE_URL` (WPM-R138,
/// WPM-D91); `None` when it is unset or not valid. An invalid one is logged, once.
#[must_use]
pub fn course_url(course_ref: &str) -> Option<String> {
    static WARNED: std::sync::Once = std::sync::Once::new();
    let raw = crate::compat::env_var("WPM_LMS_COURSE_URL")?;
    match rules::parse_template(&raw) {
        Ok(template) => rules::course_url(&template, course_ref),
        Err(why) => {
            WARNED.call_once(|| tracing::warn!("WPM_LMS_COURSE_URL is ignored: {why}"));
            None
        }
    }
}

/// One completion event from the LMS.
#[derive(Debug, Deserialize)]
struct CompletionEvent {
    /// The learner, as the identity service names them (`person:` URN).
    person_ref: String,
    /// Disambiguates a person with workers in several organizations.
    #[serde(default)]
    organization_ref: Option<String>,
    /// The `course:` or `courseinstance:` URN.
    course_ref: String,
    completed_on: chrono::NaiveDate,
    #[serde(default)]
    certificate_expires_on: Option<chrono::NaiveDate>,
    /// CPD credit, in hours — or points; not both.
    #[serde(default)]
    hours: Option<f64>,
    #[serde(default)]
    points: Option<f64>,
    /// The LMS's own id for this event: the idempotency key.
    external_ref: String,
}

/// `POST /api/lms/completions` body.
#[derive(Debug, Deserialize)]
struct CompletionBatch {
    completions: Vec<CompletionEvent>,
}

/// What happened to one event.
#[derive(serde::Serialize)]
struct Outcome {
    external_ref: String,
    /// `applied`, `unchanged`, `unmatched`, `ambiguous`, or `invalid`.
    result: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    enrollment: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cpd: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
}

impl Outcome {
    fn rejected(external_ref: &str, outcome: &'static str, reason: impl Into<String>) -> Self {
        Self {
            external_ref: external_ref.to_string(),
            result: outcome,
            enrollment: None,
            cpd: None,
            reason: Some(reason.into()),
        }
    }
}

/// `POST /api/lms/completions` — apply a batch of completions.
#[debug_handler]
async fn completions(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(batch): Json<CompletionBatch>,
) -> Result<Response> {
    if batch.completions.len() > rules::MAX_BATCH {
        return Err(unprocessable(&format!(
            "at most {} completions per batch",
            rules::MAX_BATCH
        )));
    }
    let today = chrono::Utc::now().date_naive();
    let mut results = Vec::with_capacity(batch.completions.len());
    for event in &batch.completions {
        results.push(apply(&ctx, &caller, event, today).await?);
    }
    let count = |name: &str| results.iter().filter(|r| r.result == name).count();
    format::json(serde_json::json!({
        "summary": {
            "received": results.len(),
            "applied": count("applied"),
            "unchanged": count("unchanged"),
            "unmatched": count("unmatched"),
            "ambiguous": count("ambiguous"),
            "invalid": count("invalid"),
        },
        "results": results,
    }))
}

/// Complete or create the worker's enrollment for the event's course, auditing a change.
async fn sync_enrollment(
    connection: &mut sqlx::PgConnection,
    caller: &MaybeAuthUser,
    worker: Uuid,
    event: &CompletionEvent,
) -> Result<&'static str> {
    let existing = sqlx::query!(
        "SELECT pid, status, completed_on, certificate_expires_on
         FROM training_enrollments
         WHERE worker_pid = $1 AND course_ref = $2 AND deleted_at IS NULL
         LIMIT 1",
        worker,
        event.course_ref,
    )
    .fetch_optional(&mut *connection)
    .await?;
    let snapshot = existing.as_ref().map(|e| rules::Existing {
        status: e.status.as_str(),
        completed_on: e.completed_on,
        certificate_expires_on: e.certificate_expires_on,
    });
    let action = rules::enrollment_action(
        snapshot.as_ref(),
        event.completed_on,
        event.certificate_expires_on,
    );
    Ok(match (action, existing) {
        (rules::EnrollmentAction::Unchanged, _) => "unchanged",
        (rules::EnrollmentAction::Complete, Some(row)) => {
            sqlx::query!(
                "UPDATE training_enrollments
                 SET status = 'completed', completed_on = $2, certificate_expires_on = $3
                 WHERE pid = $1",
                row.pid,
                event.completed_on,
                event.certificate_expires_on,
            )
            .execute(&mut *connection)
            .await?;
            db::audit(
                connection,
                "training_enrollment",
                row.pid,
                "lms_completed",
                caller.actor(),
                None,
            )
            .await?;
            "updated"
        }
        (_, _) => {
            let pid = Uuid::new_v4();
            sqlx::query!(
                "INSERT INTO training_enrollments
                     (pid, worker_pid, course_ref, status, completed_on, certificate_expires_on)
                 VALUES ($1, $2, $3, 'completed', $4, $5)",
                pid,
                worker,
                event.course_ref,
                event.completed_on,
                event.certificate_expires_on,
            )
            .execute(&mut *connection)
            .await?;
            db::audit(
                connection,
                "training_enrollment",
                pid,
                "lms_created",
                caller.actor(),
                None,
            )
            .await?;
            "created"
        }
    })
}

/// Record the CPD credit an event carries, once per `(worker, external_ref)`.
async fn record_credit(
    connection: &mut sqlx::PgConnection,
    caller: &MaybeAuthUser,
    worker: Uuid,
    event: &CompletionEvent,
    credit: Option<(&'static str, i64)>,
) -> Result<&'static str> {
    let reference = event.external_ref.trim();
    let Some((unit, hundredths)) = credit else {
        return Ok("none");
    };
    let already = sqlx::query_scalar!(
        r#"SELECT EXISTS (
               SELECT 1 FROM cpd_entries
               WHERE worker_pid = $1 AND external_ref = $2 AND deleted_at IS NULL
           ) AS "already!""#,
        worker,
        reference,
    )
    .fetch_one(&mut *connection)
    .await?;
    if already {
        return Ok("already_recorded");
    }
    let pid = Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO cpd_entries
             (pid, worker_pid, entry_date, activity, category, unit, amount_hundredths,
              source, external_ref, course_ref)
         VALUES ($1, $2, $3, $4, 'course', $5, $6, 'lms', $7, $8)",
        pid,
        worker,
        event.completed_on,
        format!("LMS completion: {}", event.course_ref),
        unit,
        hundredths,
        reference,
        event.course_ref,
    )
    .execute(&mut *connection)
    .await?;
    db::audit(
        connection,
        "cpd_entry",
        pid,
        "lms_recorded",
        caller.actor(),
        None,
    )
    .await?;
    Ok("recorded")
}

/// Apply one event in its own transaction.
async fn apply(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    event: &CompletionEvent,
    today: chrono::NaiveDate,
) -> Result<Outcome> {
    let reference = event.external_ref.trim();
    if let Err(reason) = rules::validate_completion(
        &event.external_ref,
        event.completed_on,
        event.certificate_expires_on,
        today,
    ) {
        return Ok(Outcome::rejected(reference, "invalid", reason));
    }
    match entity_ref::EntityRef::from_str(&event.course_ref) {
        Ok(r)
            if r.entity_type == entity_ref::EntityType::Course
                || r.entity_type == entity_ref::EntityType::CourseInstance => {}
        _ => {
            return Ok(Outcome::rejected(
                reference,
                "invalid",
                "course_ref must be a course: or courseinstance: URN",
            ));
        }
    }
    let credit = match rules::cpd_credit(event.hours, event.points) {
        Ok(credit) => credit,
        Err(reason) => return Ok(Outcome::rejected(reference, "invalid", reason)),
    };

    // Resolve the learner to exactly one live worker.
    let found = sqlx::query_scalar!(
        "SELECT pid FROM workers
         WHERE deleted_at IS NULL
           AND person_ref = $1
           AND ($2::text IS NULL OR organization_ref = $2)",
        event.person_ref,
        event.organization_ref,
    )
    .fetch_all(pool(ctx))
    .await?;
    let worker = match found.as_slice() {
        [] => {
            return Ok(Outcome::rejected(
                reference,
                "unmatched",
                "no worker for person_ref",
            ));
        }
        [only] => *only,
        _ => {
            return Ok(Outcome::rejected(
                reference,
                "ambiguous",
                "person_ref has workers in several organizations; give organization_ref",
            ));
        }
    };

    let mut tx = pool(ctx).begin().await?;
    let enrollment = sync_enrollment(&mut tx, caller, worker, event).await?;
    let cpd = record_credit(&mut tx, caller, worker, event, credit).await?;
    tx.commit().await?;

    let unchanged = rules::is_unchanged(enrollment, cpd);
    Ok(Outcome {
        external_ref: reference.to_string(),
        result: if unchanged { "unchanged" } else { "applied" },
        enrollment: Some(enrollment),
        cpd: Some(cpd),
        reason: None,
    })
}

/// The LMS routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/lms/completions", post(completions))
}
