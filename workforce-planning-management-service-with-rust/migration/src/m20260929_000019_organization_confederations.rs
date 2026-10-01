//! Migration: `organization_confederations` — a parent organization
//! that transitively contains member organizations (WPM-Rxx: org
//! confederation). One edge per direct parent→child relationship;
//! arbitrary-depth nesting (a confederation of confederations) is
//! expressed by chaining edges, walked in Rust at read time
//! (`crate::rules::org_access::descendants_of`), not recursive SQL —
//! this table stays a small, demo-scale edge list, matching
//! `organization_memberships`'s own "walk in Rust" convention.
//! `parent_organization_ref`/`child_organization_ref` are upstream
//! `EntityRef` URNs, per this service's identity-boundary convention
//! (AGENTS.md) — this table never owns organization identity, only the
//! reference to it, exactly like `organization_memberships`.

use sea_orm_migration::prelude::*;

/// The organization-confederations migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create `organization_confederations` + its lookup indexes.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        conn.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS organization_confederations (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 parent_organization_ref VARCHAR NOT NULL,
                 child_organization_ref VARCHAR NOT NULL,
                 starts_on DATE NOT NULL,
                 ends_on DATE NULL,
                 deleted_at TIMESTAMPTZ NULL
             )",
        )
        .await?;
        // One live edge per (parent, child) pair at a time — a child
        // can have several live parents (matrix confederations are
        // allowed; this only forbids declaring the same edge twice).
        conn.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS organization_confederations_key \
             ON organization_confederations (parent_organization_ref, child_organization_ref) \
             WHERE deleted_at IS NULL",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS organization_confederations_parent \
             ON organization_confederations (parent_organization_ref)",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS organization_confederations_child \
             ON organization_confederations (child_organization_ref)",
        )
        .await?;
        Ok(())
    }

    /// Drop `organization_confederations` (rollback).
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS organization_confederations")
            .await?;
        Ok(())
    }
}
