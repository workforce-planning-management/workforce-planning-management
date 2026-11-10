-- Migration: **coverage evidence** (WPM-R220, WPM-D154, WPM-T420). For an organization and a jurisdiction, the workers' compensation or
-- employer's liability cover that organization holds: its kind, who insures it, the policy reference, the minimum cover, when it is
-- valid, and a *reference* to the certificate (never a copy of it). Records the fact and its expiry; pays and adjudicates nothing.

CREATE TABLE IF NOT EXISTS coverage_evidence (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 organization_ref VARCHAR(255) NOT NULL,
                 jurisdiction VARCHAR(20) NOT NULL CHECK (jurisdiction ~ '^[A-Z0-9][A-Z0-9-]{1,19}$'),
                 kind VARCHAR NOT NULL CHECK (kind IN ('statutory_scheme', 'private_insurance', 'self_insured', 'elective_none')),
                 insurer VARCHAR CHECK (insurer IS NULL OR length(btrim(insurer)) BETWEEN 1 AND 200),
                 policy_reference VARCHAR CHECK (policy_reference IS NULL OR length(btrim(policy_reference)) BETWEEN 1 AND 100),
                 minimum_cover_minor BIGINT CHECK (minimum_cover_minor IS NULL OR minimum_cover_minor >= 0),
                 currency VARCHAR(3) CHECK (currency IS NULL OR currency ~ '^[A-Z]{3}$'),
                 valid_from DATE,
                 valid_to DATE,
                 certificate_ref VARCHAR CHECK (certificate_ref IS NULL OR length(btrim(certificate_ref)) BETWEEN 1 AND 255),
                 recorded_by VARCHAR,
                 retired_on DATE,
                 CHECK (minimum_cover_minor IS NULL OR currency IS NOT NULL),
                 CHECK (valid_from IS NULL OR valid_to IS NULL OR valid_from <= valid_to)
             );

CREATE INDEX IF NOT EXISTS coverage_evidence_by_org ON coverage_evidence (organization_ref, jurisdiction);
