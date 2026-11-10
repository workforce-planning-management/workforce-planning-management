-- Migration: aspiration **visibility tiers**. `shared` (a boolean: private, or
-- visible to anyone who can view the record) becomes `visibility`:
-- `private` (just the person), `manager` (their management chain — the
-- people above them in `manager_pid`), or `everyone`. Existing shared
-- aspirations keep the visibility they had: `everyone`.

ALTER TABLE worker_aspirations ADD COLUMN IF NOT EXISTS visibility VARCHAR NOT NULL DEFAULT 'private';

DO $$ BEGIN IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'worker_aspirations' AND column_name = 'shared') THEN UPDATE worker_aspirations SET visibility = 'everyone' WHERE shared; ALTER TABLE worker_aspirations DROP COLUMN shared; END IF; END $$;
