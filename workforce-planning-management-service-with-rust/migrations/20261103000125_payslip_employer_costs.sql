-- Migration: a payslip's **employer-side cost lines** (WPM-R223, WPM-D159, WPM-D170). The lines are a stub cost to the employer, never
-- deducted from net: `employer_costs` holds each line (name, class, rate, the pay it was applied to, the amount) and
-- `employer_cost_minor` their total. Both default to nothing, which is also what an organization with no lines configured has.

ALTER TABLE payslips ADD COLUMN IF NOT EXISTS employer_cost_minor BIGINT NOT NULL DEFAULT 0
                 CHECK (employer_cost_minor >= 0);

ALTER TABLE payslips ADD COLUMN IF NOT EXISTS employer_costs JSONB NOT NULL DEFAULT '[]'::jsonb;
