-- Migration: a workforce plan's financial assumptions — an optional annual
-- budget (minor units + ISO-4217 currency) and an employer on-cost
-- percentage — so a plan can be costed and compared with what is affordable.

ALTER TABLE workforce_plans ADD COLUMN IF NOT EXISTS budget_minor BIGINT NULL;

ALTER TABLE workforce_plans ADD COLUMN IF NOT EXISTS budget_currency VARCHAR NULL;

ALTER TABLE workforce_plans ADD COLUMN IF NOT EXISTS on_cost_bp INTEGER NULL;
