//! Migration: a CPD entry may name the skill it developed (WPM-R157). The link is what lets a credential issued from the entry carry
//! the skill's ESCO or SFIA reference. It is optional: an entry with no skill is issued with no alignment, not a guessed one.

use sea_orm_migration::prelude::*;

/// The migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Add `cpd_entries.skill_pid`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared("ALTER TABLE cpd_entries ADD COLUMN IF NOT EXISTS skill_pid UUID")
            .await?;
        Ok(())
    }

    /// Drop the column.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared("ALTER TABLE cpd_entries DROP COLUMN IF EXISTS skill_pid")
            .await?;
        Ok(())
    }
}
