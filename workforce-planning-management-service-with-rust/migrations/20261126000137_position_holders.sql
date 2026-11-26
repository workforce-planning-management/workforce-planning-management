-- Migration: **the holder history of a position** (WPM-R290, WPM-D233, WPM-D236, WPM-T649). Setting a position's holder overwrote it, and the
-- audit row kept only that a link was made, so the number of holders a post has had was lost. Each tenure is now a row: who, from when,
-- until when. A row is added when a holder is linked and closed (`ended_on`) when the holder is unlinked or replaced; it is otherwise
-- never changed. Two changes only are allowed, and a trigger refuses any other: closing an open tenure once, and erasure cutting the
-- worker (`worker_pid` to NULL), which keeps the tenure's dates so the count of holders and the time a post stood empty survive an
-- erasure without naming anyone (WPM-D236). A holder already linked when the history began gets one row with `start_known` false.

CREATE TABLE IF NOT EXISTS position_holders (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 position_pid UUID NOT NULL,
                 organization_ref VARCHAR NOT NULL,
                 worker_pid UUID NULL,
                 started_on DATE NOT NULL,
                 start_known BOOLEAN NOT NULL DEFAULT TRUE,
                 ended_on DATE NULL,
                 recorded_by VARCHAR NULL,
                 CHECK (ended_on IS NULL OR ended_on >= started_on)
             );

-- At most one open tenure per position.
CREATE UNIQUE INDEX IF NOT EXISTS position_holders_one_open ON position_holders (position_pid) WHERE ended_on IS NULL;
CREATE INDEX IF NOT EXISTS position_holders_position ON position_holders (position_pid, started_on);
CREATE INDEX IF NOT EXISTS position_holders_worker ON position_holders (worker_pid) WHERE worker_pid IS NOT NULL;

CREATE OR REPLACE FUNCTION position_holders_guard() RETURNS trigger AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        RAISE EXCEPTION 'position_holders is append-only: DELETE is not allowed' USING ERRCODE = 'integrity_constraint_violation';
    END IF;
    IF NEW.id <> OLD.id OR NEW.position_pid <> OLD.position_pid OR NEW.organization_ref <> OLD.organization_ref
       OR NEW.started_on <> OLD.started_on OR NEW.start_known <> OLD.start_known OR NEW.created_at <> OLD.created_at
       OR NEW.recorded_by IS DISTINCT FROM OLD.recorded_by THEN
        RAISE EXCEPTION 'position_holders: only the end of a tenure and the erasure of its worker may change' USING ERRCODE = 'integrity_constraint_violation';
    END IF;
    IF NEW.ended_on IS DISTINCT FROM OLD.ended_on AND OLD.ended_on IS NOT NULL THEN
        RAISE EXCEPTION 'position_holders: a closed tenure is not reopened or moved' USING ERRCODE = 'integrity_constraint_violation';
    END IF;
    IF NEW.worker_pid IS DISTINCT FROM OLD.worker_pid AND NEW.worker_pid IS NOT NULL THEN
        RAISE EXCEPTION 'position_holders: a worker is only ever cut from a tenure, never changed' USING ERRCODE = 'integrity_constraint_violation';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS position_holders_no_change ON position_holders;

CREATE TRIGGER position_holders_no_change BEFORE UPDATE OR DELETE ON position_holders
                 FOR EACH ROW EXECUTE FUNCTION position_holders_guard();

-- The holders linked now, each with a start that is not known.
INSERT INTO position_holders (position_pid, organization_ref, worker_pid, started_on, start_known, recorded_by)
SELECT p.pid, p.organization_ref, p.holder_worker_pid, p.updated_at::date, FALSE, 'backfill'
  FROM positions p
 WHERE p.holder_worker_pid IS NOT NULL
   AND NOT EXISTS (SELECT 1 FROM position_holders h WHERE h.position_pid = p.pid);
