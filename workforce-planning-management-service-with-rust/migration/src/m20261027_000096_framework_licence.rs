//! Migration: **the framework licence record** (WPM-R113) — the version
//! imported, whether its wording may be redistributed, the licence holder,
//! the licence reference, when the licence was last checked, and (already
//! present) the attribution. `redistribution` defaults to `unknown`, which
//! the pure `rules::framework::publishable` treats as a no. The two
//! frameworks already imported are open (OGL v3.0; Commission Decision
//! 2011/833/EU) and are set so.

use sea_orm_migration::prelude::*;

/// The framework-licence migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Add the licence columns and mark the two open frameworks.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        for column in [
            "version VARCHAR NULL",
            "redistribution VARCHAR NOT NULL DEFAULT 'unknown'",
            "licence_holder VARCHAR NULL",
            "licence_reference VARCHAR NULL",
            "licence_checked_on DATE NULL",
        ] {
            conn.execute_unprepared(&format!(
                "ALTER TABLE capability_frameworks ADD COLUMN IF NOT EXISTS {column}"
            ))
            .await?;
        }
        conn.execute_unprepared(
            "ALTER TABLE capability_frameworks DROP CONSTRAINT IF EXISTS \
             capability_frameworks_redistribution_chk",
        )
        .await?;
        conn.execute_unprepared(
            "ALTER TABLE capability_frameworks ADD CONSTRAINT \
             capability_frameworks_redistribution_chk \
             CHECK (redistribution IN ('open', 'internal_only', 'unknown'))",
        )
        .await?;
        conn.execute_unprepared(
            "UPDATE capability_frameworks SET redistribution = 'open' \
             WHERE slug IN ('uk-gdad-pcf', 'esco') AND redistribution = 'unknown'",
        )
        .await?;
        Ok(())
    }

    /// Drop the licence columns.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        conn.execute_unprepared(
            "ALTER TABLE capability_frameworks DROP CONSTRAINT IF EXISTS \
             capability_frameworks_redistribution_chk",
        )
        .await?;
        for column in [
            "version",
            "redistribution",
            "licence_holder",
            "licence_reference",
            "licence_checked_on",
        ] {
            conn.execute_unprepared(&format!(
                "ALTER TABLE capability_frameworks DROP COLUMN IF EXISTS {column}"
            ))
            .await?;
        }
        Ok(())
    }
}
