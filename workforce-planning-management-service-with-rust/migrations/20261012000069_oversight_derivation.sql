-- Migration: **derived oversight measures** (WPM-R67, WPM-T114). A derived measure names which
-- computation supplies its readings; a reported measure names none. The pairing is a CHECK, so a
-- derived measure cannot exist without its derivation and a reported one cannot carry a stale key.

ALTER TABLE oversight_measures ADD COLUMN IF NOT EXISTS derivation VARCHAR;

ALTER TABLE oversight_measures DROP CONSTRAINT IF EXISTS oversight_measures_derivation_pairs;

ALTER TABLE oversight_measures ADD CONSTRAINT oversight_measures_derivation_pairs CHECK ((origin = 'derived') = (derivation IS NOT NULL));
