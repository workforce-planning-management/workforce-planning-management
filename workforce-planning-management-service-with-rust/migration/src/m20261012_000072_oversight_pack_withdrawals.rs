//! Migration: **withdrawing a monthly pack** (WPM-R69, WPM-T116). A frozen pack is never changed or
//! removed, so a mistaken one is *withdrawn*: a separate, append-only row says so and why, and every
//! reader sees the mark beside the unchanged payload.

use sea_orm_migration::prelude::*;

/// The withdrawal migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create `oversight_pack_withdrawals`, append-only.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        for sql in [
            "CREATE TABLE IF NOT EXISTS oversight_pack_withdrawals (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 pack_pid UUID NOT NULL UNIQUE,
                 withdrawn_on DATE NOT NULL,
                 reason VARCHAR NOT NULL,
                 CHECK (length(btrim(reason)) > 0)
             )",
            "DROP TRIGGER IF EXISTS oversight_pack_withdrawals_no_change ON oversight_pack_withdrawals",
            "CREATE TRIGGER oversight_pack_withdrawals_no_change BEFORE UPDATE OR DELETE ON oversight_pack_withdrawals
                 FOR EACH ROW EXECUTE FUNCTION oversight_packs_append_only()",
        ] {
            db.execute_unprepared(sql).await?;
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
            .execute_unprepared("DROP TABLE IF EXISTS oversight_pack_withdrawals")
            .await?;
        Ok(())
    }
}
