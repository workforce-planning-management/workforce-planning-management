-- Migration: **cross-organization groups**. `groups.scope` is `organization`
-- (the default: one organization) or `confederation` (a community that spans
-- the organization named in `organization_ref` and every organization beneath
-- it in the confederation tree). Existing groups stay `organization`.

ALTER TABLE groups ADD COLUMN IF NOT EXISTS scope VARCHAR NOT NULL DEFAULT 'organization';
