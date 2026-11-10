-- Migration: **the framework licence record** (WPM-R113) — the version
-- imported, whether its wording may be redistributed, the licence holder,
-- the licence reference, when the licence was last checked, and (already
-- present) the attribution. `redistribution` defaults to `unknown`, which
-- the pure `rules::framework::publishable` treats as a no. The two
-- frameworks already imported are open (OGL v3.0; Commission Decision
-- 2011/833/EU) and are set so.

ALTER TABLE capability_frameworks ADD COLUMN IF NOT EXISTS version VARCHAR NULL;

ALTER TABLE capability_frameworks ADD COLUMN IF NOT EXISTS redistribution VARCHAR NOT NULL DEFAULT 'unknown';

ALTER TABLE capability_frameworks ADD COLUMN IF NOT EXISTS licence_holder VARCHAR NULL;

ALTER TABLE capability_frameworks ADD COLUMN IF NOT EXISTS licence_reference VARCHAR NULL;

ALTER TABLE capability_frameworks ADD COLUMN IF NOT EXISTS licence_checked_on DATE NULL;

ALTER TABLE capability_frameworks DROP CONSTRAINT IF EXISTS capability_frameworks_redistribution_chk;

ALTER TABLE capability_frameworks ADD CONSTRAINT capability_frameworks_redistribution_chk CHECK (redistribution IN ('open', 'internal_only', 'unknown'));

UPDATE capability_frameworks SET redistribution = 'open' WHERE slug IN ('uk-gdad-pcf', 'esco') AND redistribution = 'unknown';
