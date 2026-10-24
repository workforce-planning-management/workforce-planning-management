//! Migration: **categories of workers** for pay reporting (WPM-R144, WPM-D99). A category is the employer's own
//! grouping of roles that do the same work or work of equal value, with the basis stated. It holds no personal
//! data: the roles are job titles, and who sits in a category is worked out when a report is read.

use sea_orm_migration::prelude::*;

/// The migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create `pay_categories`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS pay_categories (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 organization_ref VARCHAR NOT NULL,
                 name VARCHAR NOT NULL CHECK (length(btrim(name)) BETWEEN 1 AND 120),
                 basis TEXT NOT NULL CHECK (length(btrim(basis)) BETWEEN 10 AND 2000),
                 job_titles JSONB NOT NULL
                     CHECK (jsonb_typeof(job_titles) = 'array' AND jsonb_array_length(job_titles) BETWEEN 1 AND 50),
                 set_by VARCHAR,
                 deleted_at TIMESTAMPTZ
             )",
        )
        .await?;
        // One live category per name in an organization.
        db.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS pay_categories_org_name
                 ON pay_categories (organization_ref, lower(name)) WHERE deleted_at IS NULL",
        )
        .await?;
        // The retention sweep counts and deletes by `deleted_at` (WPM-R142).
        db.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS pay_categories_deleted_at_idx
                 ON pay_categories (deleted_at) WHERE deleted_at IS NOT NULL",
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
            .execute_unprepared("DROP TABLE IF EXISTS pay_categories")
            .await?;
        Ok(())
    }
}
