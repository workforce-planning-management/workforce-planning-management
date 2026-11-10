-- Migration: benefits awareness (WPM-R26) — generalise the wellbeing
-- entitlement rules with a closed `kind` (`health | benefit`) and an
-- optional link to a benefit plan, so a `benefit` rule can signpost
-- enrolment. The predicate vocabulary is untouched (WPM-D17), and
-- enrolment state is never stored on the rule (WPM-D18).

ALTER TABLE wellbeing_entitlements
                     ADD COLUMN IF NOT EXISTS kind VARCHAR NOT NULL DEFAULT 'health',
                     ADD COLUMN IF NOT EXISTS benefit_plan_pid UUID NULL;;
