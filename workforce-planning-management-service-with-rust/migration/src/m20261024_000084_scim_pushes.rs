//! Migration: **SCIM push state** (WPM-R150, WPM-D102). For each worker sent to the identity provider: the id the provider gave
//! the user, a fingerprint of what was last sent (so an unchanged user is not sent again), and the retry state. It holds no
//! name, no contact detail and no body, only a reference to the worker and the provider's own id.

use sea_orm_migration::prelude::*;

/// The migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create `scim_pushes`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared(
                "CREATE TABLE IF NOT EXISTS scim_pushes (
                     created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                     updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                     id SERIAL PRIMARY KEY,
                     pid UUID NOT NULL UNIQUE,
                     worker_pid UUID NOT NULL UNIQUE,
                     scim_id VARCHAR CHECK (scim_id IS NULL OR length(scim_id) BETWEEN 1 AND 200),
                     body_hash VARCHAR CHECK (body_hash IS NULL OR length(body_hash) = 64),
                     pushed_at TIMESTAMPTZ,
                     attempts INTEGER NOT NULL DEFAULT 0 CHECK (attempts >= 0),
                     next_attempt_at TIMESTAMPTZ,
                     last_error VARCHAR CHECK (last_error IS NULL OR length(last_error) <= 200)
                 )",
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
            .execute_unprepared("DROP TABLE IF EXISTS scim_pushes")
            .await?;
        Ok(())
    }
}
