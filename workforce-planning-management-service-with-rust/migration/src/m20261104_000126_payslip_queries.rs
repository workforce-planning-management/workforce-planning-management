//! Migration: **payslip queries** (WPM-R178, WPM-T324). A person flags one of their payslips with a short comment; payroll
//! sees it and closes it. A query changes no amount. At most one is open per payslip.

use sea_orm_migration::prelude::*;

/// The migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create `payslip_queries`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS payslip_queries (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 payslip_pid UUID NOT NULL,
                 worker_pid UUID NOT NULL,
                 comment VARCHAR NOT NULL CHECK (length(btrim(comment)) BETWEEN 1 AND 500),
                 raised_on DATE NOT NULL,
                 closed_on DATE,
                 closed_by VARCHAR,
                 deleted_at TIMESTAMPTZ,
                 CHECK (closed_on IS NULL OR closed_on >= raised_on)
             )",
        )
        .await?;
        db.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS payslip_queries_one_open
                 ON payslip_queries (payslip_pid) WHERE closed_on IS NULL AND deleted_at IS NULL",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS payslip_queries_worker ON payslip_queries (worker_pid)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS payslip_queries_deleted_at_idx ON payslip_queries (deleted_at)
                 WHERE deleted_at IS NOT NULL",
        )
        .await?;
        Ok(())
    }

    /// Drop `payslip_queries`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS payslip_queries")
            .await?;
        Ok(())
    }
}
