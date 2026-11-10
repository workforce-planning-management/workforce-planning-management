-- Migration: **groups belong to an organization**. `groups.organization_ref`
-- is backfilled from the groups' members (the organization most of them work
-- in); a group with no members has nothing to say where it belongs, so it is
-- retired (soft-deleted, `organization_ref` empty). Group names are unique
-- within an organization, not globally.

ALTER TABLE groups ADD COLUMN IF NOT EXISTS organization_ref VARCHAR NULL;

UPDATE groups g SET organization_ref = (
                 SELECT w.organization_ref FROM group_members gm
                 JOIN workers w ON w.pid = gm.worker_pid
                 WHERE gm.group_pid = g.pid
                 GROUP BY w.organization_ref
                 ORDER BY COUNT(*) DESC, w.organization_ref
                 LIMIT 1)
             WHERE g.organization_ref IS NULL;

UPDATE groups SET organization_ref = '', deleted_at = COALESCE(deleted_at, now())
             WHERE organization_ref IS NULL;

ALTER TABLE groups ALTER COLUMN organization_ref SET NOT NULL;

DROP INDEX IF EXISTS groups_name_live;

CREATE UNIQUE INDEX IF NOT EXISTS groups_name_live ON groups (organization_ref, lower(name)) WHERE deleted_at IS NULL;
