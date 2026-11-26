-- Migration: **the state history of a position** (WPM-R288, WPM-R289, WPM-D233, WPM-D234, WPM-T648). Every move of a position is a
-- row that is added and never changed, so the age of an open vacancy can be worked out. A post that already had its state when the
-- history began gets one backfilled row with `start_known` false: its start is unknown, never zero. A move to `frozen` is recorded
-- with a reason, the role that decided and a review date. No holder and no person is in the table; the actor is whoever recorded the
-- move, as in every other audited row. The table is not soft-deleted: it is the history, and it goes when the organization's data
-- does (a position's pid is the key, not a foreign key, as for the other append-only tables).

CREATE TABLE IF NOT EXISTS position_state_moves (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 position_pid UUID NOT NULL,
                 organization_ref VARCHAR NOT NULL,
                 from_state VARCHAR NULL,
                 to_state VARCHAR NOT NULL
                     CHECK (to_state IN ('planned', 'approved', 'open', 'filled', 'frozen', 'abolished')),
                 moved_on DATE NOT NULL,
                 start_known BOOLEAN NOT NULL DEFAULT TRUE,
                 freeze_reason VARCHAR NULL
                     CHECK (freeze_reason IS NULL OR freeze_reason IN ('reorganization', 'funding', 'demand_review', 'other')),
                 decided_by_role VARCHAR NULL CHECK (decided_by_role IS NULL OR char_length(decided_by_role) BETWEEN 1 AND 80),
                 review_on DATE NULL,
                 recorded_by VARCHAR NULL
             );

-- Append-only: a row is never changed or removed.
CREATE OR REPLACE FUNCTION position_state_moves_append_only() RETURNS trigger AS $$
BEGIN
    RAISE EXCEPTION 'position_state_moves is append-only: % is not allowed', TG_OP USING ERRCODE = 'integrity_constraint_violation';
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS position_state_moves_no_change ON position_state_moves;

CREATE TRIGGER position_state_moves_no_change BEFORE UPDATE OR DELETE ON position_state_moves
                 FOR EACH ROW EXECUTE FUNCTION position_state_moves_append_only();

CREATE INDEX IF NOT EXISTS position_state_moves_position ON position_state_moves (position_pid, id);
CREATE INDEX IF NOT EXISTS position_state_moves_organization ON position_state_moves (organization_ref);

-- Rows that exist once the history can start: one per position, with the state it has now, and no known date of the move.
INSERT INTO position_state_moves (position_pid, organization_ref, from_state, to_state, moved_on, start_known, recorded_by)
SELECT p.pid, p.organization_ref, NULL, p.state, p.created_at::date, FALSE, 'backfill'
  FROM positions p
 WHERE NOT EXISTS (SELECT 1 FROM position_state_moves m WHERE m.position_pid = p.pid);
