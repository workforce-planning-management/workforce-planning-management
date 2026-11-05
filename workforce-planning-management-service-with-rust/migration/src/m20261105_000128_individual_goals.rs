//! Migration: **individual goals** (WPM-R224 to WPM-R226, WPM-T429). A goal belongs to one person and follows them through the
//! year. These are not the older `goals` table, which holds weighted lines inside one review (WPM-R10) and stays as it is.

use sea_orm_migration::prelude::*;

/// The migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create the tables.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        for statement in [
            "CREATE TABLE IF NOT EXISTS worker_goals (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 worker_pid UUID NOT NULL,
                 organization_ref VARCHAR NOT NULL,
                 title VARCHAR NOT NULL CHECK (length(btrim(title)) BETWEEN 1 AND 200),
                 description VARCHAR,
                 period_start DATE NOT NULL,
                 period_end DATE NOT NULL,
                 status VARCHAR NOT NULL CHECK (status IN ('proposed', 'agreed', 'in_progress', 'achieved',
                     'partly_achieved', 'not_achieved', 'withdrawn')),
                 proposed_by VARCHAR NOT NULL CHECK (proposed_by IN ('worker', 'manager')),
                 agreed_on DATE,
                 skill_pid UUID,
                 plan_item_pid UUID,
                 aligns_to VARCHAR,
                 deleted_at TIMESTAMPTZ,
                 CHECK (period_end >= period_start)
             )",
            "CREATE INDEX IF NOT EXISTS worker_goals_worker ON worker_goals (worker_pid)",
            "CREATE INDEX IF NOT EXISTS worker_goals_deleted_at_idx ON worker_goals (deleted_at)
                 WHERE deleted_at IS NOT NULL",
            "CREATE TABLE IF NOT EXISTS goal_measures (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 goal_pid UUID NOT NULL REFERENCES worker_goals (pid) ON DELETE CASCADE,
                 worker_pid UUID NOT NULL,
                 name VARCHAR NOT NULL CHECK (length(btrim(name)) BETWEEN 1 AND 200),
                 kind VARCHAR NOT NULL CHECK (kind IN ('number', 'milestone')),
                 unit VARCHAR,
                 start_value BIGINT,
                 target_value BIGINT,
                 current_value BIGINT,
                 done BOOLEAN
             )",
            "CREATE INDEX IF NOT EXISTS goal_measures_goal ON goal_measures (goal_pid)",
            "CREATE INDEX IF NOT EXISTS goal_measures_worker ON goal_measures (worker_pid)",
            "CREATE TABLE IF NOT EXISTS goal_check_ins (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 goal_pid UUID NOT NULL REFERENCES worker_goals (pid) ON DELETE CASCADE,
                 worker_pid UUID NOT NULL,
                 checked_on DATE NOT NULL,
                 status VARCHAR NOT NULL CHECK (status IN ('on_track', 'at_risk', 'off_track')),
                 note VARCHAR,
                 recorded_by VARCHAR
             )",
            "CREATE INDEX IF NOT EXISTS goal_check_ins_goal ON goal_check_ins (goal_pid)",
            "CREATE INDEX IF NOT EXISTS goal_check_ins_worker ON goal_check_ins (worker_pid)",
            "CREATE TABLE IF NOT EXISTS review_goal_outcomes (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 review_pid UUID NOT NULL,
                 goal_pid UUID NOT NULL,
                 worker_pid UUID NOT NULL,
                 reviewer_outcome VARCHAR,
                 reviewee_outcome VARCHAR,
                 snapshot JSONB,
                 UNIQUE (review_pid, goal_pid)
             )",
            "CREATE INDEX IF NOT EXISTS review_goal_outcomes_worker ON review_goal_outcomes (worker_pid)",
        ] {
            db.execute_unprepared(statement).await?;
        }
        Ok(())
    }

    /// Drop the tables.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        for table in [
            "review_goal_outcomes",
            "goal_check_ins",
            "goal_measures",
            "worker_goals",
        ] {
            db.execute_unprepared(&format!("DROP TABLE IF EXISTS {table}"))
                .await?;
        }
        Ok(())
    }
}
