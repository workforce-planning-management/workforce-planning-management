-- Migration: benefits (WPM-R9) and talent-development tables
-- (WPM-R10–R12) — `benefit_plans` + `benefit_enrollments`,
-- `review_cycles` / `reviews` / `goals` / `feedback_entries`,
-- `training_enrollments`, `succession_plans` +
-- `succession_candidates`.

CREATE TABLE IF NOT EXISTS benefit_plans (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 name VARCHAR NOT NULL,
                 kind VARCHAR NOT NULL,
                 provider VARCHAR NOT NULL,
                 worker_cost_minor BIGINT NOT NULL,
                 employer_cost_minor BIGINT NOT NULL,
                 currency VARCHAR NOT NULL,
                 deleted_at TIMESTAMPTZ NULL
             );

CREATE TABLE IF NOT EXISTS benefit_enrollments (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 plan_pid UUID NOT NULL,
                 worker_pid UUID NOT NULL,
                 starts_on DATE NOT NULL,
                 ends_on DATE NULL,
                 deleted_at TIMESTAMPTZ NULL
             );

CREATE UNIQUE INDEX IF NOT EXISTS benefit_enrollments_key ON benefit_enrollments (plan_pid, worker_pid) WHERE deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS review_cycles (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 name VARCHAR NOT NULL,
                 period_start DATE NOT NULL,
                 period_end DATE NOT NULL,
                 status VARCHAR NOT NULL,
                 deleted_at TIMESTAMPTZ NULL
             );

CREATE TABLE IF NOT EXISTS reviews (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 cycle_pid UUID NOT NULL,
                 worker_pid UUID NOT NULL,
                 reviewer_ref VARCHAR NOT NULL,
                 status VARCHAR NOT NULL,
                 rating INTEGER NULL,
                 content VARCHAR NULL,
                 deleted_at TIMESTAMPTZ NULL
             );

CREATE INDEX IF NOT EXISTS reviews_worker ON reviews (worker_pid);

CREATE TABLE IF NOT EXISTS goals (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 review_pid UUID NOT NULL,
                 title VARCHAR NOT NULL,
                 weight_percent INTEGER NOT NULL,
                 status VARCHAR NOT NULL,
                 deleted_at TIMESTAMPTZ NULL
             );

CREATE TABLE IF NOT EXISTS feedback_entries (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 review_pid UUID NOT NULL,
                 author_ref VARCHAR NOT NULL,
                 content VARCHAR NOT NULL,
                 deleted_at TIMESTAMPTZ NULL
             );

CREATE TABLE IF NOT EXISTS training_enrollments (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 worker_pid UUID NOT NULL,
                 course_ref VARCHAR NOT NULL,
                 status VARCHAR NOT NULL,
                 completed_on DATE NULL,
                 certificate_expires_on DATE NULL,
                 deleted_at TIMESTAMPTZ NULL
             );

CREATE INDEX IF NOT EXISTS training_enrollments_worker ON training_enrollments (worker_pid);

CREATE TABLE IF NOT EXISTS succession_plans (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 role_title VARCHAR NOT NULL,
                 department VARCHAR NOT NULL,
                 criticality INTEGER NOT NULL,
                 incumbent_pid UUID NULL,
                 deleted_at TIMESTAMPTZ NULL
             );

CREATE TABLE IF NOT EXISTS succession_candidates (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 plan_pid UUID NOT NULL,
                 worker_pid UUID NOT NULL,
                 readiness VARCHAR NOT NULL,
                 rank INTEGER NOT NULL,
                 deleted_at TIMESTAMPTZ NULL
             );
