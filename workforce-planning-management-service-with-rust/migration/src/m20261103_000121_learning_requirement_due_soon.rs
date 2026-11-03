//! Migration: a learning requirement's own **due-soon window** (WPM-D168, WPM-T434). Null means the deployment's window applies
//! (`WPM_LEARNING_DUE_SOON_DAYS`, else 90 calendar days); a number is that requirement's window, 1 to 366.

use sea_orm_migration::prelude::*;

/// The migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Add `learning_requirements.due_soon_days`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared(
                "ALTER TABLE learning_requirements ADD COLUMN IF NOT EXISTS due_soon_days INTEGER
                     CHECK (due_soon_days IS NULL OR due_soon_days BETWEEN 1 AND 366)",
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
                "ALTER TABLE learning_requirements DROP COLUMN IF EXISTS due_soon_days",
            )
            .await?;
        Ok(())
    }
}
