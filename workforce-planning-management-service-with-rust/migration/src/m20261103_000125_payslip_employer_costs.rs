//! Migration: a payslip's **employer-side cost lines** (WPM-R223, WPM-D159, WPM-D170). The lines are a stub cost to the employer, never
//! deducted from net: `employer_costs` holds each line (name, class, rate, the pay it was applied to, the amount) and
//! `employer_cost_minor` their total. Both default to nothing, which is also what an organization with no lines configured has.

use sea_orm_migration::prelude::*;

/// The migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Add `payslips.employer_cost_minor` and `payslips.employer_costs`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        db.execute_unprepared(
            "ALTER TABLE payslips ADD COLUMN IF NOT EXISTS employer_cost_minor BIGINT NOT NULL DEFAULT 0
                 CHECK (employer_cost_minor >= 0)",
        )
        .await?;
        db.execute_unprepared(
            "ALTER TABLE payslips ADD COLUMN IF NOT EXISTS employer_costs JSONB NOT NULL DEFAULT '[]'::jsonb",
        )
        .await?;
        Ok(())
    }

    /// Drop them.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        db.execute_unprepared("ALTER TABLE payslips DROP COLUMN IF EXISTS employer_costs")
            .await?;
        db.execute_unprepared("ALTER TABLE payslips DROP COLUMN IF EXISTS employer_cost_minor")
            .await?;
        Ok(())
    }
}
