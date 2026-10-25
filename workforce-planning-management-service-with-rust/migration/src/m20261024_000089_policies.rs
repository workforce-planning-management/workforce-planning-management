//! Migration: **policies and acknowledgements** (WPM-R156, WPM-D105). A policy has numbered versions; a version has a title, a
//! reference to the document (WPM holds **no policy text**), an effective date, an audience and a number of calendar days to
//! acknowledge it. An acknowledgement is one worker saying they have read one version, on a date. A version never changes: a
//! change is a new version, which needs a new acknowledgement.

use sea_orm_migration::prelude::*;

/// The migration.
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
        let db = m.get_connection();
        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS policies (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 organization_ref VARCHAR NOT NULL,
                 name VARCHAR NOT NULL CHECK (length(btrim(name)) BETWEEN 1 AND 120),
                 recorded_by VARCHAR,
                 UNIQUE (organization_ref, name)
             )",
        )
        .await?;
        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS policy_versions (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 policy_pid UUID NOT NULL,
                 version INTEGER NOT NULL CHECK (version >= 1),
                 title VARCHAR NOT NULL CHECK (length(btrim(title)) BETWEEN 1 AND 160),
                 document_ref VARCHAR NOT NULL CHECK (length(document_ref) BETWEEN 1 AND 300),
                 effective_on DATE NOT NULL,
                 audience_kind VARCHAR NOT NULL CHECK (audience_kind IN ('all', 'department', 'job_title')),
                 audience_value VARCHAR,
                 acknowledge_within_days INTEGER NOT NULL DEFAULT 30 CHECK (acknowledge_within_days BETWEEN 1 AND 365),
                 recorded_by VARCHAR,
                 UNIQUE (policy_pid, version),
                 CHECK ((audience_kind = 'all') = (audience_value IS NULL))
             )",
        )
        .await?;
        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS policy_acknowledgements (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 version_pid UUID NOT NULL,
                 worker_pid UUID NOT NULL,
                 acknowledged_on DATE NOT NULL,
                 UNIQUE (version_pid, worker_pid)
             )",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS policy_acknowledgements_by_worker ON policy_acknowledgements (worker_pid)",
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
        for t in ["policy_acknowledgements", "policy_versions", "policies"] {
            db.execute_unprepared(&format!("DROP TABLE IF EXISTS {t}"))
                .await?;
        }
        Ok(())
    }
}
