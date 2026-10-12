//! Migration: **cutover follow-up** (WPM-T117). A cutover may name its site more than one way
//! (`site_aliases`, a JSON list of text), and the early-warning table learns a third kind of
//! condition, a cutover whose people side is at risk.

use sea_orm_migration::prelude::*;

/// The follow-up migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Add `site_aliases` and widen the early-warning kinds.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        for sql in [
            "ALTER TABLE cutovers ADD COLUMN IF NOT EXISTS site_aliases JSONB NOT NULL DEFAULT '[]'::jsonb",
            "ALTER TABLE cutovers DROP CONSTRAINT IF EXISTS cutovers_site_aliases_list",
            "ALTER TABLE cutovers ADD CONSTRAINT cutovers_site_aliases_list CHECK (jsonb_typeof(site_aliases) = 'array')",
            "ALTER TABLE early_warnings DROP CONSTRAINT IF EXISTS early_warnings_kind_check",
            "ALTER TABLE early_warnings ADD CONSTRAINT early_warnings_kind_check
                 CHECK (kind IN ('pool_over_committed', 'measure_breach', 'cutover_at_risk'))",
        ] {
            db.execute_unprepared(sql).await?;
        }
        Ok(())
    }

    /// Put the earlier shape back.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        for sql in [
            "DELETE FROM early_warnings WHERE kind = 'cutover_at_risk'",
            "ALTER TABLE early_warnings DROP CONSTRAINT IF EXISTS early_warnings_kind_check",
            "ALTER TABLE early_warnings ADD CONSTRAINT early_warnings_kind_check
                 CHECK (kind IN ('pool_over_committed', 'measure_breach'))",
            "ALTER TABLE cutovers DROP CONSTRAINT IF EXISTS cutovers_site_aliases_list",
            "ALTER TABLE cutovers DROP COLUMN IF EXISTS site_aliases",
        ] {
            db.execute_unprepared(sql).await?;
        }
        Ok(())
    }
}
