//! Reasonable adjustments (WPM-R33 / WPM-D25) — barrier-based
//! requests ("what gets in the way, how it affects the work, what
//! change would reduce it"), a decision lifecycle with in-app
//! notification, and content-tier masking: a masked read keeps
//! category + status and withholds the words. No diagnosis is ever
//! required, and the schema has nowhere to put one.

use super::row_json;
use axa::extract::{Json, Path};
use axa::prelude::*;
use serde::Deserialize;

use super::{ensure_valid, record_rejection, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::db::{self, pool};
use crate::models::_entities::adjustment_requests;
use crate::models::records;
use crate::rules::adjustments as rules;
use crate::validation::Problems;

/// A `{pid}` reference response.
#[derive(serde::Serialize)]
struct PidRef {
    pid: String,
}

/// `POST /api/workers/{pid}/adjustment-requests` body — the useful
/// bit and nothing else: barrier, impact, change.
#[derive(Debug, Deserialize)]
struct RequestPayload {
    category: String,
    barrier: String,
    impact: String,
    adjustment: String,
}

/// `POST /api/workers/{pid}/adjustment-requests` — put it in
/// writing. All three texts required; `$sub` ownership applies (HR
/// may file on behalf per policy). Audited.
#[debug_handler]
async fn create_request(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<RequestPayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_token("category", rules::ADJUSTMENT_CATEGORIES, &payload.category);
    problems.require_text("barrier", &payload.barrier);
    problems.cap_text("barrier", &payload.barrier);
    problems.require_text("impact", &payload.impact);
    problems.cap_text("impact", &payload.impact);
    problems.require_text("adjustment", &payload.adjustment);
    problems.cap_text("adjustment", &payload.adjustment);
    ensure_valid(&problems.into_vec())?;
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    auth::authorize_record(
        &caller,
        authentication_verifier::Action::Write,
        &auth::worker_resource_attrs(&worker),
    )
    .map_err(record_rejection)?;
    let mut tx = pool(&ctx).begin().await?;
    let row_pid = uuid::Uuid::new_v4();
    sqlx::query!(
        "INSERT INTO adjustment_requests (pid, worker_pid, category, barrier, impact, adjustment, status)
         VALUES ($1, $2, $3, $4, $5, $6, 'requested')",
        row_pid,
        worker.pid,
        payload.category,
        payload.barrier,
        payload.impact,
        payload.adjustment,
    )
    .execute(&mut *tx)
    .await?;
    // The audit row records that a request was made — not its words.
    db::audit(
        &mut tx,
        "adjustment_request",
        row_pid,
        "created",
        caller.actor(),
        Some(serde_json::json!({ "category": payload.category })),
    )
    .await?;
    tx.commit().await?;
    format::json(PidRef {
        pid: row_pid.to_string(),
    })
}

/// `GET /api/workers/{pid}/adjustment-requests` — the worker's
/// requests, newest first. `$sub`-owned; content-tier: a masked read
/// keeps category + status and **withholds the words**; unmasked
/// reads are audited.
#[debug_handler]
async fn list_requests(
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
    let rows = sqlx::query!(
        "SELECT pid, category, status, barrier, impact, adjustment, decision_note, decided_on
         FROM adjustment_requests WHERE worker_pid = $1 AND deleted_at IS NULL ORDER BY id DESC",
        worker.pid,
    )
    .fetch_all(pool(&ctx))
    .await?;
    if !masked && !rows.is_empty() {
        db::audit(
            &mut *pool(&ctx).acquire().await?,
            "adjustment_request",
            worker.pid,
            "adjustments_read",
            caller.actor(),
            None,
        )
        .await?;
    }
    let view: Vec<serde_json::Value> = rows
        .iter()
        .map(|row| {
            if masked {
                serde_json::json!({
                    "pid": row.pid, "category": row.category, "status": row.status,
                    "decided_on": row.decided_on, "words_withheld": true,
                })
            } else {
                serde_json::json!({
                    "pid": row.pid, "category": row.category, "status": row.status,
                    "barrier": row.barrier, "impact": row.impact,
                    "adjustment": row.adjustment,
                    "decision_note": row.decision_note, "decided_on": row.decided_on,
                    "words_withheld": false,
                })
            }
        })
        .collect();
    format::json(view)
}

/// `POST /api/adjustment-requests/{pid}/status` body.
#[derive(Debug, Deserialize)]
struct DecisionPayload {
    to: String,
    #[serde(default)]
    note: Option<String>,
}

/// `POST /api/adjustment-requests/{pid}/status` — decide (pure
/// machine); stamps the date, records the practical note, audits, and
/// notifies the worker in-app.
#[debug_handler]
async fn decide(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<DecisionPayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.cap_opt("note", payload.note.as_deref());
    ensure_valid(&problems.into_vec())?;
    let row = sqlx::query!(
        "SELECT pid, worker_pid, category, status FROM adjustment_requests WHERE pid = $1 AND deleted_at IS NULL",
        records::parse_pid(&pid)?,
    )
    .fetch_optional(pool(&ctx))
    .await?
    .ok_or(Error::NotFound)?;
    rules::transition(&row.status, &payload.to).map_err(|reason| unprocessable(&reason))?;
    let mut tx = pool(&ctx).begin().await?;
    let updated = sqlx::query_as!(
        adjustment_requests::Model,
        r#"UPDATE adjustment_requests
           SET status = $2, decided_on = $3, decision_note = COALESCE($4, decision_note)
           WHERE pid = $1
           RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, category, barrier, impact,
                     adjustment, status, decision_note, decided_on, deleted_at AS "deleted_at: _""#,
        row.pid,
        payload.to,
        chrono::Utc::now().date_naive(),
        payload.note,
    )
    .fetch_one(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "adjustment_request",
        row.pid,
        "status_changed",
        caller.actor(),
        Some(serde_json::json!({ "from": row.status, "to": payload.to })),
    )
    .await?;
    // Tell the worker — the body names the category and the state,
    // never the words (WPM-D23 reference-only posture).
    db::notify(
        &mut tx,
        row.worker_pid,
        "adjustment_update",
        &format!("Adjustment request ({}): {}", row.category, payload.to),
        serde_json::json!({ "adjustment_request_pid": row.pid }),
    )
    .await?;
    tx.commit().await?;
    format::json(row_json(&updated)?)
}

/// The adjustment routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/workers/{pid}/adjustment-requests", post(create_request))
        .add("/workers/{pid}/adjustment-requests", get(list_requests))
        .add("/adjustment-requests/{pid}/status", post(decide))
}
