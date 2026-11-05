//! Migration: **the injury and illness notification record** (WPM-R221, WPM-D158, WPM-D179, WPM-T421). An incident and each report of
//! it. There is deliberately no text column: no description, no injury, no body part and no disease. A reference points to the statutory
//! record, which stays in the authority's portal or the deployer's own restricted store. Erasure clears `worker_pid` and keeps the row.

use sea_orm_migration::prelude::*;

/// The migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create the tables.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        for statement in [
            "CREATE TABLE IF NOT EXISTS injury_records (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 organization_ref VARCHAR NOT NULL,
                 jurisdiction VARCHAR NOT NULL,
                 kind VARCHAR NOT NULL
                     CHECK (kind IN ('accident', 'illness', 'dangerous_occurrence')),
                 worker_pid UUID,
                 occurred_at TIMESTAMPTZ NOT NULL,
                 learned_at TIMESTAMPTZ,
                 class VARCHAR NOT NULL
                     CHECK (class IN ('no_injury', 'no_absence', 'absence', 'serious', 'fatal')),
                 absence_days INTEGER CHECK (absence_days >= 0),
                 people_affected INTEGER NOT NULL DEFAULT 1 CHECK (people_affected >= 0),
                 statutory_record_ref VARCHAR,
                 keep_until DATE,
                 recorded_by VARCHAR
             )",
            "CREATE INDEX IF NOT EXISTS injury_records_organization ON injury_records (organization_ref)",
            "CREATE INDEX IF NOT EXISTS injury_records_worker ON injury_records (worker_pid)",
            "CREATE TABLE IF NOT EXISTS injury_reports (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 record_pid UUID NOT NULL REFERENCES injury_records (pid),
                 recipient VARCHAR NOT NULL
                     CHECK (recipient IN ('safety_authority', 'labour_inspectorate', 'insurer',
                                          'social_insurance', 'health_authority')),
                 due_at TIMESTAMPTZ,
                 sent_at TIMESTAMPTZ,
                 method VARCHAR CHECK (method IN ('online', 'phone', 'form')),
                 reference VARCHAR,
                 recorded_by VARCHAR,
                 CHECK ((sent_at IS NULL) = (method IS NULL))
             )",
            "CREATE INDEX IF NOT EXISTS injury_reports_record ON injury_reports (record_pid)",
        ] {
            db.execute_unprepared(statement).await?;
        }
        Ok(())
    }

    /// Drop the tables.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared(
                "DROP TABLE IF EXISTS injury_reports; DROP TABLE IF EXISTS injury_records",
            )
            .await?;
        Ok(())
    }
}
