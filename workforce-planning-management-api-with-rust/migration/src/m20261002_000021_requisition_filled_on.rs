//! Migration: `requisitions.filled_on` — the date a requisition moved to
//! `filled`, so time-to-fill (`filled_on - opened_on`) can be derived.
//! Nullable: requisitions filled before this column existed have no
//! recorded fill date and are left out of time-to-fill, not guessed.

use sea_orm_migration::prelude::*;

/// The requisition-filled-on migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Add the nullable `filled_on` column.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared(
                "ALTER TABLE requisitions ADD COLUMN IF NOT EXISTS filled_on DATE NULL",
            )
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
            .execute_unprepared("ALTER TABLE requisitions DROP COLUMN IF EXISTS filled_on")
            .await?;
        Ok(())
    }
}
