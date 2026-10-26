//! The AI and scoring deployer register (WPM-R148, WPM-D103): what a deployer recorded, per scoring feature, under Article 26
//! of the AI Act. Any signed-in caller reads it (it is a notice of how people are scored); HR and administrators write it.
//! It records; it does not classify a system, certify conformity or count as evidence of compliance. Spec: `spec/regulatory.md`.

use chrono::{NaiveDate, Utc};
use loco_rs::prelude::*;
use sea_orm::{ActiveValue, TransactionTrait};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use super::unprocessable;
use crate::auth::MaybeAuthUser;
use crate::models::_entities::ai_register_entries;
use crate::models::audit_logs::Model as Audit;
use crate::rules::ai_register::{self as rules, Entry};

fn entry_of(m: &ai_register_entries::Model) -> Entry {
    Entry {
        annex_category: m.annex_category.clone(),
        overseer_role: m.overseer_role.clone(),
        representatives_informed_on: m.representatives_informed_on,
        workers_informed_on: m.workers_informed_on,
        log_retention_months: m.log_retention_months,
        retention_note: m.retention_note.clone(),
    }
}

/// The date the Annex III duties apply: `WPM_AI_ACT_APPLIES_FROM` (YYYY-MM-DD), else the default.
fn applies_from() -> NaiveDate {
    crate::compat::env_var("WPM_AI_ACT_APPLIES_FROM")
        .and_then(|v| NaiveDate::parse_from_str(v.trim(), "%Y-%m-%d").ok())
        .unwrap_or_else(rules::default_applies_from)
}

/// `GET /api/ai-register` — every scoring feature with what is recorded for it; what is not recorded is `unknown`.
#[debug_handler]
async fn register(State(ctx): State<AppContext>) -> Result<Response> {
    let held = ai_register_entries::Entity::find().all(&ctx.db).await?;
    let rows: Vec<rules::Row> = rules::FEATURES
        .iter()
        .map(|f| {
            let entry = held.iter().find(|m| m.feature_id == f.id).map(entry_of);
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
    let actor = caller.actor().map(ToString::to_string);
    let txn = ctx.db.begin().await?;
    let existing = ai_register_entries::Entity::find()
        .filter(ai_register_entries::Column::FeatureId.eq(feature.id))
        .one(&txn)
        .await?;
    let pid = if let Some(row) = existing {
        let pid = row.pid;
        let mut active: ai_register_entries::ActiveModel = row.into();
        active.annex_category = ActiveValue::set(entry.annex_category.clone());
        active.overseer_role = ActiveValue::set(entry.overseer_role.clone());
        active.representatives_informed_on = ActiveValue::set(entry.representatives_informed_on);
        active.workers_informed_on = ActiveValue::set(entry.workers_informed_on);
        active.log_retention_months = ActiveValue::set(entry.log_retention_months);
        active.retention_note = ActiveValue::set(entry.retention_note.clone());
        active.recorded_by = ActiveValue::set(actor.clone());
        active.update(&txn).await?;
        pid
    } else {
        let pid = Uuid::new_v4();
        ai_register_entries::ActiveModel {
            pid: ActiveValue::set(pid),
            feature_id: ActiveValue::set(feature.id.to_string()),
            annex_category: ActiveValue::set(entry.annex_category.clone()),
            overseer_role: ActiveValue::set(entry.overseer_role.clone()),
            representatives_informed_on: ActiveValue::set(entry.representatives_informed_on),
            workers_informed_on: ActiveValue::set(entry.workers_informed_on),
            log_retention_months: ActiveValue::set(entry.log_retention_months),
            retention_note: ActiveValue::set(entry.retention_note.clone()),
            recorded_by: ActiveValue::set(actor.clone()),
            ..Default::default()
        }
        .insert(&txn)
        .await?;
        pid
    };
    Audit::record(
        &txn,
        "ai_register",
        pid,
        "ai_register_recorded",
        caller.actor(),
        Some(json!({ "feature": feature.id, "consultation_pid": payload.consultation_pid })),
    )
    .await?;
    txn.commit().await?;
    format::json(json!({ "pid": pid, "feature": feature.id, "decision_gate": gate }))
}

/// The routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/ai-register", get(register))
        .add("/ai-register/{feature}", put(record))
}
