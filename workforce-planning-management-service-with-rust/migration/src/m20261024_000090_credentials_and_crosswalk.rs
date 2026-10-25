//! Migration: **credential issuances** and **skill mappings** (WPM-R157, WPM-R158, WPM-D109).
//!
//! An issuance records that a credential was issued: who held the entry, what it was made from, when, the SHA-256 of the signed
//! credential and its position in the revocation list. **The credential itself is not stored** (the worker holds it) and it names
//! no person. A mapping is a statement that a skill declared under one record counts as another, with its confidence, its basis
//! and the source or the person's judgement it rests on.

use sea_orm_migration::prelude::*;

/// The migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create `credential_issuances` and `skill_mappings`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS credential_issuances (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 worker_pid UUID NOT NULL,
                 kind VARCHAR NOT NULL CHECK (kind IN ('cpd_entry')),
                 source_pid UUID NOT NULL,
                 issued_at TIMESTAMPTZ NOT NULL,
                 revoked_at TIMESTAMPTZ,
                 revoked_by VARCHAR,
                 sha256 VARCHAR NOT NULL CHECK (length(sha256) = 64)
             )",
        )
        .await?;
        db.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS credential_issuances_one_live ON credential_issuances (kind, source_pid) WHERE revoked_at IS NULL",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS credential_issuances_by_worker ON credential_issuances (worker_pid)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS skill_mappings (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 from_skill_pid UUID NOT NULL,
                 to_skill_pid UUID NOT NULL,
                 confidence VARCHAR NOT NULL CHECK (confidence IN ('exact', 'close', 'broader', 'narrower')),
                 basis VARCHAR NOT NULL CHECK (basis IN ('authority', 'judgement')),
                 source VARCHAR NOT NULL CHECK (length(btrim(source)) BETWEEN 1 AND 300),
                 recorded_by VARCHAR,
                 recorded_on DATE NOT NULL,
                 UNIQUE (from_skill_pid, to_skill_pid),
                 CHECK (from_skill_pid <> to_skill_pid)
             )",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS skill_mappings_by_target ON skill_mappings (to_skill_pid)",
        )
        .await?;
        Ok(())
    }

    /// Drop the tables.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        db.execute_unprepared("DROP TABLE IF EXISTS skill_mappings").await?;
        db.execute_unprepared("DROP TABLE IF EXISTS credential_issuances").await?;
        Ok(())
    }
}
