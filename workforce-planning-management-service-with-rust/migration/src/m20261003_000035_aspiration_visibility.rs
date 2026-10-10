//! Migration: aspiration **visibility tiers**. `shared` (a boolean: private, or
//! visible to anyone who can view the record) becomes `visibility`:
//! `private` (just the person), `manager` (their management chain — the
//! people above them in `manager_pid`), or `everyone`. Existing shared
//! aspirations keep the visibility they had: `everyone`.

use sea_orm_migration::prelude::*;

/// The aspiration-visibility migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Add `visibility`, carry `shared` across, drop `shared`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        conn.execute_unprepared(
            "ALTER TABLE worker_aspirations ADD COLUMN IF NOT EXISTS visibility VARCHAR NOT NULL DEFAULT 'private'",
        )
        .await?;
        conn.execute_unprepared(
            "DO $$ BEGIN \
               IF EXISTS (SELECT 1 FROM information_schema.columns \
                          WHERE table_name = 'worker_aspirations' AND column_name = 'shared') THEN \
                 UPDATE worker_aspirations SET visibility = 'everyone' WHERE shared; \
                 ALTER TABLE worker_aspirations DROP COLUMN shared; \
               END IF; \
             END $$",
        )
        .await?;
        Ok(())
    }

    /// Restore `shared` (anything visible beyond the person counts as shared).
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        conn.execute_unprepared(
            "ALTER TABLE worker_aspirations ADD COLUMN IF NOT EXISTS shared BOOLEAN NOT NULL DEFAULT false",
        )
        .await?;
        conn.execute_unprepared("UPDATE worker_aspirations SET shared = (visibility <> 'private')")
            .await?;
        conn.execute_unprepared("ALTER TABLE worker_aspirations DROP COLUMN IF EXISTS visibility")
            .await?;
        Ok(())
    }
}
