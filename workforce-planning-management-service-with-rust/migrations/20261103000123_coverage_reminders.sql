-- Migration: the record of coverage-expiry reminders already sent (WPM-R220, WPM-T420), so a re-run or two overlapping schedules tell
-- no one twice. One row per coverage record, end date and kind; a renewal with a new end date is a new reminder. It holds no person.

CREATE TABLE IF NOT EXISTS coverage_reminders (
                     created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                     updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                     id SERIAL PRIMARY KEY,
                     coverage_pid UUID NOT NULL,
                     valid_to DATE NOT NULL,
                     kind VARCHAR NOT NULL CHECK (kind IN ('expiring', 'expired')),
                     reminded_on DATE NOT NULL,
                     UNIQUE (coverage_pid, valid_to, kind)
                 );
