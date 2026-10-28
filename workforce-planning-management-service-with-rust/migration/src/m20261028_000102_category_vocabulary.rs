//! Migration: **assignment category vocabularies** (WPM-R162, WPM-D116). Per organization, each entry is a code and a label and,
//! where it comes from a national standard, that standard's name and its own code. A job role may say which staff group it belongs
//! to, so the two can be checked for agreement. The codes are configuration the deployer imports; none is shipped.

use sea_orm_migration::prelude::*;

/// The migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create `category_vocabulary`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared(
                "CREATE TABLE IF NOT EXISTS category_vocabulary (
                     created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                     updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                     id SERIAL PRIMARY KEY,
                     pid UUID NOT NULL UNIQUE,
                     organization_ref VARCHAR NOT NULL,
                     kind VARCHAR NOT NULL
                         CHECK (kind IN ('site', 'area_of_work', 'job_role', 'staff_group', 'occupation_code')),
                     code VARCHAR NOT NULL CHECK (length(btrim(code)) BETWEEN 1 AND 60),
                     label VARCHAR NOT NULL CHECK (length(btrim(label)) BETWEEN 1 AND 120),
                     standard VARCHAR CHECK (standard IS NULL OR length(btrim(standard)) BETWEEN 1 AND 60),
                     standard_code VARCHAR CHECK (standard_code IS NULL OR length(btrim(standard_code)) BETWEEN 1 AND 60),
                     staff_group VARCHAR CHECK (staff_group IS NULL OR (kind = 'job_role' AND length(btrim(staff_group)) BETWEEN 1 AND 60)),
                     recorded_by VARCHAR,
                     UNIQUE (organization_ref, kind, code)
                 )",
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
            .execute_unprepared("DROP TABLE IF EXISTS category_vocabulary")
            .await?;
        Ok(())
    }
}
