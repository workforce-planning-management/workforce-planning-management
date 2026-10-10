//! Migration: **groups** — communities of practice, communities of interest,
//! and other self-organised sets of people. A worker can belong to **many**
//! groups at once; each membership has a start and (when they leave) a stop,
//! like roles and skill levels, so past membership is kept.

use sea_orm_migration::prelude::*;

/// The groups migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create `groups` and `group_members`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        conn.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS groups (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 name VARCHAR NOT NULL,
                 kind VARCHAR NOT NULL,
                 description VARCHAR NULL,
                 deleted_at TIMESTAMPTZ NULL
             )",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS groups_name_live \
             ON groups (lower(name)) WHERE deleted_at IS NULL",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS group_members (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 group_pid UUID NOT NULL,
                 worker_pid UUID NOT NULL,
                 role VARCHAR NOT NULL DEFAULT 'member',
                 joined_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 left_at TIMESTAMPTZ NULL,
                 recorded_by VARCHAR NULL,
                 on_behalf BOOLEAN NOT NULL DEFAULT false
             )",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS group_members_current \
             ON group_members (group_pid, worker_pid) WHERE left_at IS NULL",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS group_members_worker ON group_members (worker_pid)",
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
        conn.execute_unprepared("DROP TABLE IF EXISTS group_members")
            .await?;
        conn.execute_unprepared("DROP TABLE IF EXISTS groups")
            .await?;
        Ok(())
    }
}
