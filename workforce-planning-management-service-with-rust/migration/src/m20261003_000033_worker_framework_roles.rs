//! Migration: `worker_framework_roles` — the role a person says they hold
//! *now* in an external capability framework: a UK GDAD PCF role level (a
//! role profile) or an ESCO occupation. One per worker per framework. A
//! person's own statement about themselves: owned by the worker, so it joins
//! subject-access export and erasure. The skills they select under it are
//! ordinary skill declarations (`worker_skills`).

use sea_orm_migration::prelude::*;

/// The worker-framework-roles migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create the table + its uniqueness key.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        conn.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS worker_framework_roles (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 worker_pid UUID NOT NULL,
                 framework_slug VARCHAR NOT NULL,
                 role_profile_pid UUID NULL,
                 esco_occupation_uri VARCHAR NULL,
                 role_label VARCHAR NOT NULL,
                 selected_on DATE NOT NULL
             )",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS worker_framework_roles_key \
             ON worker_framework_roles (worker_pid, framework_slug)",
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
            .execute_unprepared("DROP TABLE IF EXISTS worker_framework_roles")
            .await?;
        Ok(())
    }
}
