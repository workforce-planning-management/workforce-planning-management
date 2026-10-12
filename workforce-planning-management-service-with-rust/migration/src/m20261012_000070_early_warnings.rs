//! Migration: **early warnings** (WPM-R68, WPM-T115). One row per condition that became true: a skill
//! pool over-committed, or an oversight measure in breach. It records the date the condition was
//! first seen true, when it stopped, and when a person acknowledged it, so time to report can be
//! measured. A row names a pool or a measure, never a person.

use sea_orm_migration::prelude::*;

/// The early-warning migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create `early_warnings`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        for sql in [
            "CREATE TABLE IF NOT EXISTS early_warnings (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 organization_ref VARCHAR NOT NULL,
                 kind VARCHAR NOT NULL,
                 subject_pid UUID NOT NULL,
                 subject_name VARCHAR NOT NULL,
                 first_true_on DATE NOT NULL,
                 cleared_on DATE,
                 acknowledged_on DATE,
                 CHECK (kind IN ('pool_over_committed', 'measure_breach')),
                 CHECK (cleared_on IS NULL OR cleared_on >= first_true_on),
                 CHECK (acknowledged_on IS NULL OR acknowledged_on >= first_true_on)
             )",
            "CREATE UNIQUE INDEX IF NOT EXISTS early_warnings_one_open
                 ON early_warnings (kind, subject_pid) WHERE cleared_on IS NULL",
            "CREATE INDEX IF NOT EXISTS early_warnings_org ON early_warnings (organization_ref, first_true_on)",
        ] {
            db.execute_unprepared(sql).await?;
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
            .execute_unprepared("DROP TABLE IF EXISTS early_warnings")
            .await?;
        Ok(())
    }
}
