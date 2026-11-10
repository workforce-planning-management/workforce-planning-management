-- Migration: the anonymous wellbeing pulse (WPM-R28) — surveys and
-- their responses. The response table is **anonymous by
-- construction** (WPM-D20): it carries the survey, the department,
-- the score, and the date — there is deliberately no author column,
-- and no hash that could stand in for one.

CREATE TABLE IF NOT EXISTS pulse_surveys (
                     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                     updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                     id SERIAL PRIMARY KEY,
                     pid UUID NOT NULL UNIQUE,
                     name VARCHAR NOT NULL UNIQUE,
                     question VARCHAR NOT NULL,
                     active_from DATE NULL,
                     active_until DATE NULL,
                     deleted_at TIMESTAMPTZ NULL
                 );
                 CREATE TABLE IF NOT EXISTS pulse_responses (
                     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                     updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                     id SERIAL PRIMARY KEY,
                     pid UUID NOT NULL UNIQUE,
                     survey_pid UUID NOT NULL,
                     department VARCHAR NOT NULL,
                     score INTEGER NOT NULL,
                     submitted_on DATE NOT NULL
                 );
                 CREATE INDEX IF NOT EXISTS pulse_responses_survey
                     ON pulse_responses (survey_pid);;
