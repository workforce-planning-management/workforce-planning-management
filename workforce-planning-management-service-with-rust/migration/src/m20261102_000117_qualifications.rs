//! Migration: **qualifications** (WPM-R171, WPM-T316). A qualification a person holds is `declared` until HR marks it `verified` with an
//! evidence reference (where the proof is held: no certificate is stored). A role profile may require qualifications; the gap is read
//! beside the skill gap.

use sea_orm_migration::prelude::*;

/// The migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create `qualifications` and `role_qualification_requirements`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS qualifications (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 worker_pid UUID NOT NULL,
                 kind VARCHAR NOT NULL CHECK (kind IN ('academic', 'professional', 'vocational', 'licence')),
                 title VARCHAR NOT NULL CHECK (length(btrim(title)) BETWEEN 1 AND 200),
                 awarding_body VARCHAR CHECK (awarding_body IS NULL OR length(btrim(awarding_body)) BETWEEN 1 AND 200),
                 framework VARCHAR CHECK (framework IS NULL OR length(btrim(framework)) BETWEEN 1 AND 40),
                 level INTEGER CHECK (level IS NULL OR level BETWEEN 0 AND 20),
                 awarded_on DATE,
                 expires_on DATE,
                 verified_by VARCHAR,
                 verified_on DATE,
                 evidence_ref VARCHAR CHECK (evidence_ref IS NULL OR length(btrim(evidence_ref)) BETWEEN 1 AND 200),
                 recorded_by VARCHAR,
                 CHECK (verified_on IS NULL OR evidence_ref IS NOT NULL),
                 CHECK ((framework IS NULL) = (level IS NULL))
             )",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS qualifications_by_worker ON qualifications (worker_pid)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS role_qualification_requirements (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 role_profile_pid UUID NOT NULL,
                 title VARCHAR NOT NULL CHECK (length(btrim(title)) BETWEEN 1 AND 200),
                 kind VARCHAR CHECK (kind IS NULL OR kind IN ('academic', 'professional', 'vocational', 'licence')),
                 framework VARCHAR CHECK (framework IS NULL OR length(btrim(framework)) BETWEEN 1 AND 40),
                 min_level INTEGER CHECK (min_level IS NULL OR min_level BETWEEN 0 AND 20),
                 recorded_by VARCHAR,
                 CHECK ((framework IS NULL) = (min_level IS NULL))
             )",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS role_qualification_requirements_by_role ON role_qualification_requirements (role_profile_pid)",
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
        let db = m.get_connection();
        db.execute_unprepared("DROP TABLE IF EXISTS role_qualification_requirements")
            .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS qualifications")
            .await?;
        Ok(())
    }
}
