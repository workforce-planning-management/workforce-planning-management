-- Migration: **a peer event id is unique per claim key, not across every claim** (WPM-T397, WPM-D148).
--
-- `programme_demands_peer_event` was `(source, peer_event_id)`, so an event id applied to one claim anywhere could not be applied to
-- another, and the route answered it as a duplicate of a claim the caller might not see. The repeat check now looks only among the
-- claims of the event's own key (pool, programme, month, source), and the index matches it: the same id for another key is a new event
-- there. Idempotency within a key is unchanged (one live claim per key is already `programme_demands_live`).

DROP INDEX IF EXISTS programme_demands_peer_event;

CREATE UNIQUE INDEX IF NOT EXISTS programme_demands_peer_event
                 ON programme_demands (pool_pid, programme_ref, month, source, peer_event_id) WHERE peer_event_id IS NOT NULL;
