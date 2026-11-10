-- Migration: the **SFIA catalogue** (WPM-R109) — skills and the levels each
-- is defined at, loaded by `import_sfia` from the deployer's licensed
-- download. Nothing here ships with WPM. A skill or level a newer version
-- drops is kept with `retired_in` set, never deleted, so a declared skill
-- never points at nothing.

CREATE TABLE IF NOT EXISTS sfia_skills (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 code VARCHAR NOT NULL UNIQUE,
                 name VARCHAR NOT NULL,
                 category VARCHAR NOT NULL,
                 subcategory VARCHAR NOT NULL,
                 description VARCHAR NOT NULL,
                 retired_in VARCHAR NULL
             );

CREATE TABLE IF NOT EXISTS sfia_skill_levels (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 code VARCHAR NOT NULL,
                 level INTEGER NOT NULL CHECK (level BETWEEN 1 AND 7),
                 description VARCHAR NOT NULL,
                 attributes VARCHAR NULL,
                 retired_in VARCHAR NULL,
                 UNIQUE (code, level)
             );
