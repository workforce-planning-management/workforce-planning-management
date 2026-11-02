//! Migration: **`qualification_reminders`** (WPM-T435). That the person and their manager were told about one expiry date of one
//! qualification, once. A reminder is recorded per qualification, per expiry date and per kind, so the daily task is idempotent:
//! a re-run tells no one twice, and a renewal (a new expiry date) gets a new reminder.
//!
//! No title, no free text: only who, which qualification, which date, which kind and when. It is about one person, so it is exported
//! with the worker and erased with them.

use sea_orm_migration::prelude::*;

/// The qualification-reminders migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create `qualification_reminders`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        for statement in [
            "CREATE TABLE IF NOT EXISTS qualification_reminders (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 worker_pid UUID NOT NULL,
                 qualification_pid UUID NOT NULL,
                 expires_on DATE NOT NULL,
                 kind VARCHAR NOT NULL,
                 reminded_on DATE NOT NULL,
                 UNIQUE (qualification_pid, expires_on, kind),
                 CHECK (kind IN ('expiring', 'expired'))
             )",
            "CREATE INDEX IF NOT EXISTS qualification_reminders_worker ON qualification_reminders (worker_pid)",
        ] {
            db.execute_unprepared(statement).await?;
        }
        Ok(())
    }

    /// Drop it again.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS qualification_reminders")
            .await?;
        Ok(())
    }
}
