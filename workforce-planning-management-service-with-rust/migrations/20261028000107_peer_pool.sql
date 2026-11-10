-- Migration: **a pool's link to its peer's pool, and where a demand claim came from** (WPM-R213, WPM-D139, WPM-D140).
--
-- `skill_pools.peer_pool_ref` is the other app's id (a UUID) for the pool that means the same skills; null is the normal case and
-- leaves the pool local. `programme_demands.source` is `local` (entered here) or `ppm` (received); the live-claim uniqueness now
-- includes the source, so a received claim and a local one for the same programme, pool and month can sit side by side and are
-- never summed. A received claim keeps the peer's event id (idempotent receive) and the time it arrived.

ALTER TABLE skill_pools ADD COLUMN IF NOT EXISTS peer_pool_ref VARCHAR NULL
                 CHECK (peer_pool_ref IS NULL OR peer_pool_ref ~ '^[0-9a-f-]{36}$');

CREATE UNIQUE INDEX IF NOT EXISTS skill_pools_peer_once
                 ON skill_pools (organization_ref, peer_pool_ref) WHERE peer_pool_ref IS NOT NULL AND deleted_at IS NULL;

ALTER TABLE programme_demands ADD COLUMN IF NOT EXISTS source VARCHAR NOT NULL DEFAULT 'local'
                 CHECK (source IN ('local', 'ppm'));

ALTER TABLE programme_demands ADD COLUMN IF NOT EXISTS peer_event_id VARCHAR NULL;

ALTER TABLE programme_demands ADD COLUMN IF NOT EXISTS received_at TIMESTAMPTZ NULL;

CREATE UNIQUE INDEX IF NOT EXISTS programme_demands_peer_event
                 ON programme_demands (source, peer_event_id) WHERE peer_event_id IS NOT NULL;

DROP INDEX IF EXISTS programme_demands_live;

CREATE UNIQUE INDEX IF NOT EXISTS programme_demands_live
                 ON programme_demands (programme_ref, pool_pid, month, source) WHERE deleted_at IS NULL;
