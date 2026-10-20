//! Audit + event read surface: the recent trail, the per-record
//! slice, the **department-scoped query** (spec `audit.md`), and the
//! recent event stream.

use loco_rs::prelude::*;
use serde::Deserialize;

use crate::models::audit_logs::Model as Audit;
use crate::models::event_outbox::Model as Outbox;
use crate::models::records;
use crate::streaming;

/// `GET /api/audits/recent` — the most recent entries (cap 100).
#[debug_handler]
async fn recent(State(ctx): State<AppContext>) -> Result<Response> {
    format::json(Audit::recent(&ctx.db, 100).await?)
}

/// `GET /api/audits/verify` — check the whole audit chain (WPM-R115). Reports the first break,
/// the number of entries and the head hash. Privileged callers only.
#[debug_handler]
async fn verify(State(ctx): State<AppContext>) -> Result<Response> {
    format::json(Audit::verify_chain(&ctx.db).await?)
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
    let rows = Audit::export_after(&ctx.db, after, limit).await?;
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
    format::json(Audit::for_entity(&ctx.db, records::parse_pid(&pid)?).await?)
}

/// `GET /api/audits?department=<name>&since=<rfc3339>` — everything
/// that changed in one department since a timestamp (cap 200).
#[derive(Debug, Deserialize)]
struct DepartmentParams {
    department: String,
    since: chrono::DateTime<chrono::FixedOffset>,
}

#[debug_handler]
async fn department(
    State(ctx): State<AppContext>,
    Query(params): Query<DepartmentParams>,
) -> Result<Response> {
    format::json(Audit::for_department_since(&ctx.db, &params.department, params.since, 200).await?)
}

/// `GET /api/events/recent` — the flat event view (memory ring under
/// the default transport; the outbox table under `outbox`).
#[debug_handler]
async fn events_recent(State(ctx): State<AppContext>) -> Result<Response> {
    if streaming::transport().is_outbox() {
        format::json(Outbox::recent(&ctx.db, 100).await?)
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
