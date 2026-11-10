-- Migration: **tolerances on a workforce plan** (WPM-R140). One row per plan and dimension: the
-- largest deviation allowed, and the share of it at which green turns amber.

CREATE TABLE IF NOT EXISTS plan_tolerances (
                     created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                     updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                     id SERIAL PRIMARY KEY,
                     pid UUID NOT NULL UNIQUE,
                     plan_pid UUID NOT NULL,
                     dimension VARCHAR NOT NULL
                         CHECK (dimension IN ('time', 'cost', 'resources', 'risk')),
                     limit_value BIGINT NOT NULL CHECK (limit_value BETWEEN 0 AND 1000000),
                     amber_percent INTEGER NOT NULL DEFAULT 75
                         CHECK (amber_percent BETWEEN 1 AND 99),
                     UNIQUE (plan_pid, dimension)
                 );
