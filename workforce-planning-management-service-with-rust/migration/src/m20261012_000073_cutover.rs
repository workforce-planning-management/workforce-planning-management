//! Migration: **cutover readiness** (WPM-R70, WPM-T117). A cutover is a programme's go-live at a site
//! over a window of dates. It lists the roles that must be staffed with trained people (each with the
//! course that trains it and the headcount), the skill pools it depends on, and optionally the on-call
//! rota that gives hypercare cover. Nothing here names a person.

use sea_orm_migration::prelude::*;

/// The cutover migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create the three tables.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        for sql in [
            "CREATE TABLE IF NOT EXISTS cutovers (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 organization_ref VARCHAR NOT NULL,
                 programme_ref VARCHAR NOT NULL,
                 name VARCHAR NOT NULL,
                 site VARCHAR NOT NULL,
                 window_start DATE NOT NULL,
                 window_end DATE NOT NULL,
                 rota_pid UUID,
                 CHECK (window_end >= window_start),
                 CHECK (length(btrim(name)) > 0),
                 CHECK (length(btrim(site)) > 0)
             )",
            "CREATE INDEX IF NOT EXISTS cutovers_org ON cutovers (organization_ref)",
            "CREATE TABLE IF NOT EXISTS cutover_roles (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 cutover_pid UUID NOT NULL,
                 role_profile_pid UUID NOT NULL,
                 course_ref VARCHAR NOT NULL,
                 required_headcount INTEGER NOT NULL,
                 CHECK (required_headcount > 0),
                 CHECK (length(btrim(course_ref)) > 0),
                 UNIQUE (cutover_pid, role_profile_pid)
             )",
            "CREATE TABLE IF NOT EXISTS cutover_pools (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 cutover_pid UUID NOT NULL,
                 pool_pid UUID NOT NULL,
                 UNIQUE (cutover_pid, pool_pid)
             )",
        ] {
            db.execute_unprepared(sql).await?;
        }
        Ok(())
    }

    /// Drop the three tables.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        for t in ["cutover_pools", "cutover_roles", "cutovers"] {
            m.get_connection()
                .execute_unprepared(&format!("DROP TABLE IF EXISTS {t}"))
                .await?;
        }
        Ok(())
    }
}
