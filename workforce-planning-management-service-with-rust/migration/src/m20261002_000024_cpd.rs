//! Migration: the CPD ledger — continuing professional development as a
//! first-class record. `cpd_requirements` (hours or points required in a
//! period, for everyone or one job title), `cpd_entries` (what a worker
//! did, with evidence and optional verification; also the landing table
//! for LMS completions via `source`/`external_ref`), and
//! `professional_registrations` (a registration with an expiry).
//! Amounts are stored as hundredths of a unit so no float rounds a total.

use sea_orm_migration::prelude::*;

/// The CPD migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create the three tables + indexes.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        conn.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS cpd_requirements (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 name VARCHAR NOT NULL,
                 unit VARCHAR NOT NULL,
                 required_hundredths BIGINT NOT NULL,
                 period_start DATE NOT NULL,
                 period_end DATE NOT NULL,
                 job_title VARCHAR NULL,
                 deleted_at TIMESTAMPTZ NULL
             )",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS cpd_entries (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 worker_pid UUID NOT NULL,
                 entry_date DATE NOT NULL,
                 activity VARCHAR NOT NULL,
                 category VARCHAR NOT NULL,
                 unit VARCHAR NOT NULL,
                 amount_hundredths BIGINT NOT NULL,
                 evidence_note VARCHAR NULL,
                 evidence_url VARCHAR NULL,
                 source VARCHAR NOT NULL DEFAULT 'manual',
                 external_ref VARCHAR NULL,
                 verified_on DATE NULL,
                 verified_by VARCHAR NULL,
                 deleted_at TIMESTAMPTZ NULL
             )",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS cpd_entries_worker ON cpd_entries (worker_pid, entry_date)",
        )
        .await?;
        // One live entry per (worker, external reference): LMS sync idempotency.
        conn.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS cpd_entries_external_key \
             ON cpd_entries (worker_pid, external_ref) \
             WHERE external_ref IS NOT NULL AND deleted_at IS NULL",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS professional_registrations (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 worker_pid UUID NOT NULL,
                 body VARCHAR NOT NULL,
                 reference VARCHAR NULL,
                 registered_on DATE NULL,
                 expires_on DATE NULL,
                 deleted_at TIMESTAMPTZ NULL
             )",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS professional_registrations_worker \
             ON professional_registrations (worker_pid)",
        )
        .await?;
        Ok(())
    }

    /// Drop the three tables.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        for table in [
            "professional_registrations",
            "cpd_entries",
            "cpd_requirements",
        ] {
            conn.execute_unprepared(&format!("DROP TABLE IF EXISTS {table}"))
                .await?;
        }
        Ok(())
    }
}
