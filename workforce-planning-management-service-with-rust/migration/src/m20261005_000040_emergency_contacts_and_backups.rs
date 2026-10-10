//! Migration: **emergency contacts** and **backups**.
//!
//! - `emergency_contacts`: people a worker names to be reached in an
//!   emergency. Third-party personal data, kept by the worker and HR only.
//! - `worker_backups`: the colleague(s) who cover for a worker when they are
//!   out — ranked (`priority` 1 is the first choice), optionally for a dated
//!   window. Visible to anyone who can see the worker, so colleagues know who
//!   to ask.

use sea_orm_migration::prelude::*;

/// The emergency-contacts and backups migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create both tables.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        conn.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS emergency_contacts (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 worker_pid UUID NOT NULL,
                 name VARCHAR NOT NULL,
                 relationship VARCHAR NOT NULL,
                 phone VARCHAR NOT NULL,
                 alt_phone VARCHAR NULL,
                 email VARCHAR NULL,
                 priority INTEGER NOT NULL DEFAULT 1,
                 note VARCHAR NULL,
                 recorded_by VARCHAR NULL,
                 on_behalf BOOLEAN NOT NULL DEFAULT false,
                 deleted_at TIMESTAMPTZ NULL,
                 CHECK (priority >= 1)
             )",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS emergency_contacts_worker ON emergency_contacts (worker_pid)",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS worker_backups (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 worker_pid UUID NOT NULL,
                 backup_pid UUID NOT NULL,
                 priority INTEGER NOT NULL DEFAULT 1,
                 starts_on DATE NULL,
                 ends_on DATE NULL,
                 note VARCHAR NULL,
                 recorded_by VARCHAR NULL,
                 on_behalf BOOLEAN NOT NULL DEFAULT false,
                 deleted_at TIMESTAMPTZ NULL,
                 CHECK (worker_pid <> backup_pid),
                 CHECK (priority >= 1),
                 CHECK (starts_on IS NULL OR ends_on IS NULL OR ends_on >= starts_on)
             )",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS worker_backups_current \
             ON worker_backups (worker_pid, backup_pid) WHERE deleted_at IS NULL",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS worker_backups_backup ON worker_backups (backup_pid)",
        )
        .await?;
        Ok(())
    }

    /// Drop both tables.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        conn.execute_unprepared("DROP TABLE IF EXISTS worker_backups")
            .await?;
        conn.execute_unprepared("DROP TABLE IF EXISTS emergency_contacts")
            .await?;
        Ok(())
    }
}
