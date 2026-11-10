-- Migration: **announcement extras** — a department audience, link
-- attachments, and read receipts.
--
-- - `announcements.department`: when set, only that department (and the
--   organization's editors) see the post; `NULL` is everyone.
-- - `announcements.links`: up to three `{label, url}` links (https only,
--   validated by the service).
-- - `announcement_reads`: who has read a post. The reader sees their own;
--   editors see only a **count**, never who.

ALTER TABLE announcements ADD COLUMN IF NOT EXISTS department VARCHAR NULL;

ALTER TABLE announcements ADD COLUMN IF NOT EXISTS links JSONB NOT NULL DEFAULT '[]'::jsonb;

CREATE TABLE IF NOT EXISTS announcement_reads (
                 id SERIAL PRIMARY KEY,
                 announcement_pid UUID NOT NULL,
                 worker_pid UUID NOT NULL,
                 read_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 UNIQUE (announcement_pid, worker_pid)
             );

CREATE INDEX IF NOT EXISTS announcement_reads_worker ON announcement_reads (worker_pid);
