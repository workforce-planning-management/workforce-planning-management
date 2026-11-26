-- Migration: **a holder leaving starts a vacancy** (WPM-R288, WPM-T648). A position in `approved`, `open` or `frozen` may have a holder
-- linked (a frozen post may keep its holder), and unlinking it does not change the state, so without a row of its own the age of the
-- vacancy would count from the last state move, which was made while the post was still held. When the holder of a post in one of
-- those states is unlinked, a row is added with from and to the same state and `holder_left` true; the vacancy starts there. Which
-- holder left is not recorded here (the holder history is WPM-T649).

ALTER TABLE position_state_moves ADD COLUMN IF NOT EXISTS holder_left BOOLEAN NOT NULL DEFAULT FALSE;

-- The vacancy report reads one organization's rows in position and insertion order: one index serves the filter and the order.
CREATE INDEX IF NOT EXISTS position_state_moves_organization_position ON position_state_moves (organization_ref, position_pid, id);
DROP INDEX IF EXISTS position_state_moves_organization;
