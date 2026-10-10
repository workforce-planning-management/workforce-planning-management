//! Migration: **scrub the reason from sick-leave requests** (WPM-R118, WPM-D73).
//!
//! A sick-leave request records that it is sick leave and its dates, never why: a diagnosis is
//! special-category health data the service has no need to hold, and the field was free text.
//! The service now refuses a reason on sick leave; this migration removes the ones already
//! stored. It is **one-way**: the text is gone, and `down` cannot restore it. Other kinds of
//! leave keep their reason. Take a backup first, as for every upgrade.

use sea_orm_migration::prelude::*;

/// The scrub migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Remove the reason from every sick-leave request.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared(
                "UPDATE leave_requests SET reason = NULL WHERE kind = 'sick' AND reason IS NOT NULL",
            )
            .await?;
        Ok(())
    }

    /// Nothing to undo: the scrubbed text cannot be restored.
    ///
    /// # Errors
    ///
    /// Never.
    async fn down(&self, _m: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
