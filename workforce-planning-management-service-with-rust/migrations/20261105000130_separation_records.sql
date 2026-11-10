-- Migration: **the separation record** (WPM-R222, WPM-T422). The last day and the reason stay on the leaver record; this holds only
-- what that record does not: the final pay date and a reference to the pay summary, which is a pointer and never an amount.

CREATE TABLE IF NOT EXISTS separation_records (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 movement_pid UUID NOT NULL UNIQUE,
                 worker_pid UUID NOT NULL,
                 final_pay_on DATE,
                 pay_summary_ref VARCHAR,
                 recorded_by VARCHAR
             );

CREATE INDEX IF NOT EXISTS separation_records_worker ON separation_records (worker_pid);
