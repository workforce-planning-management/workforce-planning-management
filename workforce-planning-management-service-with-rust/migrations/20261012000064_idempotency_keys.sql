-- Migration: **idempotency keys** (WPM-R132, WPM-D80). One row per (caller, key): the fingerprint of
-- the request that claimed the key and, once it finishes, the response to replay. A row lives for one
-- calendar day and is then deleted, so the stored response (which may hold what the first response
-- held) is not kept. It is not tied to a person, so it is neither exported nor erased with one.

CREATE TABLE IF NOT EXISTS idempotency_keys (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 actor VARCHAR NOT NULL,
                 idem_key VARCHAR NOT NULL,
                 fingerprint VARCHAR NOT NULL,
                 status_code INTEGER NULL,
                 content_type VARCHAR NULL,
                 response_body BYTEA NULL,
                 UNIQUE (actor, idem_key)
             );

CREATE INDEX IF NOT EXISTS idempotency_keys_created_at ON idempotency_keys (created_at);
