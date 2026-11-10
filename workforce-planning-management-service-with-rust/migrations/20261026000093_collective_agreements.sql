-- Migration: **collective agreements** as reference data (WPM-R103, WPM-D68). An agreement is a dated, sourced, versioned document the
-- organization has with a recognised body, tied to one bargaining unit, with the few terms WPM can use: a pay scale reference, a weekly
-- hours limit, a notice period and an agreed consultation procedure. Like a pay scale it is transcribed and cited, not interpreted.
-- A new version is a new row. **There is no column for who an agreement applies to**: that is derived from the unit's rules.

CREATE TABLE IF NOT EXISTS collective_agreements (
                     created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                     updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
                     id SERIAL PRIMARY KEY,
                     pid UUID NOT NULL UNIQUE,
                     unit_pid UUID NOT NULL,
                     title VARCHAR NOT NULL CHECK (length(btrim(title)) BETWEEN 1 AND 120),
                     version INTEGER NOT NULL CHECK (version >= 1),
                     effective_from DATE NOT NULL,
                     effective_to DATE,
                     source VARCHAR NOT NULL CHECK (length(btrim(source)) BETWEEN 1 AND 300),
                     pay_scale_id VARCHAR CHECK (pay_scale_id IS NULL OR length(btrim(pay_scale_id)) BETWEEN 1 AND 80),
                     max_weekly_hours INTEGER CHECK (max_weekly_hours IS NULL OR max_weekly_hours BETWEEN 1 AND 168),
                     notice_weeks INTEGER CHECK (notice_weeks IS NULL OR notice_weeks BETWEEN 0 AND 104),
                     consultation_days INTEGER CHECK (consultation_days IS NULL OR consultation_days BETWEEN 1 AND 365),
                     recorded_by VARCHAR,
                     CHECK (effective_to IS NULL OR effective_to >= effective_from),
                     UNIQUE (unit_pid, title, version)
                 );
