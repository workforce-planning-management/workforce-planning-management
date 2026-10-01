//! Migration: `organization_memberships` — multi-organization access
//! in one sign-in (no switcher; unified/simultaneous visibility). One
//! table covers both cases: a person literally employed in an org
//! (`worker_pid` populated, `role = "member"`) and a person holding a
//! staff/admin-style privilege in an org without necessarily being
//! employed there (`worker_pid` null). `organization_ref` is the
//! upstream `EntityRef` URN, per this service's identity-boundary
//! convention (AGENTS.md) — this table never owns organization
//! identity, only the reference to it.

use sea_orm_migration::prelude::*;

/// The organization-memberships migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create `organization_memberships` + its lookup indexes.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        conn.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS organization_memberships (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 person_ref VARCHAR NOT NULL,
                 organization_ref VARCHAR NOT NULL,
                 worker_pid UUID NULL,
                 role VARCHAR NOT NULL,
                 starts_on DATE NOT NULL,
                 ends_on DATE NULL,
                 deleted_at TIMESTAMPTZ NULL
             )",
        )
        .await?;
        // One live (person, org, role) grant at a time (mirrors
        // benefit_enrollments_key's shape and its accepted caveat: an
        // expired-but-not-yet-revoked row still occupies this slot).
        conn.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS organization_memberships_key \
             ON organization_memberships (person_ref, organization_ref, role) \
             WHERE deleted_at IS NULL",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS organization_memberships_person \
             ON organization_memberships (person_ref)",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS organization_memberships_org \
             ON organization_memberships (organization_ref)",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS organization_memberships_worker \
             ON organization_memberships (worker_pid)",
        )
        .await?;
        Ok(())
    }

    /// Drop `organization_memberships` (rollback).
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        m.get_connection()
            .execute_unprepared("DROP TABLE IF EXISTS organization_memberships")
            .await?;
        Ok(())
    }
}
