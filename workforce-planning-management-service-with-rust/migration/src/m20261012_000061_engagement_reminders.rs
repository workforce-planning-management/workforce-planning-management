//! Migration: **end-of-engagement reminders and decisions** (WPM-R81). A reminder is recorded per
//! worker, per end date and per kind, so the daily task is idempotent: it tells the manager and HR
//! once about each end date, and again only when an extension gives the engagement a new one. A
//! decision (extend, convert, end) is recorded against the end date it settles, with who and when.
//!
//! Neither table holds a rate, a reason or free text. Both are about one person, so both are exported
//! with the worker and erased with them.

use sea_orm_migration::prelude::*;

/// The engagement-reminders migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create `engagement_reminders` and `engagement_decisions`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        for statement in [
            "CREATE TABLE IF NOT EXISTS engagement_reminders (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 worker_pid UUID NOT NULL,
                 ends_on DATE NOT NULL,
                 kind VARCHAR NOT NULL,
                 reminded_on DATE NOT NULL,
                 UNIQUE (worker_pid, ends_on, kind),
                 CHECK (kind IN ('ending', 'ended_undecided'))
             )",
            "CREATE TABLE IF NOT EXISTS engagement_decisions (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 worker_pid UUID NOT NULL,
                 ends_on DATE NOT NULL,
                 decision VARCHAR NOT NULL,
                 decided_by VARCHAR NULL,
                 decided_on DATE NOT NULL,
                 CHECK (decision IN ('extend', 'convert', 'end'))
             )",
            "CREATE INDEX IF NOT EXISTS engagement_decisions_worker ON engagement_decisions (worker_pid, ends_on)",
        ] {
            db.execute_unprepared(statement).await?;
        }
        Ok(())
    }

    /// Drop them again.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        db.execute_unprepared("DROP TABLE IF EXISTS engagement_decisions")
            .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS engagement_reminders")
            .await?;
        Ok(())
    }
}
