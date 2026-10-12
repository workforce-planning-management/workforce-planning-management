//! Migration: **governance commitments** (WPM-R64, WPM-D45). A recurring commitment that oversight
//! asks of a **role** (a meeting, review, return or self-assessment), with how often it recurs, how
//! long one occurrence takes, and how many people in the role take part. No person, no timesheet.

use sea_orm_migration::prelude::*;

/// The governance-commitments migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create `governance_commitments`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared(
                "CREATE TABLE IF NOT EXISTS governance_commitments (
                     created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                     updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                     id SERIAL PRIMARY KEY,
                     pid UUID NOT NULL UNIQUE,
                     organization_ref VARCHAR NOT NULL,
                     name VARCHAR NOT NULL,
                     role VARCHAR NOT NULL,
                     senior BOOLEAN NOT NULL DEFAULT FALSE,
                     kind VARCHAR NOT NULL,
                     frequency VARCHAR NOT NULL,
                     minutes INTEGER NOT NULL,
                     holders INTEGER NOT NULL DEFAULT 1,
                     recorded_by VARCHAR NULL,
                     CHECK (kind IN ('meeting', 'review', 'return', 'self_assessment')),
                     CHECK (frequency IN ('weekly', 'fortnightly', 'monthly', 'quarterly', 'half_yearly', 'annually')),
                     CHECK (minutes BETWEEN 1 AND 2400),
                     CHECK (holders BETWEEN 1 AND 1000)
                 )",
            )
            .await?;
        m.get_connection()
            .execute_unprepared(
                "CREATE INDEX IF NOT EXISTS governance_commitments_org ON governance_commitments (organization_ref)",
            )
            .await?;
        Ok(())
    }

    /// Drop it again.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS governance_commitments")
            .await?;
        Ok(())
    }
}
