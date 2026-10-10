//! Migration: **expense claims**. A claim is a worker's request to be repaid for
//! money they spent, built from dated, categorised items in one currency, decided
//! by someone other than the claimant, and marked reimbursed. Items carry the
//! claimant's `worker_pid` so they travel with the person in the subject-access
//! export and erasure.

use sea_orm_migration::prelude::*;

/// The expense-claims migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create `expense_claims` and `expense_items`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        conn.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS expense_claims (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 worker_pid UUID NOT NULL,
                 title VARCHAR NOT NULL,
                 description VARCHAR NULL,
                 currency VARCHAR NOT NULL,
                 status VARCHAR NOT NULL DEFAULT 'draft',
                 submitted_at TIMESTAMPTZ NULL,
                 decided_by VARCHAR NULL,
                 decided_at TIMESTAMPTZ NULL,
                 decision_note VARCHAR NULL,
                 reimbursed_on DATE NULL,
                 reimbursed_by VARCHAR NULL,
                 recorded_by VARCHAR NULL,
                 on_behalf BOOLEAN NOT NULL DEFAULT FALSE,
                 deleted_at TIMESTAMPTZ NULL,
                 CHECK (status IN ('draft','submitted','approved','rejected','reimbursed','cancelled')),
                 CHECK (char_length(currency) = 3)
             )",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS expense_claims_worker ON expense_claims (worker_pid)",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS expense_claims_status ON expense_claims (status) \
             WHERE deleted_at IS NULL",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS expense_items (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 claim_pid UUID NOT NULL,
                 worker_pid UUID NOT NULL,
                 incurred_on DATE NOT NULL,
                 category VARCHAR NOT NULL,
                 amount_minor BIGINT NOT NULL,
                 description VARCHAR NULL,
                 receipt_ref VARCHAR NULL,
                 CHECK (amount_minor > 0)
             )",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS expense_items_claim ON expense_items (claim_pid)",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS expense_items_worker ON expense_items (worker_pid)",
        )
        .await?;
        Ok(())
    }

    /// Drop them again.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        for table in ["expense_items", "expense_claims"] {
            conn.execute_unprepared(&format!("DROP TABLE IF EXISTS {table}"))
                .await?;
        }
        Ok(())
    }
}
