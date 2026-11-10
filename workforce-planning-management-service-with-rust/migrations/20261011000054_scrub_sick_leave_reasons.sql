-- Migration: **scrub the reason from sick-leave requests** (WPM-R118, WPM-D73).
--
-- A sick-leave request records that it is sick leave and its dates, never why: a diagnosis is
-- special-category health data the service has no need to hold, and the field was free text.
-- The service now refuses a reason on sick leave; this migration removes the ones already
-- stored. It is **one-way**: the text is gone, and `down` cannot restore it. Other kinds of
-- leave keep their reason. Take a backup first, as for every upgrade.

UPDATE leave_requests SET reason = NULL WHERE kind = 'sick' AND reason IS NOT NULL;
