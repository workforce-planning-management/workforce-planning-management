//! Migration: **positions** (WPM-R152, WPM-D104). A position is a budgeted seat that exists whether or not anyone holds it: a cost
//! centre, a budget in FTE percent, a funding kind and a state. Who holds it, the requisition that fills it and the plan demand line
//! it came from are links by pid; nothing is copied. **There is no column for pay.** A position is never deleted; it is abolished.

use sea_orm_migration::prelude::*;

/// The migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create `positions`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS positions (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 organization_ref VARCHAR NOT NULL,
                 cost_centre VARCHAR NOT NULL CHECK (length(btrim(cost_centre)) BETWEEN 1 AND 80),
                 title VARCHAR NOT NULL CHECK (length(btrim(title)) BETWEEN 1 AND 120),
                 budget_fte_percent INTEGER NOT NULL CHECK (budget_fte_percent BETWEEN 1 AND 100),
                 funding_kind VARCHAR NOT NULL CHECK (funding_kind IN ('core', 'time_limited', 'external', 'programme')),
                 funding_ends_on DATE,
                 programme_ref VARCHAR,
                 state VARCHAR NOT NULL DEFAULT 'planned'
                     CHECK (state IN ('planned', 'approved', 'open', 'filled', 'frozen', 'abolished')),
                 holder_worker_pid UUID,
                 requisition_pid UUID,
                 demand_line_pid UUID,
                 recorded_by VARCHAR,
                 CHECK (funding_kind NOT IN ('time_limited', 'external') OR funding_ends_on IS NOT NULL),
                 CHECK ((funding_kind = 'programme') = (programme_ref IS NOT NULL))
             )",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS positions_by_organization ON positions (organization_ref, cost_centre)",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS positions_by_holder ON positions (holder_worker_pid) WHERE holder_worker_pid IS NOT NULL",
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
            .execute_unprepared("DROP TABLE IF EXISTS positions")
            .await?;
        Ok(())
    }
}
