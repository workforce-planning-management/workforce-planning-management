-- Migration: **course requests** (WPM-R236, WPM-T464). A person asks to take a course whose recorded fee is not zero, and a manager
-- above them decides. At most one request for a course is open at a time. No fee is copied here, so what the manager reads is the
-- catalogue's, and no reason is kept for a decision.

CREATE TABLE IF NOT EXISTS course_requests (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 worker_pid UUID NOT NULL,
                 course_ref VARCHAR NOT NULL,
                 status VARCHAR NOT NULL DEFAULT 'requested'
                     CHECK (status IN ('requested', 'approved', 'declined', 'withdrawn')),
                 decided_by VARCHAR,
                 decided_on DATE
             );

CREATE INDEX IF NOT EXISTS course_requests_worker ON course_requests (worker_pid);

CREATE UNIQUE INDEX IF NOT EXISTS course_requests_one_open ON course_requests (worker_pid, course_ref)
                 WHERE status = 'requested';
