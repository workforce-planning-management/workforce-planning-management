//! Migration: **equipment** (WPM-R155). An asset has a kind, a tag and an owner role; an issue records which worker holds it
//! from when to when. **There is no column for a location, a cost, a value or a reading**: WPM does not track where an asset is.
//! A returned or written-off asset is a closed issue, never a deleted row; at most one issue per asset is open at a time.

use sea_orm_migration::prelude::*;

/// The migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create `assets` and `asset_issues`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS assets (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 organization_ref VARCHAR NOT NULL,
                 kind VARCHAR NOT NULL CHECK (kind IN ('laptop', 'phone', 'pass', 'key', 'other')),
                 tag VARCHAR NOT NULL CHECK (length(btrim(tag)) BETWEEN 1 AND 60),
                 owner_role VARCHAR NOT NULL CHECK (length(btrim(owner_role)) BETWEEN 1 AND 60),
                 recorded_by VARCHAR,
                 UNIQUE (organization_ref, tag)
             )",
        )
        .await?;
        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS asset_issues (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 asset_pid UUID NOT NULL,
                 worker_pid UUID NOT NULL,
                 issued_on DATE NOT NULL,
                 returned_on DATE,
                 outcome VARCHAR CHECK (outcome IN ('returned', 'written_off')),
                 note VARCHAR CHECK (note IS NULL OR length(note) <= 300),
                 recorded_by VARCHAR,
                 CHECK ((returned_on IS NULL) = (outcome IS NULL)),
                 CHECK (returned_on IS NULL OR returned_on >= issued_on),
                 CHECK (outcome IS DISTINCT FROM 'written_off' OR length(btrim(coalesce(note, ''))) > 0)
             )",
        )
        .await?;
        db.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS asset_issues_one_open ON asset_issues (asset_pid) WHERE returned_on IS NULL",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS asset_issues_by_worker ON asset_issues (worker_pid)",
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
        db.execute_unprepared("DROP TABLE IF EXISTS asset_issues")
            .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS assets").await?;
        Ok(())
    }
}
