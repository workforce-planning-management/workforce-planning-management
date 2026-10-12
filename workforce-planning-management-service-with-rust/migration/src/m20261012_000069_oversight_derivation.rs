//! Migration: **derived oversight measures** (WPM-R67, WPM-T114). A derived measure names which
//! computation supplies its readings; a reported measure names none. The pairing is a CHECK, so a
//! derived measure cannot exist without its derivation and a reported one cannot carry a stale key.

use sea_orm_migration::prelude::*;

/// The derivation migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Add `derivation` and its pairing with `origin`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        for sql in [
            "ALTER TABLE oversight_measures ADD COLUMN IF NOT EXISTS derivation VARCHAR",
            "ALTER TABLE oversight_measures DROP CONSTRAINT IF EXISTS oversight_measures_derivation_pairs",
            "ALTER TABLE oversight_measures ADD CONSTRAINT oversight_measures_derivation_pairs \
             CHECK ((origin = 'derived') = (derivation IS NOT NULL))",
        ] {
            db.execute_unprepared(sql).await?;
        }
        Ok(())
    }

    /// Drop the column.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        db.execute_unprepared(
            "ALTER TABLE oversight_measures DROP CONSTRAINT IF EXISTS oversight_measures_derivation_pairs",
        )
        .await?;
        db.execute_unprepared("ALTER TABLE oversight_measures DROP COLUMN IF EXISTS derivation")
            .await?;
        Ok(())
    }
}
