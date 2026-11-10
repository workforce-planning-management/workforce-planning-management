-- Migration: **cutover follow-up** (WPM-T117). A cutover may name its site more than one way
-- (`site_aliases`, a JSON list of text), and the early-warning table learns a third kind of
-- condition, a cutover whose people side is at risk.

ALTER TABLE cutovers ADD COLUMN IF NOT EXISTS site_aliases JSONB NOT NULL DEFAULT '[]'::jsonb;

ALTER TABLE cutovers DROP CONSTRAINT IF EXISTS cutovers_site_aliases_list;

ALTER TABLE cutovers ADD CONSTRAINT cutovers_site_aliases_list CHECK (jsonb_typeof(site_aliases) = 'array');

ALTER TABLE early_warnings DROP CONSTRAINT IF EXISTS early_warnings_kind_check;

ALTER TABLE early_warnings ADD CONSTRAINT early_warnings_kind_check
                 CHECK (kind IN ('pool_over_committed', 'measure_breach', 'cutover_at_risk'));
