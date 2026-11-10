//! **Plain SQL** (WPM-D184): each query is written out with `sqlx::query!` and checked against the schema at build time, so a reviewer
//! reads exactly what runs. While `SeaORM` is being retired, both share one pool. A handler moves whole, because a transaction belongs to
//! one library and cannot span the two.

use axa::AppContext;
use sqlx::PgPool;
use uuid::Uuid;

/// The connection pool: since axa 0.11, the context's own database handle (WPM-T482).
#[must_use]
pub const fn pool(ctx: &AppContext) -> &PgPool {
    &ctx.db
}

/// A `SeaORM` connection over the same pool, for the code not yet moved to `sqlx` (WPM-T466). axa 0.11 makes the context's handle
/// the `sqlx` pool when both features are on, so `SeaORM` wraps it here; the wrapper holds the pool's own `Arc`, so it opens nothing.
/// It goes when `SeaORM` does.
#[must_use]
pub fn orm(ctx: &AppContext) -> sea_orm::DatabaseConnection {
    sea_orm::SqlxPostgresConnector::from_sqlx_postgres_pool(ctx.db.clone())
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
