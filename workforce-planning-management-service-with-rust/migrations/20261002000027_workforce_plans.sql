-- Migration: workforce plans and demand lines (WPM-R36, WPM-D26). A plan
-- is a *draft world*: aggregate hypothetical headcount per department
-- (optionally per role profile) at target dates. It never references a
-- worker row, so a scenario cannot leak into live records and a
-- discarded one leaves no residue.

CREATE TABLE IF NOT EXISTS workforce_plans (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 name VARCHAR NOT NULL,
                 organization_ref VARCHAR NOT NULL,
                 horizon_start DATE NOT NULL,
                 horizon_end DATE NOT NULL,
                 rationale VARCHAR NULL,
                 attrition_bp INTEGER NULL,
                 status VARCHAR NOT NULL,
                 deleted_at TIMESTAMPTZ NULL
             );

CREATE UNIQUE INDEX IF NOT EXISTS workforce_plans_one_active ON workforce_plans (organization_ref) WHERE status = 'active' AND deleted_at IS NULL;

CREATE TABLE IF NOT EXISTS plan_demand_lines (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 plan_pid UUID NOT NULL,
                 department VARCHAR NOT NULL,
                 role_profile_pid UUID NULL,
                 target_on DATE NOT NULL,
                 target_headcount INTEGER NOT NULL,
                 note VARCHAR NULL
             );

CREATE UNIQUE INDEX IF NOT EXISTS plan_demand_lines_key ON plan_demand_lines (plan_pid, department, COALESCE(role_profile_pid, '00000000-0000-0000-0000-000000000000'::uuid), target_on);
