# Workforce Planning Management — Specification

This directory is the **single source of truth** for the cross-cutting
Workforce Planning Management (WPM) specification, shared by both editions.
Each subproject's own `spec/` adds stack-specific detail and links back
here.

> ⚠️ **Demo software, not a production HR system.** This project models
> WPM practice for demonstration and integration purposes. It is not
> a payroll system of record, not employment-law advice, and holds no
> real personal data. See [regulatory.md](regulatory.md).

## What this project is

An **all-in-one HR platform** managing an organization's workforce
through the whole employee lifecycle — hiring to retirement. Where
traditional HR software handles administrative record-keeping, WPM
adds **strategic workforce optimization and talent development**:

1. **Talent acquisition & onboarding** — requisitions, an applicant
   tracking pipeline, a candidate pool, digitized onboarding.
2. **Workforce management** — time & attendance, absence/leave,
   shift scheduling, advisory working-time guardrails.
3. **HR service delivery** — the employee record as the single source
   of employment truth, org charts, self-service, benefits, wellbeing
   & benefits-awareness prompts, the anonymous pulse, ergonomic (DSE)
   assessments, reasonable adjustments, in-app notifications, and
   subject rights (access / erasure / retention).
4. **Talent management & development** — performance reviews, 360°
   multi-rater appraisals, training via the family's course registry,
   assessments (aptitude / personality / psychometric / selection /
   cognitive), skills & learning paths, mentorships, development
   plans, talent pipelines, early careers, succession, workforce
   intelligence.
5. **Payroll & compensation** — payroll runs, payslips, salary
   benchmarking.

Cross-cutting capabilities added since the five pillars were first
delivered: **people & cover** (employee directory, emergency contacts,
backups, the on-call rota), **skills gaps and training time**,
**joiners and leavers** with a last-day handover, **announcements**, a
**CEO dashboard** and workforce insights, strategic workforce planning
(WPM-R34–R38), job-capability frameworks (UK GDAD PCF, ESCO), reporting
lines and groups, organization memberships and transfers, and
**16 content locales** served under `/<locale>/` routes.

It is a **consumer application** (the case-folder / patient-flow /
project-portfolio-management shape): it does not register identities
itself. A human is a [person-service](../../person/person-service-with-loco/)
record; their professional identity is a
[worker-service](../../worker/worker-service-with-loco/) record; the
employer is an [organization-service](../../organization/organization-service-with-loco/)
record; training courses live in the
[course-service](../../course/course-service-with-loco/). WPM owns only
the **employment relationship and its operational state**: employee
records, requisitions, applications, time, leave, shifts, benefits,
reviews, enrollments, succession, payroll — always referencing
identities by `EntityRef` URN, never duplicating them.

## Two editions

| Subproject                                                                                                     | Role                           | Stack                                   |
| -------------------------------------------------------------------------------------------------------------- | ------------------------------ | --------------------------------------- |
| [workforce-planning-management-api-with-rust](../workforce-planning-management-api-with-rust/)         | Back-end JSON API              | Rust, Loco (Axum + SeaORM), PostgreSQL  |
| [workforce-planning-management-ui-with-svelte](../workforce-planning-management-ui-with-svelte/) | HR / manager / self-service UI | SvelteKit 2, Svelte 5 runes, TypeScript |

## Specification (topic files)

| File                                               | Covers                                                                                    |
| -------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| [purpose.md](purpose.md)                           | Problem statement, goals, the five pillars                                                |
| [scope.md](scope.md)                               | In/out of scope; the boundary with the identity services                                  |
| [domain-model.md](domain-model.md)                 | Employee, Requisition, Application, TimeEntry, LeaveRequest, Shift, Review, Appraisal, PayrollRun, … |
| [talent-acquisition.md](talent-acquisition.md)     | Pillar 1: ATS pipeline, candidate pool, onboarding checklists                             |
| [workforce-management.md](workforce-management.md) | Pillar 2: time & attendance, absence, scheduling, working-time guardrails                 |
| [hr-core.md](hr-core.md)                           | Pillar 3: the employee record, org chart, self-service, benefits, wellbeing, adjustments  |
| [talent-development.md](talent-development.md)     | Pillar 4: reviews, 360°s, LMS via course-service, assessments, succession                 |
| [strategic-workforce-planning.md](strategic-workforce-planning.md) | Forecast talent and skill needs, gap analysis vs future goals, strategic alignment (WPM-R34–R38) |
| [esco/index.md](esco/index.md) | ESCO: the EU occupations, skills, and qualifications classification — what it is, licence, how WPM relates |
| [uk-gdad-pcf/index.md](uk-gdad-pcf/index.md) | UK Government Digital and Data Profession Capability Framework — shape, licence, mapping to WPM role profiles and CPD |
| [payroll-compensation.md](payroll-compensation.md) | Pillar 5: payroll runs, payslips, benchmarking                                            |
| [people-directory-and-cover.md](people-directory-and-cover.md) | Employee directory, emergency contacts, backups (cover), the on-call rota with swaps and reminders (WPM-R39–R42) |
| [pay-scales.md](pay-scales.md) | NHS Agenda for Change pay scale for Wales (2026/27), salary placement and progression lookup (WPM-R51) |
| [job-levels.md](job-levels.md) | Google technical job levels L3–L11 as reference data; a worker's level and a role's level and pay band (WPM-R52–R53) |
| [skills-and-training.md](skills-and-training.md)   | Skills gap analysis and training time recommendations (WPM-R43–R44)                       |
| [joiners-and-leavers.md](joiners-and-leavers.md)   | Joiner / leaver records with dated checklists; a leaver's last-day handover and audit trail (WPM-R45–R46) |
| [communication-and-leadership.md](communication-and-leadership.md) | Announcement feed, workforce insights, the CEO dashboard (WPM-R47–R49)         |
| [locales-for-global-sharing-with-svelte/index.md](locales-for-global-sharing-with-svelte/index.md) | Locales end to end; **how WPM applies it** — content locales, `/en-001/` routes, `/en/` aliases, Sveltia CMS (WPM-R50) |
| [integrations.md](integrations.md)                 | Upstream family services; EntityRef URNs; `employed_by` links                             |
| [auth.md](auth.md)                                 | SSO, ABAC personas (employee / manager / HR / payroll), masking                           |
| [audit.md](audit.md)                               | Audit trail, events, sensitive-read logging                                               |
| [architecture.md](architecture.md)                 | Editions, layering, pure-core rules, persistence                                          |
| [testing.md](testing.md)                           | Test strategy per edition                                                                 |
| [regulatory.md](regulatory.md)                     | Demo status; UK GDPR / employment-records posture; subject rights (WPM-R30)               |
| [roadmap.md](roadmap.md)                           | Beyond the v1 queue                                                                       |
| [glossary.md](glossary.md)                         | ATS, FTE, LMS, requisition, accrual, …                                                    |

## Specification-driven delivery (SDD)

Three lock-step files drive delivery:

- [requirements.md](requirements.md) — numbered requirements (`WPM-R*`)
  with user stories and acceptance criteria.
- [design.md](design.md) — numbered design decisions (`WPM-D*`).
- [tasks.md](tasks.md) — **the live delivery checklist** (`WPM-T*`),
  phased; every task traces to design and requirement ids.

A change starts in `requirements.md`, is shaped in `design.md`, is
queued in `tasks.md`, and only then lands as code in a subproject.
**No code lands without the spec describing it.**

**Status (2026-10-06):** tasks WPM-T1–T89 delivered (one deliberate
deferral: employee expense claims — see the last entry of
[tasks.md](tasks.md)); requirements WPM-R1–R50, design decisions
WPM-D1–D40. Current verification: 307 unit tests, 58 database-backed request
tests, the auth enforcement and Keycloak suites, 74 front-end unit tests and
33 Playwright specs — see [testing.md](testing.md).

The load-bearing design thread (decisions WPM-D17–D25, extended by
WPM-D29–D36): **what must
not be stored gets no column** (no health cohort, no symptom, no
diagnosis, no pulse author), **what must not be disclosed gets no
endpoint** (no rater-level 360 content, no per-adjustment reporting),
and **every limit is stated in the payload rather than hidden**
(derivation strings, named exclusions, `null`-not-zero rates,
k-floors that withhold their counts). Since WPM-T69 the same thread
runs through the newer surfaces: an undeclared skill is *unknown*, not a
shortfall (WPM-D32); a cover or on-call day with nobody available says so
(WPM-D31); a training recommendation states what it rests on (WPM-D33);
announcement read receipts are counts, never names (WPM-D35).

## References

- Sibling consumer apps (the shape this follows):
  [patient-flow](../../patient-flow/spec/index.md),
  [case-folder](../../case-folder/spec/index.md),
  [project-portfolio-management](../../project-portfolio-management/spec/index.md)
- Family contracts: [cross-service-linking](../../agents/share/cross-service-linking.md)
  (the `employed_by` worker→organization edge is a registry v1 kind),
  [authentication-sessions](../../agents/share/authentication-sessions.md),
  [authorization-attributes](../../agents/share/authorization-attributes.md),
  [security](../../agents/share/security.md)
