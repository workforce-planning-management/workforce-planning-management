//! Migration: **training time recommendations**.
//!
//! - `skills.hours_per_level`: how many hours of training it typically takes
//!   to gain one proficiency level in a skill — a planning estimate set by
//!   whoever owns the skill catalogue; `NULL` falls back to the service
//!   default.
//! - `skill_courses`: catalogue courses that build a skill — an upstream
//!   course reference, a title, the hours it takes, and how many levels it
//!   typically adds. A recommendation prefers these; without one it says it is
//!   an estimate.

use sea_orm_migration::prelude::*;

/// The training-time migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Add the column and the table.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        conn.execute_unprepared(
            "ALTER TABLE skills ADD COLUMN IF NOT EXISTS hours_per_level INTEGER NULL \
             CHECK (hours_per_level IS NULL OR hours_per_level BETWEEN 1 AND 500)",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS skill_courses (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 skill_pid UUID NOT NULL,
                 course_ref VARCHAR NOT NULL,
                 title VARCHAR NOT NULL,
                 hours INTEGER NOT NULL CHECK (hours BETWEEN 1 AND 1000),
                 levels INTEGER NOT NULL DEFAULT 1 CHECK (levels BETWEEN 1 AND 4),
                 deleted_at TIMESTAMPTZ NULL
             )",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS skill_courses_current \
             ON skill_courses (skill_pid, course_ref) WHERE deleted_at IS NULL",
        )
        .await?;
        Ok(())
    }

    /// Drop them again.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        conn.execute_unprepared("DROP TABLE IF EXISTS skill_courses").await?;
        conn.execute_unprepared("ALTER TABLE skills DROP COLUMN IF EXISTS hours_per_level").await?;
        Ok(())
    }
}
