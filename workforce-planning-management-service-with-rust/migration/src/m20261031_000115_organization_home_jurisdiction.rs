//! Migration: an **organization's home jurisdiction** (WPM-D153, WPM-T384): the jurisdiction code of the legal entity's registered
//! jurisdiction, set by HR of the organization. A reader about the organization as a whole (the pay gap report, the joint
//! assessment, retention) resolves the chain from it; a reader about one worker uses that worker's `work_jurisdiction`.
//!
//! There is no `organizations` table (an organization is a reference), so the setting has a table of its own, keyed by the
//! reference. Absent or `NULL` is unknown: the reader falls back to its `WPM_*` setting and then its default, never to a guess.

use sea_orm_migration::prelude::*;

/// The migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create `organization_settings` with its one column, `home_jurisdiction`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared(
                "CREATE TABLE IF NOT EXISTS organization_settings (
                     organization_ref VARCHAR(255) PRIMARY KEY,
                     home_jurisdiction VARCHAR(7)
                         CHECK (home_jurisdiction ~ '^[A-Z0-9]{2,3}(-[A-Z0-9]{1,3})?$'),
                     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                     updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
                 )",
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
            .execute_unprepared("DROP TABLE IF EXISTS organization_settings")
            .await?;
        Ok(())
    }
}
