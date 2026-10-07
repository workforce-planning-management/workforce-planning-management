# Architecture

```
 employee / manager / HR browser
        │  (cookie session; no token in JS)
        ▼
 workforce-planning-management-ui-with-svelte  (SvelteKit BFF)
        │  Authorization: Bearer v4.public.…
        ▼
 workforce-planning-management-api-with-rust  (Loco: Axum + SeaORM + PostgreSQL)
        │  EntityRef lookups (read-only, cached, stub-able)
        ▼
 person / worker / organization / course / authentication services
```

## Service edition

Loco-idiomatic layout (the patient-flow shape — the closest sibling):

```
src/
├── app.rs                loco Hooks
├── controllers/          ~40 modules, one per area: hiring (acquisition),
│                         hr_core, workforce, payroll, learning,
│                         development, talent, assessments, appraisals,
│                         wellbeing, ergonomics, adjustments, privacy,
│                         intelligence (metrics, insights, capability),
│                         planning, roles, framework_roles, esco,
│                         organizations, groups, reporting, career,
│                         transfers, directory, contacts, rotas,
│                         rota_swaps, announcements, skill_gaps,
│                         training_plan, movements, handover, …
├── models/               helpers (+ notifications push) + _entities/
├── clients.rs            stub-first upstream lookups (display names,
│                         birth dates — cached, never stored)
├── rules/                pure core: every lifecycle machine
│                         (requisition, application, review, payroll
│                         run, mentorship, appraisal, adjustment,
│                         placement, …), leave/time/scheduling
│                         arithmetic, working-time guardrails,
│                         wellbeing eligibility + prompt machine,
│                         pulse k-floor, 360 group floor, assessment
│                         category↔scale map, DSE completion gate,
│                         erasure/retention rules, payslip
│                         arithmetic, org-chart cycle check
├── auth.rs               offline PASETO + ABAC + personas + mask
├── streaming.rs          envelope + memory/outbox transports
├── validation.rs         caps + tokens + URN shapes → 422
└── openapi.rs            OpenAPI 3 doc
├── tasks/                loco tasks: seed, snapshot_headcount,
│                         rota_reminders, pay_progression_reminders,
│                         import_framework, import_esco
migration/                sea-orm-migration (crate root, 49 sets)
config/abac-policy.reference.json   the shipped, matrix-verified
                                    persona policy (WPM-G1 runbook)
```

Key decisions (numbered in [design.md](design.md)): normalized
relational schema (constraints and lifecycles, not DTO-as-JSONB);
every state machine and money calculation in the DB-free pure core;
minor-unit money; ETag-conditional dashboards; family fixtures
(`#![forbid(unsafe_code)]`, clippy-pedantic, OTLP, `Accepts-version`,
Podman). **All-plural table names** (the loco `create_table`
pluralization gotcha is documented family knowledge).

## Front-end edition

SvelteKit 2 + Svelte 5 runes SPA + same-origin BFF proxy
(patient-flow/PPM pattern), dependency-light, **17 content locales
served under `/<locale>/` routes** (one full-code address each; a bare
`/` follows `navigator.languages`; a `reroute` hook; strings are
content in `content/locales/<locale>/ui.json`, edited through Sveltia CMS —
[locales spec](locales-for-global-sharing-with-svelte/index.md), WPM-R50). Views per
pillar: requisition/application boards, onboarding tracker, team
calendar + rota + working-time and ergonomic-issue panels, the
employee profile (a self-service hub: wellbeing prompts, pulse,
notifications, 360s + "my 360 requests", ergonomics, reasonable
adjustments, subject-access download, erase action) + org chart,
review and enrollment panels, `/wellbeing` (entitlement rules,
uptake, pulse results), `/privacy` (retention report + sweep),
payroll run screen, benchmarking table, and the HR dashboard — plus,
since WPM-T69: `/metrics` (with insights), `/directory`, `/rota`,
`/announcements`, `/skill-gaps`, `/movements`, `/planning`, `/roles`,
`/skills`, `/groups` and the one-screen `/ceo` dashboard (sized for an
iPad, 9th generation).
