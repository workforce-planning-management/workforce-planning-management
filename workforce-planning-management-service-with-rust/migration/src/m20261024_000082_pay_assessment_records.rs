//! Migration: **joint pay assessment records** (WPM-R146). What a person has recorded for one category of workers and one
//! pay report: whether the gap was justified and on what, whether and when it was remedied, and whether and when the joint
//! assessment with workers' representatives was carried out, with its outcome. The service computes whether an assessment is
//! due from these and the report; it never concludes that anyone was paid unfairly.

use sea_orm_migration::prelude::*;

/// The migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create `pay_assessment_records`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS pay_assessment_records (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 organization_ref VARCHAR NOT NULL,
                 category VARCHAR NOT NULL CHECK (length(btrim(category)) BETWEEN 1 AND 120),
                 reported_on DATE NOT NULL,
                 justified BOOLEAN NOT NULL DEFAULT FALSE,
                 justification TEXT CHECK (justification IS NULL OR length(btrim(justification)) BETWEEN 10 AND 1000),
                 remedied_on DATE CHECK (remedied_on IS NULL OR remedied_on >= reported_on),
                 assessed_on DATE CHECK (assessed_on IS NULL OR assessed_on >= reported_on),
                 outcome VARCHAR CHECK (outcome IS NULL OR outcome IN
                     ('no_unjustified_difference', 'difference_found_remedy_planned', 'difference_remedied')),
                 representatives VARCHAR CHECK (representatives IS NULL OR length(representatives) <= 200),
                 recorded_by VARCHAR,
                 CHECK (justified = (justification IS NOT NULL)),
                 CHECK ((assessed_on IS NULL) = (outcome IS NULL))
             )",
        )
        .await?;
        db.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS pay_assessment_records_one
                 ON pay_assessment_records (organization_ref, lower(category), reported_on)",
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
            .execute_unprepared("DROP TABLE IF EXISTS pay_assessment_records")
            .await?;
        Ok(())
    }
}
