//! Migration: **cross-organization groups**. `groups.scope` is `organization`
//! (the default: one organization) or `confederation` (a community that spans
//! the organization named in `organization_ref` and every organization beneath
//! it in the confederation tree). Existing groups stay `organization`.

use sea_orm_migration::prelude::*;

/// The group-scope migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Add `scope`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared(
                "ALTER TABLE groups ADD COLUMN IF NOT EXISTS scope VARCHAR NOT NULL DEFAULT 'organization'",
            )
            .await?;
        Ok(())
    }

    /// Drop `scope`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared("ALTER TABLE groups DROP COLUMN IF EXISTS scope")
            .await?;
        Ok(())
    }
}
