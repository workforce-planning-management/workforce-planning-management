//! Migration: a position's **overtime status** (WPM-R219, WPM-D157, WPM-T417). Whether a post is exempt from statutory overtime is a
//! fact a person records, never one WPM infers from duties, so the column defaults to `unknown` and says so until someone records it.

use sea_orm_migration::prelude::*;

/// The migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Add `positions.overtime_status`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared(
                "ALTER TABLE positions ADD COLUMN IF NOT EXISTS overtime_status VARCHAR NOT NULL DEFAULT 'unknown'
                     CHECK (overtime_status IN ('exempt', 'non_exempt', 'unknown'))",
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
            .execute_unprepared("ALTER TABLE positions DROP COLUMN IF EXISTS overtime_status")
            .await?;
        Ok(())
    }
}
