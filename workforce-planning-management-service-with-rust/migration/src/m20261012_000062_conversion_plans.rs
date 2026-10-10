//! Migration: **conversion plans** (WPM-R98, WPM-D66). What a person intends for a fixed-term or
//! contractor engagement: convert, extend, end or not yet decided; the permanent post it would
//! become; who proposed it and who approved it; and when to look again.
//!
//! No pay figure is held. A plan is about one person, so it is exported with the worker and erased
//! with them. At most one plan per worker is open (proposed or approved) at a time.

use sea_orm_migration::prelude::*;

/// The conversion-plans migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create `conversion_plans`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        for statement in [
            "CREATE TABLE IF NOT EXISTS conversion_plans (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 worker_pid UUID NOT NULL,
                 intent VARCHAR NOT NULL,
                 target_on DATE NOT NULL,
                 department VARCHAR NULL,
                 role_profile_ref VARCHAR NULL,
                 post_funding_kind VARCHAR NULL,
                 post_funding_ends_on DATE NULL,
                 reason VARCHAR NULL,
                 proposed_by VARCHAR NULL,
                 approved_by VARCHAR NULL,
                 status VARCHAR NOT NULL DEFAULT 'proposed',
                 review_on DATE NULL,
                 settled_on DATE NULL,
                 CHECK (intent IN ('convert', 'extend', 'end', 'undecided')),
                 CHECK (status IN ('proposed', 'approved', 'done', 'abandoned')),
                 CHECK (post_funding_kind IS NULL
                        OR post_funding_kind IN ('core', 'time_limited', 'external', 'programme')),
                 CHECK (reason IS NULL OR char_length(reason) <= 500)
             )",
            "CREATE INDEX IF NOT EXISTS conversion_plans_worker ON conversion_plans (worker_pid, id)",
            "CREATE UNIQUE INDEX IF NOT EXISTS conversion_plans_one_open
                 ON conversion_plans (worker_pid) WHERE status IN ('proposed', 'approved')",
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
            .execute_unprepared("DROP TABLE IF EXISTS conversion_plans")
            .await?;
        Ok(())
    }
}
