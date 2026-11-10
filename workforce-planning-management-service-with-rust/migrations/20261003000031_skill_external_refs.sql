-- Migration: `skill_external_refs` — how a catalogue skill is known in an
-- external framework: its exact name in the UK GDAD PCF, its concept URI in
-- ESCO. One reference per skill per framework, and one skill per reference,
-- so an import matches by reference (surviving a rename) rather than by
-- name. WPM keeps its own skills catalogue; this is a *reference*, never a
-- second skills model.

CREATE TABLE IF NOT EXISTS skill_external_refs (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 skill_pid UUID NOT NULL,
                 framework_slug VARCHAR NOT NULL,
                 ref VARCHAR NOT NULL,
                 label VARCHAR NULL,
                 version VARCHAR NULL
             );

CREATE UNIQUE INDEX IF NOT EXISTS skill_external_refs_skill_key ON skill_external_refs (skill_pid, framework_slug);

CREATE UNIQUE INDEX IF NOT EXISTS skill_external_refs_ref_key ON skill_external_refs (framework_slug, ref);
