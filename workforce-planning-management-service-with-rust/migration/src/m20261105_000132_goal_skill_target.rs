//! Migration: **a goal's skill target** (WPM-R234, WPM-T462). The level a skill-building goal aims at, so the goal can show the person's
//! declared level beside it. Null means the goal names no target, and nothing is claimed either way.

use sea_orm_migration::prelude::*;

/// The migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Add the column.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared(
                "ALTER TABLE worker_goals ADD COLUMN skill_target_level integer
                   CHECK (skill_target_level BETWEEN 1 AND 5)",
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
            .execute_unprepared("ALTER TABLE worker_goals DROP COLUMN skill_target_level")
            .await?;
        Ok(())
    }
}
