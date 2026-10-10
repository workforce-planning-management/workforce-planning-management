//! Migration: **resignations** (WPM-R127). A worker logs an intent to resign; a person accepts it.
//!
//! The reason is optional and from a closed list, and is the worker's own: it is shown only to them
//! and in small-group-safe aggregates. The row is about one person, so it is exported with the worker
//! and erased with them. At most one resignation per worker is open (`logged`) at a time.

use sea_orm_migration::prelude::*;

/// The resignations migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create `resignations`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        for statement in [
            "CREATE TABLE IF NOT EXISTS resignations (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 worker_pid UUID NOT NULL,
                 logged_on DATE NOT NULL,
                 proposed_last_day DATE NOT NULL,
                 reason VARCHAR NULL,
                 status VARCHAR NOT NULL DEFAULT 'logged',
                 agreed_last_day DATE NULL,
                 movement_pid UUID NULL,
                 decided_by VARCHAR NULL,
                 decided_at TIMESTAMPTZ NULL,
                 CHECK (status IN ('logged', 'withdrawn', 'accepted', 'rescinded'))
             )",
            "CREATE UNIQUE INDEX IF NOT EXISTS resignations_one_open
                 ON resignations (worker_pid) WHERE status = 'logged'",
            "CREATE INDEX IF NOT EXISTS resignations_worker ON resignations (worker_pid)",
        ] {
            db.execute_unprepared(statement).await?;
        }
        Ok(())
    }

    /// Drop it again.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS resignations")
            .await?;
        Ok(())
    }
}
