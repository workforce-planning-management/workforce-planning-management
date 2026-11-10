-- Migration: **capability frameworks** — a framework a set of role
-- profiles was imported from (the UK GDAD Profession Capability Framework
-- first), with its attribution and scale; and the provenance columns that
-- tie a role profile to its framework, profession, role, and level, and a
-- requirement to the level it was stated at on the framework's own scale.
-- The source level is kept next to WPM's 1–5 `min_proficiency` so the
-- conversion is visible and never pretends two scales are one.

CREATE TABLE IF NOT EXISTS capability_frameworks (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 slug VARCHAR NOT NULL UNIQUE,
                 name VARCHAR NOT NULL,
                 source_url VARCHAR NULL,
                 licence VARCHAR NULL,
                 attribution VARCHAR NULL,
                 scale_max INTEGER NOT NULL,
                 scale_labels VARCHAR NULL,
                 imported_on DATE NOT NULL,
                 note VARCHAR NULL
             );

ALTER TABLE role_profiles ADD COLUMN IF NOT EXISTS framework_slug VARCHAR NULL;

ALTER TABLE role_profiles ADD COLUMN IF NOT EXISTS external_ref VARCHAR NULL;

ALTER TABLE role_profiles ADD COLUMN IF NOT EXISTS profession VARCHAR NULL;

ALTER TABLE role_profiles ADD COLUMN IF NOT EXISTS role_name VARCHAR NULL;

ALTER TABLE role_profiles ADD COLUMN IF NOT EXISTS level_name VARCHAR NULL;

ALTER TABLE role_profiles ADD COLUMN IF NOT EXISTS level_order INTEGER NULL;

ALTER TABLE role_profiles ADD COLUMN IF NOT EXISTS management_track BOOLEAN NOT NULL DEFAULT false;

CREATE UNIQUE INDEX IF NOT EXISTS role_profiles_framework_ref ON role_profiles (framework_slug, external_ref) WHERE framework_slug IS NOT NULL AND deleted_at IS NULL;

ALTER TABLE role_skill_requirements ADD COLUMN IF NOT EXISTS source_level INTEGER NULL;

ALTER TABLE role_skill_requirements ADD COLUMN IF NOT EXISTS source_scale_max INTEGER NULL;
