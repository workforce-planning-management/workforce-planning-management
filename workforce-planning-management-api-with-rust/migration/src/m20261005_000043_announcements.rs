//! Migration: **announcements**. A feed of company or organization news:
//! a title and body, optionally pinned, optionally scheduled
//! (`publish_on`) and expiring (`expires_on`).

use sea_orm_migration::prelude::*;

/// The announcements migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Create `announcements`.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        conn.execute_unprepared(
            "CREATE TABLE IF NOT EXISTS announcements (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 organization_ref VARCHAR NOT NULL,
                 title VARCHAR NOT NULL,
                 body VARCHAR NOT NULL,
                 pinned BOOLEAN NOT NULL DEFAULT false,
                 publish_on DATE NOT NULL DEFAULT CURRENT_DATE,
                 expires_on DATE NULL,
                 author VARCHAR NULL,
                 deleted_at TIMESTAMPTZ NULL,
                 CHECK (expires_on IS NULL OR expires_on >= publish_on)
             )",
        )
        .await?;
        conn.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS announcements_org ON announcements (organization_ref)",
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
            .execute_unprepared("DROP TABLE IF EXISTS announcements")
            .await?;
        Ok(())
    }
}
