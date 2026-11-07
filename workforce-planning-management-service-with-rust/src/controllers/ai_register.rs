//! The AI and scoring deployer register (WPM-R148, WPM-D103): what a deployer recorded, per scoring feature, under Article 26
//! of the AI Act. Any signed-in caller reads it (it is a notice of how people are scored); HR and administrators write it.
//! It records; it does not classify a system, certify conformity or count as evidence of compliance. Spec: `spec/regulatory.md`.

use axa::extract::{Json, Path};
use axa::prelude::*;
use chrono::{NaiveDate, Utc};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use super::unprocessable;
use crate::auth::MaybeAuthUser;
use crate::db::{self, pool};
use crate::rules::ai_register::{self as rules, Entry};

/// The date the Annex III duties apply: `WPM_AI_ACT_APPLIES_FROM` (YYYY-MM-DD), else the default.
pub(crate) fn applies_from() -> NaiveDate {
    crate::compat::env_var("WPM_AI_ACT_APPLIES_FROM")
        .and_then(|v| NaiveDate::parse_from_str(v.trim(), "%Y-%m-%d").ok())
        .unwrap_or_else(rules::default_applies_from)
}

/// `GET /api/ai-register` — every scoring feature with what is recorded for it; what is not recorded is `unknown`.
#[debug_handler]
async fn register(State(ctx): State<AppContext>) -> Result<Response> {
    let held = sqlx::query!(
        "SELECT feature_id, annex_category, overseer_role, representatives_informed_on, workers_informed_on,
                log_retention_months, retention_note
         FROM ai_register_entries"
    )
    .fetch_all(pool(&ctx))
    .await?;
    let rows: Vec<rules::Row> = rules::FEATURES
        .iter()
        .map(|f| {
            let entry = held.iter().find(|m| m.feature_id == f.id).map(|m| Entry {
                annex_category: m.annex_category.clone(),
                overseer_role: m.overseer_role.clone(),
                representatives_informed_on: m.representatives_informed_on,
                workers_informed_on: m.workers_informed_on,
                log_retention_months: m.log_retention_months,
                retention_note: m.retention_note.clone(),
            });
            rules::row(f, entry.as_ref())
        })
        .collect();
    let from = applies_from();
    format::json(json!({
        "applies_from": from,
        "applies_now": Utc::now().date_naive() >= from,
        "features": rows,
        "categories": rules::CATEGORIES,
        "minimum_log_months": rules::MINIMUM_LOG_MONTHS,
        "note": "This register records what the deployer says. WPM does not classify a system as high-risk, does not certify \
                 conformity, and a filled-in row is not evidence of compliance. A field with nothing recorded is unknown.",
    }))
}

#[derive(Debug, Deserialize)]
struct Payload {
    annex_category: Option<String>,
    overseer_role: Option<String>,
    representatives_informed_on: Option<NaiveDate>,
    workers_informed_on: Option<NaiveDate>,
    log_retention_months: Option<i32>,
    retention_note: Option<String>,
    /// A consultation record for this change, when a decision gate asks for one (WPM-R108).
    consultation_pid: Option<Uuid>,
}

/// `PUT /api/ai-register/{feature}` — replace what is recorded for a feature (`S1` to `S8`). A field left out is cleared.
#[debug_handler]
async fn record(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(feature_id): Path<String>,
    Json(payload): Json<Payload>,
) -> Result<Response> {
    let feature = rules::feature(&feature_id).ok_or(Error::NotFound)?;
    let clean = |t: Option<String>| t.map(|t| t.trim().to_string()).filter(|t| !t.is_empty());
    let entry = Entry {
        annex_category: clean(payload.annex_category),
        overseer_role: clean(payload.overseer_role),
        representatives_informed_on: payload.representatives_informed_on,
        workers_informed_on: payload.workers_informed_on,
        log_retention_months: payload.log_retention_months,
        retention_note: clean(payload.retention_note),
    };
    let problems = rules::validate(&entry, Utc::now().date_naive());
    if !problems.is_empty() {
        return Err(unprocessable(&problems.join("; ")));
    }
    // A new or changed score may need consultation (WPM-R108): point at the gate and ask for the reference; it advises, it does not block.
    let gate = super::decision_gates::pointer(
        &ctx,
        None,
        "new_or_changed_score",
        payload.consultation_pid,
    )
    .await?;
    let mut tx = pool(&ctx).begin().await?;
    let existing = sqlx::query_scalar!(
        "SELECT pid FROM ai_register_entries WHERE feature_id = $1",
        feature.id,
    )
    .fetch_optional(&mut *tx)
    .await?;
    let pid = if let Some(pid) = existing {
        sqlx::query!(
            "UPDATE ai_register_entries
             SET annex_category = $2, overseer_role = $3, representatives_informed_on = $4, workers_informed_on = $5,
                 log_retention_months = $6, retention_note = $7, recorded_by = $8
             WHERE pid = $1",
            pid,
            entry.annex_category,
            entry.overseer_role,
            entry.representatives_informed_on,
            entry.workers_informed_on,
            entry.log_retention_months,
            entry.retention_note,
            caller.actor(),
        )
        .execute(&mut *tx)
        .await?;
        pid
    } else {
        let pid = Uuid::new_v4();
        sqlx::query!(
            "INSERT INTO ai_register_entries
                 (pid, feature_id, annex_category, overseer_role, representatives_informed_on, workers_informed_on,
                  log_retention_months, retention_note, recorded_by)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            pid,
            feature.id,
            entry.annex_category,
            entry.overseer_role,
            entry.representatives_informed_on,
            entry.workers_informed_on,
            entry.log_retention_months,
            entry.retention_note,
            caller.actor(),
        )
        .execute(&mut *tx)
        .await?;
        pid
    };
    db::audit(
        &mut tx,
        "ai_register",
        pid,
        "ai_register_recorded",
        caller.actor(),
        Some(json!({ "feature": feature.id, "consultation_pid": payload.consultation_pid })),
    )
    .await?;
    tx.commit().await?;
    format::json(json!({ "pid": pid, "feature": feature.id, "decision_gate": gate }))
}

/// The routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/ai-register", get(register))
        .add("/ai-register/{feature}", put(record))
}
