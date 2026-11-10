-- Migration: **the framework's own level kept on declared skills and
-- requirements** (WPM-R111, WPM-D70). A declared skill gets the framework,
-- the level and the scale it was stated on; a requirement already had the
-- level and scale and now also names the framework, so a 4 of 7 is never
-- mistaken for a 4 of 4. WPM's 1–5 `proficiency` stays the figure every
-- comparison uses.

ALTER TABLE worker_skills ADD COLUMN IF NOT EXISTS source_framework VARCHAR NULL;

ALTER TABLE worker_skills ADD COLUMN IF NOT EXISTS source_level INTEGER NULL;

ALTER TABLE worker_skills ADD COLUMN IF NOT EXISTS source_scale_max INTEGER NULL;

ALTER TABLE role_skill_requirements ADD COLUMN IF NOT EXISTS source_framework VARCHAR NULL;

UPDATE role_skill_requirements SET source_framework = 'uk-gdad-pcf' WHERE source_level IS NOT NULL AND source_framework IS NULL;
