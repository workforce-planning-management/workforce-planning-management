//! Migration: **the AI and scoring deployer register** (WPM-R148, WPM-D103). For each feature that scores or ranks people,
//! what the deployer recorded under Article 26 of the AI Act: the Annex III category it chose, the role responsible for
//! human oversight, when workers' representatives and the affected workers were informed, and how long the system's logs are
//! kept. A role, never a person. Nothing recorded is "unknown", and a filled-in row is not evidence of compliance.

use sea_orm_migration::prelude::*;

/// The migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create `ai_register_entries`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared(
                "CREATE TABLE IF NOT EXISTS ai_register_entries (
                     created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                     updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                     id SERIAL PRIMARY KEY,
                     pid UUID NOT NULL UNIQUE,
                     feature_id VARCHAR NOT NULL UNIQUE CHECK (length(feature_id) BETWEEN 1 AND 20),
                     annex_category VARCHAR CHECK (annex_category IS NULL OR annex_category IN
                         ('recruitment_selection', 'work_relationship_decisions', 'task_allocation', 'monitoring_evaluation', 'not_high_risk')),
                     overseer_role VARCHAR CHECK (overseer_role IS NULL OR length(btrim(overseer_role)) BETWEEN 1 AND 120),
                     representatives_informed_on DATE,
                     workers_informed_on DATE,
                     log_retention_months INTEGER CHECK (log_retention_months IS NULL OR log_retention_months BETWEEN 1 AND 240),
                     retention_note VARCHAR CHECK (retention_note IS NULL OR length(retention_note) <= 500),
                     recorded_by VARCHAR
                 )",
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
            .execute_unprepared("DROP TABLE IF EXISTS ai_register_entries")
            .await?;
        Ok(())
    }
}
