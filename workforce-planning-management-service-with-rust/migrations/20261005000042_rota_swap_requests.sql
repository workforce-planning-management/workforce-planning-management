-- Migration: **rota swap requests**. A person on call asks a colleague to
-- take their on-call days in a window; if the colleague accepts, the days
-- the requester was on call become overrides for the colleague.

CREATE TABLE IF NOT EXISTS rota_swap_requests (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 rota_pid UUID NOT NULL,
                 requester_pid UUID NOT NULL,
                 taker_pid UUID NOT NULL,
                 starts_on DATE NOT NULL,
                 ends_on DATE NOT NULL,
                 note VARCHAR NULL,
                 status VARCHAR NOT NULL DEFAULT 'requested',
                 decided_at TIMESTAMPTZ NULL,
                 created_by VARCHAR NULL,
                 CHECK (requester_pid <> taker_pid),
                 CHECK (ends_on >= starts_on),
                 CHECK (status IN ('requested', 'accepted', 'declined', 'cancelled'))
             );

CREATE INDEX IF NOT EXISTS rota_swap_requests_rota ON rota_swap_requests (rota_pid);

CREATE INDEX IF NOT EXISTS rota_swap_requests_people ON rota_swap_requests (requester_pid, taker_pid);
