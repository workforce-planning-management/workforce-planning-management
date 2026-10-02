//! Migration: `mobility_interests` — an employee expressing interest in a
//! role profile or an open requisition (internal mobility, WPM-R37).
//! Employee-initiated and owned by the worker: visible to them, with
//! only aggregate counts visible to anyone else (WPM-D28). Withdrawing is
//! a soft-delete.

use sea_orm_migration::prelude::*;

/// The mobility-interests migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create the table + its uniqueness keys.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        conn.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS mobility_interests (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 worker_pid UUID NOT NULL,
                 role_profile_pid UUID NULL,
                 requisition_pid UUID NULL,
                 note VARCHAR NULL,
                 deleted_at TIMESTAMPTZ NULL,
                 CHECK (num_nonnulls(role_profile_pid, requisition_pid) = 1)
             )",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS mobility_interests_role_key \
             ON mobility_interests (worker_pid, role_profile_pid) \
             WHERE role_profile_pid IS NOT NULL AND deleted_at IS NULL",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS mobility_interests_requisition_key \
             ON mobility_interests (worker_pid, requisition_pid) \
             WHERE requisition_pid IS NOT NULL AND deleted_at IS NULL",
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
            .execute_unprepared("DROP TABLE IF EXISTS mobility_interests")
            .await?;
        Ok(())
    }
}
