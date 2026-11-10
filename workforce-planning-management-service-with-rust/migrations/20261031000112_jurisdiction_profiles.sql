-- Migration: **jurisdiction profiles** (WPM-D144, WPM-T380): the codes an organization keeps (`jurisdictions`), the dated, sourced
-- parameter rows (`jurisdiction_parameters`, append-only), the place a worker's work is done (`workers.work_jurisdiction`), and a
-- week-rollup index that serves any working-time ceiling rather than one regime's.
--
-- The values are a deployer's statements, not WPM's: the rules read them through `rules::jurisdiction::resolve`, and a parameter
-- with no row is unknown ("no limit configured"), never a default.

CREATE TABLE IF NOT EXISTS jurisdictions (
                 id SERIAL PRIMARY KEY,
                 organization_ref VARCHAR(255) NOT NULL,
                 code VARCHAR(7) NOT NULL CHECK (code ~ '^[A-Z0-9]{2,3}(-[A-Z0-9]{1,3})?$'),
                 layer VARCHAR(16) NOT NULL CHECK (layer IN ('supranational', 'national', 'subnational')),
                 parent_code VARCHAR(7) CHECK (parent_code ~ '^[A-Z0-9]{2,3}(-[A-Z0-9]{1,3})?$'),
                 name VARCHAR(100),
                 created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                 UNIQUE (organization_ref, code),
                 CHECK (parent_code IS NULL OR parent_code <> code)
             );

CREATE TABLE IF NOT EXISTS jurisdiction_parameters (
                 id BIGSERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 organization_ref VARCHAR(255) NOT NULL,
                 code VARCHAR(7) NOT NULL CHECK (code ~ '^[A-Z0-9]{2,3}(-[A-Z0-9]{1,3})?$'),
                 key VARCHAR(100) NOT NULL,
                 value JSONB NOT NULL,
                 effective_from DATE NOT NULL,
                 effective_to DATE,
                 source VARCHAR(500) NOT NULL CHECK (length(btrim(source)) > 0),
                 checked_on DATE,
                 status VARCHAR(16) NOT NULL CHECK (status IN ('example', 'unverified', 'read')),
                 created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                 CHECK (effective_to IS NULL OR effective_to >= effective_from),
                 CHECK (status <> 'read' OR checked_on IS NOT NULL)
             );

CREATE INDEX IF NOT EXISTS jurisdiction_parameters_org
                 ON jurisdiction_parameters (organization_ref, code, key, effective_from);

CREATE OR REPLACE FUNCTION jurisdiction_parameters_append_only() RETURNS trigger AS $$
             BEGIN
                 RAISE EXCEPTION 'jurisdiction_parameters is append-only: add a row with a later effective_from';
             END;
             $$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS jurisdiction_parameters_append_only ON jurisdiction_parameters;

CREATE TRIGGER jurisdiction_parameters_append_only
                 BEFORE UPDATE OR DELETE ON jurisdiction_parameters
                 FOR EACH ROW EXECUTE FUNCTION jurisdiction_parameters_append_only();

ALTER TABLE workers ADD COLUMN IF NOT EXISTS work_jurisdiction VARCHAR(7)
                 CHECK (work_jurisdiction ~ '^[A-Z0-9]{2,3}(-[A-Z0-9]{1,3})?$');

DROP INDEX IF EXISTS time_entry_weeks_heavy;

CREATE INDEX IF NOT EXISTS time_entry_weeks_week_minutes
                 ON time_entry_weeks (week_start, minutes, worker_pid);
