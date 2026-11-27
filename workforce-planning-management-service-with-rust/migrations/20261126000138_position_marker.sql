-- Migration: **a position says whether it is critical and what it is graded at** (WPM-R255, WPM-R290, WPM-D237, WPM-T558, WPM-T660).
-- The owner decided on 2026-11-26 that positions carry a critical-role marker and a band, so the vacancy age and the continuity of
-- critical posts can be reported by band and for critical posts. `critical` is a statement HR makes about the post, never derived from
-- who holds it; it is the list succession on positions reads (WPM-R197: one list, not two). The grade is a reference to a published
-- pay scale (scale, band, optional pay point), never an amount: a position still has no pay column (WPM-D104, WPM-D208). An
-- unrecorded grade is unknown, never the entry point.

ALTER TABLE positions ADD COLUMN IF NOT EXISTS critical BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE positions ADD COLUMN IF NOT EXISTS grade_scale VARCHAR NULL;
ALTER TABLE positions ADD COLUMN IF NOT EXISTS grade_band VARCHAR NULL;
ALTER TABLE positions ADD COLUMN IF NOT EXISTS grade_pay_point INTEGER NULL;

ALTER TABLE positions DROP CONSTRAINT IF EXISTS positions_grade_whole;
ALTER TABLE positions ADD CONSTRAINT positions_grade_whole CHECK (
    (grade_scale IS NULL) = (grade_band IS NULL)
    AND (grade_pay_point IS NULL OR (grade_band IS NOT NULL AND grade_pay_point >= 1))
);
