-- Migration: `organization_confederations` — a parent organization
-- that transitively contains member organizations (WPM-Rxx: org
-- confederation). One edge per direct parent→child relationship;
-- arbitrary-depth nesting (a confederation of confederations) is
-- expressed by chaining edges, walked in Rust at read time
-- (`crate::rules::org_access::descendants_of`), not recursive SQL —
-- this table stays a small, demo-scale edge list, matching
-- `organization_memberships`'s own "walk in Rust" convention.
-- `parent_organization_ref`/`child_organization_ref` are upstream
-- `EntityRef` URNs, per this service's identity-boundary convention
-- (AGENTS.md) — this table never owns organization identity, only the
-- reference to it, exactly like `organization_memberships`.

CREATE TABLE IF NOT EXISTS organization_confederations (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 parent_organization_ref VARCHAR NOT NULL,
                 child_organization_ref VARCHAR NOT NULL,
                 starts_on DATE NOT NULL,
                 ends_on DATE NULL,
                 deleted_at TIMESTAMPTZ NULL
             );

CREATE UNIQUE INDEX IF NOT EXISTS organization_confederations_key ON organization_confederations (parent_organization_ref, child_organization_ref) WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS organization_confederations_parent ON organization_confederations (parent_organization_ref);

CREATE INDEX IF NOT EXISTS organization_confederations_child ON organization_confederations (child_organization_ref);
