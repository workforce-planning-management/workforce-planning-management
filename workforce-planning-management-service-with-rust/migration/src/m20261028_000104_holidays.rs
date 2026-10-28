//! Migration: **public holidays** (WPM-R5). One row per organization, region and holiday: the date the rule gives, the day it is
//! taken (a weekend date may move to a weekday), and whether it was generated from a region's rules or added by hand. Re-running a
//! generation changes no row it did not make.

use sea_orm_migration::prelude::*;

/// The migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create `holidays`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS holidays (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 organization_ref VARCHAR NOT NULL,
                 region VARCHAR NOT NULL CHECK (region ~ '^[A-Z0-9]{2,3}(-[A-Z0-9]{1,3})?$'),
                 year INTEGER NOT NULL CHECK (year BETWEEN 1970 AND 2100),
                 name VARCHAR NOT NULL CHECK (length(btrim(name)) BETWEEN 1 AND 120),
                 natural_on DATE NOT NULL,
                 observed_on DATE NOT NULL,
                 source VARCHAR NOT NULL CHECK (source IN ('generated', 'manual')),
                 recorded_by VARCHAR,
                 UNIQUE (organization_ref, region, natural_on, name)
             )",
        )
        .await?;
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS holidays_by_region_year ON holidays (organization_ref, region, year)",
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
            .execute_unprepared("DROP TABLE IF EXISTS holidays")
            .await?;
        Ok(())
    }
}
