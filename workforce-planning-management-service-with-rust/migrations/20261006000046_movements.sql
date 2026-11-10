-- Migration: **joiners and leavers**.
--
-- - `movements`: someone joining or leaving, on an effective day (the start
--   day, or the leaver's last day), with a reason for a leaver. At most one
--   open movement of each kind per worker.
-- - `movement_items`: its dated checklist.
-- - `handover_actions`: the audit trail of what a leaver held and who it
--   went to — one row per reassignment, closure or revocation.

CREATE TABLE IF NOT EXISTS movements (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 worker_pid UUID NOT NULL,
                 kind VARCHAR NOT NULL CHECK (kind IN ('joiner', 'leaver')),
                 effective_on DATE NOT NULL,
                 reason VARCHAR NULL,
                 status VARCHAR NOT NULL DEFAULT 'open'
                     CHECK (status IN ('open', 'completed', 'cancelled')),
                 notes VARCHAR NULL,
                 created_by VARCHAR NULL,
                 completed_at TIMESTAMPTZ NULL
             );

CREATE UNIQUE INDEX IF NOT EXISTS movements_one_open ON movements (worker_pid, kind) WHERE status = 'open';

CREATE TABLE IF NOT EXISTS movement_items (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 movement_pid UUID NOT NULL,
                 position INTEGER NOT NULL,
                 title VARCHAR NOT NULL,
                 category VARCHAR NOT NULL,
                 due_on DATE NOT NULL,
                 assignee_pid UUID NULL,
                 done_on DATE NULL,
                 done_by VARCHAR NULL,
                 skipped_reason VARCHAR NULL
             );

CREATE INDEX IF NOT EXISTS movement_items_movement ON movement_items (movement_pid);

CREATE INDEX IF NOT EXISTS movement_items_assignee ON movement_items (assignee_pid);

CREATE TABLE IF NOT EXISTS handover_actions (
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 movement_pid UUID NOT NULL,
                 kind VARCHAR NOT NULL,
                 subject_pid UUID NOT NULL,
                 subject_label VARCHAR NULL,
                 from_worker UUID NOT NULL,
                 to_worker UUID NULL,
                 action VARCHAR NOT NULL,
                 note VARCHAR NULL,
                 performed_by VARCHAR NULL,
                 performed_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
             );

CREATE INDEX IF NOT EXISTS handover_actions_movement ON handover_actions (movement_pid);
