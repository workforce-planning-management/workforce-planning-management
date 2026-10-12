//! Migration: **oversight** (WPM-R65, WPM-R66, WPM-D48, WPM-D49). An organization's own scale of
//! levels, its position history, the exit criteria of each level, the measures behind each criterion
//! (threshold, direction, sustain rule) and the readings entered for them. Nothing is pre-loaded in
//! code; a synthetic demo scale is added only by the seed task and is labelled synthetic.

use sea_orm_migration::prelude::*;

/// The oversight migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create the five tables.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        for sql in [
            "CREATE TABLE IF NOT EXISTS oversight_levels (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 organization_ref VARCHAR NOT NULL,
                 ordinal INTEGER NOT NULL,
                 name VARCHAR NOT NULL,
                 synthetic BOOLEAN NOT NULL DEFAULT FALSE,
                 UNIQUE (organization_ref, ordinal),
                 CHECK (ordinal BETWEEN 1 AND 20)
             )",
            "CREATE TABLE IF NOT EXISTS oversight_positions (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 organization_ref VARCHAR NOT NULL,
                 level_pid UUID NOT NULL,
                 effective_on DATE NOT NULL,
                 deciding_body VARCHAR NOT NULL,
                 reason VARCHAR NOT NULL,
                 recorded_by VARCHAR NULL
             )",
            "CREATE TABLE IF NOT EXISTS oversight_criteria (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 level_pid UUID NOT NULL,
                 name VARCHAR NOT NULL,
                 synthetic BOOLEAN NOT NULL DEFAULT FALSE
             )",
            "CREATE TABLE IF NOT EXISTS oversight_measures (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 criterion_pid UUID NOT NULL,
                 name VARCHAR NOT NULL,
                 origin VARCHAR NOT NULL,
                 unit VARCHAR NOT NULL,
                 threshold BIGINT NOT NULL,
                 direction VARCHAR NOT NULL,
                 sustain INTEGER NOT NULL,
                 CHECK (origin IN ('derived', 'reported')),
                 CHECK (direction IN ('at_most', 'at_least')),
                 CHECK (sustain BETWEEN 1 AND 120)
             )",
            "CREATE TABLE IF NOT EXISTS oversight_readings (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 measure_pid UUID NOT NULL,
                 period_on DATE NOT NULL,
                 value BIGINT NULL,
                 evidence VARCHAR NULL,
                 source_document VARCHAR NULL,
                 recorded_by VARCHAR NULL,
                 UNIQUE (measure_pid, period_on),
                 CHECK (EXTRACT(DAY FROM period_on) = 1)
             )",
            "CREATE INDEX IF NOT EXISTS oversight_positions_org ON oversight_positions (organization_ref, effective_on)",
        ] {
            db.execute_unprepared(sql).await?;
        }
        Ok(())
    }

    /// Drop them again.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        for t in [
            "oversight_readings",
            "oversight_measures",
            "oversight_criteria",
            "oversight_positions",
            "oversight_levels",
        ] {
            m.get_connection()
                .execute_unprepared(&format!("DROP TABLE IF EXISTS {t}"))
                .await?;
        }
        Ok(())
    }
}
