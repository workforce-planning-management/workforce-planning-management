-- Migration: the payroll & compensation tables (WPM-R13, WPM-R14) —
-- `payroll_runs`, `payslips` (minor units; deductions as JSONB
-- lines), `benchmarks`.

CREATE TABLE IF NOT EXISTS payroll_runs (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 organization_ref VARCHAR NOT NULL,
                 period_start DATE NOT NULL,
                 period_end DATE NOT NULL,
                 status VARCHAR NOT NULL,
                 approved_by VARCHAR NULL,
                 notes VARCHAR NULL,
                 deleted_at TIMESTAMPTZ NULL
             );

CREATE TABLE IF NOT EXISTS payslips (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 run_pid UUID NOT NULL,
                 worker_pid UUID NOT NULL,
                 currency VARCHAR NOT NULL,
                 gross_minor BIGINT NOT NULL,
                 deductions JSONB NOT NULL,
                 net_minor BIGINT NOT NULL,
                 deleted_at TIMESTAMPTZ NULL
             );

CREATE INDEX IF NOT EXISTS payslips_run ON payslips (run_pid);

CREATE INDEX IF NOT EXISTS payslips_worker ON payslips (worker_pid);

CREATE TABLE IF NOT EXISTS benchmarks (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 job_title VARCHAR NOT NULL,
                 currency VARCHAR NOT NULL,
                 min_minor BIGINT NOT NULL,
                 median_minor BIGINT NOT NULL,
                 max_minor BIGINT NOT NULL,
                 source VARCHAR NOT NULL,
                 as_of DATE NOT NULL,
                 deleted_at TIMESTAMPTZ NULL
             );
