//! Migration: **grades**. A worker's current job level (one row per worker; the
//! level framework and number are validated by the service against the
//! reference ladder, not by the database), and an optional job level and pay
//! band on a role profile.

use sea_orm_migration::prelude::*;

/// The grades migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create `worker_job_levels`; add the grade columns to `role_profiles`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        conn.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS worker_job_levels (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 worker_pid UUID NOT NULL UNIQUE,
                 framework VARCHAR NOT NULL,
                 level_number INTEGER NOT NULL,
                 effective_on DATE NOT NULL,
                 recorded_by VARCHAR NULL,
                 on_behalf BOOLEAN NOT NULL DEFAULT FALSE,
                 CHECK (level_number BETWEEN 1 AND 100)
             )",
        )
        .await?;
        for column in [
            "job_level_framework VARCHAR NULL",
            "job_level INTEGER NULL",
            "pay_scale_id VARCHAR NULL",
            "pay_band VARCHAR NULL",
        ] {
            conn.execute_unprepared(&format!(
                "ALTER TABLE role_profiles ADD COLUMN IF NOT EXISTS {column}"
            ))
            .await?;
        }
        // A framework and its level, or a scale and its band, travel together.
        conn.execute_unprepared(
            "ALTER TABLE role_profiles DROP CONSTRAINT IF EXISTS role_profiles_grade_pairs",
        )
        .await?;
        conn.execute_unprepared(
            "ALTER TABLE role_profiles ADD CONSTRAINT role_profiles_grade_pairs CHECK (
                 (job_level_framework IS NULL) = (job_level IS NULL)
                 AND (pay_scale_id IS NULL) = (pay_band IS NULL)
             )",
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
        conn.execute_unprepared(
            "ALTER TABLE role_profiles DROP CONSTRAINT IF EXISTS role_profiles_grade_pairs",
        )
        .await?;
        for column in [
            "job_level_framework",
            "job_level",
            "pay_scale_id",
            "pay_band",
        ] {
            conn.execute_unprepared(&format!(
                "ALTER TABLE role_profiles DROP COLUMN IF EXISTS {column}"
            ))
            .await?;
        }
        conn.execute_unprepared("DROP TABLE IF EXISTS worker_job_levels")
            .await?;
        Ok(())
    }
}
