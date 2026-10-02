//! Migration: `role_profiles` + `role_skill_requirements` — what a role
//! requires (WPM-R34), so a competency gap can be measured against a role
//! rather than a person. A profile is keyed by job title (the benchmark
//! key); each requirement is a catalogue skill with a minimum
//! proficiency (1–5) and an importance. Requirements are join rows and
//! are deleted outright; profiles soft-delete.

use sea_orm_migration::prelude::*;

/// The role-profiles migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create both tables + their uniqueness keys.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        conn.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS role_profiles (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 job_title VARCHAR NOT NULL,
                 description VARCHAR NULL,
                 source_ref VARCHAR NULL,
                 deleted_at TIMESTAMPTZ NULL
             )",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS role_profiles_job_title_key \
             ON role_profiles (job_title) WHERE deleted_at IS NULL",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS role_skill_requirements (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 role_profile_pid UUID NOT NULL,
                 skill_pid UUID NOT NULL,
                 min_proficiency INTEGER NOT NULL,
                 importance VARCHAR NOT NULL,
                 note VARCHAR NULL
             )",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS role_skill_requirements_key \
             ON role_skill_requirements (role_profile_pid, skill_pid)",
        )
        .await?;
        Ok(())
    }

    /// Drop both tables.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        conn.execute_unprepared("DROP TABLE IF EXISTS role_skill_requirements")
            .await?;
        conn.execute_unprepared("DROP TABLE IF EXISTS role_profiles")
            .await?;
        Ok(())
    }
}
