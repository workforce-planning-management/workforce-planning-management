//! Shared record helpers: parsing a path `pid` and reading the active (not soft-deleted) worker.
//!
//! Both return the **`axa`** `Result` with `Error::NotFound` for an unparsable pid or a missing row, so every
//! `GET /…/{pid}` contract is an honest 404 (family lesson, pinned in the request tests).

use axa::prelude::*;
use uuid::Uuid;

use super::_entities::workers;

/// Parse a path `pid` string to a [`Uuid`], mapping failure to `404`
/// (an unparsable pid can never name a record).
///
/// # Errors
///
/// [`Error::NotFound`] when `pid` is not a UUID.
pub fn parse_pid(pid: &str) -> Result<Uuid> {
    Uuid::parse_str(pid).map_err(|_| Error::NotFound)
}

/// The active (not soft-deleted) worker by public id.
///
/// # Errors
///
/// [`Error::NotFound`] when absent or soft-deleted; any query error otherwise.
pub async fn worker_by_pid(ctx: &AppContext, pid: Uuid) -> Result<workers::Model> {
    sqlx::query_as!(
        workers::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, person_ref,
                  upstream_worker_ref, organization_ref, worker_number, display_name, status, employment_type,
                  fte_percent, department, job_title, location, work_jurisdiction, manager_pid, salary_minor,
                  salary_currency, hired_on, terminated_on, engagement_ends_on, deleted_at AS "deleted_at: _"
             FROM workers WHERE pid = $1 AND deleted_at IS NULL"#,
        pid,
    )
    .fetch_optional(crate::db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)
}
