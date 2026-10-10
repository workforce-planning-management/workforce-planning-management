//! Migration: create the `workers` table — the single source of
//! employment truth (WPM-R7). Identities are `EntityRef` URNs; the
//! worker number is unique per organization; salary is minor units
//! (sensitive — masked at the read surface).
//!
//! Written as explicit SQL (family lesson: the loco `create_table`
//! helper pluralizes names; explicit SQL also gives BIGINT/DATE
//! control).

use sea_orm_migration::prelude::*;

/// The `workers` migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create `workers` + its uniqueness and lookup indexes.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        conn.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS workers (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 person_ref VARCHAR NOT NULL,
                 upstream_worker_ref VARCHAR NULL,
                 organization_ref VARCHAR NOT NULL,
                 worker_number VARCHAR NOT NULL,
                 display_name VARCHAR NOT NULL,
                 status VARCHAR NOT NULL,
                 employment_type VARCHAR NOT NULL,
                 fte_percent INTEGER NOT NULL,
                 department VARCHAR NOT NULL,
                 job_title VARCHAR NOT NULL,
                 manager_pid UUID NULL,
                 salary_minor BIGINT NULL,
                 salary_currency VARCHAR NULL,
                 hired_on DATE NOT NULL,
                 terminated_on DATE NULL,
                 deleted_at TIMESTAMPTZ NULL
             )",
        )
        .await?;
        // The per-organization worker-number uniqueness (WPM-R7),
        // scoped to live rows so a re-used number after termination +
        // soft delete stays possible.
        conn.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS workers_org_number \
             ON workers (organization_ref, worker_number) WHERE deleted_at IS NULL",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS workers_department ON workers (department)",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS workers_manager ON workers (manager_pid)",
        )
        .await?;
        Ok(())
    }

    /// Drop `workers` (rollback).
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS workers")
            .await?;
        Ok(())
    }
}
