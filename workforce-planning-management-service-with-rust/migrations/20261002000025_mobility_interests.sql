-- Migration: `mobility_interests` — an employee expressing interest in a
-- role profile or an open requisition (internal mobility, WPM-R37).
-- Employee-initiated and owned by the worker: visible to them, with
-- only aggregate counts visible to anyone else (WPM-D28). Withdrawing is
-- a soft-delete.

CREATE TABLE IF NOT EXISTS mobility_interests (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 worker_pid UUID NOT NULL,
                 role_profile_pid UUID NULL,
                 requisition_pid UUID NULL,
                 note VARCHAR NULL,
                 deleted_at TIMESTAMPTZ NULL,
                 CHECK (num_nonnulls(role_profile_pid, requisition_pid) = 1)
             );

CREATE UNIQUE INDEX IF NOT EXISTS mobility_interests_role_key ON mobility_interests (worker_pid, role_profile_pid) WHERE role_profile_pid IS NOT NULL AND deleted_at IS NULL;

CREATE UNIQUE INDEX IF NOT EXISTS mobility_interests_requisition_key ON mobility_interests (worker_pid, requisition_pid) WHERE requisition_pid IS NOT NULL AND deleted_at IS NULL;
