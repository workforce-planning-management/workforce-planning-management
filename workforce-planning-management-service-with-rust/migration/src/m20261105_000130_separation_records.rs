//! Migration: **the separation record** (WPM-R222, WPM-T422). The last day and the reason stay on the leaver record; this holds only
//! what that record does not: the final pay date and a reference to the pay summary, which is a pointer and never an amount.

use sea_orm_migration::prelude::*;

/// The migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create the table.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        for statement in [
            "CREATE TABLE IF NOT EXISTS separation_records (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 movement_pid UUID NOT NULL UNIQUE,
                 worker_pid UUID NOT NULL,
                 final_pay_on DATE,
                 pay_summary_ref VARCHAR,
                 recorded_by VARCHAR
             )",
            "CREATE INDEX IF NOT EXISTS separation_records_worker ON separation_records (worker_pid)",
        ] {
            db.execute_unprepared(statement).await?;
        }
        Ok(())
    }

    /// Drop the table.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS separation_records")
            .await?;
        Ok(())
    }
}
