//! Migration: **joiners and leavers**.
//!
//! - `movements`: someone joining or leaving, on an effective day (the start
//!   day, or the leaver's last day), with a reason for a leaver. At most one
//!   open movement of each kind per worker.
//! - `movement_items`: its dated checklist.
//! - `handover_actions`: the audit trail of what a leaver held and who it
//!   went to — one row per reassignment, closure or revocation.

use sea_orm_migration::prelude::*;

/// The joiners-and-leavers migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create the three tables.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        conn.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS movements (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 worker_pid UUID NOT NULL,
                 kind VARCHAR NOT NULL CHECK (kind IN ('joiner', 'leaver')),
                 effective_on DATE NOT NULL,
                 reason VARCHAR NULL,
                 status VARCHAR NOT NULL DEFAULT 'open'
                     CHECK (status IN ('open', 'completed', 'cancelled')),
                 notes VARCHAR NULL,
                 created_by VARCHAR NULL,
                 completed_at TIMESTAMPTZ NULL
             )",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS movements_one_open \
             ON movements (worker_pid, kind) WHERE status = 'open'",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS movement_items (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 movement_pid UUID NOT NULL,
                 position INTEGER NOT NULL,
                 title VARCHAR NOT NULL,
                 category VARCHAR NOT NULL,
                 due_on DATE NOT NULL,
                 assignee_pid UUID NULL,
                 done_on DATE NULL,
                 done_by VARCHAR NULL,
                 skipped_reason VARCHAR NULL
             )",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS movement_items_movement ON movement_items (movement_pid)",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS movement_items_assignee ON movement_items (assignee_pid)",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS handover_actions (
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 movement_pid UUID NOT NULL,
                 kind VARCHAR NOT NULL,
                 subject_pid UUID NOT NULL,
                 subject_label VARCHAR NULL,
                 from_worker UUID NOT NULL,
                 to_worker UUID NULL,
                 action VARCHAR NOT NULL,
                 note VARCHAR NULL,
                 performed_by VARCHAR NULL,
                 performed_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
             )",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS handover_actions_movement ON handover_actions (movement_pid)",
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
        for table in ["handover_actions", "movement_items", "movements"] {
            conn.execute_unprepared(&format!("DROP TABLE IF EXISTS {table}")).await?;
        }
        Ok(())
    }
}
