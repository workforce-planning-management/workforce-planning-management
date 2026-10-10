//! Migration: **generic pay scale id**. The one reference pay scale is now
//! `national-2026-27`. Until this change the service knew exactly one scale, and
//! it validated every stored id against it, so every stored id is the old id of
//! that same scale: each is moved to the new id. A worker's pay position and a
//! role profile's pay band keep their band and step.

use sea_orm_migration::prelude::*;

/// The generic-pay-scale-id migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

/// The id of the one scale the service knows.
const SCALE_ID: &str = "national-2026-27";

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Move every stored scale id to [`SCALE_ID`].
    ///
    /// # Errors
    ///
    /// Propagates any SQL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        conn.execute_unprepared(&format!(
            "UPDATE worker_pay_positions SET scale_id = '{SCALE_ID}' \
             WHERE scale_id <> '{SCALE_ID}'"
        ))
        .await?;
        conn.execute_unprepared(&format!(
            "UPDATE role_profiles SET pay_scale_id = '{SCALE_ID}' \
             WHERE pay_scale_id IS NOT NULL AND pay_scale_id <> '{SCALE_ID}'"
        ))
        .await?;
        Ok(())
    }

    /// Nothing to undo: the old id is not restored, and the rows stay valid
    /// against the service's one scale.
    ///
    /// # Errors
    ///
    /// None.
    async fn down(&self, _m: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
