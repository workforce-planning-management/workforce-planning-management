//! Migration: **the supplier's contact** on a contractor's details (WPM-R206). A `person:` URN (the contact-relationship-management
//! app keys its contacts by `person_ref`). WPM stores the reference and nothing about the contact: the name is looked up from the owner.

use sea_orm_migration::prelude::*;

/// The migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Add the column.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared(
                "ALTER TABLE worker_contractor_details ADD COLUMN IF NOT EXISTS supplier_contact_ref VARCHAR NULL
                     CHECK (supplier_contact_ref IS NULL OR supplier_contact_ref ~ '^person:[0-9a-f-]{36}$')",
            )
            .await?;
        Ok(())
    }

    /// Drop the column.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared(
                "ALTER TABLE worker_contractor_details DROP COLUMN IF EXISTS supplier_contact_ref",
            )
            .await?;
        Ok(())
    }
}
