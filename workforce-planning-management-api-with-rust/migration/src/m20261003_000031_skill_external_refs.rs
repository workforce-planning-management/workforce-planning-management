//! Migration: `skill_external_refs` — how a catalogue skill is known in an
//! external framework: its exact name in the UK GDAD PCF, its concept URI in
//! ESCO. One reference per skill per framework, and one skill per reference,
//! so an import matches by reference (surviving a rename) rather than by
//! name. WPM keeps its own skills catalogue; this is a *reference*, never a
//! second skills model.

use sea_orm_migration::prelude::*;

/// The skill-external-refs migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create the table + its uniqueness keys.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        conn.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS skill_external_refs (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 skill_pid UUID NOT NULL,
                 framework_slug VARCHAR NOT NULL,
                 ref VARCHAR NOT NULL,
                 label VARCHAR NULL,
                 version VARCHAR NULL
             )",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS skill_external_refs_skill_key \
             ON skill_external_refs (skill_pid, framework_slug)",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS skill_external_refs_ref_key \
             ON skill_external_refs (framework_slug, ref)",
        )
        .await?;
        Ok(())
    }

    /// Drop the table.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS skill_external_refs")
            .await?;
        Ok(())
    }
}
