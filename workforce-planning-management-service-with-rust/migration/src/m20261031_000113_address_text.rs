//! Migration: a worker's address as **one free-text block** (WPM-D152, WPM-T382), beside the structured fields that
//! stay. The UI offers the block for a country its address order and label table lacks; the person writes the
//! address in their own order. Nothing existing is rewritten: a row's structured fields are kept as they are.

use sea_orm_migration::prelude::*;

/// The migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Add `worker_contact_details.address_text`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared(
                "ALTER TABLE worker_contact_details ADD COLUMN IF NOT EXISTS address_text TEXT NULL",
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
            .execute_unprepared(
                "ALTER TABLE worker_contact_details DROP COLUMN IF EXISTS address_text",
            )
            .await?;
        Ok(())
    }
}
