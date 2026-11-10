-- Migration: **a CPD entry remembers its course** (WPM-R138). An entry that came from the learning
-- system now carries the course it was for, so a link to the course is built from data and not
-- from a sentence. Entries already there are filled from the sentence the sync wrote.

ALTER TABLE cpd_entries ADD COLUMN IF NOT EXISTS course_ref VARCHAR;

UPDATE cpd_entries SET course_ref = substring(activity FROM 'LMS completion: (.*)$')
               WHERE source = 'lms' AND course_ref IS NULL AND activity LIKE 'LMS completion: %';
