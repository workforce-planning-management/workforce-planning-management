//! Migration: **a plan's target organization** (WPM-R153). The roles a plan intends the organization to have, each with a
//! headcount and a parent role. It is made of roles, never of people: there is no worker column, so a restructure cannot be
//! drawn onto named individuals here. The live structure is read from workers when the two are compared and is not copied.

use sea_orm_migration::prelude::*;

/// The migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create `plan_org_nodes`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let db = m.get_connection();
        db.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS plan_org_nodes (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 plan_pid UUID NOT NULL REFERENCES workforce_plans (pid) ON DELETE CASCADE,
                 node_key VARCHAR NOT NULL CHECK (length(node_key) BETWEEN 1 AND 60 AND node_key !~ '\\s'),
                 parent_key VARCHAR,
                 department VARCHAR NOT NULL CHECK (length(btrim(department)) BETWEEN 1 AND 80),
                 title VARCHAR NOT NULL CHECK (length(btrim(title)) BETWEEN 1 AND 120),
                 headcount INTEGER NOT NULL CHECK (headcount BETWEEN 1 AND 10000),
                 UNIQUE (plan_pid, node_key),
                 CHECK (parent_key IS NULL OR parent_key <> node_key)
             )",
        )
        .await?;
        Ok(())
    }

    /// Drop the table.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS plan_org_nodes")
            .await?;
        Ok(())
    }
}
