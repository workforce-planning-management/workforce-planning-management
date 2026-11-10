-- Migration: **dotted-line reports**. A worker can have any number of
-- dotted-line managers — a secondary, usually functional or project,
-- reporting relationship alongside the solid `manager_pid` line. It can be
-- anyone (not limited to the solid-line chain), is dated like other history,
-- and never changes the solid-line org chart.

CREATE TABLE IF NOT EXISTS dotted_line_reports (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 report_pid UUID NOT NULL,
                 manager_pid UUID NOT NULL,
                 note VARCHAR NULL,
                 started_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 ended_at TIMESTAMPTZ NULL,
                 recorded_by VARCHAR NULL,
                 on_behalf BOOLEAN NOT NULL DEFAULT false,
                 CHECK (report_pid <> manager_pid)
             );

CREATE UNIQUE INDEX IF NOT EXISTS dotted_line_reports_current ON dotted_line_reports (report_pid, manager_pid) WHERE ended_at IS NULL;

CREATE INDEX IF NOT EXISTS dotted_line_reports_manager ON dotted_line_reports (manager_pid);
