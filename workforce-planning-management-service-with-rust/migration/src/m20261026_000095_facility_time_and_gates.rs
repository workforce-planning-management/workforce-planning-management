//! Migration: **facility time as an aggregate** and **decision gates** (WPM-R107, WPM-R108, WPM-D68). Facility time is the hours released
//! for representative duties, recorded as **minutes per body per period**: there is no column for a person and no kind of time entry for it,
//! so a budget can be agreed without making a list of representatives. A decision gate says that a kind of change needs consultation with a body.

use sea_orm_migration::prelude::*;

/// The migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create `facility_time_periods` and `decision_gates`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS facility_time_periods (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 body_pid UUID NOT NULL,
                 period_start DATE NOT NULL,
                 period_end DATE NOT NULL,
                 minutes INTEGER NOT NULL CHECK (minutes BETWEEN 0 AND 6000000),
                 budget_minutes INTEGER CHECK (budget_minutes IS NULL OR budget_minutes BETWEEN 0 AND 6000000),
                 recorded_by VARCHAR,
                 CHECK (period_end >= period_start),
                 UNIQUE (body_pid, period_start, period_end)
             )",
        )
        .await?;
        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS decision_gates (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 organization_ref VARCHAR NOT NULL,
                 change_kind VARCHAR NOT NULL CHECK (change_kind IN ('redundancy_proposal', 'shift_pattern_change', 'new_monitoring', 'new_or_changed_score', 'pay_structure_change', 'other')),
                 body_pid UUID NOT NULL,
                 note VARCHAR CHECK (note IS NULL OR length(note) <= 200),
                 recorded_by VARCHAR,
                 UNIQUE (organization_ref, change_kind, body_pid)
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
        let db = m.get_connection();
        db.execute_unprepared("DROP TABLE IF EXISTS decision_gates")
            .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS facility_time_periods")
            .await?;
        Ok(())
    }
}
