//! Ergonomic (DSE) workstation assessments (WPM-R32 / WPM-D24) — a
//! checklist per worker + workstation, answered item by item,
//! completed only when every item is answered; open issues surface in
//! a rota-tier department report. About the workstation, never the
//! body: no symptom field exists anywhere on this surface.

use super::row_json;
use axa::prelude::*;
use serde::Deserialize;
use uuid::Uuid;

use super::{ensure_valid, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::db::{self, pool};
use crate::models::_entities::{ergonomic_assessments, ergonomic_items};
use crate::models::records;
use crate::rules::ergonomics as rules;
use crate::validation::Problems;

/// A `{pid}` reference response.
#[derive(serde::Serialize)]
struct PidRef {
    pid: String,
}

/// `POST /api/workers/{pid}/ergonomic-assessments` body. Omitted
/// `items` ⇒ the default checklist, stored by key (WPM-T386); given
/// `items` are stored as typed, with no key.
#[derive(Debug, Deserialize)]
struct AssessmentPayload {
    workstation: String,
    #[serde(default)]
    items: Vec<String>,
}

/// `POST /api/workers/{pid}/ergonomic-assessments` — open an
/// assessment with its checklist (default DSE items when none given).
#[debug_handler]
async fn create_assessment(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<AssessmentPayload>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.require_text("workstation", &payload.workstation);
    problems.cap_text("workstation", &payload.workstation);
    problems.cap_list("items", &payload.items);
    ensure_valid(&problems.into_vec())?;
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    // (key, text): a default item is stored by its key, with its English text for an API reader
    let items: Vec<(Option<&str>, String)> = if payload.items.is_empty() {
        rules::DSE_ITEMS
            .iter()
            .map(|i| (Some(i.key), i.text.to_string()))
            .collect()
    } else {
        payload.items.iter().map(|n| (None, n.clone())).collect()
    };
    let assessment_pid = Uuid::new_v4();
    let mut tx = pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO ergonomic_assessments (pid, worker_pid, workstation, status)
         VALUES ($1, $2, $3, 'open')",
        assessment_pid,
        worker.pid,
        payload.workstation,
    )
    .execute(&mut *tx)
    .await?;
    for (key, name) in &items {
        sqlx::query!(
            "INSERT INTO ergonomic_items (pid, assessment_pid, name, item_key)
             VALUES ($1, $2, $3, $4)",
            Uuid::new_v4(),
            assessment_pid,
            name,
            *key,
        )
        .execute(&mut *tx)
        .await?;
    }
    db::audit(
        &mut tx,
        "ergonomic_assessment",
        assessment_pid,
        "created",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(PidRef {
        pid: assessment_pid.to_string(),
    })
}

/// `GET /api/workers/{pid}/ergonomic-assessments` — the worker's
/// assessments with their items.
#[debug_handler]
async fn list_assessments(
    State(ctx): State<AppContext>,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    let assessments = sqlx::query!(
        "SELECT pid, workstation, status, assessed_on FROM ergonomic_assessments
         WHERE worker_pid = $1 AND deleted_at IS NULL
         ORDER BY id DESC",
        worker.pid,
    )
    .fetch_all(pool(&ctx))
    .await?;
    let mut view = Vec::new();
    for assessment in &assessments {
        let items = sqlx::query_as!(
            ergonomic_items::Model,
            r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _",
                  id, pid, assessment_pid, name, item_key, ok, note, deleted_at AS "deleted_at: _"
               FROM ergonomic_items
               WHERE assessment_pid = $1 AND deleted_at IS NULL
               ORDER BY id"#,
            assessment.pid,
        )
        .fetch_all(pool(&ctx))
        .await?;
        // a row written before keys, holding a default sentence, is shown by that sentence's key
        let shown: Vec<serde_json::Value> = items
            .iter()
            .map(|item| {
                let mut value = serde_json::to_value(item)?;
                value["item_key"] =
                    serde_json::json!(rules::display_key(item.item_key.as_deref(), &item.name));
                Ok(value)
            })
            .collect::<std::result::Result<_, serde_json::Error>>()
            .map_err(|e| Error::Message(e.to_string()))?;
        view.push(serde_json::json!({
            "pid": assessment.pid,
            "workstation": assessment.workstation,
            "status": assessment.status,
            "assessed_on": assessment.assessed_on,
            "open_issues": rules::open_issues(
                &items.iter().map(|i| i.ok).collect::<Vec<_>>()
            ),
            "items": shown,
        }));
    }
    format::json(view)
}

/// `PUT /api/ergonomic-items/{pid}` body — `ok` / `issue` + an
/// **equipment** note.
#[derive(Debug, Deserialize)]
struct ItemAnswer {
    ok: bool,
    #[serde(default)]
    note: Option<String>,
}

/// `PUT /api/ergonomic-items/{pid}` — answer one checklist item
/// (only while the assessment is open).
#[debug_handler]
async fn answer_item(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<ItemAnswer>,
) -> Result<Response> {
    let mut problems = Problems::new();
    problems.cap_opt("note", payload.note.as_deref());
    ensure_valid(&problems.into_vec())?;
    let item = sqlx::query!(
        "SELECT pid, assessment_pid FROM ergonomic_items WHERE pid = $1 AND deleted_at IS NULL",
        records::parse_pid(&pid)?,
    )
    .fetch_optional(pool(&ctx))
    .await?
    .ok_or(Error::NotFound)?;
    let status = sqlx::query_scalar!(
        "SELECT status FROM ergonomic_assessments WHERE pid = $1 AND deleted_at IS NULL",
        item.assessment_pid,
    )
    .fetch_optional(pool(&ctx))
    .await?
    .ok_or(Error::NotFound)?;
    if status != "open" {
        return Err(unprocessable(
            "a completed assessment is a record; open a new one",
        ));
    }
    let mut tx = pool(&ctx).begin().await?;
    let updated = sqlx::query_as!(
        ergonomic_items::Model,
        r#"UPDATE ergonomic_items SET ok = $2, note = $3 WHERE pid = $1
           RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _",
                  id, pid, assessment_pid, name, item_key, ok, note, deleted_at AS "deleted_at: _""#,
        item.pid,
        payload.ok,
        payload.note,
    )
    .fetch_one(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "ergonomic_item",
        item.pid,
        "answered",
        caller.actor(),
        Some(serde_json::json!({ "ok": payload.ok })),
    )
    .await?;
    tx.commit().await?;
    format::json(row_json(&updated)?)
}

/// `POST /api/ergonomic-assessments/{pid}/complete` — completing
/// requires every item answered; stamps the date.
#[debug_handler]
async fn complete_assessment(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let assessment = sqlx::query!(
        "SELECT pid, status FROM ergonomic_assessments WHERE pid = $1 AND deleted_at IS NULL",
        records::parse_pid(&pid)?,
    )
    .fetch_optional(pool(&ctx))
    .await?
    .ok_or(Error::NotFound)?;
    if assessment.status != "open" {
        return Err(unprocessable("already completed"));
    }
    let answers: Vec<Option<bool>> = sqlx::query_scalar!(
        "SELECT ok FROM ergonomic_items WHERE assessment_pid = $1 AND deleted_at IS NULL",
        assessment.pid,
    )
    .fetch_all(pool(&ctx))
    .await?;
    rules::may_complete(&answers).map_err(|reason| unprocessable(&reason))?;
    let issues = rules::open_issues(&answers);
    let mut tx = pool(&ctx).begin().await?;
    let updated = sqlx::query_as!(
        ergonomic_assessments::Model,
        r#"UPDATE ergonomic_assessments SET status = 'completed', assessed_on = $2 WHERE pid = $1
           RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _",
                  id, pid, worker_pid, workstation, status, assessed_on, deleted_at AS "deleted_at: _""#,
        assessment.pid,
        chrono::Utc::now().date_naive(),
    )
    .fetch_one(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "ergonomic_assessment",
        assessment.pid,
        "completed",
        caller.actor(),
        Some(serde_json::json!({ "open_issues": issues })),
    )
    .await?;
    tx.commit().await?;
    format::json(row_json(&updated)?)
}

/// `GET /api/ergonomics/issues` — every `issue`-flagged item on a
/// live assessment, grouped by department (rota-tier visibility,
/// WPM-R27 precedent): worker, workstation, item, note; plus
/// per-department counts.
#[debug_handler]
async fn issues(State(ctx): State<AppContext>) -> Result<Response> {
    // An item counts only while its assessment and that assessment's worker are live.
    let rows = sqlx::query!(
        "SELECT i.name, i.item_key, i.note, a.workstation, a.status,
                w.pid AS worker_pid, w.department, w.display_name
         FROM ergonomic_items i
         JOIN ergonomic_assessments a ON a.pid = i.assessment_pid AND a.deleted_at IS NULL
         JOIN workers w ON w.pid = a.worker_pid AND w.deleted_at IS NULL
         WHERE i.deleted_at IS NULL AND i.ok = false
         ORDER BY i.id",
    )
    .fetch_all(pool(&ctx))
    .await?;
    let mut listed = Vec::new();
    let mut by_department: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();
    for row in &rows {
        *by_department.entry(row.department.clone()).or_default() += 1;
        listed.push(serde_json::json!({
            "department": row.department,
            "worker_pid": row.worker_pid,
            "display_name": row.display_name,
            "workstation": row.workstation,
            "item": row.name,
            "item_key": rules::display_key(row.item_key.as_deref(), &row.name),
            "note": row.note,
            "assessment_status": row.status,
        }));
    }
    format::json(serde_json::json!({
        "as_of": chrono::Utc::now(),
        "by_department": by_department,
        "issues": listed,
        "derivation": "issue-flagged checklist items on live assessments; equipment and \
                       environment facts only — no symptom field exists (WPM-D24); \
                       visibility is rota-tier",
    }))
}

/// The ergonomics routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add(
            "/workers/{pid}/ergonomic-assessments",
            post(create_assessment),
        )
        .add(
            "/workers/{pid}/ergonomic-assessments",
            get(list_assessments),
        )
        .add("/ergonomic-items/{pid}", put(answer_item))
        .add(
            "/ergonomic-assessments/{pid}/complete",
            post(complete_assessment),
        )
        .add("/ergonomics/issues", get(issues))
}
