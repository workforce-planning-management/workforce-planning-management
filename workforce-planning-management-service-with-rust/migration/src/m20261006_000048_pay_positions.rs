//! Migration: **pay positions**. A worker's current band and step on a
//! reference pay scale, and the date they reached that step (from which the
//! next step's eligibility date is computed). One row per worker; the scale,
//! band and step are validated by the service against the reference scale.

use sea_orm_migration::prelude::*;

/// The pay-positions migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create `worker_pay_positions`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared(
                "CREATE TABLE IF NOT EXISTS worker_pay_positions (
                     created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                     updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                     id SERIAL PRIMARY KEY,
                     worker_pid UUID NOT NULL UNIQUE,
                     scale_id VARCHAR NOT NULL,
                     band VARCHAR NOT NULL,
                     step INTEGER NOT NULL,
                     step_since DATE NOT NULL,
                     recorded_by VARCHAR NULL,
                     on_behalf BOOLEAN NOT NULL DEFAULT FALSE,
                     CHECK (step BETWEEN 1 AND 10)
                 )",
            )
            .await?;
        Ok(())
    }

    /// Drop it again.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS worker_pay_positions")
            .await?;
        Ok(())
    }
}
