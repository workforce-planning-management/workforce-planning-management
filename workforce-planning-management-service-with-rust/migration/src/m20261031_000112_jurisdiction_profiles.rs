//! Migration: **jurisdiction profiles** (WPM-D144, WPM-T380): the codes an organization keeps (`jurisdictions`), the dated, sourced
//! parameter rows (`jurisdiction_parameters`, append-only), the place a worker's work is done (`workers.work_jurisdiction`), and a
//! week-rollup index that serves any working-time ceiling rather than one regime's.
//!
//! The values are a deployer's statements, not WPM's: the rules read them through `rules::jurisdiction::resolve`, and a parameter
//! with no row is unknown ("no limit configured"), never a default.

use sea_orm_migration::prelude::*;

/// The migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create the two tables, the append-only guard, the worker column, and swap the rollup's partial index.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        for sql in [
            // One row per code an organization keeps: its layer and the next wider code. The chain is current, not dated.
            "CREATE TABLE IF NOT EXISTS jurisdictions (
                 id SERIAL PRIMARY KEY,
                 organization_ref VARCHAR(255) NOT NULL,
                 code VARCHAR(7) NOT NULL CHECK (code ~ '^[A-Z0-9]{2,3}(-[A-Z0-9]{1,3})?$'),
                 layer VARCHAR(16) NOT NULL CHECK (layer IN ('supranational', 'national', 'subnational')),
                 parent_code VARCHAR(7) CHECK (parent_code ~ '^[A-Z0-9]{2,3}(-[A-Z0-9]{1,3})?$'),
                 name VARCHAR(100),
                 created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                 UNIQUE (organization_ref, code),
                 CHECK (parent_code IS NULL OR parent_code <> code)
             )",
            // One dated, sourced statement of one parameter for one code. Never updated or deleted: a change is a new row.
            "CREATE TABLE IF NOT EXISTS jurisdiction_parameters (
                 id BIGSERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 organization_ref VARCHAR(255) NOT NULL,
                 code VARCHAR(7) NOT NULL CHECK (code ~ '^[A-Z0-9]{2,3}(-[A-Z0-9]{1,3})?$'),
                 key VARCHAR(100) NOT NULL,
                 value JSONB NOT NULL,
                 effective_from DATE NOT NULL,
                 effective_to DATE,
                 source VARCHAR(500) NOT NULL CHECK (length(btrim(source)) > 0),
                 checked_on DATE,
                 status VARCHAR(16) NOT NULL CHECK (status IN ('example', 'unverified', 'read')),
                 created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                 CHECK (effective_to IS NULL OR effective_to >= effective_from),
                 CHECK (status <> 'read' OR checked_on IS NOT NULL)
             )",
            "CREATE INDEX IF NOT EXISTS jurisdiction_parameters_org
                 ON jurisdiction_parameters (organization_ref, code, key, effective_from)",
            "CREATE OR REPLACE FUNCTION jurisdiction_parameters_append_only() RETURNS trigger AS $$
             BEGIN
                 RAISE EXCEPTION 'jurisdiction_parameters is append-only: add a row with a later effective_from';
             END;
             $$ LANGUAGE plpgsql",
            "DROP TRIGGER IF EXISTS jurisdiction_parameters_append_only ON jurisdiction_parameters",
            "CREATE TRIGGER jurisdiction_parameters_append_only
                 BEFORE UPDATE OR DELETE ON jurisdiction_parameters
                 FOR EACH ROW EXECUTE FUNCTION jurisdiction_parameters_append_only()",
            // Where the work is done (a jurisdiction code), set by HR; not nationality, citizenship or residence.
            "ALTER TABLE workers ADD COLUMN IF NOT EXISTS work_jurisdiction VARCHAR(7)
                 CHECK (work_jurisdiction ~ '^[A-Z0-9]{2,3}(-[A-Z0-9]{1,3})?$')",
            // The partial index held one regime's floor (2,720 minutes: 48 h x 17 weeks over 18 calendar weeks). A ceiling is now a
            // profile's, so the floor is computed per query; this index serves any floor.
            "DROP INDEX IF EXISTS time_entry_weeks_heavy",
            "CREATE INDEX IF NOT EXISTS time_entry_weeks_week_minutes
                 ON time_entry_weeks (week_start, minutes, worker_pid)",
        ] {
            db.execute_unprepared(sql).await?;
        }
        Ok(())
    }

    /// Drop what `up` added and restore the old partial index.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        for sql in [
            "DROP INDEX IF EXISTS time_entry_weeks_week_minutes",
            "CREATE INDEX IF NOT EXISTS time_entry_weeks_heavy
                 ON time_entry_weeks (week_start, worker_pid) WHERE minutes >= 2720",
            "ALTER TABLE workers DROP COLUMN IF EXISTS work_jurisdiction",
            "DROP TABLE IF EXISTS jurisdiction_parameters",
            "DROP FUNCTION IF EXISTS jurisdiction_parameters_append_only()",
            "DROP TABLE IF EXISTS jurisdictions",
        ] {
            db.execute_unprepared(sql).await?;
        }
        Ok(())
    }
}
