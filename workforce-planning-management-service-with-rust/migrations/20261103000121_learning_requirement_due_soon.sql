-- Migration: a learning requirement's own **due-soon window** (WPM-D168, WPM-T434). Null means the deployment's window applies
-- (`WPM_LEARNING_DUE_SOON_DAYS`, else 90 calendar days); a number is that requirement's window, 1 to 366.

ALTER TABLE learning_requirements ADD COLUMN IF NOT EXISTS due_soon_days INTEGER
                     CHECK (due_soon_days IS NULL OR due_soon_days BETWEEN 1 AND 366);
