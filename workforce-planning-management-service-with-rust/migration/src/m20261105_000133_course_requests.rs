//! Migration: **course requests** (WPM-R236, WPM-T464). A person asks to take a course whose recorded fee is not zero, and a manager
//! above them decides. At most one request for a course is open at a time. No fee is copied here, so what the manager reads is the
//! catalogue's, and no reason is kept for a decision.

use sea_orm_migration::prelude::*;

/// The migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create the table.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        for statement in [
            "CREATE TABLE IF NOT EXISTS course_requests (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 worker_pid UUID NOT NULL,
                 course_ref VARCHAR NOT NULL,
                 status VARCHAR NOT NULL DEFAULT 'requested'
                     CHECK (status IN ('requested', 'approved', 'declined', 'withdrawn')),
                 decided_by VARCHAR,
                 decided_on DATE
             )",
            "CREATE INDEX IF NOT EXISTS course_requests_worker ON course_requests (worker_pid)",
            "CREATE UNIQUE INDEX IF NOT EXISTS course_requests_one_open ON course_requests (worker_pid, course_ref)
                 WHERE status = 'requested'",
        ] {
            db.execute_unprepared(statement).await?;
        }
        Ok(())
    }

    /// Drop the table.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS course_requests")
            .await?;
        Ok(())
    }
}
