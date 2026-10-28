//! Migration: **the framework's own level kept on declared skills and
//! requirements** (WPM-R111, WPM-D70). A declared skill gets the framework,
//! the level and the scale it was stated on; a requirement already had the
//! level and scale and now also names the framework, so a 4 of 7 is never
//! mistaken for a 4 of 4. WPM's 1–5 `proficiency` stays the figure every
//! comparison uses.

use sea_orm_migration::prelude::*;

/// The native-levels migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Add the columns.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        for column in [
            "source_framework VARCHAR NULL",
            "source_level INTEGER NULL",
            "source_scale_max INTEGER NULL",
        ] {
            conn.execute_unprepared(&format!(
                "ALTER TABLE worker_skills ADD COLUMN IF NOT EXISTS {column}"
            ))
            .await?;
        }
        conn.execute_unprepared(
            "ALTER TABLE role_skill_requirements ADD COLUMN IF NOT EXISTS source_framework VARCHAR NULL",
        )
        .await?;
        conn.execute_unprepared(
            "UPDATE role_skill_requirements SET source_framework = 'uk-gdad-pcf' \
             WHERE source_level IS NOT NULL AND source_framework IS NULL",
        )
        .await?;
        Ok(())
    }

    /// Drop the columns.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        conn.execute_unprepared(
            "ALTER TABLE role_skill_requirements DROP COLUMN IF EXISTS source_framework",
        )
        .await?;
        for column in ["source_framework", "source_level", "source_scale_max"] {
            conn.execute_unprepared(&format!(
                "ALTER TABLE worker_skills DROP COLUMN IF EXISTS {column}"
            ))
            .await?;
        }
        Ok(())
    }
}
