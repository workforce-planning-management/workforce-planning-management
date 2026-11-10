-- Migration: **on-call rotas**. A rota is a named rotation of workers in one
-- organization: every `period_days` the duty passes to the next member in
-- order, counting from `starts_on`. Overrides cover swaps — a named worker
-- on call for a date window regardless of the rotation.

CREATE TABLE IF NOT EXISTS rotas (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 organization_ref VARCHAR NOT NULL,
                 name VARCHAR NOT NULL,
                 description VARCHAR NULL,
                 period_days INTEGER NOT NULL,
                 starts_on DATE NOT NULL,
                 deleted_at TIMESTAMPTZ NULL,
                 CHECK (period_days BETWEEN 1 AND 31)
             );

CREATE UNIQUE INDEX IF NOT EXISTS rotas_org_name ON rotas (organization_ref, lower(name)) WHERE deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS rota_members (
                 id SERIAL PRIMARY KEY,
                 rota_pid UUID NOT NULL,
                 worker_pid UUID NOT NULL,
                 position INTEGER NOT NULL,
                 UNIQUE (rota_pid, worker_pid),
                 UNIQUE (rota_pid, position)
             );

CREATE INDEX IF NOT EXISTS rota_members_worker ON rota_members (worker_pid);

CREATE TABLE IF NOT EXISTS rota_overrides (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 rota_pid UUID NOT NULL,
                 worker_pid UUID NOT NULL,
                 starts_on DATE NOT NULL,
                 ends_on DATE NOT NULL,
                 note VARCHAR NULL,
                 created_by VARCHAR NULL,
                 CHECK (ends_on >= starts_on)
             );

CREATE INDEX IF NOT EXISTS rota_overrides_rota ON rota_overrides (rota_pid);
