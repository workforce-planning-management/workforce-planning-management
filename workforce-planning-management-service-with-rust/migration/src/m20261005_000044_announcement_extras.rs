//! Migration: **announcement extras** — a department audience, link
//! attachments, and read receipts.
//!
//! - `announcements.department`: when set, only that department (and the
//!   organization's editors) see the post; `NULL` is everyone.
//! - `announcements.links`: up to three `{label, url}` links (https only,
//!   validated by the service).
//! - `announcement_reads`: who has read a post. The reader sees their own;
//!   editors see only a **count**, never who.

use sea_orm_migration::prelude::*;

/// The announcement-extras migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Add the columns and the read table.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        conn.execute_unprepared(
            "ALTER TABLE announcements ADD COLUMN IF NOT EXISTS department VARCHAR NULL",
        )
        .await?;
        conn.execute_unprepared(
            "ALTER TABLE announcements ADD COLUMN IF NOT EXISTS links JSONB NOT NULL DEFAULT '[]'::jsonb",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS announcement_reads (
                 id SERIAL PRIMARY KEY,
                 announcement_pid UUID NOT NULL,
                 worker_pid UUID NOT NULL,
                 read_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 UNIQUE (announcement_pid, worker_pid)
             )",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS announcement_reads_worker ON announcement_reads (worker_pid)",
        )
        .await?;
        Ok(())
    }

    /// Drop them again.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        conn.execute_unprepared("DROP TABLE IF EXISTS announcement_reads")
            .await?;
        conn.execute_unprepared("ALTER TABLE announcements DROP COLUMN IF EXISTS links")
            .await?;
        conn.execute_unprepared("ALTER TABLE announcements DROP COLUMN IF EXISTS department")
            .await?;
        Ok(())
    }
}
