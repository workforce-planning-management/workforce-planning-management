//! Audit + event read surface: the recent trail, the per-record
//! slice, the **department-scoped query** (spec `audit.md`), and the
//! recent event stream.

use axa::extract::{Path, Query};
use axa::prelude::*;
use serde::Deserialize;

use crate::db;
use crate::models::_entities::audit_logs::Model as Audit;
use crate::models::audit_logs::ChainReport;
use crate::models::records;
use crate::streaming::{self, Envelope, EventView};

/// `GET /api/audits/recent` — the most recent entries (cap 100).
#[debug_handler]
async fn recent(State(ctx): State<AppContext>) -> Result<Response> {
    let rows = sqlx::query_as!(
        Audit,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, entity, entity_pid, action, actor,
                  snapshot, chain_seq, prev_hash, entry_hash
             FROM audit_logs ORDER BY id DESC LIMIT 100"#,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    format::json(rows)
}

/// Check the whole chain: every entry's hash is the hash of its content and the entry before it, in order. Reports the first
/// place it breaks, the number of entries and the head hash to record somewhere the database cannot reach.
async fn verify_chain(ctx: &AppContext) -> Result<ChainReport> {
    let broken = sqlx::query_scalar!(
        r#"SELECT chain_seq AS "chain_seq!" FROM (
               SELECT chain_seq, prev_hash, entry_hash,
                      audit_entry_hash(prev_hash, chain_seq, entity, entity_pid, action, actor, snapshot, created_at) AS recomputed,
                      lag(entry_hash) OVER (ORDER BY chain_seq) AS previous,
                      lag(chain_seq) OVER (ORDER BY chain_seq) AS previous_seq
                 FROM audit_logs
           ) t
           WHERE entry_hash <> recomputed
              OR prev_hash <> COALESCE(previous, repeat('0', 64))
              OR chain_seq <> COALESCE(previous_seq, 0) + 1
           ORDER BY chain_seq LIMIT 1"#
    )
    .fetch_optional(db::pool(ctx))
    .await?;
    let head = sqlx::query!(
        r#"SELECT count(*)::bigint AS "entries!",
                  (SELECT entry_hash FROM audit_logs ORDER BY chain_seq DESC LIMIT 1) AS head_hash
             FROM audit_logs"#
    )
    .fetch_one(db::pool(ctx))
    .await?;
    Ok(ChainReport {
        ok: broken.is_none(),
        entries: head.entries,
        head_hash: head.head_hash,
        first_break: broken,
    })
}

/// `GET /api/audits/verify` — check the whole audit chain (WPM-R115). Reports the first break,
/// the number of entries and the head hash. Privileged callers only.
#[debug_handler]
async fn verify(State(ctx): State<AppContext>) -> Result<Response> {
    format::json(verify_chain(&ctx).await?)
}

/// Entries per page of the export, by default and at most.
const EXPORT_DEFAULT_LIMIT: u64 = 500;
const EXPORT_MAX_LIMIT: u64 = 1000;

#[derive(Debug, Deserialize)]
struct ExportParams {
    /// Resume after this `chain_seq` (the last one the monitor saw); absent means from the start.
    #[serde(default)]
    after: Option<i64>,
    #[serde(default)]
    limit: Option<u64>,
}

/// `GET /api/audits/export?after=<chain_seq>&limit=<n>` — the audit trail as JSON Lines for a security monitoring
/// system (WPM-R122): one entry per line, oldest first, resumable. Each line has the chain sequence, time, entity,
/// entity pid, action, actor and the two hashes, **not** the snapshot (which can carry record content).
/// `X-Next-After` names the last `chain_seq` returned; poll again with it. Privileged callers only.
#[debug_handler]
async fn export(
    State(ctx): State<AppContext>,
    Query(params): Query<ExportParams>,
) -> Result<Response> {
    let after = params.after.unwrap_or(0);
    if after < 0 {
        return Err(super::unprocessable("after must not be negative"));
    }
    let limit = params
        .limit
        .filter(|l| *l > 0)
        .unwrap_or(EXPORT_DEFAULT_LIMIT)
        .min(EXPORT_MAX_LIMIT);
    let rows = sqlx::query_as!(
        Audit,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, entity, entity_pid, action, actor,
                  snapshot, chain_seq, prev_hash, entry_hash
             FROM audit_logs WHERE chain_seq > $1 ORDER BY chain_seq LIMIT $2"#,
        after,
        i64::try_from(limit).unwrap_or(i64::MAX),
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let next = rows.last().map_or(after, |r| r.chain_seq);
    let body: String = rows.iter().map(crate::rules::audit_export::line).collect();
    Ok((
        [
            (
                axum::http::header::CONTENT_TYPE,
                "application/x-ndjson; charset=utf-8".to_string(),
            ),
            (
                axum::http::HeaderName::from_static("x-next-after"),
                next.to_string(),
            ),
        ],
        body,
    )
        .into_response())
}

/// `GET /api/audits/{entity_pid}` — one record's trail.
#[debug_handler]
async fn for_entity(State(ctx): State<AppContext>, Path(pid): Path<String>) -> Result<Response> {
    let rows = sqlx::query_as!(
        Audit,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, entity, entity_pid, action, actor,
                  snapshot, chain_seq, prev_hash, entry_hash
             FROM audit_logs WHERE entity_pid = $1 ORDER BY id DESC"#,
        records::parse_pid(&pid)?,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    format::json(rows)
}

/// `GET /api/audits?department=<name>&since=<rfc3339>` — everything
/// that changed in one department since a timestamp (cap 200).
#[derive(Debug, Deserialize)]
struct DepartmentParams {
    department: String,
    since: chrono::DateTime<chrono::FixedOffset>,
}

/// The cap on the department-scoped query.
const DEPARTMENT_LIMIT: usize = 200;

#[debug_handler]
async fn department(
    State(ctx): State<AppContext>,
    Query(params): Query<DepartmentParams>,
) -> Result<Response> {
    // Ten times the cap since the timestamp, then the department filter over the snapshot JSON, newest first.
    let rows = sqlx::query_as!(
        Audit,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, entity, entity_pid, action, actor,
                  snapshot, chain_seq, prev_hash, entry_hash
             FROM audit_logs WHERE created_at >= $1 ORDER BY id DESC LIMIT $2"#,
        params.since,
        i64::try_from(DEPARTMENT_LIMIT * 10).unwrap_or(i64::MAX),
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let rows: Vec<Audit> = rows
        .into_iter()
        .filter(|row| {
            row.snapshot
                .as_ref()
                .and_then(|s| s.get("department"))
                .and_then(serde_json::Value::as_str)
                == Some(params.department.as_str())
        })
        .take(DEPARTMENT_LIMIT)
        .collect();
    format::json(rows)
}

/// `GET /api/events/recent` — the flat event view (memory ring under
/// the default transport; the outbox table under `outbox`).
#[debug_handler]
async fn events_recent(State(ctx): State<AppContext>) -> Result<Response> {
    if streaming::transport().is_outbox() {
        let payloads =
            sqlx::query_scalar!("SELECT payload FROM event_outbox ORDER BY id DESC LIMIT 100")
                .fetch_all(db::pool(&ctx))
                .await?;
        let views: Vec<EventView> = payloads
            .into_iter()
            .filter_map(|payload| serde_json::from_value::<Envelope>(payload).ok())
            .map(|env| EventView::from(&env))
            .collect();
        format::json(views)
    } else {
        format::json(streaming::recent(100))
    }
}

/// The audit/event routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/audits/recent", get(recent))
        .add("/audits/verify", get(verify))
        .add("/audits/export", get(export))
        .add("/audits", get(department))
        .add("/audits/{entity_pid}", get(for_entity))
        .add("/events/recent", get(events_recent))
}
