-- Migration: **withdrawing a monthly pack** (WPM-R69, WPM-T116). A frozen pack is never changed or
-- removed, so a mistaken one is *withdrawn*: a separate, append-only row says so and why, and every
-- reader sees the mark beside the unchanged payload.

CREATE TABLE IF NOT EXISTS oversight_pack_withdrawals (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 pack_pid UUID NOT NULL UNIQUE,
                 withdrawn_on DATE NOT NULL,
                 reason VARCHAR NOT NULL,
                 CHECK (length(btrim(reason)) > 0)
             );

DROP TRIGGER IF EXISTS oversight_pack_withdrawals_no_change ON oversight_pack_withdrawals;

CREATE TRIGGER oversight_pack_withdrawals_no_change BEFORE UPDATE OR DELETE ON oversight_pack_withdrawals
                 FOR EACH ROW EXECUTE FUNCTION oversight_packs_append_only();
