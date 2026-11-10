-- Migration: **`SFIAplus` development resources** (WPM-R112) — typical tasks,
-- suggested training, certifications and memberships a SFIA skill carries at
-- a level, loaded by `import_sfiaplus` from what the deployer is licensed
-- for. They are **pointers**: WPM books and buys nothing from them, and names
-- the source wherever they are shown. A resource a newer file drops is kept
-- with `retired_in` set.

CREATE TABLE IF NOT EXISTS sfiaplus_resources (
                     created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                     updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                     id SERIAL PRIMARY KEY,
                     code VARCHAR NOT NULL,
                     level INTEGER NOT NULL CHECK (level BETWEEN 1 AND 7),
                     kind VARCHAR NOT NULL
                         CHECK (kind IN ('typical_task', 'training', 'certification', 'membership')),
                     title VARCHAR NOT NULL,
                     provider VARCHAR NULL,
                     url VARCHAR NULL CHECK (url IS NULL OR url ~ '^https?://'),
                     retired_in VARCHAR NULL,
                     UNIQUE (code, level, kind, title)
                 );
