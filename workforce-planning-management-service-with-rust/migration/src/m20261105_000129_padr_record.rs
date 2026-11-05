//! Migration: **the PADR record** (WPM-R172, WPM-D176, WPM-T317). A review gains a kind, and a PADR has no review cycle, because it
//! falls due twelve months after the person's last one. Each version holds its content, its two signatures and, once signed, the
//! objectives as they stood (WPM-R225, WPM-D165).

use sea_orm_migration::prelude::*;

/// The migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Add the kind and create the table.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        for statement in [
            "ALTER TABLE reviews ADD COLUMN IF NOT EXISTS kind VARCHAR NOT NULL DEFAULT 'performance'
                 CHECK (kind IN ('performance', 'padr'))",
            "ALTER TABLE reviews ALTER COLUMN cycle_pid DROP NOT NULL",
            "ALTER TABLE reviews ADD CONSTRAINT reviews_performance_has_a_cycle
                 CHECK (kind = 'padr' OR cycle_pid IS NOT NULL)",
            "CREATE UNIQUE INDEX IF NOT EXISTS reviews_one_open_padr ON reviews (worker_pid)
                 WHERE kind = 'padr' AND status = 'draft' AND deleted_at IS NULL",
            "CREATE TABLE IF NOT EXISTS padr_versions (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 review_pid UUID NOT NULL REFERENCES reviews (pid) ON DELETE CASCADE,
                 worker_pid UUID NOT NULL,
                 version INTEGER NOT NULL CHECK (version >= 1),
                 period_start DATE NOT NULL,
                 period_end DATE NOT NULL,
                 values_summary VARCHAR,
                 wellbeing_offered BOOLEAN,
                 development_needs JSONB NOT NULL DEFAULT '[]',
                 reviewer_signed_on DATE,
                 reviewee_signed_on DATE,
                 reviewee_comment VARCHAR,
                 objectives JSONB,
                 UNIQUE (review_pid, version),
                 CHECK (period_end >= period_start)
             )",
            "CREATE INDEX IF NOT EXISTS padr_versions_worker ON padr_versions (worker_pid)",
        ] {
            db.execute_unprepared(statement).await?;
        }
        Ok(())
    }

    /// Drop the table and the kind. A PADR has no cycle, so the rows go first.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        for statement in [
            "DROP TABLE IF EXISTS padr_versions",
            "DELETE FROM reviews WHERE kind = 'padr'",
            "DROP INDEX IF EXISTS reviews_one_open_padr",
            "ALTER TABLE reviews DROP CONSTRAINT IF EXISTS reviews_performance_has_a_cycle",
            "ALTER TABLE reviews ALTER COLUMN cycle_pid SET NOT NULL",
            "ALTER TABLE reviews DROP COLUMN IF EXISTS kind",
        ] {
            db.execute_unprepared(statement).await?;
        }
        Ok(())
    }
}
