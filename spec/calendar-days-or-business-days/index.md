# Calendar days or business days

Any duration counted in days says which kind of day it counts, immediately and
explicitly: **calendar days** or **business days**. A bare "days" is ambiguous
and is not used for a duration.

| Write | Not |
| --- | --- |
| within 30 calendar days | within 30 days |
| 5 business days' notice | 5 days' notice |
| a period of 1–31 calendar days | a period of 1–31 days |
| ±28 calendar days | ±28 days |
| a 30-calendar-day window | a 30-day window |

## Scope

- Spec files, requirements, design decisions, task entries, and plans.
- Documentation, READMEs, agent files, and changelogs.
- User-facing strings in every locale. A translation names the kind of day in
  its own language.
- API descriptions, OpenAPI text, error messages, and log messages.

## What the terms mean

- A **calendar day** is any day, including weekends and public holidays.
- A **business day** is Monday to Friday, excluding public holidays. A
  requirement that counts business days names the holiday calendar it uses, or
  says that it ignores public holidays.

## What the rule does not cover

- A single day that is not a duration, such as "on the last day", "a day's
  rota", or "the publish day".
- Other units: hours, weeks, months, and years.
- Existing code identifiers and API fields. A new identifier that holds a count
  of days names its kind, for example `calendar_days` or `business_days`.
