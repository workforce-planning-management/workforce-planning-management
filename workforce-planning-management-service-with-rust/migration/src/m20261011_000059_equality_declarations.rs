//! Migration: **equality monitoring declarations** (WPM-R125, WPM-D74). A worker's own, voluntary
//! answer to each category a deployer monitors.
//!
//! This is special-category data. The table exists always, but nothing writes to it unless a
//! deployer records a lawful basis (`WPM_EQUALITY_MONITORING_BASIS`). It is readable by the worker
//! only, never by a manager, HR or payroll; the only output is an aggregate with small groups
//! withheld. It is exported to the worker alone, erased with them, and not in the retention sweep.

use sea_orm_migration::prelude::*;

/// The equality-declarations migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create `equality_declarations`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared(
                "CREATE TABLE IF NOT EXISTS equality_declarations (
                     created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                     updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                     id SERIAL PRIMARY KEY,
                     worker_pid UUID NOT NULL,
                     category VARCHAR NOT NULL,
                     value VARCHAR NOT NULL,
                     UNIQUE (worker_pid, category)
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
            .execute_unprepared("DROP TABLE IF EXISTS equality_declarations")
            .await?;
        Ok(())
    }
}
