-- Migration: the AI-driven change tracker — `change_initiatives` (an
-- automation / AI / process change with a lifecycle), the roles it
-- displaces, reshapes, or creates (`initiative_role_impacts`), and the
-- skills whose demand it moves (`initiative_skill_shifts`). It tracks
-- roles and skills, never people (WPM-D28).

CREATE TABLE IF NOT EXISTS change_initiatives (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 name VARCHAR NOT NULL,
                 description VARCHAR NULL,
                 kind VARCHAR NOT NULL,
                 status VARCHAR NOT NULL,
                 starts_on DATE NULL,
                 deleted_at TIMESTAMPTZ NULL
             );

CREATE TABLE IF NOT EXISTS initiative_role_impacts (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 initiative_pid UUID NOT NULL,
                 role_profile_pid UUID NOT NULL,
                 impact VARCHAR NOT NULL,
                 timeframe VARCHAR NOT NULL,
                 note VARCHAR NULL
             );

CREATE UNIQUE INDEX IF NOT EXISTS initiative_role_impacts_key ON initiative_role_impacts (initiative_pid, role_profile_pid);

CREATE TABLE IF NOT EXISTS initiative_skill_shifts (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 initiative_pid UUID NOT NULL,
                 skill_pid UUID NOT NULL,
                 direction VARCHAR NOT NULL,
                 note VARCHAR NULL
             );

CREATE UNIQUE INDEX IF NOT EXISTS initiative_skill_shifts_key ON initiative_skill_shifts (initiative_pid, skill_pid);
