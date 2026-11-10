-- Migration: `workers.location` — an optional free-text work location
-- (office, site, or city) so the org chart can be viewed by location.
-- Nullable: an absent location is "unknown", never a default place.

ALTER TABLE workers ADD COLUMN IF NOT EXISTS location VARCHAR NULL;
