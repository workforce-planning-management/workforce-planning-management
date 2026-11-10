-- Migration: **announcements**. A feed of company or organization news:
-- a title and body, optionally pinned, optionally scheduled
-- (`publish_on`) and expiring (`expires_on`).

CREATE TABLE IF NOT EXISTS announcements (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 organization_ref VARCHAR NOT NULL,
                 title VARCHAR NOT NULL,
                 body VARCHAR NOT NULL,
                 pinned BOOLEAN NOT NULL DEFAULT false,
                 publish_on DATE NOT NULL DEFAULT CURRENT_DATE,
                 expires_on DATE NULL,
                 author VARCHAR NULL,
                 deleted_at TIMESTAMPTZ NULL,
                 CHECK (expires_on IS NULL OR expires_on >= publish_on)
             );

CREATE INDEX IF NOT EXISTS announcements_org ON announcements (organization_ref);
