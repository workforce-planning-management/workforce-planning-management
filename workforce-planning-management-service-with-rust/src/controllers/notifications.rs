//! In-app notifications (WPM-R31 / WPM-D23) — the worker's own
//! list (unread first) and the mark-read action. Reference-only rows;
//! the write side lives in the handlers that make the change.

use axa::extract::Path;
use axa::prelude::*;
use chrono::Utc;

use super::{record_rejection, row_json, with_page_headers};
use crate::auth::{self, MaybeAuthUser};
use crate::db::{bound, pool};
use crate::models::_entities::notifications;
use crate::models::records;

/// `GET /api/workers/{pid}/notifications` — the worker's
/// notifications, unread first, newest within each. `$sub`-owned.
#[debug_handler]
async fn list_notifications(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    axa::extract::Query(page): axa::extract::Query<super::Page>,
) -> Result<Response> {
    let (limit, offset) = super::window(page)?;
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    auth::authorize_record(
        &caller,
        authentication_verifier::Action::Read,
        &auth::worker_resource_attrs(&worker),
    )
    .map_err(record_rejection)?;
    let total = sqlx::query_scalar!(
        r#"SELECT count(*) AS "total!" FROM notifications WHERE worker_pid = $1"#,
        worker.pid,
    )
    .fetch_one(pool(&ctx))
    .await?;
    // Unread first, then newest first.
    let rows = sqlx::query_as!(
        notifications::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, kind, body, data,
                  read_at AS "read_at: _"
           FROM notifications WHERE worker_pid = $1
           ORDER BY read_at IS NOT NULL, id DESC
           LIMIT $2 OFFSET $3"#,
        worker.pid,
        bound(limit),
        bound(offset),
    )
    .fetch_all(pool(&ctx))
    .await?;
    let shown = rows.iter().map(row_json).collect::<Result<Vec<_>>>()?;
    Ok(with_page_headers(
        format::json(shown)?,
        total.unsigned_abs(),
        limit,
        offset,
    ))
}

/// `POST /api/notifications/{pid}/read` — mark one notification read.
/// Owner-only: the record-level check runs against the notification's
/// worker, so nobody clears another person's bell.
#[debug_handler]
async fn mark_read(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let notification = sqlx::query!(
        "SELECT pid, worker_pid FROM notifications WHERE pid = $1",
        records::parse_pid(&pid)?,
    )
    .fetch_optional(pool(&ctx))
    .await?
    .ok_or(Error::NotFound)?;
    let owner = records::find_worker(&ctx.db, notification.worker_pid).await?;
    auth::authorize_record(
        &caller,
        authentication_verifier::Action::Write,
        &auth::worker_resource_attrs(&owner),
    )
    .map_err(record_rejection)?;
    let updated = sqlx::query_as!(
        notifications::Model,
        r#"UPDATE notifications SET read_at = $1 WHERE pid = $2
           RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, kind, body, data,
                     read_at AS "read_at: _""#,
        Utc::now(),
        notification.pid,
    )
    .fetch_one(pool(&ctx))
    .await?;
    format::json(row_json(&updated)?)
}

/// The notification routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/workers/{pid}/notifications", get(list_notifications))
        .add("/notifications/{pid}/read", post(mark_read))
}
