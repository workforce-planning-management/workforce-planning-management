//! Migration: **groups belong to an organization**. `groups.organization_ref`
//! is backfilled from the groups' members (the organization most of them work
//! in); a group with no members has nothing to say where it belongs, so it is
//! retired (soft-deleted, `organization_ref` empty). Group names are unique
//! within an organization, not globally.

use sea_orm_migration::prelude::*;

/// The group-organizations migration.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    /// Add and backfill `organization_ref`; re-key the name index.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error.
    async fn up(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        conn.execute_unprepared(
            "ALTER TABLE groups ADD COLUMN IF NOT EXISTS organization_ref VARCHAR NULL",
        )
        .await?;
        conn.execute_unprepared(
            "UPDATE groups g SET organization_ref = (
                 SELECT w.organization_ref FROM group_members gm
                 JOIN workers w ON w.pid = gm.worker_pid
                 WHERE gm.group_pid = g.pid
                 GROUP BY w.organization_ref
                 ORDER BY COUNT(*) DESC, w.organization_ref
                 LIMIT 1)
             WHERE g.organization_ref IS NULL",
        )
        .await?;
        conn.execute_unprepared(
            "UPDATE groups SET organization_ref = '', deleted_at = COALESCE(deleted_at, now())
             WHERE organization_ref IS NULL",
        )
        .await?;
        conn.execute_unprepared("ALTER TABLE groups ALTER COLUMN organization_ref SET NOT NULL")
            .await?;
        conn.execute_unprepared("DROP INDEX IF EXISTS groups_name_live")
            .await?;
        conn.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS groups_name_live \
             ON groups (organization_ref, lower(name)) WHERE deleted_at IS NULL",
        )
        .await?;
        Ok(())
    }

    /// Restore global name uniqueness and drop the column.
    ///
    /// # Errors
    ///
    /// Propagates any DDL error (a duplicate name across organizations would
    /// block restoring the global index).
    async fn down(&self, m: &SchemaManager) -> Result<(), DbErr> {
        let conn = m.get_connection();
        conn.execute_unprepared("DROP INDEX IF EXISTS groups_name_live")
            .await?;
        conn.execute_unprepared("ALTER TABLE groups DROP COLUMN IF EXISTS organization_ref")
            .await?;
        conn.execute_unprepared(
            "CREATE UNIQUE INDEX IF NOT EXISTS groups_name_live \
             ON groups (lower(name)) WHERE deleted_at IS NULL",
        )
        .await?;
        Ok(())
    }
}
