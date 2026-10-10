//! Migration: **engagements** (WPM-R79, WPM-R80). The date an engagement ends, a dated
//! history of extensions, a contractor's details (supplier, route, rate) and an optional
//! employment-status assessment.
//!
//! `workers.engagement_ends_on` is the **last day** of the engagement. Existing
//! fixed-term, contractor and intern rows get no invented date: they stay empty and are
//! listed as "end date missing" until HR records one.
//!
//! The rate is personal and sensitive, so it lives in `worker_contractor_details`, not on
//! the worker row that many endpoints return. The three new tables are about one person,
//! so they are exported with the worker and erased with them.

use sea_orm_migration::prelude::*;

/// The engagements migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Add the column and create the three tables.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        for statement in [
            "ALTER TABLE workers ADD COLUMN IF NOT EXISTS engagement_ends_on DATE NULL",
            "CREATE TABLE IF NOT EXISTS engagement_extensions (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 worker_pid UUID NOT NULL,
                 previous_end DATE NOT NULL,
                 new_end DATE NOT NULL,
                 reason VARCHAR NULL,
                 decided_by VARCHAR NULL,
                 CHECK (new_end > previous_end)
             )",
            "CREATE INDEX IF NOT EXISTS engagement_extensions_worker ON engagement_extensions (worker_pid)",
            "CREATE TABLE IF NOT EXISTS worker_contractor_details (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 worker_pid UUID NOT NULL UNIQUE,
                 supplier_ref VARCHAR NULL,
                 route VARCHAR NULL,
                 rate_minor BIGINT NULL,
                 rate_currency VARCHAR NULL,
                 rate_basis VARCHAR NULL,
                 recorded_by VARCHAR NULL,
                 CHECK (rate_minor IS NULL OR rate_minor >= 0),
                 CHECK ((rate_minor IS NULL) = (rate_currency IS NULL)),
                 CHECK ((rate_minor IS NULL) = (rate_basis IS NULL))
             )",
            "CREATE TABLE IF NOT EXISTS engagement_status_assessments (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 worker_pid UUID NOT NULL,
                 outcome VARCHAR NOT NULL,
                 assessed_on DATE NOT NULL,
                 reviewed_by VARCHAR NULL
             )",
            "CREATE INDEX IF NOT EXISTS engagement_status_assessments_worker ON engagement_status_assessments (worker_pid)",
        ] {
            db.execute_unprepared(statement).await?;
        }
        Ok(())
    }

    /// Remove them again.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        for statement in [
            "DROP TABLE IF EXISTS engagement_status_assessments",
            "DROP TABLE IF EXISTS worker_contractor_details",
            "DROP TABLE IF EXISTS engagement_extensions",
            "ALTER TABLE workers DROP COLUMN IF EXISTS engagement_ends_on",
        ] {
            db.execute_unprepared(statement).await?;
        }
        Ok(())
    }
}
