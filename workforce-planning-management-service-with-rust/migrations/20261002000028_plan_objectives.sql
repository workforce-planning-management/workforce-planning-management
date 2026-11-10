-- Migration: strategic objectives for a plan and their links to demand
-- lines (WPM-R38), so headcount and competencies can be shown to serve —
-- or not serve — the strategy.

CREATE TABLE IF NOT EXISTS plan_objectives (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 plan_pid UUID NOT NULL,
                 title VARCHAR NOT NULL,
                 owner VARCHAR NULL
             );

CREATE TABLE IF NOT EXISTS demand_line_objectives (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 line_pid UUID NOT NULL,
                 objective_pid UUID NOT NULL
             );

CREATE UNIQUE INDEX IF NOT EXISTS demand_line_objectives_key ON demand_line_objectives (line_pid, objective_pid);
