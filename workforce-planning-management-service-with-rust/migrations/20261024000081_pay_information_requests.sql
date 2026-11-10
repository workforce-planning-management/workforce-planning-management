-- Migration: **pay information requests** (WPM-R145). A worker's request to know their own pay level and the average
-- pay, by sex, of their category of workers. The row holds the dates only: the figures are computed when HR answers and
-- are not stored.

CREATE TABLE IF NOT EXISTS pay_information_requests (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 worker_pid UUID NOT NULL,
                 asked_on DATE NOT NULL,
                 due_on DATE NOT NULL CHECK (due_on >= asked_on),
                 answered_on DATE CHECK (answered_on IS NULL OR answered_on >= asked_on),
                 answered_by VARCHAR,
                 deleted_at TIMESTAMPTZ
             );

CREATE UNIQUE INDEX IF NOT EXISTS pay_information_requests_one_open
                 ON pay_information_requests (worker_pid) WHERE answered_on IS NULL AND deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS pay_information_requests_deleted_at_idx
                 ON pay_information_requests (deleted_at) WHERE deleted_at IS NOT NULL;
