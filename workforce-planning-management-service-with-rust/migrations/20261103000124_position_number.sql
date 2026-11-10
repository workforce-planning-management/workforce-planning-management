-- Migration: a position's **number** (WPM-T434). The number a post is known by in an establishment list, so a learning requirement's
-- audience can name positions. It is optional and unique within an organization; a position with no number is not matched by a
-- position audience (the answer is `unknown`, never a guess).

ALTER TABLE positions ADD COLUMN IF NOT EXISTS number VARCHAR
                 CHECK (number IS NULL OR length(btrim(number)) BETWEEN 1 AND 40);

CREATE UNIQUE INDEX IF NOT EXISTS positions_number_unique ON positions (organization_ref, number) WHERE number IS NOT NULL;
