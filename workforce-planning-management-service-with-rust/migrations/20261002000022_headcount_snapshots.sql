-- Migration: `headcount_snapshots` — an append-only, aggregate-only
-- history of employed headcount per organization × department × date,
-- so a forecast has a trend to project from. History cannot be
-- reconstructed after the fact (a worker's department or FTE changes
-- in place), which is why the table exists before anything reads it.
-- No worker identity, salary, or sensitive attribute is stored.

CREATE TABLE IF NOT EXISTS headcount_snapshots (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 organization_ref VARCHAR NOT NULL,
                 department VARCHAR NOT NULL,
                 as_of DATE NOT NULL,
                 headcount INTEGER NOT NULL,
                 fte_percent_total BIGINT NOT NULL,
                 starters INTEGER NULL,
                 leavers INTEGER NULL
             );

CREATE UNIQUE INDEX IF NOT EXISTS headcount_snapshots_key ON headcount_snapshots (organization_ref, department, as_of);
