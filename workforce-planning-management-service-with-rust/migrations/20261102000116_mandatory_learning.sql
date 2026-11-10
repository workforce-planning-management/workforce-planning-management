-- Migration: **statutory and mandatory learning** (WPM-R170, WPM-D117, WPM-T315). A requirement says what must be learned, on what
-- basis (`statutory`, which names its source, or `mandatory`), how often it is refreshed, which courses satisfy it and who it is aimed
-- at. A completion is a fact HR records for training done elsewhere; a completed training enrolment for one of the courses counts as
-- well and needs no row here. A completion holds a worker, a date and where it was done: no certificate and no score.

CREATE TABLE IF NOT EXISTS learning_requirements (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 organization_ref VARCHAR(255) NOT NULL,
                 subject VARCHAR NOT NULL CHECK (length(btrim(subject)) BETWEEN 1 AND 120),
                 basis VARCHAR NOT NULL CHECK (basis IN ('statutory', 'mandatory')),
                 source VARCHAR CHECK (source IS NULL OR length(btrim(source)) BETWEEN 1 AND 200),
                 refresher_months INTEGER CHECK (refresher_months IS NULL OR refresher_months BETWEEN 1 AND 600),
                 courses JSONB NOT NULL DEFAULT '[]'::jsonb,
                 audience JSONB NOT NULL DEFAULT '{}'::jsonb,
                 accepts_transferred BOOLEAN NOT NULL DEFAULT TRUE,
                 retired_on DATE,
                 recorded_by VARCHAR,
                 CHECK (basis <> 'statutory' OR source IS NOT NULL)
             );

CREATE INDEX IF NOT EXISTS learning_requirements_by_org ON learning_requirements (organization_ref);

CREATE TABLE IF NOT EXISTS learning_completions (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 requirement_pid UUID NOT NULL,
                 worker_pid UUID NOT NULL,
                 completed_on DATE NOT NULL,
                 source VARCHAR CHECK (source IS NULL OR length(btrim(source)) BETWEEN 1 AND 120),
                 source_organization VARCHAR(255),
                 recorded_by VARCHAR
             );

CREATE INDEX IF NOT EXISTS learning_completions_by_worker ON learning_completions (worker_pid, requirement_pid);
