//! Migration: **idempotency keys** (WPM-R132, WPM-D80). One row per (caller, key): the fingerprint of
//! the request that claimed the key and, once it finishes, the response to replay. A row lives for one
//! calendar day and is then deleted, so the stored response (which may hold what the first response
//! held) is not kept. It is not tied to a person, so it is neither exported nor erased with one.

use sea_orm_migration::prelude::*;

/// The idempotency-keys migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create `idempotency_keys`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS idempotency_keys (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 actor VARCHAR NOT NULL,
                 idem_key VARCHAR NOT NULL,
                 fingerprint VARCHAR NOT NULL,
                 status_code INTEGER NULL,
                 content_type VARCHAR NULL,
                 response_body BYTEA NULL,
                 UNIQUE (actor, idem_key)
             )",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idempotency_keys_created_at ON idempotency_keys (created_at)",
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
            .execute_unprepared("DROP TABLE IF EXISTS idempotency_keys")
            .await?;
        Ok(())
    }
}
