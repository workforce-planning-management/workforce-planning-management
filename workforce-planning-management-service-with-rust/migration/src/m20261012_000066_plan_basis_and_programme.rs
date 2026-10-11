//! Migration: programme, FTE and basis on demand lines and requisitions (WPM-R82, WPM-T140).
//! Existing rows become `permanent` with no programme and no FTE target (an unknown is not a
//! zero). Nothing here is money. The demand-line key gains the basis and the programme, so a
//! permanent and a fixed-term line for the same department, role and date can both exist.

use sea_orm_migration::prelude::*;

/// The plan-basis-and-programme migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Add the columns and checks.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        conn.execute_unprepared(
            "ALTER TABLE plan_demand_lines \
             ADD COLUMN IF NOT EXISTS programme_ref VARCHAR NULL, \
             ADD COLUMN IF NOT EXISTS target_fte_percent INTEGER NULL, \
             ADD COLUMN IF NOT EXISTS basis VARCHAR NOT NULL DEFAULT 'permanent', \
             ADD COLUMN IF NOT EXISTS expected_ends_on DATE NULL",
        )
        .await?;
        conn.execute_unprepared(
            "ALTER TABLE plan_demand_lines \
             ADD CONSTRAINT plan_demand_lines_basis_check \
             CHECK (basis IN ('permanent','fixed_term','contractor','intern')), \
             ADD CONSTRAINT plan_demand_lines_fte_check \
             CHECK (target_fte_percent IS NULL OR target_fte_percent >= 0)",
        )
        .await?;
        conn.execute_unprepared("DROP INDEX IF EXISTS plan_demand_lines_key")
            .await?;
        conn.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS plan_demand_lines_key \
             ON plan_demand_lines (plan_pid, department, \
                 COALESCE(role_profile_pid, '00000000-0000-0000-0000-000000000000'::uuid), \
                 target_on, basis, COALESCE(programme_ref, ''))",
        )
        .await?;
        conn.execute_unprepared(
            "ALTER TABLE requisitions \
             ADD COLUMN IF NOT EXISTS programme_ref VARCHAR NULL, \
             ADD COLUMN IF NOT EXISTS basis VARCHAR NOT NULL DEFAULT 'permanent', \
             ADD COLUMN IF NOT EXISTS expected_duration_months INTEGER NULL",
        )
        .await?;
        conn.execute_unprepared(
            "ALTER TABLE requisitions \
             ADD CONSTRAINT requisitions_basis_check \
             CHECK (basis IN ('permanent','fixed_term','contractor','intern')), \
             ADD CONSTRAINT requisitions_duration_check \
             CHECK (expected_duration_months IS NULL OR expected_duration_months BETWEEN 1 AND 120)",
        )
        .await?;
        Ok(())
    }

    /// Drop the columns and checks.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        // The columns go, and with them the longer key; the original key returns. It fails if lines
        // that differ only by basis or programme exist: merge them first.
        conn.execute_unprepared("DROP INDEX IF EXISTS plan_demand_lines_key")
            .await?;
        conn.execute_unprepared(
            "ALTER TABLE plan_demand_lines \
             DROP COLUMN IF EXISTS programme_ref, DROP COLUMN IF EXISTS target_fte_percent, \
             DROP COLUMN IF EXISTS basis, DROP COLUMN IF EXISTS expected_ends_on",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS plan_demand_lines_key \
             ON plan_demand_lines (plan_pid, department, \
                 COALESCE(role_profile_pid, '00000000-0000-0000-0000-000000000000'::uuid), target_on)",
        )
        .await?;
        conn.execute_unprepared(
            "ALTER TABLE requisitions \
             DROP COLUMN IF EXISTS programme_ref, DROP COLUMN IF EXISTS basis, \
             DROP COLUMN IF EXISTS expected_duration_months",
        )
        .await?;
        Ok(())
    }
}
