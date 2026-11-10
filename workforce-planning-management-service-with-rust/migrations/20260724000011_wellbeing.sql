-- Migration: the wellbeing tables (WPM-R25) — configurable
-- **health-entitlement rules** (e.g. NHS vaccination cohorts) and the
-- per-worker **acknowledgements** of their prompts.
--
-- The rule row can hold only non-clinical predicates — an age band and
-- department / job-title lists (WPM-D17): there is deliberately no
-- column a health-status cohort could be expressed in. The
-- acknowledgement row records the worker's response to a prompt
-- (`booked | done | declined | dismissed`) — an HR workflow fact,
-- never a vaccination status.

CREATE TABLE IF NOT EXISTS wellbeing_entitlements (
                     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                     updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                     id SERIAL PRIMARY KEY,
                     pid UUID NOT NULL UNIQUE,
                     name VARCHAR NOT NULL UNIQUE,
                     description VARCHAR NOT NULL,
                     info_url VARCHAR NULL,
                     min_age INTEGER NULL,
                     max_age INTEGER NULL,
                     departments JSONB NOT NULL DEFAULT '[]',
                     job_titles JSONB NOT NULL DEFAULT '[]',
                     doses INTEGER NOT NULL DEFAULT 1,
                     active_from DATE NULL,
                     active_until DATE NULL,
                     deleted_at TIMESTAMPTZ NULL
                 );
                 CREATE TABLE IF NOT EXISTS entitlement_acknowledgements (
                     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                     updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
                     id SERIAL PRIMARY KEY,
                     pid UUID NOT NULL UNIQUE,
                     entitlement_pid UUID NOT NULL,
                     worker_pid UUID NOT NULL,
                     response VARCHAR NOT NULL,
                     responded_on DATE NOT NULL,
                     reminded_on DATE NULL,
                     UNIQUE (entitlement_pid, worker_pid)
                 );
                 CREATE INDEX IF NOT EXISTS entitlement_acks_worker
                     ON entitlement_acknowledgements (worker_pid);
                 CREATE INDEX IF NOT EXISTS entitlement_acks_entitlement
                     ON entitlement_acknowledgements (entitlement_pid);;
