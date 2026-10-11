//! Migration: split the headcount snapshots by basis (WPM-R84, WPM-T139) — contractor and fixed-term
//! headcount and FTE, so the contingent share has a trend. The columns are nullable: a row recorded
//! before this change reports the split as unknown, never as zero. Aggregates only; no one is named.

use sea_orm_migration::prelude::*;

/// The snapshots-by-basis migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Add the four columns.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared(
                "ALTER TABLE headcount_snapshots \
                 ADD COLUMN IF NOT EXISTS contractor_headcount INTEGER NULL, \
                 ADD COLUMN IF NOT EXISTS contractor_fte_percent_total BIGINT NULL, \
                 ADD COLUMN IF NOT EXISTS fixed_term_headcount INTEGER NULL, \
                 ADD COLUMN IF NOT EXISTS fixed_term_fte_percent_total BIGINT NULL",
            )
            .await?;
        Ok(())
    }

    /// Drop the four columns.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared(
                "ALTER TABLE headcount_snapshots \
                 DROP COLUMN IF EXISTS contractor_headcount, \
                 DROP COLUMN IF EXISTS contractor_fte_percent_total, \
                 DROP COLUMN IF EXISTS fixed_term_headcount, \
                 DROP COLUMN IF EXISTS fixed_term_fte_percent_total",
            )
            .await?;
        Ok(())
    }
}
