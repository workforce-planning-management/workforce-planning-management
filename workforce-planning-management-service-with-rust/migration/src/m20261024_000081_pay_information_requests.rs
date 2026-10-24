//! Migration: **pay information requests** (WPM-R145). A worker's request to know their own pay level and the average
//! pay, by sex, of their category of workers. The row holds the dates only: the figures are computed when HR answers and
//! are not stored.

use sea_orm_migration::prelude::*;

/// The migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create `pay_information_requests`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS pay_information_requests (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 worker_pid UUID NOT NULL,
                 asked_on DATE NOT NULL,
                 due_on DATE NOT NULL CHECK (due_on >= asked_on),
                 answered_on DATE CHECK (answered_on IS NULL OR answered_on >= asked_on),
                 answered_by VARCHAR,
                 deleted_at TIMESTAMPTZ
             )",
        )
        .await?;
        // A worker has at most one request waiting for an answer.
        db.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS pay_information_requests_one_open
                 ON pay_information_requests (worker_pid) WHERE answered_on IS NULL AND deleted_at IS NULL",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS pay_information_requests_deleted_at_idx
                 ON pay_information_requests (deleted_at) WHERE deleted_at IS NOT NULL",
        )
        .await?;
        Ok(())
    }

    /// Drop the table.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS pay_information_requests")
            .await?;
        Ok(())
    }
}
