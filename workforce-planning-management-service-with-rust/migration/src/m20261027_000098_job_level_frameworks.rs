//! Migration: **job-level frameworks in the database** (WPM-R110). Job levels
//! were coded reference data (Google's L3–L11); an imported ladder, SFIA's
//! seven levels of responsibility, has the same shape here. `redistribution`
//! follows the framework licence guard (WPM-R113): `unknown` unless the
//! importer says otherwise. A worker's or role's level stays
//! `(framework, level number)`, so nothing about them changes.

use sea_orm_migration::prelude::*;

/// The job-level-frameworks migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create the two tables.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        conn.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS job_level_frameworks (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 slug VARCHAR NOT NULL UNIQUE,
                 name VARCHAR NOT NULL,
                 organization VARCHAR NOT NULL,
                 track VARCHAR NOT NULL,
                 source VARCHAR NOT NULL,
                 version VARCHAR NULL,
                 redistribution VARCHAR NOT NULL DEFAULT 'unknown'
                     CHECK (redistribution IN ('open', 'internal_only', 'unknown'))
             )",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS job_level_levels (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 framework VARCHAR NOT NULL,
                 number INTEGER NOT NULL CHECK (number BETWEEN 1 AND 20),
                 code VARCHAR NOT NULL,
                 title VARCHAR NOT NULL,
                 summary VARCHAR NOT NULL,
                 attributes VARCHAR NULL,
                 UNIQUE (framework, number)
             )",
        )
        .await?;
        Ok(())
    }

    /// Drop the tables.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        conn.execute_unprepared("DROP TABLE IF EXISTS job_level_levels")
            .await?;
        conn.execute_unprepared("DROP TABLE IF EXISTS job_level_frameworks")
            .await?;
        Ok(())
    }
}
