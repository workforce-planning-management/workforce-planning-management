//! Migration: a position's **number** (WPM-T434). The number a post is known by in an establishment list, so a learning requirement's
//! audience can name positions. It is optional and unique within an organization; a position with no number is not matched by a
//! position audience (the answer is `unknown`, never a guess).

use sea_orm_migration::prelude::*;

/// The migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Add `positions.number` and its uniqueness within an organization.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        db.execute_unprepared(
            "ALTER TABLE positions ADD COLUMN IF NOT EXISTS number VARCHAR
                 CHECK (number IS NULL OR length(btrim(number)) BETWEEN 1 AND 40)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS positions_number_unique ON positions (organization_ref, number) WHERE number IS NOT NULL",
        )
        .await?;
        Ok(())
    }

    /// Drop them.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        db.execute_unprepared("DROP INDEX IF EXISTS positions_number_unique")
            .await?;
        db.execute_unprepared("ALTER TABLE positions DROP COLUMN IF EXISTS number")
            .await?;
        Ok(())
    }
}
