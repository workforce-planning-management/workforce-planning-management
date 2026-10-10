//! Migration: a worker's own **contact details** (WPM-R124): home address, telephone numbers and
//! personal e-mail, kept by the worker themself.
//!
//! One row per worker, replaced as a whole when the worker changes it. The previous value is not
//! kept (minimisation); the audit entry records that it changed, never what it was. The table is
//! about one person, so it is exported with the worker and erased with them.

use sea_orm_migration::prelude::*;

/// The contact-details migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create `worker_contact_details`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared(
                "CREATE TABLE IF NOT EXISTS worker_contact_details (
                     created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                     updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                     id SERIAL PRIMARY KEY,
                     worker_pid UUID NOT NULL UNIQUE,
                     address_line1 VARCHAR NULL,
                     address_line2 VARCHAR NULL,
                     city VARCHAR NULL,
                     region VARCHAR NULL,
                     postcode VARCHAR NULL,
                     country VARCHAR NULL,
                     phone_mobile VARCHAR NULL,
                     phone_home VARCHAR NULL,
                     personal_email VARCHAR NULL,
                     recorded_by VARCHAR NULL,
                     on_behalf BOOLEAN NOT NULL DEFAULT FALSE
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
            .execute_unprepared("DROP TABLE IF EXISTS worker_contact_details")
            .await?;
        Ok(())
    }
}
