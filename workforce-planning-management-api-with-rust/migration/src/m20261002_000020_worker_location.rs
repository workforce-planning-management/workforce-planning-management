//! Migration: `workers.location` — an optional free-text work location
//! (office, site, or city) so the org chart can be viewed by location.
//! Nullable: an absent location is "unknown", never a default place.

use sea_orm_migration::prelude::*;

/// The worker-location migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Add the nullable `location` column.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared(
                "ALTER TABLE workers ADD COLUMN IF NOT EXISTS location VARCHAR NULL",
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
            .execute_unprepared("ALTER TABLE workers DROP COLUMN IF EXISTS location")
            .await?;
        Ok(())
    }
}
