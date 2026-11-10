-- Migration: **learning economics** (WPM-R227 to WPM-R230, WPM-T446, WPM-T450, WPM-T451). A course may carry a fee, a plan may
-- carry a stated yearly benefit with its source, and a department may hold a learning budget. No fee is unknown, never free,
-- so the fee columns are nullable and both or neither.

ALTER TABLE skill_courses
                 ADD COLUMN IF NOT EXISTS fee_minor BIGINT,
                 ADD COLUMN IF NOT EXISTS fee_currency VARCHAR;

ALTER TABLE skill_courses DROP CONSTRAINT IF EXISTS skill_courses_fee_whole;

ALTER TABLE skill_courses ADD CONSTRAINT skill_courses_fee_whole CHECK (
                 (fee_minor IS NULL AND fee_currency IS NULL)
                 OR (fee_minor IS NOT NULL AND fee_minor >= 0 AND fee_currency IS NOT NULL)
             );

CREATE TABLE IF NOT EXISTS development_plan_benefits (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 plan_pid UUID NOT NULL,
                 worker_pid UUID NOT NULL,
                 year INTEGER NOT NULL CHECK (year BETWEEN 1 AND 600),
                 amount_minor BIGINT NOT NULL CHECK (amount_minor > 0),
                 currency VARCHAR NOT NULL,
                 source VARCHAR NOT NULL CHECK (length(btrim(source)) BETWEEN 1 AND 500),
                 stated_by VARCHAR,
                 stated_on DATE NOT NULL,
                 deleted_at TIMESTAMPTZ
             );

CREATE UNIQUE INDEX IF NOT EXISTS development_plan_benefits_one_a_year
                 ON development_plan_benefits (plan_pid, year) WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS development_plan_benefits_worker
                 ON development_plan_benefits (worker_pid);

CREATE INDEX IF NOT EXISTS development_plan_benefits_deleted_at_idx
                 ON development_plan_benefits (deleted_at) WHERE deleted_at IS NOT NULL;

CREATE TABLE IF NOT EXISTS learning_budgets (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 organization_ref VARCHAR NOT NULL,
                 department VARCHAR NOT NULL,
                 year INTEGER NOT NULL CHECK (year BETWEEN 2000 AND 2200),
                 amount_minor BIGINT NOT NULL CHECK (amount_minor >= 0),
                 currency VARCHAR NOT NULL,
                 recorded_by VARCHAR,
                 UNIQUE (organization_ref, department, year, currency)
             );
