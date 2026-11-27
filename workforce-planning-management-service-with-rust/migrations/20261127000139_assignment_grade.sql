-- Migration: **an assignment carries its grade** (WPM-R255, WPM-D208, WPM-T558). The band and pay point an assignment is graded at,
-- as a reference to a published pay scale and never an amount. An assignment is a person's, so the grade travels with it: in the
-- subject-access export with the assignment, and gone with it on erasure. A change of grade is a change of the assignment, so it ends
-- one and starts another like any other change (WPM-D112). Unrecorded is unknown, never the entry point.

ALTER TABLE assignments ADD COLUMN IF NOT EXISTS grade_scale VARCHAR NULL;
ALTER TABLE assignments ADD COLUMN IF NOT EXISTS grade_band VARCHAR NULL;
ALTER TABLE assignments ADD COLUMN IF NOT EXISTS grade_pay_point INTEGER NULL;

ALTER TABLE assignments DROP CONSTRAINT IF EXISTS assignments_grade_whole;
ALTER TABLE assignments ADD CONSTRAINT assignments_grade_whole CHECK (
    (grade_scale IS NULL) = (grade_band IS NULL)
    AND (grade_pay_point IS NULL OR (grade_band IS NOT NULL AND grade_pay_point >= 1))
);
