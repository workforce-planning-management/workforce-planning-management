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

use loco_rs::prelude::*;
use sea_orm::{ActiveValue, TransactionTrait};
use serde::Deserialize;
use std::str::FromStr;
use uuid::Uuid;

use super::unprocessable;
use crate::auth::MaybeAuthUser;
use crate::models::_entities::{cpd_entries, training_enrollments, workers};
use crate::models::audit_logs::Model as Audit;
use crate::rules::lms as rules;

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

/// Apply one event in its own transaction.
#[allow(clippy::too_many_lines)] // resolve learner → enrollment → CPD credit, linearly
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
    let mut query = workers::Entity::find()
        .filter(workers::Column::DeletedAt.is_null())
        .filter(workers::Column::PersonRef.eq(&event.person_ref));
    if let Some(org) = &event.organization_ref {
        query = query.filter(workers::Column::OrganizationRef.eq(org));
    }
    let found = query.all(&ctx.db).await?;
    let worker = match found.as_slice() {
        [] => {
            return Ok(Outcome::rejected(
                reference,
                "unmatched",
                "no worker for person_ref",
            ));
        }
        [only] => only,
        _ => {
            return Ok(Outcome::rejected(
                reference,
                "ambiguous",
                "person_ref has workers in several organizations; give organization_ref",
            ));
        }
    };

    let txn = ctx.db.begin().await?;
    // Enrollment.
    let existing = training_enrollments::Entity::find()
        .filter(training_enrollments::Column::WorkerPid.eq(worker.pid))
        .filter(training_enrollments::Column::CourseRef.eq(&event.course_ref))
        .filter(training_enrollments::Column::DeletedAt.is_null())
        .one(&txn)
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
    let enrollment = match (action, existing) {
        (rules::EnrollmentAction::Unchanged, _) => "unchanged",
        (rules::EnrollmentAction::Complete, Some(row)) => {
            let mut active: training_enrollments::ActiveModel = row.into();
            active.status = ActiveValue::set("completed".to_string());
            active.completed_on = ActiveValue::set(Some(event.completed_on));
            active.certificate_expires_on = ActiveValue::set(event.certificate_expires_on);
            let row = active.update(&txn).await?;
            Audit::record(
                &txn,
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
            let row = training_enrollments::ActiveModel {
                pid: ActiveValue::set(Uuid::new_v4()),
                worker_pid: ActiveValue::set(worker.pid),
                course_ref: ActiveValue::set(event.course_ref.clone()),
                status: ActiveValue::set("completed".to_string()),
                completed_on: ActiveValue::set(Some(event.completed_on)),
                certificate_expires_on: ActiveValue::set(event.certificate_expires_on),
                deleted_at: ActiveValue::set(None),
                ..Default::default()
            }
            .insert(&txn)
            .await?;
            Audit::record(
                &txn,
                "training_enrollment",
                row.pid,
                "lms_created",
                caller.actor(),
                None,
            )
            .await?;
            "created"
        }
    };

    // CPD credit, idempotent on (worker, external_ref).
    let cpd = if let Some((unit, hundredths)) = credit {
        let already = cpd_entries::Entity::find()
            .filter(cpd_entries::Column::WorkerPid.eq(worker.pid))
            .filter(cpd_entries::Column::ExternalRef.eq(reference))
            .filter(cpd_entries::Column::DeletedAt.is_null())
            .one(&txn)
            .await?
            .is_some();
        if already {
            "already_recorded"
        } else {
            let row = cpd_entries::ActiveModel {
                pid: ActiveValue::set(Uuid::new_v4()),
                worker_pid: ActiveValue::set(worker.pid),
                entry_date: ActiveValue::set(event.completed_on),
                activity: ActiveValue::set(format!("LMS completion: {}", event.course_ref)),
                category: ActiveValue::set("course".to_string()),
                unit: ActiveValue::set(unit.to_string()),
                amount_hundredths: ActiveValue::set(hundredths),
                evidence_note: ActiveValue::set(None),
                evidence_url: ActiveValue::set(None),
                source: ActiveValue::set("lms".to_string()),
                external_ref: ActiveValue::set(Some(reference.to_string())),
                verified_on: ActiveValue::set(None),
                verified_by: ActiveValue::set(None),
                deleted_at: ActiveValue::set(None),
                ..Default::default()
            }
            .insert(&txn)
            .await?;
            Audit::record(
                &txn,
                "cpd_entry",
                row.pid,
                "lms_recorded",
                caller.actor(),
                None,
            )
            .await?;
            "recorded"
        }
    } else {
        "none"
    };
    txn.commit().await?;

    let unchanged = enrollment == "unchanged" && matches!(cpd, "none" | "already_recorded");
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
