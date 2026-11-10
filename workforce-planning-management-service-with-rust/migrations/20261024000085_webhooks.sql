-- Migration: **outbound webhooks** (WPM-R151, WPM-D106). A subscription is an endpoint, the event kinds it wants and the name of
-- the environment variable that holds its signing secret (a reference, never the secret). A delivery is one event for one
-- subscription: its kind and the URN it is about, and how the attempts went. **There is no column for a payload body**, a name
-- or a personal field: the body is rebuilt from the reference when it is sent.

CREATE TABLE IF NOT EXISTS webhook_subscriptions (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 name VARCHAR NOT NULL CHECK (length(btrim(name)) BETWEEN 1 AND 80),
                 url VARCHAR NOT NULL CHECK (length(url) BETWEEN 8 AND 500),
                 kinds TEXT[] NOT NULL CHECK (cardinality(kinds) >= 1),
                 secret_ref VARCHAR NOT NULL CHECK (secret_ref ~ '^WPM_WEBHOOK_SECRET_[A-Z0-9_]{1,60}$'),
                 active BOOLEAN NOT NULL DEFAULT TRUE,
                 created_by VARCHAR
             );

CREATE TABLE IF NOT EXISTS webhook_deliveries (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 subscription_pid UUID NOT NULL REFERENCES webhook_subscriptions (pid) ON DELETE CASCADE,
                 event_id UUID NOT NULL,
                 kind VARCHAR NOT NULL,
                 subject VARCHAR NOT NULL CHECK (length(subject) <= 200),
                 occurred_at TIMESTAMPTZ NOT NULL,
                 status VARCHAR NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'delivered', 'failed')),
                 attempts INTEGER NOT NULL DEFAULT 0 CHECK (attempts >= 0),
                 last_status INTEGER,
                 last_error VARCHAR CHECK (last_error IS NULL OR length(last_error) <= 200),
                 next_attempt_at TIMESTAMPTZ,
                 delivered_at TIMESTAMPTZ,
                 UNIQUE (subscription_pid, event_id)
             );

CREATE INDEX IF NOT EXISTS webhook_deliveries_due ON webhook_deliveries (next_attempt_at) WHERE status = 'pending';
