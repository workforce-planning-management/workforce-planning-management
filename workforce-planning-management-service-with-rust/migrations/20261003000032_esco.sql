-- Migration: a pinned local copy of **ESCO** (`spec/esco/index.md`) — its
-- occupations, skills, and the essential/optional relations between them —
-- loaded from a CSV download by `import_esco`. ESCO is a classification WPM
-- *references*; the catalogue and role profiles stay WPM's own. The copy is
-- replaced wholesale on each import so it is reproducible for a given
-- ESCO version.

CREATE TABLE IF NOT EXISTS esco_skills (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 uri VARCHAR NOT NULL UNIQUE,
                 label VARCHAR NOT NULL,
                 skill_type VARCHAR NULL,
                 reuse_level VARCHAR NULL
             );

CREATE INDEX IF NOT EXISTS esco_skills_label ON esco_skills (lower(label));

CREATE TABLE IF NOT EXISTS esco_occupations (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 uri VARCHAR NOT NULL UNIQUE,
                 label VARCHAR NOT NULL,
                 isco_code VARCHAR NULL,
                 description VARCHAR NULL
             );

CREATE INDEX IF NOT EXISTS esco_occupations_label ON esco_occupations (lower(label));

CREATE TABLE IF NOT EXISTS esco_occupation_skills (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 occupation_uri VARCHAR NOT NULL,
                 skill_uri VARCHAR NOT NULL,
                 relation VARCHAR NOT NULL
             );

CREATE UNIQUE INDEX IF NOT EXISTS esco_occupation_skills_key ON esco_occupation_skills (occupation_uri, skill_uri);
