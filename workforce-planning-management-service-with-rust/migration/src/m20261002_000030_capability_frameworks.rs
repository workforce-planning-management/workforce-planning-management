//! Migration: **capability frameworks** — a framework a set of role
//! profiles was imported from (the UK GDAD Profession Capability Framework
//! first), with its attribution and scale; and the provenance columns that
//! tie a role profile to its framework, profession, role, and level, and a
//! requirement to the level it was stated at on the framework's own scale.
//! The source level is kept next to WPM's 1–5 `min_proficiency` so the
//! conversion is visible and never pretends two scales are one.

use sea_orm_migration::prelude::*;

/// The capability-frameworks migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create `capability_frameworks` and add provenance columns.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        conn.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS capability_frameworks (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 slug VARCHAR NOT NULL UNIQUE,
                 name VARCHAR NOT NULL,
                 source_url VARCHAR NULL,
                 licence VARCHAR NULL,
                 attribution VARCHAR NULL,
                 scale_max INTEGER NOT NULL,
                 scale_labels VARCHAR NULL,
                 imported_on DATE NOT NULL,
                 note VARCHAR NULL
             )",
        )
        .await?;
        for column in [
            "framework_slug VARCHAR NULL",
            "external_ref VARCHAR NULL",
            "profession VARCHAR NULL",
            "role_name VARCHAR NULL",
            "level_name VARCHAR NULL",
            "level_order INTEGER NULL",
            "management_track BOOLEAN NOT NULL DEFAULT false",
        ] {
            conn.execute_unprepared(&format!(
                "ALTER TABLE role_profiles ADD COLUMN IF NOT EXISTS {column}"
            ))
            .await?;
        }
        conn.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS role_profiles_framework_ref \
             ON role_profiles (framework_slug, external_ref) \
             WHERE framework_slug IS NOT NULL AND deleted_at IS NULL",
        )
        .await?;
        for column in ["source_level INTEGER NULL", "source_scale_max INTEGER NULL"] {
            conn.execute_unprepared(&format!(
                "ALTER TABLE role_skill_requirements ADD COLUMN IF NOT EXISTS {column}"
            ))
            .await?;
        }
        Ok(())
    }

    /// Drop the table and columns.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        conn.execute_unprepared("DROP INDEX IF EXISTS role_profiles_framework_ref")
            .await?;
        for column in ["source_level", "source_scale_max"] {
            conn.execute_unprepared(&format!(
                "ALTER TABLE role_skill_requirements DROP COLUMN IF EXISTS {column}"
            ))
            .await?;
        }
        for column in [
            "framework_slug",
            "external_ref",
            "profession",
            "role_name",
            "level_name",
            "level_order",
            "management_track",
        ] {
            conn.execute_unprepared(&format!(
                "ALTER TABLE role_profiles DROP COLUMN IF EXISTS {column}"
            ))
            .await?;
        }
        conn.execute_unprepared("DROP TABLE IF EXISTS capability_frameworks")
            .await?;
        Ok(())
    }
}
