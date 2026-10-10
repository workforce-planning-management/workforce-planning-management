//! Migration: the AI-driven change tracker — `change_initiatives` (an
//! automation / AI / process change with a lifecycle), the roles it
//! displaces, reshapes, or creates (`initiative_role_impacts`), and the
//! skills whose demand it moves (`initiative_skill_shifts`). It tracks
//! roles and skills, never people (WPM-D28).

use sea_orm_migration::prelude::*;

/// The change-initiatives migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create the three tables + uniqueness keys.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        conn.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS change_initiatives (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 name VARCHAR NOT NULL,
                 description VARCHAR NULL,
                 kind VARCHAR NOT NULL,
                 status VARCHAR NOT NULL,
                 starts_on DATE NULL,
                 deleted_at TIMESTAMPTZ NULL
             )",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS initiative_role_impacts (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 initiative_pid UUID NOT NULL,
                 role_profile_pid UUID NOT NULL,
                 impact VARCHAR NOT NULL,
                 timeframe VARCHAR NOT NULL,
                 note VARCHAR NULL
             )",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS initiative_role_impacts_key \
             ON initiative_role_impacts (initiative_pid, role_profile_pid)",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS initiative_skill_shifts (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 initiative_pid UUID NOT NULL,
                 skill_pid UUID NOT NULL,
                 direction VARCHAR NOT NULL,
                 note VARCHAR NULL
             )",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS initiative_skill_shifts_key \
             ON initiative_skill_shifts (initiative_pid, skill_pid)",
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
        for table in [
            "initiative_skill_shifts",
            "initiative_role_impacts",
            "change_initiatives",
        ] {
            conn.execute_unprepared(&format!("DROP TABLE IF EXISTS {table}"))
                .await?;
        }
        Ok(())
    }
}
