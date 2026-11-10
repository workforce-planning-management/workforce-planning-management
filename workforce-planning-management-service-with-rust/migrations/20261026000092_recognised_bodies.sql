-- Migration: **recognised bodies and bargaining units** (WPM-R102, WPM-D68). A body is a thing the *organization* recognises (a union, a
-- works council, a staff forum): a name, a kind, a basis, a date and a contact **role**. A bargaining unit is the part of the
-- workforce a body covers, defined by rules over facts WPM already holds. **There is no column for who belongs to a body, who
-- represents one, or who is covered**: coverage is computed, as a count with a floor, and never stored.

CREATE TABLE IF NOT EXISTS recognised_bodies (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 organization_ref VARCHAR NOT NULL,
                 name VARCHAR NOT NULL CHECK (length(btrim(name)) BETWEEN 1 AND 120),
                 kind VARCHAR NOT NULL CHECK (kind IN ('union', 'works_council', 'staff_forum')),
                 basis VARCHAR NOT NULL CHECK (basis IN ('voluntary_agreement', 'statutory', 'other')),
                 recognised_on DATE NOT NULL,
                 contact_role VARCHAR NOT NULL CHECK (length(btrim(contact_role)) BETWEEN 1 AND 80),
                 recorded_by VARCHAR,
                 UNIQUE (organization_ref, name)
             );

CREATE TABLE IF NOT EXISTS bargaining_units (
                 created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                 id SERIAL PRIMARY KEY,
                 pid UUID NOT NULL UNIQUE,
                 body_pid UUID NOT NULL,
                 name VARCHAR NOT NULL CHECK (length(btrim(name)) BETWEEN 1 AND 120),
                 department VARCHAR CHECK (department IS NULL OR length(btrim(department)) BETWEEN 1 AND 80),
                 location VARCHAR CHECK (location IS NULL OR length(btrim(location)) BETWEEN 1 AND 80),
                 employment_type VARCHAR CHECK (employment_type IS NULL OR length(btrim(employment_type)) BETWEEN 1 AND 80),
                 job_title VARCHAR CHECK (job_title IS NULL OR length(btrim(job_title)) BETWEEN 1 AND 80),
                 recorded_by VARCHAR,
                 CHECK (department IS NOT NULL OR location IS NOT NULL OR employment_type IS NOT NULL OR job_title IS NOT NULL),
                 UNIQUE (body_pid, name)
             );
