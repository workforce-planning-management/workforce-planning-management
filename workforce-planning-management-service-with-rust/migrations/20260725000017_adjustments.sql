-- Migration: reasonable adjustments (WPM-R33 / WPM-D25) — the
-- barrier, its impact, and the change that would reduce it. There is
-- deliberately **no diagnosis, condition, or medical-evidence
-- column**: the schema cannot hold what must never be required.

CREATE TABLE IF NOT EXISTS adjustment_requests (
                     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                     updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                     id SERIAL PRIMARY KEY,
                     pid UUID NOT NULL UNIQUE,
                     worker_pid UUID NOT NULL,
                     category VARCHAR NOT NULL,
                     barrier VARCHAR NOT NULL,
                     impact VARCHAR NOT NULL,
                     adjustment VARCHAR NOT NULL,
                     status VARCHAR NOT NULL DEFAULT 'requested',
                     decision_note VARCHAR NULL,
                     decided_on DATE NULL,
                     deleted_at TIMESTAMPTZ NULL
                 );
                 CREATE INDEX IF NOT EXISTS adjustment_requests_worker
                     ON adjustment_requests (worker_pid);;
