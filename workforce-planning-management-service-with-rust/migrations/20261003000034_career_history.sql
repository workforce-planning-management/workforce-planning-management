-- Migration: **career history and aspirations**. Time becomes first-class:
--
-- - `worker_framework_roles` gains `started_at` / `ended_at`: changing a role
--   closes the previous one and opens the next, so a person's past roles are
--   simply the closed rows (and can also be added retrospectively). Only one
--   role per framework is *current* (`ended_at IS NULL`).
-- - `worker_skill_history`: every change to a declared skill's level opens a
--   new row and closes the old, so past skills and levels can be read back
--   ("what did they have in March?"). Existing declarations are backfilled.
-- - `worker_aspirations`: future roles and skill targets — aspirations,
--   learning goals, growth ideas — private to the person unless they share
--   them.
--
-- Each row records who made it and whether they acted on someone else's
-- behalf (HR setting a role for a worker).

ALTER TABLE worker_framework_roles ADD COLUMN IF NOT EXISTS started_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP;

ALTER TABLE worker_framework_roles ADD COLUMN IF NOT EXISTS ended_at TIMESTAMPTZ NULL;

ALTER TABLE worker_framework_roles ADD COLUMN IF NOT EXISTS recorded_by VARCHAR NULL;

ALTER TABLE worker_framework_roles ADD COLUMN IF NOT EXISTS on_behalf BOOLEAN NOT NULL DEFAULT false;

UPDATE worker_framework_roles SET started_at = selected_on::timestamptz WHERE ended_at IS NULL;

DROP INDEX IF EXISTS worker_framework_roles_key;

CREATE UNIQUE INDEX IF NOT EXISTS worker_framework_roles_current ON worker_framework_roles (worker_pid, framework_slug) WHERE ended_at IS NULL;

CREATE INDEX IF NOT EXISTS worker_framework_roles_history ON worker_framework_roles (worker_pid, framework_slug, started_at);

CREATE TABLE IF NOT EXISTS worker_skill_history (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 worker_pid UUID NOT NULL,
                 skill_pid UUID NOT NULL,
                 proficiency INTEGER NOT NULL,
                 started_at TIMESTAMPTZ NOT NULL,
                 ended_at TIMESTAMPTZ NULL,
                 source VARCHAR NOT NULL,
                 recorded_by VARCHAR NULL,
                 on_behalf BOOLEAN NOT NULL DEFAULT false
             );

CREATE UNIQUE INDEX IF NOT EXISTS worker_skill_history_open ON worker_skill_history (worker_pid, skill_pid) WHERE ended_at IS NULL;

CREATE INDEX IF NOT EXISTS worker_skill_history_timeline ON worker_skill_history (worker_pid, skill_pid, started_at);

INSERT INTO worker_skill_history (pid, worker_pid, skill_pid, proficiency, started_at, source) SELECT gen_random_uuid(), ws.worker_pid, ws.skill_pid, ws.proficiency, ws.assessed_on::timestamptz, 'backfill' FROM worker_skills ws WHERE ws.deleted_at IS NULL AND NOT EXISTS (SELECT 1 FROM worker_skill_history h WHERE h.worker_pid = ws.worker_pid AND h.skill_pid = ws.skill_pid AND h.ended_at IS NULL);

CREATE TABLE IF NOT EXISTS worker_aspirations (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 worker_pid UUID NOT NULL,
                 kind VARCHAR NOT NULL,
                 framework_slug VARCHAR NULL,
                 role_profile_pid UUID NULL,
                 esco_occupation_uri VARCHAR NULL,
                 role_label VARCHAR NULL,
                 skill_pid UUID NULL,
                 target_level INTEGER NULL,
                 horizon VARCHAR NOT NULL,
                 status VARCHAR NOT NULL,
                 note VARCHAR NULL,
                 shared BOOLEAN NOT NULL DEFAULT false,
                 recorded_by VARCHAR NULL,
                 on_behalf BOOLEAN NOT NULL DEFAULT false,
                 deleted_at TIMESTAMPTZ NULL,
                 CHECK (
                     (kind = 'role' AND role_label IS NOT NULL)
                     OR (kind = 'skill' AND skill_pid IS NOT NULL AND target_level IS NOT NULL)
                 )
             );

CREATE INDEX IF NOT EXISTS worker_aspirations_worker ON worker_aspirations (worker_pid);
