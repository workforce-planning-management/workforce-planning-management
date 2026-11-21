-- Migration: **per-person allocation** (WPM-R275, WPM-R276, WPM-D211, WPM-T598). A share of one of a worker's assignments planned for
-- a programme, product or service over a run of months. It records what a person is planned to do, never what they did or how well:
-- there is no column for effort, hours, a rating or a utilization figure. The worker is held as well as the assignment so that the
-- subject-access export and erasure find every row by person. Soft-deleted, so the retention sweep removes a deleted row after its
-- horizon.
--
-- Tracking is switched on per organization (`capacity_settings.allocation_tracked`): where it is off, a person's standing is
-- unknown, never zero (WPM-D46). Per pool and per unit waits for organizational units (WPM-R257).

CREATE TABLE IF NOT EXISTS allocations (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 worker_pid UUID NOT NULL,
                 assignment_pid UUID NOT NULL,
                 organization_ref VARCHAR NOT NULL,
                 source_ref VARCHAR NOT NULL CHECK (char_length(source_ref) BETWEEN 3 AND 200),
                 pool_pid UUID NULL,
                 from_month DATE NOT NULL CHECK (EXTRACT(DAY FROM from_month) = 1),
                 to_month DATE NOT NULL CHECK (EXTRACT(DAY FROM to_month) = 1),
                 fte_centi INTEGER NOT NULL CHECK (fte_centi BETWEEN 1 AND 100),
                 status VARCHAR NOT NULL DEFAULT 'tentative'
                     CHECK (status IN ('tentative', 'confirmed', 'ended')),
                 note VARCHAR NULL CHECK (note IS NULL OR char_length(note) <= 500),
                 recorded_by VARCHAR NULL,
                 deleted_at TIMESTAMPTZ NULL,
                 CHECK (to_month >= from_month)
             );

CREATE INDEX IF NOT EXISTS allocations_worker ON allocations (worker_pid) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS allocations_organization_months ON allocations (organization_ref, from_month, to_month)
                 WHERE deleted_at IS NULL;
-- The retention report and sweep read only the deleted rows (WPM-R142).
CREATE INDEX IF NOT EXISTS allocations_deleted_at ON allocations (deleted_at) WHERE deleted_at IS NOT NULL;

ALTER TABLE capacity_settings ADD COLUMN IF NOT EXISTS allocation_tracked BOOLEAN NOT NULL DEFAULT FALSE;
