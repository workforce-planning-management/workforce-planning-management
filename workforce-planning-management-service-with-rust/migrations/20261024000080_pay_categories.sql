-- Migration: **categories of workers** for pay reporting (WPM-R144, WPM-D99). A category is the employer's own
-- grouping of roles that do the same work or work of equal value, with the basis stated. It holds no personal
-- data: the roles are job titles, and who sits in a category is worked out when a report is read.

CREATE TABLE IF NOT EXISTS pay_categories (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 organization_ref VARCHAR NOT NULL,
                 name VARCHAR NOT NULL CHECK (length(btrim(name)) BETWEEN 1 AND 120),
                 basis TEXT NOT NULL CHECK (length(btrim(basis)) BETWEEN 10 AND 2000),
                 job_titles JSONB NOT NULL
                     CHECK (jsonb_typeof(job_titles) = 'array' AND jsonb_array_length(job_titles) BETWEEN 1 AND 50),
                 set_by VARCHAR,
                 deleted_at TIMESTAMPTZ
             );

CREATE UNIQUE INDEX IF NOT EXISTS pay_categories_org_name
                 ON pay_categories (organization_ref, lower(name)) WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS pay_categories_deleted_at_idx
                 ON pay_categories (deleted_at) WHERE deleted_at IS NOT NULL;
