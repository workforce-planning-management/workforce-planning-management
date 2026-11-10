-- Migration: **grades**. A worker's current job level (one row per worker; the
-- level framework and number are validated by the service against the
-- reference ladder, not by the database), and an optional job level and pay
-- band on a role profile.

CREATE TABLE IF NOT EXISTS worker_job_levels (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 worker_pid UUID NOT NULL UNIQUE,
                 framework VARCHAR NOT NULL,
                 level_number INTEGER NOT NULL,
                 effective_on DATE NOT NULL,
                 recorded_by VARCHAR NULL,
                 on_behalf BOOLEAN NOT NULL DEFAULT FALSE,
                 CHECK (level_number BETWEEN 1 AND 100)
             );

ALTER TABLE role_profiles ADD COLUMN IF NOT EXISTS job_level_framework VARCHAR NULL;

ALTER TABLE role_profiles ADD COLUMN IF NOT EXISTS job_level INTEGER NULL;

ALTER TABLE role_profiles ADD COLUMN IF NOT EXISTS pay_scale_id VARCHAR NULL;

ALTER TABLE role_profiles ADD COLUMN IF NOT EXISTS pay_band VARCHAR NULL;

ALTER TABLE role_profiles DROP CONSTRAINT IF EXISTS role_profiles_grade_pairs;

ALTER TABLE role_profiles ADD CONSTRAINT role_profiles_grade_pairs CHECK (
                 (job_level_framework IS NULL) = (job_level IS NULL)
                 AND (pay_scale_id IS NULL) = (pay_band IS NULL)
             );
