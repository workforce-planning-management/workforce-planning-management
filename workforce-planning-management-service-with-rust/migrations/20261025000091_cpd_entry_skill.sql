-- Migration: a CPD entry may name the skill it developed (WPM-R157). The link is what lets a credential issued from the entry carry
-- the skill's ESCO or SFIA reference. It is optional: an entry with no skill is issued with no alignment, not a guessed one.

ALTER TABLE cpd_entries ADD COLUMN IF NOT EXISTS skill_pid UUID;
