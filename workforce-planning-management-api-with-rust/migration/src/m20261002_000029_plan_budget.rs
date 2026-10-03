//! Migration: a workforce plan's financial assumptions — an optional annual
//! budget (minor units + ISO-4217 currency) and an employer on-cost
//! percentage — so a plan can be costed and compared with what is affordable.

use sea_orm_migration::prelude::*;

/// The plan-budget migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Add the three nullable columns.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        for column in [
            "budget_minor BIGINT NULL",
            "budget_currency VARCHAR NULL",
            "on_cost_bp INTEGER NULL",
        ] {
            conn.execute_unprepared(&format!(
                "ALTER TABLE workforce_plans ADD COLUMN IF NOT EXISTS {column}"
            ))
            .await?;
        }
        Ok(())
    }

    /// Drop the columns.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        for column in ["budget_minor", "budget_currency", "on_cost_bp"] {
            conn.execute_unprepared(&format!(
                "ALTER TABLE workforce_plans DROP COLUMN IF EXISTS {column}"
            ))
            .await?;
        }
        Ok(())
    }
}
