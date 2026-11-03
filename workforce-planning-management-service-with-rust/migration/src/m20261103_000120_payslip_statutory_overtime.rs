//! Migration: the **statutory overtime premium** on a payslip (WPM-R219, WPM-D169, WPM-T418). The premium is already inside `gross_minor`;
//! this column says how much of the gross it is, so a payslip can show the line and nobody has to subtract to find it. Zero where none was
//! derived (no recorded status, no configured bands, an exempt post), which is also what every earlier payslip holds.

use sea_orm_migration::prelude::*;

/// The migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Add `payslips.statutory_overtime_minor`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared(
                "ALTER TABLE payslips ADD COLUMN IF NOT EXISTS statutory_overtime_minor BIGINT NOT NULL DEFAULT 0
                     CHECK (statutory_overtime_minor >= 0)",
            )
            .await?;
        Ok(())
    }

    /// Drop it.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared(
                "ALTER TABLE payslips DROP COLUMN IF EXISTS statutory_overtime_minor",
            )
            .await?;
        Ok(())
    }
}
