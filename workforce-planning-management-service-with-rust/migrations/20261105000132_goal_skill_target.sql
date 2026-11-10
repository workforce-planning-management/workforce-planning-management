-- Migration: **a goal's skill target** (WPM-R234, WPM-T462). The level a skill-building goal aims at, so the goal can show the person's
-- declared level beside it. Null means the goal names no target, and nothing is claimed either way.

ALTER TABLE worker_goals ADD COLUMN skill_target_level integer
                   CHECK (skill_target_level BETWEEN 1 AND 5);
