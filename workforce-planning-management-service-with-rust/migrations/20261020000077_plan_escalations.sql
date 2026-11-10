-- Migration: **escalations of a breached tolerance** (WPM-R141). One row per escalation: what was
-- breached and how far when it was raised, how far up it has gone, and what was decided.

CREATE TABLE IF NOT EXISTS plan_escalations (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 plan_pid UUID NOT NULL,
                 dimension VARCHAR NOT NULL
                     CHECK (dimension IN ('time', 'cost', 'resources', 'risk')),
                 level INTEGER NOT NULL DEFAULT 1 CHECK (level BETWEEN 1 AND 3),
                 status VARCHAR NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'decided')),
                 raised_by VARCHAR NULL,
                 raised_on DATE NOT NULL,
                 level_raised_on DATE NULL,
                 note VARCHAR NULL CHECK (char_length(note) <= 500),
                 rag_at_raise VARCHAR NOT NULL CHECK (rag_at_raise IN ('amber', 'red')),
                 deviation_at_raise BIGINT NULL,
                 limit_at_raise BIGINT NOT NULL,
                 decision VARCHAR NULL
                     CHECK (decision IN ('accept_deviation', 'change_plan', 'stop')),
                 decided_by VARCHAR NULL,
                 decided_on DATE NULL,
                 decision_note VARCHAR NULL CHECK (char_length(decision_note) <= 500),
                 rag_at_decision VARCHAR NULL,
                 deviation_at_decision BIGINT NULL,
                 CHECK ((status = 'open') = (decision IS NULL))
             );

CREATE UNIQUE INDEX IF NOT EXISTS plan_escalations_one_open
                 ON plan_escalations (plan_pid, dimension) WHERE status = 'open';

CREATE INDEX IF NOT EXISTS plan_escalations_plan ON plan_escalations (plan_pid);
