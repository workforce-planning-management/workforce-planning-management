-- Migration: a position's **overtime status** (WPM-R219, WPM-D157, WPM-T417). Whether a post is exempt from statutory overtime is a
-- fact a person records, never one WPM infers from duties, so the column defaults to `unknown` and says so until someone records it.

ALTER TABLE positions ADD COLUMN IF NOT EXISTS overtime_status VARCHAR NOT NULL DEFAULT 'unknown'
                     CHECK (overtime_status IN ('exempt', 'non_exempt', 'unknown'));
