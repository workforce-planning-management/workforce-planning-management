-- Migration: **workplace health requirements** (WPM-R128, WPM-D75). A requirement an area sets
-- (such as a required immunization), and, per worker, only the **status** against it.
--
-- This is health data, so it is narrow on purpose: a status token, the day it was recorded and the
-- day it falls due again. There is no column for a diagnosis, a product, a batch or the reason for
-- an exemption. The tables exist always but nothing writes to them unless a deployer records a
-- lawful basis (`WPM_HEALTH_REQUIREMENTS_BASIS`). Records are exported to the worker alone, erased
-- with them, and not in the retention sweep.

CREATE TABLE IF NOT EXISTS health_requirements (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 name VARCHAR NOT NULL,
                 description VARCHAR NULL,
                 departments JSONB NOT NULL DEFAULT '[]'::jsonb,
                 recheck_calendar_days INTEGER NULL,
                 reminder_calendar_days INTEGER NOT NULL DEFAULT 60,
                 deleted_at TIMESTAMPTZ NULL,
                 CHECK (recheck_calendar_days IS NULL OR recheck_calendar_days BETWEEN 1 AND 3660),
                 CHECK (reminder_calendar_days BETWEEN 0 AND 3660)
             );

CREATE UNIQUE INDEX IF NOT EXISTS health_requirements_name_live
                 ON health_requirements (lower(name)) WHERE deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS worker_health_records (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 worker_pid UUID NOT NULL,
                 requirement_pid UUID NOT NULL,
                 status VARCHAR NOT NULL,
                 recorded_on DATE NOT NULL,
                 next_due DATE NULL,
                 recorded_by VARCHAR NULL,
                 UNIQUE (worker_pid, requirement_pid),
                 CHECK (status IN ('up_to_date', 'exempt_recorded', 'declined'))
             );
