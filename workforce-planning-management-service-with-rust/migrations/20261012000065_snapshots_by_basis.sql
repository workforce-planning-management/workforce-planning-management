-- Migration: split the headcount snapshots by basis (WPM-R84, WPM-T139) — contractor and fixed-term
-- headcount and FTE, so the contingent share has a trend. The columns are nullable: a row recorded
-- before this change reports the split as unknown, never as zero. Aggregates only; no one is named.

ALTER TABLE headcount_snapshots ADD COLUMN IF NOT EXISTS contractor_headcount INTEGER NULL, ADD COLUMN IF NOT EXISTS contractor_fte_percent_total BIGINT NULL, ADD COLUMN IF NOT EXISTS fixed_term_headcount INTEGER NULL, ADD COLUMN IF NOT EXISTS fixed_term_fte_percent_total BIGINT NULL;
