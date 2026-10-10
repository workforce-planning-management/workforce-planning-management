//! Migration: **flexible working requests** (WPM-R126). A worker asks for a different working
//! arrangement; a person decides within a date the software computes; a refusal can be appealed
//! once, to someone other than who refused.
//!
//! The free text a worker writes (the effect on the team and how it might be managed) is theirs;
//! the audit trail never carries it. The table is about one person, so it is exported with the
//! worker and erased with them.

use sea_orm_migration::prelude::*;

/// The flexible-working migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create `flexible_working_requests`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        for statement in [
            "CREATE TABLE IF NOT EXISTS flexible_working_requests (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 worker_pid UUID NOT NULL,
                 kind VARCHAR NOT NULL,
                 requested_on DATE NOT NULL,
                 proposed_start DATE NOT NULL,
                 proposed_fte_percent INTEGER NULL,
                 effect_on_team VARCHAR NULL,
                 how_to_manage VARCHAR NULL,
                 status VARCHAR NOT NULL DEFAULT 'requested',
                 decide_by DATE NOT NULL,
                 decided_on DATE NULL,
                 decided_by VARCHAR NULL,
                 decision_reason VARCHAR NULL,
                 decision_note VARCHAR NULL,
                 counter_note VARCHAR NULL,
                 counter_fte_percent INTEGER NULL,
                 trial_until DATE NULL,
                 appealed_on DATE NULL,
                 appeal_note VARCHAR NULL,
                 appeal_decided_by VARCHAR NULL,
                 CHECK (proposed_fte_percent IS NULL OR proposed_fte_percent BETWEEN 1 AND 100),
                 CHECK (counter_fte_percent IS NULL OR counter_fte_percent BETWEEN 1 AND 100)
             )",
            "CREATE INDEX IF NOT EXISTS flexible_working_worker ON flexible_working_requests (worker_pid)",
            "CREATE INDEX IF NOT EXISTS flexible_working_open ON flexible_working_requests (status, decide_by)",
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
            .execute_unprepared("DROP TABLE IF EXISTS flexible_working_requests")
            .await?;
        Ok(())
    }
}
