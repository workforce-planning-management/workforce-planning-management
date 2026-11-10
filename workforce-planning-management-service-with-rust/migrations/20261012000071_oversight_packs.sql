-- Migration: **monthly oversight packs** (WPM-R69, WPM-T116). A pack freezes the oversight position,
-- criteria, measures and capacity view as of a date. It is written once: a trigger refuses any later
-- update or delete, so the figures an oversight body read are the figures that stay on record.

CREATE TABLE IF NOT EXISTS oversight_packs (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 organization_ref VARCHAR NOT NULL,
                 as_of DATE NOT NULL,
                 month DATE NOT NULL,
                 payload JSONB NOT NULL,
                 CHECK (EXTRACT(DAY FROM month) = 1)
             );

CREATE INDEX IF NOT EXISTS oversight_packs_org ON oversight_packs (organization_ref, month);

CREATE OR REPLACE FUNCTION oversight_packs_append_only() RETURNS trigger AS $$
             BEGIN
                 RAISE EXCEPTION 'oversight_packs is append-only: a frozen pack is never changed or removed';
             END;
             $$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS oversight_packs_no_change ON oversight_packs;

CREATE TRIGGER oversight_packs_no_change BEFORE UPDATE OR DELETE ON oversight_packs
                 FOR EACH ROW EXECUTE FUNCTION oversight_packs_append_only();
