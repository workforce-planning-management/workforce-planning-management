//! Migration: **on-call rotas**. A rota is a named rotation of workers in one
//! organization: every `period_days` the duty passes to the next member in
//! order, counting from `starts_on`. Overrides cover swaps — a named worker
//! on call for a date window regardless of the rotation.

use sea_orm_migration::prelude::*;

/// The on-call rota migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create `rotas`, `rota_members` and `rota_overrides`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        conn.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS rotas (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 organization_ref VARCHAR NOT NULL,
                 name VARCHAR NOT NULL,
                 description VARCHAR NULL,
                 period_days INTEGER NOT NULL,
                 starts_on DATE NOT NULL,
                 deleted_at TIMESTAMPTZ NULL,
                 CHECK (period_days BETWEEN 1 AND 31)
             )",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS rotas_org_name ON rotas (organization_ref, lower(name)) \
             WHERE deleted_at IS NULL",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS rota_members (
                 id SERIAL PRIMARY KEY,
                 rota_pid UUID NOT NULL,
                 worker_pid UUID NOT NULL,
                 position INTEGER NOT NULL,
                 UNIQUE (rota_pid, worker_pid),
                 UNIQUE (rota_pid, position)
             )",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS rota_members_worker ON rota_members (worker_pid)",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS rota_overrides (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 rota_pid UUID NOT NULL,
                 worker_pid UUID NOT NULL,
                 starts_on DATE NOT NULL,
                 ends_on DATE NOT NULL,
                 note VARCHAR NULL,
                 created_by VARCHAR NULL,
                 CHECK (ends_on >= starts_on)
             )",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS rota_overrides_rota ON rota_overrides (rota_pid)",
        )
        .await?;
        Ok(())
    }

    /// Drop the tables.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        for table in ["rota_overrides", "rota_members", "rotas"] {
            conn.execute_unprepared(&format!("DROP TABLE IF EXISTS {table}"))
                .await?;
        }
        Ok(())
    }
}
