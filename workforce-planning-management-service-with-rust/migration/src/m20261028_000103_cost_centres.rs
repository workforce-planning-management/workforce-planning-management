//! Migration: **cost centres** (WPM-R163, WPM-D116). A register per organization: a code, a name, an optional parent and the days it
//! is valid. A cost centre is closed, never deleted. Assignments gain a `cost_centre` that names a code in the register (`NULL` is
//! unknown). The free-text `positions.cost_centre` is left as it is and listed as unmatched until HR resolves it.

use sea_orm_migration::prelude::*;

/// The migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create `cost_centres` and add `assignments.cost_centre`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS cost_centres (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 organization_ref VARCHAR NOT NULL,
                 code VARCHAR NOT NULL CHECK (length(btrim(code)) BETWEEN 1 AND 20),
                 name VARCHAR NOT NULL CHECK (length(btrim(name)) BETWEEN 1 AND 120),
                 parent_code VARCHAR CHECK (parent_code IS NULL OR parent_code <> code),
                 valid_from DATE NOT NULL,
                 valid_to DATE,
                 recorded_by VARCHAR,
                 CHECK (valid_to IS NULL OR valid_to >= valid_from),
                 UNIQUE (organization_ref, code)
             )",
        )
        .await?;
        db.execute_unprepared(
            "ALTER TABLE assignments ADD COLUMN IF NOT EXISTS cost_centre VARCHAR
                 CHECK (cost_centre IS NULL OR length(btrim(cost_centre)) BETWEEN 1 AND 20)",
        )
        .await?;
        Ok(())
    }

    /// Drop the column and the table.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        db.execute_unprepared("ALTER TABLE assignments DROP COLUMN IF EXISTS cost_centre")
            .await?;
        db.execute_unprepared("DROP TABLE IF EXISTS cost_centres")
            .await?;
        Ok(())
    }
}
