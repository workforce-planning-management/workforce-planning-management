-- Migration: `requisitions.filled_on` — the date a requisition moved to
-- `filled`, so time-to-fill (`filled_on - opened_on`) can be derived.
-- Nullable: requisitions filled before this column existed have no
-- recorded fill date and are left out of time-to-fill, not guessed.

ALTER TABLE requisitions ADD COLUMN IF NOT EXISTS filled_on DATE NULL;
