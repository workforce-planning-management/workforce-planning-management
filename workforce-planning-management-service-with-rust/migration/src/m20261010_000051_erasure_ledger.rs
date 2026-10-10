//! Migration: the **erasure ledger** (WPM-R89, WPM-D63). When a worker is erased,
//! their pid and the time are recorded here, in the same transaction. After a
//! database is restored from a backup, replaying the ledger erases again anyone
//! who was erased after the backup was taken, so a restore does not bring back
//! what a subject asked to be erased.
//!
//! The row holds a pid and a timestamp only. The pid no longer identifies
//! anyone once the worker row has been erased (WPM-D22), so the ledger is not
//! personal data and needs no retention, export or erasure wiring.

use sea_orm_migration::prelude::*;

/// The erasure-ledger migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create `erasure_ledger`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared(
                "CREATE TABLE IF NOT EXISTS erasure_ledger (
                     worker_pid UUID PRIMARY KEY,
                     erased_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
                 )",
            )
            .await?;
        Ok(())
    }

    /// Drop it again.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS erasure_ledger")
            .await?;
        Ok(())
    }
}
