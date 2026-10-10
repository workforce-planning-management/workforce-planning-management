# Workforce Planning Management (WPM)

An **all-in-one HR platform** demonstrating the employee lifecycle
hire to retire: talent acquisition & onboarding, workforce management
(time, leave, scheduling, the on-call rota), HR service delivery (employee
record and directory, org chart, self-service, benefits, wellbeing,
ergonomics, subject rights), talent management & development (reviews, 360°s,
training, skills gaps and training time, succession), strategic workforce
planning, joiners and leavers, announcements, a one-screen CEO dashboard, and
payroll & compensation — in **17 locales**. See
[spec/index.md](spec/index.md) for the full specification.

> ⚠️ **Demo software.** Not a production HR or payroll system;
> statutory calculations are illustrative stubs; no real personal
> data anywhere in the repository. See
> [spec/regulatory.md](spec/regulatory.md).

**Status: the demonstration is implemented through WPM-T97 (2026-07-18 → 2026-10-07). Since
then: production-readiness, operations and information-governance work (WPM-T102, T119,
T123–T128, T144–T155), the delivery-capacity API (WPM-T104–T107) and two corrections
(WPM-T134, T136). Much more is proposed but not built: see
[spec/tasks.md](spec/tasks.md) and [spec/implementation-status.md](spec/implementation-status.md).**
Verified on 2026-10-11 for the service: 379 unit tests and 73 database-backed request tests,
with the enforcement (including need-to-know reads, contractor rates and expenses), security, retention-schedule and Entra suites. The Keycloak suite (real
Keycloak 26) and the front-end counts (80 unit tests and 45 Playwright specs) were last
measured on 2026-10-07 and were not rerun for this update. See each
subproject's own README for its own gate status, and [spec/testing.md](spec/testing.md).

## Subprojects

| Subproject | Role | Stack |
| --- | --- | --- |
| [workforce-planning-management-service-with-rust](workforce-planning-management-service-with-rust/) | Back-end JSON API | Rust, Loco (Axum + SeaORM), PostgreSQL |
| [workforce-planning-management-ui-with-svelte](workforce-planning-management-ui-with-svelte/) | HR / manager / self-service UI | SvelteKit 2, Svelte 5 runes, TypeScript |

Each subproject is self-contained: it owns its own `README.md`,
`AGENTS.md` (working agreements), `CHANGELOG.md`, and `spec/`
(stack-specific detail; the cross-cutting specification lives at the
repo root in [spec/](spec/)).

WPM is a **consumer application**: it does not register identities
itself. A person is a `person-service` record, their professional
identity a `worker-service` record, the employer an
`organization-service` record; training courses live in
`course-service`. WPM owns only the employment relationship and its
operational state, referencing identities by `EntityRef` URN, never
duplicating them. See [spec/scope.md](spec/scope.md) and
[spec/integrations.md](spec/integrations.md).

## Running

```sh
# Back-end
cd workforce-planning-management-service-with-rust
cargo run -- db migrate && cargo run -- task seed && cargo run -- start

# Front-end (in another shell)
cd workforce-planning-management-ui-with-svelte
pnpm install && pnpm dev
```

See [INSTALL.md](INSTALL.md) for prerequisites and the full build/test
commands.

## What it does

| Area | Highlights |
| --- | --- |
| Hiring and the record | requisitions, applicant pipeline, onboarding, worker records, org chart, reporting lines, groups |
| Workforce | time and leave, shifts with working-time guardrails, **the on-call rota** (swaps, reminders), emergency contacts, **backups**, **the directory** |
| Development | reviews, 360°s, assessments, learning, CPD, **skills gap analysis**, **training time recommendations**, succession |
| Leadership | strategic workforce planning, metrics and **insights**, **a CEO dashboard on one iPad screen** |
| Joiners and leavers | dated checklists; a leaver's **last-day handover** with an audit trail |
| Communication | **announcements** with audiences, links and read counts |
| Rights and wellbeing | wellbeing prompts, the anonymous pulse, adjustments, subject access / erasure / retention |
| Global | 17 content locales at `/en-001/`, `/cy-001/`, … each at its one full-code address; strings edited through Sveltia CMS |

Design stance (see [spec/index.md](spec/index.md)): what must not be stored gets
no column; what must not be disclosed gets no endpoint; an unknown is never a
zero; roll-ups name no one; every estimate says what it rests on.

## Specification-driven delivery (SDD)

Three lock-step files drive delivery: [spec/requirements.md](spec/requirements.md)
(`WPM-R*`), [spec/design.md](spec/design.md) (`WPM-D*`), and
[spec/tasks.md](spec/tasks.md) — the live delivery checklist (`WPM-T*`).
A change starts in `requirements.md`, is shaped in `design.md`, is
queued in `tasks.md`, and only then lands as code. **No code lands
without the spec describing it.**

## Documentation

- [spec/index.md](spec/index.md) — the cross-cutting specification
  (purpose, scope, domain model, the five pillars, the newer topic files,
  auth, audit, architecture, testing, regulatory posture, roadmap, glossary).
- [INSTALL.md](INSTALL.md) — build, run, test, schedule the tasks, edit
  translations. [NEWS.md](NEWS.md), [CHANGELOG.md](CHANGELOG.md),
  [COMPARISONS.md](COMPARISONS.md), [BENCHMARKS.md](BENCHMARKS.md).
- [AGENTS.md](AGENTS.md) and [AGENTS/](AGENTS/spec-driven-delivery.md) — working
  agreements for humans and AI agents (spec-driven delivery, backend, frontend,
  privacy and data rules, localization, testing, git, docs).
- [llms.txt](llms.txt) and [llms.json](llms.json) — a map of this repository
  for AI agents.
- [CONTRIBUTING.md](CONTRIBUTING.md) — ground rules and how to
  contribute.
- [AI_STATEMENT.md](AI_STATEMENT.md) — how AI tools are used to
  develop this software.

## License

See [LICENSE.md](LICENSE.md) (texts in [LICENSE/](LICENSE/index.md)). Each subproject declares its own SPDX
license expression in its manifest.
