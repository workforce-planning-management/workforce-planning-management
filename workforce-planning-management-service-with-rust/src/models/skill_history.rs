//! The **skill history** recorder: every change to a worker's declared level
//! in a skill closes the interval they held the old level and opens a new one,
//! so past skills and levels can be read back. One helper, called from each
//! place that declares or clears a skill, so the history cannot drift from
//! the declarations.

use axa::prelude::*;
use uuid::Uuid;

/// Record that, as of now, `worker_pid` holds `skill_pid` at `level`
/// (`None` = no longer declared), on `connection` (the caller's transaction). A no-op when nothing changed.
///
/// `source` says how it was recorded (`declared`, `framework`); `on_behalf` is
/// whether someone other than the person made the change.
///
/// # Errors
/// A database error.
pub async fn record_in(
    connection: &mut sqlx::PgConnection,
    worker_pid: Uuid,
    skill_pid: Uuid,
    level: Option<i32>,
    source: &str,
    actor: Option<&str>,
    on_behalf: bool,
) -> Result<()> {
    let open = sqlx::query_scalar!(
        "SELECT proficiency FROM worker_skill_history WHERE worker_pid = $1 AND skill_pid = $2 AND ended_at IS NULL",
        worker_pid,
        skill_pid,
    )
    .fetch_optional(&mut *connection)
    .await?;
    if open.is_some() && open == level {
        return Ok(());
    }
    if open.is_some() {
        sqlx::query!(
            "UPDATE worker_skill_history SET ended_at = now()
              WHERE worker_pid = $1 AND skill_pid = $2 AND ended_at IS NULL",
            worker_pid,
            skill_pid,
        )
        .execute(&mut *connection)
        .await?;
    }
    if let Some(level) = level {
        sqlx::query!(
            "INSERT INTO worker_skill_history
                 (pid, worker_pid, skill_pid, proficiency, started_at, source, recorded_by, on_behalf)
             VALUES ($1, $2, $3, $4, now(), $5, $6, $7)",
            Uuid::new_v4(),
            worker_pid,
            skill_pid,
            level,
            source,
            actor,
            on_behalf,
        )
        .execute(&mut *connection)
        .await?;
    }
    Ok(())
}
