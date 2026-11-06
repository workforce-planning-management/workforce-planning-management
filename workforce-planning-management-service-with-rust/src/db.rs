//! **Plain SQL** (WPM-D184): each query is written out with `sqlx::query!` and checked against the schema at build time, so a reviewer
//! reads exactly what runs. While `SeaORM` is being retired, both share one pool. A handler moves whole, because a transaction belongs to
//! one library and cannot span the two.

use axa::AppContext;
use sqlx::PgPool;
use uuid::Uuid;

/// The connection pool.
#[must_use]
pub fn pool(ctx: &AppContext) -> &PgPool {
    ctx.db.get_postgres_connection_pool()
}

/// A page bound for `LIMIT` or `OFFSET`, which `PostgreSQL` takes as a signed integer.
#[must_use]
pub fn bound(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

/// Whether `cause` is a unique constraint refusing a row.
#[must_use]
pub fn is_unique_violation(cause: &sqlx::Error) -> bool {
    cause
        .as_database_error()
        .is_some_and(sqlx::error::DatabaseError::is_unique_violation)
}

/// Record an audit entry: the audit write for a `sqlx` transaction. The database chains it.
///
/// # Errors
/// Propagates a database failure.
pub async fn audit(
    connection: &mut sqlx::PgConnection,
    entity: &str,
    entity_pid: Uuid,
    action: &str,
    actor: Option<&str>,
    snapshot: Option<serde_json::Value>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO audit_logs (entity, entity_pid, action, actor, snapshot) VALUES ($1, $2, $3, $4, $5)",
        entity,
        entity_pid,
        action,
        actor,
        snapshot,
    )
    .execute(connection)
    .await?;
    Ok(())
}

/// Tell a worker something: the notification write for a `sqlx` transaction.
///
/// # Errors
/// Propagates a database failure.
pub async fn notify(
    connection: &mut sqlx::PgConnection,
    worker_pid: Uuid,
    kind: &str,
    body: &str,
    data: serde_json::Value,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO notifications (pid, worker_pid, kind, body, data) VALUES ($1, $2, $3, $4, $5)",
        Uuid::new_v4(),
        worker_pid,
        kind,
        body,
        data,
    )
    .execute(connection)
    .await?;
    Ok(())
}
