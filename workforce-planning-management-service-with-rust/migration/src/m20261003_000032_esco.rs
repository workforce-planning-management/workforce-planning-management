//! Migration: a pinned local copy of **ESCO** (`spec/esco/index.md`) — its
//! occupations, skills, and the essential/optional relations between them —
//! loaded from a CSV download by `import_esco`. ESCO is a classification WPM
//! *references*; the catalogue and role profiles stay WPM's own. The copy is
//! replaced wholesale on each import so it is reproducible for a given
//! ESCO version.

use sea_orm_migration::prelude::*;

/// The ESCO migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create the three ESCO tables + indexes.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        conn.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS esco_skills (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 uri VARCHAR NOT NULL UNIQUE,
                 label VARCHAR NOT NULL,
                 skill_type VARCHAR NULL,
                 reuse_level VARCHAR NULL
             )",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS esco_skills_label ON esco_skills (lower(label))",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS esco_occupations (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 uri VARCHAR NOT NULL UNIQUE,
                 label VARCHAR NOT NULL,
                 isco_code VARCHAR NULL,
                 description VARCHAR NULL
             )",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS esco_occupations_label ON esco_occupations (lower(label))",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS esco_occupation_skills (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 occupation_uri VARCHAR NOT NULL,
                 skill_uri VARCHAR NOT NULL,
                 relation VARCHAR NOT NULL
             )",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS esco_occupation_skills_key \
             ON esco_occupation_skills (occupation_uri, skill_uri)",
        )
        .await?;
        Ok(())
    }

    /// Drop the three tables.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        for table in ["esco_occupation_skills", "esco_occupations", "esco_skills"] {
            conn.execute_unprepared(&format!("DROP TABLE IF EXISTS {table}"))
                .await?;
        }
        Ok(())
    }
}
