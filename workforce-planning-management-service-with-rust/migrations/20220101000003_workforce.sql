-- Migration: the workforce-management tables (WPM-R4–R6) —
-- `time_entries` (whole minutes), `leave_entitlements` +
-- `leave_requests` (whole days), `shifts` + `shift_assignments`.

CREATE TABLE IF NOT EXISTS time_entries (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 worker_pid UUID NOT NULL,
                 worked_on DATE NOT NULL,
                 minutes INTEGER NOT NULL,
                 kind VARCHAR NOT NULL,
                 status VARCHAR NOT NULL,
                 notes VARCHAR NULL,
                 deleted_at TIMESTAMPTZ NULL
             );

CREATE INDEX IF NOT EXISTS time_entries_worker_day ON time_entries (worker_pid, worked_on);

CREATE TABLE IF NOT EXISTS leave_entitlements (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 worker_pid UUID NOT NULL,
                 kind VARCHAR NOT NULL,
                 year INTEGER NOT NULL,
                 entitled_days INTEGER NOT NULL,
                 used_days INTEGER NOT NULL DEFAULT 0,
                 deleted_at TIMESTAMPTZ NULL
             );

CREATE UNIQUE INDEX IF NOT EXISTS leave_entitlements_key ON leave_entitlements (worker_pid, kind, year) WHERE deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS leave_requests (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 worker_pid UUID NOT NULL,
                 kind VARCHAR NOT NULL,
                 start_on DATE NOT NULL,
                 end_on DATE NOT NULL,
                 days INTEGER NOT NULL,
                 status VARCHAR NOT NULL,
                 negative_balance BOOLEAN NOT NULL DEFAULT FALSE,
                 reason VARCHAR NULL,
                 decided_by VARCHAR NULL,
                 decided_at TIMESTAMPTZ NULL,
                 deleted_at TIMESTAMPTZ NULL
             );

CREATE INDEX IF NOT EXISTS leave_requests_worker ON leave_requests (worker_pid);

CREATE TABLE IF NOT EXISTS shifts (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 department VARCHAR NOT NULL,
                 starts_at TIMESTAMPTZ NOT NULL,
                 ends_at TIMESTAMPTZ NOT NULL,
                 required_headcount INTEGER NOT NULL,
                 deleted_at TIMESTAMPTZ NULL
             );

CREATE INDEX IF NOT EXISTS shifts_department ON shifts (department, starts_at);

CREATE TABLE IF NOT EXISTS shift_assignments (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 shift_pid UUID NOT NULL,
                 worker_pid UUID NOT NULL,
                 deleted_at TIMESTAMPTZ NULL
             );

CREATE INDEX IF NOT EXISTS shift_assignments_worker ON shift_assignments (worker_pid);
