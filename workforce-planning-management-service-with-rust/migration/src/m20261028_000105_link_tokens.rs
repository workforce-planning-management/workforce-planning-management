//! Migration: **single-use link tokens**. A link in a message that lets one person do one thing once. Only the SHA-256 hash of the
//! token is stored (64 hex characters), never the token, so a copy of this table holds nothing that works as a link. A token has a
//! purpose, an expiry, and a `used_at` that is set once and never cleared.

use sea_orm_migration::prelude::*;

/// The migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create `link_tokens`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS link_tokens (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 token_hash CHAR(64) NOT NULL UNIQUE CHECK (token_hash ~ '^[0-9a-f]{64}$'),
                 purpose VARCHAR NOT NULL CHECK (purpose ~ '^[a-z0-9_]{1,60}$'),
                 subject_pid UUID,
                 expires_at TIMESTAMPTZ NOT NULL,
                 used_at TIMESTAMPTZ,
                 CHECK (used_at IS NULL OR used_at >= created_at)
             )",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS link_tokens_by_expiry ON link_tokens (expires_at)",
        )
        .await?;
        // A used token's time is written once: it cannot be cleared or moved to make the token usable again.
        db.execute_unprepared(
            "CREATE OR REPLACE FUNCTION link_tokens_used_once() RETURNS trigger AS $$
             BEGIN
                 IF OLD.used_at IS NOT NULL AND NEW.used_at IS DISTINCT FROM OLD.used_at THEN
                     RAISE EXCEPTION 'a used link token cannot be used again';
                 END IF;
                 RETURN NEW;
             END;
             $$ LANGUAGE plpgsql",
        )
        .await?;
        db.execute_unprepared("DROP TRIGGER IF EXISTS link_tokens_used_once ON link_tokens")
            .await?;
        db.execute_unprepared(
            "CREATE TRIGGER link_tokens_used_once BEFORE UPDATE ON link_tokens
             FOR EACH ROW EXECUTE FUNCTION link_tokens_used_once()",
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
        let db = m.get_connection();
        db.execute_unprepared("DROP TRIGGER IF EXISTS link_tokens_used_once ON link_tokens")
            .await?;
        db.execute_unprepared("DROP FUNCTION IF EXISTS link_tokens_used_once()")
            .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS link_tokens")
            .await?;
        Ok(())
    }
}
