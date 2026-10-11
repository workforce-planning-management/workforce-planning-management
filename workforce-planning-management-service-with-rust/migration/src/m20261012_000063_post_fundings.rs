//! Migration: **post funding** (WPM-R62, WPM-R83, WPM-R98). Where a worker's post is funded from
//! (`core`, `time_limited`, `external`, `programme`), the programme it belongs to if any, and the date
//! the funding ends. A kind and a date only: **no amount of money**. One record per worker; it is
//! about one person's post, so it is exported with the worker and erased with them.

use sea_orm_migration::prelude::*;

/// The post-funding migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create `post_fundings`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared(
                "CREATE TABLE IF NOT EXISTS post_fundings (
                     created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                     updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                     id SERIAL PRIMARY KEY,
                     worker_pid UUID NOT NULL UNIQUE,
                     kind VARCHAR NOT NULL,
                     programme_ref VARCHAR NULL,
                     ends_on DATE NULL,
                     recorded_by VARCHAR NULL,
                     CHECK (kind IN ('core', 'time_limited', 'external', 'programme'))
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
            .execute_unprepared("DROP TABLE IF EXISTS post_fundings")
            .await?;
        Ok(())
    }
}
