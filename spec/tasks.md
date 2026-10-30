# Tasks — delivery checklist

Status legend: `[x]` done · `[~]` in progress · `[ ]` not started.
Every task traces to design (WPM-D*) and requirement (WPM-R*) ids.
Three-part rule applies: a behavioural change lands as spec edit +
code + tests in one PR.

**Status (2026-10-07):** WPM-T0–T97 are done, with **no open deferral** (the one
deliberate deferral, employee expense claims, was delivered as WPM-T97). Verification
now: 317 service unit tests, 62 database-backed request tests (all passing against
PostgreSQL 18), the auth enforcement matrix, the expense-claim enforcement test and the
Keycloak suite (passing, the latter against a real Keycloak 26), 74 front-end unit tests
and 37 Playwright specs; svelte-check 0 errors; `cargo fmt`, `cargo clippy -D warnings`
and `prettier` clean across the repo. Older entries' remarks such as "Rust unbuilt" or
"DB-gated … not run" **predate** the verification runs recorded in the Phase 10 note and
WPM-T73 and no longer apply. Requirements WPM-R39–R55 and design decisions WPM-D29–D42
live in the topic files listed in [index.md](index.md).

**Since then (2026-10-09):** WPM-T102 (generic pay scale), WPM-T119 (durations say
calendar days), WPM-T123, T125–T128 and T145 (production readiness) are done;
WPM-T124 (CI) and WPM-T144 (release) are written but have **never run on GitHub**
(see [production-readiness.md](production-readiness.md)). Counts are generated in
[implementation-status.md](implementation-status.md).

**Proposed (2026-10-09), not started:** Phase 12, delivery capacity and oversight
readiness (WPM-T103–T118); the backup plan, restore drill, data protection impact
assessment and gates in Phase 13 (WPM-T129–T132); and Phase 14, programme-linked
planning and contingent workforce (WPM-T133–T143). See [plan.md](plan.md).
Scheduling (critical chain, critical path and similar) lives in
project-portfolio-management (WPM-D55).

**Proposed (2026-10-21), not started:** Phase 22, code quality (WPM-T246–T267, four defects
first), and Phase 23, regulatory reporting, integration and planning depth (WPM-T268–T285,
WPM-R144–R158, WPM-D99–D106). See [plan.md](plan.md) sections N and O.

## Phase 0 — specification

- [x] WPM-T0 Cross-cutting spec round: topic files + SDD trio, both
      edition doc scaffolds, root AGENTS.md wiring. (all WPM-D*, WPM-R*)
      — landed 2026-07-18. No code.

## Phase 1 — service skeleton & employee core (WPM-R7, WPM-R17)

- [x] WPM-T1 Scaffold `workforce-planning-management-service-with-rust`:
      loco app, config, migration crate, family fixtures (forbid-unsafe,
      tracing/OTLP, `/metrics.prom`, OpenAPI + Swagger, `Accepts-version`
      middleware, health routes). (WPM-D12)
- [x] WPM-T2 Employee migrations + models + CRUD: URN validation,
      unique employee number per organization, status state machine in
      the pure core, org-chart derivation + cycle refusal, salary in
      minor units, audit + event seam (`WPM_EVENT_TRANSPORT=memory`).
      (WPM-D1–D4, WPM-D9; WPM-R7, WPM-R16)
- [x] WPM-T3 Upstream client seam: one generic `EntityRef`-keyed
      resolver over person / worker / organization / course, `http` +
      `stub` via a single `Mode` enum, config-selected; display-name
      + birth-date caches; stub-mode boot test. (WPM-D11)
- [x] WPM-T4 Seed task: synthetic org (~40 employees across
      departments, managers, salaries) — synthetic data only. (WPM-R17)

## Phase 2 — talent acquisition & onboarding (WPM-R1–R3)

- [x] WPM-T5 Requisition + Candidate + Application + Interview
      migrations/models/controllers; pipeline state machines in the pure
      core; consent-expiry exclusion + purge list; hire-creates-employee
      in one transaction. (WPM-D3, WPM-D8, WPM-D9; WPM-R1, WPM-R2)
- [x] WPM-T6 Onboarding checklists: templates → OnboardingItem
      instantiation, mandatory-complete-or-waived activation gate with
      recorded reasons. (WPM-D3; WPM-R3)

## Phase 3 — workforce management (WPM-R4–R6)

- [x] WPM-T7 TimeEntry + approval flow; >24h refusal; FTE-scaled
      overtime derivation in the pure core. (WPM-D3; WPM-R4)
- [x] WPM-T8 LeaveEntitlement + LeaveRequest: balance arithmetic
      (annual refusal, sick negative-flag), approval decrements balance
      in-tx, `FOR UPDATE` race serialization. (WPM-D3, WPM-D9; WPM-R5)
- [x] WPM-T9 Shift + ShiftAssignment: double-booking + leave-conflict
      refusal in the pure core; department day-rota view. (WPM-D3;
      WPM-R6)

## Phase 4 — HR service delivery (WPM-R8, WPM-R9)

- [x] WPM-T10 BenefitPlan + BenefitEnrollment: minor-unit costs,
      eligibility window, double-enrolment refusal. (WPM-D4; WPM-R9)
- [x] WPM-T11 Self-service surface: ownership (`$sub`) policy pins —
      own record/payslips/balances/shared reviews readable, own
      leave/time writable, others' refused. (WPM-D6; WPM-R8, WPM-R15)

## Phase 5 — talent development (WPM-R10–R12)

- [x] WPM-T12 ReviewCycle / Review / Goal / FeedbackEntry: review
      state machine (draft → submitted → calibrated → shared),
      author/subject/HR visibility, content-read audit. (WPM-D3, WPM-D7;
      WPM-R10)
- [x] WPM-T13 TrainingEnrollment over course URNs + the
      expiring-certificates report. (WPM-D10; WPM-R11)
- [x] WPM-T14 SuccessionPlan + SuccessionCandidate + the gap report
      (criticality ≥ 4 without `ready_now`); read audit. (WPM-D7;
      WPM-R12)

## Phase 6 — payroll & compensation (WPM-R13, WPM-R14)

- [x] WPM-T15 PayrollRun + Payslip: run state machine, pure-core
      payslip derivation (salary × FTE pro-rating + approved overtime −
      benefit deductions, stub tax tables), `net = gross − Σ deductions`
      invariant, overflow refusal, approved-run immutability. (WPM-D3,
      WPM-D4, WPM-D5, WPM-D9; WPM-R13)
- [x] WPM-T16 Benchmark rows + comparison view with `below_min` /
      `above_max` flags, payroll/HR-persona gated. (WPM-D4, WPM-D6;
      WPM-R14)

## Phase 7 — auth activation surface (WPM-R15, WPM-R16)

- [x] WPM-T17 `auth.rs`: offline PASETO verify + blanket
      `WPM_REQUIRE_AUTH` guard (guard-all / deny-unless-public) + ABAC +
      record-level `resource.person`/`department`/`status` attrs + `mask`
      obligation on salary/payslips/reviews; sensitive-read audit wiring;
      persona test matrix in its own enforcement binary. (WPM-D6, WPM-D7,
      WPM-D12)

> Phases 1–7 landed 2026-07-18 in one implementation round
> (`workforce-planning-management-service-with-rust`, copy-adapted from
> patient-flow): 7 migrations (23 domain tables + audit + outbox),
> pure `rules/` core (lifecycle tables, leave/time arithmetic,
> org-cycle, payslip arithmetic incl. the net invariant + overflow
> refusal, benchmark flags), 5 pillar controllers + audits/docs/
> metrics, `auth.rs` with `resource.person` `$sub` ownership +
> salary/payslip masking. 71 DB-free unit tests, 7 request tests
> (hire journey, 404/cycle/uniqueness pins, time caps + overtime,
> leave balance journey incl. decided-race pin + cancel-restores,
> shift conflicts, payroll derivation incl. approved-run
> immutability, benchmark flags), 1 enforcement persona-matrix
> binary — all green against Postgres 18; clippy-pedantic clean;
> live smoke verified (migrate → seed 40 employees → org chart →
> version negotiation → 406 → OpenAPI 57 paths → Prometheus).
> Notes: the seed grants entitlements for reports only; `/api/shifts`
> rota attaches assignments per shift; CI workflows deferred
> (nested workflows don't run in the monorepo).

## Phase 8 — front-end (all WPM-R*)

- [x] WPM-T18 Scaffold `workforce-planning-management-ui-with-svelte`:
      SvelteKit 2 + Svelte 5 runes SPA, BFF proxy + session flow,
      13-locale i18n from the start, typed API client + `money()`.
      (WPM-D12)
- [x] WPM-T19 Views: requisition/application boards, onboarding
      tracker, team calendar + rota, employee profile + org chart,
      review + enrollment panels, payroll run screen, benchmarking
      table, HR dashboard; vitest + `page.route`-stubbed Playwright.
      (WPM-D6, WPM-D12)

> WPM-T18/T19 landed 2026-07-18: SvelteKit 2 + Svelte 5 runes SPA
> (copy-adapted from the patient-flow front-end: BFF proxy + session
> flow + `Accepts-version` stamping) with a dependency-free 48-key ×
> 13-locale i18n module (parity-tested, RTL for ar/ur), typed WPM
> client + `money()` (masked/absent renders an em dash, never 0),
> and views: HR dashboard tiles, employee list (masked salary as
> first-class state) + profile (onboarding/balances/leave/payslips/
> reviews/training), recursive org chart, requisition status board +
> application pipeline with in-row hire, workforce (pending-leave
> approvals + rota), development (gaps/succession/expiring), payroll
> runs + run detail with lifecycle actions, benchmarks + comparison
> flags. svelte-check 0 errors; 5 vitest (money honesty, i18n
> parity, API path map) + 4 Playwright (page.route-stubbed,
> unstubbed = 404-loud) green.

## Production gates (before any non-demo exposure)

- [~] WPM-G1 Activate `WPM_REQUIRE_AUTH` + mount a real ABAC policy;
      verify the persona matrix against the deployment's attributes.
      **Code side landed as WPM-T31 (2026-07-25)** — the shipped
      reference policy, the activation runbook (spec `auth.md`), and
      the enforcement matrix verifying the reference file itself.
      The remaining act — setting the flag and attributes on a real
      deployment — is operational by design.
- [~] WPM-G2 Retention schedules + subject-access/erasure flows;
      jurisdiction-correct payroll tables; equality-law review of any
      scoring ([regulatory.md](regulatory.md)). **Code side landed as
      WPM-T30 (2026-07-25)**; the remaining items — lawful-basis
      mapping, jurisdiction payroll tables, equality-law review, and
      coordination of subject rights with the upstream identity
      services — are operational/legal work, not code.

- [x] WPM-T20 (2026-07-20) **Learning & development.** Migration
      `m20260720_000008_learning` (skills catalog + declared
      `worker_skills`, `learning_paths` + steps + `path_enrollments`,
      `mentorships` + `mentorship_sessions`). `controllers/learning.rs`:
      the skills framework (catalog; declared proficiency 1–5 + optional
      target, upsert), learning paths (ordered course steps; idempotent
      enrolment; honest per-member **progress** — a step counts only
      against a _completed_ `training_enrollments` row for its
      `course_ref`), and mentorships (proposed→active→completed lifecycle
      in `rules::learning`; sessions only on an active pairing).
      Derived views: skills matrix + gaps by department,
      training-analytics (completion ratio = completed / non-failed +
      cert-expiry by department), mentorship overview (active pairs,
      mentor load, unmatched active employees, stale actives). Front-end:
      `/learning` (matrix + gaps + analytics + path progress) and
      `/mentorship`. **Acceptance:** proficiency / lifecycle / progress
      pure pins; the seeded L&D round-trip green first run — full
      `--ignored` suite 8/8 vs Postgres 18; clippy pedantic clean;
      svelte-check 0; vitest 5; Playwright 7.

- [x] WPM-T21 (2026-07-23) **Assessments — aptitude / personality /
      psychometric / selection.** Migration `m20260723_000009_assessments`
      (`assessment_instruments` + `assessments` + `assessment_results`).
      `rules/assessment.rs` (pure): the category↔scale map with the
      psychometric overlap (`category_permits`), the lifecycle machine,
      integer score bounds, the band split, currency
      (completed + unexpired), the mean-with-its-terms, and the
      not-assessed gap list. `controllers/assessments.rs`: the instrument
      catalog, sittings against a candidate or employee (application-linked
      sittings must match the candidate), per-scale result upsert with the
      band derived, the lifecycle move (completion requires ≥ 1 result and
      derives `expires_on` from the instrument validity), the derived
      per-subject profile, the hiring view
      (`/api/applications/{pid}/assessments`), and aggregate analytics.
      Sensitivity: `mask` obligation on every read path, unmasked scored
      reads audited, no individual score in the analytics.
      **Acceptance:** 12 DB-free pure/controller pins (mapping,
      psychometric overlap, lifecycle, bands, bounds, currency, masking,
      declared-scale gating, expiry arithmetic that cannot panic) +
      the DB-gated `assessment_round_trip` request suite; `cargo test`
      green (104 unit); clippy pedantic clean.

- [x] WPM-T22 (2026-07-23) **Talent strategy — succession, upskilling,
      reskilling, pipelines, apprenticeships, internships, workforce
      intelligence.** Migration `m20260723_000010_talent`
      (`development_plans` + items, `talent_pipelines` + `pipeline_members`,
      `early_career_programs` + `program_placements`, and
      `succession_plans.risk_of_loss` / `.vacancy_expected_on`).
      `rules/talent.rs` (pure): upskill/reskill target coherence, the plan
      and placement and pipeline machines (including the deliberate
      `ready → developing` regression), the 1–5 step rule, declared vs
      **verified** progress, off-the-job-hours completion gate,
      conversion rate over completed placements only, bench coverage,
      single-point-of-failure (criticality × risk of loss), terms-carrying
      ratios, and tenure buckets. `controllers/talent.rs` (plans,
      pipelines, early careers) + `controllers/intelligence.rs` (the four
      read-only `/api/workforce-intelligence/*` views) + succession
      updates in `controllers/development.rs`
      (`PUT /api/succession-plans/{pid}`,
      `PUT /api/succession-candidates/{pid}` — readiness may go down).
      **Acceptance:** 14 pure rules pins + 3 controller projection pins +
      the DB-gated `development_plans_track_claimed_and_verified_progress`
      and `pipelines_apprenticeships_and_intelligence` request suites;
      `cargo test` green (104 unit, 11 DB-gated); clippy pedantic clean;
      OpenAPI covers every new path (`spec_shape` extended).

- [x] WPM-T23 (2026-07-24) **Wellbeing — health entitlement prompts.**
      Migration `m20260724_000011_wellbeing` (`wellbeing_entitlements`
      rule rows — no column can express a health-status cohort, per
      WPM-D17 — + `entitlement_acknowledgements`, unique per
      employee + entitlement). `rules/wellbeing.rs` (pure): panic-free
      age arithmetic (leap-day pins), eligibility over age band /
      department / job title / active window with **unknown age failing
      a banded rule**, and the prompt machine (one reminder max for
      multi-dose `booked`/`done`; declining is final).
      `controllers/wellbeing.rs`: rule CRUD, the `$sub`-owned
      self-service prompt view (serving the reminder stamps it), the
      audited acknowledgement upsert
      (`booked | done | declined | dismissed`), and the HR
      `/api/wellbeing/uptake` view — aggregate counts only with WPM-D16
      terms; no manager surface. `clients.rs` gains a best-effort
      cached `birth_date` person lookup (never stored). Front-end:
      profile **Health entitlements** card + `/wellbeing` HR admin
      (rules + create + soft-close + uptake), 11 i18n keys × 13
      locales. **Acceptance:** 12 pure pins; DB-gated
      `wellbeing_round_trip` green first run — full `--ignored` suite
      12/12 + enforcement vs Postgres 18 (116 unit); clippy pedantic
      clean; svelte-check 0; vitest 10; Playwright 8; OpenAPI covers
      the five new paths. (WPM-D6, WPM-D7, WPM-D11, WPM-D16, WPM-D17;
      WPM-R25)

- [x] WPM-T24 (2026-07-25) **Benefits-awareness engine.** Migration
      `m20260725_000012_benefits_awareness` generalises
      `wellbeing_entitlements` with a closed `kind`
      (`health | benefit`, defaulted `health` so WPM-T23 rows are
      untouched) and an optional `benefit_plan_pid` (validated live;
      refused on a `health` rule). The predicate and acknowledgement
      vocabularies are unchanged (WPM-D17). A plan-linked prompt
      carries the plan reference, and goes **quiet automatically** for
      an employee with a live enrolment in that plan — derived per
      request from `benefit_enrollments`, never stored (WPM-D18);
      enrolment stays the WPM-R9 endpoint. Rule list gains `?kind=`
      (validated); the uptake rows carry the kind. Front-end: kind
      select + chip on `/wellbeing`, kind chip on the profile card,
      2 i18n keys × 13 locales. **Acceptance:** kind-vocabulary pin;
      DB-gated `benefits_awareness_round_trip` (kind gate, dead-plan
      404, plan-carrying prompt, enrolment-quietens pin, `?kind=`
      filter + 422, kind in uptake with null-not-zero rate) — full
      `--ignored` suite 13/13 vs Postgres 18 (117 unit); clippy
      pedantic clean; svelte-check 0; vitest 10; Playwright 8.
      (WPM-D16, WPM-D17, WPM-D18; WPM-R26)

- [x] WPM-T25 (2026-07-25) **Enrolment conversion in the uptake view.**
      For a plan-linked rule, `GET /api/wellbeing/uptake` also reports
      `enrolment_conversion` — of the **distinct** employees who
      acknowledged the prompt, how many now hold a live enrolment in
      the linked plan — WPM-D16 terms (`null`, never `0`, with nobody
      acknowledged), derived per request from `benefit_enrollments`
      and never stored (WPM-D18); `null` for a rule with no linked
      plan; the derivation string names both formulas. Front-end: a
      conversion cell on the `/wellbeing` uptake table, 1 i18n key ×
      13 locales. **Acceptance:** DB-gated pins (1/2 conversion after
      one acknowledger enrols, health-rule `null`, empty-denominator
      `null`, still no employee pid in the payload) — full suite
      13/13 vs Postgres 18; clippy pedantic clean; svelte-check 0;
      vitest 10; Playwright 8. (WPM-D16, WPM-D18; WPM-R26)

- [x] WPM-T26 (2026-07-25) **Working-time guardrails.** No new stored
      state: `rules/working_time.rs` (pure — 17-week/48-hour average
      as an integer boundary comparison with WPM-D16 terms, 11-hour
      rest-gap detection over sorted shift intervals with overlap
      clamping and malformed-interval skipping; leap-safe, panic-free)
      + `GET /api/workforce/working-time?department=&as_of=` in the
      workforce controller: per-employee flags over **recorded** (not
      merely approved) minutes in the trailing 17 weeks and rest-gap
      breaches across recent **and planned** assignments (±28 calendar days).
      Advisory only — nothing is refused (new WPM-D19); visibility
      equals the rota's. Front-end: a Working-time panel on
      `/workforce` (flags + all-clear state), 4 i18n keys × 13
      locales. **Acceptance:** 3 pure pins (terms incl. null-not-zero,
      exact 48 h boundary, rest-gap matrix) + the DB-gated
      `working_time_guardrails` request test (over-average flag with
      its terms, 10 h turnaround = one 600-min breach, modest week
      unflagged, department scoping) — full `--ignored` suite 14/14
      vs Postgres 18 (120 unit); clippy pedantic clean; svelte-check
      0; vitest 10; Playwright 8. (WPM-D16, WPM-D19; WPM-R27)

- [x] WPM-T27 (2026-07-25) **Anonymous wellbeing pulse.** Migration
      `m20260725_000013_pulse` (`pulse_surveys` + `pulse_responses` —
      the response row has **no author column**, by design).
      `rules/pulse.rs` (pure): the 1–5 scale, the inclusive survey
      window, and the k-floored aggregation (`K_ANONYMITY = 5`, a
      constant not configuration; a suppressed cell withholds its
      count; clamped, panic-free). Endpoints: survey create/list,
      `POST /api/pulse-surveys/{pid}/responses` ($sub-owned submit;
      identity used to derive the department and enforce ownership,
      then dropped; **actor-less** audit row; no handle returned;
      window-gated `422`), and `GET …/results` (per-department +
      overall cells, suppressed or disclosed with count/distribution/
      mean; derivation states counts are responses, not respondents —
      new WPM-D20). Front-end: pulse submit card (1–5 + thanks state)
      on the profile, k-floored results blocks on `/wellbeing`;
      5 i18n keys × 13 locales. **Acceptance:** 4 pure pins (scale,
      window, k-floor incl. count-withholding, bad-row clamping) +
      the DB-gated `pulse_round_trip` (closed-survey 422, bad-score
      422, 4-response suppression, 5th response discloses, small
      finance cell stays suppressed with count withheld, no employee
      pid anywhere in results, actor-less audit rows) — full
      `--ignored` suite 15/15 vs Postgres 18 (124 unit); clippy
      pedantic clean; svelte-check 0; vitest 10; Playwright 8.
      (WPM-D16, WPM-D20; WPM-R28)

- [x] WPM-T28 (2026-07-25) **360° appraisals.** Migration
      `m20260725_000014_appraisals` (`appraisals` + nominations +
      responses; the response row links to its nomination **by
      design** — procedural anonymity, new WPM-D21). `rules/
      appraisal.rs` (pure): the one-way lifecycle
      (draft → collecting → shared), the group vocabulary
      (`self | manager | peer | report`; external deferred), rater
      bounds (≤ 12; ≥ 3 non-self to collect), score-completeness
      (every declared competency, 1–5, nothing undeclared), the
      WPM-D21 group floor (`peer`/`report` disclose at 3;
      `manager`/`self` at 1 by convention), and the count-carrying
      mean (empty ⇒ `None`, never 0).
      `controllers/appraisals.rs`: create (auto self nomination, in
      one tx), nominate (draft-only, frozen at collecting), status
      gates, `$sub`-owned once-per-rater responses (collecting only),
      the detail view (who responded, never what), and the shared-only
      **report** (group × competency count + mean, group-pooled
      comments sorted alphabetically so ordering reveals no
      submission sequence, withheld cells hide their count; reads
      audited per the WPM-R10 posture; development-facing, not a
      payroll input). Front-end: a 360° panel on the employee profile
      (create / nominate / lifecycle / respond / report), 8 i18n keys
      × 13 locales. **Acceptance:** 4 pure pins (lifecycle, floor
      matrix, score matrix, count-carrying mean) + the DB-gated
      `appraisal_round_trip` (auto-self, closed groups, subject-only-
      self, min-rater gate, frozen nominations, completeness 422s,
      once-per-rater, no rater content on the detail view,
      shared-only report, manager-discloses-at-1, peer-withheld-at-2
      with count hidden, responses closed once shared, audited report
      read) — full `--ignored` suite 16/16 vs Postgres 18 (128 unit);
      clippy pedantic clean; svelte-check 0; vitest 10; Playwright 8.
      (WPM-D3, WPM-D7, WPM-D21; WPM-R29)

- [x] WPM-T29 (2026-07-25) **Rater self-service for 360s.**
      `GET /api/employees/{pid}/appraisal-requests` — the rater's own
      pending requests: `collecting` appraisals where they are
      nominated and unanswered, with subject / group / competencies
      (`$sub`-owned; discloses only that they were invited). Front-end:
      a "My 360 requests" panel on the profile with inline scoring +
      comment (responding clears the request). **Acceptance:** the
      round-trip pins one pending request naming subject/group/
      competencies, an empty list for the non-nominated, and
      responded ⇒ no longer pending — suite 16/16 vs Postgres 18
      (128 unit); clippy pedantic clean; svelte-check 0; vitest 10;
      Playwright 8. (WPM-D21; WPM-R29)

- [x] WPM-T30 (2026-07-25) **Subject rights & retention (the code
      side of WPM-G2).** `rules/privacy.rs` (pure): `erasable`
      (terminated/retired only — the open relationship is the lawful
      basis), the floored retention horizon (`WPM_RETENTION_DAYS`,
      default 365, **floor 30** — a zero horizon would turn soft-
      delete into hard-delete), and the 38-table sweep list (sorted,
      deduped, pinned). `controllers/privacy.rs`:
      `GET /api/employees/{pid}/subject-access` (one audited JSON
      document across every table keyed to the employee, including
      their authored 360 responses; **exclusions named in the
      payload** — pulse responses are structurally impossible, other
      raters' 360 content is third-party, upstream identity records
      are the deployment's coordination duty);
      `POST …/erase` (anonymise per WPM-D22: identity fields scrubbed
      + tombstone `person:` URN, salary nulled, row soft-deleted,
      authored notes/comments/session-notes scrubbed, appraisals-as-
      subject closed, acknowledgements deleted — payroll rows remain;
      refused `422` on an open employment; audited with counts);
      `GET /api/retention` + `POST /api/retention/sweep` (hard-delete
      past-horizon soft-deletes, scrub expired-consent candidates;
      audited with counts). `/erase` and `/sweep` join
      `DESTRUCTIVE_POST_SUFFIXES` (⇒ `access=admin` under
      enforcement). Front-end: a "Download my data" link on the
      profile (1 i18n key × 13 locales). **Acceptance:** 3 pure pins
      (erasable matrix, horizon default/floor, sweep-list soundness) +
      the DB-gated `subject_rights_round_trip` (export gathers the
      footprint + names exclusions + audited; erase refused while
      active, then anonymises via offboarding→terminated with counts
      in the audit snapshot; report floored; empty sweep audited) —
      full `--ignored` suite 17/17 vs Postgres 18 (131 unit); clippy
      pedantic clean; svelte-check 0; vitest 10; Playwright 8.
      (WPM-D7, WPM-D22; WPM-R30, WPM-G2)

- [x] WPM-T31 (2026-07-25) **Auth activation surface (the code side
      of WPM-G1).** Ships
      `config/abac-policy.reference.json` — the spec `auth.md`
      personas as policy: svc/admin everything; `payroll=true`
      unmasked read; `hr=true` write + **masked** read (salary stays
      payroll + self); `resource.person = $sub` self-read unmasked;
      masked-read fallback — plus the **activation runbook** in
      `auth.md` (mount → keys → flag → verify; known engine limits
      stated: self-service writes need a coarse write allow;
      manager scoping is per-department). Two masking gaps found and
      fixed during verification: `subject-access` now **refuses** a
      masked caller (`403` — a full export cannot be "masked"), and
      the 360 report withholds comments (review-content tier) from
      masked callers while keeping the numeric aggregates. The
      enforcement binary now mounts **the shipped reference file
      itself** (`WPM_ABAC_POLICY_FILE`) and extends the matrix:
      payroll-unmasked vs hr-masked reads, subject-access self-200 /
      masked-403, `/erase`+`/sweep` destructive (hr 403; svc sweep
      200; svc erase of an **active** employment still 422 — the
      lawful basis holds regardless of privilege), and the
      masked-report comment withholding. **Acceptance:** enforcement
      matrix green first run; full `--ignored` suite 17/17 + 131 unit
      vs Postgres 18; clippy pedantic clean. (WPM-D6, WPM-D7,
      WPM-D21, WPM-D22; WPM-R15, WPM-G1)

- [x] WPM-T32 (2026-07-25) **Privacy front-end.** The Svelte app
      gains the ops surface for WPM-R30: a `/privacy` page (retention
      report — horizon, per-table past-horizon counts, expired-consent
      candidates, derivation — with the sweep button and its result
      counts) and an **Erase (anonymise)** action on the employee
      profile, shown only for `terminated`/`retired` employees,
      confirm-gated, navigating away after the record 404s. Three
      client functions with a path pin; nav link; 8 i18n keys × 13
      locales. **Acceptance:** svelte-check 0; vitest 10 (path map
      extended); Playwright 9 (new stubbed `/privacy` spec: report
      renders, sweep posts and shows counts). (WPM-R30)

- [x] WPM-T33 (2026-07-25) **360° notifications.** Migration
      `m20260725_000015_notifications` (`notifications`: employee,
      kind, neutral body, reference data, `read_at`). `rules/notify.rs`
      (pure): the closed kinds and the fan-out — `collecting` ⇒ every
      rater (self included; the self-assessment is a task),
      `shared` ⇒ the subject. The appraisal status handler pushes the
      rows (reference-only bodies — never scores or comments; the
      WPM-D21 guarantee survives the bell, new WPM-D23);
      `GET /api/employees/{pid}/notifications` (unread first,
      `$sub`-owned) + `POST /api/notifications/{pid}/read`
      (owner-only). Erasure deletes the employee's notifications
      (`notifications_deleted` in the audit snapshot); the
      subject-access export includes them. Outbound channels are
      stated out of scope — WPM holds no contact details. Front-end:
      a Notifications panel on the profile with mark-read; 2 i18n
      keys × 13 locales. **Acceptance:** fan-out pure pins; the 360
      round-trip pins rater + self request bells, the subject's
      shared bell, no rater content in any bell, and mark-read —
      suite 17/17 vs Postgres 18 (133 unit); clippy pedantic clean;
      svelte-check 0; vitest 10; Playwright 9. (WPM-D21, WPM-D23;
      WPM-R31)

- [x] WPM-T34 (2026-07-25) **Ergonomic (DSE) workstation
      assessments.** Migration `m20260725_000016_ergonomics`
      (`ergonomic_assessments` + `ergonomic_items`).
      `rules/ergonomics.rs` (pure): the default 8-item DSE checklist
      (a test pins that **no item names a symptom** — WPM-D24: the
      workstation, never the body), the every-item-answered completion
      gate (WPM-D15 posture), and the open-issue count.
      `controllers/ergonomics.rs`: create (default or custom
      checklist), answer (`ok`/`issue` + equipment note; open
      assessments only — completed ones freeze), complete (stamps the
      date; audited with the issue count), and
      `GET /api/ergonomics/issues` (rota-tier department report).
      Erasure scrubs item notes + soft-deletes the employee's
      assessments; subject access includes them; both tables join the
      sweep list (now 40, pinned). Front-end: an Ergonomics panel on
      the profile (create / answer / complete) and an issues table on
      `/workforce`; 7 i18n keys × 13 locales. **Acceptance:** 3 pure
      pins + the DB-gated `ergonomics_round_trip` (default checklist,
      completion gate, freeze-after-complete, department issue report
      with note, custom checklist) — suite 18/18 vs Postgres 18
      (136 unit); clippy pedantic clean; svelte-check 0; vitest 10;
      Playwright 9. (WPM-D15, WPM-D24; WPM-R32)

- [x] WPM-T35 (2026-07-25) **Cognitive (IQ-style) testing.** The
      assessment vocabulary (WPM-R20) gains a fifth category,
      `cognitive`, with standard index scales (verbal comprehension,
      working memory, processing speed, spatial reasoning, fluid
      reasoning) — per-scale readings through the existing engine
      (masked reads, audited unmasked reads, distribution-only
      analytics all apply unchanged), **no composite score exists**,
      and `category_permits` refuses cognitive scales on `selection`
      instruments — an IQ scale cannot ride into hiring unreviewed;
      `psychometric` batteries span it. **Acceptance:** the
      `cognitive_category_scales_and_overlap` pin (own scales permit,
      psychometric spans, selection refuses, aptitude refuses) + the
      existing exhaustiveness pins re-derive over the widened
      vocabulary — 137 unit green; clippy pedantic clean. (WPM-D13;
      WPM-R20)

- [x] WPM-T36 (2026-07-25) **Reasonable adjustments
      (neurodiversity-inclusive).** Migration
      `m20260725_000017_adjustments` (`adjustment_requests`: category,
      barrier, impact, adjustment, decision note — **no diagnosis,
      condition, or medical-evidence column exists**, new WPM-D25).
      `rules/adjustments.rs` (pure): the practical suggestion
      categories (a test pins no entry names a condition) and the
      lifecycle (`requested → agreed|declined|withdrawn`;
      `agreed → in_place|withdrawn`; declined is terminal — ask anew,
      each ask stays on the record). `controllers/adjustments.rs`:
      create (`$sub`-owned; all three texts required; the audit row
      records the category, never the words), list (content-tier:
      masked reads keep category + status and withhold the words;
      unmasked reads audited), decide (practical note, audited,
      in-app `adjustment_update` notification carrying category +
      state only). Erasure scrubs the words + soft-deletes; subject
      access includes requests verbatim ("save a copy"); the table
      joins the sweep (41, pinned). **No aggregate reporting surface
      exists** — stated in WPM-R33. Front-end: a Reasonable-
      adjustments panel on the profile with the barrier/impact/change
      prompts and decision actions; 10 i18n keys × 13 locales.
      **Acceptance:** 2 pure pins + the DB-gated
      `adjustments_round_trip` (barrier required, closed practical
      categories, audited unmasked read, agree-with-note → in-place,
      declined-terminal via the machine pin, two notification bells
      with no words, export carries the words verbatim) — suite 19/19
      vs Postgres 18 (139 unit; the assessment category pin updated
      to 5); clippy pedantic clean; svelte-check 0; vitest 10;
      Playwright 9. (WPM-D23, WPM-D25; WPM-R33)

- [x] WPM-T37 **`require_ref` EntityType coverage.** *(resolved
      2026-09-05.)* `validation.rs`'s
      shared `require_ref`/`ref_opt` helper is exercised by
      `controllers/hr_core.rs`, `acquisition.rs`, `development.rs`,
      `payroll.rs`, and `talent.rs` against `EntityType::Worker`,
      `::Organization`, `::Course`, and `::CourseInstance`, but its
      own unit test (`ref_rules`) only ever passes `EntityType::Person`
      (verified: `grep -n "EntityType::" src/validation.rs` shows every
      call in the test using `Person`, while `grep -rn
      "entity_ref::EntityType::" src/controllers/*.rs` shows `Worker`/
      `Organization`/`Course`/`CourseInstance` all in live use). The
      wrong-type/malformed/bad-uuid branches are therefore proven for
      exactly one of the five accepted types. **Acceptance:**
      `ref_rules` (or a new parametrised test) exercises the
      wrong-type-detected branch for at least `Worker`, `Organization`,
      and `Course` in addition to `Person`; `cargo test` green; clippy
      pedantic clean. (WPM-D9)
      - **Resolved.** New
        `ref_rules_wrong_type_worker_organization_and_course` test in
        `src/validation.rs` exercises both the wrong-type rejection (a
        `person:` ref where `Worker`/`Organization`/`Course` is
        expected) and the matching-type acceptance for each, alongside
        the pre-existing `Person`-only `ref_rules` test.
        `CourseInstance` is not additionally covered — the acceptance
        criterion's "at least" three types is met, and the fourth
        would be a mechanical repeat of the same pattern.

- [x] WPM-T38 **Front-end sign-in gate.** No `+layout.server.ts`
      exists anywhere under `src/routes`, and `hooks.server.ts` only
      reads the session cookie into `locals.sessionId` without ever
      redirecting; `api/proxy/[...path]/+server.ts` forwards
      unauthenticated when `WPM_REQUIRE_AUTH` is off and otherwise
      just relays whatever the upstream 401 is (verified: `find
      src/routes -iname "+layout.server.ts"` returns nothing; no
      `redirect(` call exists in any `.server.ts` under `src/routes`).
      A visitor with no session therefore reaches every profile/
      payroll/wellbeing page and only discovers they are signed out
      when an API call silently fails, rather than being redirected to
      sign-in — the page-visit auth guard the repo root `tasks.md`
      WEB-1 finding found on only 5 of 16 family front-ends.
      **Acceptance:** a root `+layout.server.ts` (or per-protected-
      route guard) redirects a visitor with no `locals.sessionId` to
      `/signin` (excluding the public sign-in/verify routes); a
      Playwright spec pins the redirect; svelte-check 0; existing
      vitest/Playwright suites stay green. (WPM-D12)
      **Resolution (2026-09-05):** chose the root `+layout.server.ts`
      path rather than person-front-end's narrower per-mutation-page
      `requireSignedIn` guard — WPM's pages mix read content with
      embedded actions (the retention sweep button on `/privacy`,
      payroll lifecycle actions on `/payroll/[id]`, …) rather than
      separating reads and writes onto dedicated routes, so there is
      no small "mutation-only" subset to gate instead; a root gate
      naturally covers WPM's actual page shape. New
      `src/routes/+layout.server.ts` redirects to `/signin` (303) for
      any path other than `/signin`/`/verify` when `locals.sessionId`
      is `null`. **A real technical wrinkle, found and resolved rather
      than assumed:** this app runs `ssr = false` (root `+layout.ts`,
      SPA mode), so the redirect is invisible in the raw HTTP response
      to the *document* request (confirmed via `curl` — the initial
      `GET /employees` still returns `200` with an empty shell) and
      only surfaces through SvelteKit's client-side data-fetch
      (confirmed via `curl http://…/employees/__data.json` returning
      `{"type":"redirect","location":"/signin"}`), which a real browser
      picks up during hydration and turns into a client-side
      navigation — a plain `curl` check on the page URL alone would
      have looked like the gate does nothing. `tests/e2e/smoke.spec.ts`
      gained a `signIn()` helper that injects a fake
      `__Host-mxi_session` cookie via Playwright's `context.addCookies`
      (the server only checks the cookie's *presence*, never its
      validity, so a fabricated value passes the gate without a real
      authentication-service round trip) and a `"sign-in gate
      (WPM-T38)"` describe block proving the redirect for a
      signed-out visitor on both a named route and the dashboard, plus
      that `/signin` itself stays reachable with no session. Every
      **pre-existing** smoke test — all of which assume a signed-in
      visitor implicitly — moved under a new `"signed-in smoke
      coverage"` describe whose `beforeEach` calls `signIn()` before
      the existing route stubs, so none of them silently redirect to
      `/signin` instead of rendering the page under test; this was
      verified empirically (all 12 Playwright tests green), not
      assumed from reading the diff. Verified: `npm run check`
      (svelte-check: 409 files, 0 errors, 0 warnings), `npx playwright
      test` (12 passed), `npx vitest run` (10 passed, unchanged).

- [x] WPM-T39 **Front-end honesty-format module.** The null-ratio /
      no-data rendering rules (training completion %, conversion
      rate, benchmark below/above-min flags, pulse means, appraisal
      count-carrying means) are inlined per-route rather than
      centralised: `src/lib/` has no `format.ts` counterpart to the
      CMS front-end's `$lib/format` (verified: `find src/lib -maxdepth
      2 -type f` lists only `api/`, `components/`, `config.ts`,
      `i18n.svelte.ts`, and `server/`; the em-dash/no-data pattern
      recurs inline in `routes/learning`, `routes/mentorship`, and
      `routes/employees/[pid]`), and `tests/unit/` carries only
      `wpm.test.ts` (money/i18n/paths) and `rename-migration.test.ts`
      — none of the ratio/no-data rendering paths are unit-tested.
      **Acceptance:** a `$lib/format.ts` (or equivalent) centralises
      the null-not-zero ratio/percentage rendering with its own vitest
      suite (zero-denominator, present-value, negative-flag cases);
      the routes that currently inline the logic call it instead;
      svelte-check 0; Playwright green. (WPM-D16)
  - **Resolved (2026-09-06).** New `src/lib/format.ts` (mirroring the
    CMS front-end's), with a new shared `Ratio` type
    (`src/lib/api/types.ts`, replacing three duplicated anonymous
    `{numerator, denominator, value}` inline type literals in
    `src/lib/api/wpm.ts`): `percent`/`workings`/`percentWithWorkings`
    for a service-supplied `Ratio`, `percentOf` for a raw `done`/`total`
    pair the service doesn't wrap in a `Ratio` (learning-path step
    progress), and `mean` for a one-decimal sample mean. Wired into
    `routes/learning` (training-completion rate + path-progress
    percentage, replacing a locally-scoped `pct()` helper),
    `routes/wellbeing` (uptake rate, enrolment conversion, and the
    pulse-cell mean), and `routes/employees/[pid]` (appraisal
    competency means). **`routes/mentorship` needed no change** — on
    inspection it has no ratio/mean rendering, only `??` presence
    fallbacks; the task's own verification note listing it was
    imprecise about which three routes actually carry the pattern.
    **Benchmark below/above-min flags** (`routes/benchmarks`) were
    deliberately **not** extracted: that's a single, non-duplicated
    presence check (`{#if row.flag}…{:else}—{/if}`) with no rounding
    or ratio math to centralise — nothing to unify. **One behaviour
    fix in passing, not just a refactor**: the three duplicated
    ratio-rendering call sites had drifted into two *inconsistent*
    null conventions — `routes/learning`'s `completion_rate` always
    showed the raw `(numerator/denominator)` working even when the
    percentage was dashed (e.g. `"— (0/5)"`), while
    `routes/wellbeing`'s `uptake_rate`/`enrolment_conversion` collapsed
    to a bare `"—"` with no working at all. `percentWithWorkings`
    picks the latter, more common convention as canonical (2 of 3
    sites already did it that way, and a working next to a dashed
    percentage was arguably confusing rather than informative);
    `routes/wellbeing`'s pulse-mean label also silently rendered the
    literal string `"undefined"` when a cell's `mean` was absent
    (`value.mean?.toFixed(1)` with no fallback) — `mean()` now returns
    `null` for that case and the caller falls back to `"—"`. New
    `tests/unit/format.test.ts` (17 cases: present/zero/null value for
    each of `percent`/`workings`/`percentWithWorkings`/`percentOf`, and
    present/zero/absent for `mean`, each pinning the null-not-zero
    direction both ways — a real `0` must render as `0%`/`0.0`, never
    collapse into "no data"). `npx vitest run` 30/30 (was 13);
    `svelte-check` 0 errors/warnings; `npx playwright test` 12/12
    unchanged (the existing fixtures exercise only non-null ratios, so
    the null-convention unification above doesn't touch any pinned
    text); `npm run build` clean.

- [x] WPM-T40 **List pagination headers.** No controller emits
      `X-Total-Count`/`X-Limit`/`X-Offset`, and list handlers cap with
      a hardcoded `.limit(...)` (e.g. `hr_core.rs`'s `.limit(500)` and
      `.limit(200)`) rather than accepting `?limit=&offset=` (verified:
      `grep -rln "X-Total-Count" src/` is empty; `grep -n "\.limit("
      src/controllers/hr_core.rs` shows the two hardcoded caps) — the
      family-wide contract in `agents/share/restful.md` (headers
      report the true total, `limit` clamps rather than rejects,
      `offset` is bounded per SEC-G7). **Acceptance:** at least the
      employee list and one other high-traffic list endpoint accept
      `?limit=&offset=`, clamp `limit` to a documented `MAX_LIMIT`,
      bound `offset` (400 past it), and set the three response
      headers; a request test pins clamping and the offset bound;
      clippy pedantic clean. (WPM-D9, WPM-R7)
      **Resolution (2026-09-05):** ported the family's `Page`/
      `with_page_headers` pattern (already used by care-pathway-service,
      case-service, organization-service, …) into
      `src/controllers/mod.rs` — `MAX_LIMIT` (500), `MAX_OFFSET`
      (10 000), `Page { limit, offset }` (`resolve`/`check_offset`), and
      `with_page_headers`. `limit`/`offset` are declared as plain fields
      on each query struct rather than `#[serde(flatten)]`-ed onto
      `Page` — a flattened struct deserializes `axum::extract::Query`
      values from a string-keyed map, so `limit=2` would arrive as the
      string `"2"` and fail to parse as a `u64`, a spurious `400` on a
      valid request; the `Page` value is constructed explicitly inside
      each handler instead. `GET /api/employees` (default limit 500,
      unchanged) and `GET /api/benefit-plans` (default limit 200,
      unchanged) both now accept `?limit=&offset=`, clamp `limit` to
      `MAX_LIMIT`, reject an out-of-bound `offset` with `400`, and stamp
      all three headers; the employee list's total reflects its
      `?department=`/`?status=` filters (counted before paging, via
      `Select::count`). New DB-gated tests
      (`tests/requests/pagination.rs`): `worker_list_paginates_and_clamps`
      (page size, true total, limit clamp, offset bound) and
      `benefit_plan_list_paginates` (page size + offset bound). Verified:
      `cargo build --lib`, `cargo clippy --all-targets -- -D warnings`,
      `cargo test --lib` (140 passed), `cargo test -- --ignored` against
      a real Postgres (21 passed, including both new tests), `cargo fmt
      --check`.

- [x] WPM-T41 (2026-10-02) **Workforce capability analysis.**
      `rules/capability.rs` (pure): `depth_status` grades each skill
      `undeclared` / `no_proficient` / `thin` / `adequate` from declared
      proficiency, and `validate_thresholds` bounds the caller's
      `min_proficiency` (1–5) and `min_depth` (≥ 1). New read-only
      `GET /api/workforce-intelligence/capability-analysis` in
      `controllers/intelligence.rs`: per-skill declared / proficient
      counts, proficient departments, proficient share (terms-carrying
      ratio), category rollup, and the thresholds echoed back; no new
      stored state. Pure rules pinned (4 tests, run standalone); DB-gated
      `capability_analysis_reports_skill_depth` added. **Not yet
      verified (API):** `cargo build`/`clippy`/`test` — the sibling
      `authentication-verifier` crate is absent on this machine.
      Front-end: `capabilityAnalysis()` client + a "Capability analysis"
      section on `/learning` (proficiency-bar and depth selectors,
      per-skill depth table); path-map unit test extended; svelte-check
      0, vitest 35/35, build green.

- [x] WPM-T42 (2026-10-02) **Org-chart view modes.** `/org-chart` gains
      a view switch — by manager (the existing tree), by department, by
      level — over the same service-derived manager forest; no API
      change, so every view lists exactly the same people. Regrouping
      lives in `src/lib/orgViews.ts` (pure, 6 unit tests); strings added
      to all 16 locales. A fourth **by tenure** view reads the new
      `tenure` band on each org-chart node (`controllers/hr_core.rs`,
      from `rules::talent::months_of_service` + `tenure_bucket`; Rust
      change unbuilt — see WPM-T41). **By location** view: migration
      `m20261002_000020_worker_location` adds nullable `workers.location`;
      `rules::org::normalize_location` (pure; trimmed, blank ⇒ unknown,
      ≤ 100 chars; 1 test, run standalone) backs `location` on
      `POST`/`PUT /api/workers`; the org-chart node carries it; the UI
      groups by place with "not recorded" last and shows it on the
      worker page. Rust/migration unbuilt and un-run (see WPM-T41). svelte-check 0, vitest
      41/41, build green.

- [~] WPM-T43 (2026-10-02) **Testcontainers Keycloak integration test.**
      `tests/keycloak.rs` (feature `keycloak`, `#[ignore]`) starts a real
      Keycloak via `testcontainers` on Podman, imports
      `tests/keycloak/realm-wpm.json`, mints tokens with the password
      grant, and checks 401 / 403 / policy-authorised writes against the
      booted app (see `spec/testcontainers-keycloak/index.md`).
      **Verified (2026-10-02):** the container start, realm import and
      password-grant token flow ran against a real Keycloak 26.0 on
      Podman via a standalone `testcontainers` 0.28 program — `hr-user`'s
      access token carries `aud=wpm-api`, `realm_access.roles`
      (`wpm-hr`, `wpm-payroll`), `groups=["/org/engineering"]` and
      `organization_ref`; `plain-user`'s carries none of the role / group
      / org claims; the JWKS endpoint serves RS256 keys. **Not yet run:**
      `tests/keycloak.rs` itself (the Rust crate cannot build here —
      missing sibling crates). Note: the Podman VM had no outbound
      network, so the image was fetched on the host from the registry API
      and `podman load`ed.

- [x] WPM-T44 (2026-10-02) **Workforce metrics layer.**
      `rules/metrics.rs` (pure; 5 tests run standalone): one `DEFINITIONS`
      vocabulary plus `headcount_on` (hire and termination dates both
      respected), `starters`, `leavers`, `turnover_rate` (leavers ÷ mean
      of opening and closing headcount; `None`, never 0, without a base)
      and `span_of_control`. New read-only
      `GET /api/workforce-intelligence/metrics?from=&to=` returns the
      numbers beside their definitions. **Time-to-fill** is `filled_on - opened_on` in
      calendar days over requisitions filled in the period (mean, median, count);
      migration `m20261002_000021_requisition_filled_on` adds nullable
      `requisitions.filled_on`, set when a requisition moves to `filled`;
      requisitions with a missing or out-of-order date are left out, not
      guessed (rules pinned: `time_to_fill_days`, `fill_time_summary`).
      Requisitions filled before the column existed have no fill date.
      **Headcount reconciled:** `rules::metrics::is_employed_on` is now
      the single definition of "employed"; `/overview`, `/capability`,
      `/capability-analysis` and `/metrics` all count workers employed
      on the date (previously `/overview` counted every live record,
      terminated included). DB-gated tests pin `/overview` ==
      `/metrics` closing headcount. Rust unbuilt (see WPM-T41). **Front-end:** `/metrics` route
      (period picker, each metric beside the service's definition,
      `format.rate` null-not-zero), nav link, `workforceMetrics()` client +
      path-map test, strings in all 16 locales; svelte-check 0, vitest
      43/43, build green.

## Phase 10 — strategic workforce planning (WPM-R34–R38, WPM-D26–D28)

> **Verification (2026-10-02, WPM-T41–T46, T51–T55):** the Rust compiles
> and the full database-backed request suite passes — **41 of 41** against a
> real PostgreSQL 18 (every migration applied) — using a scratch copy with
> signature-only stubs of the two sibling crates (`entity-ref`,
> `authentication-verifier`; `EntityRef` parsing stubbed faithfully, the
> ABAC policy / PASETO verifier **not**). So auth behaviour
> (`tests/enforcement.rs`) and the Keycloak test (`tests/keycloak.rs`) were
> still unrun *at that time* — **both have since been run and pass (WPM-T73,
> 2026-10-04, against the real sibling crates and a real Keycloak 26)**; the
> "DB-gated … not run" remarks in individual task entries above predate these
> runs. `cargo clippy` is clean on every new file.

Design: [strategic-workforce-planning.md](strategic-workforce-planning.md).
Order matters — each task's inputs come from the one before; pure core
first in each, per the three-part rule.

- [x] WPM-T45 (2026-10-02) **Headcount snapshot job.** Migration
      `m20261002_000022_headcount_snapshots` (append-only, aggregate
      only, unique per organization × department × date); pure
      `rules::metrics::snapshot_rows` (employed headcount, FTE in
      hundredths, starters/leavers over the window since the previous
      snapshot — `None`, not 0, for the first; 2 tests); loco task
      `snapshot_headcount [as_of:YYYY-MM-DD]` (`tasks/snapshot.rs`,
      idempotent: an existing row is skipped); read endpoint
      `GET /api/workforce-intelligence/headcount-history`, scoped to the
      caller's organizations. **Schedule it** (daily or weekly) in
      deployment — it records nothing by itself. **Verified:** the Rust
      type-checks (`cargo check --all-targets --workspace`) against
      signature-only stubs of the two missing sibling crates, my files are
      clippy-clean, and the lib tests pass; the DB-gated
      `headcount_snapshots_are_recorded_idempotently` has **not** been run.
      Also fixed: `rules::privacy::SOFT_DELETED_TABLES` was no longer
      sorted after the `employee_skills` → `worker_skills` rename
      (`sweep_table_list_is_sound` was failing).
- [x] WPM-T46 (2026-10-02) **Role profiles + required skills.**
      Migration `m20261002_000023_role_profiles` (`role_profiles`, one
      live per job title; `role_skill_requirements`, one per profile ×
      skill); pure `rules::roles` (`normalize_job_title`,
      `validate_requirement`: proficiency 1–5, importance
      `critical`/`important`/`useful`; 2 tests); `controllers/roles.rs`:
      `POST`/`GET /api/role-profiles`, `GET /api/role-profiles/{pid}`
      (requirements critical first), `PUT`/`DELETE
      …/requirements` (upsert / remove), all audited; `role_profiles` joins
      the retention sweep list (now 42). Front-end `/roles`: profile
      select + create, requirements table, add/remove, nav link
      (`nav.roles` in all 16 locales; page headings in English like
      `/learning`). Not yet done: importing from ESCO / UK GDAD PCF (see
      `spec/esco/`, `spec/uk-gdad-pcf/`). **Verified:** Rust
      type-checks and lib tests pass against stubbed sibling crates,
      clippy-clean on the new files; DB-gated
      `role_profiles_hold_required_skills` not run. svelte-check 0,
      vitest 43/43, build green.
- [x] WPM-T47 (2026-10-03) **Workforce plans + demand lines.** Migration
      `m20261002_000027_workforce_plans` (`workforce_plans`, at most one
      `active` per organization via a partial unique index;
      `plan_demand_lines`, one per plan × department × role × date). Pure
      `rules::planning`: `draft → active → archived` machine through the
      shared `lifecycle::check`, `validate_plan` (ordered horizon, attrition
      0–10000 bp), `validate_line` (date inside the horizon). Audited
      endpoints: `POST`/`GET /api/workforce-plans`, `GET …/{pid}`, `POST
      …/status`, `PUT`/`DELETE …/demand-lines`; an archived plan is
      read-only; plans respect the caller's organization scope. A plan holds
      **aggregate hypothetical headcount only and never references a worker
      row** (WPM-D26). `workforce_plans` joins the retention sweep list
      (now 48).
- [x] WPM-T48 (2026-10-03) **Forecast + gap analysis.** Pure core in
      `rules::planning`: `project_supply` (opening less expected leavers at
      the stated attrition, rounded to a person, never negative, **no hires
      assumed**), `headcount_gap` (demand − supply), `suggested_levers`
      (suggestions, not decisions), `competency_shortfall`, and
      `observed_attrition_bp` — annualised leavers over mean headcount from
      snapshots, `None` (**insufficient history**) with fewer than 3
      snapshots, a window under 60 calendar days, or any missing leavers figure
      (WPM-D27). 7 tests. `GET /api/workforce-plans/{pid}/forecast` returns
      per department × date: opening, planned demand (lines add up),
      projected supply, gap, levers, and **competency gaps** per demand line
      with a role profile — employed workers in the department already
      proficient in each required skill vs the headcount needed, plus the
      reskill pool (declared below the bar). The attrition source
      (`plan_assumption` / `observed_snapshots` / `insufficient_history`) and
      `hires_assumed: 0` are echoed. Aggregate only.
- [x] WPM-T49 (2026-10-03) **Strategic alignment view.** Migration
      `m20261002_000028_plan_objectives` (`plan_objectives`,
      `demand_line_objectives`). `POST …/objectives`, `PUT
      …/demand-lines/{line}/objectives` (objectives must belong to the
      plan). `GET …/alignment`: share of planned headcount whose line serves
      an objective (terms-carrying, null-not-zero), objectives with no demand
      (**unresourced**), demand lines serving no objective (**unaligned**),
      and critical roles with no ready successor via the shared
      single-point-of-failure rule (in the plan's departments and overall).
- [x] WPM-T50 (2026-10-03) **Front-end `/planning`.** Plan table with
      status, horizon, lines and total planned headcount (comparison), create
      form (organization from the caller's scope, optional attrition %),
      lifecycle buttons, the forecast table with the assumption and
      "insufficient history" stated plainly, competency-gap sub-rows,
      alignment panel, demand-line editor with objective checkboxes, add
      demand and add objective forms; nav link `nav.planning` in all 16
      locales (page copy English, as `/learning`). svelte-check 0, vitest
      43/43, build green. **Verified:** Rust type-checks, 132 lib tests, and
      the database-backed suite **33/33** against PostgreSQL 18 including
      `workforce_plan_forecasts_gaps_and_alignment`; clippy-clean on the new
      files. Not done: an observed per-department attrition rate (the
      observed rate is organization-wide), and plan-to-plan diffing beyond
      the totals table.

- [x] WPM-T51 (2026-10-02) **Skills gap against a target role.** Pure
      `rules::gap` (`grade` met / below / undeclared, `shortfall` only when
      known, `readiness` critical + overall; 3 tests). `GET
      /api/workers/{pid}/role-gap?role_profile_pid=` (one worker's
      declarations vs a profile, terms-carrying readiness ratios) and `GET
      /api/role-profiles/{pid}/gap` (per requirement: employed workers
      meeting / below / undeclared and coverage; aggregate, no one named).
      Compares **declarations**; `undeclared` is unknown, never a numeric
      shortfall. Front-end: a "Can we staff this role today?" table on
      `/roles` and a "Compare to a role" panel (`RoleGap.svelte`) on the
      worker page. Rust type-checks and lib tests pass against stubbed
      sibling crates; DB-gated `role_gap_grades_declarations_against_a_role`
      not run. svelte-check 0, vitest 43/43, build green.

- [x] WPM-T52 (2026-10-02) **CPD ledger.** Migration
      `m20261002_000024_cpd` (`cpd_requirements`, `cpd_entries`,
      `professional_registrations`; amounts stored as hundredths of a
      unit). Pure `rules::cpd` (units hours/points, categories, entry and
      requirement validation, `progress` recorded vs verified,
      `registration_status` with a 90-calendar-day window; 6 tests). Controller
      `controllers/cpd.rs`: requirements (create/list, optionally scoped to
      a job title), worker entries (record with optional evidence note/link,
      list, verify, withdraw), registrations with expiry status, per-worker
      `cpd-progress`, and an aggregate `/api/cpd/overview` over employed
      workers (met on recorded and on verified entries as terms-carrying
      ratios; registrations expiring/expired). All audited. **Privacy:**
      entries and registrations join subject-access export and erasure
      (free text and links scrubbed, rows soft-deleted) and the retention
      sweep list (now 45). `source`/`external_ref` + a unique index make the
      table the landing place for LMS completions (WPM-T54).
      Front-end `/cpd` (overview, define requirement, per-worker progress,
      ledger with verify, record form) and nav link in all 16 locales.
      Rust type-checks and 114 lib tests pass against stubbed sibling
      crates, clippy-clean on the new files; DB-gated
      `cpd_ledger_tracks_progress_and_registrations` not run. svelte-check
      0, vitest 43/43, build green. Not done: a CPD requirement derived from
      a gap or objective (WPM-T48/T49).

- [x] WPM-T53 (2026-10-02) **Skills matching + internal mobility.**
      Employee-facing by design (WPM-D28): the tool never ranks people or
      selects anyone. Pure `rules::mobility` (`compare_fit` orders *roles*
      for one worker by that worker's own fit — critical requirements
      first, exact cross-multiplied fractions, no-requirement roles last;
      `interest_target`, `same_title`; 4 tests). Migration
      `m20261002_000025_mobility_interests` (exactly one of role profile /
      requisition, one live interest per worker × target). `GET
      /api/workers/{pid}/role-matches`, `…/opportunities` (open
      requisitions in the worker's organization with the worker's fit, or
      `null` fit when no profile exists), `POST`/`GET
      …/mobility-interests`, `DELETE /api/mobility-interests/{pid}`
      (withdraw), and `GET /api/mobility/interest-summary` — **aggregate
      counts only, never who**. Privacy: interests join subject-access
      export, erasure (note scrubbed, withdrawn) and the retention sweep
      list (now 46). Front-end: a self-service `Mobility` panel on the
      worker page and an "Employee interest" table on `/roles`. Rust
      type-checks and 118 lib tests pass against stubbed sibling crates,
      clippy-clean; DB-gated
      `internal_mobility_matches_own_skills_and_keeps_interest_private` not
      run. svelte-check 0, vitest 43/43, build green. Not done: a manager
      view of *who* is interested (deliberately — would invite ranking);
      notifying a hiring manager is a design question.

- [x] WPM-T54 (2026-10-02) **LMS completion sync.** The LMS owns courses
      and learners (WPM-D10) and pushes completions; WPM is not the LMS.
      Pure `rules::lms` (`validate_completion`, `cpd_credit` hours *or*
      points, `enrollment_action` Create / Complete / Unchanged; 3 tests).
      `POST /api/lms/completions` takes up to 500 events
      `{person_ref, organization_ref?, course_ref, completed_on,
      certificate_expires_on?, hours? | points?, external_ref}` and returns a
      per-event `result` — `applied`, `unchanged`, `unmatched` (no such
      worker), `ambiguous` (several organizations; send `organization_ref`),
      or `invalid` — so one bad event never poisons the batch. Each applied
      event creates or completes the worker's **training enrollment** (which
      already feeds certificate-expiry reporting and learning-path progress)
      and, when it carries credit, lands a **CPD entry** with `source =
      "lms"`. Idempotent per event: `external_ref` is the key (a unique
      index on worker + external_ref), so a redelivery changes nothing.
      Intended caller is the LMS's service account (`svc` ABAC attribute);
      transport (push vs. polling, xAPI translation) is the integration's
      job. The CPD ledger marks LMS entries. Rust type-checks and 121 lib
      tests pass against stubbed sibling crates, clippy-clean; DB-gated
      `lms_completions_update_enrollments_and_cpd_idempotently` not run.
      Not done: mapping a course to the skills it evidences.

- [x] WPM-T55 (2026-10-02) **AI-driven change tracker.** Operational
      support for an organization moving through automation or AI-driven
      role change; it tracks *roles and skills*, never people (WPM-D28).
      Migration `m20261002_000026_change_initiatives`
      (`change_initiatives`, `initiative_role_impacts`,
      `initiative_skill_shifts`). Pure `rules::change`: lifecycle
      `draft → active → completed | cancelled` through the shared
      `lifecycle::check`, role impacts `displaced` / `reshaped` /
      `created` with a timeframe, skill shifts `rising` / `declining`,
      `skill_readiness` tally (undeclared kept apart from below); 4 tests.
      Audited CRUD in `controllers/change.rs`; an initiative is read-only
      once closed. `GET /api/change-initiatives/{pid}/readiness` is
      **aggregate only**: per affected role, employed workers and how many
      have an active `reskill` development plan (WPM-T22), and per rising
      skill how many workers in displaced/reshaped roles meet, fall below,
      or have not declared it (bar defaults to 3) — no individual named or
      ranked. `change_initiatives` joins the retention sweep list (now 47).
      Front-end `/change` (initiative picker + create, lifecycle buttons,
      roles and skills readiness tables, record role impact / skill shift)
      and nav link in all 16 locales. Rust type-checks and 125 lib tests pass
      against stubbed sibling crates, clippy-clean; DB-gated
      `change_tracker_reports_aggregate_readiness` not run. svelte-check 0,
      vitest 43/43, build green. Not done: linking an initiative's rising
      skills to suggested CPD requirements or reskill plans (WPM-T48/T49).

- [x] WPM-T56 (2026-10-03) **Financial-planning-led workforce planning.**
      Answers "what can we afford to hire" over a plan's headcount gaps.
      Migration `m20261002_000029_plan_budget` (plan `budget_minor` +
      `budget_currency`, `on_cost_bp`; validated together / bounded). Pure
      `rules::cost`: `cohort_average_minor` (**published only for a cohort
      of at least 5 salaried workers** — an average of one or two people is
      their salary), `with_on_cost`, `hires_to_close`, `pick_unit_cost`
      (benchmark median beats department average beats nothing — never a
      guess), `weighted_unit_cost`, `annual_cost`, `against_budget`,
      `valid_currency`; 6 tests. `GET
      /api/workforce-plans/{pid}/cost?currency=`: hires needed per
      department × date (from the forecast gap), unit cost and its source,
      annual cost with the plan's on-cost, a total that **never mixes
      currencies**, uncosted groups with a reason (`insufficient_history` /
      `no_unit_cost` / `salary_not_visible`), and affordability against the
      budget. Each department is costed at its latest target date; annual
      run-rate of salary only (no recruitment or onboarding). **Salary is
      sensitive:** the money is withheld (`salary_visible: false`) from any
      caller whose policy decision carries the `mask` obligation — the same
      rule that masks a worker's salary. Front-end: budget, currency and
      on-cost on the plan form and a "Cost of closing the gaps by hiring"
      panel on `/planning`. **Verified:** Rust type-checks, 138 lib tests,
      database-backed suite **34/34** against PostgreSQL 18 (including
      `workforce_plan_costs_hiring_against_a_budget`), clippy-clean on the
      new files; svelte-check 0, vitest 43/43, build green. Not done:
      recruitment/onboarding cost, multi-year phasing, per-department
      budgets, and exercising the masked path against the real policy
      engine (stubbed here).

- [x] WPM-T57 (2026-10-03) **Job-capability frameworks: UK GDAD PCF import.**
      Frameworks become first-class: migration
      `m20261002_000030_capability_frameworks` (`capability_frameworks` with
      attribution, licence, scale; provenance columns on `role_profiles` —
      framework, external ref, profession, role, level, order, management
      track; `source_level` / `source_scale_max` on requirements). Pure
      `rules::framework` (role-summary and baseline parsers, slug parts,
      `LevelMapping::{Identity, Linear}`, `job_title`, `is_management_track`;
      6 tests) and `rules::roles::progression_diff` (1 test). Loco task
      `import_framework dir:<clone> [scale:] [overwrite_levels:true]`
      (`tasks/import_framework.rs`): idempotent, skips and counts skills with
      no stated level, never overwrites a planner's level or importance. API:
      `GET /api/capability-frameworks`, `GET /api/role-profiles?framework=`,
      `GET /api/role-profiles/{pid}/progression`. Front-end `/roles`:
      framework filter, profession › role › level grouping, attribution and
      licence on every imported profile, the framework's wording per skill,
      framework level beside WPM's, and a "Next level up" panel. **First real
      import:** 201 profiles / 161 skills / 1,654 requirements from the local
      clone; re-run created nothing. **Verified:** Rust type-checks, 145 lib
      tests, database-backed suite **35/35** against PostgreSQL 18 (including
      `capability_framework_import_and_progression` over a synthetic
      fixture), clippy-clean on the new files; svelte-check 0, vitest 43/43,
      build green. Details and the mapping decision: `spec/uk-gdad-pcf/`.
      Not done: ESCO (see `spec/esco/`); other frameworks (SFIA, …); skill
      categories (all `other`); editing a skill; per-level importance.

- [x] WPM-T58 (2026-10-03) **Lily Gantt chart and kanban board.** Added
      `@lilydesignsystem/svelte-gantt-chart` and
      `@lilydesignsystem/svelte-kanban-board` (0.1.0; there is no separate
      "helpers" package — the helper components live in
      `@lilydesignsystem/svelte-headless`, already a dependency). Both are
      headless: no CSS and label-gated controls, so `LilyKanban.svelte` and
      `LilyGantt.svelte` give them labels and minimal styling. The kanban
      moves cards by pointer **or** a keyboard-accessible "Move to…" menu
      (WCAG 2.5.7). Used on `/change` (initiative lifecycle board — a move is
      a status transition; the service refuses illegal ones with 422 and the
      reload puts the card back) and `/planning` (plan status board, and a
      read-only **plan timeline**: each plan's horizon as a bar, each demand
      line a milestone, no editing offered). The requisitions board keeps
      SVAR's kanban. English labels, like the other new pages. 2 render
      tests; svelte-check 0, vitest 45/45, build green.

- [x] WPM-T59 (2026-10-03) **Skills: edit, categorise, external references.**
      Migration `m20261003_000031_skill_external_refs` — how a catalogue
      skill is known in an external framework (the PCF name, an ESCO concept
      URI): one reference per skill per framework and one skill per
      reference. WPM keeps its own catalogue; this is a reference, never a
      second skills model. `PUT /api/skills/{pid}` renames (names are
      unique) and recategorises (closed set); `GET /api/skills` now carries
      each skill's `external_refs`; `POST`/`DELETE /api/skills/{pid}/refs…`.
      **Categorising:** pure `rules::learning::suggest_category` — ordered
      keyword rules (compliance, leadership, domain, technical) returning the
      category *and the keyword that triggered it*; `GET
      /api/skills/category-suggestions` lists suggestions for skills still
      `other` and `POST …/apply` applies the current suggestion to chosen
      skills — suggestions only, never automatic. **Measured on the real
      PCF skills: 73 of 161 get a suggestion (about 45%)**; the other 88
      stay `other` for a planner. The PCF import now matches skills by
      reference first and records the PCF name, so **a planner's rename
      survives a re-import with no duplicate** (pinned by test). Front-end
      `/skills`: counts by category, search and filter, inline rename,
      per-skill category select, suggestion table with Accept / Apply all,
      reference chips; `nav.skills` in all 16 locales. OpenAPI literal
      recursion limit raised (`lib.rs`). **Verified:** Rust type-checks, 147
      lib tests, database-backed suite **36/36** against PostgreSQL 18,
      clippy-clean on the new files; svelte-check 0, vitest 45/45, build
      green. Not done: deleting a skill; merging duplicates.

- [x] WPM-T60 (2026-10-03) **ESCO import and seeding.** Migration
      `m20261003_000032_esco` (`esco_skills`, `esco_occupations`,
      `esco_occupation_skills`) — a pinned local reference copy, replaced
      wholesale per import. Pure `rules::csv` (RFC 4180 parser, 4 tests) and
      `rules::esco` (header-resolved parsers for skills, occupations and
      relations; the draft category and importance mappings; exact label
      normalisation; 5 tests). Loco task `import_esco dir:<csv dir>
      [lang:en] [version:]` loads the files, writes the `esco` framework row
      (attribution, pinned version, "no proficiency scale" note), and links
      catalogue skills by **exact normalised label only** — ambiguous labels
      are counted and left. API in `controllers/esco.rs`: occupation and
      skill search, an occupation's skills, and `POST
      /api/role-profiles/from-esco` which **requires the planner's starting
      level** (ESCO states none), drafts essential skills as `important` and
      optional as `useful`, matches or creates and links catalogue skills, and
      allows one profile per occupation; manual links via the `/skills/{pid}/refs`
      endpoint are checked against the pinned copy. Front-end: "Start from an
      ESCO occupation" on `/roles` and "Link ESCO" on `/skills`. **Verified:**
      Rust type-checks, 156 lib tests, database-backed suite **37/37** against
      PostgreSQL 18 (including `esco_import_search_and_seeding` on a
      synthetic fixture), clippy-clean on the new files; svelte-check 0,
      vitest 45/45, build green. **Not verified against real ESCO files**
      (the download is emailed): the importer is built from the published
      structure and stops with a message naming any missing column. Not done:
      qualifications / EQF; ESCO alternative labels for matching; other
      languages beyond choosing `lang:`.

- [x] WPM-T61 (2026-10-03) **Skill delete and merge; build fix.** Closes the
      WPM-T59 gap. Pure `rules::skill_merge` (higher level and target kept;
      stronger importance kept; `Usage` and `deletable`; 3 tests). `GET
      /api/skills/{pid}/usage` (declarations, profile requirements, plan
      items, initiative shifts), `DELETE /api/skills/{pid}` — **only a skill
      nothing uses**, else `422` telling the planner to merge — and `POST
      /api/skills/{pid}/merge {into_pid}`: one transaction repoints every
      table that references the skill and, where both skills already appear
      (a worker who declared both, a profile requiring both, a plan listing
      both), **keeps the stronger statement** (higher proficiency and
      target, higher minimum, stronger importance, the target's reference per
      framework), then retires the source; audited with the counts. Front-end
      `/skills`: Merge… (choose a target, confirm) and Delete (confirm; the
      server refuses a skill in use). **Also fixed:** `models/memberships.rs`
      called `order_by_asc` without importing `QueryOrder`, so the crate did
      not compile; the import is added (confirmed: the scratch build now
      compiles with no patch). **Verified:** Rust type-checks, 159 lib tests,
      database-backed suite **38/38** against PostgreSQL 18 (including
      `skills_merge_keeps_the_stronger_statement_and_delete_refuses_use`),
      clippy-clean on the new files; svelte-check 0, vitest 45/45, build
      green. Not done: undoing a merge; merging more than two skills at once.

- [x] WPM-T62 (2026-10-03) **Real-data verification: ESCO v1.2.1 and the UK
      GDAD PCF.** Both frameworks loaded from real data and exercised through
      the running API. **PCF:** 201 profiles / 161 skills / 1,675
      requirements; fixed a real-file baseline format that the first import
      had skipped (21 lines → 0). **ESCO:** `scripts/esco-fetch.py` builds the
      CSVs from ESCO's public API (taxonomy crawl closing over nested
      occupations, skills-hierarchy crawl, bulk skill lookup, curl transport,
      resumable cache); loaded **3,039 occupations (= ESCO's published
      figure), 13,653 skills (portal states 13,939 / 13,890 — not
      reconciled), 126,051 relations**, none dangling, in seconds; seeding the
      real *software developer* occupation drafted 108 requirements in 0.37 s.
      Honest findings recorded in `spec/esco/` and `spec/uk-gdad-pcf/`: only 2
      of 161 PCF skills link by exact label; the draft category mapping labels
      all ESCO *knowledge* skills `domain` (crude — use ESCO's skills hierarchy
      next); the `test` environment recreates the database on boot. The CSVs
      are API-derived, not the official emailed package.

- [x] WPM-T63 (2026-10-03) **My role and skills in the UK GDAD PCF and ESCO.**
      Each person selects their **current role** in each framework — a PCF
      role level (a role profile) or an ESCO occupation — and then the
      **skills they have, at their own level**. Migration
      `m20261003_000033_worker_framework_roles` (one per worker per framework;
      worker-owned, so it joins subject-access export and erasure). Pure
      `rules::framework_roles` (selectable frameworks; a bounded batch with
      no blank or repeated refs and every level 1–5; 2 tests). API in
      `controllers/framework_roles.rs`: `GET /api/frameworks/selectable`,
      `GET`/`PUT`/`DELETE /api/workers/{pid}/framework-roles[/{framework}]`,
      `GET …/{framework}/skills` (the role's skills with the person's declared
      level — for the PCF each requirement with the framework's own level and
      wording as a *prompt*; for ESCO the essential then optional skills) and
      `PUT …/{framework}/skills` (all-or-nothing; `null` deselects). Selected
      skills are **ordinary `worker_skills` declarations**, so gap analysis,
      matching and capability pick them up; an ESCO skill is resolved to — or
      created and linked as — a catalogue skill (the resolution logic is now
      shared with role seeding). **The person chooses their own level**; the
      framework's wording informs, it does not decide, and nothing here is an
      assessment. Writes pass the worker-record authorization pass. Clearing a
      role leaves the declared skills. Front-end `/me` ("My profile"): a
      panel per framework (PCF role-level picker grouped profession › role;
      ESCO occupation search), a skills table with "I have it" and level, only
      the changes saved, add-another-ESCO-skill search; the person's worker
      record comes from their organization memberships; `nav.me` in all 16
      locales. **Verified:** Rust type-checks, 161 lib tests, database-backed
      suite **39/39** against PostgreSQL 18 (including
      `a_person_selects_their_pcf_and_esco_roles_and_skills`), clippy-clean on
      the new files; svelte-check 0, vitest 47/47 (including a panel test that
      pins "saves only what changed"), build green. Not done: choosing a role
      for *another* person (HR on behalf of); history of past roles; a "why am
      I not in this role yet" gap view from the selection (the existing
      role-gap panel does that for the PCF).

- [x] WPM-T64 (2026-10-03) **HR on behalf; past roles and skills over time;
      aspirations.** Builds on WPM-T63. Migration
      `m20261003_000034_career_history`: **time becomes first-class.**
      - **On behalf of.** HR (or anyone the policy lets write to a worker's
        record) can set a role and select skills for someone else; the same
        authorization pass applies. Every row records `recorded_by` and
        `on_behalf` (`auth::acting_for_other` — the caller's `sub` is not the
        person), the audit entry carries it, and the UI shows "on their
        behalf" on history rows and a notice while HR edits another person's
        role. `/workers/{pid}` now carries the role-and-skills panels.
      - **Past roles.** `worker_framework_roles` gains `started_at` /
        `ended_at`: changing a role *closes* the previous one and opens the
        next (choosing the same role again changes nothing); clearing closes
        it. Only one role per framework is current (a partial unique index).
        `GET /api/workers/{pid}/role-history`, and `POST …/framework-roles/
        {framework}/past` records a role held in the past with first and last
        day — it may not overlap another role in the framework, must end after
        it starts and not in the future.
      - **Past skills and levels.** `worker_skill_history`: every declaration
        change closes the old level's interval and opens a new one, through one
        recorder (`models::skill_history::record`) called from the skill
        declaration endpoint and the framework selection; declaring the same
        level again is not a change. Existing declarations were **backfilled**
        (start = the date last assessed) — verified by hand on synthetic rows,
        soft-deleted ones ignored, re-run idempotent. `GET …/skill-history`,
        `POST …/skill-history/past` (retrospective, no overlap per skill) and
        `GET …/skills-as-of?at=` ("what did they have on this date": roles and
        skills at the end of that day, from the history).
      - **Future: aspirations, learning goals, growth ideas.**
        `worker_aspirations`: a *skill* target (level 1–5) or a *role* (PCF
        level or ESCO occupation), a horizon (`within_1y` … `someday`), a status
        (`idea` → `achieved` / `dropped`) and a note in the person's own words.
        **Private by default**: the person sees all of theirs, anyone else only
        those they **share** (and is told how many are hidden). Each shows
        progress against today's declared skills (a skill: current level and
        gap; a PCF role: requirements met; an ESCO role: essential skills had).
      Pure `rules::career` (half-open intervals, past-entry validation,
      overlap, level-at, who-is-the-person, aspiration validation, who may
      view; 6 tests). Privacy: history and aspirations join subject-access
      export and erasure. Front-end: `CareerHistory`, `Aspirations`,
      `RolePicker` on `/me` and `/workers/{pid}`. **Verified:** Rust
      type-checks, 167 lib tests, database-backed suite **40/40** against
      PostgreSQL 18 (including `career_history_and_aspirations`), clippy-clean
      on the new files; svelte-check 0, vitest 52/52, build green.
      **Not verified end to end:** the *on behalf* path and the hiding of
      unshared aspirations from other viewers depend on the real access-control
      engine (stubbed here; enforcement is off in tests, so everyone reads as
      the person) — the deciding rules (`is_self`, `can_view`) are unit-tested,
      and the UI messaging is component-tested. **Limits:** "shared" means
      visible to anyone who can view the record (no manager-only tier);
      retrospective dates are whole calendar days in UTC; skills merged by `/skills`
      merge are not re-recorded in the history; no edit/delete of a past
      entry yet.

- [x] WPM-T65 (2026-10-03) **Reporting lines, manager visibility of
      aspirations, groups.** Builds on WPM-T64.
      - **Upline / downline.** `GET /api/workers/{pid}/upline` (the chain
        above, nearest first; `level` 1 is the direct manager),
        `GET …/downline` (everyone below, with `depth`, `report_kind` and a
        direct/indirect summary) and `GET …/reports?kind=direct|indirect`.
        **Direct report** = depth 1; **indirect report** = reports of reports,
        any depth. Pure `rules::org::{upline, downline, report_kind}`
        (cycle-safe); employed workers in the caller's organizations only.
      - **Managers see downline aspirations.** Aspiration `shared` (boolean)
        became `visibility`: `private` (only the person), `manager` (everyone
        above them in the chain — indirect managers included) or `everyone`
        (migration `…000035`; existing shared rows became `everyone`).
        `GET …/downline-aspirations` lists the manager's whole downline with
        only what is visible to them. **Private items, and how many there are,
        are never revealed** (the earlier "N private hidden" count was
        removed). `rules::career::{Viewer, can_view}`.
      - **Groups** (communities of practice / interest / other; migration
        `…000036`): `groups` and `group_members`, CRUD plus
        `POST/DELETE /api/groups/{pid}/members` and
        `GET /api/workers/{pid}/groups`. **A worker can be in many groups at
        once**; membership has `joined_at`/`left_at` (leaving closes it, past
        membership is kept), is a worker-record write (a person manages their
        own, HR on their behalf, recorded as such), and joins subject-access
        export and erasure. A group is a label: it changes no manager, access
        or pay. Names are unique ignoring case.
      - **Multiple frameworks at once** already held from WPM-T63/64: one
        *current* role per (worker, framework), so a PCF role and an ESCO role
        are held together; both panels sit on `/me` and `/workers/{pid}`.
      - UI: `GroupsPanel`, `TeamAspirations`, a visibility selector on
        `Aspirations`. **Verified:** Rust type-checks, 171 lib tests,
        database-backed suite **41/41** against PostgreSQL 18 (new
        `reporting_lines_downline_aspirations_and_groups`), svelte-check 0,
        vitest 54/54, build green. **Not verified:** the *manager* resolution
        for a real caller (`viewer_of`: caller `sub` against `person_ref`s in
        the upline) needs the real identity flow — with enforcement off every
        caller reads as the person/manager, so only the rule, not the
        enforcement path, is tested. **Limits:** no nav entry (panels live on
        `/me` and the worker page); no group-level skill roll-up; groups are
        global, not per organization; downline reports ignore workers outside
        the caller's organizations.

- [x] WPM-T66 (2026-10-03) **Groups page and skill roll-up; dotted-line
      reports.** Builds on WPM-T65.
      - **`/groups` page** with a nav entry (`nav.groups` in all 16 locales):
        list, start a group, and per group its members and what it knows.
      - **Group skill roll-up** `GET /api/groups/{pid}/skills`: per skill, how
        many current members declare it, coverage (null for an empty group,
        never zero) and the spread of levels 1–5. **Aggregate only and
        floored:** a skill declared by fewer than three members — or any skill
        in a group of fewer than three — is withheld (and counted), so a
        distribution cannot point at one person; no member is named. Declared,
        not inferred. Pure `rules::groups::rollup`.
      - **Dotted-line reports** (migration `…000037`): a secondary reporting
        line, **anyone** may be a dotted-line manager (not only the solid-line
        chain), a worker may have several, dated like other history (ending one
        keeps it). `GET /api/workers/{pid}/dotted-line` (managers and reports,
        `?include_past=`), `POST …/dotted-line-managers`,
        `DELETE …/dotted-line-managers/{manager_pid}`. A write to the report's
        record (self, or HR on their behalf, recorded). Not a person
        themselves; loops allowed (not a hierarchy). **A dotted line changes
        no org chart, adds no direct/indirect report, and grants no access to
        manager-visible aspirations.** Joins export and erasure.
      **Verified:** Rust type-checks, 173 lib tests, database suite **41/41**
      (roll-up floor and coverage, dotted lines incl. loop/self/ended-kept),
      clippy-clean on the new code, svelte-check 0, vitest 55/55, build green.
      **Limits:** the dotted-line manager must be a worker the caller can see;
      groups remain global; a roll-up shows only the floor and distribution —
      no trend over time.

- [x] WPM-T67 (2026-10-03) **Groups per organization.** Builds on WPM-T65/66.
      Migration `…000038`: `groups.organization_ref` (required). **Backfill**
      (verified by hand on synthetic rows): a group takes the organization
      most of its members work in; a group with no members has nothing to say
      where it belongs, so it is retired (soft-deleted, empty ref). Names are
      now unique **within an organization** (the same name may exist in two).
      - `POST /api/groups` requires `organization_ref`, which must be one the
        caller can read; it cannot be changed later.
      - Reads are scoped like the org chart: `GET /api/groups` lists only the
        caller's organizations (`?organization_ref=` narrows to one); a group
        in another organization is `404` for members, skills, join, leave,
        update and retire; `GET /api/workers/{pid}/groups` hides groups outside
        the caller's scope.
      - **A worker can only join groups in their own organization** (`422`).
      - UI: `/groups` has an organization selector (from the caller's scope);
        the groups panel on `/me` and the worker page lists and creates groups
        in that worker's organization.
      **Verified:** Rust type-checks, 173 lib tests, database suite **41/41**
      (required org, cross-organization join refused, same name in another
      org, `?organization_ref=` filter), svelte-check 0, vitest 55/55, build.
      **Not verified:** the `404` for a group outside the caller's scope
      depends on the real membership scope (off in tests, where scope is
      unrestricted). **Limits:** a group cannot move organization or span
      several; a worker who changes organization keeps old memberships until
      they leave.

- [x] WPM-T68 (2026-10-03) **Organization transfers; cross-organization
      (confederation) groups.** Builds on WPM-T67.
      - **Transfers.** A worker's organization was never editable (`PUT
        /api/workers/{pid}` does not take it), so there was nothing to trigger
        "end memberships when someone moves". `POST /api/workers/{pid}/transfer`
        is that act: a write to the worker's record, to an organization the
        caller can read. **Group memberships no longer open to the new
        organization end** (closed with a stop time, kept as history); ones
        that still fit stay. The response lists what ended and says whether the
        solid-line manager is now in another organization — **it does not
        change the manager** (a deliberate, separate decision). Audited.
      - **Confederation groups** (migration `…000039`, `groups.scope`):
        `organization` (default) or `confederation` — a community covering the
        organization in `organization_ref` and **every organization beneath it**
        in the confederation tree. Creating one needs an organization that has
        member organizations. Members may come from any covered organization;
        an outsider is refused. Readable by anyone who can read **any**
        covered organization (so a member of a child org sees the community
        its parent started). `GET /api/groups?organization_ref=` now means
        "groups open to that organization": its own plus communities covering
        it. Pure `rules::groups::{covered_orgs, is_open_to, visible_to,
        memberships_to_end}` (3 new tests).
      - UI: a "spans member organizations" option and chip on `/groups`; the
        panel on `/me` lists what is open to the worker's organization;
        `transferWorker` client (no transfer screen yet).
      **Verified:** Rust type-checks, 176 lib tests, database suite **42/42**
      (new `confederation_groups_and_transfers`: tree membership, outsider
      refused, organization group stays its own, filter, transfer ends only
      what no longer fits and keeps history, a covered move keeps the
      community, no-op and malformed refs refused), clippy-clean on the new
      code, svelte-check 0, vitest 55/55, build green. **Not verified:** the
      caller-scope checks (read/create/transfer targets) need the real
      membership scope — off in tests. **Limits:** no transfer UI; a
      confederation edge ending later does not retroactively end memberships
      (only a transfer does); a confederation group cannot be converted back
      to one organization.

## Phase 9 — strategic workforce-planning capabilities (research backlog, unscoped)

Not tasks yet — each item below needs a design pass (a `spec/*.md`
addition with new WPM-D/WPM-R ids) before it can become a scoped WPM-T
entry, per the Phase 0 precedent and the "Three-part rule" at the top
of this file. Researched 2026-09-28 against current workforce-planning
and HR-tech vendor practice (Orgvue, Anaplan, Workday, SAP
SuccessFactors, Visier, ChartHop, Gloat, Eightfold, Fuel50) to ground
each definition, not guessed. Grouped by theme; cross-references note
where WPM already has a partial foundation to build on.

### Priority triage — strategic workforce planning, CPD, upskilling (re-ranked 2026-10-02)

**Focus (set 2026-10-02):** forecast future talent and skill needs;
identify the gaps between current capabilities and future goals; align
headcount and competencies with strategy. This pulls the draft-plan /
future-state work forward from "design pass needed" to a scoped design —
[strategic-workforce-planning.md](strategic-workforce-planning.md) — and
Phase 10 above.

**Existing foundation:** declared skills + targets (WPM-T20),
upskill/reskill development plans (WPM-T22), succession and talent
pipelines (WPM-T14, WPM-T22), capability analysis (WPM-T41), shared
metrics and one headcount definition (WPM-T44). **Gaps found:** no role
requirements (a target is per worker), no headcount history, no plan or
demand, and no CPD model (hours/points, required-per-period target,
registrations).

| Rank | Work | Serves | Tasks |
|---|---|---|---|
| 1 | Headcount snapshot job | Forecasting (history is unrecoverable) | WPM-T45 — **done** |
| 2 | Role profiles + required skills | Gap vs future goals; unlocks matching, mobility, reskill targets | WPM-T46 — **done** (+ gap: WPM-T51) |
| 3 | Workforce plans + demand lines | Future-state / scenario modeling (the strategy side) | WPM-T47 |
| 4 | Forecast + gap analysis | The core analytical process | WPM-T48 |
| 5 | Strategic alignment view | Headcount and competencies vs strategy | WPM-T49 |
| 6 | Front-end `/planning` | Surfaces 3–5 | WPM-T50 |
| 7 | CPD ledger (hours/points vs requirement, registrations) | Upskilling; independent of 1–6 | **done — WPM-T52** |
| 8 | Skills matching + internal mobility | Consumes role profiles (T46) | **done — WPM-T53** |
| 9 | LMS completion sync | Feeds CPD ledger and skills evidence | **done — WPM-T54** |
| 10 | AI-driven change tracker | Reuses reskill plans + role profiles | **done — WPM-T55** |

Deferred: analytics narrative, financial-planning-led planning (both
build on 3–5).

### Strategic planning, scenario modeling & forecasting

- [x] **Workforce transformation.** *(the plan / demand / gap / alignment machinery is WPM-T47–T50; the current-vs-target org-shape diff is WPM-T280, 2026-10-24, by role without the cost and skills comparison.)* The umbrella discipline —
      redesigning structure, headcount, and skills around a strategic
      shift (a restructure, an M&A, an automation programme) rather
      than incremental headcount changes. Orgvue frames it as
      connecting strategy to structure so leaders can see how a
      workforce decision moves cost, skills, and performance together.
      For WPM this would sit above the existing `succession_plans` /
      `talent_pipelines` tables (WPM-T14, WPM-T22) as a
      higher-altitude "current structure vs. target structure" diff
      view, not a new data model of its own.
- [x] **Workforce scenario modeling.** *(plans as draft worlds with several coexisting scenarios: WPM-T47; compared in the `/planning` totals table.)* "What-if" comparison of
      multiple future headcount/cost/skills states before committing —
      e.g. Anaplan's strength is modelling flexibility tied to
      financial forecasts for restructuring, M&A workforce impact, or
      multi-geography headcount plans. WPM has no scenario/draft
      concept anywhere in the schema today — every table is the
      current live state. A scenario feature needs a first-class
      "draft plan" object that can hold hypothetical headcount/org
      changes without touching live `employees` rows, plus a diff view
      against the live org chart (WPM-T2's cycle-safe derivation could
      likely be reused for a draft tree too).
- [x] **Workforce forecasting.** *(snapshots WPM-T45 + transparent supply projection WPM-T48; organization-wide observed attrition only.)* Projecting future headcount/attrition/
      cost from historical trend + planned change, distinct from
      scenario modeling (forecasting projects one likely path;
      scenario modeling compares several deliberate ones). Needs a
      time-series of historical headcount snapshots WPM does not
      currently retain (the schema is present-state only outside of
      `audit`/`outbox` rows) — a forecasting feature would first need a
      periodic headcount-snapshot job before any projection math is
      possible.
- [x] **Future-state modeling.** *(the same draft-plan primitive: WPM-T47–T49.)* Closely related to transformation and
      scenario modeling above — SHRM/Orgvue describe workforce
      planning as three pillars (current state, desired future state,
      the path between them). WPM's "current state" pillar is already
      strong (employees, org chart, skills, succession); "future
      state" and "path" have no representation yet. Likely the same
      draft-plan primitive as scenario modeling above, rather than a
      separate concept — worth designing together.

### Skills, talent, and internal mobility

- [x] **Skills, intelligence, and talent matching.** *(employee-facing role matching landed as WPM-T53; inferred skills remain out of scope.)* The Gloat/
      Eightfold/Fuel50 category: a continuously-updated skills graph
      matching people to roles, gigs, or projects, not just open
      requisitions. WPM already has a real (if simpler) skills
      foundation — `worker_skills` with declared 1–5 proficiency +
      optional target (WPM-T20) — that a matching feature could
      extend, rather than reintroducing a duplicate model. The gap
      versus the vendor category: today's skills data is
      self-declared, not inferred, and nothing matches a skill profile
      against an open requisition or pipeline (WPM-T5, WPM-T22)
      automatically.
- [x] **Internal mobility.** *(landed as WPM-T53.)* Employees moving laterally or upward
      inside the org rather than leaving — the core use case of the
      Gloat/Fuel50 "talent marketplace" category, distinct from
      `talent_pipelines`/succession (WPM-T22), which are
      manager/HR-curated rather than employee-initiated. WPM has the
      skills and profile data an internal-mobility feature would read
      from, but no employee-facing "browse open roles that fit my
      skills" surface, and no data model for an employee expressing
      interest in a move. Adjacent to — and should reuse the audience/
      visibility conventions of — the existing self-service surface
      (WPM-T11) and "My 360 requests" pattern (WPM-T29).
- [x] **Skills gap identification.** *(against a target role: WPM-T51; against a *future* role waits on the plans in WPM-T47/T48.)* WPM already ships this at the
      department level — the skills-matrix gap report (WPM-T20,
      `below_target` counts) and the cognitive/psychometric assessment
      gap views (WPM-T21, WPM-T35). The vendor bar (Visier, ChartHop)
      goes further: gaps compared against a *future* role's
      requirements, not just today's declared target — which depends
      on the future-state modeling item above existing first.
- [x] **Workforce capability analysis.** *(landed as WPM-T41.)* Aggregate "do we have enough
      of the critical skills to compete" reporting across the whole
      org, as distinct from the per-employee/per-department gap
      reports above — a strategic rather than operational read of the
      same skills data. Likely a new read-only aggregate view over
      `worker_skills` + `learning_paths` (WPM-T20) rather than new
      stored state, in the same spirit as the existing
      workforce-intelligence read views (WPM-T22,
      `/api/workforce-intelligence/*`).

### Metrics, analytics, and visualization

- [x] **Workforce metrics.** *(landed as WPM-T44, including time-to-fill and the headcount reconciliation.)* The base layer every item on this list
      depends on — standard counts/rates (headcount, turnover, time-
      to-fill, span of control, tenure mix) as a shared, named
      vocabulary rather than one-off numbers computed per screen. WPM
      already computes several of these ad hoc (dashboard tiles,
      benchmark rows (WPM-T16), working-time flags (WPM-T26)); a
      metrics layer would centralise the *definitions* so "headcount"
      means the same thing everywhere it's shown — the honesty-format
      module (WPM-T39) is the right precedent for how to centralise
      this kind of shared, tested derivation rather than reinventing
      it per route.
- [x] **Workforce analytics and insights.** *(landed as WPM-T69.)* The narrative/diagnostic
      layer on top of workforce metrics — Visier's own framing is
      turning the metrics above into "why" and "what should we do
      about it" rather than raw numbers on a dashboard. This is the
      most open-ended item on this list and the one most dependent on
      workforce metrics existing first as a well-defined foundation.
- [x] **Organizational visualization.** *(manager / department / level / tenure / location views landed as WPM-T42.)* WPM already has a recursive
      org-chart derivation (WPM-T2) but only ever renders one view of
      it. ChartHop's category is a *highly visual*, multi-mode org
      chart — by department, manager, location, job level, tenure —
      synced live from the underlying data. For WPM this is a
      front-end-heavy extension of the existing `/org-chart` route
      (view-mode switches over the same derivation) rather than a new
      data model.

### Cross-cutting

- [x] **Financial planning-led workforce planning.** *(landed as WPM-T56.)* FP&A-driven
      headcount planning — workforce cost is typically ~70% of opex,
      so Finance (not just HR) needs to model compensation strategy
      and headcount affordability directly, per BARC/Visier's framing
      of HR-Finance alignment. WPM's payroll domain (WPM-T15, WPM-T16)
      already holds the compensation data this would need to read;
      the gap is a Finance-facing "what can we afford to hire" planning
      view over that data, most naturally landing alongside the
      scenario-modeling / draft-plan work above rather than as its own
      silo.
- [x] **Support for AI-driven change.** *(operational tracker: WPM-T55.)* Gartner and SHRM's 2026
      CHRO research frames this as two things: (1) HR tooling that
      itself uses AI (inference, matching, forecasting — several items
      above), and (2) HR *supporting* an organization through AI-driven
      role and workflow change (which roles are displaced/reshaped,
      what reskilling is needed). WPM's existing pure-core rules
      style (no ML dependency anywhere in the stack today) suggests
      (2) — an operational workflow for tracking role/skill impact
      from an automation initiative — is the better fit than adding a
      model-inference dependency for (1).
- [x] **Support for learning management system (LMS) users.** *(completion sync: WPM-T54.)* WPM's
      own `training_enrollments`/course-URN model (WPM-T13) already
      assumes an upstream course catalog; the vendor pattern (xAPI/
      SCORM/LTI, or a REST roster-sync integration) is a *real* LMS
      feeding completions and skill evidence back into WPM rather than
      WPM being the LMS itself. This is closer to WPM-T3's existing
      upstream-client seam (`EntityRef`-keyed resolver, `http`/`stub`
      mode) than to a new subsystem — likely a new upstream client
      plus a webhook or polling endpoint for completion events.

## Phase 11 — insights, localization, people and cover, skills, joiners and leavers (WPM-R39–R50, WPM-D29–D37)

Delivered 2026-10-04 → 2026-10-06, after the Phase 9 backlog above was
triaged. Specified in the topic files
[people-directory-and-cover.md](people-directory-and-cover.md),
[skills-and-training.md](skills-and-training.md),
[joiners-and-leavers.md](joiners-and-leavers.md),
[communication-and-leadership.md](communication-and-leadership.md) and the
[locales spec](locales-for-global-sharing-with-svelte/index.md). Several entries
came out of the 2026-10-05 benchmark scan (`.sota/last-scan.json`).

- [x] WPM-T69 (2026-10-04) **Workforce insights.** *(traces to WPM-R49)* Builds on WPM-T44.
      `rules/insights.rs` (pure; 7 tests pass standalone): `derive` turns
      the shared metrics into findings (`turnover_high`,
      `headcount_shrinking`/`growing`, `span_wide`/`narrow`,
      `time_to_fill_slow`), each with an observation and a next step,
      attention-level first; thresholds are published heuristics
      (`THRESHOLDS`), and an unknown metric yields no finding. New
      `GET /api/workforce-intelligence/insights?from=&to=`; the metrics
      computation was extracted into `period_metrics` so both views share
      it. **Verified (2026-10-04):** compiles against the real sibling
      crates (`~/git/sixarm/main-x-service`); lib tests 183 pass; DB suite
      **43/43** with the new `workforce_insights_flag_a_shrinking_headcount`
      (3 → 1 headcount flagged, inverted period 422); clippy clean on the
      new code. **Front-end:** insights list under the metrics table on
      `/metrics`, `workforceInsights()` client + path-map test, strings in
      all 16 locales (observation/suggestion text is server English);
      svelte-check 0, vitest 55/55, build green.

- [x] WPM-T70 (2026-10-04) **Locale content routes and Sveltia CMS.** *(traces to WPM-R50, WPM-D37)*
      UI strings moved out of `i18n.svelte.ts` into content:
      `workforce-planning-management-ui-with-svelte/content/locales/<locale>/ui.json`
      for `ar-001 bn-001 cy-001 de-001 de-de en-001 en-gb en-us es-001
      fr-001 hi-001 id-001 pt-001 ru-001 ur-001 zh-001` (nested on the key
      dots, flattened at build; missing keys fall back to the `-001` base,
      then `en-001`). Routes: `/en-001/workers`, `/cy-001/…` via a
      `reroute` hook (`src/hooks.ts`, route tree unchanged) and the pure
      `src/lib/locales.ts`; bare aliases `/en/…`, `/cy/…` 301 to their
      `-001` locale; an unprefixed visit 302s to the cookie /
      `Accept-Language` / `en-001` locale (`/api`, `/_app`, `/assets`,
      `/admin`, the SSO and signout endpoints are exempt). The URL prefix
      drives the UI locale; the picker navigates; internal links use
      `l(path)`; `<html lang dir>` is set server-side. Sveltia CMS at
      `/admin/` (`static/admin/`), `config.yml` generated by
      `pnpm cms-config` from `en-001` (i18n `multiple_folders`; a test
      pins it up to date). Also commits the `/metrics` insights section
      (WPM-T69 UI). svelte-check 0, vitest 60/60, Playwright 19/19, build
      green. **Not verified:** the CMS itself (needs GitHub auth; the
      generated config follows the Sveltia docs but has not been loaded
      in a browser); en-gb/en-us/de-de are full copies, not overrides.

- [x] WPM-T71 (2026-10-04) **Regional locale overrides; es-es; localized insights.** *(traces to WPM-R49, WPM-R50)*
      Regional locales now hold only overrides of their `-001` base
      (`en-gb`: the `-ise` spellings; `en-us`, `de-de`, `es-es`: empty);
      `es-es` added (`/es-es/…`). Insights carry `params` (the figures)
      beside their English text; `/metrics` renders each finding from its
      `code` + `params` through `insights.<code>.observation|suggestion`
      strings in the 13 `-001` locales (`tp()`), falling back to the
      server's English for a code the client does not know. Rust: 8
      insights tests pass, clippy clean; the DB suite was not re-run after
      adding `params` (a response-field addition). Vitest 62/62,
      Playwright 19/19.

- [x] WPM-T72 (2026-10-04) **Localized `/admin/`.** *(traces to WPM-R50)* The Sveltia CMS shell is
      now a per-locale route (`src/routes/admin/+server.ts`):
      `/en-001/admin/`, `/cy-001/admin/`, `/en/admin/` (alias redirect);
      an unprefixed `/admin/` redirects like any page. It sets `<html
      lang dir>` and the CMS's own interface language, which Sveltia reads
      from the `sveltia-cms.prefs` localStorage `locale` (it has no config
      option; found in its source) — mapped by `CMS_UI_LOCALE` onto the 29
      languages the CMS ships (bn, cy, hi, id, ur have none, so English /
      British English). `/admin/config.yml` stays static. Replaces
      `static/admin/index.html`. Vitest 63/63, Playwright 20/20 (CMS
      script stubbed); the CMS itself still unexercised.

- [x] WPM-T73 (2026-10-04) **Auth suites run; two Keycloak-backend bugs fixed.** *(traces to WPM-R15)*
      Against the real sibling crates (`~/git/sixarm/main-x-service`) and a
      real Postgres: `tests/enforcement.rs` passes (1/1), and
      `tests/keycloak.rs` (real Keycloak 26 via Testcontainers on Podman)
      passes (1/1) after two fixes in `src/auth/keycloak.rs`: (1) the
      `keycloak` feature did not compile (`attrs_from_keycloak_claims(&claims)`
      borrowed after partial moves); (2) every token was refused
      `InvalidAlgorithm` because `Validation` listed RS256 and ES256
      together — it now validates the header's algorithm if it is on the
      allow-list. Auth lib tests 26/26 under the feature. Clippy (pedantic) is now
      clean on `auth.rs` and `auth/keycloak.rs` in both backends (older
      warnings cleared in a follow-up).

- [x] WPM-T74 (2026-10-05) **Employee directory.** *(traces to WPM-R39, WPM-D29)* From the SOTA scan
      (orangehrm `orangehrmCorporateDirectoryPlugin`). Pure
      `rules/directory.rs` (5 tests): `search` — every whitespace term must
      match name, title, department, location or manager name,
      case-insensitively; optional exact department; stable order by name
      then pid. `GET /api/directory?q=&department=&limit=&offset=`
      (`controllers/directory.rs`): workers *employed today*
      (`is_employed_on`) in the caller's organization scope, paged with
      `x-total-count`; **nothing sensitive** (no salary, dates, person
      ref), and a manager is named only if in the same readable set. UI
      `/directory` (debounced search, department picker, table linking to
      the worker), nav link, `employeeDirectory()` client, 6 strings in 13
      locales (CMS config regenerated). DB suite 44/44 (the new test pins
      the exclusion of a terminated worker and the absence of sensitive
      fields); clippy clean on the new files; svelte-check 0, vitest 63/63,
      Playwright 21/21, build green. The announcement feed half of the
      comparator's capability is not done (`.sota` scores it partial).

- [x] WPM-T75 (2026-10-05) **Emergency contacts and backups.** *(traces to WPM-R40, WPM-R41, WPM-D30, WPM-D31)* Each person
      provides both about themselves (HR can on their behalf, recorded as
      such). Migration `m20261005_000040_emergency_contacts_and_backups`.
      - **Emergency contacts** (`rules/emergency.rs`, 4 tests): name,
        relationship, a dialable phone (5–15 digits), optional alternate
        phone, email and note; ranked (1 = first to call), up to 5. **Third-
        party personal data**: only the worker and whoever may write their
        record (HR) can read or change them — a manager cannot — and audit
        entries name no contact detail. `GET|POST /api/workers/{pid}/
        emergency-contacts`, `PUT|DELETE /api/emergency-contacts/{pid}`.
      - **Backups** (`rules/cover.rs`, 6 tests): the colleague(s) who cover
        when the person is out sick or on leave — ranked, up to 3, optionally
        for a dated window; must be someone else, currently employed, in an
        organization the caller can read, not named twice. Visible to anyone
        who can see the worker. `GET|POST /api/workers/{pid}/backups`,
        `PUT|DELETE /api/backups/{pid}`, and `GET /api/workers/{pid}/cover?on=`
        — the best-ranked backup in window who is employed and **not on
        approved leave**; `covered_by` is null when nobody can (said plainly,
        not guessed).
      - **Privacy:** both are in the subject-access export (and backups
        naming the person); erasure deletes them; both tables join the
        retention sweep list (52). Also removed a duplicated dotted-line
        statement/key in the erasure.
      - **UI:** `EmergencyContacts` and `Backups` panels on `/me` and
        `/workers/{pid}` (the contacts panel renders nothing for anyone who
        may not see it), client functions, 21 strings in 13 locales.
      DB suite 46/46 (two new tests), clippy clean on the new files, lib 260,
      svelte-check 0, vitest 63/63, Playwright 22/22, build green. The
      directory's "covered by" for someone on leave landed as WPM-T76.

- [x] WPM-T76 (2026-10-05) **Directory shows who covers for someone on leave.** *(traces to WPM-R39, WPM-R41, WPM-D31)*
      Builds on WPM-T74/T75. A worker on **approved leave today** is listed as
      `away_today` — never the kind of leave or any reason, which can be health
      data — with `covered_by`: their best-ranked backup who is employed and
      not away themselves (`rules::cover::resolve`), or null when nobody can
      (the UI says "Nobody covering"). Only backups inside the caller's
      readable set are considered, so a cover is always someone the caller
      could find in the directory. UI: an "Away today" chip and "Covered
      by …" under the name; 3 strings in 13 locales. DB suite 46/46 (the
      directory test now covers away, covered, and not-away), clippy clean,
      svelte-check 0, vitest 63/63, Playwright 22/22, build green.

- [x] WPM-T77 (2026-10-05) **On-call rota.** *(traces to WPM-R42, WPM-D31)* Migration
      `m20261005_000041_oncall_rotas` (`rotas`, `rota_members`,
      `rota_overrides`). Not the shift day-view (`GET /api/shifts`, WPM-R6): a
      *rotation* — a named rota in one organization where, every
      `period_days` (1–31), the duty passes to the next member in order,
      counting from `starts_on`. Pure `rules/rota.rs` (8 tests): `assign`,
      `schedule`, `runs`, `load`, `validate_rota`.
      - **Swaps** (overrides): a worker on call for a date window regardless
        of the rotation; a later swap wins on shared days.
      - **Leave-aware:** a scheduled member on approved leave (or no longer
        employed) is **skipped** to the next available member in order and the
        stretch says so (`source`: `rotation` / `skipped` / `override`); if
        nobody can take a day it is unassigned — said plainly, never guessed.
      - **API:** `POST|GET /api/rotas`; `GET|PUT|DELETE /api/rotas/{pid}`
        (schedule as runs over `?from=&to=`, default 28 calendar days, at most 92;
        members in order; swaps; days-on-call per member; who is on call
        today); `GET /api/rotas/{pid}/on-call?on=`; `POST /api/rotas/{pid}/
        overrides`, `DELETE /api/rota-overrides/{pid}`;
        `GET /api/workers/{pid}/on-call` (a person's own stretches). Members
        must be employed workers of the rota's organization; rotas outside the
        caller's organizations do not exist to them; who may change a rota is
        the route-level policy's write gate, as for shifts.
      - **Privacy:** membership and swaps are in the subject-access export and
        removed on erasure; `rotas` joins the retention sweep list (53).
      - **UI:** `/rota` (pick a rota, on call now, the next four weeks as
        stretches with why, days-on-call, members, swaps, create form, retire),
        a "My on-call" panel on `/me`, nav link, client functions, 23 strings
        in 13 locales.
      DB suite 47/47 (rotation, swap, leave-skip, re-order, list, retire, window
      cap), clippy clean on the new files, lib 268, svelte-check 0, vitest
      63/63, Playwright 23/23, build green. **Not done:** swapping *requests*
      between people (a swap is recorded by whoever may write the rota), on-call
      pay or time-off-in-lieu, notifying the person coming on call, and showing
      "on call" in the directory.

- [x] WPM-T78 (2026-10-05) **Directory shows who is on call.** *(traces to WPM-R39, WPM-R42)* Builds on
      WPM-T74/T77. Each directory entry carries `on_call`: the names of the
      on-call rotas the worker is on call for **today**
      (`rotas::on_call_today` — the rotas in the caller's organizations, one
      day each, leave-skipping applied), so a person on call because they
      were skipped in for someone away shows too. The rota name is also
      searchable (`q=platform on-call`). UI: an "On call: <rota>" chip beside
      the name; reuses the existing "On call" string (no new translations).
      DB suite 47/47 (the directory test now covers on-call and not),
      clippy clean, lib 269, svelte-check 0, vitest 63/63, Playwright 23/23,
      build green.

- [x] WPM-T79 (2026-10-05) **Rota notifications.** *(traces to WPM-R42, WPM-D23)* Builds on WPM-T77. Two new
      in-app notification kinds (reference-only, WPM-D23 — the rota's name and
      dates, nothing about anyone's availability or leave): `rota_added` to
      each person *newly named* in a rota's membership (on creation, and when
      an update adds someone — re-ordering or removing tells nobody; pure
      `notify::rota_added_recipients`), and `on_call_swap` to the person a swap
      puts on call ("You are on call for <rota> from <date> to <date>").
      Fixed on the way: reading the old members inside the update transaction
      waited on a second pooled connection and returned 500 — they are read
      before it opens. DB suite 47/47 (the rota test checks both kinds), lib
      270. The *reminder* landed as WPM-T80; the inbox shows these with the
      existing generic body.

- [x] WPM-T80 (2026-10-05) **On-call reminders.** *(traces to WPM-R42)* Builds on WPM-T77/T79. Loco
      task `rota_reminders [days_ahead:0–14, default 1] [as_of:YYYY-MM-DD]`
      (`tasks/rota_reminders.rs`): tells each person whose on-call turn
      **starts** that day — on call then, and *not* the day before (a new
      turn, a swap beginning, a skip-in, or the rota's first day; pure
      `rota::turn_starts`, 3 tests) — "Your on-call turn for <rota> starts on
      <date>" (`on_call_reminder`, reference-only). **Idempotent**: a person
      already reminded for that rota and day is skipped, so a re-run or two
      overlapping schedules send nothing twice; someone carrying on is not
      reminded again. **Schedule it daily** in deployment — it sends nothing
      by itself. DB suite 48/48 (new test: one reminder to the right person,
      none to the other, none on re-run, none mid-turn), lib 272, clippy clean
      on the new files.

- [x] WPM-T81 (2026-10-05) **Rota swap requests.** *(traces to WPM-R42)* Builds on WPM-T77/T79.
      Migration `m20261005_000042_rota_swap_requests`. A person on call asks a
      colleague to take their on-call days in a window; the colleague accepts
      or declines, the requester can cancel while it is open (decided once —
      `rota::swap_can_move`). **On acceptance only the requester's *own*
      on-call stretches inside the window become overrides** for the colleague
      (`rota::stretches_for`) — a swap never moves anyone else's days, so a
      window wider than the requester's turn is fine. The requester must
      actually be on call in the window; the colleague must be an employed
      worker of the rota's organization; a duplicate open request is refused.
      Each side acts for themself (or HR on their behalf): a request is a
      write to the requester's record, a decision a write to the colleague's.
      `POST|GET /api/rotas/{pid}/swap-requests`, `POST /api/rota-swap-requests/
      {pid}/accept|decline|cancel`, `GET /api/workers/{pid}/swap-requests`
      (incoming / outgoing). Notifications `swap_requested` (to the colleague)
      and `swap_decided` (to the other party). Requests are in the
      subject-access export and erased with either person. UI: a request form
      and list on `/rota`; incoming (accept / decline) and outgoing (cancel)
      on the "My on-call" panel; 10 strings in 13 locales. DB suite 49/49 (new
      test: wrong-window and self refused, duplicate refused, accept moves one
      stretch and leaves the taker's own turn alone, decide-once, decline and
      cancel move nothing), clippy clean, lib 275, svelte-check 0, vitest
      63/63, Playwright 23/23, build green.

- [x] WPM-T82 (2026-10-05) **Announcement feed.** *(traces to WPM-R47)* From the SOTA scan (orangehrm
      Buzz). Migration `m20261005_000043_announcements`; pure
      `rules/announcements.rs` (3 tests): validation (title ≤ 200, body ≤ 5000,
      expiry not before publish), `status` (scheduled / live / expired — live
      from the publish day through the expiry day inclusive), `feed_order`
      (pinned first, then newest published, then newest created). **Plain
      text only** — the API stores a body verbatim and the UI shows it with
      line breaks, escaped, so a post can never carry markup. Everyone who can
      read an organization reads its *live* posts; **posting, editing,
      retiring and seeing scheduled or expired posts belong to the
      organization's `hr_admin` / `org_admin`** when auth is on (open when it
      is off, as elsewhere). `POST|GET /api/announcements` (`?organization=`,
      `include=all` for editors, paged with `x-total-count`),
      `GET|PUT|DELETE /api/announcements/{pid}`; `announcements` joins the
      retention sweep list (54). UI: `/announcements` (feed, editors' post form
      and retire, show scheduled and expired), a "Latest announcements" panel
      (3 posts) on the signed-in home, nav link, 16 strings in 13 locales. DB
      suite 50/50 (validation, ordering, hidden scheduled/expired, paging,
      edit, retire), clippy clean, lib 278, svelte-check 0, vitest 63/63,
      Playwright 24/24, build green. The extras
      (read receipts, department audiences, link attachments) landed as WPM-T86.
      **Not done:** comments or reactions, rich text, file uploads.

- [x] WPM-T83 (2026-10-05) **CEO dashboard.** *(traces to WPM-R48, WPM-D36)* `/ceo`: the whole picture on
      **one screen with no scrolling**, sized for an **iPad (9th gen), 2160 ×
      1620 device pixels = 1080 × 810 CSS pixels at 2×** (the first brief said
      iPad Pro 13", 2752 × 2064; corrected to the 9th gen). Everything is in
      `em` off a size that follows the viewport's short side, with a three-row
      grid sharing the height, so the same layout fits 1080 × 810 @2× and a
      literal 2160 × 1620 @1×. Ten tiles from existing views: headcount (with
      change vs a year ago), turnover, open vacancies (+ median time-to-fill),
      succession gaps, a six-month headcount line, skills adequately covered,
      span of control, top insights (rendered in the UI language from code +
      figures), who is on call now per rota, and the latest announcement. A
      figure that cannot be loaded shows "—", never 0; no new endpoint. The
      trend is **month-end headcount computed from hire and termination
      dates** via `/metrics?from=&to=` (no snapshot job needed). Charts follow
      the dataviz method: a single-series 2px line with a direct end label and
      no legend box, hairline grid, a 9px marker with a surface ring,
      hover/touch crosshair + tooltip, a visually-hidden data table, the
      reference palette's light and dark tokens, and status shown as icon +
      label (▲ Attention, ✓ No gaps), never colour alone. Pure
      `lib/ceo.ts` (`trendDates`, `delta` — never a % of zero, `paddedRange`,
      4 tests). Playwright checks both viewports with worst-case long text:
      the page and body do not scroll, every tile is inside the screen, **and
      no tile clips its own content** (tiles are `overflow: hidden`, so the
      page check alone would miss that). 8 strings in 13 locales. svelte-check
      0, vitest 67/67, Playwright 26/26, build green. **Not done:** the
      palette validator was not re-run (the reference palette's own values
      are used unchanged); drill-down from a tile; a time-range control; the
      app's own top bar is not shrunk on this screen.

- [x] WPM-T84 (2026-10-05) **CEO dashboard: ten tiles → six.** *(traces to WPM-R48)* By request. Kept:
      headcount (with change vs a year ago), turnover, open vacancies (+ median
      time-to-fill), succession gaps, the six-month headcount line, and the top
      insights (now up to four). **Dropped** from the screen: skills coverage,
      span of control, who is on call, and the latest announcement — with
      their four fetches (capability analysis, rotas, announcements) and the
      unused `ceo.skills` string; those views are still on their own pages.
      Two rows now (four KPI tiles, then trend + insights), so the figures,
      chart and text are larger; the chart's drawing box is taller so it fills
      its tile instead of letterboxing. Fixed on the way: the hidden data table
      under the chart was styled as the clipping element, but **a table
      ignores `overflow`**, so its rows stretched the page's scroll height by
      243 px — it now sits in a clipping wrapper (the fit test caught it).
      Both fit checks (1080 × 810 @2×, 2160 × 1620 @1×: no page scroll, every
      tile inside the screen, no tile clipping) pass with 5 long insights as
      worst case. svelte-check 0, vitest 67/67, Playwright 26/26, build green.

- [x] WPM-T85 (2026-10-05) **CEO dashboard follow-ups.** *(traces to WPM-R48, WPM-D36)* Builds on WPM-T83/T84.
      - **Drill-down:** every tile is a link to the page behind it (headcount,
        turnover, the trend and the insights → `/metrics`; vacancies →
        `/requisitions`; succession gaps → `/development`), locale-prefixed,
        with a visible focus ring.
      - **Period control** (30 calendar days · 90 calendar days · 12 months · year to date;
        default 12 months, the service's own default window): applies to
        turnover, leavers, median time-to-fill, the change in headcount and
        the insights; the trend (six months) and the current counts
        (vacancies, gaps) do not depend on it. It is in the URL (`?range=90d`)
        so a view can be shared; a stale or edited value falls back to the
        default; a late response for a period the user has clicked past is
        dropped. Pure `lib/ceo.ts` `RANGES`, `parseRange`, `rangeDates`
        (inclusive dates; "12 months" goes back to the same date last year,
        clamped to the month's end, as the service does). 6 strings in 13
        locales (`ceo.vs12` replaced by `ceo.vsStart`).
      - **Palette validator run** (it had been skipped): the series colour
        passes all checks in light (`#2a78d6` on `#fcfcfb`) and dark
        (`#3987e5` on `#1a1a19`) — one series, so the CVD checks are n/a. The
        text colours were checked for WCAG contrast: all ≥ 4.5:1 except the
        light-mode muted grey (3.5:1) used for axis labels and "—", which now
        use the secondary ink (7.7:1).
      - KPI tiles are top-aligned so the four figures share a baseline.
      Fit still holds at 1080 × 810 @2× and 2160 × 1620 @1× with the control
      bar; Playwright also checks the links, that the 12-month control is
      pressed by default, and that choosing 90 calendar days puts `range=90d` in the
      URL and reloads metrics with `from` = 89 calendar days back. svelte-check 0,
      vitest 68/68, Playwright 26/26, build green.

- [x] WPM-T86 (2026-10-05) **Announcement extras.** *(traces to WPM-R47, WPM-D35)* Builds on WPM-T82. Migration
      `m20261005_000044_announcement_extras` (`announcements.department`,
      `announcements.links`, `announcement_reads`).
      - **Department audience:** a post can be aimed at one department; only
        that department (case-insensitive) and the organization's editors see
        it. Enforced when auth is on, from the caller's *own* worker records
        (their `sub` is the id in `person:<id>`); when it is off, or to
        preview, the explicit `?department=` filter shows organization-wide
        posts plus that department's. Edit can re-aim (`department`) or widen it
        (`clear_department`). Pure `rules::announcements::audience_includes`.
      - **Link attachments:** up to three `{label, url}`; **https only** (no
        `http:`, `javascript:`, `data:`, protocol-relative or spaced
        addresses), label ≤ 100, address ≤ 500
        (`rules::announcements::validate_links`). The UI re-checks `https://`
        before rendering and opens links with `rel="noopener noreferrer"`.
        There is no file storage in the service, so this is links, not uploads.
      - **Read receipts:** `POST /api/announcements/{pid}/read` (the person, or
        HR on their behalf; idempotent; only for posts they can see — another
        department's is a 404) and `GET /api/workers/{pid}/announcement-reads`
        (their own). **Editors see only a count** (`read_count` on the feed),
        never who. Reads are in the subject-access export and deleted on
        erasure.
      - **UI:** department chip, link list, an "Unread" chip and "Mark as
        read" button for the reader, the read count for editors, and the
        department and three link fields in the post form; 7 strings in 13
        locales. DB suite 51/51 (links refused, department filter, edit,
        idempotent read, wrong-department read 404, count without identities),
        clippy clean on the changed files, lib 280, svelte-check 0, vitest
        68/68, Playwright 26/26, build green. **Not done:** an audience list
        for the count ("N of M"), per-person reminders for unread posts,
        comments or reactions, rich text, file uploads.

- [x] WPM-T87 (2026-10-06) **Skills gap analysis.** *(traces to WPM-R43, WPM-D32)* Builds on WPM-T20/T41/T51.
      Pure `rules/skill_gap.rs` (5 tests). A *need* has a level, an importance
      (`critical` 3 · `important` 2 · `useful` 1) and a source: **role** (the
      worker's current UK GDAD PCF role profile's requirements), **target**
      (the person's own skill target) or **aspiration** (their private skill
      aspirations). Needs merge per skill at the highest level and strongest
      importance with all sources kept, and are graded `met` / `below` /
      `undeclared`. **Priority = importance weight × levels short.**
      `undeclared` is *unknown*: no shortfall, no priority, ranked after real
      gaps — a prompt to assess, never a gap of some size.
      - `GET /api/workers/{pid}/skill-gaps` — one person, ranked. Aspirations
        are included **only when the caller may write the person's record
        (themself, or HR)** (`includes_aspirations` says so).
      - `GET /api/workforce-intelligence/skill-gaps?department=&limit=` — per
        skill over employed workers in the caller's organizations: needed by,
        met, below, undeclared, levels short, critical-below, score,
        departments; ranked by score then critical-below. **Counts only —
        nobody is named, and aspirations are never included** (tested).
      - UI: a "My skill gaps" panel on `/me` and `/workers/{pid}` (sources
        shown; unknown shown as "Not declared"), `/skill-gaps` (department
        filter, ranked table), nav link, 23 strings in 13 locales. DB suite
        52/52 (merge across sources, ranking, unknown, roll-up, no names, no
        aspirations), clippy clean, lib 285, svelte-check 0, vitest 68/68,
        Playwright 27/27, build green. **Not done:** ESCO-role requirements (only
        the PCF role profile gives requirements); gaps against a *next* role
        (promotion readiness); trends over time. Training time recommendations
        followed as WPM-T88.

- [x] WPM-T88 (2026-10-06) **Training time recommendations.** *(traces to WPM-R44, WPM-D33)* Builds on WPM-T87.
      Migration `m20261006_000045_training_time` (`skills.hours_per_level`,
      `skill_courses`); pure `rules/training.rs` (7 tests).
      - **What it recommends:** for each skill a person is **below**, in
        priority order — catalogue courses they have *not* completed, cheapest
        hours-per-level first until the gap is covered; for any levels the
        courses leave, the skill's own `hours_per_level` (or the service
        default, 30). Each recommendation says what it **rests on**: `courses`,
        `mixed` or `estimate`. Planning estimates, not promises.
      - **A skill the person has not declared gets no hours** — it is listed
        under `assess_first`: unknown needs assessing, not training.
      - **When:** items run one after another at a weekly pace, whole weeks
        each, from a start day (default today); `weekly_hours` 1–40, default 4
        scaled by the person's FTE (half-time → 2, at least 1).
      - `GET /api/workers/{pid}/training-plan?weekly_hours=&start=` (hours,
        weeks, per-item dates, total, finish date, `assess_first`).
        `GET /api/workforce-intelligence/training-demand?department=` — total
        hours, people with gaps, average per person, hours by skill (and how
        many people are on the *estimate* basis) and by department; hours and
        counts only, nobody named, no aspirations.
      - **Catalogue:** `GET|POST /api/skills/{pid}/courses` (ref, title, hours
        1–1000, levels added 1–4; a duplicate is refused),
        `DELETE /api/skill-courses/{pid}`, `PUT /api/skills/{pid}/training-hours`
        (1–500, null = default); `skill_courses` joins the retention sweep
        list (55).
      - **UI:** a "Training plan" section under "My skill gaps" (pace input,
        hours, basis chip, courses, dates, totals, assess-first), on
        `/skill-gaps` the workforce training time and a catalogue editor; 30
        strings in 13 locales. DB suite 53/53 (cheapest-first, mixed/estimate,
        completed course dropped, schedule dates, assess-first, bad pace
        refused, demand without names), clippy clean, lib 292, svelte-check 0,
        vitest 68/68, Playwright 27/27, build green. **Not done:** diminishing
        returns per level, course prerequisites or availability, cost, linking
        to the upstream course catalogue, booking a place, and the
        joiner/leaver work that followed (WPM-T89).

- [x] WPM-T89 (2026-10-06) **People joining and leaving the organisation.** *(traces to WPM-R45, WPM-R46, WPM-D34)*
      Migration `m20261006_000046_movements` (`movements`, `movement_items`,
      `handover_actions`); pure `rules/movements.rs` (5 tests).
      - **Records:** `POST /api/workers/{pid}/movements` opens a *joiner* (start
        day, default the hire date) or *leaver* (last day required; a reason:
        resignation, redundancy, retirement, end of contract, dismissal, other;
        the person must still be employed). At most one open record of each
        kind per person. Written by whoever may write the worker's record
        (the person, or HR).
      - **Dated checklists:** built from a standard template with each item
        dated relative to the effective day — a joiner's (−7 contract and
        right-to-work, −5 equipment, −2 accounts, 0 welcome and buddy, +5
        first-week check-in, +30 review) and a leaver's (−28 notice, −14
        handover plan, −7 knowledge transfer, −3 exit interview, 0 equipment,
        access and reassignment, +5 final pay). Each item is `done`, `skipped`
        (with a reason), `overdue`, `due_today` or `upcoming`; the list shows
        progress and the overdue count. People items go to the manager; any item
        can be assigned, added, reopened. `GET /api/movements?kind=&status=`,
        `GET /api/movements/{pid}`, `…/items`, `movement-items/{pid}/done|skip|
        reopen|assign`, `…/complete|cancel`.
      - **Last-day handover** (leavers): `GET /api/movements/{pid}/handover`
        lists everything they still hold as of the last day — **ownerships**
        (direct reports, dotted-line reports, groups they lead, their on-call
        rota seat, mentorships they give), **bookings** (future shifts, on-call
        swaps, being named as someone's backup), **tasks** (checklist items
        assigned to them on others' records) and **access** (their organization
        roles). `POST …/handover` reassigns or closes one item (a direct report
        *needs* a new manager — the cycle check applies; access can only be
        **revoked**, ended on the last day; the new holder must be an employed
        worker of the same organization); `POST …/handover/all` hands
        everything handable to one successor and revokes access, reporting
        anything that could not move rather than skipping it silently.
      - **Audit trail:** each action is one transaction — the change, a
        `handover_actions` row (kind, thing, from, to, action, note, who, when),
        an audit entry and a `handover_received` notification to the new
        holder; `GET …/handover/actions` lists them oldest first.
      - **Completion:** a record cannot be completed while checklist items are
        open or a leaver still holds anything — the 422 names what is left; a
        completed record takes no more changes.
      - **Privacy:** records and handover actions are in the subject-access
        export; erasure scrubs notes and unassigns the person's tasks but keeps
        the audit rows.
      - **UI:** `/movements` (leavers and joiners with progress and an overdue
        chip, start form) and `/movements/{pid}` (checklist with icon + label
        states, skip with a reason, add items; the handover table with
        per-item reassign / close / revoke, hand-everything, the audit trail,
        complete and cancel); nav link; 57 strings in 13 locales. DB suite
        54/54 (a full leaver journey: dated checklist, validations, twelve held
        items, refusals, one-by-one and bulk handover, trail, completion
        gating), clippy clean, lib 297, svelte-check 0, vitest 68/68,
        Playwright 28/28, build green. **Not done / not tested:** the group-lead handover is
        implemented but **not covered by the DB test** (no group is set up in
        it); requisition and appraisal ownership are not in the inventory (only
        what the model holds today); handover *on the last day automatically* (it is
        done by a person; a scheduled reminder would be a follow-up); a leaver's
        pending leave approvals; returning to work.

- [x] WPM-T90 (2026-10-06) **Whole-repo lint pass.** `cargo fmt`,
      `cargo clippy --all-targets -- -D warnings` (default and `--features
      keycloak`) and `pnpm lint` are clean across the repository; before, only
      new files had been kept clean. Mechanical: formatting (53 files), twelve
      `assert!(….is_empty())` test assertions, a wildcard import in
      `auth/paseto.rs`, and one justified `#[allow(clippy::too_many_lines)]`
      (`controllers/directory.rs`). Full suites re-run afterwards: 297 unit, 54
      request, the enforcement test, 68 vitest, 28 Playwright, build green.

- [x] WPM-T91 (2026-10-06) **Sitemap generator.** *(traces to WPM-R50)* A
      generated `/sitemap.xml` and `/robots.txt` for the front-end. Pure
      `src/lib/sitemap.ts` (unit-tested) + two `+server.ts` routes (no locale
      redirect: files with an extension are exempt). Every **public** page —
      `/`, `/tour`, `/signin` — in every locale (17 × 3 = 51 URLs), each with
      `xhtml:link rel="alternate"` for its sibling locales and `x-default` →
      `en-001`; the `hreflang` of a `<language>-001` locale is its bare language
      (search engines do not accept the UN numeric region `001`), a regional one is
      `en-GB`/`de-DE`/`es-ES`. XML-escaped; the aliases (`/en/…`, redirects) and
      every signed-in page are absent by construction — the public page list is
      **shared with the sign-in gate** (`src/lib/publicPages.ts`) and a test
      pins every indexable page as public. `/robots.txt` names the sitemap and
      disallows `/api/`, `/admin`, `/*/admin`, `/signout`. Optional
      `WPM_PUBLIC_URL` (server-side) overrides the origin behind a proxy.
      vitest 74 (6 new: URL count and uniqueness, the gate pin, hreflang mapping,
      alternates and `x-default`, XML validity and escaping, robots), Playwright
      29 (the new spec fetches both files), svelte-check 0, prettier clean, build
      green; the rendered file was also parsed with a real XML parser (51 URLs,
      18 alternates each). **Not done:** `<lastmod>` (the public pages are static
      content with no modification date); submitting the sitemap to a search
      console; a static `sitemap.xml` for a static host (it is generated per
      request by the server); `noindex` on signed-in pages (they redirect to
      sign-in).

- [x] WPM-T92 (2026-10-06) **Public-sector national pay scale.**
      *(traces to WPM-R51, WPM-D38; research and the full table are in
      [pay-scales.md](pay-scales.md))* Researched against the government's published
      pay letter (Annex 1, read from the circular itself): from
      1 April 2026, +3.3%, **three steps with years-to-progression for bands 5–9,
      two for bands 3–4, a single rate for band 2, band 1 closed**, plus the
      sleeping-in and on-call allowances. This differs from the structure the
      pay-calculator sites show, so the circular alone is the source. Pure
      `rules/pay_scale.rs` (the transcribed scale; `locate`, `progression`; 5 unit
      tests incl. well-formedness of every band, each position, the boundaries and
      the progression months), `controllers/pay_scales.rs` (`GET /api/pay-scales`,
      `/{id}`, `/{id}/position`), 3 OpenAPI entries, 1 request test (the served
      scale, each position, progression, 8 refusals, 404s). UI: client functions
      (path-map test), types, `/pay-scales` (table, allowances, lookup form), nav
      link, strings in all 12 `-001` locales (AI-written, unreviewed). **Stateless
      by design (WPM-D38):** no table, no worker field, so no masking, export or
      erasure wiring. Verified: 302 unit, 55 request, clippy `-D warnings` and
      `fmt` clean, vitest 74, Playwright 30 (twice), svelte-check 0, build,
      prettier. **Not done:** a worker's band and step with a progression date and
      reminder (would store a pay position per person — a separate decision);
      linking roles to bands; England/Scotland/N. Ireland scales; bank-worker and
      hourly rates; applying the on-call allowance to the rota; **the page was not
      looked at in a browser** (only the stubbed Playwright spec).

- [x] WPM-T93 (2026-10-06) **Google technical job levels (L3–L11).**
      *(traces to WPM-R52, WPM-D39; [job-levels.md](job-levels.md))* From the
      summary the maintainer pasted — a **secondary, unofficial** source, said so in
      the payload and the page. Pure `rules/job_levels.rs` (the ladder: code, title,
      summary, typical experience and management equivalent; `level` lookup by
      `L5`/`l5`/`5`, `next_above`; 2 unit tests incl. every level present once,
      codes matching numbers, and the fields the source omits left `None`),
      `controllers/job_levels.rs` (`GET /api/job-levels`, `/{id}`,
      `/{id}/levels/{code}`), 3 OpenAPI entries, 1 request test (the ladder,
      nulls for unstated fields, no pay on any level, top has no next level,
      4 × 404). UI: client functions (path-map test), types, `/job-levels`,
      nav link, strings in the 12 `-001` locales (AI-written, unreviewed).
      **No pay and no stored data (WPM-D39).** Verified: 304 unit, 56 request,
      clippy `-D warnings` and `fmt` clean, vitest 74, Playwright 31 (the first run
      after a build hit the known preview-server race with 3 failures; the re-run
      was 31/31), svelte-check 0, build, prettier. **Not done:** assigning a level to
      a worker or role; mapping role profiles or the UK capability framework to
      levels; other tracks and companies; an official source; the page was not
      looked at in a browser.

- [x] WPM-T94 (2026-10-06) **Assign a job level to a worker or a role; link a level
      to a pay band on a role.** *(traces to WPM-R53, WPM-D40;
      [job-levels.md](job-levels.md))* Builds on WPM-T92–T93. Migration 47
      (`worker_job_levels`; `job_level_framework`/`job_level`/`pay_scale_id`/`pay_band`
      on `role_profiles`, with a CHECK that each pair travels together). Pure
      `rules/grade.rs` (`resolve_level`, `resolve_band`, `validate_effective_on`; 3
      unit tests), `controllers/grades.rs` (`/api/workers/{pid}/job-level`,
      `/api/role-profiles/{pid}/grade`: GET/PUT/DELETE each), OpenAPI. **Worker level:
      person and HR only; the audit entry names no level; exported and erased with the
      person** (a statement appended at the end of the erasure list, `job_levels_deleted`
      added to the audit snapshot). **Role grade:** a level and/or a pay band, validated
      against the reference ladders; PUT replaces; the link is the editor's
      statement — **no level↔band equivalence is shipped or derived**, because no source
      gives one. UI: `JobLevel` panel on `/me` and the worker page (hidden when 403),
      `RoleGrade` panel on `/roles`, client functions (path-map test), strings in the 12
      `-001` locales (AI-written, unreviewed). Verified: 307 unit, 58 request (2 new: the
      worker's level end to end incl. validation, audit without the level, export and
      erasure by a direct row count; a role's level→band link, PUT-replaces, DELETE, 404),
      enforcement test, clippy `-D warnings` and fmt clean, svelte-check 0, vitest 74,
      Playwright 33 (twice; 2 new), build, prettier. **Not done:** level history and
      promotion dates (deliberate, D40); a headcount-by-level roll-up (a count of one
      would name someone); a level or band on a requisition; the page was not looked
      at in a browser; the manager-cannot-read rule is by default policy and was not
      exercised under enforcement.

- [x] WPM-T95 (2026-10-06) **Visual review of the new pages.** *(hygiene; traces to
      WPM-T92–T94)* The pay-scales, job-levels, roles-grade and `/me` job-level views
      had only been checked through stubbed specs. Screenshotted each in light and dark
      at 1080×810, 1080×810 (iPad 9th gen at 2×) and 390×844, against a stubbed API, and
      read them. **Defects found and fixed:** band `8a` rendered `8A` (a global
      upper-casing `th` style); whole-pound pay as `£26,300.00` where the circular prints
      `£26,300` (new `moneyWhole`); form labels run together; the job-levels table
      unreadable on a phone (words broken mid-word; now an `overflow-x` box); the level
      select overflowing the Grade panel on a phone. Measured: no page scrolls sideways in
      any of the 24 page/size/theme combinations. **Also found:** another project's preview
      server on port 4173 made the first screenshots show an unrelated site, because
      Playwright reuses whatever listens there — added `PW_PORT`, and the sitemap spec no
      longer hard-codes the origin. Re-verified: svelte-check 0, vitest 74, Playwright 33
      (on `PW_PORT=4181`), prettier. **Not done / seen but left:** the older `/roles`
      requirement tables break header words on a phone ("SKI LL"); the **dark** theme was
      checked, the other themes were not; the CMS `/admin/` flow still needs a
      GitHub login and was not opened.

- [x] WPM-T96 (2026-10-06) **A worker's pay band and step, eligibility date and reminder.**
      *(traces to WPM-R54, WPM-D41; [pay-scales.md](pay-scales.md))* Builds on WPM-T92.
      Migration 48 (`worker_pay_positions`). Pure `rules/pay_position.rs` (`validate`,
      `validate_step_since`, `eligible_on` = `step_since` + the step's whole years with a leap-day
      clamp, `standing` at_top/due/not_yet with `days_remaining`, `annual_minor`; 5 unit tests),
      `controllers/pay_positions.rs` (`/api/workers/{pid}/pay-position` GET/PUT/DELETE), OpenAPI,
      the loco task `pay_progression_reminders [days_ahead:0–90] [as_of]` (idempotent per
      eligibility date; employed-on-the-date only), notification kind `pay_step_due` (closed list
      updated). **The worker and HR only; audit and reminder name no band, step or amount;
      exported and erased with the person** (statement appended, `pay_positions_deleted` in the
      audit snapshot). UI: `PayPosition` panel on `/me` and the worker page (hidden when 403),
      client functions (path-map test), strings in the 12 `-001` locales (AI-written,
      unreviewed). Also added the missing **hidden-on-403 spec** for the job-level and
      pay-position panels. Verified: 312 unit, 60 request (2 new: validation, dates, top step,
      audit without the position, export, clear/404, erasure by row count; reminders window,
      idempotence, no pay in the message), enforcement test, clippy `-D warnings` and fmt clean,
      svelte-check 0, vitest 74, Playwright 35 (twice, `PW_PORT`), build, prettier; the panel was
      screenshotted on desktop (light) and phone (dark) and read. **Not done:** telling HR or a
      manager (deliberate, D41); a team "who is due" view (a count of one names someone);
      position history; comparing the position with the salary field (deliberate); the
      `pay_progression_reminders` task is **not scheduled** — the operator must run it daily;
      manager-cannot-read was not exercised under enforcement.

- [x] WPM-T101 (2026-10-07) **Locales information brought up to date across `spec/`.** *(docs only;
      traces to WPM-R50, WPM-D37, WPM-D43, WPM-D44)* Audited every `spec/` file that mentions locales
      against the code. **Stale and fixed:** `spec/index.md` said 16 locales (17), `design.md` said
      "13-locale i18n" (17). **Added** to the locales spec: a table of the 17 locales (picker label,
      kind, what each holds — 13 languages at `-001` with all **541** keys, 4 regional of which only
      `en-gb` has overrides, 5, and `de-de`, `en-us`, `es-es` are empty and fall back), a note that
      there is no `cy-gb`, the picker-label rule, and an "Adding a locale" checklist; **glossary** terms
      (content locale, `-001` locale, regional locale, locale address); `architecture.md` (one address,
      bare `/` by browser language); `testing.md` (the directory-name check). Counts were computed from
      `content/locales/`, not copied. **Not changed:** the generic "book side" guidance in the locales spec
      (peer ids, slugs, `locales/<code>/` for a book repo) describes a different repository and was
      left as written.

- [x] WPM-T100 (2026-10-07) **No more `/<language>/` forwarding.** *(traces to WPM-R50, WPM-D44;
      amends WPM-D37; [locales](locales-for-global-sharing-with-svelte/index.md))* The request read
      "remove route forwarding `/<language>-001/` to `/<language>/`", but the app only ever
      forwarded the **other way**: a bare `/en/…` 301-redirected to `/en-001/…` (the alias asked for on
      2026-10-04). **Interpreted as removing that alias forwarding**, in line with the rule that no
      bare language is a directory; a one-commit revert restores it if the opposite was meant.
      Now only a full content code (`/en-001/…`, `/en-gb/…`) is a locale prefix: `splitLocale` has no
      `alias` result, the 301 block is gone from `hooks.server.ts`, and `/cy/workers` is an ordinary
      unprefixed path — sent under the visitor's own locale (`/en-001/cy/workers`, an unknown page)
      like any other. A bare language still **matches a tag** (cookie, `Accept-Language`,
      `navigator.language`): `LOCALE_ALIASES` was renamed `LANGUAGE_LOCALE` so it no longer reads as a
      URL alias. Comments and the locales spec, README, requirements, testing, UI spec and agent guide
      updated. Verified: svelte-check 0, vitest 80 (the split tests now pin `/cy/workers/abc` →
      no locale), the e2e spec replaced by one proving `/cy/workers` is **not** forwarded to
      `/cy-001/` while `/cy-001/workers` still is, prettier, build. **Not done / seen:** old bookmarks
      to `/en/…` now land on a 404 page under the visitor's locale rather than redirecting; the
      machine was under heavy load from other projects while testing, so the Playwright result was
      taken with two workers (see below).

- [x] WPM-T99 (2026-10-07) **Locale directories are `<language>-<region>`.** *(traces to WPM-R50;
      [locales](locales-for-global-sharing-with-svelte/index.md))* Asked to delete two-letter locale
      directories such as `locales/en/`: **there were none** — all 17 directories under
      `content/locales/` were already `<language>-<region>` (`en-001`, `en-gb`, …), and no other
      locale directory exists in the repo (the only two-letter directory is the `/me` route), so
      nothing was deleted. Added the rule to the locales spec (a "Locale directory names" section:
      lower-case ISO 639 language + ISO 3166-1 alpha-2 country *or* UN `001`; a bare language is only a
      URL alias, a tag, or a row in `locales.tsv`) and to `AGENTS/localization.md`, and corrected the
      spec's "locale code priority order" from bare codes (with `sp`, a slip for Spanish) to
      `en-001`, `cy-001`, `zh-001`, `es-001`, `ar-001`. Enforced by 3 vitest tests (every directory
      matches the shape; the directories are exactly the served `LOCALES`; the shape rejects `en`,
      `en_GB`, `EN-GB`, `en-1`, `en-gbr`) — **mutation-checked**: a stray `content/locales/en/`
      fails them. vitest 80. **Not done:** `locales.tsv` still lists bare languages by design (it is a
      language table, not a directory list).

- [x] WPM-T98 (2026-10-07) **A bare `/` follows the browser's language.** *(traces to WPM-R50,
      WPM-D43; [locales](locales-for-global-sharing-with-svelte/index.md))* Reads
      `navigator.languages` (else `navigator.language`) and redirects to the matching locale
      route: pure `localeFromNavigator` in `src/lib/locales.ts` (underscore tags accepted; an
      exact regional locale beats the language's `-001`; the first served tag in preference
      order wins; **`null` when none is served**, so the caller picks the fallback). The
      example in the request, `cy_GB`, gives **`/cy-001/`** — there is no `cy-gb` locale — and
      `en_GB` gives `/en-gb/`, `de-DE` `/de-de/`. Wiring: `hooks.server.ts` no longer redirects
      a bare `/` that has **no remembered-locale cookie** (a remembered choice still wins and
      is redirected server-side, as before) and instead puts a `<noscript>` meta refresh to the
      `Accept-Language` pick in the page; `routes/+layout.server.ts` passes that pick as
      `fallbackLocale`; `routes/+layout.ts` redirects (307) before the page renders. **Every other
      unprefixed path keeps the server rule** (`/signin` → `Accept-Language`) — a deliberate
      boundary. Verified: vitest 77 (3 new: regional vs language match, preference order, `null`
      for unserved), Playwright 45 (8 new: four browser languages incl. `cy-GB`, the unserved
      fallback, remembered cookie beating the browser, `/signin` unchanged, the no-JS refresh, and
      **a spec that makes `navigator` and `Accept-Language` disagree** — mutation-checked: with the
      client redirect reduced to the server's pick it fails, landing on `/fr-001`), svelte-check 0,
      prettier, build. **Not done:** other unprefixed paths do not consult the browser (a deep link
      such as `/workers` still uses `Accept-Language`); `navigator.language` is not re-read after the
      first visit (the cookie then holds the choice, so a later change of browser language is
      ignored until the cookie expires or the picker is used); no `cy-gb` content locale exists.

- [x] WPM-T97 (2026-10-07) **Employee expense claims.** *(traces to WPM-R55, WPM-D42;
      [expense-claims.md](expense-claims.md))* Delivers the one table-stakes gap from the SOTA
      scan (`frappe/hrms` `expense_claim`, `orangehrm` `orangehrmClaimPlugin`, `odoo`
      `hr_expense`), deferred 2026-10-05. Migration 49 (`expense_claims`, `expense_items`).
      Pure `rules/expenses.rs` (the lifecycle, item validation, checked totals, duplicate flags,
      and `may_view`/`may_edit`/`may_decide`; 5 unit tests), `controllers/expenses.rs` (12 routes:
      claims, items, submit/withdraw/cancel, approve/reject/reimburse, and the decision queue),
      OpenAPI, notification kinds `expense_submitted` and `expense_decided` (reference-only).
      **The claimant never decides their own claim, even as HR; the queue omits their own;
      a stranger gets a 404.** Audit entries and notifications carry no amount, title or
      description. Privacy: `expense_claims` on the retention list (56 tables); in the
      subject-access export; **erasure cancels draft/submitted claims, keeps the amounts of
      approved/reimbursed ones, and scrubs every free-text field.** UI: `ExpenseClaimView`
      (actions from the service's `can` flags), `ExpenseClaims` panel on `/me` and the worker
      page (hidden on 403), `/expenses` decision queue, nav link, client functions (path-map
      test), strings in the 12 `-001` locales (AI-written, unreviewed). Verified: 317 unit; 62
      request (2 new: draft → submitted → approved → reimbursed with validation, duplicates
      flagged, notifications and audit free of amounts, a rejection path and a cancelled draft;
      export, and erasure checked by row); a **new enforcement binary**
      (`tests/enforcement_expenses.rs`) for who may see, write and decide — **mutation-checked:
      with the not-the-claimant rule removed it fails at "HR cannot approve their own claim"**;
      the original enforcement test; clippy `-D warnings` and fmt clean; svelte-check 0, vitest
      74, Playwright 37 (twice, `PW_PORT`), build, prettier. The screens were screenshotted (light
      and dark, desktop and phone) and read: moved Amount beside Date so the key figure shows on
      a phone, separated the new-claim form, stopped words wrapping mid-word. **Found by the
      specs:** the queue stayed "open" on a claim after the filter changed (now reset), and
      "Add item" cleared fields after the request returned, wiping what was typed for the next
      item (now cleared at submit, restored on failure). **Not done:** payroll integration
      (reimbursed is a date, not a payslip line); receipts as files; mileage and per-diem rates;
      category limits or budgets; multi-level approval or delegation; a team spend view;
      the manager path under the real `hr_admin`-less default policy was tested with an open
      blanket policy, so the **deployment's own policy** still decides who may reach these
      routes at all.

- [x] WPM-T102 (2026-10-09) **The pay scale is generic.** *(traces to WPM-R51, WPM-R54,
      WPM-D38; [pay-scales.md](pay-scales.md))* Builds on WPM-T92 and WPM-T96. The pay
      scale no longer names a specific health service, government or nation. The scale is
      `rules::pay_scale::national_2026_27`: id `national-2026-27`, name "Public-sector pay
      scale 2026/27", framework `banded-pay`, nation `national`, source "Government pay
      circular for 2026/27, Annex 1", and the on-call allowances lose the place name.
      **Figures, bands, steps, allowances and behaviour are unchanged.** Migration 50
      (`m20261009_000050_generic_pay_scale_id`) moves every stored scale id in
      `worker_pay_positions` and `role_profiles` to the new id. That is safe because the
      service knew only one scale and validated every stored id against it. Its `down` is a
      no-op. The spec, glossary, index, requirements, testing notes, `llms.txt`/`llms.json`,
      NEWS and the three changelogs are reworded to match, and the earlier WPM-T92 and
      WPM-T95 entries are redacted. Verified in a scratch workspace with the real sibling
      crates: 317 unit tests; 62 request tests against PostgreSQL 18 (Podman), including
      pay scales, pay positions and grades with the new id, and migration 50 applied; clippy
      `--all-targets -D warnings` and fmt clean; svelte-check 0, vitest 80, Playwright 45
      (`PW_PORT=4187`); prettier clean on `src`. The migration's update SQL was run by hand
      on a real schema and moved an old-id pay position to the new id, keeping its band and
      step. **Not tested:** migration 50 on a database that already holds old-id rows (the
      suite's database starts empty), and the `role_profiles` update with a matching row.
      The enforcement and Keycloak suites were not run (no auth code changed).
      **Trade-off:** the source no longer names its circular, so a reader cannot trace the
      figures to a document from the spec alone.

- [x] WPM-T119 (2026-10-09) **Every duration in days says "calendar days".** *(traces to
      [calendar-days-or-business-days](calendar-days-or-business-days/index.md))* Every count
      of days in the service is a difference of dates, including leave spans and leave
      balances, so every one is **calendar days**; none is business days. Relabelled:
      - the spec, READMEs, INSTALL and changelogs;
      - API messages: the leave span cap, the leave balance refusal, the rota and swap window
        caps, the time-to-fill insight and its threshold note, the metrics derivation, two
        OpenAPI summaries, the pay reminder task's help and error, and the "30-calendar-day
        review held" joiner checklist item;
      - UI strings in the 13 `-001` locales (`pr.horizon`, `common.days`, `metrics.days`,
        the time-to-fill insight, the CEO period buttons, the pay step eligibility line),
        AI-written and unreviewed;
      - code comments that state a duration.

      Verified: 317 unit tests; 62 request tests (PostgreSQL 18, Podman); clippy
      `--all-targets -D warnings` and fmt clean; svelte-check 0, vitest 80, Playwright 45
      (`PW_PORT=4188`), including the CEO dashboard fit-on-one-screen specs with the longer
      period buttons; prettier clean on `src`. **Not done:** no screenshot was taken of the
      longer labels (the fit specs passed, but nobody looked at the screens); the
      translations are not reviewed by native speakers; API field names such as
      `horizon_days`, `days_ahead` and `days` are unchanged (the rule covers new identifiers
      only). **Finding:** leave entitlements and balances count calendar days, weekends
      included, where employers usually count working days. That is now visible in every
      label. Counting business days would be a behaviour change and is not made here.

## Phase 12 — delivery capacity and oversight readiness (proposed WPM-R56–R70, WPM-D45–D50)

Proposed 2026-10-09, not started. The plan, the proposed requirements and the
decisions are in [plan.md](plan.md); they move into a topic file at WPM-T103.
Generic by design: levels, criteria and measures are configuration, and no real
organization, government, board or oversight framework appears in code, seed
data or tests (WPM-D48).

- [ ] WPM-T103 **Spec round.** *(traces to WPM-R56–R70, WPM-D45–D50)* Move the
      requirements and decisions from [plan.md](plan.md) into
      `delivery-capacity-and-oversight.md`; settle the three open questions in
      plan.md section 6; update [domain-model.md](domain-model.md) (the tables in
      plan.md section 4), [auth.md](auth.md) (the HR-only key-person list, the
      `post_fundings` reads, a read-only partner role), [audit.md](audit.md),
      [regulatory.md](regulatory.md) (`post_fundings`), [glossary.md](glossary.md)
      (skill pool, operations reservation, constraint pool, single point of
      failure, contingent share, funding horizon, oversight level, exit
      criterion, sustain rule), [index.md](index.md), `llms.txt`, and the next
      free ids in `AGENTS/spec-driven-delivery.md`. No code.
- [x] WPM-T104 (2026-10-11) **Pure core: pool supply and capacity.** *(traces to WPM-R56, WPM-R59, WPM-D45, WPM-D46)*
      `rules/capacity.rs`: pool membership by role profile or skill at a minimum; supply as FTE weighted by
      the days employed and not on approved leave, less the operations reservation, in whole hundredths of an
      FTE; the pool × month grid with partner rows; four results (`fits`, `fits_with_partners`, `over`,
      `unknown`); the constraint pool. An unanswered partner makes a short month `unknown`, never `over`.
      Verified by unit tests at each boundary (a pool with no members, an exactly-met demand and one
      hundredth over, a hire part-way through a month, overlapping leave ranges, a partner silent in one
      month, a tie for the constraint pool). Described in [delivery-capacity.md](delivery-capacity.md).
- [x] WPM-T105 (2026-10-11) **Pure core: start check and work-in-progress limit.** *(traces to WPM-R60, WPM-D50)*
      In the same module: failing lines and by how much, the earliest delay in months at which every line is
      known to fit, and the active-programme limit on the constraint pool (the candidate counted once). A
      delay that pushes a line outside the horizon, or into a month that cannot be said, is not a fit.
      Unit tests include a programme that can never fit and a shift past the horizon. **Limit:** proposed
      claims of other programmes are not counted against the one being checked.
- [x] WPM-T106 (2026-10-11) **Skill pools, programme demand, partner commitments.** *(traces to WPM-R56–R58)*
      Migration `000052` (six tables: pools, members, demand, commitments, the work-in-progress limit, start
      decisions; none holds personal data), entities, `controllers/capacity.rs`, nineteen operations in the
      OpenAPI document. Programme by `thing:` URN, partner by `organization:` URN, a first-of-month, a bounded
      positive FTE; a claim can be set `proposed` or `closed` but only becomes `active` through a start
      decision; every write audited; pools, members, demand and commitments soft-deleted. Request tests for
      validation, wrong-kind URNs, duplicates, upsert and soft delete. **Not as the task said:** "planner
      role" is not enforced distinctly, because the policy has no such role; writes are HR writes.
      **Decision to confirm:** a programme is a `thing:` URN because the entity-reference crate has no
      programme type.
- [x] WPM-T107 (2026-10-11) **Capacity view and start check endpoints.** *(traces to WPM-R59, WPM-R60)*
      `GET /api/capacity` (pool × month, horizon, partners shown as unknown where silent, constraint pool,
      derivation text), `POST /api/capacity/start-check` (stores nothing; optional what-if lines) and
      `POST /api/capacity/start-decisions` (proceed or defer, a required reason, the check re-run on the
      server, proceed makes the claims active; the audit entry omits the reason). Verified by a request test
      on a synthetic seed: three full-time and one half-time worker, one on approved leave for a month,
      another with only a *requested* leave (not counted), one pool over-committed in one month, a partner
      silent in another, a failing check with the earliest fit, the limit exceeded, and a deferral then a
      proceed. **Mutation-checked** twice (treating an unanswered partner as zero, and counting unapproved
      leave): each fails the test. Full suites: 353 unit and 66 request tests, clippy and fmt clean.
      **Not verified:** any user interface (none exists), enforcement tests for these routes with sign-in on,
      and behaviour against a large organization (the view loads an organization's workers in memory).
- [x] WPM-T108 (2026-10-11) **Pure core and endpoint: key-person risk.** *(WPM-R61, WPM-D47)* Pure
      `rules::key_person` (`assess`, `bar`, four statuses) and `controllers/key_person.rs`. For each skill marked `critical` on a
      live role profile: **holders** are employed people in the caller's scope whose declared proficiency meets the **highest**
      critical minimum for that skill; **backups** are distinct, current (not ended) named backups who also hold it;
      **ready successors** are distinct people ranked `ready_now` on a live succession plan whose incumbent is a holder. One holder
      with neither is a `single_point_of_failure`; no holder is `no_holder` (a gap, not a single point); two or more is `shared`.
      `GET /api/workforce-intelligence/key-person` (any signed-in caller, counts only, per skill and per department) and
      `GET /api/workforce-intelligence/key-person/named` (the sole holder of each single point of failure by worker number; HR only;
      every read audited). Unit tests (six), a request test on a seeded team (a backup below the bar does not cover, a ready-in-two-years
      successor does not cover, a ready-now one does, a second holder shares it) and enforcement assertions (manager and neighbour read the
      counts and get 403 on the named list; the worker gets 403; HR reads it). **Mutation-checked** (successors ignored; backups ignored;
      bar the lowest; shared read as a single point; unqualified backup counted; any readiness counted; named route open), each failing a
      test. **Not done / limits:** no floor is applied to the counts, because one holder is the thing the register exists to show, so a
      department with one holder reveals that someone holds it (no name; the department can be small); "pool" is the department until
      plans define pools; skill levels are the declared ones, not assessed; no user interface; the successor need not hold the skill.
- [x] WPM-T109 (2026-10-11) **Contingent share and funding horizon.** *(WPM-R62)* The post funding table (WPM-T161), and
      `GET /api/workforce-intelligence/contingent`: contractor and fixed-term share of FTE, each apart, the twelve-month funding
      horizon (people and FTE per month, plus funding already ended), and the **trend** of the share from the snapshots split by
      basis (WPM-T139): `ok`, or `insufficient_history` with no points when fewer than three usable dates span under 60 calendar
      days (WPM-D27). A date counts only if every row on it has the split (an older row is unknown, never a zero) and at least 10
      people were employed. HR only and audited. The floor is the equality-monitoring default of 10; the department breakdown shows
      only when every department reaches it, a month of one to nine people is withheld, and no total is given. Export and erasure
      with the worker were done in WPM-T161. **Retention:** `post_fundings` is not soft-deleted (clearing it, or erasing the worker,
      deletes the row), so there is no deleted state for the sweep to hold; the retention schedule says so, and says the snapshot
      counts are kept. Unit tests for the share, the horizon and the trend, request tests, and an HR-only assertion in the
      conversion enforcement binary; **mutation-checked** (floor ignored for months, for departments and for the trend; route open
      to all; unsplit read as zero; span ignored; fixed-term counted as contractor), each failing a test. **Not done:** the "pool"
      is the department until plans define pools; a user interface (WPM-T143). The split is not added to the per-department rows of
      `headcount-history`, because a department of a few people would reveal who is contingent.
      Original scope: Migration
      for `post_fundings` (kind, end date; no money). Contingent share per pool
      from `employment_type` `contractor` and `fixed_term` (shown separately), with
      trend from headcount snapshots split by basis (WPM-T139)
      (`insufficient_history` when too short, as WPM-D27). Funding horizon per
      pool per month. Retention list, subject access export and erasure wiring,
      each checked by a request test.
- [x] WPM-T110 (2026-10-11) **Conversion lever in workforce plans.** *(WPM-R63)* The forecast offers
      `convert_contingent` (after `hire`) for a department with a shortfall **and** at least one contractor or fixed-term
      worker employed today; with no contract FTE, or no shortfall, it is not offered. The evidence is `contract_fte_now` on
      each department row: headcount and FTE, contractors and fixed-term staff apart (`rules::planning::contract_fte`). Unit
      tests (counts per basis, the lever offered only with a shortfall and contract FTE), an assertion in the request test of
      plans by basis, and a Playwright assertion that the lever reads in words; **mutation-checked** (offered always; fixed-term
      counted as another basis), each failing a test. The lever's label is in the 13 locales. **Not done:** the funding horizon
      the requirement lists as evidence is not shown in the plan, because it is HR-only and audited (WPM-T109), so the plan
      points to the contingent view instead; the lever does not yet read open conversion plans (WPM-T164); `contract_fte_now`
      is not drawn in the UI; the 12 non-English labels are machine-written and unreviewed.
- [x] WPM-T111 (2026-10-11) **Governance load register.** *(WPM-R64, WPM-D45)* Migration `000067`
      (`governance_commitments`: organization, name, role, senior, kind, frequency, minutes, holders). Pure
      `rules::governance`: occurrences a year per frequency, person-minutes = occurrences × minutes × people, hours a month in
      tenths rounded to nearest, **summed before rounding**; a role is senior if any of its commitments says so. Routes:
      `GET/POST /api/governance-commitments?organization=`, `PUT/DELETE /api/governance-commitments/{pid}`,
      `GET /api/governance-commitments/load?organization=` (hours per role, heaviest first, and the senior and all-roles totals,
      with a `basis` sentence saying it is an estimate). Organization-scoped (out of scope is `404`), every write audited, bad kind,
      frequency, minutes, holders, blank role or a non-organization URN is `422`. No per-person rows: the role is a text label and
      `holders` is a count. Unit tests (eight) and a request test for the monthly totals, refusals, change, removal and scoping;
      **mutation-checked** (senior filter ignored; people not multiplied; seniority decided by the last entry; rounding down;
      weekly count; zero people allowed), each failing a test. **Not done:** a user interface; the role label is free text, so
      nothing stops someone typing a person's name there (the guide says not to, and a role is not checked against the job
      titles); who may write is the organization scope only, not a named role; no enforcement test that a partner role is
      refused, because reads are open to any caller in scope; the hours are entered estimates and nothing checks them against
      calendars (WPM-D45: no timesheets).
- [x] WPM-T112 (2026-10-12) **Pure core: oversight criteria and the sustain rule.**
      *(WPM-R65, WPM-R66, WPM-D48, WPM-D49)* `rules::oversight`: `meets` (the threshold itself meets it, `at_most` or `at_least`),
      `evaluate_measure` (looks back from the latest period one period at a time and counts the run), `evaluate_criterion` (met only
      when every linked measure is; not met when any is definitely not; otherwise unknown; no measure is unknown, never vacuously met),
      `validate_spec` (direction, sustain 1 to 120 periods) and `validate_reported` (an evidence note and a source document).
      The answer is `met`, `not_met` or `unknown`, with the run, the periods counted, how many rest on a reported reading rather than
      a derived one, and the reason the run stopped (`sustained`, a breach, no reading or a `null` reading, with the period). A `null`
      or missing reading is never a pass and never a breach; it makes the answer unknown unless a breach is found first. Nothing in
      the module can output a level change (WPM-D49). 13 unit tests (threshold edges both ways, runs, an old breach outside the window,
      null, a missing period, a breach after a gap, too little history, mixed derived and reported, criteria); **mutation-checked**
      (either edge made exclusive; null skipped; breach read as unknown; a definite breach outweighed by unknown; an empty criterion
      read as met; reported not counted; sustain off by one), each failing a test. Skipping a missing period makes the loop run forever,
      which a test run would show as a hang, not a failure. **Not done:** storing anything (WPM-T113), the measures (WPM-T114); periods
      are whole numbers the caller numbers consecutively, so calendar months and quarters are the caller's mapping, and a measure whose
      period is not monthly is not yet defined.
- [x] WPM-T113 (2026-10-12) **Oversight levels, positions, criteria, measures.** *(WPM-R65,
      WPM-R66)* Migration `000068` (five tables), `controllers/oversight.rs`, nine operations in the OpenAPI document. An organization
      adds its own levels (ordinal 1 to 20, a name; nothing is pre-loaded), **records** a position (level, effective date, deciding body
      as free text, published reason; all required), adds exit criteria to a level, measures to a criterion (threshold, direction
      `at_most` or `at_least`, a sustain rule of 1 to 120 months) and enters monthly readings. A value needs an evidence note and a
      source document; `null` says the data is missing. `GET /api/oversight/status?organization=` returns the scale, the positions
      newest first, and for the criteria of the **current** level the verdict of `rules::oversight` as of the last complete month, with
      the run, the required length, why the run stopped (with the month) and how many months rest on a reported reading. It never
      changes a level. Scoped to the organization (out of scope is `404`); every write audited; removing a level with a position or a
      criterion is refused. The seed task adds a three-level scale, one position, one criterion and one measure, every name starting
      "Synthetic" and flagged `synthetic`. One request test (empty start, scale, refusals, position, criterion and measure, a short
      run, a null month, a breach, a threshold-equal value, no level movement, removal); **mutation-checked** (null read as zero; the
      current month counted; evidence not required; reading not aligned to the month), each failing the test. Full suites: 535 unit
      and 90 request tests. **Not done / limits:** only `reported` measures can be created (a derived measure would have no readings
      until WPM-T114); the criteria shown are those of the level the organization is at, which this task takes to be the exit criteria
      of that level; no enforcement test with sign-in on, so the organization scope on these routes is checked by the shared helper
      and not by a test of its own; writes are open to any caller in the organization's scope (the policy has no oversight role);
      no user interface; nothing yet marks a criterion for the level above.
- [x] WPM-T114 **Derived workforce measures.** *(WPM-R67)* A measure is now `derived` or `reported`. A derived measure names a
      computation (`derivation`, paired with `origin` by a CHECK, migration `000069`) and takes its unit; it takes no entered reading
      (a `PUT` is refused). The status computes one reading per month, only for the `sustain` months the rule needs, and shows them as
      `derived_values`. Three derivations, pure in `rules::derived`: `turnover_rate` (the metrics layer's rate for one calendar
      month, basis points), `contingent_share` (contractor plus fixed-term FTE over all FTE, from the last snapshot in the month) and
      `overcommitted_pools` (pools `over` that month, from the capacity view). Each is `null`, never `0`, when it cannot speak (no
      one to divide by, below the floor of 10, a snapshot row from before the split by basis, no snapshot, no pool, a partner who has
      not answered); a `null` stops the run, so it is never a pass. `GET /api/oversight/derivations` returns each definition. The
      candidate measures (sickness absence, the pulse, required training) are defined in
      [workforce-measures.md](workforce-measures.md) and not built. Unit tests per derivation (rounding, floor, missing split,
      unknown cell) and one request test (the catalogue, four refusals, the values shown for each measure, a null month, a boundary
      value meeting an at-most, no pool being unknown, an over pool, nothing stored); **mutation-checked** (the earliest snapshot used
      instead of the newest; the closing headcount taken at the start of the month; a missing split read as zero), each failing a test.
      Full suites: 541 unit and 91 request tests, enforcement, clippy, fmt. **Not done / limits:** single points of failure and
      governance hours are not derived, because neither is dated, so a past month cannot be computed (they join when the pack freezes
      them, WPM-T116); the three candidates are definitions only, and required training is blocked on a data change; the capacity
      figure for a past month is computed from the data on file now and moves if that data is corrected; the turnover floor is applied
      to the larger of the two headcounts, which is a choice the metrics layer does not make; nothing checked against real data; the
      seed adds no derived measure; no user interface.
- [x] WPM-T115 **Early warning and time to report.** *(WPM-R68, WPM-D82)* Migration `000070` adds `early_warnings` (kind, the pool or
      measure, `first_true_on`, `cleared_on`, `acknowledged_on`; one open row per kind and subject, enforced by a partial unique
      index). A sweep (`POST /api/early-warnings/sweep?organization=`, and the daily task `early_warning_sweep`) finds the
      conditions true now (a pool short in any of the next six months; a measure of the current level in breach for the last
      complete month), opens a warning for each new one, closes those that stopped, and tells the organization's HR administrators
      with a new reference-only notification kind `early_warning` (the name of the pool or measure, no figure). `GET
      /api/early-warnings?organization=` lists them with days open, days to acknowledge and the time to report; `POST
      /api/early-warnings/{pid}/acknowledge` is audited and works once. Time to report is the longest calendar days to
      acknowledgement, null while any warning is unacknowledged, and is a derived oversight measure (`time_to_report`) per month.
      Pure rules in `rules::early_warning` (reconcile, days, time to report) with unit tests; one request test (two warnings open,
      HR told twice, a second sweep changes nothing, no figure in the text, unknown until all acknowledged, the longest not the
      quickest, a cleared condition, a returning one as a new warning, the derived measure); **mutation-checked** (shortest instead
      of longest; unacknowledged skipped; breach read in the wrong month; never cleared), each failing the test. Full suites: 546
      unit and 92 request tests, enforcement, clippy, fmt. **Not done / limits:** the recipient is the HR administrators, not an
      owner, because pools and measures have none (WPM-D82); the pool threshold is fixed at any shortfall in six months, with no
      per-pool setting; "first true" is the first sweep that saw it, so it is as late as the sweep is; no scheduling of the task is
      configured here; an unanswered partner makes a pool-month unknown, not over, so it does not warn; no email, and no user
      interface; the sweep is open to any caller in the organization's scope.
- [x] WPM-T116 **Monthly pack and partner view.** *(WPM-R69, WPM-D83)* Migration `000071` adds `oversight_packs` (organization, as-of
      date, month, a JSON payload), append-only by a trigger. `POST /api/oversight/packs` freezes the oversight status (position,
      criteria, measures, verdicts; the same function the status route uses) and the capacity view (pools by name, month cells,
      shortfall, constraint pool; no member, partner or programme) as of today; `GET /api/oversight/packs?organization=` lists
      them newest first without payloads; `GET /api/oversight/packs/{pid}` returns the stored payload, the same for every audience.
      A new membership role `partner`: a caller whose live memberships are all `partner` may read packs in their scope and nothing
      else, and may write nothing (the need-to-know middleware; pure rules `partner_may` and `is_partner_only`). Unit tests for the
      two rules; a request test (frozen pack unchanged by later data, a later pack shows the change, newest first, no worker in the
      payload); an **enforcement test with sign-in on** (a partner and a member read the identical payload; the payload carries no
      worker number, display name, `worker:` or `person:`; a partner is refused the live status, capacity, workers, pools, early
      warnings, the derivation list and every write; a partner who is also a member is not restricted; a partner of another
      organization gets `404`; direct SQL update and delete of a pack fail); **mutation-checked** (a partner may read anything; any
      partner role restricts; pack read ignoring scope; the guard switched off), each failing the enforcement test. Full suites: 548
      unit and 93 request tests, every enforcement binary, clippy, fmt. **Follow-up pass (same task), after review:** the pack now also carries the key-person counts (skills across the organization, no department row, no person) and the governance totals (no role); the daily-or-monthly task `oversight_pack_freeze` freezes the month for each organization with a position and skips one that already has it; a mistaken pack is withdrawn (`POST /api/oversight/packs/{pid}/withdraw`, migration `000072`, a separate append-only row with the date and reason, shown beside the unchanged payload to every reader); a partner can also read `/api/me/organizations/scope` to find their organizations; the pack leaves out the free-text deciding body of a position (the status route keeps it); freezing and withdrawing are for an administrator (`hr_admin` or `org_admin` of the organization, or a token with HR, payroll, service or administrator rights), refused `403` otherwise. Tests added: the scheduled freeze is idempotent for a month and the next month is frozen; the enforcement test now has an ordinary member and a partner-who-is-a-member refused the freeze, an administrator allowed by role, the withdrawal (once, with a reason, unchanged payload, same mark for partner and member), the key-person and governance figures as numbers with no roles, and the deciding body absent; **mutation-checked** again (deciding body kept in the current position; kept in the list; anyone may freeze; withdraw twice; freeze twice), each failing a test. Full suites: 548 unit, 94 request, every enforcement binary. **Not done / limits:** no user interface (the pack page is part of WPM-T118); the pack's key-person and governance figures are not yet derived as oversight measures month by month (a pack freezes them, a later task can read them back); nothing schedules the task here, it is run by the deployment's scheduler; a withdrawn pack still reads (with the mark), because the record of what was shown stays; the role gate matters only where the policy lets a non-administrator write, since the shipped policy already limits writes to HR, payroll, service and administrator tokens; a partner cannot read any other `/api/me` route.
- [x] WPM-T117 **Cutover readiness.** *(WPM-R70)* Migration `000073` adds `cutovers` (programme, site, window of at most 92 calendar
      days, optional rota), `cutover_roles` (role profile, the course that trains it, headcount) and `cutover_pools`. `POST
      /api/cutovers`, `POST /api/cutovers/{pid}/roles` and `/pools`, `DELETE /api/cutovers/{pid}`, `GET /api/cutovers?organization=`,
      and `GET /api/cutovers/{pid}/readiness`. Pure rules in `rules::cutover` (trained staff, hypercare, key-person risk, overall,
      window) with unit tests; the controller gathers trained people at the site (completed the course on or before the window's
      first day, certificate lasting through the window), the uncovered days of the rota (`rotas::compute`, so leave and swaps count)
      and the key-person status of the skills in the listed pools. Each check is `ready`, `at_risk` or `unknown` with reasons; the
      overall result is the worst, and an unknown is never ready. A shortfall that people with no recorded site could close is
      unknown, not at risk. The answer names no one and says the go/no-go is a human decision. One request test (refusals; every
      check unknown then ready; an unplaced person making it unknown; a known shortfall; a late completion, a lapsing certificate, a
      trained person at another site and a trained person in another role not counted; one holder then two; a skill outside the
      listed pools not counted; a rota that starts late; no rota; the list and removal); **mutation-checked** (completion date
      ignored; completion judged at the window's end; expiry ignored; everyone counted as in the role; skills of other pools
      included; site ignored; covered days read as uncovered), each failing the test. Topic file: [cutover-readiness.md](cutover-readiness.md).
      Full suites at first build: 558 unit, 95 request, every enforcement binary, clippy, fmt.
      **Follow-up pass (same task), after review:** key-person risk is now read **at the site** (only people recorded there count as
      holders or backups; a holder elsewhere in the organization does not cover; holders with no site on file make a site risk
      unknown, not at risk); a site is matched by its name and its **aliases** (`site_aliases`, up to ten, migration `000074`),
      without case, punctuation or spacing (`St. Mary's  Hospital` equals `st marys hospital`); the training link is checked: a role
      refuses a course the learning records do not know (no catalogue entry, no enrollment), and a course that later disappears from
      them makes the check `unknown` with the reason, not a silent zero; planning (create, add role, add pool, remove) is for an
      administrator of the organization, as for the monthly pack, and reading stays open to the organization's scope; a cutover whose
      window has not ended and whose people side is `at_risk` opens a `cutover_at_risk` early warning in the sweep (WPM-T115) and
      tells the HR administrators, and closes it when the risk passes. New enforcement binary `enforcement_cutover` (a member the
      policy lets write is refused; a partner refused; an administrator of another organization gets `404`; an administrator of the
      organization plans, adds roles and pools and removes; the readiness and list are read by the organization's member, not by
      an outsider, a partner or an anonymous caller). **Mutation-checked** again (key-person ignoring the site; backups from other
      sites counting; a plain lower-case site match; a renamed course not noticed; an unknown course accepted; the wrong cutover
      condition; a role added by a member), each failing a test (one survivor, backups from other sites, led to a stronger test).
      Full suites: 561 unit, 95 request, every enforcement binary. **Not done / limits:** a site spelt a way neither the name nor an
      alias covers is a different site (aliases are the fix, and nothing suggests them); the course link is still the course
      reference text, checked against the learning records when the role is added and again when read, so a renamed course makes
      the check unknown until the role is re-pointed (there is no endpoint to change a role's course: remove the cutover or the role
      and add it again, and roles cannot be removed singly); nothing checks that the sweep runs, as for every early warning; no
      user interface: see WPM-T118 (reads only; planning a cutover is by API).
- [x] WPM-T118 **UI and verification.** *(WPM-R56–R70)* Four pages and a client. `/capacity`: the heatmap of skill pools by month
      (component `CapacityHeatmap`, also used in a frozen pack): each cell gives supply / demand in FTE, and its result as a **mark and
      a word as well as a colour**; an `unknown` month is dashed and hatched with "?", never a zero and never a pass; the constraint
      pool and the programmes drawing on it (with no limit set said so). `/resilience`: the critical skills with holders, backups and
      ready successors, the number of single points of failure, and the governance load (senior hours a month by role); counts and
      hours only. `/oversight`: the current level, the exit criteria with each measure's run ("2 of 3 months"), why a run stopped
      (breach, no reading, a missing reading), and the monthly packs: freeze, open, withdraw with a reason; a refusal for want of
      the administrator role is said in words; a withdrawn pack shows its mark in the list and open; the deciding body is not in a
      pack. `/cutover`: each cutover with the three checks and the worst, each with a mark and word, and the reasons. Shared
      `OrgPicker` (the caller's organizations, or a typed reference with none) and `stateLabel` (the word `unknown` in the
      vocabulary reads "not enough to compare", a pay-comparison meaning, so these pages use `common.unknown`). Ten client functions
      added to `wpm.ts` and to the path-map test in call order. 77 catalogue strings and 28 vocabulary values in the 13 `-001`
      locales; this also adds the 20 vocabulary tokens that earlier tasks (the oversight, early-warning, pack and cutover tasks among them) had put in the service
      without a `values` entry, which the vocabulary test was failing on. The CMS config is regenerated.
      **Verified:** 113 unit tests (path map, parity, no English left outside the catalogue, every server token has a value),
      `svelte-check` 0 errors, prettier clean, build, and **81 Playwright specs** (21 new: five behaviour specs and sixteen
      page × light/dark × desktop/phone checks that the page does not scroll sideways, each with a screenshot). Screenshots were
      **looked at** in light and dark at desktop and phone sizes: the dark ones set the saved Lily theme, since the app's dark mode is
      a theme the picker loads and the browser colour scheme alone leaves the page light (the older `/contingent` "dark" screenshots
      were light for that reason). **Mutation-checked** (an unknown cell tinted as a pass; a cutover unknown marked with a tick; the
      reason a run stopped hidden; a refused freeze swallowed; the withdrawn mark hidden), each failing a spec; the first two needed
      assertions added (the class and the mark), since the specs read the status attribute only. The service was not changed, so the
      Rust suites were not rerun.
      **Not done / limits:** no page to **plan** a cutover, add its roles and pools, record a level or position, enter oversight
      measures and readings or capacity pools and demand (the pages read; the writes are by API); the cutover reasons and the
      oversight and capacity "basis" texts come from the service in English, and the page says so; the early warnings (WPM-T115)
      have no page; the heatmap shows twelve months at most and does not link a pool to its members (by design); "1 months over" is
      not pluralised (the strings carry no plural forms); the new translations are AI-written and unreviewed, like the rest
      ([REVIEW.md](../workforce-planning-management-ui-with-svelte/content/locales/REVIEW.md)); no real-service run of the new pages
      (the live suite has no spec for them); the key-person list reads the caller's whole scope, not the chosen organization (the
      route takes none); screen reader behaviour was not tested beyond the marks and words being text.
      **Follow-up pass (same task), after review:** *Writes:* forms on the pages for planning a cutover (create, add a role, add a pool,
      remove after a confirmation), skill pools (create, add a member, a programme's claim), oversight (a level, the position, a
      criterion, a measure, a reading; a missing value is sent as `null` with no evidence) and a new page `/warnings` (the early
      warnings with days open and time to report, acknowledge, "Look now"); a refusal (403) is said in words and any other failure
      shows the service's message. *English text:* cutover checks now carry `details` (`{code, params}`, same order as the English
      `reasons`; 12 codes) and the page says them in the reader's language, falling back to the English text, with a note, only for a code
      it does not know; the "how this is worked out" texts (capacity, key-person, governance, oversight, cutover, early warnings) are in
      the catalogue, with the service's own English beneath. *Plurals:* the month count is worded without a number-dependent noun
      ("Months over: 1 · unknown: 1"). *Key-person scope:* `GET /api/workforce-intelligence/key-person` and `/named` take
      `?organization=` (outside the caller's scope `404`), and the page asks for the chosen organization. *Live suite:* four specs
      (`tests/live/oversight.spec.ts`) ran against the real service in a scratch workspace and passed (one needed a rerun after the
      service's own `429` rate limit, which the page showed in words). *Screen reader:* the heatmap is checked as a table by role (column
      headers, a row header, a word in each cell, the mark hidden), which is the part that can be tested without a screen reader.
      **Verified:** 561 unit, 95 request and every enforcement binary (`enforcement`, `_conversion`, `_cutover`, `_engagements`, `_equality`,
      `_expenses`, `_flexible`, `_health`, `_me`, `_need_to_know`, `_oversight_pack`, `retention_schedule`, `security`; the Keycloak ones
      need their feature and were not run), clippy, fmt; 113 vitest, `svelte-check`, prettier, build, **94 Playwright specs** (13 new);
      screenshots looked at (light and dark, desktop and phone). **Mutation-checked** (service: no `404` for another organization's
      register, the organization filter ignored, a wrong reason code; UI: percent not converted to basis points, reasons ignoring the
      data, a missing reading sent with evidence, the organization not passed, the refusal not said), each failing a test.
      **Still not done:** the translations (now 88 more strings, including long explanatory texts) are AI-written and unreviewed, and
      added to the review record; no person has read them; a screen reader was not run; removing a level, criterion or measure, and
      changing a pool member or a claim's status, is by API only; the live specs are not part of the default run and the rate limit
      can fail a quick rerun.
      **Second follow-up (2026-10-13): Lily upgrade, a demo tour, two bugs found by using the app.** *Lily:* `@lilydesignsystem/*`
      upgraded (picker bar 0.1 to 0.3, the locale, theme, share and text-size pickers to 0.2, headless 0.4, themes 0.4) and the
      header now carries the **link picker** (a home icon: every page, in the reader's language, the current one marked, locale kept)
      and the **search picker** (sends the query to the directory, which opens with it filled in from `?q=`). Four strings in 13
      locales. The 0.2 pickers add hover tooltips that the headless themes do not position: in flow, one widened its picker's box on
      hover and moved the button out from under the pointer, so a click on the language or text-size button did nothing (found
      because the existing locale-switcher spec failed); the tooltips are now out of flow in the layout CSS, with a spec that hovers
      and clicks every picker. *SSO:* `/signin/sso` (and under any locale) threw a `500`, since SvelteKit's `redirect()` refuses an
      address on another origin; it now returns a plain 303, with a spec. **Not done:** the UI's SSO contract (`/api/auth/sso/start`
      and `/sso/callback`) does not match the sibling authentication service, which exposes `/api/auth/oidc/login` and bridges back
      through the magic-link `/verify?token=`; SSO has not been run end to end against Keycloak. *Request loop:* the role picker
      on `/me` asked for the GDAD role profiles again after every answer when the answer was empty (it read the list length in its
      own effect): 744 requests in three seconds on a service with no such profiles, enough to trip the service's rate limit and
      break every later page; fixed (asked once), with a unit test that fails on the old code. *Demo:* where `WPM_DEMO_API_URL` names
      a separate service instance that holds only fictitious data (authentication off), signing in as **example@example.com** (no
      email is sent) sets a `demo:` session and opens `/demo`, a guided tour of every page (34 steps in nine sections, the same
      sections as the public tour), and a banner on each page says the data is fictitious with Step n of 34, Previous and Next.
      The demo session reaches that service and nowhere else (no token exchange at the authentication service; with no demo
      configured the `demo:` prefix opens nothing, a `401`); its memberships (an administrator of the demo organization, the first
      permanent worker as "me") are made up in the proxy, since nothing else says who the demo user is. `scripts/demo-data.py`
      (standard library, public API only, so every rule that guards a write applies) fills a seeded service with fictitious
      capacity, governance, oversight, a rota, a cutover, 11 contractors with funding, a conversion plan, a workforce plan, groups,
      learning, a mentorship, reviews, succession, CPD, wellbeing, a pulse survey, announcements, payroll, an expense claim and a
      change initiative: it ran with 0 failed steps and every one of the 34 pages loaded with data and no error. Strings: 11 in 13
      locales. **Verified:** 123 vitest (the demo session, the sign-in branch with and without a demo and for another address, the
      proxy routing and refusal, the made-up memberships, the tour order, the role-picker loop), svelte-check, prettier, build, 106
      Playwright specs (the tour page, Next and Previous, locale kept, no banner for an ordinary session, four screenshot sizes
      checked for sideways scroll and looked at). **Not done / limits:** the demo's caller-specific pages (my leave, the expense
      decisions queue for "my reports") show little, since the demo user has no real identity; the demo is not reset on a timer
      (rerun the script on a fresh database); demo data is loaded by a script, not a Loco task; `/workers` on a phone scrolls
      sideways (not touched here); the demo hint is client-rendered; Keycloak was not launched.
      **Found by the first real trial (Safari engine):** signing in as the example address did nothing. The server set the
      `Secure`, `__Host-` session cookie over `http://localhost`, which Chromium and Firefox keep and Safari (WebKit) refuses, so the
      next page sent the visitor back to sign in; the Playwright specs run Chromium only and passed. On plain http on this machine
      (localhost, 127.0.0.1, ::1) the session now travels in a cookie without the prefix and without `Secure` (`mxi_session`);
      anywhere else only the host-locked cookie counts and a cookie of the other name is ignored (unit-tested both ways, and the
      magic-link and SSO verify routes and sign-out use the same helpers). Checked by signing in with Firefox, Chromium and WebKit
      through Playwright: all three reach the tour and keep the session. The specs still run Chromium only.
      **Third follow-up (2026-10-13): the home page, a board, organizations.** *Home:* the signed-in dashboard's headline is "My
      Home Page" and it has **six tiles** (active workers, open vacancies, succession gaps, and new: single points of failure,
      early warnings not acknowledged, pools over capacity), each linking to its page; the three new ones read across every
      organization the person belongs to, and one that cannot be read shows a dash, never 0. The header says **Sign out** (a POST
      form; best effort at the authentication service, the cookie is cleared regardless) when signed in and **Sign in** when not,
      which it did not before (it always said Sign in and the app had no sign-out control at all). *Board:* `/<locale>/kanban`
      ("My board") shows the signed-in person's development-plan steps on `@lilydesignsystem/svelte-kanban-board`, one column
      for each status (planned, in progress, achieved, abandoned); a card moves by drag or by its Move menu, the move is saved at
      once (`PUT /api/development-plan-items/{pid}`) and the board is read again so a refused move goes back and its reason stays
      shown; marking a step achieved says the declared skill level is unchanged. The existing board component's labels were
      English literals and are now in the catalogue (the requisition board gets them too; "Cards: n", with no plural noun).
      *Organizations:* organizations have no names in this system (a URN), so the demo's are four organizations that have
      people (the seeded one and three regions of 12 people each, with governance, an oversight scale and position, two with a
      skill pool and a claim) and a parent that contains them all (`organization-confederations`); the demo user is an
      administrator of each, made up in the proxy, and "me" (the first permanent worker) has a development plan with a step in
      each of three statuses. **Verified:** 129 vitest, svelte-check, prettier, build, 120 Playwright specs (the six tiles and
      their figures, a failing tile, Sign in versus Sign out signed in and out, signing out, the Welsh headline and button,
      the board's columns, a saved move, a refused move put back, no worker record, no plan, Welsh, four screenshot sizes);
      the demo ran with 0 failed steps and the home page showed 5 organizations and the populated board. **Not done:** converting
      the data tables to `@lilydesignsystem/svelte-data-grid` (see below); the organizations have no names to show in the picker.
      **Data grid:** the grid (0.1.0) renders text cells only (`format` returns a string; no cell snippet), so it cannot hold the
      links, buttons, chips and inputs that 36 of the UI's 73 tables carry, and the other 37 are plain text. Nothing was converted,
      pending a decision on whether to convert only the plain-text tables.
      **Update 2026-10-13:** `@lilydesignsystem/svelte-data-grid` 0.4.0 adds `cell` snippets, `mode="table"`, `rowHeader` and
      `rowAttrs`, so link/chip tables can convert. `DataTable.svelte` wraps it (sortable headers, locale-aware collation, unknown
      sorts last and shows "—"). Converted: `/directory`, `/job-levels`, `/movements`. **Not done:** the other ~70 tables;
      pagination, column visibility and the filter box (each needs new strings in every locale).
      **Update 2026-10-17: SVAR removed.** `/requisitions` (board) and `/workers` (list, with a filter box and 25 per page) now use the
      Lily kanban and `DataTable`; the six `@svar-ui` packages and `svar-filter-augment.d.ts` are gone. The worker filter is a
      single text box over every column (was a per-field rule builder); the worker name is now a link, not row selection.
      Verified: svelte-check 0, 129 vitest, 127 Playwright. Not looked at: screenshots of the two pages; drag on the board.
      **Update 2026-10-14:** `dataTable.*` strings (columns, show column, previous/next page, page status, sort announcement) are
      in all 12 `-001` locales (AI-written, unreviewed); `DataTable` takes `pageSize` (pager shown only when rows overflow) and
      shows the column picker when a column is `hidable`. Converted 31 more tables; 29 stay plain with a comment (headerless
      lists with no header strings, `<tfoot>`/colspan rows, inline forms/selects that must keep focus, matrices, a hidden chart
      table, and `movement-items`, whose spec reads `tr[data-state]`). **Verified:** svelte-check 0 errors, 129 vitest, 121
      Playwright specs pass (one pickers spec failed once in the full run and passed on re-run). **Not checked:** no screenshots
      looked at; no spec exercises the pager or the column picker; column visibility is not yet used by any table.
      **Update 2026-10-14 (headerless tables):** 27 `columns.*` header strings added to the 12 `-001` locales (AI-written,
      unreviewed; fr "Dates"/"Table" allow-listed as the same word). Converted 14 more tables: `workers/[pid]` (5), `workforce`
      (4), `wellbeing` (3), `development` (2), `privacy` (1) — headers now name what the old cells labelled inline (payslips drop
      "Gross:"/"Net:", enrolment conversion drops its prefix). Still plain: `contingent` figures list, `movement-items`,
      `held-items`, the tables with inline forms. **Verified:** svelte-check 0 errors, 129 vitest, 121 Playwright. **Not checked:**
      screenshots.
      **Update 2026-10-14 (upgrade and calendar):** all `@lilydesignsystem` packages raised to latest (calendar 0.3.0, data-grid
      0.5.0, gantt-chart 0.3.0, headless 0.5.0, kanban-board 0.3.0). `CalendarPanel.svelte` wraps `svelte-calendar` (week, four
      weeks, month; weekday and month names from Intl; 12 `calendar.*` strings in the 12 `-001` locales, AI-written, unreviewed)
      and is on `/workforce` (pending leave, shifts), the worker page (leave), `/rota` (on-call runs) and `/cpd` (ledger entries,
      requirement period marked). Events show a text title, never colour alone. **Verified:** svelte-check 0 errors, 129 vitest,
      Playwright (new `calendar.spec.ts`: entry on its day, period marked, view switch, Welsh) and a screenshot looked at.
      **Follow-up 2026-10-14:** registration expiries join the `/cpd` calendar and `/development` gets a certificate-expiry
      calendar (`calendar.expiring`, all 12 locales). Today is outlined, not filled (the fill read poorly in dark). The grid now
      fits a 390px phone without page scroll. Playwright 126 pass; light/dark/phone screenshots of the `/cpd` calendar looked at.
      **Not done:** day-click or drag-to-reschedule, team calendars across organizations; the other calendars (`/workforce`,
      `/rota`, worker page, `/development`) have no screenshots or content assertions.
      **Update 2026-10-17:** all `@lilydesignsystem` packages raised again (calendar 0.5.0, data-grid 0.9.0, gantt-chart 0.5.0,
      kanban-board 0.6.0, pickers 0.2–0.4, themes 0.5.0); svelte-check 0, 129 vitest, 126 Playwright (one full run flaked after
      the install and passed on re-run). New in those versions and not yet used: typed fields and in-place edit (`onFieldEdit`,
      `onCellEdit`), CSV/TSV/Excel export, column reorder and pinning, `virtualize`, calendar agenda view. Export would be a new
      disclosure path, so each use needs a privacy review first (AGENTS/privacy-and-data-rules.md).
      **Follow-up 2026-10-17 (adoption):** `DataTable` now passes `empty`, `footer`, `locale` and `exportName`; `training-plan` and
      `expense-items` are converted (totals rows as footers); the first column is pinned on five wide tables; date columns on
      seven tables use the grid's `date` type (shown in the reader's locale, no longer ISO). Calendars open on the agenda on a
      phone (`calendar.agenda`, `agendaEmpty`), and leave status and shift fill are typed event fields. Kanban columns say when
      they are empty (`kanban.emptyColumn`). **Export** (CSV and Excel, `dataTable.export*`) is on five tables that name no
      one: metrics, training analytics, contingent departments, pay bands, CPD overview. It is a new disclosure path and has no
      requirement or decision entry yet: **a requirement and a decision are still to be written** (spec-first rule).
      **Not done:** in-place editing (`onCellEdit`, `onFieldEdit`) on the grid, calendar and Gantt, which needs a write endpoint
      and a permission decision per table; Gantt task fields; `movable` columns (needs announcement strings).
      **Verified:** svelte-check 0, 129 vitest, 127 Playwright (a new spec downloads the CPD overview CSV and checks that an
      unknown stays a dash); one phone screenshot of the agenda looked at. Localized dates in the grids were not looked at.
WPM-T120–T122 (critical chain buffers) were withdrawn on 2026-10-09 by
[WPM-D55](design.md): critical chain, critical path and similar scheduling
capabilities live in project-portfolio-management. The ids are not reused.

## Phase 13 — production readiness (WPM-R72–R78, WPM-R87–R88, WPM-D52–D54)

Specified in [production-readiness.md](production-readiness.md) (what is built)
and [plan.md](plan.md) section F (what is still proposed: the backup plan and
the data protection impact assessment). Started 2026-10-09 from a review of
what stops a buyer deploying a named version.

- [x] WPM-T123 (2026-10-09) **Standalone build.** *(traces to WPM-R77)* The two shared
      crates, `entity-ref` and `authentication-verifier`, are **copied into**
      `workforce-planning-management-service-with-rust/crates/` (benches, fuzz targets and
      their own locks dropped), and `Cargo.toml` points at them: no `../../` path
      remains. `compose.test.yaml` mounts an in-repo `postgres-init/` instead of the
      missing `../../ci/postgres-init`, and its comments no longer name monorepo
      scripts. Added `Containerfile` (multi-stage, non-root, `LOCO_ENV=production`),
      `config/demo.yaml` and a root `compose.yaml` (PostgreSQL + API; sign-in off,
      explicitly, with a warning). Verified: the service directory **alone** (no sibling
      directories) passes `cargo check --all-targets`, clippy, 328 unit tests and the
      database suites; the `demo` config booted against PostgreSQL 18, ran the migrations
      and answered. **Not verified:** `Containerfile` and `compose.yaml` were never
      built or run (the Podman VM has no outbound network, so base images and crates
      cannot be fetched); a clone into a truly empty directory; the UI is **not
      containerized** (it has only `adapter-auto`, and signing in needs the identity
      service). **Found:** `config/production.yaml` could not parse with `SMTP_USER` and
      `SMTP_PASSWORD` unset (empty values rendered as YAML null); now quoted. **Trade-off:**
      the vendored crates will drift from their upstream; the choice was to copy rather
      than publish to crates.io, which needs the owner's account.
- [~] WPM-T124 (2026-10-09) **Automated checks.** *(WPM-R77, WPM-R88)* Written, **never run
      on GitHub**: `.github/workflows/ci.yml` (documents checks; fmt, clippy `-D warnings`
      and unit tests; the database, enforcement, `enforcement_expenses` and `security`
      suites against a Postgres 18 service; `cargo deny check`; the UI with Playwright and
      an advisory `pnpm audit`; a non-required Keycloak job and OWASP ZAP baseline) and
      `.github/dependabot.yml` (cargo, npm, actions). Every command was run locally except
      the Keycloak suite, the ZAP scan and `pnpm audit` (no network or container runtime
      for them). `cargo deny check` passes against the cached advisory database, which may
      be stale. **Not done:** requiring the jobs through branch protection (a repository
      setting); the axe accessibility tests (none exist in this repo); `actions-rs` was not
      carried over, so there is nothing to replace.
- [x] WPM-T125 (2026-10-09, root `LICENSE.md` added 2026-10-10) **License files.**
      *(WPM-R78)* One full-text file per license in `LICENSE/`: `LICENSE-MIT`,
      `LICENSE-APACHE`, `LICENSE-BSD-3-CLAUSE`, `LICENSE-GPL-2.0`, `LICENSE-GPL-3.0`, taken
      from the SPDX license list (MIT and BSD carry `Copyright (c) 2026 Joel Parker
      Henderson`). `LICENSE/index.md` (and its `README.md` copy) gives the reason for the
      options. A **root `LICENSE.md`** states each subproject's SPDX expression and links
      the texts; README, CITATION.cff, AI_STATEMENT and the special-files spec point at
      it. `scripts/check-licenses.py` checks the manifests' `license` fields against the
      files and against both summaries, and runs in CI. **Not verified:** that GitHub
      detects a license from `LICENSE.md` plus a `LICENSE/` directory (it recognises a
      root file by name and content, and `LICENSE.md` holds an SPDX expression rather
      than a license text, so detection may still fail); and the "why several options"
      wording is the assistant's, for the owner to confirm. The workforce UI's
      `package.json` stays `MIT OR Apache-2.0`, as its spec says.
- [x] WPM-T126 (2026-10-09) **Sign-in enforcement on by default.** *(WPM-R72, WPM-D52)*
      `auth::parse_require_auth`: on unless an explicit `0`/`false`/`no`/`off`.
      `auth::startup_check` (pure) refuses to boot in `production` with enforcement off or
      with no key source; a warning at boot and every ten minutes while it is off; public
      `GET /_posture`. `/metrics.prom` is no longer public. Existing request suites opt out
      through `tests/requests/mod.rs::request_open`. Verified: unit tests for the default
      and the startup check; the enforcement persona matrix (now also: metrics 401 without
      and 200 with a service token; `/_posture` open); the `security` binary running with
      no variable set gets a `401`; **mutation-checked** (flipping the default to off fails
      the unit test and the live test); run for real: the default boot answers `401`,
      `LOCO_ENV=production` with `WPM_REQUIRE_AUTH=0` or with no key source exits with the
      refusal message, and with a key source boots and answers `401`. **Not done:** the UI
      banner "sign-in not enforced" (the UI would call `/_posture`; nothing reads it yet).
- [x] WPM-T127 (2026-10-09) **Security headers and CORS.** *(WPM-R73)* `src/security.rs`
      (`security_headers`, pure) as the outermost API layer: CSP, nosniff, referrer,
      permissions, COOP, CORP, HSTS over TLS, `no-store` on `/api`. UI:
      `src/lib/security-headers.ts` applied in `hooks.server.ts`, and `csp` in
      `vite.config.ts`. `config/production.yaml` CORS is an allow-list from the required
      `WPM_CORS_ORIGIN`. Verified: 5 unit tests in the API, 3 in the UI; the `security`
      binary checks the headers on a `200`, a `401` and a `429`; a Playwright spec loads
      `/signin` and `/en-001/tour`, checks the headers and that the policy blocked
      nothing (46 specs pass); run for real in the production environment, an allowed
      origin gets `access-control-allow-origin` and another origin does not.
      **Not done:** the pages behind sign-in were not checked for policy violations (they
      need the stubbed API; only two public pages were); `style-src` still allows
      `'unsafe-inline'`; the Swagger UI policy allows inline scripts; HSTS is only sent
      when the request is over TLS.
- [x] WPM-T128 (2026-10-09) **Rate limiting.** *(WPM-R74, WPM-D54)* In-memory fixed-window
      limiter in `src/security.rs`: 600 requests per minute, and 20 for erase, sweep, import,
      merge, deduplicate, token and export routes; `429` with `Retry-After`; both
      configurable, `0` disables. The caller is the network address, **not** the token:
      the first design keyed on the bearer token or `X-Forwarded-For`, which a guesser
      could vary to avoid every limit; it was changed before this landed.
      `X-Forwarded-For` counts only with `WPM_TRUST_FORWARDED`. Verified: unit tests (trip,
      window reset, callers and classes apart, zero disables, the key ignoring a token and an
      untrusted header); the `security` binary (trips, `Retry-After`, headers on the `429`,
      varying the token does not help, health exempt, the sensitive class is stricter).
      **Not done:** limits are per instance, so several instances each count alone;
      behind a proxy that does not overwrite `X-Forwarded-For`, spoofing is possible once
      `WPM_TRUST_FORWARDED` is set; no `tower_governor`.
- [ ] WPM-T129 *(superseded by WPM-T149–T150)* **Backup plan.** *(WPM-R75)* Write `spec/backup-and-restore.md`:
      RPO and RTO defaults, base backups with WAL archiving, a nightly logical dump,
      encryption and off-host storage, retention in calendar days, the erasure ledger
      and its replay, restore drills, and the runbook. Update regulatory.md and the
      retention section.
- [ ] WPM-T130 *(superseded by WPM-T148–T149)* **Erasure ledger and restore drill.** *(WPM-R75, WPM-D53)* Migration
      for `erasure_ledger` (pid, date); the erasure path writes it; a
      `restore_replay_erasures` task re-applies every entry after a given date; a
      scripted drill restores into a scratch database (Podman), replays erasures and
      checks row counts and migrations. Request test: erase, restore an earlier dump,
      replay, and the record is gone again.
- [ ] WPM-T131 *(superseded by WPM-T151)* **Data protection impact assessment.** *(WPM-R76)* Write `spec/dpia.md`:
      the processing, necessity and proportionality, risks to people and their
      likelihood and severity, and the measures in the design, each linked to its
      decision id. Mark it a template completed for the demo, with what a deployer
      must add and who signs it off. Add a checklist line to spec-driven-delivery.md:
      a requirement that adds personal data updates the DPIA.
- [ ] WPM-T132 **Production gates refresh.** *(WPM-R72–R78)* Add gates WPM-G3
      (security headers and rate limits verified on the deployment), WPM-G4 (a restore
      drill passed within the last 90 calendar days) and WPM-G5 (the deployer's DPIA
      signed off), and mark WPM-G1 as code-complete once WPM-T126 lands.

- [~] WPM-T144 (2026-10-09) **A release people can pin.** *(WPM-R87)* Written, **never run**:
      `.github/workflows/release.yml` on a `v*` tag that equals the crate version builds the
      API image from `Containerfile`, pushes it to GHCR by version and commit, generates a
      CycloneDX bill of materials (`cargo cyclonedx`, run locally: 535 components) and
      creates a GitHub release with generated notes and the bill attached. The first tag is
      its first test. **Not done:** a first tag and release; image signing; a UI image.
- [x] WPM-T145 (2026-10-09) **Documents match the code.** *(WPM-R88)*
      `scripts/status.py` generates `spec/implementation-status.md` from the files and
      `--check` fails when it is stale (CI runs it). `scripts/check-links.py` found 56 dead
      relative links; the 50 that pointed at sibling repositories or at files that were never
      here are now plain text (the two example-heavy folders `spec/special-files-for-public-repos`
      and `spec/oxford-spelling`, and `.sota`, are skipped), and the vendored crates' own
      docs were cleaned; it now reports 0, and CI runs it. `scripts/check-days.py` enforces
      `spec/calendar-days-or-business-days` in CI. `lychee` was not used: both scripts run
      offline with no extra tools, but neither fetches external URLs. **Not done:** the
      project-portfolio-management repository's roadmap corrections (another repository);
      the roughly 20 dead links named in the review are in that repository too.

## Phase 14 — programme-linked planning and contingent workforce (proposed WPM-R79–R86, WPM-R98, WPM-D56–D59, WPM-D66)

Proposed 2026-10-09, not started, in response to a user complaint: "It plans
headcount by department, with no link to programmes and minimal tracking of
contractors and fixed-term staff." The research, requirements and decisions are
in [plan.md](plan.md) section G. WPM-T134 and WPM-T136 fix a spec drift and a
payroll defect and need nothing else; do them first.

- [ ] WPM-T133 **Spec round.** *(traces to WPM-R79–R86, WPM-D56–D59)* Move section G
      into `programmes-and-contingent-workforce.md`; update
      [domain-model.md](domain-model.md), [auth.md](auth.md) (rate masking, HR-only
      named lists), [audit.md](audit.md), [regulatory.md](regulatory.md) (new personal
      data, the DPIA once WPM-T131 exists), [glossary.md](glossary.md) (engagement
      basis, engagement end, extension, supplier, engagement route), the index and
      `llms.txt`. No code.
- [x] WPM-T134 (2026-10-10) **Correct the employment type in the spec.** *(traces to WPM-D56)* `spec/domain-model.md`
      listed `full_time | part_time | contract | intern`; the code enforces `permanent | fixed_term |
      contractor | intern`. The spec now matches the code and says the working pattern is `fte_percent`. A
      search of the other specs found no other use of the old values; the OpenAPI text and the UI labels
      were not re-read for them.
- [x] WPM-T135 (2026-10-11) **Pure core: engagements and basis-aware supply.** *(traces to WPM-R79, WPM-R85, WPM-D57)*
      `rules/engagement.rs`: the four bases; which need, allow or refuse an end date (permanent refuses,
      fixed-term and contractor need one, an intern may have one); an end after the start; an extension
      that moves the end later; the reminder window in calendar days (default 60, 1 to 366); the standing of
      an engagement (open-ended, **missing end date**, running, ending soon, **past end undecided**, past end
      decided); much-extended and long-running flags (strictly above the threshold; a month-end start does
      not overflow). An end date is the **last day** of the engagement. `rules/planning.rs`:
      `project_by_basis` projects each basis on its own terms, in headcount and FTE, naming each assumption:
      known end dates for fixed-term, contractor and intern; attrition for **permanent staff only**; someone
      with no end date counted as continuing and counted in the assumptions; permanent staff with no rate
      (`insufficient_history`) give an unknown supply and an unknown total, never zero, while no permanent
      staff is a known zero. Verified by 14 new unit tests (367 in all) including an engagement ending on the
      target date, an extension past it, and no permanent history; clippy and fmt clean; a boundary mutation
      (end on the target date no longer counted) fails a test. **Not done:** nothing calls these yet; the
      forecast endpoint still uses the single attrition rate (WPM-T137, T140 wire them in).
- [x] WPM-T136 (2026-10-10) **Payroll excludes contractors.** *(traces to WPM-R80, WPM-D58)* `rules::payroll::on_payroll` and
      `OFF_PAYROLL_TYPES` (`contractor`); the run's calculation filters them out, and
      `spec/payroll-compensation.md` says so. Verified: a unit test over every employment type (and an
      unknown one stays paid); a request test with a permanent, a fixed-term and a contractor worker, all
      with a salary, giving two payslips and none for the contractor; **mutation-checked** (removing the
      filter fails it). Full suites unchanged: 334 unit and 64 request tests pass, clippy and fmt clean.
      **Behaviour change:** a contractor who was in a run before now gets no payslip. **Not done:** the
      benchmark comparison still includes every worker with a salary (it is not payroll); the contractor's
      rate fields (WPM-T137) do not exist yet.
- [x] WPM-T137 (2026-10-11) **Engagement fields, extensions and contractor details.** *(traces to WPM-R79, WPM-R80)*
      Migration `000053`: `workers.engagement_ends_on`, `engagement_extensions`, `worker_contractor_details`
      and `engagement_status_assessments`. `controllers/engagements.rs` and eight operations in the OpenAPI
      document; the end-date rule on create, on hire and on update (a set date moves only by extension).
      Existing fixed-term and contractor rows get no invented date and are listed by
      `GET /api/engagements/missing-end-date`. The rate is masked like salary; audit entries carry no rate,
      reason or outcome; the three tables are in the subject-access export and the erasure. Verified by three
      request tests (the rule per basis, including hire; the missing list; details, assessments, export and
      erasure) and a new enforcement binary `enforcement_engagements` (payroll and the worker see the rate; HR
      and a stranger see it masked; writing is HR's), **mutation-checked** three ways (the rate unmasked,
      erasure forgetting the details, no end-date check on create). **Differs from the task:** supplier, route
      and rate are in `worker_contractor_details`, not columns on `workers`, so no existing endpoint returns
      the rate; the new tables are not in the retention sweep. The seed gives its fixed-term workers an end date
      and the payroll test now supplies one. Described in [engagements.md](engagements.md).
      **Not verified:** any user interface (none), and the rate against a production policy other than the
      reference.
- [x] WPM-T138 (2026-10-12) **End-of-engagement reminders.** *(traces to WPM-R81)* The Loco task `engagement_end_reminders [days_ahead:N] [as_of:YYYY-MM-DD]`
      (default 60 calendar days; schedule daily) tells the worker's manager and each `hr_admin` of their organization
      about an engagement that **ends inside the window**, and once about one that **has ended with no recorded
      decision**; migration `000061` records each (worker, end date, kind) so it is **idempotent**, and an extension gives a
      new end date and so a new reminder. The notification names the worker and the date and **carries no rate, supplier
      or reason** (verified). `POST /api/workers/{pid}/engagement/decision` records extend, convert or end against the end
      date it settles, by the line manager or a privileged caller, never the worker; the engagement view shows the
      decision and the standing becomes `past_end_decided`; a decision made for one end date settles nothing after an
      extension, and the history is kept; the audit names the event, not the decision. Verified by a request test (manager
      and HR told; a decided one and a permanent one and one outside the window not; a re-run tells no one twice; the
      window moving; an extension; validation; the standing) and a unit test, **mutation-checked** four ways (no
      idempotence, caught by the unique key; a decided engagement still asked about; the rate reaching the notification; no
      past-end reminder). **Also found and fixed:** the OpenAPI entries for the WPM-T137 engagement routes had been lost
      when files were synced from a scratch copy, and twelve learning routes were never documented; all are restored or added,
      and a new unit test (`openapi::tests::every_registered_route_is_documented`, mutation-checked) fails when any route the
      controllers register under `/api` has no path in the document. **Not done:** the task is not
      scheduled by anything in this repository (a deployer's cron runs it, as for the others); HR is the `hr_admin` members
      of the worker's own organization only, not of an organization above it; and a decision of *convert* records intent
      only (conversion plans are WPM-T162 to T164).
- [x] WPM-T139 (2026-10-11) **Snapshots by basis.** *(WPM-R84)* Migration `000065` adds nullable contractor and fixed-term headcount and FTE to `headcount_snapshots`; `snapshot_headcount` fills them (`rules::metrics::snapshot_rows`, tested); older rows are NULL, which means unknown. Not exposed per department. Original scope: Migration adding per-basis
      headcount and FTE to `headcount_snapshots`; the snapshot task fills them; older
      rows report `unknown` for the split. Request test.
- [x] WPM-T140 (2026-10-11) **Programme, FTE and basis on demand lines and requisitions.** *(WPM-R82)* Migration `000066`:
      `plan_demand_lines` gains `programme_ref` (a `thing:` URN), `target_fte_percent` (not stated is unknown, never assumed to be
      the headcount), `basis` (default `permanent`) and `expected_ends_on`; `requisitions` gain `programme_ref`, `basis` and
      `expected_duration_months`. Checks in the database and in `rules::planning`: an end date (or a duration in calendar months)
      is required for `fixed_term` and `contractor`, allowed for `intern`, refused for `permanent`, after the target date, at most
      120 months; an FTE target is between 0 and 100 for each head. The demand-line key now includes basis and programme, so a
      permanent and a fixed-term line for one department, role and date can both exist (**found by a request test: the first
      attempt failed with a 500 on the old unique index, and the migration now replaces it**). **The hire carries the
      requisition's basis, and the end date it implies** (hire date plus the duration, to the day before), unless the hire states
      its own. **The forecast** now projects each basis on its own terms through `project_by_basis` (WPM-T135): permanent staff by
      the attrition rate, fixed-term, contractor and intern on their recorded end dates, so it also gives supply by basis, FTE,
      the FTE gap (only when every line states an FTE), demand by basis and by programme, and a plan-wide list by programme and
      date; the levers add `fixed_term_hire` and `engage_contractor` for a shortfall. 6 new unit tests; 2 request tests.
      **Mutation-checked** six ways (an unknown FTE summed as zero; the duration off by one day; the FTE bound; an end date on the
      target date; the hire ignoring the requisition's basis; supply ignoring end dates), each failing a test. **Not done:** the
      hire does not create a post funding from the requisition's programme (funding stays HR's record, WPM-T161); the supply is
      not split by programme (workers carry a programme only through their funding); a lever is a suggestion, not a plan action;
      the user interface (WPM-T143). **Changed behaviour:** with no permanent staff in a department the supply is now known without
      an attrition rate, where before it was withheld.
- [x] WPM-T141 (2026-10-11) **Programme roll-up.** *(WPM-R83, WPM-D59)* `GET /api/workforce-intelligence/programmes` (HR only,
      audited as `programme_rollup_read`): per programme (the programme URN on a worker's post funding, WPM-T161) and month for twelve
      months from the current one: FTE funded (funding with no end, or ending on or after the first of the month), the contractor and
      fixed-term share of that FTE, engagements ending and funding ending. Programmes are linked through posts, not time recorded.
      Aggregate only: a programme with fewer than 10 people is left out and counted in `withheld_programmes`, a month figure for one to
      nine people is withheld (zero is shown), and no total across programmes is given. `rules::programmes`: 6 unit tests; a request
      test and an HR-only assertion in the conversion enforcement binary. **Mutation-checked** (programme floor ignored; month floor
      ignored; funded figure always shown; the funding end boundary, which a first run did not catch and a test now does), each failing a
      test. **Not done:** plan demand lines by programme (WPM-T140), the user interface (WPM-T143), and programmes owned by
      project-portfolio-management are only references here (the URN is not looked up).
- [x] WPM-T142 (2026-10-11) **Contingent workforce view.** *(WPM-R86)* The engagement figures in
      `GET /api/workforce-intelligence/contingent` (HR only, audited): ending in 0–30, 31–60 and 61–90 calendar days as
      **separate bands** (cumulative figures would let a withheld band be worked out by subtraction), past the end with no
      decision recorded for the current end date, extended more than 2 times, and contractors engaged longer than 24 calendar
      months. One to nine is withheld, zero is shown, no total. And the **named lists** behind each figure:
      `GET /api/engagements/contingent-list?list=` (`ending_within_30`, `ending_31_to_60`, `ending_61_to_90`,
      `past_end_undecided`, `much_extended`, `long_running_contractors`), HR only, **audited** (`contingent_list_read`), by worker
      number, department, basis and dates; no pay, no name, no floor. One definition (`rules::engagement::on_list`) serves the count
      and the list, with a test that they agree. Unit tests for each rule; request tests; the conversion enforcement binary shows the
      worker, the manager and a neighbour refused both the aggregate and the list. **Mutation-checked** (floor ignored; a band made
      cumulative; the decision ignored; a threshold made non-strict; the list route opened to all; a band edge moved), each failing a
      test. **Not done:** headcount and FTE by basis per programme and pool (pools are the department until plans define them;
      programme lines are WPM-T140); the thresholds being configurable (constants); the user interface (WPM-T143).
      Original scope: Aggregates per department,
      pool and programme; ending in the next 30, 60 and 90 calendar days; past end
      with no decision; extended beyond the threshold; contractors over the
      long-running threshold. HR-only named lists with audited reads. Enforcement test
      that a manager sees aggregates only.
- [x] WPM-T143 (2026-10-11) **UI and verification.** *(WPM-R79–R86)* The page `/contingent` (HR; the nav gains it): share of
      FTE for contractors and fixed-term staff apart, the engagement figures, "Show who" by worker number (the read is audited), funding
      ending by month, the trend (`insufficient_history` says so), and the programme roll-up. A figure of one to nine people is
      "withheld", never 0, and no total is shown. The worker page gains an **engagement panel** (basis, end, standing, extensions,
      decision; HR sees the funding beside the contract end with a ⚠ and words when they differ; a contractor's supplier, route and
      rate, **masked** as a state). The plan page's line form takes basis, FTE (empty is unknown) and programme, and the forecast
      shows demand and supply by basis, the FTE gap, demand by programme, and the levers in words (**found by a browser test: they
      printed raw tokens**). Requisition cards show basis, duration and programme. Client functions in the path-map test. **62
      strings and 64 vocabulary values in all 13 `-001` locales**, AI-written and unreviewed; that also fixed the `untranslated` test
      that had been failing, because the vocabularies of WPM-T142, T161 and the conversion plans had no `values` entries. 11 + 1
      Playwright specs (`contingent.spec.ts`) over a stubbed API, including each light and dark theme at a desktop and a
      phone size, whose screenshots were **looked at**: the first phone layout split words ("Sho w who") and clipped a column, and was
      fixed (the three-column table fits; wide tables scroll inside their card). **Mutation-checked** four ways (withheld shown as 0;
      the masked rate not honoured; FTE sent when not stated; a refused read shown as an error), each failing a test. Full suites:
      113 vitest, 60 stubbed Playwright specs and **7 live ones against a real service**, `svelte-check` 0 errors, prettier clean, the CMS
      config current. **Follow-up (2026-10-11):** (1) **the first run against the real API found the proxy forwarded to
      `<root>/x` instead of `<root>/api/x`, so every real call was a 404**; fixed, with a unit test, and `WPM_API_URL` documented as the
      service root. (2) Requisitions can be **created in the UI** (basis, duration, programme); with no membership to choose from the
      organization URN is typed, also on the plan form (**found by the live run**). (3) The suite's order sensitivity: with a spec file
      sorted before `smoke.spec.ts`, two dashboard specs timed out at "Loading…". It was **not root-caused**: the preview server answered
      every request in under 75 ms (checked through a logging proxy), no request reached the API, and the trace shows the page idle for
      5.4 s while the theme stylesheet "received" for the same time; it affects only specs that start in the first seconds of a parallel
      run, and alone they take 0.2 s. Mitigated by a 20 s `expect` timeout (four full runs, then three, all pass with the file first); a
      real failure still fails, only later. **Not verified / not done:** the live run used enforcement **off** (`WPM_REQUIRE_AUTH=0`) and
      a fake session cookie, so the UI was not run through the real sign-in and token exchange, nor with the policy refusing a caller; the
      live data was seeded by a script that is not kept (`tests/live/README.md` says what to seed); the live suite is opt-in and not in
      CI; no native-speaker review; Arabic, Urdu and Bengali were not looked at on screen; `contingent.employed` has no plural forms.
- [x] WPM-T161 (2026-10-12) **Funding source on the engagement.** *(traces to WPM-R98, WPM-R62, WPM-R83)* The part of WPM-T109
      this needs: migration `000063` `post_fundings` (one record per worker: kind `core`, `time_limited`, `external` or
      `programme`, optional programme thing: URN, optional end date; **no amount**); `GET/PUT/DELETE /api/workers/{pid}/funding`.
      The read shows the funding **beside the contract end** and how they compare (`matched`, `contract_outlasts_funding`,
      `funding_outlasts_contract`, `funding_open_ended`, `unknown`; an unknown is never a match) with `needs_attention` for the two
      mismatches. Time-limited and external funding need an end; no end before the engagement started. HR reads and writes: the
      handler itself requires HR, because the policy decides by action only (a manager given write access was **found able to
      write it and fixed**, with the enforcement test now covering it). In the subject-access export, erased with the worker, audited
      without values. 10 unit tests, a request test and the conversion enforcement binary; **mutation-checked** (handler not
      requiring HR; comparison swapped; no erasure). **Not done:** the trend and retention sweep of WPM-T109 (the share and the funding horizon now exist); the contingent workforce view (WPM-T142); a user interface. *(traces to WPM-R98, WPM-R62, WPM-R83)* Show each fixed-term and
      contractor engagement's funding source (kind, programme, funding end) beside its contract end date in
      the worker's engagement panel and the contingent workforce view; flag a contract that outlasts its
      funding and a funding that outlasts its contract. Pure core rule and unit tests (equal, contract
      longer, funding longer, funding unknown). Depends on WPM-T109 and WPM-T137.
- [x] WPM-T162 (2026-10-12) **Pure core: conversion plans.** *(WPM-R98, WPM-D66)* `rules/conversion.rs`: the intents
      (`convert`, `extend`, `end`, `undecided`) and statuses (`proposed`, `approved`, `done`, `abandoned`); exactly four
      transitions; "nobody approves their own plan" (two unknown actors count as the same person); "a conversion needs a
      funded permanent post, or a stated reason"; "the decision date falls before the contract end" (not on it); no date
      in the past; a post whose funding ends before the conversion is refused; a reason of at most 500 characters; an
      undecided plan cannot be approved; only an approved conversion is marked done; and the flags
      `past_end_without_approved_plan` and `review_due`. 16 unit tests, one for each rule and each refusal.
- [x] WPM-T163 (2026-10-12) **Conversion plans: table, API and audit.** *(WPM-R98)* Migration `000062` creates
      `conversion_plans` (worker, intent, target date, department, role profile reference, post funding kind and end,
      reason, proposer, approver, status, review date, settled date; **no pay figure**; checks on intent, status, funding
      kind and reason length; at most one open plan per worker, enforced by a partial unique index). Routes:
      `GET/POST /api/workers/{pid}/conversion-plans`, `GET /api/conversion-plans` (open plans and contracts past their end
      with no approved plan, with flags), and `POST /api/conversion-plans/{pid}/approve|abandon|done`; documented in
      OpenAPI. **Reads are HR-only** (classified `Privileged` in `rules/access.rs`); the worker's line manager or a
      privileged caller proposes, approves and carries out, never the worker, and nobody approves their own. **Approving
      records the matching engagement decision** against the current end date in the same transaction. **Marking a
      conversion done makes the worker permanent and clears the contract end in one transaction**; the plan, the extensions
      and the decisions stay as history. Audited without the reason; in the subject-access export; erased with the worker.
      **Hardened (2026-10-12):** approve, abandon and done re-read the plan and the worker under a row lock inside the transaction, so two callers acting at once are taken one after the other (the lock is reasoned about, not proven by a test: the test pool has one connection); a second open plan is refused by the database as well as the handler, and a lost race answers 422, not 500. Verified by a request test (refusals, one open plan, the database rule, the reminder, abandon, approve, export, erase) and the new
      `enforcement_conversion` binary (reference policy plus a `decider` rule: worker, stranger and manager outside the chain
      refused; only HR reads; the proposer cannot approve; done changes the worker; the audit holds no reason).
      **Mutation-checked** five ways (a proposer may approve; done keeps the end date; reads open to all; the reason in the
      reminder; no erasure), each making a test fail. **Not verified:** a contractor converted to permanent keeps their
      contractor details and is not added to payroll by this step (those belong to the processes that own them); an
      organization above the worker's does not decide; no user interface.
- [x] WPM-T164 **Conversion plans feed the planner and the reminders.** *(WPM-R98, WPM-R63, WPM-R81)* **Done
      (2026-10-13), the planner:** each department of the forecast now carries `conversions`: the open plans to convert its people
      to permanent, **agreed** (approved by a person) and **hoped** (only proposed) kept apart, as headcount and FTE (pure
      `rules::planning::conversion_cover`, one unit test; a request test: none, proposed is a hope, approved is agreed and no longer
      a hope, an abandoned plan counts for nothing, the projection is unchanged, the forecast names no one). Neither changes the
      projected supply (an agreed conversion keeps a worker beyond their end date, so the projection understates it; the planner
      decides). The derivation text says so. **Done (2026-10-12):** the end-of-engagement reminder (WPM-T138) names an open plan's status and review date, never its
      reason (verified, mutation-checked); approving a plan records the decision a manager takes; a done conversion changes
      the worker in one transaction; `GET /api/conversion-plans` lists open plans apart from the candidates. **Done (2026-10-18), the UI:** the planning page shows agreed and hoped conversions
      beside the levers in each forecast row, apart and with a note that neither changes the projected supply (an unknown FTE is
      not shown as 0); the worker page has a conversion panel (`ConversionPanel`) that lists the plan and its history, proposes,
      approves, abandons and carries out (the buttons follow the status: approve only a proposed plan, carry out only an approved
      conversion, no form while a plan is open) and shows the service's refusal; the contingent page has a "Conversion plans" list
      of open plans and engagements past their end with none, loaded only on request (the service audits the read), naming workers
      by number and never showing a reason. A caller the service refuses sees no panel and no error. Client functions and types in
      `wpm.ts` (unit-tested paths); 26 keys in `conversion.*`, 4 in `plan.*` and 2 flag values in all 13 locales (AI-written, unreviewed).
      Verified: svelte-check 0 errors; 145 unit tests; 134 Playwright specs, four new (propose, approve and carry out with the
      buttons following status; a refusal and a 403; the contingent list on request with no reason in it) and a planning assertion,
      each **mutation-checked** (approve shown on any status; hoped showing the agreed count; the reason leaking into the list), and I
      looked at screenshots of the panel with no plan and with an approved one. **Run against the real service** (current build,
      fresh database, `scripts/demo-data.py`, sign-in enforcement off, 2026-10-18): the panel showed the demo's proposed plan; approve
      and carry out both returned 200 and the worker became permanent with no end date; the planner's engineering row showed hoped 1
      (1 FTE) as the forecast returned; the contingent list named the engagement with its plan; no HTTP error or page error. That run
      found a missing space in the history line, fixed. **Not verified:** a signed-in non-HR caller (enforcement was off, so the 403 path
      never ran for real); a manager who is not HR proposing through the UI against the real service (the service side is covered: `tests/enforcement_conversion.rs`, run on 2026-10-18 with enforcement on, shows the line manager's proposal accepted, the same manager and the worker refused the plan read with 403, and a manager outside the chain refused the proposal; only the browser path is not run; the panel's propose-only mode, shown when the plan read is refused
      with 403 and the viewer is not the worker, was tested against a stub that refuses the read: the proposal is sent, nothing of the plan is
      shown, the service's refusal is shown; the three behaviours were **mutation-checked** — the 403 handling, the sent notice and the
      worker-themselves guard, the last only after a test was added for it). Read from the code, not run: a 403 on the plan read comes only from
      the need-to-know guard (`workers/{}/conversion-plans` is `Privileged` in `src/rules/access.rs`); the handler itself answers 404 for a
      worker outside the caller's scope and the session layer 401, so a 403 there means "not privileged"; the panel in a locale other than Arabic, Urdu and English on screen (the worker-page panel, with a plan and a flag, was looked at in Arabic at desktop width and in Urdu and English at 375px: right to left mirrored, no horizontal overflow; the propose form, the planner row and the contingent list were looked at in Urdu at 375px against the real service and none overflows sideways, the same three in Arabic checked for overflow only; the planner's row label wraps to a word a line there, as the table's existing "by basis" row does (a wide table on a phone, left alone));
      the planning row and the contingent list in a screenshot; the reminder text on the worker page.

## Phase 15 — operations, information governance, OIDC and locales (WPM-R89–R94, WPM-D60–D64)

Specified in [operations-and-governance.md](operations-and-governance.md), 2026-10-10.
WPM-T146 supersedes the proposed WPM-T129–T131 (backup plan, erasure ledger, impact
assessment), which stay listed in Phase 13 as superseded.

- [x] WPM-T146 (2026-10-10) **Spec round.** *(traces to WPM-R89–R94, WPM-D60–D64)* The topic file
      [operations-and-governance.md](operations-and-governance.md), this phase, the index,
      `llms.txt`/`llms.json`, requirements and design pointers, next free ids (R95, D65, T156).
      No code.
- [x] WPM-T147 (2026-10-10) **Retention schedule per kind.** *(traces to WPM-R91, WPM-D60)* Pure
      `rules::privacy`: `RecordKind` (eight kinds), `kind_of` (every one of the 56 swept tables in
      exactly one kind, pinned), a default horizon per kind (180, 2190, 2190, 730, 730, 1095, 365,
      365 calendar days), `horizon_for` (own override, then legacy `WPM_RETENTION_DAYS`, then default,
      floored at 30, junk ignored). `GET /api/retention/schedule`; the report and the sweep use each
      table's own horizon; candidates' expired consent uses the recruitment horizon.
      **Changed contract:** `GET /api/retention` and the sweep return `horizons` per kind in place of
      `horizon_days`. Verified: 5 new unit tests; a new binary `tests/retention_schedule.rs` (one
      override set, one junk override ignored; rows deleted 150 and 200 calendar days ago are
      swept for the 100-calendar-day kind and kept or swept correctly for the 180-calendar-day kind); the existing
      privacy request test updated. **Not mutation-checked.** The default horizons are the
      assistant's cautious guesses, to be replaced by the deployer (the schedule says so).
- [x] WPM-T148 (2026-10-10) **Erasure ledger and replay.** *(traces to WPM-R89, WPM-D63)* Migration 51
      `erasure_ledger` (pid, time). The erasure path writes it in the same transaction, and the
      scrub statements moved into one function (`erasure_statements`) used by both the live
      erasure and the replay, so they cannot drift. `replay_erasures` (library) and the loco task
      `replay_erasures [since:YYYY-MM-DD] [file:PATH]`; `file:` imports an exported ledger, and
      a malformed line stops the import (a skipped erasure would be silent). Verified: unit tests
      for the file parser (including SQL in a field); a request test that erases, "restores" the
      person and their note by raw SQL, replays, and checks they are gone again, that a second
      replay is idempotent, that an unknown pid is counted not an error, that `since` limits it,
      and that the replay is audited; and the restore drill (T149) end to end.
      **Design flaw found and fixed on the way:** the ledger lives in the database a restore
      replaces, so an erasure made after the last dump would be lost with it. Hence
      `scripts/export-erasure-ledger.sh` and the `file:` option (WPM-D63). **Not done:**
      point-in-time recovery with WAL, which would make the ledger complete without the file.
- [x] WPM-T149 (2026-10-10) **Backup, restore and the drill.** *(traces to WPM-R89)* `scripts/backup.sh`
      (custom-format `pg_dump`, owner-only, checksum, prune by `BACKUP_RETENTION_DAYS`),
      `scripts/restore.sh` (checksum, refuses a non-empty target, refuses to start without
      `WPM_BIN` unless `--skip-replay`, replays erasures), `scripts/export-erasure-ledger.sh`,
      `scripts/restore-drill.sh`, and the `restore-drill` job in CI. Run locally against
      PostgreSQL 18: migrate and seed, back up, erase a worker after the backup, export the
      ledger, restore into an empty database, **97 tables match by row checksum**, **51
      migrations recorded**, the erased person is back, the replay erases them again, the real
      service on the restored database answers (39 workers, the erased one a 404): **passed**,
      restore step under a second on the seeded demo. Safety paths checked by hand: a
      non-empty target is refused (exit 3), a corrupted dump fails its checksum, a missing
      `WPM_BIN` stops **before** anything is restored. **Not verified:** the CI job (never run
      on the hosting platform; it installs the PostgreSQL 18 client from the vendor repository);
      the drill on a database of real size; WAL archiving. It is a smoke test of restored data,
      **not** the full request suite, which truncates its database on boot.
- [x] WPM-T150 (2026-10-10) **Operations documents.** *(traces to WPM-R89, WPM-R90)* `spec/operations/`:
      [backup-and-restore.md](operations/backup-and-restore.md) (RPO and RTO in two tiers: nightly
      dump 24 hours, with WAL archiving 15 minutes; RTO 4 hours; what is and is not proven),
      [deployment.md](operations/deployment.md) (every environment variable, probes, scheduled
      tasks, proxy and TLS), [upgrade-and-migration.md](operations/upgrade-and-migration.md)
      (forward-only, back up first, the way back is a restore) and
      [runbook.md](operations/runbook.md). **Not done:** a Helm chart (the compose file and the
      Containerfile are the deployment artifacts, and neither has been built where this was
      written); the RTO is a target, not a measurement on real data.
- [x] WPM-T151 (2026-10-10) **Information governance pack.** *(traces to WPM-R92, WPM-D61)*
      `spec/governance/`: [dpia.md](governance/dpia.md) (template completed for the demo, twelve
      risks with the evidence for each measure), [data-flow.md](governance/data-flow.md)
      (a Mermaid diagram and the table behind it),
      [record-of-processing.md](governance/record-of-processing.md),
      [retention-schedule.md](governance/retention-schedule.md),
      [assessment-checklist.md](governance/assessment-checklist.md) (46 questions in six sections;
      the count of answers is generated into the status page), [accessibility-statement.md](governance/accessibility-statement.md)
      (says plainly that the software has **not** been audited, with no skip link and no automated
      accessibility test) and [official-language-statement.md](governance/official-language-statement.md).
      **Not done:** a deployer cannot "complete" it from this alone: lawful bases, the data
      protection officer, processors, the privacy notice and every sign-off are theirs, and
      are marked so. No one outside the project has reviewed the pack.
      **Found while writing it:** the content manager at `/admin` loads an unpinned script from
      a public CDN with no integrity check (recorded as a gap), and the new
      `Cross-Origin-Opener-Policy` would have broken its sign-in popup (fixed: the admin path
      gets `same-origin-allow-popups`, tested).
- [x] WPM-T152 (2026-10-10) **Sign-in with Entra ID.** *(traces to WPM-R93, WPM-D62)* The OIDC backend now
      reads Entra's top-level `roles` claim as well as `realm_access.roles`, and takes the
      subject from `oid` when `WPM_OIDC_SUBJECT_CLAIM=oid` (a token with no `oid` is refused,
      not given its `sub`). [operations/entra-sign-in.md](operations/entra-sign-in.md) documents
      direct and brokered sign-in, the app roles to personas, the settings, and the limits.
      Verified: 3 new unit tests; **`tests/oidc_entra.rs`**, which runs an Entra-compatible OIDC
      provider in the test process (discovery, keys, authorization endpoint, token endpoint with
      PKCE and single-use codes), signs two users in by the authorization-code flow, boots the
      service from the discovery document with sign-in left at its default, and checks a user
      with no role reads but cannot write, `wpm-hr` can write, and a wrong audience, another
      tenant, an expired token, a token signed by another key and an HS256 confusion token are
      each a 401; **mutation-checked** (ignoring `roles` fails the live and the unit tests).
      Clippy clean under the `keycloak` feature; a new CI job. **Not tested:** the interactive
      browser redirect, consent and MFA (the broker's); a live Entra tenant (the provider
      follows the documented token shape and was not captured from one); the Keycloak brokering
      route; group claims. Two throwaway RSA keys are committed under `tests/fixtures/oidc/`
      (a secret scanner may flag them; the README there says why it should not).
- [x] WPM-T153 (2026-10-10) **Move all page text into the catalogue.** *(traces to WPM-R94)* A scanner-based
      transformer (tokenizes the template, respecting tags, quoted attributes and `{expressions}`)
      moved every literal heading, label, button, placeholder, title and message of **41 files** (the
      pages and the components) into `pages.<route>.*` and `comp.<component>.*`, with `{name}`
      placeholders filled by the new `tf()`; the 22 per-route `title` values in `+page.ts` and
      `+page.server.ts` became `titles.*` read through `pageTitle()` (locale from the address, since a
      `load` also runs on the server); about 45 places that printed a server token raw
      (`on_leave`, `ai_assistance`, a status, a category) now go through `tv()` and **199 `values.*`
      entries**; and a few strings in script blocks (notices, kind labels, "Other profiles", sign-in
      link errors) were converted by hand. 710 new keys in all (1,251 in total). Verified: svelte-check
      0 errors; the English output is unchanged (47 Playwright specs, the existing ones untouched
      except for the pay-scale stub names). **Not done / limits:** the **server's own messages** (an
      error the API returns) are shown as the server sends them, in English; **names, titles and
      anything a person typed** are shown as entered; a **token the catalogue does not know** is
      shown humanized in English (the test below pins that every token the server's vocabularies
      define is known, but a status the server computes as a plain string outside those lists, such
      as a registration's `valid`, was added by hand and others may exist); **dates and numbers**
      are shown in ISO form, not localized; the **content manager** at `/admin` is a third-party
      tool with its own languages; plural forms are written "skill(s)" because the catalogue has no
      plural rules.
- [x] WPM-T154 (2026-10-10) **Translate, and test for the untranslated.** *(traces to WPM-R94, WPM-D64)* The 611
      unique new English strings were translated into the 12 other `-001` locales (Arabic, Bengali,
      Welsh, German, Spanish, French, Hindi, Indonesian, Portuguese, Russian, Urdu, Chinese) by the
      AI assistant, validated by a script (every index present once, placeholders identical to the
      English, nothing empty, nothing identical to English unless allowed) and merged in the
      catalogue's own order with only additions to each file. New tests
      (`tests/unit/untranslated.test.ts`): **no literal visible text in any template** (zero found;
      **mutation-checked**: a literal `<p>` added to a page fails it); **no locale repeats English**
      except an allow-list with a stated reason (`tests/unit/untranslated.allow.json`: 4 brands or
      acronyms for every locale, and 1 to 23 cognates or loanwords for ten languages; mutation-checked:
      a repeated English value fails); **every token in the server's Rust vocabularies has a
      `values.*` entry**. Locale parity (existing) passes; 101 unit tests in all. A new Playwright spec
      loads `/cy-001/mentorship` and checks its headings are Welsh and none of the English survives.
      **Not verified:** the quality of any translation (see T155); pages other than the Welsh
      mentorship page were not opened in another language; right-to-left layout of the new strings
      (Arabic, Urdu) was not looked at.
- [ ] WPM-T155 **Professional review of the translations.** *(traces to WPM-R94)* Not done, and cannot be done by
      the assistant. `workforce-planning-management-ui-with-svelte/content/locales/REVIEW.md` records every
      locale as AI-written and not reviewed, with the steps for a reviewer. The allow-list is the first
      place to look: each entry is a word the assistant left as English or a cognate a native speaker
      should confirm. One weak spot seen while reading the older keys: German "Band"
      (a pay band) is probably not what a native speaker would choose.
## Phase 16 — issues register and change requests (proposed WPM-R95–R97, WPM-D65)

Proposed 2026-10-10, not started. They are owned by project-portfolio-management under WPM-D55 and
WPM-D65; the requirement text is in [ppm-issues-and-changes.md](ppm-issues-and-changes.md). WPM's own
tasks are the capacity answer and the contract, below.

- [~] WPM-T156 **Spec round.** *(traces to WPM-R95–R97, WPM-D65)* This phase, the topic file, the index,
      `llms.txt`/`llms.json`, requirements and design pointers, next free ids. Done as text on
      2026-10-10 except the pointers in `index.md`, `llms.txt` and `llms.json` (tracked here).
- [ ] WPM-T157 **Hand the requirements to project-portfolio-management.** *(WPM-R95, WPM-R96)* Copy
      [ppm-issues-and-changes.md](ppm-issues-and-changes.md) into that repository as its FR-14 and the
      change-request requirement, agree the call that WPM answers (programme URN plus proposed deltas),
      and record the agreed contract here. **Outside this repository**: it is not done until that
      repository holds them.
- [x] (2026-10-28) WPM-T158 **What-if capacity answer.** *(WPM-R97, WPM-D46, WPM-D50)* Pure `rules::capacity` addition: given
      the current demand and a set of proposed FTE deltas per pool per month, return the pool-months
      that become or stop being over-committed, the constraint pool before and after, and the
      earliest fitting start; `unknown` where supply or partner commitments are unknown. A
      `POST /api/capacity/what-if` that reads, computes and **stores nothing**. Depends on the
      capacity model (WPM-T104–T107). Unit tests (a delta that fits, one that breaks a pool-month,
      one in an unknown month) and a request test.
      **Done (2026-10-28):** `rules::capacity::{what_if, change_problems, Change, WhatIf, Answer}` over the existing pool views (the cell rule is now one function, `classify`, shared by the view and the what-if):
      a proposal of changes (pool, month, centi-FTE, negative to give back) is applied to the programme's **active** claim, never below what the programme holds; the answer lists only the touched pool-months
      (before and after status and balance), counts those that become or stop being over-committed, names the constraint pool before and after, and finds the **earliest start that fits** by moving the
      whole proposal later a month at a time (an unknown month is not a fit). `unknown` where a partner has not answered; with **no skill pools at all** the answer is `unknown` with its reason, never "no impact".
      `POST /api/capacity/what-if` reads the view, computes and **stores nothing** (`stored: false`); the audit entry holds counts and the answer, never a figure or the programme. 11 unit tests (a change that fits,
      one that breaks a pool-month, one in an unknown month, giving capacity back, the programme floor, not-worse-than-before, the earliest start, none within the horizon, validation, the view unchanged, and the
      view and what-if agreeing on a cell) and a request test (fits; breaks and the constraint pool; the earliest start; giving back more than held; nothing stored; no name, number, partner or other programme
      in the answer; five refusals; an organization with no pools; the audit trail). **Mutation-checked** eight ways (the worse-than-before rule; the programme floor, first with a test that could not tell it from
      the zero clamp, now fixed; an unknown month counted as a fit, likewise; the programme filter; the empty-organization branch), all caught. `cargo test --lib` 991, full `--ignored` suite 176, clippy clean.
      **Not done:** the contingent effect (T159); the proposal's horizon starts at its earliest month and spans at most 24 months; programme demand is read from claims with status `active` (a claim becomes
      active only through a start decision); no UI; PPM's side of the call is unverified.
- [x] (2026-10-28) WPM-T159 **Contingent effect in the what-if answer.** *(WPM-R97)* Where a proposed change moves a
      programme's end date, list the fixed-term and contractor engagements (counts per pool, no names)
      that would end before it. Depends on WPM-T135 and WPM-T142.
      **Done (2026-10-28):** `POST /api/capacity/what-if` takes an optional `programme_new_end`; the answer then carries `contingent_ending`, per pool, the number of fixed-term and contractor
      engagements still running (not terminated, with an end date on or after today) that end **before** it (`rules::capacity::ends_before_programme`; one ending on the new end date itself covers it).
      A pool is matched by the same role-profile and skill test as supply (`holds_sql`, shared with the capacity view). A count under the small-group floor of 5 (`K_ANONYMITY`) is `null`, zero is
      shown, and **there is no total** (a worker can sit in more than one pool, so a total would double-count). Interns are not counted, nor are other organizations' workers. Names no one; the dates
      are read for the count and go no further. Unit tests (the boundaries; the floor, zero and no total) and a request test (counts, floor, a later end, no name, none when no end is given, a bad date);
      **mutation-checked** seven ways (past engagements, the end-date boundary, the floor, the basis filter, the terminated filter, the organization scope), all caught, three after the test gained
      an intern, a terminated worker and another organization's worker that it had lacked. **Not done:** it counts engagements by their own end date, not by the days the programme actually holds them.
- [x] (2026-10-29) WPM-T160 **Verification and documents.** *(WPM-R97)* Enforcement test that the what-if answer names no
      one and that a manager sees counts only; update the DPIA (`spec/governance/dpia.md`), the record of
      processing and the data-flow diagram if WPM stores anything new (it should not).
      **Part done (2026-10-28):** a request test checks the answer names no one (no worker, number, name, partner or other programme) and stores nothing (the same rows before and after); the DPIA and the
      record of processing need no new row because the what-if holds nothing and carries no personal data, and the data-flow diagram is unchanged. **Not done:** the persona enforcement test ("a manager sees counts
      only"): the endpoint answers every caller in the organization's scope with the same pool-month figures, and no persona-matrix test was written; whether that is the right audience is a decision for the
      product owner, because pool supply and demand are planning figures and not a person's data.
      **Finished (2026-10-29):** `tests/enforcement_what_if.rs` (own binary, `--ignored`) runs the persona matrix against the live route with sign-in on: signed out 401; the service, HR and a planner
      (a deployer rule added in the test) 200 with the **same payload**; an ordinary manager 403 (the policy treats a POST as a write and gives a manager none); a partner-only caller 403; another
      organization's HR and an HR caller with no membership 404. Every answer is walked for people keys (`worker_pid(s)`, `person_ref`, `display_name`, `worker_number`, `workers`, `people`, `names`, `email`) and
      for seeded names and the other programme's reference, and `members` must be a bare count; the capacity view is walked the same way for a manager, HR and a planner (200) and refused for a partner (403) and
      another organization (404). **Audience recorded as WPM-D142, proposed, needs product-owner confirmation:** nothing is widened; a manager cannot ask the what-if unless a deployer's policy lets them
      write, and a read-class exception would be a new decision. **Mutation-checked** two ways, both caught (a `worker_pids` key added to the answer; the organization scope check removed). Verified: the new
      test, `cargo test --test enforcement` and `--test mod capacity` (15) against a throwaway Postgres 18, `cargo test --lib` 997, clippy clean. **Not verified:** the peer-supply endpoint is not in the
      matrix (it is read-open like the view and was not walked); no Keycloak or Entra token shapes; the full `--ignored` suite was not rerun.

## Phase 17 — equality impact for scoring (proposed WPM-R99–R101, WPM-D67)

Specified in [plan.md](plan.md) section I. The template and the register of today's scores are in
[equality-impact/](equality-impact/template.md).

- [x] WPM-T165 (2026-10-10) **Template and register.** *(traces to WPM-R99, WPM-D67)* The template
      (`spec/equality-impact/template.md`, thirteen sections) and the register with an assessment for each
      score WPM has today: succession readiness, pipeline stage, assessment band, review rating and mean,
      internal-mobility fit, skill-gap priority, change readiness and insight flags; and the recorded
      decision **not** to build an individual risk-of-loss score. **They are first drafts by the AI
      assistant; no one with equality or legal responsibility has reviewed or approved any of them**, and
      the demo has no real data, so the evidence sections say "not yet measurable". A claim that the
      software tells raters what not to use was found untrue while writing and corrected: that is T167.
- [~] (2026-10-27) WPM-T166 **Scoring register in code.** *(WPM-R99)* `rules::scoring::REGISTER` (id, kind, inputs, the
      decision informed, assessment file, status, review date) and a test that every registered score has
      an assessment with all thirteen sections, a status that is approved or conditional, and a review
      date not yet passed; a lint test that names a new `pub fn` in `rules/` containing score, rank,
      rating, band, readiness or risk that is not in the register.
      **Done in part (2026-10-27):** `rules::scoring::{REGISTER, Entry, Kind, Status, ready_for_use, unassessed, find}`: S1 to S8 from the register document plus **S10**, a
      stated skill level counted on WPM's 1–5 (also added to `spec/equality-impact/register.md`); S9 stays unbuilt and a test says so. Tests (8): ids unique; every AI-register
      feature (S1 to S8) is registered; each assessment names a section that exists in `register.md` and its status agrees with that section's text; a score with no assessment
      (S10) is named and is never ready; the gate needs an approved or conditional status **and** a review date set and not passed; no score is ready today; and the lint
      (a new `pub fn` in `src/rules/` whose name has score, rank, rating, band, readiness or risk must be registered or exempt with a reason; seven exemptions, each with its
      reason, and a test that none is stale). **Mutation-checked:** a probe `pub fn flight_risk_score` made the lint fail; removed, it passes.
      **Not as the task is worded, and why:** (1) the task wants every registered score to have an assessment with **all thirteen sections** and a status that is
      **approved or conditional**; neither is true: the assessments are short drafts in `register.md` (not thirteen-section files) and **no one has approved any of them**, so
      the register says `Draft` and `ready_for_use` is closed for every score. I did not mark anything approved to satisfy the wording. Writing the thirteen sections and
      getting an approval are the deployer's and an equality lead's work; (2) S10 has no assessment yet; (3) `review_by` is unset everywhere for the same reason. **Verified:**
      `cargo test --lib` and `cargo clippy --all-targets -- -D warnings` (see below for the last full run).

- [ ] WPM-T167 **Scores say what they are.** *(WPM-R101)* Add to every score payload and screen: its inputs, "advisory;
      a person decides" and the assessment id; put the "do not use absence, leave, working pattern or
      adjustments" guidance beside the succession rating, the pipeline stage, the review rating and the
      assessment band; strings in the 13 locales under the catalogue rules (WPM-R94).
- [ ] WPM-T168 **Adverse-impact check.** *(WPM-R100)* Pure core: favourable-outcome rate per group, ratio to
      the best-treated group, small groups withheld, input missing treated as unknown; an endpoint taking
      a deployer-supplied grouping and **storing nothing**, HR-only and audited without the grouping.
      Unit tests (a clear gap, equal rates, a small group, an empty group) and a request test that
      nothing is persisted.
- [ ] WPM-T169 **Guards.** *(WPM-R99, WPM-D67, WPM-D68)* A test with a deny-list of fields that must never be inputs to
      any registered score (protected characteristics, union membership or activity, sickness absence,
      leave, adjustments, postcode); a test that no `risk_of_loss`, `flight_risk` or similar individual
      measure exists in the code or API; both mutation-checked.

## Phase 18 — worker unions (proposed WPM-R102–R108, WPM-D68)

Researched in [worker-unions.md](worker-unions.md); specified in [plan.md](plan.md) section J.

- [x] WPM-T170 (2026-10-10) **Concept note.** *(traces to WPM-R102–R108, WPM-D68)* `spec/worker-unions.md`: what unions, recognition,
      collective agreements, works councils and representatives are; which planning acts trigger
      collective duties (the Great Britain and EU consultation rules as examples, with sources); why
      membership is special category data and how a blacklisting history shapes the design; seven
      proposed capabilities and what is deliberately left out. **Researched from web sources with
      the limits stated in the note, partly from the assistant's own knowledge marked unverified; no
      employment lawyer or union has reviewed it.**
- [x] (2026-10-26) WPM-T171 **Guard first.** *(WPM-D68)* Tests that fail if any table, column, export, audit entry, label or time
      kind could reveal membership or representative status; update the impact assessment, record of
      processing and data-flow diagram to say WPM holds none and where a deployer's own check-off sits.
      Done: `tests/no_union_membership.rs`, DB-free. It reads the service and migration sources, the config, the API document and the English
      catalogue and fails on a name that would hold membership or representative status (`union_member*`, `trade_union*`, `steward`, `check_off`,
      `is_representative`, `union_activity` and the like, as whole identifiers or parts of one) or a phrase that would say it ("union member", "shop
      steward", "check-off", "representative status"...). `check_offset` and "Trade Union and Labour Relations Act" pass. DPIA R15, the data-flow
      "does not flow" list and the record of processing now say WPM holds none and that check-off stays in the deployer's payroll. **Verified:** 4 tests
      (the guard flags and passes the right examples; it reads the files it claims to); two **mutation checks** (a `union_member_id` name in a rules file,
      and "Shop steward" in the English catalogue: each failed the right test, then restored). **Not verified:** the deny-list is a floor, not a proof
      (a field that reveals membership under another name passes; a reviewer must catch it); locales other than English are not scanned; no time
      kind, leave kind or deduction label list exists yet to check by value.
- [x] (2026-10-26) WPM-T172 **Pure core: consultation obligations.** *(WPM-R104)* Thresholds and periods as **configuration with a
      named source** (a Great Britain profile and an EU-directive profile as examples, marked as the
      deployer's to confirm); from a proposal (headcount reduction at an establishment, planned
      termination date) say whether a duty may apply, the latest start for consultation, and the checklist of
      notices; the period counted in calendar days. Unit tests at each threshold edge, the 90-calendar-day window,
      and a proposal that crosses it.
      Done: `rules::consultation` (pure, no clock: `today` is an input). A `Profile` is data with a named source: `great_britain()` (20 in 90 days;
      consultation and authority notice 30 days for 20–99 and 45 for 100 or more) and `eu_directive_98_59()` (10 / 10% rounded up / 30 by
      establishment size over 30 days, or 20 in 90 days at any size; notice to the authority 30 days ahead; no invented fixed consultation
      period), each marked for the deployer to confirm. `assess(profile, proposal, today)` returns a suggestion with its evidence (counted against
      needed, window), the latest start and notice dates, days remaining, whether late, the checklist as tokens, `deployer_to_confirm: true`;
      a count under every threshold is `may_apply: false`, not "clear", with no dates and no checklist. Dismissals count together in the best
      window that includes the proposal's date, never summed across windows. An agreed procedure only lengthens the period. A proposal is counts
      and dates; nothing names a person. **Verified:** 10 unit tests (each threshold edge, the 89/90-day boundary both ways, windows not summed, the
      percentage rounded up, the agreed period, lateness, extreme values); two **mutation checks** (`>=` made `>`, and the window length off by one:
      both failed tests, then restored); clippy pedantic clean. **Not verified:** the thresholds and periods against the statutes (taken from the
      concept note's sources and the assistant's own knowledge of the directive, no lawyer has read them); nothing calls it yet (no endpoint,
      no table: T173–T175 do that). **Note:** the phase lists T171 (the guard tests) first; this was done ahead of it, at the user's choice, and T171 is still open.
- [x] (2026-10-26) WPM-T173 **Recognised bodies and bargaining units.** *(WPM-R102)* Tables and API for bodies and units (rules over
      department, role profile, location, employment basis, grade); the coverage count as a floored
      aggregate; contact is a role. Request tests and enforcement for who may edit.
      Done (service only): migration 92 `recognised_bodies` (name, kind union/works_council/staff_forum, basis, recognised_on, `contact_role`) and `bargaining_units`
      (name and rules over department, location, employment_type, job_title; at least one, enforced by a CHECK); no column for a member, a representative,
      a contact person or who is covered. `rules::bargaining` (pure: validation, `covers`, `coverage`: shown from the floor, withheld under it, `no_one` for a
      rule that matches nobody). `GET/POST /api/recognised-bodies`, `GET/POST /api/recognised-bodies/{pid}/units` (HR; audited without names; in the
      need-to-know table, the OpenAPI document and the UI's copy of it). **Verified:** 6 unit tests; a request test against Postgres 18 (twelve carers in Leeds,
      three in York, a nurse: case and edge spaces do not matter, three and one are withheld, a rule matching nobody is `no_one`, a leaver and a deleted record
      stop being counted, the answer names no worker or number); an enforcement test with real tokens (anonymous 401, a worker 403 on every route, HR 200);
      `route_coverage`; the T171 guard still passes; two **mutation checks** (the floor off by one, caught by the unit test; the controller's floor set to 0,
      caught by the request test) and a third (a location rule matching a worker with no location, caught); clippy pedantic; the full `--ignored` suite (156).
      **Not verified:** no UI (screens belong with T178); **role profile and grade are not unit rules** because the worker record holds neither (only
      department, location, employment type and job title), so the spec's list is narrower here; no way to edit or retire a body or unit; coverage reads the
      four facts of every worker in the organization per request (no 200k timing taken); the floor is the equality-monitoring default (10), not set per
      body; a difference between two units' counts can still hint at a few people (a floor stops a count, not every comparison).
- [x] (2026-10-26) WPM-T174 **Collective agreements as reference data.** *(WPM-R103)* Dated, sourced, versioned agreements linked to a
      unit and to pay-scale and working-time references; "applies to a worker" derived, HR and payroll only,
      never shown as membership. Tests.
      Done (service only): migration 93 `collective_agreements` (title, version, effective_from/to, `source`, and four optional terms: a pay scale id the
      service knows, max weekly hours 1–168, notice weeks 0–104, an agreed consultation period 1–365 days), tied to a bargaining unit, with no column for who it
      applies to. `rules::agreement` (pure: validation, a new version must take effect after the newest, `in_force` = the newest version whose dates include the
      day). `POST/GET /api/bargaining-units/{pid}/agreements` (HR; the same title is a new version) and `GET /api/workers/{pid}/collective-agreements?on=`
      (HR and payroll; **derived** from the unit rules over the worker's department, location, employment type and job title; audited in a transaction; the answer
      names no body, unit or membership and says so). **Verified:** 4 unit tests; a request test against Postgres 18 (versioning, a lapsed agreement, a
      version in force today and on an earlier day, none before the first, a worker the rules do not cover gets none, nothing in the answer names the body, the
      unit or "member"); an enforcement test with real tokens (anonymous 401, a worker 403 on all three routes including their own, HR 200, payroll 200 on the
      worker read); `route_coverage`; the T171 guard; **three mutation checks** (`>=` made `>` on the last day, "covers" made always true, "in force" inverted:
      each caught); clippy pedantic; the audit-in-transaction test caught my first version of the audit write, and it is fixed; full `--ignored` suite (157).
      **Not verified:** nothing reads `consultation_days` yet (T172's `agreed_period_days` is not fed from it: that wiring belongs with T175); working-time
      limits are stored but not applied to the working-time guardrails; `max_weekly_hours` and the pay scale are not checked against each other or against
      pay; no UI (T178); no way to retire or correct an agreement except by a new version; a worker covered by units of two bodies gets both agreements.
- [x] (2026-10-26) WPM-T175 **Consultation register and timeline.** *(WPM-R105)* Per proposal: body, dates, topics, document references,
      outcome; roles not names; audited without text; a timeline view that uses T172.
      Done (service only): migration 94 `consultations` (the body, an optional plan of the organization, the establishment, the threshold profile, size,
      dismissals and earliest termination as counts and dates, topics from a closed list, one document reference with no spaces, begun and ended days, the
      outcome open/agreed/no_agreement/withdrawn, an optional agreement; CHECKs tie "open" to "no end day" and an agreement to "agreed"). **No column for an
      attendee, a representative or what was said.** `rules::consultation_register` (pure: topics cleaned, opening, beginning once, closing, and `timeliness`
      against the latest start: nothing due, no fixed period, not begun with days left, not begun and late, begun in time (the last day counts), begun late).
      `GET/POST /api/consultations`, `POST .../{pid}/begin`, `POST .../{pid}/close`, `GET .../{pid}/timeline` (HR; writes audited without text). The timeline
      runs `rules::consultation::assess` over the record, with the other proposals at the same establishment counted into the window and the longest
      `consultation_days` of the body's agreements in force today as the agreed period, so T174's term is now used. **Verified:** 12 unit tests; a request
      test against Postgres 18 (bad profile, topic, reference, dismissals, plan and body; topics cleaned and ordered; a 60-day agreed period beating the law's
      30; 25 and 5 at one site counted as 30 while a proposal at another site is not; under the threshold nothing is due; begin once and not in the future;
      closing with a foreign agreement, a mismatched outcome, an unknown outcome and a future day all refused; closed means closed; never begun can only be
      withdrawn; the register holds no attendee, representative, minute or member); an enforcement test with real tokens (anonymous 401, a worker 403 on
      list, open, begin, close and timeline, HR 200); `route_coverage`; the T171 guard; **three mutation checks** (the last-day boundary, the establishment
      filter, the agreed period dropped: each caught); clippy pedantic; the full `--ignored` suite (158). **Not verified:** no UI (T178); the thresholds are
      still the deployer's to confirm (T172); a record cannot be corrected, only closed and a new one opened; the proposal is typed in, not read from a
      workforce plan's demand lines (`plan_pid` is only a link); the authority notice is shown as a date, not tracked as given; no reminder when a latest day
      approaches.
- [x] (2026-10-26) WPM-T176 **Information pack for bargaining.** *(WPM-R106)* On-request aggregates for a unit with the floor, no names, no
      individual pay; each disclosure logged. Tests that a small group is withheld.
      Done (service only): `rules::bargaining_pack` (pure) and `GET /api/bargaining-units/{pid}/information-pack` (HR). For the people the unit's rules cover: headcount and
      FTE by department and by employment type, the pay bands people sit in (no band recorded is its own group, never zero), starters and leavers in the 365 days to
      today, those on a basis other than permanent, and open vacancy posts (a unit defined by location or employment type cannot be matched to a vacancy, and says
      `unknown`). Every count is shown from the floor (10) or withheld; **when exactly one group of a breakdown is withheld the smallest shown one is withheld too**,
      because the total would otherwise give it away; a unit under the floor gives nothing at all. No name, no number, no salary, no one's band, no pid.
      **Reading it is the disclosure and is logged** (`information_pack_disclosed`: who, which unit, when) in a transaction. **Verified:** 8 unit tests; a request
      test against Postgres 18 (25 people, 3 of them contractors: both groups withheld; 12 in band B5 and 13 with none, both shown; 4 open posts; a unit of four gives
      nothing; the location unit says `unknown`; nothing in the pack names a worker, number, salary, or the body; exactly one disclosure logged); an enforcement test
      with real tokens (anonymous 401, a worker 403, HR 200); `route_coverage`; the T171 guard; **three mutation checks** (secondary suppression off, the whole-unit
      floor off, the disclosure not logged under its name: each caught); clippy pedantic; the full `--ignored` suite (159). **Not verified:** no UI (T178); no
      monthly schedule (on request only; "monthly" in WPM-R106 is not built); no read of the disclosure log through the API; complementary suppression is
      within one breakdown, not across breakdowns or across two reads of nearby days (someone who asks on consecutive days and compares can still infer a change);
      the facts of every worker in the organization are read per request (no 200k timing); starters and leavers are matched by the worker's current department
      and title, not the ones they had then; the pack gives FTE per pay band, which is a count of people times their hours, not pay.
- [x] (2026-10-26) WPM-T177 **Facility time as an aggregate; decision gates.** *(WPM-R107, WPM-R108)* Hours per body per period (no per-person
      record); a configurable list of changes that need consultation, shown when someone makes such a change,
      including a new or changed score (link to the equality impact template, section 11).
      Done (service only): migration `m20261026_000095` (`facility_time_periods`: minutes per body per period, optional budget, no column for a person;
      `decision_gates`: change kind + body, unique per organization), `rules::decision_gate` (pure), `controllers::decision_gates` (`GET/POST /api/decision-gates`,
      `GET /api/decision-gates/check`) and facility-time routes in `controllers::bargaining`. Recording an AI-register feature (`PUT /api/ai-register/{feature}`)
      now returns a `decision_gate` pointer for `new_or_changed_score`: the bodies to consult and the equality impact template (section 11), asking for a
      `consultation_pid` and **advising, never blocking**; a reference that is no record is refused (422). **Verified:** 5 new unit tests (860 in all);
      request tests against Postgres 18 for facility time and for the gate (the full `--ignored` suite, 161); enforcement tests with real tokens; the T171 guard
      (5); clippy pedantic clean on the new files. **Not verified:** no mutation checks run for this task; no UI (T178); `cargo fmt` was run but `pnpm lint`,
      gitleaks and `scripts/verify-db-roles.sh` were not; the gate list is data, so which changes the deployer gates is theirs to set; a gate is not
      checked against the body's recognition dates.
- [ ] WPM-T178 **Verification and documents.** *(WPM-R102–R108)* Full suites, screenshots of new screens, strings in the 13
      locales, DPIA and record-of-processing updates; record what was not verified.
      **Part done (2026-10-27):** the 18 new server vocabulary tokens (body kinds, agreement bases, consultation topics, gate kinds) had no `values.*`
      entry, so `untranslated.test.ts` failed; added to all 13 locales and regenerated `static/admin/config.yml`. Ran: `cargo fmt --check`, `cargo clippy
      --all-targets -- -D warnings` (clean), `cargo test --lib` (860), request suite `--test mod -- --ignored --test-threads=1` (161), `enforcement` and
      `enforcement_expenses` (1 each); `pnpm check` (0/0), `pnpm lint`, `pnpm test` (218). **Not done:** there is no UI for T173–T177 yet, so no screens,
      screenshots or Playwright specs; record-of-processing not updated (the DPIA R15 row already covers bodies and units); `gitleaks` is not installed
      here, `scripts/verify-db-roles.sh`, the Keycloak suite and Playwright were not run; the new locale strings are machine-drafted and need a native review.
      **Part done (2026-10-27, UI):** `/bargaining` (recognised bodies, a contact role, units as rules, coverage shown as a count, withheld under the floor or
      "no one"), API client functions, nav entry, 22 strings + title in the 12 full locales (machine-drafted, unreviewed), Playwright spec `bargaining.spec.ts`
      (2 passed, stubbed API). Ran `pnpm check` (0/0), `pnpm lint`, `pnpm test` (218), `cms-config`. **Still not done:** UI for the consultation register,
      information pack, facility time and decision gates (T175–T177); no screenshots looked at; the page was not run against the real service; the full
      Playwright suite, Keycloak suite and `gitleaks` were not run.
      **Part done (2026-10-27, UI, consultation register):** `/consultations` (open a record, the timeline as a suggestion with its evidence and whether
      consultation began in time, record the start, close with an outcome), client functions, nav entry, ~45 strings in the 12 full locales (machine-drafted,
      unreviewed), `consultations.spec.ts` (2 passed, stubbed API). `pnpm check` (0/0), `pnpm lint`, `pnpm test` (218) pass. **Not done:** closing a
      consultation does not link an agreement from the UI; no plan link; information pack, facility time and decision gates (T176, T177) still have no UI;
      not run against the real service; no screenshots looked at.
      **Part done (2026-10-27, UI, information pack, facility time, decision gates):** on `/bargaining` the information pack per unit (withheld cells
      say so; vacancies say "not shown" for a unit defined by location or employment type; the page says reading is logged) and facility time per body per
      period with an over-budget flag; new `/decision-gates` (set a gate; a check that advises and names what the change needs). Client functions, nav
      entry, 38 strings in the 12 full locales (machine-drafted, unreviewed), Playwright specs (7 for the three pages, stubbed API). **Ran:** `pnpm check`
      (0/0), `pnpm lint`, `pnpm test` (218), the full Playwright suite (246 passed, 19 skipped). **Not done:** not run against the real service; no
      screenshots looked at; no axe check on the new pages; Keycloak suite and `gitleaks` not run; a native review of the
      locale strings.
      **Then:** the AI-register page shows the `decision_gate` pointer after recording a feature and takes an optional consultation record reference (spec extended, 2 passed).
      Looked at one screenshot (`/bargaining` with a pack and facility time, stubbed data): empty pack sections printed a bare heading, fixed. Known small fault left: the unit count reads "1 units" (no plural forms in the catalogue). Then ran the real service (a throwaway Postgres 18 under Podman, `WPM_REQUIRE_AUTH=0`, auto-migrated) with the new live spec
      `tests/live/bargaining.spec.ts`: a body, a unit under the floor with its pack, facility time over budget, a consultation with its timeline and its start,
      and a gate with its check, all against the real API: **passed**. Looked at live screenshots of `/consultations` and `/decision-gates`: the "Body consulted"
      select showed blank (nothing preselected), fixed and re-run (live and stubbed both pass). **Still not done:** the AI-register pointer was not run live;
      a pack over a unit with real workers (the floor met) was not exercised live; `/bargaining` live screenshot not looked at; Keycloak
      (enforcement on, so the HR-only reads are not exercised with real tokens from the UI) and `gitleaks` not run.
      **Live, second run (2026-10-27):** `tests/live/bargaining-pack.spec.ts` against the real service (throwaway Postgres 18 under Podman, enforcement off):
      12 workers made through the API, a body, a unit and a gate; the unit shows "12 covered", its pack shows 12 people and "Care: 12" and no name; recording
      a score on the AI register shows the gate to consult and the equality impact template: **passed**. This closes the live gaps for the pack above the floor
      and the AI-register pointer. Still not run live: enforcement on (real tokens, Keycloak).
      **Record of processing (2026-10-27):** row 16 added to `spec/governance/record-of-processing.md` (retention: none, checked by searching the sweep code for these tables; the lawful basis is the deployer's to fill). Not reviewed by a data protection officer.
      **Accessibility (2026-10-27):** `bargaining-a11y.spec.ts` runs axe (WCAG 2 A and AA) on the three pages with data, a pack, a timeline and a check on screen:
      no violations. Automated rules only: no keyboard walk-through and no screen reader, and not in dark mode or at phone width.

## Phase 19 — SFIA and SFIAplus (proposed WPM-R109–R113, WPM-D69, WPM-D70)

Researched in [sfia.md](sfia.md); specified in [plan.md](plan.md) section K. **No SFIA or SFIAplus
content is ever added to this repository** (WPM-D69).

- [x] WPM-T179 (2026-10-10) **Concept and licence note.** *(traces to WPM-R109–R113, WPM-D69, WPM-D70)* `spec/sfia.md`: what SFIA and SFIAplus are,
      what the licences allow (internal use under the free corporate licence; no distribution or
      publication; downloads need registration), how it fits the existing framework model, five
      requirements and two decisions. **From web sources and the maintainer's summary; the current
      licence text was not read in full and must be checked before any import.**
- [ ] WPM-T180 **The licence guard and record.** *(WPM-R113)* Migration adding to `capability_frameworks` the version,
      redistribution status (`open`, `internal_only`, `unknown`), licence holder, reference, last check
      date and attribution; pure `rules::framework::publishable`; every surface that leaves the
      signed-in organization uses it. Unit tests and an integration test that an unauthenticated request
      and a public job post never carry imported wording of a non-publishable framework.
      **Part done (2026-10-27):** migration `000096` (version, `redistribution` open/internal_only/unknown with a CHECK and default `unknown`,
      licence holder, reference, last check date; `uk-gdad-pcf` and `esco` set `open`), entity fields, pure `rules::framework::{Redistribution, publishable}`
      (only `open` with a recorded attribution; anything else is a no) with 2 unit tests. `cargo fmt`, `cargo clippy --all-targets -- -D warnings` clean,
      `cargo test --lib framework` (15). **Not done:** the migration was not run against Postgres; no surface calls `publishable` yet, because the
      service has no public job-post or unauthenticated framework read today, so the integration test the task names cannot be written until one exists
      (it belongs with the first such surface); no UI for the licence record; `import_framework` and `import_esco` do not yet set version or check date.
- [x] WPM-T181 (2026-10-27) **Pure core: the SFIA CSV contract.** *(WPM-R109, WPM-D70)* Parser and validator for the documented columns
      (skill code, name, category, subcategory, description, level, level description, attributes):
      levels 1 to 7 only, a skill defined at some levels only, no invented level or text, a clear
      refusal naming the column; the `linear` mapping 1, 2, 2, 3, 4, 4, 5 pinned. A **synthetic fixture of
      invented text**, labelled not SFIA.
      **Done:** `rules/sfia.rs` (`parse_skills`, `COLUMNS`, `SFIA_SCALE_MAX`) over the shared RFC 4180 parser; refuses a wrong header, a cell count that does
      not match, a blank required cell, a level outside 1 to 7, a skill whose own fields differ between rows, and a repeated level, each naming row and column.
      Fixture `tests/fixtures/sfia-synthetic/skills.csv` (invented, labelled not SFIA). 6 unit tests incl. the pinned mapping; clippy clean.
      **Not verified:** the contract is this repo's own reading of the documented columns, not checked against a real licensed download; `attributes` is kept as
      one free-text cell; the five generic attributes per level (T183) have no contract yet.
- [x] WPM-T182 (2026-10-27) **`import_sfia` task.** *(WPM-R109, WPM-R113)* Idempotent, records the version, marks skills and levels that a
      newer version drops as retired rather than deleting them, attaches the SFIA code as an external
      reference on the WPM skill, and writes the framework row with `internal_only`. Request tests with
      the fixture; refuses a file with no recorded licence check.
      **Done:** `tasks/import_sfia.rs` (`file:`, `version:`, `licence_checked_on:` required; `licence_holder:`, `licence_reference:` optional); migration `000097`
      (`sfia_skills`, `sfia_skill_levels`, each with `retired_in`); the `sfia` framework row (scale 7, `internal_only` whatever it was before); upsert, then
      skills and levels absent from a newer file are marked retired, and restored if they return; a WPM skill is linked to a code only on an exact normalised
      name match (one per skill, one per code). `GET /api/capability-frameworks` now returns version, redistribution, holder and check date (no wording).
      **Verified** against Postgres 18 (throwaway, Podman): request test (refusals write nothing, idempotent, retire, restore, link), the scheduler-config
      test, the full `--ignored` request suite (162), `cargo test --lib` (868), clippy clean; migrations 000096 and 000097 ran. **Not verified:** a real
      licensed file; the task through `cargo loco task` on the command line (only `import_sfia()` was called); a file with several WPM skills sharing a
      name; no UI for the catalogue (T183, T184); `import_framework` and `import_esco` still do not record a version or check date.
- [x] WPM-T183 (2026-10-27) **Levels of responsibility as a job-level framework.** *(WPM-R110)* Move job-level frameworks to the
      database so an imported framework has the same shape as the coded one; import the seven levels and
      five attributes; a worker's and a role's level can be an SFIA level. UI. Tests.
      **Done:** migration `000098` (`job_level_frameworks`, `job_level_levels`; `redistribution` defaults `unknown`); coded ladders stay as they were, stored ones are
      read beside them (`StoredFramework`, a coded ladder wins on a clashing id), so `/api/job-levels*`, a worker's level and a role's level accept `sfia-levels`
      with the same validation; levels-of-responsibility CSV contract (`rules::sfia::parse_levels`: `level,title,essence,autonomy,influence,complexity,
      knowledge,business_skills`, exactly levels 1 to 7 once each, no blank cell; invented fixture `levels.csv`); `import_sfia levels_file:<path>` (needs the
      licence version and check date; `internal_only`; re-run updates in place). The job-levels page has a framework select and shows attributes; the worker
      and role pickers choose among ladders. 2 strings (`jobLevels.framework`, `jobLevels.attributes`) in the 12 full locales (machine-drafted, unreviewed).
      **Verified:** Postgres 18 (throwaway, Podman): new request test (refusals write nothing, idempotent, listing, attributes, worker and role level, 422 on a
      level the ladder lacks), full `--ignored` request suite (163), `cargo test --lib` (871), clippy clean; `pnpm check` (0/0), `pnpm lint`, `pnpm test` (218),
      `levels-and-pay.spec.ts` (7, stubbed API). **Not verified:** no real SFIA download; the UI was not run against the real service and no screenshot was looked
      at; the worker and role pickers' ladder switch has no Playwright test; attribute names show as the contract's English tokens (underscores to spaces), not
      translated; `GET /api/job-levels*` needs a signed-in caller (kept inside the organization), but the guard `publishable` is not called there because
      the route is not public; the Keycloak suite, axe and `gitleaks` were not run.
- [x] WPM-T184 (2026-10-27) **Native level on declared skills and role requirements.** *(WPM-R111, WPM-D70)* Store the framework's level and scale on a
      declared skill (role requirements already do), show the native level, its description and the
      mapping used; start a role from SFIA skills as ESCO does. Request tests and Playwright;
      strings in the 13 locales (framework wording itself is not translated).
      **Done:** migration `000099` (`worker_skills` gains `source_framework`, `source_level`, `source_scale_max`; a requirement gains `source_framework`, backfilled
      `uk-gdad-pcf` where it had a level); `rules::sfia::to_wpm` (the `linear` map, pinned) and `MAPPING_NAME`; `PUT /api/workers/{pid}/skills` and
      `PUT /api/role-profiles/{pid}/requirements` take `framework` + `framework_level` (the 1–5 may then be left out and is mapped; a stated 1–5 stands; a skill
      that has a SFIA code can only be stated at a level SFIA defines for it; a plain declaration clears the native level, a requirement edit that states none
      leaves the imported one); reads show `mapping` and the framework's own `native_description`; `GET /api/sfia/skills?q=` and `/{code}`;
      `POST /api/role-profiles/from-sfia` (picked skills at defined levels, matched by code, then exact name, else created and linked; refusals create nothing).
      UI: `/roles` start-from-SFIA panel (shown once a `sfia` framework exists), the requirement table shows the native level, the mapping and the wording;
      6 strings in the 12 full locales (machine-drafted, unreviewed). The UI's OpenAPI copy and types regenerated.
      **Verified:** Postgres 18 (throwaway, Podman): new request test (search, show, declare with mapping, the six refusals, stated 1–5, clearing, requirement,
      start-from-SFIA incl. refusals and a taken title), full `--ignored` request suite (164), `cargo test --lib` (872), clippy clean; `pnpm check` (0/0),
      `pnpm lint`, `pnpm test` (218), the full Playwright suite (252 passed, 19 skipped; 2 new, stubbed API). **Not verified:** no real SFIA file; the new panel
      was not run against the real service and no screenshot was looked at; **there is no screen that declares a skill**, so a native level on a declared skill
      is API-only (the existing client function now accepts it); a declared skill's wording is returned but no page shows it; the framework name shows as its slug
      (`sfia`); a created skill's category is `technical` for a planner to correct; no axe check; the Keycloak suite and `gitleaks` were not run.
      **Follow-up (2026-10-27), the gaps listed above:** (1) the framework shows by **name** (`source_framework_name` on requirements and declared skills);
      (2) `category` is the planner's choice on start-from-SFIA (validated against the skill categories; the panel has a select); (3) a new **Declared skills**
      panel on `/me` and the worker page declares a skill on WPM's 1–5 or SFIA's 1–7 and shows the native level, the mapping and the framework's wording (5 strings
      in the 12 full locales, unreviewed); (4) requirement wording moved to its own list under the table after a screenshot showed it running off the edge, and
      the declared line lost a double parenthesis; (5) the `/roles` profile select could show blank after seeding (a value set before its options existed): it is
      re-made when the list arrives. **Run:** a real service (throwaway Postgres 18 under Podman) loaded by the real `cargo run -- task import_sfia` from the
      invented fixtures, and `tests/live/sfia.spec.ts` against it (3 passes; screenshots of `/roles` and the declared panel looked at, both after the fixes);
      axe (WCAG 2 A and AA) on `/roles` with the panel and a SFIA requirement, and on `/me` with a SFIA level: no violations; the Keycloak suite
      (`--features keycloak`, 1 passed, a real Keycloak container: it covers token enforcement, not these routes); `gitleaks` 8 from a container over the repo's own
      files (tracked and untracked, not ignored): 7 findings, all in files not changed here and all already reviewed in `.gitleaksignore`. A scan with `--no-git` over
      the whole directory also reads `target/` and `node_modules/` (57 hits, all build output or those same files) and is the wrong mode: the project scans history.
      **Still not done:** a real SFIA download (licensed; none here); axe was automated rules only (no keyboard walk-through, screen reader, dark mode or phone
      width); the new strings are machine-drafted; I did not see the profile-select assertion fail before the fix; the profile's `Source:` line still shows the raw token
      `sfia:picked-skills`.

- [x] WPM-T185 (2026-10-27) **SFIAplus development resources.** *(WPM-R112)* A documented CSV contract and `import_sfiaplus`, resources
      attached to a skill at a level with the source named and the BCS licence recorded; shown as pointers
      in training recommendations, development plans and the skill-gap view; nothing is booked or
      bought. The format is to be agreed with the licence holder before building.
      **Done, on a provisional format:** the task says the format is to be agreed with the licence holder before building; it was built on this repository's own
      CSV contract at the maintainer's instruction, and **the contract is provisional until the holder agrees it**. Contract (`rules::sfia::parse_resources`):
      `skill_code,level,kind,title,provider,url`; `kind` is `typical_task`, `training`, `certification` or `membership`; a level 1 to 7; a web address only if it starts
      `http://` or `https://` (also a database CHECK); a repeat is refused; invented fixture `resources.csv`. Migration `000100` (`sfiaplus_resources`, `retired_in`).
      `import_sfiaplus file: version: licence_checked_on:` refuses a missing licence check, a SFIA skill not in the imported catalogue and a level SFIA does not define
      for it (nothing is written), is idempotent, retires what a newer file drops, and records the `sfiaplus` framework row as `internal_only` with the BCS licence
      holder, reference and check date. Pointers (`controllers/sfia::pointers_for`) attach to a skill by its SFIA code at the SFIA levels that **count as** the
      WPM 1–5 level a need is at; each names its source, kind, SFIA level, provider and address. Shown in `GET /api/workers/{pid}/training-plan`, `/skill-gaps`
      (only for a need the person is below) and `GET /api/sfia/skills/{code}` (per level). UI: a pointers list under the training plan on `/me` and the worker page
      (source named, "nothing is booked or bought", an address that is not http(s) is never linked; 6 strings in the 12 full locales, unreviewed).
      **Verified:** Postgres 18: request test (refusals write nothing, idempotent, the level a need counts as, a retired resource disappears from the plan, the licence
      record), full `--ignored` suite (165), `cargo test --lib` (874), clippy clean; the real `cargo run -- task import_sfiaplus` loaded 4 invented resources and refused a
      skills file given as resources; `pnpm check`, `pnpm lint`, `pnpm test` (218), the full Playwright suite (256 passed, 19 skipped; `skill-gaps.spec.ts` extended).
      **Not verified:** a real `SFIAplus` file (licensed; none here); the pointers were not looked at in a screenshot or run in the browser against the real service (stubbed
      only); no axe check on the pointers list; pointers appear in the training plan and the skill-gap API, but **not on the development-plan (goals) screens** and
      the skill-gap panel in the UI shows them only through the plan; the first-listed source names the group (all pointers in a list share one source today); the
      Keycloak suite was run once earlier in this session and not again after this change.

- [~] (2026-10-27) WPM-T186 **Equality, privacy and documents.** *(WPM-R111, WPM-R113)* Register the level and skill-level scores in the
      scoring register and complete their assessments (WPM-T166); update the DPIA, the record of processing
      and the assessment checklist (licensed content); record what was not verified.
      **Part done (2026-10-27), the documents:** record of processing row 17 (a skill level stated on a framework's own scale; licensed content, which is not
      personal data), DPIA risks R16 (licensed content passed on) and R17 (a mapped level read as a measured score), assessment checklist 1.6 (third-party licensed
      content). Checked: `python3 scripts/check-links.py` (0 dead), `cargo test --lib` (874).
      **Not done, and why:** (1) *the scoring register*: it did not exist when the documents were written; T166 has since landed in part (`rules::scoring::REGISTER`, with S10 for the mapped level
      and its lint), so the level **is** registered, but with no assessment and status `Draft`; (2) *the equality impact
      assessments*: the mapped 1–5 level and the SFIA-based gap and plan views have none, and one is a judgement a person makes with the deployer's data
      (`spec/equality-impact/template.md`, thirteen sections), not something to write here; R17 names it as the further action; (3) whether **erasure removes** a
      declared level was not checked, and the record of processing says so; (4) the licence terms were not read in full; (5) the documents were not reviewed by a data
      protection officer. T186 stays open until T166 lands and the assessments are made.


## Phase 20 — readiness for large medical and governmental organizations (WPM-R114–R123, WPM-D71–D73)

Assessed in [readiness-assessment.md](readiness-assessment.md); specified in [plan.md](plan.md) section L.

- [x] WPM-T187 (2026-10-11) **Readiness assessment.** *(traces to WPM-R114–R123)* `spec/readiness-assessment.md`: the verdict
      (not ready), what is in the repository's favour, a blocker found by probing the running service under
      enforcement, thirteen gaps with evidence, the gates only people can close, and what was not checked.
- [x] WPM-T188 (2026-10-11) **Need-to-know guard.** *(traces to WPM-R114, WPM-D71)* `rules/access.rs`: the table of every `GET` route
      under `/api` (about 150) with its audience, the relations (stranger, in scope, manager, own) and the
      decision, with unit tests and a test that scans the controllers so a new route must be classified and a
      stale entry cannot linger. `need_to_know.rs`: the middleware, inside the sign-in guard, loading the worker
      (or the review's worker) and the caller's relation, including the management chain up to 20 levels and HR
      by membership of the worker's organization. New binary `enforcement_need_to_know` (eleven callers by six
      classes), **mutation-checked** four ways (the level ignored, only a direct manager, HR of any organization,
      no guard). Before and after the same probe: a stranger went from reading sick-leave reasons, time notes,
      applicants and the audit trail to `403` on all of them. **Two older enforcement tests encoded the old
      behaviour and were changed on purpose**: a stranger no longer gets a masked read of a colleague
      (`enforcement.rs`, and the contractor rate in `enforcement_engagements.rs`). Full suites pass: 376 unit, 69
      request, six enforcement binaries. Described in [auth.md](auth.md).
      **Not verified:** the interface against a real policy and real users (it may now show an error where it
      used to show data a person should not have seen), the Keycloak backend (the guard uses the same claims, but
      that suite needs a container), and load (up to a handful of queries per worker-scoped read). **Limits:**
      applicants are not organization-scoped, so HR of any organization reads them; the guard covers reads only.
- [x] WPM-T189 (2026-10-11) **Append-only, hash-chained audit trail.** *(traces to WPM-R115, WPM-D72)* Migration `000055`: columns
      `chain_seq`, `prev_hash`, `entry_hash`; one SQL function defining the hash; a back-fill of existing rows; an
      insert trigger (under a transaction-scoped advisory lock) and a trigger refusing `UPDATE` and `DELETE`.
      `Audit::verify_chain`, `GET /api/audits/verify` (privileged), the `verify_audit_chain` task (exits with an
      error when broken), and a step in the restore drill that checks the restored chain is intact with the same
      head hash. Verified by three request tests (gapless chain and refused `UPDATE`/`DELETE`; a rewritten row and
      a removed row are found, a truncated tail is *not*, and the head hash differs; sixteen overlapping
      transactions on a separate pool make one chain) and the restore drill passing against a seeded source.
      **Mutation-checked** twice (the verifier ignoring the hash; the insert trigger without its lock, which fails
      with a duplicate `chain_seq`). One test gap found and fixed on the way: the application's test pool holds a
      single connection, so the concurrency test now opens its own pool. Documented in [audit.md](audit.md), the
      runbook and the backup document. **Limits, stated:** a superuser can disable a trigger; cutting the tail
      needs an outside record of the head hash; `TRUNCATE` is not blocked; audit writes serialise and can deadlock
      with row locks in opposite order. **Not verified:** throughput under load (WPM-T196); a migration of a large
      existing audit table (the back-fill is one loop in one transaction); the Keycloak suite.
- [x] WPM-T190 (2026-10-11) **No diagnosis in leave.** *(traces to WPM-R118, WPM-D73)* `rules::leave::validate_reason`: a sick-leave
      request with a reason is refused (a blank one is none); other kinds keep theirs. Migration `000054` scrubs
      the reason from existing sick-leave rows and is **one-way** (the text cannot be restored; take a backup
      first). Verified by a unit test, a request test (three wordings refused, blank accepted and stored empty,
      an annual-leave reason kept), **mutation-checked** (removing the check fails the test), and the scrub
      statement run against real rows in a scratch database (a sick row emptied, an annual row kept); the DPIA,
      the record of processing and the checklist say so. **Not done:** time-entry notes and adjustment words are
      free text that could still hold health detail; the service cannot tell, so they stay a deployer's
      training and policy matter. The migration itself was not run on a database with data from an older release.
- [x] WPM-T191 (2026-10-11) **Production refuses insecure data paths.** *(traces to WPM-R116)* `hardening.rs`: classify the connection
      from its URL (verified and encrypted, encrypted, local, plaintext) and the boot decision; wired into the
      boot after the sign-in check; `timeout_request` in `config/production.yaml`. Verified by two unit tests
      (nineteen URLs, including an IPv6 loopback, a socket, a password that looks like a host, and the
      production, override and non-production cases) and by **booting the real binary** in production against a
      non-loopback address: refused with the instruction; started with the override and a warning; a loopback
      address started with neither; the log shows `timeout_request` and `request_id` loaded and a response
      carries `x-request-id`. Documented in [production-readiness.md](production-readiness.md) and the
      deployment document. **Not verified:** a real TLS connection (the test server has no certificate), and a
      request actually hitting the timeout. **Limit:** `require` encrypts but does not verify the server.
- [x] WPM-T192 (2026-10-11) **Encrypted backups.** *(traces to WPM-R117)* `backup.sh` pipes the dump through AES-256-CBC with PBKDF2 (the
      plaintext never touches the disk), writes a `.hmac` tag and a checksum, and refuses without a key unless
      `WPM_BACKUP_ALLOW_PLAINTEXT=1`; `restore.sh` checks the tag before decrypting and cleans up its temporary
      directory; the restore drill now runs the encrypted path with a key made for the run. Verified by hand on real
      dumps: no key and no flag refuses (exit 2); a short key refuses; the ciphertext begins `Salted__` and holds no
      table names while the plaintext dump holds 34; a right key restores 107 tables; a wrong key, a one-byte change
      and a missing tag each refuse with exit 4 and leave the target empty; a plaintext restore still works with a
      warning; no decrypted file is left behind; the drill passes. **One bug found and fixed on the way:** the
      cleanup trap returned 1 when there was nothing to clean, which would have overridden the exit status of a
      successful plaintext restore. **Not done:** these checks are a manual session, not an automated test in CI
      (the drill covers the happy path every run); authenticated encryption and key rotation are left to the
      platform or a key management service; the cipher is not authenticated, which the tag compensates for.
- [x] WPM-T193 (2026-10-11) **Threat model.** *(traces to WPM-R121)* `spec/governance/threat-model.md`: assets, actors, trust boundaries,
      and twenty-four threats by STRIDE category, each with the control as built and the gap where there is none;
      every control named was exercised in this phase, and what was not checked is marked **not assessed** (the
      interface's session handling, fuzzing, algorithm confusion, server-side request forgery beyond the short
      timeout, a scanner baseline). **Written by the same assistant that wrote the software and reviewed by no
      one else**; an independent team should redo it (WPM-T201).
- [x] WPM-T194 (2026-10-20) **CI hardening.** *(WPM-R119)* Every `uses:` in `ci.yml` and `release.yml` is pinned to the commit SHA its tag
      pointed at, with the version in a comment (Dependabot's `github-actions` entry updates both). `scripts/check-actions-pinned.py`
      runs in the `docs` job and fails on any action that is not a 40-character SHA; shown to fail on an unpinned `actions/checkout@v7`.
      New `secrets` job: gitleaks 8.30.1, downloaded as a release binary and checked against its published SHA-256, over the whole history.
      It found 7 things, all reviewed: test and development fixtures (throwaway RSA keys in a since-removed directory, a made-up
      idempotency key, loco's default JWT secret in the test and development profiles). They are in `.gitleaksignore` with the reason;
      production reads `JWT_SECRET` from the environment. **Not done:** GitHub's own secret scanning and push protection (a repository
      setting); a pull-request-only trigger (the checks run on every push); `zaproxy/action-baseline` is pinned but is a third-party action
      that runs with the job's permissions; the gitleaks job is not yet a required check.
- [x] WPM-T195 (2026-10-20) **Release integrity.** *(WPM-R119)* The `Containerfile`'s two base images (`rust:1`, `debian:stable-slim`) are pinned
      by their index digest. `release.yml` now takes `id-token` and `attestations` write permission, signs the pushed image by digest with
      cosign (keyless, `sigstore/cosign-installer` and `actions/attest-build-provenance`, both pinned to commit SHAs), attaches SLSA build
      provenance pushed to the registry, and runs `cosign verify` (issuer and this workflow's identity) and `gh attestation verify` before the
      release is created, so an image that cannot be verified is not released. How to check an image is in `spec/production-readiness.md`
      (WPM-R87). Verified: the image builds locally from the digest-pinned bases with the version and revision labels set; the workflow
      parses and every action is pinned. **Run on a runner (2026-10-20, v0.1.2):** the signing, attestation and verification steps all passed in the release workflow before the release was
      created. **Not verified by anyone outside the workflow:** the image's package is private, so `cosign verify` and `gh attestation verify` from a
      machine that is not signed in to the organization cannot be tried; making the package public is a decision for the owner, not taken. **Not done:**
      the digests are refreshed by hand (Dependabot does not read a `Containerfile`); the demo and test images in `compose*.yaml` and the CI
      service containers were not pinned (they are now, WPM-T231); no SBOM attestation (the CycloneDX file is attached, not signed); the
      v0.1.0 and v0.1.1 images stay unsigned.
- [x] WPM-T196 (2026-10-20) **Load evidence and bounded views.** *(WPM-R120, WPM-D90)* `scripts/load-test.sh` seeds one organization at 1k, 10k and 50k workers and
      drives four routes with `oha`; the result is in `BENCHMARKS.md`. The whole-organization views grow with the organization
      (capacity 88 → 518 → 2,202 ms) and **at 50,000 workers requests fail** (pool-acquire timeouts, 19 of 39 on the heavy views, 5 on the paged list).
      So `rules::scale` and `controllers::ensure_scale` refused the directory, the capacity view and the plan forecast with a `422` above
      10,000 workers (`WPM_MAX_ORG_WORKERS`); WPM-T230 then paged the first two, so the guard now covers the plan views only. Verified: 3 unit tests, a request test (refused at a ceiling of 1, answered above), **mutation-checked**
      (removing the check from the directory fails it). **Not done:** one run on one laptop, 8 connections, auth and rate limits off, no repeat; about fifty other
      reads still load a table with `.all()`; the views are refused, not paged or moved into SQL (the proper fix, and a larger job); the count adds a query to
      each; the load test is not in CI. A deployment with more than 10,000 workers in one organization cannot use those three views until it raises the setting after measuring.
- [x] WPM-T230 (2026-10-20) **Read in pages and batches; split the two largest controllers.** *(WPM-R120, WPM-D90)* `controllers/capacity.rs` (1,411
      lines) and `controllers/planning.rs` (1,214) are directories with a file per section; the route-coverage tests follow `mod.rs`. The
      directory searches, orders and pages in SQL; the capacity view reads in batches of 2,000 and sums (`pool_weighted_days`,
      `supply_from_weighted`, a unit test that batches add up to the whole); the plan views read only the planned departments and skills; the
      guard `WPM_MAX_ORG_WORKERS` now covers the plan views only. Re-measured in `BENCHMARKS.md`: the directory at 50,000 workers went from 1,229 ms
      and failures to 159 ms and none; capacity no longer fails but is still 2.2 s there (3.0 s before it selected six columns and counted days by interval arithmetic). Verified: all 101 database-backed tests, the unit tests, new
      tests for paging order (case-insensitive, **mutation-checked**), text-not-pattern search and manager-name search. **Not done:** the capacity
      view is still linear (the next step is SQL aggregates); the plan views are still read whole and refused over 10,000; the SQL ordering and
      lower-casing is exact for ASCII names and may differ from Rust's for some non-ASCII ones; the search has no index (`strpos` scans the employed
      rows, then pages), fine at the sizes measured.
- [x] WPM-T243 (2026-10-21) **The remaining whole-table reads.** *(WPM-R142, WPM-D97)* `scripts/load-test.sh` gains `LOAD_SURVEY=<paths>` (one timed
      request per route) and `LOAD_PATHS=<paths>` (the load on chosen routes). The survey of 107 parameterless `GET` routes at 200,000 workers found
      four that failed (`IN` list over the bind limit: skill gaps, training demand, contingent, programmes), two of 22 and 36 MB (mentorship overview,
      org chart) and eight over the bar at 8 connections. All now pass: p95 under 250 ms each (capability analysis 240, skills matrix 163, contingent
      107, org chart 52, mentorship 45, training demand 68, skill gaps 73, capability 165, ergonomics 1, training analytics 1, programmes 3), and
      retention 60 (it scanned 4,000,000 time entries: p50 0.5 s in one run, 1.7 s in another; migration 79 indexes it); the standard set still passes (working time p95 964 ms,
      capacity 1,389 ms, the worst). Parity tests: a skill-gap roll-up of merged role and target needs (**mutation-checked**: weakest importance
      wins, the met boundary, the highest level, case-insensitive department), capability (targets, active plans, completed sittings; mutation-checked
      three ways), grouped shares equal one row per person (rule tests), the org-chart limit and size, the mentorship cap, and every soft-deleting
      table indexed. UI: the org chart and the mentorship page say when they show part (two strings in 13 locales, AI-written, not reviewed);
      Playwright 143, vitest 160, svelte-check 0. *Not checked:* routes that take an id; the gap SQL assumes one current PCF role per worker (a unique
      index says so); the unmatched list of the mentorship page is no longer complete by design.
- [x] WPM-T244 (2026-10-21) **The routes that take a worker or a plan.** *(WPM-R142)* `LOAD_SURVEY` and `LOAD_PATHS` accept `{WORKER}` (a worker from the middle of the
      seed) and `{PLAN}`. The survey of 60 such `GET` routes at 200,000 workers found four that read every worker to answer for a few: upline, downline,
      reports and dotted-line (0.54 to 0.60 s for one request). They now read the chain (one manager at a time up; level by level down; only the
      people on the dotted lines): p95 1.4 ms (upline) and 2.5 ms (the others), no failed request. The other 56 answer in under 150 ms for one request
      (plan forecast 98, plan tolerances 143 for one cold request; both measured under load in T239/T243). Test: a manager who has left breaks the chain above
      and below and cannot be a dotted-line manager (**mutation-checked**: with the employed filter removed it fails). *Not checked:* the downline of
      the top manager of 200,000 returned every report before (cost grows with the team, not the organization); closed in the same task by a `limit` (default 5,000, at most 20,000, nearest first) on `downline` and `reports`, the summary and `total` counting everyone and `truncated` saying whether the list is whole (tested); the seed has no
      reporting hierarchy, so the chain walks were measured short.
- [x] WPM-T242 (2026-10-21) **Open escalations on the CEO dashboard.** *(WPM-R141, WPM-D94)* A seventh tile counts the open escalations of the caller's
      organizations (the same `GET /api/escalations?organization=` inbox, one call per organization in scope) with the age of the oldest in calendar
      days, links to Planning, and shows "—" when it cannot be loaded or the caller has no organization in scope (never 0). The third row is now the trend
      (two columns), the insights and the escalations (one each). Three strings in 13 locales (AI-written, not reviewed). Verified: the CEO smoke spec at both
      screen sizes (seven tiles, none clipped, the count, the oldest age, the link), the whole Playwright suite (140), svelte-check 0, 160 vitest, prettier.
      *Not checked:* with the real service; the insights tile is narrower and shows less of each insight than before.
- [x] WPM-T241 (2026-10-21) **CPD: correct and withdraw a requirement from the page.** *(WPM-R139, WPM-D92)* The CPD page lists the defined requirements with
      Correct (name, amount and period; the unit is shown and sent unchanged) and Withdraw (a second click confirms; recorded entries are kept), over the
      existing `PUT` and `DELETE`. Six strings in 13 locales (AI-written, not reviewed). Verified: a Playwright spec stubbing the API (the body sent, the
      first click only asks, the list updates), svelte-check 0, 160 vitest, prettier. *Not checked:* against the real service in a browser; changing the job
      title restriction is not offered (it is sent unchanged).
- [x] WPM-T240 (2026-10-21) **Working time reads a weekly rollup.** *(WPM-R142, WPM-D96)* `time_entry_weeks` (worker, week, recorded minutes) is kept by a
      trigger and backfilled by migration 78; the view names the workers who can be over from a partial index on heavy weeks, sums their weeks, then
      sums their entries over the exact window, so the answer is unchanged. New tests: the window edges, the rollup equals the entries across an
      insert, update, soft delete, restore and hard delete (**mutation-checked**: moving the window edge fails it), and the floor equals the index.
      At 200,000 workers (the load test now records weekdays only, a 40-hour week of 2,400 minutes, and prints p95): working time p50 458 ms, p95 802 ms,
      p99 862 ms, no failures. Worst case, every worker with a 56-hour week (the index prunes nothing): p50 2.1 s, p99 2.7 s. *Not checked:* a real ledger's
      shape; the trigger's write cost at scale beyond the seeding of 4,000,000 entries, which completed.
- [x] WPM-T239 (2026-10-21) **Hold 200,000 workers.** *(WPM-R142, WPM-D95)* Every measured route meets the bar (p95 under 2 s, none failed at 8 connections)
      on `scripts/load-test.sh 200000`: capacity in SQL, the plan forecast and cost in groups, the key-person register and working time read by the
      database, and working time from a weekly rollup (WPM-T240); the ceiling `WPM_MAX_ORG_WORKERS` defaults to 200,000. The failures first seen were
      mostly the demo profile's single database connection. *Not done:* about 40 other whole-table reads that no measured route uses (not measured here).
- [x] WPM-T238 (2026-10-20) **The key-person register and working time are read by the database.** *(WPM-R120, WPM-D90)* At 50,000 workers both read every
      worker into memory and failed requests (1.3 s and 1.0 s). The register now reads only the people who hold a critical skill at its bar (the employed
      population is a subquery, the same `is_employed_on` rule written in SQL once in `employed_in_scope_query`), their backups and succession plans in
      batches of 5,000, and nobody when no skill is critical. Working time sums the window's minutes per worker in SQL and brings back only those over
      the ceiling, reads only the shifts inside the window of 28 calendar days either side and their assignments in batches, and reads the workers only for those flagged;
      `workers_checked` is a count. The answers are the same. At 50,000 workers with 10,000 holders and 1,000,000 time entries, one request at a time:
      register 1,292 to 65 ms, working time 965 to 117 ms, no failures; at 8 concurrent connections on the laptop that also hosts the database, 516 and
      916 ms, and working time still lost 3 requests to a pool timeout (`BENCHMARKS.md`). Verified: the existing key-person and working-time request tests
      pass unchanged, working time gained two cases (a rest breach alone flags and shows its total; a turnaround outside the window does not), the
      workforce request tests, clippy. `scripts/load-test.sh` now seeds a critical skill and 20 time entries per worker, so both routes do real work.
      **Not done:** working time's aggregate scans every entry in the window and saturates the pool at 8 concurrent requests at this size (a covering index on
      `worked_on` was tried and changed nothing, 877 ms against 916, so it was not kept); capacity remains linear; about 40 other whole-table reads.
- [x] WPM-T237 (2026-10-20) **The dashboard's counts are made by the database.** *(WPM-R120, WPM-D90)* Measured at 50,000 workers, the workforce overview, metrics and
      insights read every worker and failed requests (about 600 ms). `GET /api/workforce-intelligence/overview` now groups in SQL (department, status,
      type, hire date for tenure, one row per manager, a sum for FTE; a manager is named only if employed on the date) and `period_metrics` counts opening,
      closing, starters and leavers in one query and span of control from one row per manager; the definitions (`is_employed_on`, starters, leavers) are
      written once in SQL. Metrics and insights 655 to 51 ms, the overview 561 to 272 ms, no failures (`BENCHMARKS.md`). Verified: two new request tests
      (groups, fte, tenure, a named and an unnamed manager, an earlier `as_of`; opening, closing, starters, leavers across the period edges) that pass
      against the old code and the new, the 106 existing request tests, clippy. **Not done:** the key-person register and working time still read every worker
      and fail at 50,000; the overview still returns one row per manager (unbounded in an organization with very many managers); the module doc's
      "in-process, datasets are small" was true when written and is amended.
- [x] WPM-T236 (2026-10-20) **The CPD overview counts in the database.** *(WPM-R120, WPM-D90)* `GET /api/cpd/overview` no longer reads every worker and
      every CPD entry: per requirement one aggregate query returns who it applies to, who recorded and who verified at least the required amount (the
      definitions of `applies_to`, `in_period` and `progress`, in SQL), the headcount is a count, and only the registrations that are expired or inside
      the warning window are read. The answers are the same. Verified: a new request test with workers of the title (case and spaces ignored), another
      title, one not yet employed, entries of another unit, outside the period and withdrawn, and registrations expired, expiring, valid and for a
      worker not yet employed, **mutation-checked** (`>=` made `>` fails it); the existing CPD ledger test passes; measured in `BENCHMARKS.md`
      (50,000 workers, 323 ms, no failure). **Not done:** the entry side was not exercised at scale (the load test seeds no CPD entries); about forty
      other whole-table reads remain; the job-title match is exact after lower-casing and trimming, as before (ASCII-exact lower-casing in SQL).
- [x] WPM-T235 (2026-10-20) **Escalations of a breached tolerance.** *(WPM-R141, WPM-D94)* [tolerances.md](tolerances.md#escalations). Migration `000077`
      `plan_escalations` (level 1 to 3, open or decided, the status, deviation and limit at raising and at deciding, one open per plan and dimension,
      checked in the database). `rules::escalation`: the three levels, the decisions, `may_raise` (amber or red only, the refusal says which status it
      found), `clean_note` (at most 500 characters, required for a decision), `days_open` in calendar days, `breaches_without_escalation`. `POST
      /api/workforce-plans/{pid}/escalations`, `POST /api/escalations/{pid}/raise-level`, `POST …/decide` (an administrator of the plan's organization,
      the check shared with frozen oversight packs), `GET …/escalations` and the organization inbox `GET /api/escalations?organization=`; in-app
      notification to the organization's administrators on raising and on a level raise (the note is never copied into it); audited. The tolerances view
      carries each dimension's open escalation and `breaches_without_escalation`. UI: an Escalation column on the Tolerances table (escalate with a note,
      raise level, decide with a reason), a flag for red breaches nobody has been asked about, and a history of decided ones; 18 strings in 13 locales
      (AI-written, not reviewed). Verified: 6 unit tests, a request test through all three levels, the refusals, the decision and a second escalation,
      a new enforcement binary `enforcement_escalations` (a member who may write raises, only an administrator decides, another organization sees
      nothing, `decided_by` is the signed-in person), 3 Playwright specs, screenshots looked at, svelte-check 0, 160 vitest. **Also fixed:** the
      `enforcement_oversight_pack` and `enforcement_cutover` binaries were never run by CI; they pass and now are. **Not done:** nothing escalates by
      itself and nothing chases a late decision; levels are generic names, not tied to a role (who is "the board" is the organization's); a decision does
      not change the tolerance or the plan; no notification to the decider when a level is raised beyond the administrators; the inbox is per organization
      and is not on the CEO dashboard; the note is free text and could hold a name (the form says not to, nothing checks).
- [x] WPM-T234 (2026-10-20) **Tolerances on a workforce plan, with red, amber, green.** *(WPM-R140, WPM-D93)* [tolerances.md](tolerances.md). Migration `000076`
      `plan_tolerances` (plan, dimension, limit, amber share; unique per plan and dimension; checked in the database). `rules::tolerance`: the four
      dimensions (time in calendar days, cost and resources in basis points, risk in roles), `assess` (green, amber, red, unknown; a limit of 0 makes any
      deviation red), `overall` (worst wins; unknown ranks above green), and the four deviations, each `None` when it cannot be worked out. The
      forecast, cost and alignment handlers now return their values through `*_value` functions, so a deviation is read off the one definition of each
      figure; alignment also reports how many succession plans it looked at, so "no plans" is unknown and not 0 risk. `GET
      /api/workforce-plans/{pid}/tolerances`, `PUT` and `DELETE …/{dimension}` (audited; access table and OpenAPI entries). UI: a Tolerances table on
      `/planning` with an overall chip and a chip per dimension that is a colour, a shape (● ▲ ■ ?) and a word, with the deviation, limit and amber share
      editable; 17 strings in 13 locales (AI-written, not reviewed). Verified: 12 unit tests on the edges (the amber and red points, a zero limit, small
      limits, unknown never green, an incomplete cost total never "within budget", saturation), a request test that walks one plan through amber, red,
      green and unknown and the refusals, **mutation-checked** (an unknown made green fails both), 2 Playwright specs (statuses and the PUT/DELETE sent),
      screenshots looked at in light and dark, svelte-check 0, 160 vitest. **Not done:** nothing escalates or notifies when a plan goes red (WPM does not
      know who may accept a breach); no history of a plan's status over time; tolerances only on workforce plans, not on capacity, a cutover or a
      programme; quality and benefits (the other project-management tolerances) have no measure here; the risk figure counts critical roles without a
      ready successor and nothing else; `time` measures overdue demand, not a forecast of slipping; the Welsh word for cost is the English one and is allow-listed.
- [x] WPM-T233 (2026-10-20) **CPD: correct a requirement; pace.** *(WPM-R139, WPM-D92)* `PUT` and `DELETE /api/cpd-requirements/{pid}` (the unit
      cannot change; entries untouched; audited `updated` and `withdrawn`). `rules::cpd::pace`: `met`, `not_started`, `on_track`, `behind`, `missed`,
      days left, the amount needed per 30 calendar days (rounded up) and the amount the straight line gives by today, with its basis in the payload; on every
      requirement of `cpd-progress`. UI: a Pace and a Needed-per-30-calendar-days column and the basis line; five strings in 13 locales (AI-written, not
      reviewed). Verified: 5 unit tests on the edges (exactly on the line, one hundredth short, a one-day period, met winning before and after the
      period), a request test (behind, corrected to on track, unit change refused, withdrawn, entry kept), a Playwright assertion, OpenAPI entries,
      svelte-check 0, 160 vitest. **Not done:** the overview (HR's aggregate) has no pace; the UI cannot yet correct or withdraw a requirement (the
      API can); pace ignores that CPD is lumpy by design (WPM-D92 says so); `GET /api/cpd/overview` still reads every employed worker and every entry
      whole (not paged).
- [x] WPM-T232 (2026-10-20) **Links to the learning management system.** *(WPM-R138, WPM-D91)* `rules::lms::parse_template` (https only, one `{course}`,
      no spaces, a real host with no user-info, at most 500 characters) and `course_url` (the id after the URN's first colon, percent-encoded so it
      cannot add a path, query or host); `WPM_LMS_COURSE_URL` sets the template, unset means no links. `lms_url` appears on a CPD entry from the
      learning system, a worker's training enrolments, a learning-path step, a skill's catalogue course and a cutover role's course. Migration
      `000075` adds `cpd_entries.course_ref`, set by the completion sync and filled for earlier entries from their text. UI: the LMS chip on a CPD
      entry and the course in a worker's training table open the course in a new tab (`noopener noreferrer`); one new string in 13 locales
      (AI-written, not reviewed by a native speaker). Verified: 3 unit tests (including a course id that tries to add a path, query, fragment and
      second host), a request test (no template, a valid one, three unsafe ones), **mutation-checked**, a Playwright spec, **mutation-checked**,
      svelte-check 0, 160 vitest. **Not done:** the link is not followed or checked (the service does not know the course exists there); an
      unreachable or renamed LMS leaves dead links; one template for the whole deployment (no per-organization LMS); links are not on the certificate
      expiry list, the skills-gap plan or an exported CSV.
- [x] WPM-T231 (2026-10-20) **Every container image is pinned by digest.** *(WPM-R119)* The Postgres and Redis service containers in `ci.yml` (twelve), both
      compose files, the Stalwart `Containerfile` (base and `COPY --from`) and the load-test script name `image:tag@sha256:…` (the multi-arch index
      digest, read from the registry; two pulled by digest to check). `scripts/check-actions-pinned.py` now fails on an image reference without a
      digest in a workflow service, a compose file, a Containerfile or a script, and was shown to fail when a digest was removed. Dependabot gets
      `docker-compose` entries. **Not done:** Dependabot does not read a `Containerfile` or a workflow's service images, so those digests are updated by
      hand; a digest pins what the tag was today, it does not say the image is free of vulnerabilities (no image scan runs); the Keycloak image used by
      the tests and the demo was not among the references the checker found (it is started by the test, by tag).
- [x] WPM-T197 (2026-10-20) **Observability.** *(WPM-R122, WPM-D89)* `src/request_log.rs` is the outermost layer: it sets `X-Request-Id` on
      every response (the caller's if plain, else a UUID; checked on a hostile one) and writes one line per request with the id, method, a
      `route` with identifiers replaced by `{}`, the status and the latency. Loco's request logger, which writes the whole URI, is off in
      all four profiles. `tests/request_log.rs` installs a subscriber, sends a name in the query, a UUID in the path, a token and a user
      agent, and fails if any appears in the captured output; shown to fail when loco's logger is switched back on (it logged the UUID).
      `GET /api/audits/export` returns the trail as JSON Lines (resumable by `after`, capped, `X-Next-After`, no snapshot), classified
      Privileged in the need-to-know table and in the sensitive rate-limit class, with a request test, a unit test that the snapshot
      never leaves, and an OpenAPI entry. New CI step. Verified: 577 unit tests, `mod` (98), `enforcement`, `enforcement_expenses`,
      `security` and `chaos` pass; clippy and `cargo fmt --check` clean. **Not done:** the exported fields were not agreed with a real
      monitoring product; no push (webhook or syslog) transport, only polling; the log has no trace or span ids beyond the request id; a
      lowercase word in a path is kept in the log (WPM-D89); the export has no enforcement-matrix test of its own beyond the table entry.
- [x] WPM-T198 (2026-10-20) **Rate limits across instances.** *(WPM-R74)* `spec/operations/deployment.md` gains "Running more than one
      instance": limits are per instance, so a caller gets N times the limit behind N instances (and N times the sensitive limit); three ways
      to restore a real limit (at the proxy or gateway, recommended; divide the settings by N; sticky routing), why WPM has no shared
      counter (it would make the limiter a dependency of every request), the `WPM_TRUST_FORWARDED` condition, and what does not need
      sharing. `spec/production-readiness.md` points to it. **Not done:** no shared counter; no test (it is guidance); the proxy snippets
      are named, not given for each product.
- [x] WPM-T199 (2026-10-20) **Least-privilege database roles.** *(WPM-R123)* `postgres-roles/roles.sql` (idempotent) makes `wpm_migrator` (owns and
      changes the schema) and `wpm_app` (rows and sequences only; no create, alter, drop or truncate; no update, delete or truncate on
      `audit_logs`). `scripts/verify-db-roles.sh` migrates as the migrator, shows the runtime role refused nine schema and audit
      changes, then runs every request test (97) as `wpm_app`, each on its own clone of the migrated template database: 95 pass and the 2 tests
      that deliberately tamper with the audit trail are refused by the database. New `api-roles` CI job. `config/test.yaml` gained
      `WPM_TEST_FRESH_SCHEMA` and `WPM_TEST_TRUNCATE` (both default on, so nothing else changes). Guidance in `deployment.md`
      ("Database roles"). Verified: the script passes three times locally against Postgres 18. It first showed that the suite resets data
      by recreating the schema, which the runtime role must not do, hence a clone per test. **Not done:** the other DB-gated binaries
      (enforcement, security and so on) were not run as `wpm_app`; the backup and restore role is not defined; the extensions are
      created by an administrator, not by `roles.sql`; row-level security and per-table grants narrower than read-write are not attempted.
- [x] WPM-T200 (2026-10-20) **Token age and revocation.** *(WPM-R72)* `auth::check_token_lifetime` refuses a token whose `exp - iat` exceeds
      `WPM_MAX_TOKEN_LIFETIME_SECONDS` (default 86400; `0`/`off` removes it) or that has no `iat`, with a `401` that names neither limit nor
      token; both backends (PASETO and Keycloak/Entra) call it after signature and expiry. `spec/auth.md` "Token age and revocation" states
      that revocation is by lifetime and key rotation (there is no list and no introspection) and what the identity provider must do.
      Verified: 3 unit tests of the rule and one per backend; the 10 `enforcement*` binaries and `oidc_entra` pass. The test tokens
      (11 files minted ten thousand million seconds ahead) now live an hour and are issued now. **Not verified:** against a real Keycloak
      (the CI `api-keycloak` job does, and issues five-minute tokens). **Not done:** no revocation list, no `sid` check, and the 24-hour default is a
      judgement: a provider issuing longer tokens must raise the setting.
- [ ] WPM-T201 **Independent penetration test.** *(WPM-R121)* A person's task, recorded here when done. **Not done.**
- [ ] WPM-T202 **Independent accessibility audit.** *(WPM-R121)* A person's task. **Not done.**
- [ ] WPM-T203 **Gates refresh.** *(WPM-R121)* Fold T201, T202 and the existing WPM-T132 gates into one list and keep the
      assessment current when any of them closes. **Not done.**

## Phase 21 — self-service, requests and workplace health requirements (proposed WPM-R124–R130, WPM-D74–D77)

Specified in [self-service-and-worker-requests.md](self-service-and-worker-requests.md) and
[plan.md](plan.md) section M. **Proposed, not built.** The two bounded exceptions (WPM-R125, WPM-R128) need the
deployer's lawful basis and impact assessment before they are switched on.

- [x] WPM-T204 (2026-10-11) **Spec round.** *(traces to WPM-R124–R130, WPM-D74–D77)* The topic note, the plan section and these
      tasks; the two conflicts with earlier decisions named and bounded. **Not reviewed by a data protection
      officer, an occupational-health adviser or an employment lawyer.**
- [x] WPM-T205 (2026-10-11) **The `/api/me` write allow-list.** *(traces to WPM-R130, WPM-D76)* `rules::self_service::WRITES` and `is_write`; the sign-in
      guard skips the policy's action check for those routes only and still requires a valid token; the `me`
      controller resolves the person from the token alone. Unit tests, including a scan of the controller so a
      write route not on the list, or an entry with no route, fails the build. New binary `enforcement_me`.
      **Mutation-checked** three ways (no allow-list in the guard; any contact changeable through `/me`; the list
      widened by a `POST`). Documented in [auth.md](auth.md). **Limit:** the unit scan reads source text; a route
      added by a different mechanism than `.add(` would escape it.
- [x] WPM-T206 (2026-10-11) **Contact details.** *(traces to WPM-R124)* Migration `000056`, `rules/contact_details.rs` (shape only: lengths,
      telephone characters and 5 to 15 digits, an e-mail's outline, a two-capital-letter country), `GET`, `PUT`
      and `DELETE /api/me/contact-details`, and `GET /api/workers/{pid}/contact-details` for HR and payroll (the
      worker and privileged callers only; a read by anyone but the worker is audited). Replaced as a whole, no
      history kept, audit without values, exported and erased with the worker. Verified in `enforcement_me`: bad
      shapes refused naming the field; the audit holds the events and no value; a neighbour refused; the export
      includes it. **Not done:** telling payroll that an address changed (the audit records it; no notification
      is sent); a retention schedule for the table (it goes with the worker's erasure).
- [x] WPM-T207 (2026-10-11) **Emergency contacts by the worker.** *(traces to WPM-R124)* `GET`, `POST`, `PUT`, `DELETE /api/me/emergency-contacts`, reusing the existing
      rules (a limit of five, a ranking, an audit with no detail) by splitting the HR handlers into shared
      functions. Verified: a worker writes their own; a neighbour sees none and cannot change or remove one (404);
      the limit holds; HR still reads them through the existing route; the HR write route stays HR's.
- [x] WPM-T208 (2026-10-11) **My time-off.** *(traces to WPM-R129)* `rules/time_off.rs` (balances per kind and year: allowed, subtracted, taken,
      booked, requested, remaining, and what would remain if requests were approved; a kind with no allowance has
      no remaining figure; past absence by year and kind) and `GET /api/me/time-off?year=`, in calendar days,
      the caller's own, no reason for sick leave, no rate or score. Unit tests at each boundary and a request test
      in fixed past and future years. **Found on the way (new task WPM-T216):** under the reference policy a
      worker cannot request their own leave, because the request is a write only HR may make; this view shows
      requests but cannot create one.
- [x] WPM-T209 (2026-10-11) **Resignations: pure core.** *(traces to WPM-R127, WPM-D77)* `rules/resignation.rs`: the states (logged, withdrawn, accepted,
      rescinded) and who may move them (a worker withdraws only while it is logged; only a person accepts or rescinds);
      the earliest last day from the notice in calendar days (default 28, 0 to 366); a proposal must allow the notice;
      an agreed day between the day logged and 366 calendar days on; a reason optional and from a closed list of eleven
      (no health reasons, no free text); and a summary that withholds a department below the floor and its split by
      reason unless every cell reaches the floor, so a hidden cell cannot be worked out. Seven unit tests.
- [x] WPM-T210 (2026-10-11) **Resignations: records and API.** *(traces to WPM-R127)* Migration `000057` (`resignations`, one open per worker), the worker's side under
      `/api/me/resignation` (log, withdraw, read), and `/api/resignations` (list, summary, accept, rescind) and
      `GET /api/workers/{pid}/resignation`. **Accepting records the agreed last day and opens the leaver process
      (the movement, with its dated checklist) and tells the worker in one transaction; it does not end employment**
      (verified: the worker is still active afterwards). The manager is told it was logged and never the reason; the
      reason is visible to the worker only and in aggregates; audit entries hold none. Exported and erased with the
      worker. Verified in `enforcement_me` by a full scenario (notice and reason refused, one open at a time, manager
      and HR see no reason, a stranger and a manager are refused the list, withdraw then log again, only a person
      with authority accepts, the movement exists and is cancelled on rescind, the aggregate withholds a small group),
      **mutation-checked** three ways (the manager sees the reason; no notice check; acceptance opens no record).
      The movement logic was split into a shared function. **Differs from the plan:** "sets the engagement end" is not
      done, because a permanent engagement has no end date and moving a contingent one earlier is a shortening, not an
      extension; the leaver record carries the agreed day. HR who decides need the `hr` attribute **and** a membership
      in the organization. **Not done:** a screen, a reminder before the agreed day, and the floor is the pulse
      survey's fixed 5, not configurable.
- [x] WPM-T211 (2026-10-11) **Flexible working: pure core.** *(traces to WPM-R126, WPM-D77)* `rules/flexible_working.rs`: nine states and exactly the moves between them (tested over every
      state and action), the decide-by date in calendar months with a month-end clamp, overdue only for a request still
      waiting, the request limit over 12 calendar months ignoring withdrawn ones (zero for none), the appeal window in
      calendar days, request validation, and a closed list of refusal reasons a deployer can replace. Nine unit tests.
- [x] WPM-T212 (2026-10-11) **Flexible working: records, API and reminders.** *(traces to WPM-R126)* Migration `000058`, the worker's side under `/api/me/flexible-working` (ask, list, withdraw, respond to a
      counter-proposal, appeal) and `/api/flexible-working` (the queue, decide, decide an appeal) with `GET
      /api/workers/{pid}/flexible-working`. Configurable by the deployer: decide-by months, requests per year, appeal
      window, refusal reasons. The decider is the worker's line manager (anywhere up the chain) or a privileged
      caller, **never the worker**; an appeal is decided by **someone other than who refused**; the manager is told of
      a request and of an appeal, the worker of every decision; the audit names events and holds no text; exported
      and erased with the worker. **An approved change of hours is a proposal (`hr_to_apply_fte_percent`, the
      counter-proposal's hours after an acceptance); the contract is not changed** (verified). Verified in the new
      `enforcement_flexible` binary (the reference policy plus the one rule a deployer would add to let managers
      write), **mutation-checked** four ways (no decider check; the same person decides the appeal; the first ask's
      hours applied after a counter-proposal; no request limit). **Differs from the plan:** "reminders" are the
      `overdue` flag and `?overdue=true` list, not a scheduled task that notifies. **Not done:** that scheduled
      reminder; a screen; the defaults (two calendar months, two requests a year, 14 calendar days) are starting
      points modelled on common rules and are **not verified against any jurisdiction**.
- [x] WPM-T213 (2026-10-11) **Equality monitoring.** *(traces to WPM-R125, WPM-D74)* `rules/equality_monitoring.rs` (the gate; the deployer's categories, validated; the
      aggregate with a floor; completeness), migration `000059`, `controllers/equality_monitoring.rs`, the worker's
      routes under `/api/me/equality-monitoring`, and `GET /api/equality-monitoring/summary` (privileged, audited).
      **Off unless `WPM_EQUALITY_MONITORING_BASIS` records the lawful basis; categories alone switch nothing on; a basis
      with no valid categories stops boot; WPM ships no classification.** Only the worker reads their own answers; the
      subject-access export gives them to the worker and a note to anyone else; no audit entry carries a value; erased
      with the worker; a unit test scans the source so no other code names the table. Verified in the new
      `enforcement_equality` binary (off then on in one process; refusals and limits; the aggregate with cells under
      the floor withheld and the department breakdown withheld unless every cell passes; the export; withdrawal; erasure),
      **mutation-checked** (the gate; the floor, caught in both the integration test and the unit test; the export).
      The DPIA (R13), the record of processing, the regulatory note, the equality impact register and the deployment
      document say so. **Not verified:** the interface (none); the categories a real deployer defines; and the legal
      basis, which is the deployer's. **Known limit:** a department breakdown is all-or-nothing, so a deployer with one
      small department sees only organization-wide figures.
- [x] WPM-T214 (2026-10-11) **Workplace health requirements.** *(traces to WPM-R128, WPM-D75)* `rules/health_requirements.rs` (the gate; standing against a due date and a reminder
      window in calendar days; cleared as a yes or no; applicability by department; the re-check date; validation;
      a compliance aggregate with a floor), migration `000060` (`health_requirements`, `worker_health_records`: a status token
      and two dates, **no column for a diagnosis, product, batch or exemption reason**), `controllers/health_requirements.rs`, the
      worker's view under `/api/me/health-requirements`. **Off unless `WPM_HEALTH_REQUIREMENTS_BASIS` records the lawful basis.**
      **Written only by an occupational-health role:** a new, narrow mechanism, `rules::self_service::ATTRIBUTE_WRITES`, lets a token
      carrying `occupational_health=true` past the HR-only write policy for exactly two routes, and the handler checks the attribute
      again so HR, the service and an administrator are refused. A manager (their reports) and HR get **cleared or not cleared** and
      never the status (checked key by key and by word); the detail is occupational health's and each read is audited; no audit
      entry carries a status; exported to the worker alone; erased with the worker; two unit tests scan the source (nothing else
      names the table; the attribute list matches the routes). Verified in the new `enforcement_health` binary, **mutation-checked**
      five ways (anyone with write rights may record; the manager's list carries the status; no attribute-scoped write in the guard;
      HR may read the detail; the gate ignored). The DPIA (R14), the record of processing, the regulatory note, the domain model, the
      authorization document and the deployment document say so. **Differs from the plan:** "reminders to the worker" are the
      `reminder` flag in their view, not a scheduled notification. **Not verified:** the interface (none); any real occupational
      health arrangement; what a deployer's own rule does with "not cleared" (the software only reports it); and the legal basis,
      which is the deployer's.
- [x] WPM-T215 (2026-10-21) **My details screen, translations and verification.** *(WPM-R124–R130)* `/me/details`, linked from `/me`: contact details (replaced as a whole,
      removable), the emergency contacts component, time off (balances, this year's requests with cancel, past absence, a leave request with no reason field for sick
      leave), flexible working (new request, accept or decline a counter-proposal, appeal a refusal, withdraw, the decide-by date and "overdue"), a resignation (a second
      click to send, withdraw while logged), equality monitoring (off says so; answers changed or taken back one at a time or all) and workplace health requirements
      (off says so; own standing, dates, "cleared" or "not cleared", reminder). Everything is the caller's own through `/api/me`: no worker id is ever sent. 86 strings
      (the page, 9 contact fields, and 8 `values` tokens the vocabularies lacked) in the 13 `-001` locales, written by the assistant and **not reviewed**; the equality and
      health notices are the service's own English text. Playwright `my-details.spec.ts` (7 tests: what each panel sends, the resignation confirmation, the off states,
      no worker record) plus four screenshots read in light and dark at 1280 and 390 px (a phone showed the time-off headings breaking mid-word; the table now scrolls).
      Full suites: Playwright 150, vitest 160, svelte-check 0, prettier clean on `src`. *Not checked:* the page against the real API (the API is stubbed; `enforcement_me`
      covers the routes); a screen reader; right-to-left layout of the new page in Arabic and Urdu (strings only); equality categories and values show as the deployer wrote
      them; the leave-request kinds, flexible-working kinds and resignation reasons are copied into the page from the service's lists.
- [x] WPM-T216 (2026-10-11) **Self-service leave requests.** *(traces to WPM-R129, WPM-R130)* `POST /api/me/leave-requests` and `POST /api/me/leave-requests/{pid}/cancel` on
      the allow-list, sharing the HR route's validation (the balance, a valid span, no reason on sick leave).
      `rules::time_off::can_worker_cancel`: a request awaiting a decision can be cancelled at any time, an approved one
      only before it starts (its days come back); started or finished leave, and one already refused or cancelled, is
      HR's. A worker still cannot approve their own request. Verified in `enforcement_me`, **mutation-checked** twice
      (no cancellation rule; a request of anyone's cancellable).
- [x] WPM-T217 (2026-10-12) **Redis cache, non-personal lookups only.** *(traces to WPM-D78)* Loco's cache provider is now Redis
      in production (`REDIS_URL`) and the demo compose file, and stays in memory in development and test; the `cache_redis`
      feature is on. `src/shared_cache.rs` is a closed list of what may be cached (`Namespace`: organization names, course
      names, ESCO occupation searches), bounded to 250 milliseconds a call and **failing open** (a down Redis is a miss),
      with versioned, generational keys; the ESCO import bumps its generation. Person and worker names never take this
      route. Boot refuses a plaintext `redis://` to another host in production (`WPM_ALLOW_PLAINTEXT_REDIS` overrides).
      **A flaw found and fixed on the way:** the database transport check treated any host starting `127.` or equal to
      `localhost` prefixes as local, so `127.0.0.1.example.com` skipped the TLS requirement; both checks now parse a
      loopback address. Verified against a real Redis 7 in Podman (`WPM_TEST_REDIS_URL`): a value shared between two
      handles, invalidation reaching both, an unreachable Redis a quick miss; the running service writing
      `wpm:v1:esco_occupations:0:…` to Redis and answering a repeat search from it; unit tests for the list, keys,
      the person/worker rule, and the URL checks. **Mutation-checked** three ways (ESCO never stored; the generation ignored;
      person names shared), each making a test fail. CI runs a Redis service. **Four gaps found by checking, then closed:** (1) *TLS did not work*: Loco's
      Redis client is built with no TLS, so the documented `rediss://` could not connect; `redis` is now a direct dependency with
      rustls, and (2) because two rustls crypto providers are in the build and none is chosen, a TLS connection **panicked**;
      `shared_cache::install_tls_provider` now chooses `ring` before Loco opens the cache. Verified against a Redis 7 with
      a password and a certificate from a throwaway CA: the password works, a wrong or missing one is a quick miss, `rediss://`
      connects, and the same server is refused when its CA is not trusted; the production configuration was booted for real with a
      password Redis and a TLS Redis (healthy), with a plaintext remote Redis (refused) and with no `REDIS_URL`. (3) *The stale
      `Cargo.lock`* is regenerated (it now passes `cargo metadata --locked` in the repository itself). (4) *`REDIS_URL` is optional*
      in production: unset, the cache is in memory per instance and the boot logs a warning. (5) The name-lookup path is covered by
      `tests/shared_cache_upstream.rs`, which runs a stub sibling service over real HTTP: an organization name is fetched once and
      shared, one cached elsewhere needs no call, a person's name is found in no shared namespace, and a sibling that is down is a
      miss; **mutation-checked** three more ways (person names shared; never stored; never read). **Not verified:** a managed
      Redis service (cluster, sentinel, ACL users) was not tried; the password and TLS tests need `WPM_TEST_REDIS_*` variables and
      a certificate authority, so CI runs the plain Redis tests only; nothing else is cached yet.
- [x] WPM-T218 (2026-10-11) **Pages for the roles.** *(traces to WPM-R131, WPM-D79)* `locales/en-gb-oxendict/for/<role>/index.md` for ten roles
      (seven chief roles, strategy, security, and governance, risk, compliance), each with a `README.md` symlink, and
      `locales/en-gb-oxendict/for/index.md`; the spec topic is `spec/locales-x-for-roles/`. Each page was written from the specification and
      limited to what is built (key-person risk, the contingent trend and the conversion interface are said to be planned). Verified: the link, days
      and status checks pass. **Not verified:** no person in any of these roles reviewed a page; the facts were read from the spec, not
      re-proven by running the software; only one locale exists.
- [x] WPM-T219 (2026-10-11) **Role pages in the other locales.** *(traces to WPM-R131, WPM-D79)* The ten pages and the index in `en-001`, `en-us`,
      `en-gb` (spelling only), `zh-cn`, `hi-in` and `cy-gb` (translated, one at a time), with `zh-001`, `hi-001` and `cy-001` copied from them. Each
      folder has `index.md`, a `README.md` symlink and a `.locale-peer-id` identical across locales. Verified by a script: every locale has every file,
      the same link targets, headings and table rows as the source, and the same peer ids; the link and days checks pass. **Not verified:** no native
      speaker reviewed the Chinese, Hindi or Welsh text, which was written by an AI and says so on each page; slugs are the same in every locale, not
      native-script; the spelling conversions were checked by search, not by a reader.
- [x] WPM-T220 (2026-10-11) **Role pages in the remaining content locales.** *(traces to WPM-R131, WPM-D79)* The ten pages and the index translated, one
      language at a time, into `de-de` (copied to `de-001`), `es-es` (copied to `es-001`), `fr-001`, `pt-001`, `id-001`, `ru-001`, `ar-001`, `bn-001`
      and `ur-001`, with the same files, peer ids and note as T219. Verified by a script: every folder has every file, the same link targets,
      headings and table rows as the source and the same peer ids; the link and days checks pass. **Not verified:** no native speaker reviewed any
      translation (each page says an AI wrote it); the right-to-left pages (Arabic, Urdu) were not rendered, so their layout is unchecked; role
      names are translated by meaning and may differ from the titles a deployer's organization uses.
- [x] WPM-T221 (2026-10-11) **The UI sends an Idempotency-Key.** *(traces to WPM-R132, WPM-D80)* `api()` in the UI's client adds the header to every
      method but GET, HEAD and OPTIONS, new for each action and reusable on a retry (`idempotencyKey`); `newIdempotencyKey()` falls back to 128
      random bits where `crypto.randomUUID` is missing; the proxy forwards only a plain 16 to 64 character token (`src/lib/idempotency.ts`). 11 unit
      tests (`tests/unit/idempotency.test.ts`), **mutation-checked** four ways (no header; header on reads; a retry's key ignored; the proxy
      forwarding anything), each failing a test. **Not verified:** the browser tests (Playwright) were not run; no real API was called; one
      unrelated unit test (`untranslated.test.ts`, server tokens for equality-monitoring categories with no translation) was already failing
      before this change and still fails. **Not done:** the API ignores the key (WPM-T222).
- [x] WPM-T222 (2026-10-11) **The API recognises a repeated Idempotency-Key.** *(traces to WPM-R132, WPM-D80)* Migration `000064` `idempotency_keys` (caller, key,
      fingerprint, status, content type, response; unique per caller and key); `src/rules/idempotency.rs` (key shape, a length-prefixed SHA-256
      fingerprint of method, path with query and body, what is stored, and the decision: run, replay, clash, wait, take over; 10 unit tests);
      `src/idempotency.rs`, a middleware inside sign-in and need-to-know. Replay, `422` for a different request or a malformed key, `409` while the
      first is running, take-over of a claim older than 300 seconds, failures not kept, expiry after one calendar day, and nothing for a read or a
      keyless request. **Verified** by a request test and by the conversion enforcement binary (the worker, a neighbour and a manager outside the
      chain, sending the proposing manager's own key and request, get `403` and not the stored `200`; HR gets `422`, not a replay). **Mutation-checked**
      five ways (key not tied to the caller; fingerprint ignored; failures stored; never purged; take-over disabled), each failing a test. Also
      the OpenAPI document: a `components` section (the `Idempotency-Key` parameter, the replay header, the `409` and `422` answers, schemas for the
      funding and contingent answers), the key added to every operation that changes something by code, and 4 tests (every `$ref` resolves, only a
      change takes the key, the parameter matches the rules, the funding schemas exist), **mutation-checked** three ways. **Not verified:** behaviour with
      several instances at once or two truly concurrent requests under one key (the claim is a single `INSERT ... ON CONFLICT`, reasoned and not
      raced by a test); a response over 256 KiB is not kept; the UI was not run against this API; the OpenAPI schemas cover only the funding and
      contingent answers, not every older endpoint.
- [x] WPM-T224 (2026-10-11) **A mail server for the demo.** *(traces to WPM-D81)* `deploy/stalwart/` (a `Containerfile` on
      `stalwartlabs/stalwart:v0.16.26` plus the CLI `1.0.13`, `entrypoint.sh`, `plan.ndjson`), a `stalwart` service in `compose.yaml`
      (SMTP on `127.0.0.1:1025`, the management and JMAP port on `127.0.0.1:8025`, healthy only once the plan is applied) that the
      API waits for, `SMTP_HOST` and `SMTP_PORT` for the demo configuration, and `scripts/demo-mailbox.py` to send a test message and
      read the inbox. **Verified by running it (podman 5, Apple silicon):** from nothing the container is ready in about 6 seconds;
      a message to `anyone@wpm.test` over plain SMTP is accepted and read back over JMAP; a message to `someone@example.com` is
      refused with `550 5.1.2 Relay not allowed`; a container restart converges (3 updated, 0 created) and keeps the mail;
      `podman compose up -d stalwart` reports it healthy; the API starts in the `demo` environment with `SMTP_HOST` and `SMTP_PORT`
      set. **Not verified:** that the API itself delivers a message through it (no feature of the service sends email yet: it
      keeps in-app notifications), so delivery was shown with a standard SMTP client only; the full `podman compose up
      --build` of the API image; IMAP, STARTTLS or TLS on any port (the server has a placeholder certificate and the demo uses
      none); other CPU architectures (the CLI binary was copied from the image on this machine, the image is multi-architecture);
      Docker. Found on the way: Stalwart 0.16 starts in a setup mode with no listeners, so the server must be restarted once after
      its setup call, and the CLI needs a writable cache directory (it failed with `Permission denied` as a non-root user).
- [x] WPM-T223 (2026-10-11) **The security-testing brief in every locale, and translated slugs.** *(traces to WPM-R131, WPM-D79)* The page
      `for/independent-security-tester/` (a brief for commissioning an independent test; WPM-T201 stays open) added to every locale: copied or respelled for
      `en-001`, `en-us` and `en-gb`, translated one language at a time for the rest, with the five copy locales recopied. **Slugs are now per locale:** in
      the twelve translated languages (seventeen folders) all eleven page folders are renamed to native-script names and every link between pages, and in
      each locale's index, is rewritten. Verified by a script: every locale has exactly its eleven folders, each with `index.md`, a `README.md` symlink and
      the shared `.locale-peer-id`; link targets, headings and table rows match the source once the slugs are mapped back; the copies equal their sources; the
      link and days checks pass. The brief's folder in each translated locale was then renamed to the translation of the topic, "Independent security testing", and not of the English folder name (`独立安全测试`, `unabhängige-sicherheitsprüfung`, `اختبار-الأمن-المستقل`). **Not verified:** no native speaker reviewed any translation or slug (the right-to-left pages were not rendered); non-ASCII
      folder names were created on macOS and have not been checked out on another platform or served over a web path; the French slug avoids an apostrophe
      (`dinformation`) on purpose; the English `Independent security tester` slug names a role while the page is a brief, which the user chose.
- [x] WPM-T225 (2026-10-18) **Export a table as CSV, TSV or Excel.** *(traces to WPM-R133, WPM-D84)* `@lilydesignsystem` calendar 0.6.0,
      data-grid 0.10.0, gantt-chart 0.7.0 and kanban-board 0.7.0. `DataTable.svelte` offers TSV and a Copy button beside CSV and Excel on tables
      with an export name, a labelled pager, a column select for the search, and uses the grid's `emptyValue` and locale sorting in place of its own
      code. Its labels are now built in `src/lib/data-table-labels.ts` (pure) and gain `filterAnnouncement`; eight keys (`dataTable.exportTsv`,
      `copy`, `copied`, `copyFailed`, `pagination`, `searchField`, `searchAllFields`, `filtered`) in all 12 `-001` locales (AI-written,
      unreviewed). **Found by auditing what each export holds, and fixed:** the metrics export had an empty Value column (a cell snippet exports
      nothing), the pay-band export had bare minor units with no currency, and the training-analytics completion export lacked its workings; each
      now has an `exportValue`. **No action column** exists on the five export tables, so nothing needed `exportable: false`; a future one must
      set it. Verified: svelte-check 0 errors; 145 unit tests, including each locale's bundle against the grid's `missingLabels` (it failed
      13 tests when a label was removed) and the exact list of tables that pass `exportName` (it failed when one was renamed); 130 Playwright
      specs, including TSV, Excel and Copy from `cpd-overview` and `tests/e2e/export.spec.ts` (metrics value column and dashes, pay bands with
      currency; both failed when the `exportValue` lines were removed). **Not verified:** that export covers every page of the filtered rows and
      not only the visible page; the `.xlsx` was checked for its name, not opened; no screenshots of the new select, pager or buttons in any locale
      or on a phone; training-analytics and contingent-departments exports were not downloaded; that `pay-bands` names no one was taken from its
      name; the grid's development warning for missing labels was not read in a browser console.

- [x] WPM-T226 (2026-10-19) **Worker matcher crate.** *(traces to WPM-R134, WPM-D85)* New crate
      `workforce-planning-management-matcher` in `workforce-planning-management-matcher-with-rust/` (own `[workspace]`, no database, no clock;
      deps `chrono`, `serde`, `strsim`, `thiserror`, `unicode-normalization`). `WorkerRecord`, `MatchingEngine`, `MatchConfig` (default, `strict`,
      `lenient`; serialises; validated), `Decision` (`match`/`review`/`no_match`/`insufficient`), `Basis`, `Breakdown`, `NicknameTable`, a normalizer
      (diacritics, email with opt-in Gmail folding, phone to `+digits`, worker number, Soundex) and a scorer; batch `match_one_to_many` and
      `rank_one_to_many`. Pins: unknown is `null` not zero; below `min_fields` is `insufficient`; the same worker number matches only within one
      organization; a different date of birth or a different number in the same organization caps `match` at `review`; `Debug` shows no values;
      the result is symmetric. Verified: 37 unit tests, 7 contract tests and 1 doctest pass; clippy `-D warnings` and `cargo fmt --check` clean;
      four mutations (cap removed, `min_fields` ignored, organization ignored, match threshold unreachable) each failed at least one test.
      **Not done:** the service does not call it (no import or merge wiring, no endpoint); published to crates.io as 0.1.0 on 2026-10-19; no fuzz or property tests;
      the thresholds and weights were tuned on no data, real or synthetic, and the nickname table is English only; no benchmark; no address, no
      transliteration across scripts (a Cyrillic and a Latin spelling of one name will not match).

- [x] WPM-T227 (2026-10-19) **Data-table labels fall back to the Lily label pack.** *(traces to WPM-R135, WPM-D86)* `src/lib/data-table-labels.ts`
      gains `loadPack`, `packLocale` and `withPackFallback`; `DataTable.svelte` loads the pack for `i18n.locale` and lays the catalogue's labels over
      it. Pins: the catalogue wins; a key the catalogue names as undefined stays unset; a regional locale uses its language's pack; each of the
      13 content locales has a pack (loaded lazily, one chunk each). `vite.config.ts` inlines the package for
      Vitest because its `dist/index.js` imports `./label-pack` without an extension. Verified: vitest 160 pass, `svelte-check` 0 errors, build
      and all 135 Playwright specs pass. **Not done:** no check of what a reader hears from the pack's wording, and the packs are unreviewed
      machine drafts; the pack's other helpers (kanban, calendar, Gantt) are not used here.
- [x] WPM-T228 (2026-10-20) **Possible-duplicate check.** *(traces to WPM-R136, WPM-D87, WPM-R134)* The service depends on
      `workforce-planning-management-matcher` 0.1 from crates.io. `src/rules/duplicates.rs` (pure: `split_name`, `possible_duplicates`),
      `src/controllers/duplicates.rs` (`POST /api/workers/possible-duplicates`), the route in `app.rs`, and the OpenAPI entry (the
      route-coverage test caught its absence). Pins: a name alone is `review`; the same number in one organization is `match` with basis
      `worker_number`; another organization's number is no evidence; one word lists no one; at most 10; no email, phone or birth score in
      the answer; read-only. Verified: 9 unit tests and 2 request tests pass; the whole unit suite (570), `mod` (97), `enforcement`,
      `enforcement_expenses` and `security` pass against Postgres 18; clippy `-D warnings` and `cargo fmt --check` clean. **Not done:** no UI
      (the hire form does not call it), no match-on-import, no merge, no per-organization thresholds; not exercised with signed-in callers of
      differing scope (the scope and `Read` checks reuse the directory's, but no enforcement-matrix test covers this route).
- [x] WPM-T229 (2026-10-20) **Chaos testing.** *(traces to WPM-R137, WPM-D88)* `tests/chaos.rs` (own binary, `#[ignore]`d; tokio dev features
      `io-util`, `sync`, `time`): a TCP proxy to Postgres with modes pass, reset, blackhole and slow; a baseline write; reset and blackhole
      each checked for a bounded `5xx` that leaks nothing, no leftover row, and recovery; a slow database; then a seeded monkey of 30 rounds
      of 3 writes in a random mode. Asserts every status is `200` or `5xx`, no leak, workers present are at least the successes and at most the
      attempts, and each has exactly one `created` audit row. New CI step in `api-db`. Verified: passes with three different seeds
      (about 20 seconds each; 54, 39 and 54 of 90 writes succeeded); clippy and `cargo fmt --check` clean. It first failed, and the
      cause is recorded in WPM-D88 (a silent database holds a pool slot until the socket dies). **Not done:** no mutation check that the
      atomicity assertion fails when a write is deliberately split; faults on Redis, the identity service and the mail server are not
      injected; the monkey's requests run one after another, not concurrently; no latency percentiles; only `POST /api/workers`
      is written under fault.
- [x] WPM-T245 (2026-10-21) **API versioning by header, stated and guarded.** *(WPM-R143, WPM-D98)* The behaviour was built in `src/version.rs` from the
      start; this records it as a requirement and a decision, and adds a test (`version::tests::no_route_carries_a_version_segment`) that fails if any
      path in the OpenAPI document is `/api/v<digits>/…`. **Not checked:** the test reads the documented routes, not the router, so an
      undocumented versioned route would not be caught.

## Phase 22 — code quality (assessed 2026-10-21; [plan.md](plan.md) section O)

No new requirement: each task restores a rule in `AGENTS/`. The four defects come first.

- [x] WPM-T246 (2026-10-21) **Training-plan writes name who made them.** *(WPM-R15, WPM-R16)* `controllers/training_plan.rs`
      `add_course`, `remove_course` and `set_hours` take the caller and write an audit row (`skill_course` created and
      deleted; `skill` updated, with the new figure) in the same transaction as the change. The planned role check was not
      needed: the request-level policy already lets only a `Write` holder through (the audit's first reading was wrong;
      plan.md section O says so). Verified: `training_plans_…` request test asserts the actions; with the removal's audit
      action changed it failed (`["created", "nothing"]`); clippy `-D warnings` and `cargo fmt` clean. **Not checked:** no
      enforcement-matrix row was added for these routes (they are covered by the blanket policy test, not by name).
- [x] WPM-T247 (2026-10-21) **A GET changes nothing.** *(WPM-R25, WPM-R26)* `GET /workers/{pid}/wellbeing-prompts` no longer
      stamps `reminded_on`: the multi-dose reminder stays on the list until the worker answers it, and answering it (any
      response) records `reminded_on`, in one transaction with the audit row. The `expect` is gone because the GET no longer
      needs the acknowledgement. Verified: `wellbeing_round_trip` now fetches twice (same answer), answers, then is quiet;
      with the old stamp put back it failed; the other three wellbeing and training tests pass. **Not checked:** the UI was not
      changed or run (its answer buttons already post an acknowledgement, which is what now uses the reminder up).
- [x] WPM-T248 (2026-10-21) **The proxy forwards an allow-list.** *(WPM-D52)* `src/lib/server/proxy.ts` (pure: `forwardHeaders`,
      `isSafeApiPath`); `routes/api/proxy/[...path]/+server.ts` uses both. Forwarded: `accept`, `content-type`, `if-none-match`, a
      plain idempotency key, `accepts-version`, the exchanged bearer and `x-forwarded-for` set to `getClientAddress()`. A browser's
      `authorization`, cookie and `x-forwarded-for` are never passed on; an empty, `.` or `..` segment or a backslash is `400`
      before any call. `production-readiness.md` says so beside the rate-limit note. Verified: 8 tests in
      `tests/unit/proxy.test.ts`; with the old copy-every-header line put back, the no-session test failed; all 168 vitest,
      svelte-check 0 errors, prettier clean. **Not checked:** Playwright (its stubs answer in the browser, so the server route
      never runs there); a real browser against a running API; whether a deployment's adapter reports the right client address
      behind another proxy.
- [x] WPM-T249 (2026-10-21) **The CMS page is pinned and has a policy.** *(WPM-D52, WPM-R73)* `src/lib/server/admin-page.ts`:
      Sveltia CMS 0.230.0 (released 2026-10-06, chosen as at least two weeks old) from unpkg with its `sha384` integrity and
      `crossorigin`, and a CSP for the `/admin` response: the inline language script by its `sha256` hash, Sveltia's documented
      sources (its security page), and `api.github.com`, `www.githubstatus.com` and `avatars.githubusercontent.com` for the
      GitHub backend (the hosts read from the pinned bundle). `production-readiness.md` no longer lists it as a known gap.
      Verified: 2 unit tests (exact version, integrity shape, the hash matches the inline script in three locales, no
      `'unsafe-inline'` or `'unsafe-eval'` for scripts); in Chromium against `vite preview`, `/en-001/admin`, `/cy-001/admin`
      and `/ar-001/admin` render the sign-in screen with no console error or policy violation (Arabic in Arabic, so the hashed
      script ran; Welsh maps to `en-GB` by design); with one character of the hash changed, Chromium blocked the script and
      reported the same digest. Build, svelte-check 0 errors, 170 vitest, prettier clean; Playwright 149 passed, 4 skipped,
      1 failed (`pickers.spec.ts:69`, a hover-position spec; it passed 3 of 3 alone, so it is flaky under load, not caused here).
      **Not checked:** signing in with GitHub, or editing and saving an entry, under the policy (no GitHub login was used), so
      a host the CMS needs only after sign-in may still be missing; Firefox and Safari.
- [x] WPM-T250 (2026-10-23) **Write and audit in one transaction.** `mutate(ctx, |txn| …)` in `controllers/mod.rs`; then move the about 154
      `Audit::record(&ctx.db, …)` calls into the write's transaction, one controller per commit. Decide (plan.md section 6)
      whether `Audit::record` should stop accepting a bare connection. A test that a failing audit insert rolls back the change.
      Done: every `Audit::record` of a change now takes the change's transaction; the 19 left on `&ctx.db` are audits of reads
      (listed in `READ_AUDITS`, guarded by a source test in `models/audit_logs.rs` that fails on any new one outside a
      transaction). Reads that used the pool were moved before `begin()` (the test pool has one connection, so a pool read
      inside a transaction waits forever): `funding`, `grades`, `me`, `escalations`, `oversight_packs` (`pack_payload` then
      `insert_pack`). `career::aspiration_target` split out for the line lint. `Audit::record` stays generic over the connection
      (reads need it). Test: `a_change_whose_audit_row_fails_is_not_kept` (a trigger refuses the audit insert; the skill and
      catalogue course are not kept).
      Verified: clippy pedantic clean, 613 unit tests, the full `--ignored` request suite against a local Postgres, all passing.
      **Not checked:** the UI suites (no UI change); concurrency under load beyond the existing chain test.
- [x] WPM-T251 (2026-10-23) **Shared controller helpers.** `writable_worker` (×9), `ensure_org_in_scope` (×6 plus 23 inline), `ratio_json`
      (×6), a `forbidden(msg)` beside `unprocessable` (about 9 hand-built 403s), and `find_in` in place of local `BATCH`
      constants (`workforce.rs:742`, `key_person.rs:50`). No behaviour change; the request suites pass unchanged.
      Done: `controllers/mod.rs` now owns `writable_worker`, `ensure_org_in_scope`, `ratio_json` and `forbidden`; the local copies
      (7, 6 and 7 files, and the planning and capacity modules) are gone, about 12 inline scope checks call the helper, 11
      hand-built 403s call `forbidden`, and `workforce.rs` and `key_person.rs` use `IN_BATCH` rather than their own constant.
      The two controllers that take a `Uuid` pid (`groups.rs`, `rota_swaps.rs`) keep their own `writable_worker`.
      Verified: clippy pedantic clean, 613 unit tests, the full `--ignored` request suite (115) against a local Postgres, unchanged.
      **Not checked:** inline scope checks that return something other than `NotFound` (`duplicates.rs`, `reporting.rs`,
      `groups.rs`) were left as they are.
- [x] WPM-T252 (2026-10-23) **Responses are views, not rows.** About 46 `format::json(row)` responses (`development.rs`, `workforce.rs`,
      `ergonomics.rs`, `appraisals.rs`, `adjustments.rs`, `hr_core.rs`, `learning.rs`, `notifications.rs`) get `*_json` view
      functions. A guard test that no documented response schema carries `id` or `deleted_at`.
      Done, with a deliberate narrowing: the 27 handlers that answered `format::json(row | updated)` (a bare stored row) now
      answer `row_json(&row)?` (`controllers/mod.rs`), which drops the surrogate `id` and `deleted_at`. That is a deny-list of two
      keys, not a per-entity allow-list of `*_json` views: the entities carry only domain fields the responses already
      exposed, so the leak was those two keys. A new column that must not be disclosed still needs a view. Guards: a unit test
      of `row_json`, and a source test that fails on any `format::json(row | updated | saved)`. The "documented schema" guard in
      the task is not added: `openapi.rs` documents few response schemas and none carries those keys.
      Verified: clippy clean, 615 unit tests, the `--ignored` suite (115): 114 first run, the one failure
      (`engagements::contractor_details_and_assessments`) is a pre-existing flake (it asserts the audit JSON lacks "45000", which a
      hex hash can contain) and passed 3 of 3 alone. **Not checked:** the UI against the changed bodies (it was grepped for `id` and
      `deleted_at` reads: none found).
- [x] WPM-T253 **Arithmetic into the pure core.** `rules::display::{ratio, from_hundredths, from_tenths}` replaces the three
      fixed-point copies and the inline divisions (`learning.rs:1238`, `wellbeing.rs:497`, `cpd.rs:43`, `governance.rs:23`,
      `planning/forecast.rs:50`); pure `*_value` functions for `working_time`, `planning/forecast` and `planning/cost`, with unit
      tests.
      Part 1 done (2026-10-23): `rules::display::{ratio, from_hundredths, from_tenths}` (with unit tests) replace the three
      fixed-point copies (`governance::hours`, `cpd::units`, `forecast::centi`), `talent::ratio` now builds on `display::ratio`,
      and the inline divisions in `learning.rs`, `wellbeing.rs` and `planning/plans.rs` call it. Verified: clippy clean,
      617 unit tests, the `--ignored` request suite (115) against a local Postgres.
      Part 2 done (2026-10-26): the decisions inside those handlers are pure and tested, the handlers keep only reads and JSON:
      `rules::cost::{group_annual, budget_in}` (the reason a group has no cost, in order; a budget only in its own currency),
      `rules::planning::proficiency_split` (the bar is inclusive), `rules::working_time::{breaches_by_worker, is_flagged}`.
      The handlers were not split into whole `*_value` functions: the queries stay where they are. Verified: clippy pedantic clean
      on the lib and bins, 821 unit tests, the `--ignored` request suite (155) against Postgres 18. **Not verified:** no mutation
      check on the new rules; responses were not diffed byte for byte before and after (the request tests pass).
- [ ] WPM-T254 **Paged lists and counts in SQL.** `Page` and `check_offset` on worker- and org-scoped lists (about 68 unpaged);
      `GROUP BY` for `groups.rs:80 member_counts`; filtered reads for `intelligence.rs:653` and `:729`. Seed-200k measurement
      before and after, recorded in `BENCHMARKS.md` (WPM-R142).
      Part 1 done (2026-10-23): `groups.rs member_counts` is a `GROUP BY` in the database, not every membership row counted in Rust
      (the three group request tests pass; clippy clean).
      Part 2 done (2026-10-23): the whole-service and whole-organization lists page: `controllers::{window, paged_select, paged_vec}`
      with `LIST_DEFAULT_LIMIT` 500 (what the capped lists already returned), `x-total-count`/`x-limit`/`x-offset`, and a `400` past
      `MAX_OFFSET`. Counted and cut in SQL: cutovers, requisitions, payroll runs, organization confederations, oversight packs,
      a worker's notifications (unread first, then newest). Cut after the per-row scope or role filter, which the database cannot
      apply: movements, resignations, flexible working, conversion plans, the expense queue. The body is still a plain array
      (the object for packs), so no UI caller changes: the UI's `wpm.ts` calls pass no `limit` and read no headers, so each list
      page shows its first 500. Verified: clippy clean, 619 unit tests, `--ignored` suite 116 incl. the new
      `org_lists_carry_paging_headers_and_refuse_a_deep_offset`.
      Part 3 done (2026-10-26): `intelligence.rs` `succession` and `pipelines` read only the columns they use (plan and readiness; pipeline and
      stage; programme, status and outcome) and group them once, instead of loading every candidate, member and placement row and
      re-filtering the whole list per plan. Verified: clippy pedantic clean, the `--ignored` suite (155, incl. `talent.rs`) against
      Postgres 18. **Not measured:** no timing; the gain is by construction, not by a benchmark.
      Part 4 done (2026-10-26): the UI's "more" control, on one list. The proxy forwards `x-total-count`, `x-limit` and `x-offset`;
      `apiPage` in the client returns the rows with them (a missing or non-numeric header is `null`, never 0);
      `listRequisitionsPage`; `LoadMore.svelte` ("Showing 500 of 501" and a button, nothing when there is no more; `paging.*` in the 13 locales,
      AI-written, unreviewed); `/requisitions` loads 500, shows the control, appends the next page, and a reload after a move or a create re-reads
      every page already loaded. Verified: svelte-check 0 errors, 213 unit tests (3 new for `apiPage`), `pnpm run lint`, Playwright 238 passed / 19 skipped
      (new: a long list pages, a short list has no control; two existing stubs adjusted for the `?limit=500` query). **Not verified:** no screenshot
      looked at; the proxy header forwarding has no test of its own (the stub supplies the headers); only requisitions is wired.
      Part 5 done (2026-10-26): the control is shared. `lib/paged.ts` (pure: `readPages`, `readNext`, `hasMore`; unit-tested, 5 tests) and
      `lib/paged.svelte.ts` (`createPagedLoader`: the rows, total, more flag, a guard against an older answer, a failed "show more"
      kept apart from a failed load so the rows stay). On `/requisitions` (now using it), `/expenses`, `/payroll` and `/movements`;
      `/expenses` starts again from page one when the status changes. Verified: svelte-check 0 errors, lint, 218 unit tests, Playwright 239
      passed / 19 skipped (new: payroll pages, and a failed second page keeps the first). **Not verified:** no screenshot looked at;
      expenses and movements have no paging test of their own (the shared control and loader are tested through requisitions and payroll).
      Part 6 done (2026-10-26): the open conversion plans on `/contingent` page the same way (each page read is audited, and a page is read
      only when asked for). The other lists have no UI caller (cutovers, org-wide resignations and flexible working, a worker's notifications),
      so there is nothing to wire until a screen reads them. Verified: svelte-check 0, lint, 218 unit tests, Playwright all passing.
      **Not verified:** no screenshot; no paging test on `/contingent` (the shared control is tested on requisitions and payroll).
      **Still open:** per-worker lists (bounded by one person's history, left unpaged on purpose); the early-warning list (its totals
      cover all rows); the seed-200k before/after in `BENCHMARKS.md` (no 200k seed was run). This task stays open for those.
- [x] WPM-T255 (2026-10-23) **Split the largest controllers.** `learning.rs` (1,433 lines) into a module folder as `planning/` is; a pure
      plan-builder out of `handover::apply` (235 lines) and `lms::apply` (178).
      Done: `controllers/learning.rs` (1,433 lines) is the folder `learning/` (`catalogue` 709, `paths` 204, `mentoring` 171,
      `views` 289, and `mod.rs` with the routes), routes and behaviour unchanged. `handover::apply` (235 lines) is a dispatcher
      over one function per kind, with the pass-or-close decision a pure `rules::movements::disposition` (unit-tested);
      `lms::apply` (178) calls `sync_enrollment` and `record_credit`, and `rules::lms::is_unchanged` is pure and tested.
      The plan-builder is the per-kind function plus the pure decision, not a single plan value: each kind's facts come from
      reads that follow its first read, so a plan could not be built before the transaction without changing what is locked.
      `catalogue.rs` is still the largest section at 709 lines.
      Verified: clippy pedantic clean, 621 unit tests, the `--ignored` suite (116), unchanged.
- [x] WPM-T256 (2026-10-23) **Dependency hygiene.** `reqwest` 0.13 to match the verifier; drop `sqlx-sqlite`; one `sha2` major if the
      upstream allows. `cargo tree -d` before and after in the task note.
      Done: `reqwest` is 0.13 (feature `rustls`, as the verifier uses), so the 0.12 copy is gone; `sqlx-sqlite` is dropped from
      the `sea-orm` features and is out of the tree. `cargo tree -d`: 92 duplicate entries before, 90 after. `sha2` stays at two
      majors (0.10 comes from `sqlx-core` and `ed25519-dalek`, which the service does not control); the service's own 0.11 stays.
      Verified: clippy clean, 621 unit tests, the `--ignored` suite (116).
      **Found, not fixed:** `cargo deny check advisories` fails on RUSTSEC-2026-0385 (`ctutils` 0.4.3, via `digest` 0.11; the
      same version is in the committed lock, so a newly published advisory, not this change), and `deny.toml` carries about ten
      ignore entries that no crate matches any more. Both want their own task.
- [x] WPM-T257 (2026-10-23) **The matcher follows the house lints and is built from the sibling.** `#![forbid(unsafe_code)]`,
      `#![warn(clippy::pedantic)]`, `#![deny(missing_docs)]` (or a `[lints]` table); annotate `scorer.rs:16`; unit tests for
      `record.rs` and `error.rs`; a `[patch.crates-io]` path override in the service for development.
      Done: the matcher's `lib.rs` carries `#![forbid(unsafe_code)]`, `#![warn(clippy::pedantic)]` and `#![deny(missing_docs)]`;
      the 28 pedantic findings and every undocumented field and method are fixed (`# Errors` sections, `#[must_use]`, an
      associated function for `compare_worker_numbers`, one annotated cast in `scorer.rs`). `record.rs` and `error.rs` have unit
      tests (builders, the impossible date, `Debug` showing counts and never values, messages). The service's `Cargo.toml` carries
      the `[patch.crates-io]` path override as a commented block: committed live it would break the Containerfile build and
      rewrite `Cargo.lock`.
      Verified: matcher clippy clean, 50 tests; the override was switched on once and the service checked against the sibling.
- [x] WPM-T258 (2026-10-23) **Request tests for every route.** The about 32 routes in no test (`development.rs` ×8, `movements.rs` ×4,
      `change.rs` ×3, `handover.rs`, `audits.rs`, `expenses.rs` withdraw, `hr_core.rs`, `workforce.rs`); a route-coverage test
      that fails on a route no request test calls; more unit tests for `benchmark.rs`, `tokens.rs`, `audit_export.rs`,
      `adjustments.rs`, `framework_roles.rs` and `job_levels.rs`.
      **Part 1 (2026-10-23):** new request tests for `development.rs` (the review lifecycle, goals, feedback, redaction until
      shared, expiring certificates, succession restate and gaps), `movements.rs` (add item, assign, reopen, cancel),
      `change.rs` (initiative read-back, removing impacts and shifts, the department audit and event views),
      `workforce.rs` (unassign a shift seat) and `hr_core.rs` (end a benefit enrolment). Request suite: 123 pass, clippy clean.
      **Part 2 (2026-10-23):** `mobility.rs` request tests for `opportunities` (own organization only, own title excluded,
      drafts excluded, unknown fit null) and for `/api-docs/openapi.json` and `/swagger-ui`. Found and fixed: the Swagger page
      title read "Patient Flow Service API", left over from another project. Most of the other routes my first scan listed
      are in fact exercised (`framework_roles`, `assessments`, the `/refs/`, `/past`, demand-line and skill-pool deletes,
      expense withdraw, handover actions: tests build those URLs from variables), and `health-requirements` and
      `equality-monitoring` are covered in the `enforcement_*` binaries, not in `tests/requests`.
      **Part 3 (2026-10-23):** unit tests added to `rules::tokens` (every vocabulary non-empty, unique, `snake_case`, listed from
      the source), `benchmark`, `audit_export` (exactly the eight agreed keys, a null actor, the offset kept), `adjustments`
      (the full status matrix: exactly five legal moves), `framework_roles` (limit inclusive, trimmed duplicates) and
      `job_levels`. The controllers named in the task (`adjustments`, `framework_roles`, `job_levels`) hold no pure logic of
      their own; their rules are the modules tested here. Two defects found and fixed in `rules::job_levels`: `level("LL5")`
      resolved to L5 (every leading `L` was stripped; now one prefix), and `next_above(255)` overflowed `u8`.
      Verified: 639 unit tests (was 621), clippy clean.
      **Part 4 (2026-10-23), closing:** `tests/route_coverage.rs` (DB-free, runs in plain `cargo test`) reads every route and
      method from `app::app_routes()` (new; `Hooks::routes` calls it) and fails on a route no test calls. A call is found by
      reading the test sources: a literal URL, or a name bound to one by `let` or by a `for url in [...]` list. Eight
      routes (and the health-requirement `PUT`/`DELETE`) are called through helper closures or tables the scan cannot follow;
      they are in `BUILT_FROM_A_VARIABLE` with the test that calls each, and the test fails if an entry becomes redundant or
      names no route. It is a text scan: it cannot tell that a call asserts anything, and a tail such as `{base}/handover/actions`
      matches by its last two segments, so it can over-credit. Its first method-aware run found real gaps, closed here: the
      collection reads (`lists.rs`: eleven lists and the health probes), `removals.rs` (backup, pool member, demand line,
      tolerance, group and wellbeing-rule restatement, worker and application reads), framework-role clear, and the health
      requirement restate and retire (HR may restate; only the service may retire).
      Verified: request suite 130 pass (Postgres 18, Podman), `enforcement_health` pass, 639 unit tests, clippy clean. The other
      `enforcement_*` binaries were not re-run (only the one edited); `enforcement_health` drops the test database when it ends,
      so run it last or recreate the database.
- [x] WPM-T259 **One error vocabulary.** An internal-invariant helper (logs, answers 500 with no detail) for `Error::string`
      uses (`intelligence.rs:471`, `resignations.rs:59`); drop the bespoke 401 in `funding.rs:86`.
      Done: `controllers::internal(what)` logs the cause and answers 500 `internal error` with no detail; `controllers::unauthorized`
      replaces the bare `Error::Unauthorized` in `funding.rs` and `flexible_working.rs`. Every `Error::string` in `src/controllers/`
      (intelligence, resignations, flexible working, CPD, ESCO, equality monitoring, `row_json`) now uses `internal`. The ones in
      `app.rs` and `src/tasks/` stay: they are start-up and command-line errors read by an operator, not HTTP answers.
      Verified: 640 unit tests (one new), 130 request tests (Postgres 18, Podman), clippy clean. Not run: the `enforcement_*`
      binaries; no test drives an invariant failure through HTTP, so the 500 body is covered by the unit test only.
- [x] WPM-T260 **One loader with a stale guard.** `createLoader()` in `src/lib/` owning loading, error, data and the
      late-response guard; move the about 40 unguarded `$effect` loaders onto it. The worker page uses `Promise.allSettled` and
      an error per panel, so a failed secondary panel hides only itself. A unit test of the guard.
      Part 1 (this commit): `src/lib/loader.svelte.ts` has `createLoader` (data, error, loading, and a late-response guard via
      `latestOnly`) and `settleEach` (each panel's outcome kept apart). Moved onto it: `workers`, `payroll`, `payroll/[pid]`,
      `requisitions/[pid]`, `mentorship`, `development`, `me`, `privacy`, `workforce`, `wellbeing`. `workers/[pid]` loads the worker
      first (a failure there is the page's error), then the thirteen secondary panels with `settleEach` under the same guard;
      a failed panel shows its own `panel-error-<name>` under its heading and the rest still shows.
      Verified: 7 loader unit tests (stale answer and stale error both dropped), 177 unit tests, `svelte-check` 0 errors,
      152 Playwright specs pass (two new in `loader.spec.ts`).
      Not done: the remaining unguarded `$effect` loaders. Several already carry their own `current`/`stale` guard (capacity, ceo,
      contingent, cutover, job-levels, pay-scales, skill-gaps, warnings, resilience); the rest mix load errors with action errors in
      one `error` variable (movements, metrics, groups, planning, roles, rota, skills, learning, cpd, change, kanban,
      requisitions, org-chart, benchmarks, and the `src/lib/components/` panels) and need that split first. No Playwright test
      races two slow responses; the guard is covered by the unit test only. Left open (`[~]`).
      Part 2 (closed): `guardedLoad(fetcher, apply, fail)` in `src/lib/loader.svelte.ts` is the guard without the state, for the
      pages and panels that keep their own; it drops the answer or the failure of a load that a newer one overtook. 40 files
      now use it (the My details panels, the worker's panels, `movements`, `metrics`, `groups`, `skills`, `requisitions`,
      `kanban`, `expenses`, `rota`, `learning`, `cpd`, `change`, `planning`, `roles`, `org-chart`, `benchmarks`, `directory`, the
      dashboard tiles and the rest), 10 use `createLoader`, and 12 keep a guard of their own (listed in
      `tests/unit/loaders-are-guarded.test.ts`, which fails when a new `async function load…` has neither). Where a reset
      comes first (`loadDetail`, `loadView`, `loadWorker`, `loadProgress`) the reset stays in a small wrapper and the read is
      guarded. A multi-step read now applies all of its answer or none, so `JobLevel`, `PayPosition` and `RoleGrade`, which used to
      show the level before the ladder arrived, show nothing new if the ladder fails. Verified: 209 unit tests (three new for
      `guardedLoad`, three for the allowlist), `svelte-check` and prettier clean, 206 Playwright specs. Not done: the
      one-shot mount loads (`RoleGap`, `ContactDetailsPanel`, `HealthPanel`, and the first read on `change`, `cpd`, `learning`,
      `planning` and `roles`) are unguarded because nothing they read changes; still no browser test races two slow responses.
- [x] WPM-T261 **Errors are content.** Every catch goes through `failure()` (`src/lib/write.ts`); status codes 401, 403, 404,
      409, 422 and 5xx map to `t()` keys in the 13 locales (AI-written, marked unreviewed); one `<ErrorNote>` with
      `role="alert"` replaces the 85 `<p class="error">`. Extend `untranslated.test.ts` to script-side error strings.
      Done: `errorKey`/`errorText` in `src/lib/write.ts` map 401, 403, 404, 409, 422, 5xx and a network failure to `errors.*` keys
      in all 13 `-001` locales (AI-written, unreviewed: listed in `content/locales/REVIEW.md`). A 409 or 422 adds the service's own
      message after the words; a 5xx adds nothing. All 77 catch sites now call `errorText`, `createLoader` too, and `failure()`
      keeps the 403 "not allowed" apart. `ErrorNote.svelte` (`role="alert"`) replaced all 84 `<p class="error">` (and the one in
      `WriteResult` and the sign-in page). `untranslated.test.ts` now fails on a script that turns an exception into text itself,
      on an English sentence assigned to an error, on an error paragraph outside `ErrorNote`, and on a missing `errors.*` key.
      Verified: 188 unit tests, `svelte-check` clean, prettier clean, 152 Playwright specs (one assertion changed: a refused write
      now reads the catalogue's words and then the service's message). `pickers.spec.ts` failed once in the full run and passed on
      rerun with the oversight spec. Not checked: the translations by a speaker of each language; no screenshots were looked at;
      the pages still showing a raw service message for a 4xx other than 409/422 now show generic words, which loses detail by design.
- [x] WPM-T262 **Titles and labels.** `+page.ts` with `pageTitle(...)` for the twelve untitled pages; a label or
      `aria-label={t(...)}` for the about 20 unlabelled controls (`workers/[pid]`, `wellbeing`, `requisitions/[pid]`,
      `movements/[pid]`); move the wellbeing placeholder text into the catalogue. An axe check in Playwright on those pages.
      Done: the twelve untitled pages (`benchmarks`, `development`, `org-chart`, `payroll`, `payroll/[pid]`, `privacy`, `requisitions`,
      `requisitions/[pid]`, `wellbeing`, `workers`, `workers/[pid]`, `workforce`) have a `+page.ts` using the new `navTitle()`
      ("Workers — WPM", from the navigation name, in the locale of the address) and a `<title>` from `page.data.title`. The 21
      unnamed controls on `workers/[pid]`, `wellbeing`, `requisitions/[pid]` and `movements/[pid]` have an `aria-label` from the
      catalogue (eight new `labels.*` keys in 13 locales, AI-written, unreviewed); the wellbeing departments placeholder is now a
      catalogue string. `tests/e2e/titles-and-labels.spec.ts` checks all twelve titles and one in German, and runs axe
      (`@axe-core/playwright`, new dev dependency) on three form pages plus a stricter check that no control relies on a placeholder
      as its name (axe accepts a placeholder; it also passed with a label removed, the strict check failed, so it earns its place).
      Also changed: `errorText` now adds the service's message to a 403 as well as 409/422, because a 403's message names the rule
      (a conversion-plan spec relied on it). Verified: 188 unit tests, `svelte-check` clean, prettier clean, 168 Playwright specs.
      Not checked: axe on pages other than those three, colour contrast, or the keyboard order; `movements/[pid]` was changed
      but is not in the axe spec; the new labels were not read by a speaker of each language. The `labels.*` additions are
      recorded in `content/locales/REVIEW.md`.
- [x] WPM-T263 **Types from the OpenAPI document.** `openapi-typescript` into `src/lib/api/schema.d.ts`; the hand-written
      functions in `wpm.ts` typed from it, ending the 75 `Promise<unknown>`; a CI check that the generated file is current.
      Done. `openapi.json` is a copy of what the service serves, kept current by the service test `the_ui_copy_is_current`
      (rewrite with `WPM_WRITE_OPENAPI=1 cargo test --lib the_ui_copy`); `schema.d.ts` is made by `pnpm api:types` and kept current
      by `tests/unit/schema-current.test.ts`. Both already run in CI (`cargo test --lib`, `vitest run`), so no workflow change.
      `ANSWERS` in `src/openapi.rs` states the answer of 70 write operations, each read off its handler (and the helper that builds
      the body: `row_json` read against the entity in `src/models/_entities/`, `entry_json`, `role_row`, `dotted_json`, `member_json`,
      `level_json`, `position_json`, `measure_json`, `pack_json`, `view`, `own_json`, `view_for`, `list_tolerances` and
      `escalation_json`); `wpm.ts` types every one through `Answer<path, method>`, so the count of `Promise<unknown>` in it is 0
      (it was 65). Part 2 added 32 operations and 25 shapes (`record_schemas()`): no body and `{}` answers, `{pid}`, stored rows with
      their `created_at` and `updated_at` (a `row_json` row keeps both; only `id` and `deleted_at` go), a flexible-working request (one
      shape for withdraw, respond and appeal), a plan's tolerances (set a tolerance answers with the whole view), equality
      monitoring (`oneOf` the disabled and the enabled branch; declare and both withdraws answer with it) and the dotted-line manager
      (the manager's name fields are optional, as the handler leaves them out when the manager is not found).
      Verified: `cargo test --lib` 642 pass (the test that every `ANSWERS` entry names a real operation and a real schema covers the
      new entries), clippy clean, 209 UI unit tests, `svelte-check` clean, prettier clean; a probe file with `@ts-expect-error`
      lines (the disabled equality branch has no `categories`; a status is not a number) failed to compile when the types were wrong
      and compiled with them, so they resolve and are not `any`; it was deleted. Not checked: the schemas against live responses
      (they are read from the code, not run: the request-test binaries were not run for this); the hand-written read types in
      `types.ts` and inline in `wpm.ts` are still hand-written, since the document has no schemas for the reads; `competencies` on an
      appraisal is stored as JSON and typed as a list of strings because the create handler only accepts that; a `row_json` row's enum-like
      columns (`status`, `method`, `kind`) are typed as plain strings, not enums.
- [x] WPM-T264 **Cache the exchanged token.** Per session id until shortly before expiry, cleared on sign-out; a unit test that
      one page load exchanges once.
      Done: `src/lib/server/token-cache.ts` (`TokenCache`, `tokenExpiry`) keeps the exchanged token per session id in server memory
      until 30 s before its `exp` (read from the PASETO message, not verified: the API still verifies every token), shares one
      in-flight exchange among simultaneous calls, remembers at most 1,000 sessions, and does not keep a token whose expiry it cannot
      read. The proxy uses it; sign-out and a `401` from the API forget the token. Verified: 12 new unit tests (nine on the cache,
      three through the proxy handler: twenty calls exchange once with one bearer; a refused token is exchanged again; sign-out
      forgets), 202 UI unit tests, `svelte-check` and prettier clean. Not checked: a real authentication service (the exchange is
      stubbed), the effect on latency, more than one server process (each keeps its own cache; that is correct but untested), and
      `spec/auth.md` now says a session ended elsewhere works for up to one token lifetime here.
- [x] WPM-T265 **UI dependency and config hygiene.** Drop the five unused Lily picker packages and align `share-picker` with
      `svelte-picker-bar`; `adapter-node` in place of `adapter-auto`; remove `apiConditional`; one shared `write()` for
      oversight, capacity and cutover.
      Done: the five Lily picker packages nothing imported (`link`, `locale`, `search`, `text-size`, `theme`) are gone from
      `package.json`, and so is the direct `share-picker`; `+layout.svelte` now takes the `ShareTarget` type from `PickerBar`'s own
      props, so the bar and its share picker cannot drift. They were duplicates: `picker-bar` brings its own, older copies, which
      are what ran before and still do, so the page is unchanged. `adapter-node` replaces `adapter-auto` (`pnpm build` writes
      `build/`; `spec/operations/deployment.md` says how to run it). `apiConditional` and its `Conditional` type are removed.
      `attempt()` in `src/lib/write.ts` is the one place a failed write becomes a `Failure`; oversight (both its helpers), capacity
      and cutover use it and keep their own after-success step. Verified: 203 UI unit tests (one new), `svelte-check` clean,
      prettier clean, `pnpm build` with the Node adapter, 168 Playwright specs. Not checked: running `node build` against a real
      API, a container image (still none for the UI), or the pickers' look beyond what `pickers.spec.ts` and the layout specs assert.
- [x] WPM-T266 **Specs for every area.** Playwright specs for `/change`, `/skills`, `/groups`, `/development`, `/workforce`,
      `/payroll` and `/requisitions/[pid]`; split `smoke.spec.ts` (2,659 lines) into one file per area.
      Done: `smoke.spec.ts` (2,659 lines) is split into fifteen area files plus `sign-in.spec.ts`, with the fixtures and the signed-in
      stubs in `tests/e2e/support.ts` (`signIn`, `WORKER`, `MASKED_WORKER`, `chooseLocale`, `stubSignedIn`); the same 48 tests run,
      the test bodies unchanged. New specs, 28 tests: `change` (6), `skills` (6), `groups` (3), `development` (3), `workforce` (4),
      `payroll-list` (3), `requisition-detail` (3); each also checks the page title, a refusal in words and the empty state.
      Verified: 196 Playwright specs pass (168 before, 4 skipped as before: the My details screenshots), nothing else changed.
      Not checked: that each new spec fails when the page is broken, beyond the few assertions I saw fail first for the wrong
      expected text; no screenshots were taken or looked at. Found, not fixed: `/skills` still has English in its script
      (the aria-labels `Rename …`, `Category of …`, `Merge … into`, the `confirm()` texts and the merge and delete notices), and
      `/requisitions/[pid]` shows the raw stage in its button (`→ screened`), because the untranslated test scans template text
      and not template-literal attributes or script strings.
- [x] WPM-T267 **Split the largest pages.** Panels and forms of `workers/[pid]` (713 lines), `planning` (697) and `roles` (487)
      into components; screenshots before and after in light, dark and a phone width.
      Done: `workers/[pid]` 785 to 470 lines (components `WellbeingPrompts`, `Appraisals`, `AdjustmentRequests`,
      `ErgonomicAssessments` in `src/lib/components/worker/`), `planning` 699 to 399 (`PlanForecast`, `PlanTolerances` in
      `.../planning/`), `roles` 489 to 231 (`EscoOccupationSeed`, `RoleProfileDetail` in `.../roles/`). Each component owns its
      form state and its writes, and tells the page to read again (`onchange`) or what failed (`onfail`); `PlanTolerances` is given
      the page's `run`. `PLAN_BASES` moved into `wpm.ts` since two components need it. Before and after: `tests/e2e/page-shots.spec.ts`
      (runs only with `PAGE_SHOTS=<dir>`) writes a PNG and a text dump of the three pages in light, dark and a 390 px phone, from
      shared stubs in `tests/e2e/fixtures.ts`; the nine before and nine after PNGs are byte-identical and the nine text dumps
      identical. They are not committed (a run writes them). The moved code had almost no browser tests, so `worker-panels.spec.ts` (6)
      and `roles.spec.ts` (4) were added: each write's body, a refused write shown, the disabled buttons. Verified: 206 Playwright
      specs pass (13 skipped: the screenshot specs without their env var), 203 unit tests, `svelte-check` clean, all run after the
      split. Not checked: that the new specs fail on a broken component; interaction states the stubs do not open (an open
      appraisal's report, a shared appraisal, the tolerance forms beyond `tolerance.spec.ts`, the rest of `workers/[pid]`, still 470
      lines, including the leave and payslip tables and the onboarding and notifications lists) look the same: the screenshots are
      of the default state only. `Appraisals` re-reads the open appraisal whenever the page reads the list again (an `$effect`
      with `untrack`), where before the page did it after `load`; that timing is the one behaviour I changed on purpose.

## Phase 23 — regulatory reporting, integration and planning depth (proposed WPM-R144–R158, WPM-D99–D106; [plan.md](plan.md) section N)

- [x] (2026-10-24) WPM-T268 **Spec round.** *(WPM-R144–R158, WPM-D99–D106)* Check every date and threshold in section N against primary
      text (Directive 2023/970 and a chosen transposition, AI Act Article 26 and the Omnibus, ISO 30414:2025, ESRS S1);
      topic files `pay-transparency.md`, `human-capital-reporting.md`, `integrations.md` additions, `positions.md`; the domain
      model, auth, audit, regulatory, glossary, index and `llms.txt`; move the next-free-ids line.
      Done: every date and threshold in plan section N was checked against the primary text where it could be reached, and the
      corrections and the open items are tabled in [plan.md](plan.md) ("Spec round: what changed"). Corrected: the pay gap indicators
      (seven in Article 9(1), no separate "benefits" share; a bonus is a complementary or variable component), the reporting tiers
      (250 or more yearly from 7 June 2027, 150 to 249 every three years from 7 June 2027, 100 to 149 every three years from 7 June 2031),
      the joint pay assessment trigger (at least 5% in a category, not justified, not remedied within six months of reporting), the ESRS
      S1 requirement numbers (the revised set, Delegated Regulation 2026/1563: S1-5, S1-7, S1-12, S1-15) and the AI Act dates (Annex III from
      2 December 2027, Annex I from 2 August 2028, in force 27 July 2026). New topic files: `pay-transparency.md`,
      `human-capital-reporting.md`, `positions.md`; additions to `integrations.md` (R149–R151), `regulatory.md` (R148),
      `payroll-compensation.md` (R154), `joiners-and-leavers.md` (R155), `communication-and-leadership.md` (R156),
      `skills-and-training.md` (R157–R158), and to the domain model, auth sensitivity map, audit, glossary, index, `llms.json` and
      `llms.txt`. The next-free-ids line was already R159, D107, T286 and is unchanged. Verified: `check-links`, `check-days`,
      `llms.py --check` and `status.py` pass. Not checked, and said so in the files: the Directive and the AI Act were read as search
      excerpts of their EUR-Lex text and the Commission's FAQ and AI Act Service Desk pages, not as whole documents, because EUR-Lex answers a
      script with a bot challenge (not bypassed); no national transposition was read; the revised ESRS and ISO 30414:2025 detail is
      secondary or paywalled; paragraph numbers inside Articles 5 and 7 and the rule for putting part-time pay on a comparable basis are
      open for WPM-T269. This is a spec change only: no code, so no suite was run.
- [x] (2026-10-24) WPM-T269 **Pay gap indicators: pure core.** *(WPM-R144, WPM-D99)* `rules::pay_gap`: the seven indicators overall and per
      category, the floor, exclusion counts, `unknown` for one-sided cells. Exhaustive unit tests, including a cell of one.
      Done: `src/rules/pay_gap.rs` (21 unit tests): `report(workers, floor, basis)` returns the seven indicators (mean and median gap
      on total pay, mean and median gap in variable pay, the share of each sex receiving a variable component, four quartile
      compositions, the mean gap per category in basic and in variable pay) as `Figure::{Shown, Withheld, Unknown(reason)}`, plus
      `Exclusions` counted by first reason. Pinned: the signed gap in basis points, exact for a median of two middle values (kept
      rational until one rounding, half away from zero, in `i128`); a cell under the floor on either side withheld, with a cell of one
      woman never shown; a side with nobody `unknown` (`OneSided`), no one `NoWorkers`, a male figure of zero `NoBase`; a variable component
      nobody recorded is not zero and the worker stays in basic pay only; an exclusion count under the floor shown as such; the floor
      refused below the equality monitoring minimum; no worker key and no head count in the serialized report; amounts near the ends
      of `i64` do not wrap; the Directive's three reporting tiers and `next_due` (including 29 February). Mutating the floor
      comparison, the rounding and the no-base guard each made tests fail. Verified: `cargo test --lib` and clippy pedantic clean.
      Not done and said so: the Directive's own rule for the comparable basis of pay is not read, so the module takes pay already on
      one basis and WPM-T270 chooses and records it; the variable-gap basis is a parameter for the same reason; the median per category
      is not computed; nothing reads the database yet. Spec change: a withheld cell carries no count (`spec/pay-transparency.md`).
- [~] (2026-10-24) WPM-T270 **Pay gap indicators: records, API and screen.** *(WPM-R144)* `pay_categories` with a stated basis; an HR-only
      report; enforcement test that it names no one; CSV export with workings.
      Part 1 (API and records; the screen is not built): migration `m20261024_000080_pay_categories` (`pay_categories`: name, basis,
      job titles as JSONB, `set_by`, soft delete, a unique live name per organization, the `deleted_at` index the retention test
      wants, in the sweep list as Pay); `rules::pay_category` (validate, clash, index; 4 tests) and `rules::csv::field`/`row` (RFC 4180
      output with spreadsheet-formula neutralising); `controllers/pay_transparency.rs`: `GET/POST /api/pay-categories`, `PUT/DELETE
      /api/pay-categories/{pid}`, `GET /api/pay-transparency/gap-report` and `/gap-report/csv`, all in OpenAPI and the need-to-know table.
      What it reads and does is in [pay-transparency.md](pay-transparency.md) and corrects the first draft in three places: the
      report needs unmasked salary read (payroll, administrators; HR is refused), categories match job titles (there is no role-profile
      link on a worker), and WPM holds no variable pay, so only the basic-pay gap by category can be shown today. The guard that only
      the equality module reads the declarations now names this controller. Verified (Postgres 18, Podman): 668 unit tests, 133
      request tests (3 new: categories and their refusals, the report off and refusing bad input, and a full report with CSV and
      audit), `enforcement_pay_gap` (new: a worker 403, HR 403 on the report and 200 on categories, payroll 200, outside scope 404,
      no worker pid, name, number or head count in the report or CSV, no figure in an audit snapshot), `enforcement_equality` and
      `enforcement`, clippy pedantic clean, `route_coverage`. Not done: the **screen** (an HR page for the categories and the report,
      with its 13 locales of strings and a screenshot), so the task stays open; a per-category median; the Directive's own rule for the
      comparable basis; the other `enforcement_*` binaries were not re-run.
- [~] (2026-10-24) WPM-T271 **Pay information for the worker and the candidate.** *(WPM-R145)* `POST /api/me/pay-information-requests` with a
      computed due date; HR's answer screen; a pay range required before a requisition is advertised; a guard test that no
      candidate table has a pay-history column.
      Part 1 (rules, records and API; the HR screen is not built): migration `m20261024_000081_pay_information_requests` (dates
      only; one waiting request per worker as a unique partial index; the `deleted_at` index; in the retention sweep list, the
      subject-access export and the erasure); `rules::pay_information` (due date in calendar months with month-end clamping,
      standing open or overdue or answered, one at a time, the answer with each sex's average withheld under the floor, the pay range
      rule; 8 tests); `controllers/pay_information.rs` and the two `/api/me/pay-information-requests` routes in `me.rs` (on the
      self-service list); the answer works out figures on demand and stores none; the requisition `open` transition now refuses a
      vacancy with no pay range (`WPM_REQUIRE_PAY_RANGE`, on by default), so six existing request-test requisitions were given a
      range. Details and the corrections to the first draft (HR reads salaries masked, so working out the answer is payroll's or an
      administrator's and recording it is HR's; the yearly notice is not built) are in [pay-transparency.md](pay-transparency.md).
      Verified (Postgres 18, Podman): 676 unit tests, 137 request tests (4 new: HR's answer and queue, no equality monitoring, the
      pay range gate, the candidate guard), `enforcement_pay_information` (new: a worker's own requests with the computed due date
      and one at a time, a worker refused the queue, the answer and recording, HR refused the figures, payroll given them, payroll
      refused the write, the answer names no one, the export and the audit hold no figure), `enforcement_pay_gap`, `enforcement_equality`,
      `enforcement`, `enforcement_flexible`, `enforcement_me`, `route_coverage`, clippy pedantic clean, 209 UI unit tests and
      `svelte-check` after regenerating the types. Not done: **HR's answer screen** (and the worker's own page for asking, with the
      13 locales of strings and screenshots), so the task stays open; the yearly notice; a worker's view of an answer (HR gives it to
      them; WPM keeps no copy); the other `enforcement_*` binaries were not re-run.
- [x] (2026-10-24) WPM-T272 **Joint pay assessment trigger.** *(WPM-R146)* Pure rule over the indicators and configured threshold and
      period; a "due" signal linked to the recognised body (Phase 18). It reports; a person records the decision.
      Done: `rules::joint_assessment` (signal unknown, not due, pending or due; threshold either way; the last day in calendar months
      with month-end clamping; a late remedy does not undo it; an assessment outranks a justification; record validation; checked
      configuration; 9 tests), migration `m20261024_000082_pay_assessment_records` (one record per organization, category and report
      date, with the closed outcome list and the justification-iff-justified and outcome-iff-assessed checks), and
      `controllers/pay_assessment.rs` (`GET /api/pay-transparency/joint-assessment`, `GET` and `PUT /api/pay-transparency/assessment-records`).
      It reports; a person records. The link to the recognised body is a recorded name, because Phase 18's bodies are not built.
      Three `values.*` words for the outcomes in the 13 locales (AI-written, unreviewed). Verified (Postgres 18, Podman): 685 unit
      tests, 140 request tests (3 new: the signal across the clock and every recording, a hidden gap and a missing setup, and
      **no audit, no write** (WPM-D107): with an audit table that refuses every insert a category and an assessment record are both
      refused and rolled back), `enforcement_pay_gap` (extended: a worker and HR refused the signal, payroll given it, a worker and
      payroll refused the write, HR recording, no words in the audit), `enforcement_pay_information`, `enforcement_equality`,
      `enforcement`, `route_coverage`, clippy pedantic clean, 209 UI unit tests and `svelte-check` after regenerating the types. Not
      done: a screen for the signal and the records (with the pay gap and pay information screens, WPM-T270 and WPM-T271 part 2);
      the other `enforcement_*` binaries were not re-run.
- [x] (2026-10-24) WPM-T273 **Human capital report.** *(WPM-R147, WPM-D100)* A dated, read-only report from existing measures, each with
      metric name, definition, source and exclusions; `unknown` where WPM cannot compute it; a test that no licensed text is in
      the repo.
      Done: `rules::human_capital` (headcount, and headcount and FTE by employment type; starters, leavers and turnover; time to
      fill, median and mean; training completion; succession coverage of critical roles; each with definition, source, period,
      exclusions and its ISO area and ESRS S1 requirement for **both** the 2023 and the revised 2026/1563 sets; six named metrics WPM
      does not cover listed as gaps; 15 tests), and `controllers/human_capital.rs` (`GET /api/reports/human-capital?organization=&as_of=`,
      HR, payroll, service and administrator callers, in the organization's scope, audited). A count of 1 to 9 is withheld, zero is
      shown, and a rate is withheld with the count that would give it back. Unknown, with a reason: training hours (no source),
      collective agreement coverage (no record), and workforce cost and the pay gap (behind a stricter read, which has its own route).
      The "no licensed text" test is a shape check: everything quoted from a standard is an area, a topic or an identifier of at most
      60 characters with no sentence; it cannot prove that no paragraph was pasted elsewhere. Verified (Postgres 18, Podman): 700 unit
      tests, 141 request tests (1 new: counts, FTE, a small type withheld, turnover from ten leavers, the unknowns, no name, another
      organization empty, the future refused, the audit), `enforcement_pay_gap` (extended: anonymous refused, a worker refused, HR given
      it), `enforcement_pay_information`, `enforcement_equality`, `enforcement`, `enforcement_need_to_know`, `route_coverage`, clippy
      pedantic clean, 209 UI unit tests after regenerating the types. Not done: a screen; the mapping was not compared with ISO's text
      (paywalled) or the revised ESRS datapoints; collective agreement coverage waits for the unions work; the other `enforcement_*`
      binaries were not re-run.
- [~] (2026-10-24) WPM-T274 **AI and scoring deployer register.** *(WPM-R148, WPM-D103)* Extends `rules::scoring::REGISTER` (WPM-T166):
      category, overseer role, informed-on date, log retention; per-decision explanation readable by the person concerned.
      Done in part. `rules::scoring::REGISTER` does not exist (WPM-T166 is open), so the eight scoring features S1 to S8 are listed in
      `rules::ai_register` and T166's lint test is still to do. Built: the rule (category from a closed list, an overseer role and not
      a person, informed dates not in the future, retention with a note when under six months, unknown where nothing is recorded; 6
      tests), migration `m20261024_000083_ai_register_entries`, `controllers/ai_register.rs` (`GET /api/ai-register` for any signed-in
      caller, `PUT /api/ai-register/{feature}` for HR and administrators, audited in the transaction) and the setting
      `WPM_AI_ACT_APPLIES_FROM`. Five `values.*` words for the categories in the 13 locales (AI-written, unreviewed) and the
      regenerated CMS config. Verified (Postgres 18, Podman): 706 unit tests, 142 request tests (1 new: all unknown at first, a record,
      replace and clear, each fault refused, a short retention explained, S9 not found, and **no audit, no write**), `enforcement_pay_gap`
      (extended: anonymous refused, a worker reads and is refused the write, HR records), `enforcement_pay_information`,
      `enforcement_equality`, `enforcement`, `enforcement_need_to_know`, `route_coverage`, clippy pedantic clean, 209 UI unit tests.
      Not done: **the per-decision explanation** (the person's own factors, Article 86, which was not read); a screen; the link to the
      recognised body; the other `enforcement_*` binaries were not re-run. `cargo fmt` was not clean before this change, and only the
      files touched here were formatted.
- [x] (2026-10-24) WPM-T275 **Read-only MCP server.** *(WPM-R149, WPM-D101)* Tools over existing reads under the caller's token; a test that
      every tool maps to a documented route and that a caller without access is refused; audit per call.
      Done: `rules::mcp` (the pinned revision 2025-11-25, ten tools each one `GET` route, argument checking that lets only a date, a
      UUID, a URN or a number through, the JSON-RPC plan for `initialize`, `ping`, `tools/list`, `tools/call`; 12 tests), and
      `controllers/mcp.rs` (`POST /mcp`; the call is made through a copy of the API router that carries the sign-in and need-to-know
      layers, with the caller's `Authorization`, so there is no service account; audit with the tool name first, refused if it cannot
      be written). `tower` was added as a direct dependency for `oneshot` (already in the lock file). Setting
      `WPM_MCP_ALLOWED_ORIGINS`. Verified (Postgres 18, Podman): 718 unit tests, 143 request tests (1 new: the answer equals the
      route's own, a 404 stays a tool error, 202 / 405 / 403 / 400 on the transport, one audit row per call, and **no audit, no call**
      with a trigger refusing every audit insert), `enforcement_pay_gap` (extended with real tokens: anonymous 401, a worker refused
      the report by the API, HR given it, a path-traversal argument refused), `enforcement_pay_information`, `enforcement_equality`,
      `enforcement`, `enforcement_need_to_know`, `security`, `route_coverage`, clippy pedantic clean, 209 UI unit tests. Not done:
      OAuth protected-resource metadata and audience binding; resources, prompts and pagination; a client such as Claude or an MCP
      inspector was **not** run against it (the shapes follow the revision's tools and transports pages, and only my own tests
      exercised them); the `keycloak`, `chaos` and other `enforcement_*` binaries were not re-run.
- [x] (2026-10-24) WPM-T276 **SCIM 2.0 outbound.** *(WPM-R150, WPM-D102)* Pure mapping from worker to SCIM `User` with the Enterprise
      extension; a push task over the outbox; tested against Keycloak's SCIM or a stub server; a test that no contact field
      leaves.
      Done, with one change: the task **reconciles from current state instead of reading the outbox** (the outbox exists under one
      event transport only, and a reconcile is idempotent and self-healing). `rules::scim` (the mapping, the active rule, the step,
      the back-off, the base-URL check, a source test that neither the mapping nor the push names a contact field; 11 tests),
      migration `m20261024_000084_scim_pushes`, `tasks/scim_push.rs` (`cargo loco task scim_push`; managers first; `409` resolved by
      `externalId`; failures kept as a status and retried), erasure and the subject export wired. Settings `WPM_SCIM_BASE_URL`,
      `WPM_SCIM_TOKEN`, `WPM_SCIM_ORGANIZATION`. Verified (Postgres 18, Podman) **against a stub SCIM server that I wrote**: 729
      unit tests, 144 request tests (1 new: three created with the manager first and its id carried, a joiner not sent before the
      first day, nothing sent on a second run, a title change and a leaver replaced and deactivated, a `503` recorded as a status only
      and not retried inside the back-off, retried after it, a `409` resolved by `externalId`, and the bearer token on every call; and
      a worker with a personal email, phone and postcode whose bodies hold none of them; the privacy test now checks erasure removes
      the link), `enforcement_pay_gap`, `enforcement_pay_information`, `enforcement_equality`, `enforcement`, `enforcement_need_to_know`,
      `enforcement_me`, `security`, `route_coverage`, clippy pedantic clean. Not done: any real identity provider or Keycloak (none was
      run, so interoperability with Entra ID or Okta is **unverified**); a schedule; the handover's last day as the deprovision date
      (the termination date is used); `PATCH` and groups; the token by reference; a screen or status route for the push state.
- [x] (2026-10-24) WPM-T277 **Outbound webhooks.** *(WPM-R151, WPM-D106)* Subscriptions, HMAC signature, retry with back-off, a delivery
      log with no payload body; a test that payloads hold URNs and kinds only.
      Done: `rules::webhook` (the offered events, the four payload fields, HMAC-SHA256 written out and checked against RFC 4231
      vectors, a signature with a signed timestamp and its verifier, the retry decision, endpoint and secret-reference checks; 8
      tests), migration `m20261024_000085_webhooks` (a delivery has no body column), `controllers/webhooks.rs` (administrators only;
      audited in the transaction), `streaming::emit_on` queues deliveries on the change's own transaction, `tasks/webhook_deliver.rs`
      sends them. Erasure removes the deliveries that name a worker. Verified (Postgres 18, Podman) **against a stub receiver that I
      wrote**: 737 unit tests, 145 request tests (1 new: bad endpoints, internal addresses, a settings name and unoffered kinds
      refused; the secret never returned; an unrequested event and an inactive subscription queue nothing; the signed delivery verifies
      with the secret and fails with another; the body has exactly the four fields and no name or number; the log's columns; a `503`
      retried only after its back-off; a `400` not repeated; **no audit, no subscription**; a rolled-back change queues nothing;
      rename and switch off; removal takes the log), `enforcement_pay_gap` (extended with an administrator token: anonymous 401, a worker
      and HR 403 on every route, the administrator 200), `enforcement_pay_information`, `enforcement_equality`, `enforcement`,
      `enforcement_need_to_know`, `enforcement_me`, `security`, `route_coverage`, clippy pedantic clean, 209 UI unit tests. Not done: any
      real receiver (none was run); a schedule; a "movement" event (none is emitted); protection against a host name that resolves to
      an internal address; replay of a failed delivery; a screen; the other `enforcement_*` binaries were not re-run.
- [x] (2026-10-24) WPM-T278 **Positions: pure core.** *(WPM-R152, WPM-D104)* States and moves, vacancy and over-establishment arithmetic.
      Done: `rules::position` (states, moves and the facts a move needs; validation of cost centre, budget and funding; the per-position
      gap; the establishment per cost centre with the floor; 13 tests, **mutation-checked** with five deliberate faults, each caught).
      Two decisions the spec left open are recorded in `spec/positions.md`: a position's budget is at most 100 FTE percent (one seat), and a
      held position must have its holder unlinked before it is made vacant or abolished. Two `values.*` words (frozen, abolished) in the 13
      locales, AI-written and unreviewed. Verified: 750 unit tests, clippy pedantic clean, 209 UI unit tests. Nothing is stored or
      served yet (records, API and screen are WPM-T279), so no request test applies.
- [x] (2026-10-24) WPM-T279 **Positions: records, API and screen.** *(WPM-R152)* Links to plan demand lines, requisitions and post funding;
      an establishment view per cost centre.
      Done in part: migration `m20261024_000086_positions`, `controllers/positions.rs` (list, plan, change, move, link a holder, link a
      requisition, establishment), the holder shown to privileged callers only, the establishment floored for others, erasure cutting
      the holder link and the subject export listing seats held, and the `/positions` page (establishment, seats, plan a seat, move) in
      the 13 locales (AI-written, unreviewed; German uses VZÄ so it does not repeat English). Verified (Postgres 18, Podman): 750 unit
      tests, 146 request tests (1 new: faults refused; a seat cannot be filled or opened without its facts, and the 422 names the state;
      a requisition or holder of another organization refused; a held seat not abolished or made vacant; the establishment with 50
      over and 40 vacant not netted; planned seats outside it; an unknown holder FTE unknown; an abolished seat final; audit with no
      worker; **no audit, no change**; the privacy test now checks erasure cuts the link and leaves the seat), `enforcement_pay_gap`
      (extended: a worker refused every write, HR sees the holder, a member of the organization sees `held` and no holder, and the
      floor applies to them), the other enforcement binaries listed for T276, `route_coverage`, clippy pedantic clean; UI: 209 unit tests,
      `svelte-check` clean, 4 new Playwright specs and the whole suite (210 passed, 13 skipped; one unrelated test failed once under
      load and passed alone and on the rerun), and light, dark and phone screenshots looked at. **Not done:** the screen has no control
      to link a holder or a requisition (API only); a position is not created from a plan on approval (only recorded when a person
      picks the demand line); **on a phone the two tables wrap letter by letter and the page scrolls sideways** (the shared table
      component and the long organization URN, as on other pages; not fixed here); the plan demand line is not shown on the screen; the
      `keycloak`, `chaos` and other `enforcement_*` binaries were not re-run.
- [x] (2026-10-24) WPM-T280 **Current versus target organization.** *(WPM-R153)* Pure diff over two trees; a screen beside the org chart.
      Closes the `[~]` "Workforce transformation" item in Phase 9.
      Done in part. The spec did not say where a plan's target lives, so it is a set of role nodes in `plan_org_nodes` (decision in
      `spec/positions.md`). `rules::org_shape` (layers, span of control, roles added, removed, larger, smaller, same and moved; the
      floor with the change that would give a small count back; a target check; 9 tests, **mutation-checked**: of five deliberate
      faults four were caught and one was an equivalent mutant), migration `m20261024_000087_plan_org_nodes`, the three plan routes,
      the read-only page `/org-chart/shape` with a link from the org chart, strings in the 13 locales (AI-written, unreviewed; Spanish
      uses "Funciones" so it does not repeat English). Verified (Postgres 18, Podman): 759 unit tests, 147 request tests (1 new: no target,
      faults refused, a flatter target compared with 13 live people over three layers: spans 4.00 against 7.00, ten engineers shown and
      the manager and head withheld with their direction kept, a role moved, a role added, no worker pid or name in the answer,
      replace, and an archived plan not edited), the enforcement binaries and `route_coverage` as for T279, clippy pedantic clean; UI: 209
      unit tests, `svelte-check` clean, 5 new Playwright specs and the whole suite (215 passed, 19 skipped), light screenshot looked at.
      **Not done:** the cost and skills difference (so the Phase 9 item is closed for the shape, not for those); no way to edit the
      target on the screen (API only); one target per plan; the phone screenshot was taken and not looked at; the `keycloak`, `chaos`
      and other `enforcement_*` binaries were not re-run. One earlier webhook request test turned out to depend on the host and
      database clocks agreeing; it now starts its clock a minute on.
- [~] (2026-10-24) WPM-T281 **Pay position analysis.** *(WPM-R154, WPM-D108)* Compa-ratio and range penetration per band, outliers by
      level, department and location, with the floor.
      Done in part: the API, no screen. `rules::pay_range` (pure; compa-ratio and penetration in basis points, penetration not
      clamped and unknown for a one-rate band; groups by band, level, department and location with the floor; a total held back when
      it would give a withheld group away (WPM-D108); outliers as groups against the mean of everyone; 18 unit tests), the route
      `GET /api/reports/pay-range` (payroll-level unmasked salary read, audited with the day only), no table or column. Decision and
      definitions in `spec/payroll-compensation.md`. Verified (Postgres 18, Podman): 777 unit tests, 148 request tests (new: groups,
      a group of three withheld with no count, the total unknown as `would_reveal`, then shown without it with an outlier above and
      below at a 3-point cut-off and none at the default, nothing named, bad input 422), `enforcement_pay_gap` (401, worker 403, HR
      403 masked, payroll 200), `enforcement`, `enforcement_pay_information`, `enforcement_equality`, `enforcement_need_to_know`,
      `enforcement_me`, `security` and `route_coverage`, clippy pedantic clean; UI 209 unit tests (OpenAPI copy and types
      regenerated). Mutation check: seven deliberate faults; one (the floor off by one on a group) survived, so a boundary test was
      added and then caught it. **Not done:** a screen, a CSV, the intermediate step as a reference point, part-time normalisation
      (salary read as full-time-equivalent), the Playwright suite and `svelte-check` (no UI change beyond generated types), and the
      `keycloak` and `chaos` binaries. A lib test (`every_audit_of_a_change_is_in_its_transaction`) needed the read audit listed.
- [~] (2026-10-24) WPM-T282 **Equipment issued and returned.** *(WPM-R155)* Assets and issues; open issues on the leaver checklist.
      Done in part: the API and the leaver gate, no screen. Migration `m20261024_000088_equipment` (`assets`, `asset_issues`; no
      location, cost or value column), `rules::equipment` (5 unit tests; five deliberate faults tried, all caught; a source test
      against location, cost and value words), `controllers/equipment.rs` (asset list and add, issue, return or write-off with a
      reason, a worker's own issues), `POST /api/movements/{pid}/complete` refused while the leaver has equipment out, the subject
      export lists the issues and erasure deletes them and keeps the asset, strings for the kinds and outcomes in the 13 locales
      (AI-written, unreviewed). Verified (Postgres 18, Podman): 782 unit tests, 149 request tests (new: bad assets, a duplicate tag,
      issue refused for the future and a stranger, one holder at a time, the list not naming the holder, the leaver refused with
      equipment out and no longer for it once written off, return bounds, export and erasure), `enforcement_pay_gap` (assets: 401,
      worker 403, HR 200), the other listed `enforcement_*` binaries, `security` and `route_coverage`, clippy pedantic clean; UI 209
      unit tests. **One full request run showed 1 failure that I did not identify; two reruns of the same suite passed (149 of 149).**
      **Not done:** a screen; the owner role as a permission of its own; the manager's count of open issues; equipment as items on the
      leaver checklist and in the handover trail (only the gate and a per-worker list); retiring or importing assets; Playwright and
      `svelte-check` (no UI change beyond strings and types); the `keycloak` and `chaos` binaries.
- [~] (2026-10-24) WPM-T283 **Policies and acknowledgements.** *(WPM-R156, WPM-D105)* Versioned policies by audience; acknowledgement per
      version; manager counts; HR's named, audited record.
      Done in part: the API, no screen. Migration `m20261024_000089_policies` (no column for policy text), `rules::policy` (6 unit
      tests; six deliberate faults tried, all caught), `controllers/policies.rs`, `GET /api/me/policies` and `POST
      /api/me/policy-acknowledgements` (on the self-service write list), strings for the three audiences in the 13 locales (AI-written,
      unreviewed). Found on the way: the reference policy gives a worker no write, so an acknowledgement through
      `/api/workers/{pid}/...` would have been refused for everyone but the service; it is under `/api/me` instead, which also means no
      route acknowledges for someone else. Verified (Postgres 18, Podman): 788 unit tests, 150 request tests (new: bad versions, a
      duplicate name, a version aimed at one department, the newest version replacing the old, counts of 4 and 6 of ten, nothing for a
      team of three, no pid in a count, HR's named list, the list by reference), `enforcement_pay_gap` (a worker adds no policy; HR has no
      route to acknowledge for a worker; the worker's own acknowledgement; the first day kept; a superseded version 422; the named record
      403 for a worker and 200 for HR; counts for the manager as themself), the other listed `enforcement_*` binaries, `security`,
      `route_coverage` (it caught a missing call to the list, now added), clippy pedantic clean; UI 209 unit tests. **Not done:** a
      screen; a reminder notification; role-profile audiences; a policy on the joiners' checklist; erasure and export were extended but
      are not asserted by a test; a manager other than themself without HR access (403) is not asserted; Playwright and `svelte-check`
      (no UI change beyond strings and types); the `keycloak` and `chaos` binaries.
- [~] (2026-10-25) WPM-T284 **Verifiable credentials and the skills crosswalk.** *(WPM-R157, WPM-R158, WPM-D109)* Open Badges 3.0
      issuance with a revocation list; `skill_mappings` with basis and confidence, read by gap analysis; `unknown` for an unmapped skill.
      Done in part: the API, no screen. Pinned and read on 2026-10-25: the Open Badges 3.0.3 context, VC Data Model 2.0, Bitstring
      Status List v1.0 (W3C Recommendation, 15 May 2025). `rules::credential` (Open Badges credential; compact JWS with EdDSA, whose
      key and signature are checked against the RFC 8037 example; CRC-32 and SHA-256 against their standard values; the status list
      as 131 072 bits, left-most first, in a gzip made of stored blocks, which is allowed; 12 tests) and `rules::crosswalk` (7 tests);
      mutation-checked: 13 deliberate faults, 11 caught at first and two found weak (a level boundary and the algorithm check), fixed by
      a test each, now caught. Migration `m20261024_000090_credentials_and_crosswalk`, `controllers/credentials.rs` (issue from the
      caller's own verified CPD entry, list, revoke by the worker or HR, the issuer key and the signed status list), `controllers/
      skill_mappings.rs`, the per-person skill-gap view reading the mappings (`through_mapping`, `related_not_counted`,
      `unmapped_undeclared`), subject access and erasure, the three credential settings, strings in the 13 locales (AI-written,
      unreviewed). The credential names no person and is not stored. Two routes are public (WPM-D109), the only new ones; the
      enforcement test calls them with no token. Verified (Postgres 18, Podman): 807 unit tests, 151 request tests (new: mappings with bad
      input, a duplicate pair, a broader mapping shown and not counted, exact and close grading a met need and a short one, unmapped
      counted, the list), `enforcement_pay_gap` (the issuer key and list with no token; 401; an unverified entry 422; a verified one
      issued and **verified with the published key**; no name or pid in it; one live per entry; the list bit false, then true after HR
      revokes; a worker 403 on the HR route; reissue gets a new place; a worker states no mapping), the other `enforcement_*`
      binaries, `security`, `route_coverage` (it caught an uncalled DELETE, now called), clippy pedantic clean; UI 209 unit tests.
      Added on 2026-10-25 at the user's request, during WPM-T285: a CPD entry may name its skill (`skill_pid`, migration
      `m20261025_000091_cpd_entry_skill`), and a credential from it carries an Open Badges `alignment` for each ESCO and SFIA reference the
      skill has (ESCO first, as a link too); one without a skill has none. Tested: a unit test for the shape, the request test for
      the entry, and the enforcement test (a skill with an ESCO and a SFIA reference: both in the signed credential, in order; an
      entry with no skill: none). A pool of one connection made the first version hang on a second query inside the issuing
      transaction (a 500, `ConnectionAcquire(Timeout)`); the lookups now use the transaction. A self-declared skill is not issued from.
      **Not done:** a screen; the Data Integrity proof form; key rotation; **no third-party Open Badges or VC verifier was run against a credential**; the
      workforce roll-up does not read the crosswalk; transitive mappings; erasure's revocation and the export are not asserted by a test;
      Playwright and `svelte-check`; the `keycloak` and `chaos` binaries.
- [x] (2026-10-25) WPM-T285 **Verification and documents.** *(WPM-R144–R158)* Full suites, enforcement tests for every new route, strings
      in the 13 locales, screenshots of new screens, and what was not checked.
      Verified (Postgres 18, Podman; only the container I started, `wpm-t285-pg`, was stopped): clippy pedantic clean; **807 unit
      tests**, **152 request tests**, and every ignored integration binary: `chaos`, `enforcement`, `enforcement_conversion`,
      `enforcement_cutover`, `enforcement_engagements`, `enforcement_equality`, `enforcement_escalations`, `enforcement_expenses`,
      `enforcement_flexible`, `enforcement_health`, `enforcement_me`, `enforcement_need_to_know`, `enforcement_oversight_pack`,
      `enforcement_pay_gap`, `enforcement_pay_information`, `request_log`, `retention_schedule`, `security`,
      `shared_cache_upstream` and `route_coverage`; and, with the `keycloak` feature, **`oidc_entra` and `keycloak` against a real
      Keycloak container**. UI: `svelte-check` 0 errors and 0 warnings, 209 unit tests, Playwright 215 passed and 19 skipped.
      **Found and fixed here:** (1) `retention_schedule` had been failing since WPM-T270 and WPM-T271: two soft-deleted tables
      (`pay_categories`, `pay_information_requests`) were added to the sweep and a count in that test still said 56 (now 58), which
      no earlier run had caught because that binary had not been re-run; (2) `oidc_entra` and `keycloak` **did not compile** under the
      `keycloak` feature, and did not at commit `046028d` either (checked in a throwaway worktree): reqwest's `query` and `form`
      features are not on in that build, so a dev-dependency now turns them on for tests only; both now pass; (3) the positions page
      on a phone broke words letter by letter and scrolled the page sideways, a defect recorded in WPM-T279: tables now scroll inside
      their own box and never narrow below their words, and a `select` is no wider than its container (one rule in `app.css`, so
      every table gains it; the whole Playwright suite was re-run); the phone screenshots of positions and the organization shape were
      looked at after the change.
      **Enforcement tests for every new route.** The 52 routes added in Phase 23 (from commit `046028d`) were listed from the diff and
      searched for in the enforcement binaries by path: all but one were named in one; the exception, `POST
      /api/asset-issues/{pid}/return`, now has a test (anonymous 401, worker 403, HR 200). **That is a search by path, not a proof that
      each has a refusal test of each kind**; the tasks say which refusals each asserts, and a few are not asserted (named in their
      entries: erasure and the export for equipment, policies and credentials; a non-manager reading a team count).
      **Also done on 2026-10-25 at the user's request:** a CPD entry may name its skill and a credential from it carries the ESCO and
      SFIA references (see WPM-T284).
      Documents: `plan.md` data summary (the tables as built), WPM-D108 and WPM-D109 added there, `AGENTS/spec-driven-delivery.md` next
      free ids now **WPM-R159, WPM-D110, WPM-T286**, the locale review record, `llms.txt` and `llms.json` regenerated, `check-links`,
      `check-days` and `status.py` pass.
      **What Phase 23 left unbuilt or only partly built** (each is in its task): screens for the pay categories and gap report (T270),
      pay information (T271), the joint assessment (T272), the AI register (T274), pay position analysis (T281), equipment (T282),
      policies (T283), credentials and the crosswalk (T284); the per-decision explanation of a score (T274); the cost and skills
      comparison and editing a target on screen (T280); the holder and requisition links on screen (T279); a schedule for `scim_push`
      and `webhook_deliver`; OAuth protected-resource metadata for MCP; the workforce skill-gap roll-up reading the crosswalk; manager
      counts of open equipment.
      **Not checked, in this task or any:** no real SCIM provider, webhook receiver, MCP client, or Open Badges or Verifiable
      Credentials verifier was run against the service (stubs, and the published key, only); the Directive 2023/970, the AI Act and the
      revised ESRS were read as excerpts and a national transposition not at all (WPM-T268); every new string in the 13 locales is
      AI-written and unreviewed; the page-shot harness's "dark" shot is not dark (the app's theme comes from its own picker, not the
      browser's colour scheme), so no dark screenshot of a new screen was looked at; one earlier request-test run showed a failure that
      two reruns did not reproduce (WPM-T282); no load or soak run; no browser other than Chromium.
- [x] (2026-10-25) WPM-T286 **A screen for policies: the worker's own panel on `/me`.** *(WPM-R156, WPM-D105)* `MyPolicies.svelte` lists the
      newest version in force of each policy aimed at the worker (`GET /api/me/policies`): name, version, document reference, audience,
      and where they stand (acknowledged on a day, due by a day, or overdue by whole calendar days), with a button that posts the
      worker's own acknowledgement (`POST /api/me/policy-acknowledgements`). No one can acknowledge for another, so the panel has no
      such control. Strings under `myPolicies.*` in the 13 locales (AI-written, unreviewed; French "Révision" because "Version" repeats
      English). Verified: `svelte-check` 0 errors, 209 unit tests, Playwright 216 passed / 19 skipped (new: an overdue and a waiting
      policy, the acknowledgement posts `{version_pid}` once and the row shows the day). **Not done:** the screens for the manager's
      counts and HR's policy and version editor and named record (T283 stays `[~]` for those); I did not look at a screenshot of the
      panel, on a phone or otherwise; the API is stubbed in the test, not run against the service; no reminder notification.
- [x] (2026-10-25) WPM-T287 **A screen for equipment: a panel on a worker's page and on `/me`.** *(WPM-R155)* `EquipmentPanel.svelte` lists what the
      worker holds and has held (`GET /api/workers/{pid}/equipment`). When the service lets the caller read the organization's assets
      (HR), the panel also adds an asset, gives a free one to the worker, and closes an issue as returned or, with a note, written off.
      A refusal on the asset list hides the controls and is not shown as an error. No location, cost or value is shown or sent.
      Strings under `equipment.*` in the 13 locales (AI-written, unreviewed; Welsh "Label" because "Tag" repeats English). Verified:
      `svelte-check` 0 errors, 209 unit tests, Playwright 218 passed / 19 skipped (new: give then write off with the note required
      and posted; a caller refused the asset list sees no controls). **Not done:** no screenshot looked at, on a phone or otherwise; the
      API is stubbed, not run against the service; the tests drive `/me` and the worker page was not exercised; a manager's count of
      open equipment; the leaver checklist's equipment line on the movements screen; returned-on and issued-on date pickers (today is used).
- [x] (2026-10-25) WPM-T288 **Screens for credentials and the skills crosswalk.** *(WPM-R157, WPM-R158, WPM-D109)* `MyCredentials.svelte` on `/me`: the
      worker's verified CPD entries, each with "issue a credential"; the signed credential is shown once in a read-only box with a
      download link and is not kept; the worker sees when it was issued or revoked and can revoke it. `SkillCrosswalk.svelte` on `/skills`:
      every mapping with its two skills, confidence, basis, source and whether it counts in gap analysis ("Counted" or "Related only"),
      with a form to add one and a button to withdraw one (HR; a refusal is shown in words). Strings under `credentials.*` and
      `crosswalk.*` in the 13 locales (AI-written, unreviewed). Verified: `svelte-check` 0 errors, 209 unit tests, Playwright 220 passed
      / 19 skipped (new: issue, see the credential once, revoke, issue again; a broader mapping shown as related, adding one posts the
      expected body); the existing skills tests needed a stub for the new read, and the panel's error has its own test id so the page's
      does not clash. **Not done:** no screenshot looked at, on a phone or otherwise; the API is stubbed, not run against the service,
      so the credential download and a real verifier were not exercised; HR's revoke of someone else's credential has no screen; the
      crosswalk form does not hide itself from a caller who cannot write; the skill picker on a CPD entry (the API takes `skill_pid`).
- [x] (2026-10-25) WPM-T289 **A skill picker on the CPD entry form.** *(WPM-R157)* `/cpd` loads the catalogue and offers an optional "Skill
      developed" choice; the entry is posted with `skill_pid` only when one is chosen, and an entry that names a skill shows its name in
      the list. A credential issued from such an entry carries the skill's ESCO and SFIA alignments (WPM-T285). Two strings
      (`pages.cpd.skill`, `pages.cpd.no_skill`) in the 13 locales (AI-written, unreviewed). Verified: `svelte-check` 0 errors, 209 unit
      tests, Playwright 221 passed / 19 skipped (new: the body carries `skill_pid`, the row shows the name; with none chosen the body
      has no `skill_pid`). **Not done:** no screenshot looked at; the API is stubbed, so the alignment in a real signed credential was
      not seen from the screen (it is asserted in the service tests, WPM-T285); the picker is a plain list of the whole catalogue with no
      search, so a large catalogue is long; an existing entry's skill cannot be changed or added after the fact.
- [x] (2026-10-25) WPM-T290 **Screens for policies: HR's page and a manager's team counts.** *(WPM-R156, WPM-D105)* `/policies` (in the navigation
      drawer): the organization's policies with every version (reference, effective day, audience, calendar days to acknowledge), a form
      to name a policy, a form to add the next version (department and job-title audiences ask for the value), and the named record of a
      version by worker number, which the service audits when it is opened. `TeamPolicies.svelte` on `/me`: for each policy aimed at the
      manager, "N acknowledged, M waiting" from the floor and "Fewer than 10 people: no count is shown" under it; a version aimed at
      nobody in the team is left out, and no name is listed. Strings under `policies.*`, `teamPolicies.*`, `nav.policies` and
      `titles.policies` in the 13 locales (AI-written, unreviewed). Verified: `svelte-check` 0 errors, 209 unit tests, Playwright 223
      passed / 19 skipped (new: HR adds a policy, a department version posts the expected body, the named record lists both people; a
      manager's counts shown, withheld and absent). **Not done:** no screenshot looked at, on a phone or otherwise; the API is stubbed,
      not run against the service; the manager's panel counts only versions aimed at the manager themself (it takes the version list from
      `GET /api/me/policies`), so a policy aimed at the team but not at its manager is not counted there; the page does not hide itself
      from a caller who cannot write (the service refuses and the words are shown); no way to correct a policy's name; no reminder.
- [x] (2026-10-25) WPM-T291 **A policy can be renamed, and workers are reminded.** *(WPM-R156, WPM-D105)* Both were listed as not built under WPM-T283
      and WPM-T290; the user asked for them. `PATCH /api/policies/{pid}` corrects the name (unique in the organization, its own name is
      no clash, versions and acknowledgements untouched, audited as `policy_renamed` in the same transaction); a rename control on
      `/policies`. `rules::policy::remind_now` (pure) and the loco task `policy_reminders [days_ahead:0-30] [as_of]` push a
      `policy_due` notification to a worker who has not acknowledged the newest version in force aimed at them once its day due is
      within 3 calendar days or past, once per worker and version; the notification names the policy and the day, no text. Not
      scheduled: the operator runs it daily (deployment table updated). Verified (Postgres 18, Podman): clippy pedantic clean; 808
      unit tests (the closed list of notification kinds now has `policy_due`); 153 request tests (new: a clash, an empty and a too-long
      name, an unknown policy, the policy's own name, trimming, a version keeping its title, no audit no write, a window of 3 sends
      nothing for a due day 4 calendar days away and a window of 4 sends one each, idempotent, an acknowledged worker left alone,
      nothing sent for a day when no version was in force); every `enforcement_*` binary, `chaos`, `request_log`, `retention_schedule`,
      `security` and `route_coverage` pass; `enforcement_pay_gap` also asserts 401 unauthenticated, 403 for a worker and 200 for HR on
      the rename. Two faults tried and caught: the window comparison `<=` made `<` (unit test), the `pid != this` filter removed (the
      own-name case). UI: `svelte-check` 0 errors, 209 unit tests, Playwright 223 passed / 19 skipped (rename on `/policies`); one
      earlier full run failed `pickers.spec.ts` once ("locale") and passed three reruns and a full rerun, not explained. **Not done:**
      no screenshot looked at; the notification was not looked at on screen; the `keycloak` binaries not re-run; a second reminder for a
      version still unacknowledged; a reminder to a manager about their team; the task is not scheduled; the rename was not run through
      a real client; `shared_cache_upstream` ran 0 tests here (filtered out).
- [x] (2026-10-25) WPM-T292 **A screen for pay position analysis (`/pay-range`).** *(WPM-R154, WPM-D108)* A page in the navigation drawer that reads
      `GET /api/reports/pay-range` for an organization, an optional "as of" day and an outlier cut-off in points of compa-ratio. It
      shows the basis and derivation the service states, the currency, who was left out (an exact count only from the floor, else
      "fewer than 10"), everyone analysed, and the four groupings (band, job level, department, location): workers, mean compa-ratio,
      range penetration, and the outlier flag as a group. A group under the floor says "Fewer than 10 workers: nothing is shown" and
      no count; a figure that cannot be worked out says why (no pay or band, would reveal a withheld group, a one-rate band). Basis
      points are placed by `format.basisPoints` (digits only; the service did the arithmetic). A caller without a payroll-level read
      sees the app's refusal and no figure. Strings under `payRange.*`, `nav.payRange`, `titles.payRange` in the 13 locales
      (AI-written, unreviewed; four locales needed their own wording for "Compa-ratio"). Verified: `svelte-check` 0 errors, 210 unit
      tests (new: `basisPoints`), Playwright 225 passed / 19 skipped (new: figures, a withheld group, an outlier below, the unknown
      reason, the cut-off sent to the service; a refusal). I looked at a desktop and a 390-pixel screenshot of the page with stub
      data: the table scrolls inside its box on the phone and nothing overflows. **Not done:** the API is stubbed, so the screen was not
      seen with a real report; a grouping with no groups shows only its header row; no dark screenshot; the currency cannot be chosen
      on screen (the service defaults to the most common); the read is audited by the service but the screen does not say so.
- [x] (2026-10-25) WPM-T293 **Pay position screen: the five limits recorded under WPM-T292, and a defect the real run found.** *(WPM-R154, WPM-D108)*
      Asked for by the user. (1) **Seen with a real report:** a real service (`WPM_REQUIRE_AUTH=0`, Postgres 18 in Podman) with 54
      seeded workers (four groups on band 6, four on band 4; salary set in the database, since no API sets one) and the built UI
      through the proxy; a new live spec `tests/live/pay-range.spec.ts` (skipped without `WPM_LIVE_API=1`) asserts a small band is
      withheld, a total that would reveal it is withheld, no worker is named, and a currency nobody is paid in leaves the page saying
      so. With enforcement off the service reports no organization scope, so the spec stubs that one call; the report is real.
      (2) **An empty grouping** says "No one is placed in this grouping." instead of a lone header. (3) **Dark screenshot:** looked
      at, in the real dark theme chosen through the theme store (earlier "dark" shots were not dark); the page reads cleanly, as does
      the 390-pixel one. (4) **Currency** can be chosen: a three-letter code, sent upper-cased only when whole. (5) **The page says the
      read is audited** (the day it covers and who opened it, no figure). **Defect found by the real run and fixed:** the service said
      "fewer than 10" left out when nobody was left out, because `rules::pay_range::left(0, floor)` returned `UnderFloor`; my own unit
      test had enshrined it. Zero now says `exactly 0` (it names no one); one to nine is still not counted. A new unit test; spec
      line in `payroll-compensation.md` corrected. Verified: clippy pedantic clean; 809 unit tests; 153 request tests;
      `enforcement_pay_gap`, `enforcement_need_to_know`, `security`; UI `svelte-check` 0 errors, 210 unit tests, Playwright 226
      passed / 19 skipped (new: empty groupings, the audit line, the currency sent only when whole); the live spec 2 passed.
      **Not done:** the rest of the live suite and the other `enforcement_*`, `chaos`, `retention_schedule` binaries were not re-run
      for this change (only the listed ones); the outlier flag was seen only as "not worked out" in the real run, because the
      seeded data withholds the total, so a real "above" or "below" has not been seen on screen (the stubbed test shows it); the
      locale text of the new strings is AI-written; the app's sign-in and scope were not exercised (the live run stubs the scope).
- [x] (2026-10-25) WPM-T294 **A screen for pay categories and the gap report (`/pay-gap`), seen against a real service; a misleading reason fixed.**
      *(WPM-R144)* A page in the navigation drawer: the categories of workers (name, stated basis, job titles one per line; add, edit,
      remove, with the service's words on a refusal) and the seven indicators for an organization, an optional day, an optional
      currency and the choice of how variable pay is taken. Each figure shows women's and men's pay and the gap (positive means men
      are paid more), or says withheld ("fewer than 10 of one sex") or why it is unknown; quartiles, the gap by category, who was left
      out (exact only from the floor or when none), the reporting tier and next due day, the audit line, and a link to the CSV. Pay is
      formatted with `Intl` in the page's locale and the gap by `basisPoints`. Strings under `payGap.*`, `nav.payGap`,
      `titles.payGap` in the 13 locales (AI-written, unreviewed). **Seen against a real service** (`WPM_REQUIRE_AUTH=0`, equality
      monitoring on with a recorded basis, Postgres 18 in Podman, 48 seeded workers with sex answers and two categories; salary and
      sex set in the database): a live spec `tests/live/pay-gap.spec.ts` (skipped without `WPM_LIVE_API=1`) asserts the category with
      both sexes over the floor shows a figure and the one with three men is withheld, no worker is named on the page or in the
      CSV, and the CSV link answers. I looked at light, dark and 390-pixel screenshots. **The real run found, and I fixed:** the
      overall indicators said "Nobody in the group" when the truth is that WPM holds no variable pay; `rules::pay_gap` now gives the
      reason `variable_not_recorded` when workers can be compared but nobody's variable pay is recorded (a unit test; two faults
      tried and caught: the override removed, and its non-empty guard removed; one older test's expectation updated). It also showed
      the quartile label repeating "(1 lowest)" on every row and an "Edit" button labelled "Rename"; both fixed. I wrongly wrote in
      the live README that the service empties its database on start; it does not, the Rust request tests do, and the README is
      corrected. Verified: clippy pedantic clean; 810 unit tests; 153 request tests; `enforcement`, `enforcement_equality`,
      `enforcement_pay_gap`, `enforcement_pay_information`, `enforcement_need_to_know`, `enforcement_me`, `security`,
      `retention_schedule`, `route_coverage`; UI `svelte-check` 0 errors, 210 unit tests, Playwright 229 passed / 19 skipped (new: the
      indicators with their reasons, withheld cells, the CSV link; creating a category from titles one per line and removing it; a
      refusal), the live spec 2 passed. A first full Playwright run failed many tests by timeout while another process on the machine
      saturated it; with two workers and a longer timeout they all passed. One request test
      (`pay_assessment` audit-refusal) failed in one full run, with a panic inside the loco test harness, and passed alone and in
      the next full run: not explained. **Not done:** `chaos`, `request_log`, `shared_cache_upstream` and the other `enforcement_*`
      binaries were not re-run for this change; no real variable-pay figure has been seen (WPM holds none); a real figure with a
      negative gap was not seen; the category editor was not seen against the real service (the live run only lists); a median per
      category is not computed (a service gap); sign-in and scope were not exercised (the live run stubs the scope).
- [x] (2026-10-25) WPM-T295 **Screens for pay information: the worker's request and HR's queue and answer.** *(WPM-R145)* `MyPayInformation.svelte` on
      `/me`: the worker's own requests with the day asked and the day due (or overdue, or answered) and a button to ask, disabled while one
      waits (the service also refuses a second); no figure is shown there. `/pay-information` (in the navigation drawer): the queue
      with who asked, a status filter, "Work out the answer" (payroll-level read, audited by the service) showing the worker's own
      pay, their category and its stated basis, and each sex's average, a withheld one saying so with the floor; shown once and not
      stored; "Record as answered" keeps only the day. Strings under `payInfo.*`, `nav.payInformation`, `titles.payInformation` in
      the 13 locales (AI-written, unreviewed). **Seen against a real service** (`WPM_REQUIRE_AUTH=0`, equality monitoring on, Postgres
      18 in Podman, 48 workers, two categories; two requests inserted in the database because asking needs a token): a live spec
      `tests/live/pay-information.spec.ts` asserts the overdue request leads the queue, the answer for a woman in a category with
      both sexes over the floor shows pay and both averages, the answer for a man in a category with three men withholds his sex's
      average, only the two askers are named, and recording it as answered shows "Answered". I looked at light, dark and the answer
      screenshots. **A defect the real run showed, and its fix:** the derivation text had a stray backslash ("in the same \
      category"): a string continuation written as two backslashes is a literal backslash and a line break. 28 lines in six
      controllers had it (`pay_information`, `positions`, `skill_gaps`, `pay_assessment`, `planning/org_shape`, `training_plan`),
      some from earlier phases; all now use a single backslash, and a unit test (`source_text_guards`) fails if any source line ends
      in a doubled backslash (checked by putting one back). Verified: clippy pedantic clean; 811 unit tests; 153 request tests;
      every `enforcement_*` binary, `chaos`, `request_log`, `retention_schedule`, `security`, `route_coverage`; UI `svelte-check` 0
      errors, 210 unit tests, Playwright 232 passed / 19 skipped (new: ask once and no figure; HR's answer with a withheld average and
      recording; a refusal in words); the live spec 2 passed; a real derivation read with `curl` after the fix. **Not done:** the
      worker's panel was not run against the real service (asking needs a token, and enforcement is off there); no phone screenshot of
      the queue was looked at; no real answer for a worker with no category or no pay was seen (the stubbed UI shows the wording only);
      the yearly notice and a worker's view of an answer are not built (HR gives it to them); `keycloak` and `shared_cache_upstream`
      not run; the new strings are AI-written and unreviewed.
- [x] (2026-10-25) WPM-T296 **A screen for the joint pay assessment: a section of `/pay-gap`.** *(WPM-R146)* `JointAssessment.svelte`, under the indicators and
      using the same organization, day, currency and variable-pay choice: the threshold and the calendar months to remedy; for each
      category its gap in basic pay, the report date, and the signal in words (a joint assessment is due, with the day the period
      ended; pending until a day, or until the gap is reported; not due because the gap is under the threshold, justified, remedied
      or already assessed; cannot be said because the gap is withheld or cannot be worked out); what has been recorded; and a form
      to record, for a category and a report date, a justification (10 to 1,000 characters, stopped by the form when too short), the
      remedy day, the assessment day with its outcome (the closed list) and the representatives by name. It reports; a person
      records. Strings under `jointAssessment.*` in the 13 locales (AI-written, unreviewed; German "Stand" and French "Indication"
      because "Signal" repeats English). **Seen against a real service** (the T294 set-up: Care at a 5.00% gap, Legal withheld): a
      live spec `tests/live/joint-assessment.spec.ts` asserts the real signal is pending until reported, due once a report date
      eight months ago is recorded with nothing justified, and not due once a justification is recorded; the withheld category cannot
      be said. I looked at the screenshot with the due signal. Verified: UI `svelte-check` 0 errors, 210 unit tests, Playwright 233
      passed / 19 skipped (new: due, withheld, recording a justification and an assessment with the expected body and no remedy
      day); the live spec 1 passed (a first attempt failed because its own earlier run had already recorded data, which is why the
      README says to clear the records). **Not done:** the Rust side is unchanged, so its suites were not re-run; the screen was not
      seen on a phone or in the dark; the service's refusals (a justification shorter than ten characters, a remedy dated before the
      report) were not seen on screen: the form stops the first itself and the second was not tried; a remedy and a "remedied"
      signal were not seen against the real service; the link to a recognised body is only a name typed in (Phase 18's bodies are not
      built); the new strings are AI-written and unreviewed.
- [x] (2026-10-25) WPM-T297 **A screen for the AI and scoring register (`/ai-register`).** *(WPM-R148, WPM-D103)* A page in the navigation drawer, readable
      by anyone signed in (it is a notice of how people are scored): the date the Annex III duties apply and whether they now do;
      each of the eight scoring features (S1 to S8) with what it is, who works it out, what it informs and what it is made of (the
      service's own English wording, marked as such), and what the deployer recorded: the category chosen, the overseeing role, when
      representatives and workers were informed, how long logs are kept and why when under six months. A field with nothing recorded
      says "Not recorded", never "none"; a retention under the six months Article 26(6) names is flagged. HR and administrators record a
      feature through a form that replaces what is there (an empty field is cleared); a refusal shows the service's words. The
      service's note that the register is not a classification, a certification or evidence of compliance is shown. Strings under
      `aiRegister.*`, `nav.aiRegister`, `titles.aiRegister` in the 13 locales (AI-written, unreviewed). **Seen against a real
      service** (`WPM_REQUIRE_AUTH=0`, Postgres 18 in Podman): a live spec `tests/live/ai-register.spec.ts` asserts eight features
      with six unknown fields each, that three months of retention with no reason is refused by the service in words and nothing is
      recorded, and that with a reason it is recorded and flagged. I looked at light, dark and 390-pixel screenshots; the first
      layout stacked nine label-value pairs per card into a very long page, so they now sit side by side on a wide screen (the page is
      about a quarter shorter) and stack on a phone. Verified: UI `svelte-check` 0 errors, 210 unit tests, Playwright 235 passed / 19
      skipped (new: what is recorded and unknown, the short-retention flag, recording with the expected body; a refusal in words); the
      live spec 2 passed. **Not done:** the Rust side is unchanged, so its suites were not re-run; the per-decision explanation
      (Article 86, not read) and the link to a recognised body are not built; the page is still long (eight cards) and was not given
      a way to collapse them; only the new layout's dark and light shots were looked at after the change on the wide screen, not the
      phone; the feature text is English only (it comes from the service); the new strings are AI-written and unreviewed.
- [x] (2026-10-25) WPM-T298 **The workforce skill-gap roll-up reads the skills crosswalk.** *(WPM-R158, WPM-D109)* Listed as not built under WPM-T284: the
      roll-up is counted in SQL (`controllers/gap_sql.rs`) and did not read mappings, so a person whose declared skill was mapped to a
      need was graded as having declared nothing there, while their own view graded it. The SQL now grades a need the person did not
      declare through the best `exact` or `close` mapping from a skill they did declare (a lateral join: the most confident, then the
      highest declared level, as `rules::crosswalk::apply` does); a direct declaration is never overridden; `broader` and `narrower`
      are never counted. This also reaches the other readers of the graded set (the staff below a need, for training hours). The
      roll-up's derivation text says so. A new request test grades the same two people both ways: before any mapping both needs are
      unknown; a broader mapping changes nothing; a close mapping puts one worker above a need and a worker whose exact mapping from a
      lower declared level outranks a close one from a higher level is below it; the person's own view agrees (including
      `through_mapping`); and a direct declaration is not overridden. Two faults tried and caught: the exact-first ordering removed,
      and `broader` admitted. Verified (Postgres 18, Podman): clippy pedantic clean; 811 unit tests; 154 request tests; every
      `enforcement_*` binary, `chaos`, `request_log`, `retention_schedule`, `security`, `route_coverage`. **Not done:** the roll-up does
      not say how many of its figures went through a mapping (the per-person view does); no timing was taken, so the cost of the
      lateral join on a large catalogue of mappings and workers was unmeasured (WPM-T299 measured it and replaced the join; the
      index on `skill_mappings.to_skill_pid` I said was missing is there, `skill_mappings_by_target`); the UI was not touched, so no screen was checked; the `keycloak`
      and `shared_cache_upstream` binaries were not run.
- [x] (2026-10-25) WPM-T299 **The roll-up's crosswalk lookup measured, and made a quarter of the cost.** *(WPM-R158, WPM-R142)* WPM-T298 left the cost unmeasured. On a
      throwaway Postgres 18 (Podman) with 20 000 workers, 500 skills, 40 roles of 12 needs, 500 000 declarations and 3 000 mappings
      (240 000 needs after merging), the grading query alone took **195 ms** before the crosswalk, **1 854 ms** with the per-need
      lateral lookup of T298, and the endpoint answered in **about 3.1 s**. The lookup is now driven from the declarations and joined
      to the merged needs once (`controllers/gap_sql.rs`): **803 ms** for the query and **about 1.1 s** for the endpoint, with the
      same answer (every group's count of met, below and undeclared, 240 000 people over 1 536 groups, 273 skills ranked identically,
      compared against the lateral form), and the "workers with needs" count no longer evaluates the mapping work at all. Two
      intermediate forms were tried and timed (a join per need, 2.0 s; the declaration-driven join with an extra direct-level
      join, 1.4 s) and gave the same results. The T298 request test still grades the same people both ways and its two faults are
      still caught by it. Verified (Postgres 18, Podman): clippy pedantic clean; 811 unit tests; 154 request tests; every
      `enforcement_*` binary, `chaos`, `request_log`, `retention_schedule`, `security`, `route_coverage`. **Not done:** this is one
      dataset on one laptop under heavy load from other processes (the machine's load average was above 70 during the later runs),
      so the numbers say "about a quarter of the cost", not a guarantee; it is still about four times the 195 ms of the query
      without the crosswalk, and no larger dataset (or a department-filtered call) was timed apart from one early department read
      (0.2 s on the lateral form); the roll-up runs the grading twice (rows, and "workers with needs") and the plan still sorts 240 000
      rows, so a materialised or cached roll-up would be the next step if it is too slow; no load test of the endpoint was run; the
      first full run hung for an hour in `chaos` waiting on a package-cache lock held by another project's cargo, and passed when
      run alone.
- [x] (2026-10-25) WPM-T300 **A schedule for the recurring tasks.** *(WPM-R151, WPM-D106)* `scim_push` and `webhook_deliver` were listed as having no schedule, and
      the deployment page said to "schedule each with cron" with no file to start from. `config/scheduler.yaml` is now a job list for
      Loco's own scheduler: `webhook_deliver` every minute, `snapshot_headcount`, `early_warning_sweep`, the four reminder tasks and
      `verify_audit_chain` daily, `oversight_pack_freeze` on the first three days of the month (it is idempotent, so the first run
      writes and the others skip), and `scim_push` every 15 minutes under its own tag `scim`, because it fails every run unless
      `WPM_SCIM_BASE_URL` and `WPM_SCIM_TOKEN` are set. Start one with `cargo loco scheduler --config config/scheduler.yaml --tag
      wpm` (or `--tag scim`; `--list` shows the jobs). A new test `tests/scheduler_config.rs` loads the file through Loco's own loader
      (so every job names a real task), fails if a registered task is neither scheduled nor named as run by hand with a reason
      (`seed`, `import_esco`, `import_framework`, `replay_erasures`), and checks every cron has six or seven fields, every job has a tag,
      `scim_push` is not in the default tag, and the daily jobs leave day, month and weekday open; three faults tried and caught (a
      job removed, a task name misspelt, `scim_push` given the default tag). INSTALL, the deployment table and the integrations spec are
      updated; the old INSTALL text named only three tasks. **Seen working:** against a throwaway Postgres 18 migrated with `db
      migrate`, `--list` printed the real file's ten jobs and a scheduler on a copy with three-to-five-second schedules ran
      `webhook_deliver`, `policy_reminders` and `verify_audit_chain` as child processes, each exiting 0 (`{"delivered":0,...}`,
      `{"ok":true,"entries":0,...}`). Verified: `scheduler_config` 2 passed; the rest of the suites were not touched (the change is a
      config file, a test and documents). **Not done:** the real file's own times were not waited for (the check used a copy with
      short ones); a delivery to a real receiver and a SCIM push to a real provider were not seen through the scheduler (their task
      tests and stubs stand as before); the scheduler is not started by `compose.yaml` or any deployment manifest here (there is none
      for the API), so a deployer must start it; two schedulers at once were not tried (the tasks are idempotent, by their tests);
      job output goes to the scheduler's output, so `webhook_deliver` writes a line a minute; time zone is the host's.
- [x] (2026-10-25) WPM-T301 **The outbound webhook flow run against an independent receiver.** *(WPM-R151, WPM-D106)* "No real webhook receiver was ever run" had stood in every
      report. `scripts/check-webhook-flow.py` is a receiver written from the spec alone (it shares no code with the service: it recomputes
      `X-WPM-Signature: t=<unix>,v1=<HMAC-SHA256 of "<t>.<body>">` from the shared secret and checks the timestamp) and a driver for a
      running demo service. Run against the real service (`WPM_REQUIRE_AUTH=0`, Postgres 18 in Podman, `WPM_WEBHOOK_SECRET_E2E` set on
      both sides) and, for the first delivery, through the real Loco scheduler running `webhook_deliver` as a child process: a worker
      moved to active queued one delivery with **no body** (the log row has no column for one); the receiver verified the signature, the
      timestamp was fresh, the body was exactly `{id, kind, subject, occurred_at}` with a `worker:<pid>` URN and no name or number, and
      the log said delivered, status 200, attempt 1. A receiver answering **503**: the delivery stayed pending with status 503,
      attempt 1, and the next try one minute later; an immediate second run sent nothing; after the minute, with the receiver answering
      200, the same delivery went through on **attempt 2**. A receiver answering **400**: the delivery was marked failed, status 400,
      and was not sent again. The script passes (`--retry` includes the wait) and is repeatable. Found on the way, in my own use, not in
      the service: a subscription's `secret_ref` must be the whole variable name (`WPM_WEBHOOK_SECRET_<NAME>`), which the service says
      in words; the task's one-line summary counts every delivery due, including other subscriptions', so the script checks the log of
      its own subscription and not that summary. Verified: the script's checks pass (200, 503 then retry, 400); no Rust or UI file was
      changed, so the suites were not re-run. **Not done:** one receiver, one machine, plain `http` to `127.0.0.1` (the allowed local
      form); `https` and a real third-party receiver were not used; the scheduler's own one-minute schedule was not waited for (a
      copy with a three-second schedule delivered the first); a delivery of `worker_terminated`, `worker_retired` or `leave_approved`
      was not seen (only `worker_activated`); the script was not run against a database holding other subscriptions or real data; the
      receiver was not tried with a wrong secret (the unit tests of `rules::webhook` cover a tampered body and a wrong secret); a
      SCIM provider was not run (WPM-T276 used a stub; Keycloak has no SCIM endpoint), so that item stands.
- [x] (2026-10-25) WPM-T302 **The SCIM push run against an independent SCIM server; a back-off one step too long fixed.** *(WPM-R150, WPM-D102)* The reports said, run after run, that no real SCIM
      provider was ever run (WPM-T276 used a stub of the service's own making). `scripts/check-scim-flow.py` is a SCIM 2.0 server written from
      RFC 7643 and RFC 7644 (bearer authentication; `POST /Users` giving 201 with an `id` and `meta`, and 409 `uniqueness` on a duplicate
      `userName`; `GET /Users?filter=externalId eq "…"` giving a ListResponse; `PUT /Users/{id}` replacing, 404 for an unknown id) and a
      driver for a running demo service; the server also records violations (an attribute outside the core and enterprise schemas that WPM is
      meant to send, any contact or cost detail, an enterprise attribute without its schema URN, a `manager.value` it never issued, a wrong
      content type). Run against the real service (`WPM_REQUIRE_AUTH=0`, Postgres 18 in Podman) through the real task `scim_push`, scoped to a
      fresh organization by `WPM_SCIM_ORGANIZATION`: a manager and two reports were created **manager first**, a report's `manager.value` was the
      id the provider gave the manager, `userName` and `employeeNumber` were the employee number, `externalId` a `worker:<pid>` URN, no
      violation; a second push sent nothing (three unchanged); a job-title change gave one `PUT` to that user's own id and no extra user; a user the
      provider already held was found after `POST` answered 409 (`POST`, `GET` by `externalId`, `PUT`) with no duplicate; a 503 was recorded
      (non-zero exit, the next run waits and sends nothing); a rejected bearer token failed the push, created nothing and appeared in nothing the task
      printed; and after the back-off the push went through (`--retry` waits it out). **A defect the real run showed, and its fix:** the spec says a
      failure is retried "one minute doubling", but the task waited **two** minutes after the first failure (it passed the post-increment attempt
      count to `backoff_seconds`, whose index 0 is one minute; the webhook task already subtracted one). The task now subtracts one; the request
      test asserts the first retry is not due at 59 seconds and due at 61, and fails if the old line is put back (checked). The script's own
      checks were proved able to fail by making the server refuse `title`: both "no violation" checks then fail. Verified (Postgres 18, Podman):
      clippy pedantic clean; 811 unit tests; 154 request tests; every `enforcement_*` binary, `chaos`, `request_log`, `retention_schedule`
      (re-run alone after a package-cache lock held by another project's cargo hid its line), `security`, `route_coverage`, `scheduler_config`;
      the SCIM script passes, with the retry. **Not done:** one server I wrote, on `http://127.0.0.1` (the allowed local form), no third-party
      identity provider (Keycloak has no SCIM endpoint; a real Entra or Okta tenant was not available); the server implements only what the task
      calls (no `PATCH`, no `DELETE`, no `/ServiceProviderConfig`, no pagination beyond one page, no ETag or `If-Match`); a leaver's `active:
      false` and a deleted record's deactivation were not run through it (the request test covers them); the manager-before-report ordering
      was seen for one level only; the script was not run against a database holding other organizations (it scopes itself by organization);
      the scheduler's own 15-minute schedule was not waited for.
- [x] (2026-10-25) WPM-T303 **The credential flow run against an independent verifier.** *(WPM-R157, WPM-D109)* "No Open Badges or Verifiable Credentials verifier was ever run" stood in every
      report. `scripts/check-credential-flow.mjs` is a verifier written from RFC 7515 and RFC 8037 (the JWS), the W3C VC Data Model 2.0, Open Badges 3.0
      and the Bitstring Status List v1.0, using only Node's crypto, zlib and fetch; it shares no code with the service. It mints its own test tokens (PASETO
      v4.public with a throwaway key; `--print-env` prints the settings to start a service with) so a worker acts as themself through `/api/me`, which needs sign-in on.
      Run against the real service (sign-in on, the reference ABAC policy, Postgres 18 in Podman, a throwaway signing key): an entry nobody verified is
      refused (422) and with no token 401; the issuer's key is public and is the public half of the key the service was started with; the
      credential is a compact JWS, `alg` EdDSA, `typ` `vc+jwt`, with the issuer's `kid`, **and verifies against the published key**; the contexts are VC 2.0
      then Open Badges 3.0.3; the types are `VerifiableCredential` and `OpenBadgeCredential`; an `AchievementSubject` with no subject id; the alignments carry the
      skill's ESCO (with `targetUrl`) and SFIA references; **no name, employee number, token subject or email** is in it; the status entry is a
      `BitstringStatusListEntry` for revocation; the public list is itself a signed `BitstringStatusListCredential`, purpose revocation, multibase `u`, 131 072
      bits, this credential's bit **0**; another person cannot revoke it (404); after the worker revokes it the bit is **1** on a list that still verifies, and the
      credential's own signature is unchanged; it can be issued again as a new one; a credential with its payload changed, a credential under another key
      and a header saying `alg: none` are all refused. The checks were proved able to fail with two mutants (signatures not checked: both tamper checks fail; a
      wrong expected issuer key: the key check fails). **Found on the way, in the script and not the service:** my first "nothing about the person" check also
      forbade the activity title, but the service names the achievement by it, as the spec says (a badge needs a name); the check now asserts that explicitly.
      **A property worth knowing, not a defect:** the title is free text the worker typed, so personal detail in it would travel in the credential they share.
      Verified: the script passes (and is repeatable: its skill references are unique per run); no Rust or UI file was changed, so the suites were not re-run.
      **Not done:** one verifier I wrote, over `http://localhost`; no third-party Open Badges or wallet verifier (for example the 1EdTech validator) was run, so
      interoperability beyond the standards as I read them is unproven; the Data Integrity proof form and key rotation are not built; status list
      caching and its time-to-live were not checked; a credential for a skill assessment is not built; the worker's `/me` credentials panel was not run
      against this sign-in-on service (the UI needs a session, not a token).
- [x] (2026-10-25) WPM-T304 **OAuth protected-resource metadata for the MCP endpoint, and the MCP server run against an independent client.** *(WPM-R149, WPM-D101)* "OAuth
      protected-resource metadata" stood as not built, and no real MCP client had been run. **Built:** `GET /.well-known/oauth-protected-resource` and
      `.../mcp` (RFC 9728), public (in `is_public_path`), from the new settings `WPM_MCP_RESOURCE` (the endpoint's URL as clients reach it) and
      `WPM_MCP_AUTHORIZATION_SERVERS` (comma-separated `https` issuers): 404 until both are set, 500 if they are unusable, never a made-up document. The pure rules
      are in `rules::mcp` (`metadata_path`, `metadata_url`, `protected_resource`, `challenge`; 3 new unit tests: the path-aware well-known path, the document and
      its exact keys, the challenge). Every `401` now carries `WWW-Authenticate: Bearer` (RFC 6750; none did before) and on `/mcp`
      `Bearer resource_metadata="<the metadata URL>"`, as the MCP authorization section asks (`challenge_mw`, outside the sign-in layer). Token audience
      binding is not built: WPM is not an authorization server and does not check that a token was issued for this resource. **Tests:**
      `enforcement_pay_gap` asserts the document is public once configured and 404 before, both well-known paths, the 401 challenge with and without
      the pointer, an API route's 401 without the pointer, and a 500 for an `http` authorization server; two faults tried and caught (the `/mcp`
      path made non-public; the challenge layer removed). **The independent client** `scripts/check-mcp-flow.mjs` is written from the MCP specification
      revision 2025-11-25, JSON-RPC 2.0 and RFC 9728 with Node's fetch, sharing no code with the service, with test tokens minted from a throwaway key
      (`--print-env` prints the settings; sign-in must be on). Against the real service (sign-in on, the reference policy, Postgres 18 in Podman) all its checks
      pass: the 401 challenge and the metadata it points at (public, `resource` equal to the endpoint URL, https authorization servers, header bearer
      method, nothing about anyone); `initialize` echoing the request id and the pinned revision, capability `tools` only, `serverInfo`; the initialized
      notification answered 202 with no body; ping; `tools/list` (ten tools, each with a name, description and object input schema, unique names, none
      read like a write, none declared not read-only); a call the caller may make returning text content; a call the API refuses the caller returning a
      **tool error** (`isError`, saying 403) and not a protocol error; an unknown tool, -32602; a route-steering argument refused; the same tool for another caller
      answering for that caller; an unknown method, -32601; a body that is not JSON, 400 with -32700; a protocol version the server does not speak, 400; an
      `Origin` not listed, 403; `GET` 405; the token echoed nowhere. A mutant client expecting another revision fails four checks. **Found, not fixed:** a worker's
      `DELETE /mcp` is refused 403 by the access policy before the route can answer 405 (a service caller gets 405); the spec allows a refusal and the script
      accepts either. Verified (Postgres 18, Podman): clippy pedantic clean; 814 unit tests; 154 request tests; every `enforcement_*` binary, `chaos`,
      `request_log`, `retention_schedule`, `security`, `route_coverage`, `scheduler_config`. **Not done:** one client I wrote, over `http://localhost`, with no real
      authorization server behind the metadata (its `https://login.example.org/...` is a placeholder), so no real OAuth discovery, dynamic client registration or
      token issuance was exercised; no third-party MCP client (an IDE or an agent SDK) was run, so interoperability beyond my reading of the specification is
      unproven; the audience of a token is not checked; resources, prompts, sampling, `outputSchema` and pagination are not built; `WWW-Authenticate` has no
      `error` or `scope` parameters; the DELETE difference above.
- [x] (2026-10-25) WPM-T305 **Dependencies, the Lily design system, the Dependabot changes and the advisory policy updated.** *(WPM-R119)* **UI:** `@lilydesignsystem/svelte-headless`
      0.5.1 to 0.6.1, `svelte-locale-translations` 0.1.0 to 0.3.0, `themes` 0.5.0 to 0.6.0 and `vite` 8.3.4 to 8.3.5 (exact pins, as before; the
      other Lily packages were already latest; the theme stylesheets regenerate, 57). `svelte-check` 0 errors, 210 unit tests and Playwright 235 passed / 19 skipped,
      unchanged. **TypeScript 7.0.2 was tried and is not adopted:** `svelte-check` 4.7.7 and SvelteKit 3.1.0 fail to load it; TypeScript stays on 6.0.3 and
      the package files were restored. `pnpm audit`: no known vulnerabilities. **Rust:** `cargo update` took 26 compatible updates (`tokio` 1.53.3 to
      1.54.1, `redis` 1.7.1 to 1.8.0, `uuid` 1.28 to 1.29, `brotli`, `cc`, the ICU crates and others); a lookup of every direct dependency of the service and the matcher
      against crates.io found none with a newer breaking line, and the matcher's lockfile had nothing to update. **The three open Dependabot pull requests
      (#14, #15, #16) were applied by hand** (their checks were green): `pnpm/action-setup` 6.1.0 in CI, and Redis 7 to 8 in `compose.yaml` and
      `compose.test.yaml`; the five Redis 7 service images in `ci.yml`, which Dependabot does not read, were moved to the same Redis 8 digest, and INSTALL and
      `spec/testing.md` now say Redis 8. **Advisory policy:** `cargo deny check` warned that eleven of the twelve ignored advisories were "not encountered"; those
      eleven are removed from `deny.toml`, so each fails the build again if it returns, and the one that still matches (RUSTSEC-2024-0449, `unsafe-libyaml`,
      unmaintained, via Loco) stays with its note. `cargo deny check` passes (advisories, bans, licences, sources) and `cargo audit` reports no vulnerability for the service or the
      matcher (one allowed warning: the same crate). `scripts/check-actions-pinned.py`: all pinned. Verified (Postgres 18 and **Redis 8.10.2** in Podman, `WPM_TEST_REDIS_URL` set):
      clippy pedantic clean on the new lockfile; 814 unit tests; 154 request tests, including `redis_is_shared_between_instances_and_fails_open` against Redis 8 (it skips
      without the variable; checked both ways); every `enforcement_*` binary, `chaos`, `request_log`, `retention_schedule`, `security`, `route_coverage`, `scheduler_config`.
      **Not done:** the GitHub pull requests are not closed and nothing is pushed: the changes are committed locally only (this repository's `origin` pushes to Codeberg and fetches
      from GitHub), so closing #14 to #16 now would point at nothing; the GitHub notifications listed were for other repositories and were left alone; the Redis password and TLS
      variants of the shared-cache tests did not run (their variables are not set); `shared_cache_upstream` ran no test here (it is filtered out); the Keycloak and Entra binaries
      were not re-run; the CI workflow was not run, so the new action and Redis 8 service images are untested in CI; there is no `CLAUDE.md`, and no `*-skill` or `*.github.io`
      folder in this repository or beside it, so nothing was updated there; every agent file is already under 40 000 bytes (the largest is 7 382).
- [x] (2026-10-25) WPM-T306 **CI red after the first push of the session's work: four checks, fixed.** *(WPM-R119, WPM-R123)* After the user asked for `main` to be pushed (71 commits), CI on `3c5755a`
      failed in four jobs and I had closed Dependabot pull requests #14 to #16 naming that commit before its CI had finished. The failures were ours and
      predate the dependency commit: (1) **`cargo fmt --check`**: a great many files across the crate had never been formatted (my earlier rule of not
      running `cargo fmt` on the whole crate left the repository red against its own CI); `cargo fmt` now runs clean over 47 files, which exposed
      three missing semicolons in `rules/position.rs` and a 126-line `issue` in `controllers/credentials.rs` (one `#[allow(clippy::too_many_lines)]`
      with a reason), and one import left unused; clippy pedantic is clean again; (2) **`pnpm lint`**: `src/lib/api/wpm.ts` needed two Prettier
      passes; (3) **gitleaks over the whole history**: two findings, both reviewed fixtures and now in `.gitleaksignore` with a reason (the throwaway Ed25519
      seed in `enforcement_pay_gap.rs`, and a made-up idempotency key in a UI unit test); `gitleaks` 8.30.1 (checksum verified) finds nothing now;
      (4) **`scripts/verify-db-roles.sh`**: seven request tests install a trigger that refuses audit rows (WPM-D107), which is DDL, and failed as the
      least-privilege `wpm_app` (`permission denied for schema public`). They now send that DDL through `crate::requests::ddl`, which uses the schema owner's connection
      `WPM_TEST_DDL_URL` when the drill sets it and the test's own connection otherwise; the tamper tests' `ALTER TABLE ... DISABLE TRIGGER` stay on the app role and are still
      refused. Run locally against a throwaway Postgres 18: 151 passed and the 2 tamper tests refused by the database; the one other failure was the drill passing an
      empty `WPM_TEST_REDIS_URL` (CI sets a real one), removed. Also run locally: `pnpm run lint`, `svelte-check`, 210 unit tests, the UI build, 814 Rust unit tests,
      `route_coverage`, `scheduler_config`, the matcher's fmt and clippy, `check-links`, `check-days`, `check-actions-pinned`, `check-licenses`. **A lesson recorded in
      `AGENTS/testing-and-verification.md`:** run what CI runs before pushing. **CI result:** run 40232083345 on `42f58b4` passed all 12 jobs. **Not done:** the pull requests were closed before CI finished and their comments say CI had not run, which was true then; the full enforcement binaries were not
      re-run after `cargo fmt` (the request suite was, through the drill; the formatting is behaviour-neutral).
- [x] (2026-10-26) WPM-T307 **The MCP endpoint refuses a token that was not issued for it.** *(WPM-R159, WPM-D110)* Pure rule `rules::mcp::audience_ok` (the two URLs compared as written
      after trimming spaces and one trailing `/`; nothing lower-cased or resolved; an empty resource never matches). `controllers::mcp::token_is_for_this_resource` reads `WPM_MCP_RESOURCE`; with it
      unset nothing is bound. In `app.rs`'s `/mcp` branch a token that verifies but whose `aud` is not the resource is a `401` ("not issued for this MCP resource"), which `challenge_mw`
      gives the `resource_metadata` pointer. The service has one audience (the verifier checks one), so the deployer sets `WPM_TOKEN_AUDIENCE` (or the Keycloak audience) to the resource URL;
      WPM-D110 records why there is no second audience. `scripts/check-mcp-flow.mjs --print-env` now prints `WPM_TOKEN_AUDIENCE` and the script mints tokens for the resource.
      **Verified:** a unit test of the rule (equal, trailing slash and spaces, other path, other host, case, empty); `enforcement_pay_gap` with real tokens (a token for `main-x-service` is a `401` on `/mcp`
      with the metadata pointer once the resource is set; the earlier cases are unchanged) and a **mutation check** (the binding made always-true: that test failed, then restored); clippy pedantic clean;
      the independent client run against a real service started with the printed settings: all checks passed. **Not verified:** the positive case in Rust (a token whose audience equals the
      resource passes the binding), because that binary's verifier is a process global with the default audience: the client run covers it, with the audience set to the resource; the check that a
      token for another audience is refused ran in the client by the verifier, and the binding's own refusal only in the Rust test; the Keycloak backend (the binding reads the same `aud` claim, but no Keycloak suite was run);
      a deployment that sets `WPM_MCP_RESOURCE` and leaves the audience at its default now gets `401` on `/mcp` until it sets the audience (documented in `deployment.md`).
- [x] (2026-10-26) WPM-T308 **A manager's count of team equipment.** *(WPM-R160, WPM-D111)* `rules::equipment::team_holding` (pure: `no_one`, `withheld` under the floor, else
      `shown {holding, none}`, never more holding than the team), `GET /api/workers/{pid}/team-equipment-count` (the manager themself or HR; direct reports in work today; people, not items),
      in the need-to-know table and the OpenAPI document, and `TeamEquipment.svelte` on `/me` with `teamEquipment.*` in the 13 locales (AI-written, unreviewed). WPM-D111 records that D35 said "open issues" and this counts
      people. **Verified:** a unit test of the rule; a request test against Postgres 18 (ten reports, three hold, one holds two items and counts once, a return of one of two items keeps the person counted, a return of the
      only item drops them, three reports are withheld, no reports is `no_one`, and the answer holds no item, tag or pid) which also caught my wrong argument order for `is_employed_on` (the compiler's suggested swap was wrong too); an
      enforcement test with real tokens (anonymous 401, a manager reads their own, a worker is refused another manager's with 403, HR reads); `route_coverage`; clippy pedantic; `svelte-check` 0 errors, 210 unit tests, `pnpm run lint`, Playwright 236 passed / 19 skipped
      (new: shown, withheld and no panel for no team). **Not verified:** no screenshot looked at; the API is stubbed in the UI test and the panel was not run against the service; the translations. **Added later (2026-10-26):** the request test now also covers twelve reports where one holds, then one has left and one is soft-deleted: the count goes from 11 to 9 not holding, the holder stays counted (ran against Postgres 18).

## Phase 24 — the electronic staff record, absence and line managers, disability and adjustments, succession, deputies and acting up (proposed WPM-R161–R205, WPM-D112–D133; [plan.md](plan.md) sections P to S)

- [x] (2026-10-27) WPM-T309 **Spec round.** *(WPM-R161–R205, WPM-D112–D133)* Four requests on one day, specified together:
      the electronic staff record and pay deductions ([staff-records-and-pay.md](staff-records-and-pay.md), R161–R178, D112–D120),
      absence, sickness absence, work-related stress and help for line managers ([absence-and-line-managers.md](absence-and-line-managers.md),
      R179–R189, D121–D124), and disability, non-visible disability, neurodivergence and reasonable adjustments
      ([disability-and-adjustments.md](disability-and-adjustments.md), R190–R196, D125–D127), and succession, deputies and acting up
      ([succession-deputies-and-acting-up.md](succession-deputies-and-acting-up.md), R197–R205, D128–D133; succession extends WPM-R12,
      and it found that succession entries are missing from the subject-access export). Each asked-for item was checked against `spec/`
      first: contact details (R124), emergency contacts (R41), voluntary equality monitoring (R125), payroll runs and payslips (R13), skills
      (R43), positions (R152), review cycles, band and step (R54), the compliance view over training enrolments and reasonable
      adjustments (R33) exist and are referenced, not rebuilt. Plan sections P, Q and R, the domain model, auth, audit, regulatory,
      glossary, index, `llms.txt`, `llms.json`, the requirements and design pointers, and the next-free-ids line updated.
      **Checked against sources** (tabled in [staff-records-and-pay.md](staff-records-and-pay.md)): UK student loan thresholds and
      rates for 2026 to 2027 and the multiple-plan and default-plan rules (GOV.UK page fetched); NHS Pension Scheme member tiers from 1
      April 2026 and the 23.7% employer rate (NHSBSA and NHS Employers, through search results, not the page itself); ESR's staff
      group, job role and area of work coding (NHS England guide, search excerpts); Agenda for Change pay progression standards
      (Annex 23 and employers' policies, search excerpts); the Core Skills Training Framework's eleven subjects (Skills for Health,
      search excerpts). **Kept, not reopened:** WPM-D73 (no sick-leave reason: stress is assessed in the work instead, D121), WPM-D25
      (no condition or label: extended by D125), the union stance (no check-off deduction, D119), and WPM-D45/D59/D104 (refined, not
      reversed, by D112). **Not checked:** the student loan rounding rule; the Scottish and Northern Irish pension tiers; the current
      NHS Data Dictionary staff group list; the CSTF's per-subject refresher periods; the break in service agreements disregard; the
      seven-day fit note rule beyond common knowledge (it is configuration, none shipped); the HSE Management Standards text (named,
      not quoted); WDES metric wording; any non-UK jurisdiction. No code, migration, test or string changed, so no suite was run.

**Section P — the electronic staff record and pay deductions**

- [x] (2026-10-28) WPM-T310 **Assignments and categories, pure core.** *(WPM-R161, WPM-R162, WPM-D112, WPM-D116)* The FTE total rule, one primary,
      effective-dated change as end-and-start, job role and staff group agreement, unknown categories.
      **Done:** `rules/assignment.rs`: `fte_total`, `fte_problems` (checks the new start day and every later start inside the new
      assignment; the message names the total), `assignment_problems`, `primary_problems` (exactly one current primary; none current
      says nothing), `end_for_change` (old ends the day before the effective date, so no overlap and no gap), `agreement`
      (`Agrees`/`Mismatch`/`Unknown`; only a stated disagreement is a mismatch) and `category_or_unknown`. Dates are inclusive at both
      ends. 9 unit tests; **mutation-checked** twice (`>` to `>=` on the FTE limit; `<=` to `<` on the change day), both caught.
      `cargo fmt`, `cargo clippy --all-targets -- -D warnings` clean. **Not verified:** nothing is wired in (T311); the full `--ignored`
      request suite was not re-run, as no controller or migration changed. **Choice to confirm:** the cross-organization total
      (R161) is advisory and is not in this module; it belongs with the person-centric record (T313).
- [~] (2026-10-28) WPM-T311 **Assignments and categories, records, API and screen.** *(WPM-R161, WPM-R162)* Migration creating one primary assignment
      per worker from its columns; write-through to the worker row; `positions.holder` to the holding assignment; vocabulary import;
      aggregates with the floor; export and erasure.
      **Part done (2026-10-28), records and API:** migration `000101` (`assignments`; a partial unique index for one open primary; one primary back-filled per
      worker; triggers so a new worker gets a primary, an edit of the worker row's title, department, location or FTE corrects the open primary **in place**, and a
      termination date ends what is open); entity; `GET`/`POST /api/workers/{pid}/assignments` (the worker themself and HR read; HR adds a further assignment; a total over 100
      is a `422` naming it) and `POST /api/assignments/{pid}/change` (end and start; a primary change writes through; `effective_on` today or earlier); categories read
      `unknown` when not recorded; subject-access export and erasure; OpenAPI and the UI's types. **Verified** against Postgres 18 (throwaway, Podman): a request test
      (back-fill and trigger, over-100 refusal, change, write-through, one open primary, trigger correction without history, export, erasure), **mutation-checked** (write-through
      disabled: fails); the full `--ignored` request suite (166), `enforcement`, `enforcement_need_to_know`, `cargo test --lib` (927), clippy clean, `pnpm check` (0/0).
      **Not done:** the category vocabularies and their import, and the job role and staff group agreement check on write (the pure rule exists; with no vocabulary there is nothing
      to ask); `positions.holder` to the holding assignment and `positions.number`; aggregates by staff group, job role and area of work with the floor; ending a non-primary
      assignment; clearing a category; a future-dated change (refused, as nothing would write it through on the day); the screen. **Not verified:** the migration on a database with
      real worker rows (only the back-fill SQL on an empty one, plus the trigger on new rows); a worker whose title or department is over 120 characters is trimmed by the
      trigger, not refused; the UI; the Keycloak suite. **Finding:** a worker edit through the older route changes the open primary in place, so history is made only by
      a change; a deployer who edits the worker row directly loses "what they did on that day".
      **Further part done (2026-10-28), the vocabularies and the agreement check:** migration `000102` (`category_vocabulary`: kind, code, label, standard and its own code
      given together, and a staff group only on a job role; unique per organization, kind and code; none shipped); `GET /api/category-vocabulary?organization=&kind=` (open to a
      signed-in caller in scope) and `POST` (HR); on adding or changing an assignment, a job role whose vocabulary entry names a different staff group than the one recorded is a `422`
      (an unlisted role, a role that names no group, or a blank staff group is `Unknown`, not a mismatch). Request test (entry refusals, uniqueness, listing, both write paths),
      **mutation-checked** (agreement always true: fails); full `--ignored` suite 167, `cargo test --lib` 928, clippy clean. **Not done:** the import of a national list (an ESR
      file), a code outside a non-empty vocabulary is still accepted (the spec does not say it is refused), editing or retiring an entry, a UI. **Also found:** a second change on the
      day an assignment started is refused by the rule (`effective_on` must be after the start), which a HR user will meet when correcting a change made earlier the same day.
      **Further part done (2026-10-28), the aggregates:** `rules::assignment::breakdown` (a person counts once per category, `unknown` is its own cell, a cell under the floor shows
      nothing, **no total is returned** so a hidden cell cannot be found by subtraction) and `GET /api/assignments/breakdown?organization=&by=staff_group|job_role|area_of_work`
      (floor 10, the assignments in force today, no name or worker, the scale ceiling applies). 2 unit tests and a request test (10 workers: a full cell, two under-floor cells, the
      unknown cell, no total, a refused `by`), **mutation-checked** (floor ignored: both fail); full `--ignored` suite 168, `cargo test --lib` 930, clippy clean, `pnpm check` 0/0.
      **Not verified:** complementary suppression across two views (a reader who sets the categories apart by a second request can still difference two cells); the floor is the
      default constant, not the organization's configured one; no UI.

- [~] (2026-10-28) WPM-T312 **Cost centres, financial and subjective coding.** *(WPM-R163–R165, WPM-D116)* Register, unmatched free-text list,
      segment assembly and most-specific-rule derivation in the pure core, position numbers, the coding view.
      **Part done (2026-10-28), the pure core:** `rules/coding.rs`: `structure_problems` and `assemble` (segments from the cost centre, the subjective code, a fixed value or a
      vocabulary kind; a segment with nothing to fill it makes the code incomplete and is named; a value not exactly the segment's length is named and never padded or cut);
      `Rule`, `derive` (the most specific matching rule wins; none is `Uncoded`; a blank fact matches nothing; a rule naming nothing matches nothing; equal matches choose nothing,
      `Ambiguous`), `conflict` (equally specific rules that could match one assignment, including an exact duplicate, are refused when the second is saved), `rule_problems`;
      `CostCentre::open_on`, `check_open` (not in the register, or not open that day, is refused; first and last days are in), `chain` (parents up, loop-safe) and `creates_cycle`.
      12 unit tests, **mutation-checked** three ways (least specific wins; the conflict test ignoring values; the last day excluded), all caught; clippy clean.
      **Not done:** the register, structure and rule tables, API and screens; the unmatched `positions.cost_centre` list; position numbers; the coding view; the cost per cost centre.
      **Not verified:** the pure core above is not wired in on its own; the pay scale band, a rule field, is not yet an assignment fact (T311 holds no band).
      **Further part done (2026-10-28), the register and the assignment link:** migration `000103` (`cost_centres`: code, name, parent, valid from and to, unique per organization;
      `assignments.cost_centre`); `GET /api/cost-centres` (open to a signed-in caller in scope, each with whether it is open today and its parent chain), `POST` (HR; a parent must be
      in the register and open on the first day), `POST /api/cost-centres/{pid}/close` (never deleted, not reopened here), `GET /api/cost-centres/unmatched` (HR: the free-text
      `positions.cost_centre` values that are not a code, with the number of positions, nothing matched or created). Adding or changing an assignment with a `cost_centre` that is not
      in the register or not open on the effective day is a `422`. Request test, **mutation-checked** (open check disabled: fails); full `--ignored` suite 169, `cargo test --lib` 942,
      clippy clean, `pnpm check` 0/0. **Still not done:** the coding structure and subjective rule tables and their endpoints (the pure core exists), position numbers, the coding view,
      the establishment roll-up up the parent chain, the cost per cost centre, `positions.cost_centre` itself pointing at the register, a UI. **Not verified:** a parent that is closed
      later still leaves its children open; the register was not checked against a real finance chart of accounts.
- [~] (2026-10-28) WPM-T313 **Annual service and the person-centric record.** *(WPM-R166, WPM-R167, WPM-D113)* Interval union with a break tolerance
      (pure, clock-free), recognised prior service, `/api/me/record` and the privileged view with other employers reduced to a total.
      **Part done (2026-10-28), the pure core:** `rules/service.rs`: `service` (union of periods with overlaps counted once; a gap up to the
      configured tolerance joins **continuous** service, none set means only touching periods join; total and continuous days and whole
      years, the merged spans, and `None` plus the named references when a period has no start or ends before it starts), `prior_days`
      (recognised prior service, kept apart, never added), `anniversary`. Inclusive days; whole years count from the earliest start, so a
      year ends the day before its anniversary. 12 unit tests, **mutation-checked** twice (the gap comparison; the "reaches the as-of
      date" test for continuous service), both caught; clippy clean. **Not done:** the records (prior-service table, migration), `/api/me/record`,
      the privileged view that reduces other employers to a total, the transferred-training rule, duplicates shown as possible. **Not
      verified:** whole years across several periods are measured from the first start, a reading of R166 that HR should confirm.
- [x] (2026-10-28) WPM-T314 **Statutory and mandatory learning, pure core.** *(WPM-R170, WPM-D117)* Audience matching, the six statuses, refresher
      arithmetic, transferred completions.
      **Done:** `rules/mandatory_learning.rs`: `applies` (audience of staff group, job role, area of work and position against a person's
      current assignments: a stated miss settles it as `No`, a blank category is `Unknown`, any matching assignment is `Yes`, a restricted
      audience with no assignment is `Unknown`, an empty audience is the organization), `status` (the six statuses; the latest completion
      counts; a completion after the as-of date is ignored; a transferred one counts only where the requirement accepts it; once-is-enough
      never expires; due on the as-of day is still in date; `due_soon` inside 90 days), `rollup` (counts per status, `unknown` apart and in
      neither side of the rate, rate `None` with nothing to divide by, `Hidden` below the floor, a floor of 0 read as 1),
      `requirement_problems` (basis, a source for a statutory one, a refresher of at least a month). 15 unit tests, **mutation-checked**
      three ways (expiry on the day itself, the 90-day edge, a stated miss read as unknown), all caught; clippy clean.
      **Not done:** the records, API and screens (T315); the CSTF mapping (a deployer's import). **Choices to confirm:** `due_soon` counts
      as in date in the rate; the window is a constant, not yet configuration; expiry is the completion date plus the months, with the
      calendar's end-of-month clamp.
- [ ] WPM-T315 **Statutory and mandatory learning, records, API and screens.** *(WPM-R170)* Requirements, recorded completions, the
      person's, manager's and HR's views, roll-ups with the floor; extends the existing compliance view.
- [ ] WPM-T316 **Qualifications.** *(WPM-R171)* Declared and verified, role requirements and the qualification gap beside the skill gap.
- [~] (2026-10-28) WPM-T317 **The PADR.** *(WPM-R172)* Review kind, two sign-offs, frozen versions, the due date (pure).
      **Part done (2026-10-28), the pure core:** `rules/padr.rs`: kinds `performance` and `padr`; `SignOffs` (complete when both have signed, completed
      on the later day, frozen once either has signed), `sign` (once per signer, not dated in the future), `check_editable` (a correction is a
      new version), `due_on` and `standing` (twelve months after the last completed PADR; for someone who has never had one, a configured
      number of months after the primary start, none shipped; otherwise `Unknown`; due today is still current). 8 unit tests, mutation-checked
      twice (overdue on the due day; completion on the earlier day), both caught; clippy clean; full `cargo test --lib` 926.
      **Not done:** the review kind column, the PADR record (objectives, development needs, the offered-yes-or-no, versions), API and screens;
      the reviewee's comment is outside this module. **Not verified:** no migration or request test.
- [ ] WPM-T318 **Readiness for the next pay step.** *(WPM-R173, WPM-D120)* Configured standards, the formal-process flag, `unknown` where a
      standard cannot be evaluated.
- [ ] WPM-T319 **Date of birth and age bands.** *(WPM-R169, WPM-D114)* Column, access, the test that no scoring code names it, aggregates.
- [ ] WPM-T320 **Payroll information.** *(WPM-R174, WPM-D115)* No bank or national insurance number column; refusal of look-alikes in free text.
- [ ] WPM-T321 **Deductions.** *(WPM-R175, WPM-D119)* Order, the net-pay floor and held deductions in the pure core; no union check-off.
- [ ] WPM-T322 **Pensions.** *(WPM-R176, WPM-D118)* Schemes, tiers by annualised pensionable pay, memberships, eligibility reporting.
- [ ] WPM-T323 **Student loans.** *(WPM-R177, WPM-D118)* Plans, dated thresholds with source, one undergraduate plan plus postgraduate,
      the default-plan setting, refusal without configuration.
- [ ] WPM-T324 **My payslips.** *(WPM-R178)* Approved and paid only, year to date per tax year, print stylesheet, payslip queries.
- [ ] WPM-T325 **The staff record screen.** *(WPM-R168)* Assembles the sections; not visible, none recorded and unknown shown apart.

**Section Q — absence, sickness absence, work-related stress, and help for line managers**

- [ ] WPM-T326 **Sickness episodes, certificates and return-to-work conversations.** *(WPM-R179, WPM-R180, WPM-D123)* Episodes derived
      from requests (pure), certificate due, fit note as dates and one answer, the conversation record with versions.
- [ ] WPM-T327 **Review points, long-term absence, keeping in touch, phased return.** *(WPM-R181, WPM-R182, WPM-D122)* Rolling-window
      thresholds (pure), outcomes, contact plans that can pause, phased return weeks.
- [ ] WPM-T328 **Occupational health referrals.** *(WPM-R183, WPM-D123)* Consent before sending; advice not stored; the OH attribute.
- [ ] WPM-T329 **Stress risk assessments.** *(WPM-R184, WPM-D121)* Six areas, actions, "related to work" on an episode, pulse mapping.
- [ ] WPM-T330 **Absence measures.** *(WPM-R185)* Rate, short- and long-term, work-related share, conversations on time; feeds R147.
- [ ] WPM-T331 **The manager's action list.** *(WPM-R186, WPM-R189, WPM-D124)* Derived items (pure, as-of date), own reports by name, HR
      counts per manager with the floor, the people-manager audience, asking HR for help.
- [ ] WPM-T332 **Guidance pages.** *(WPM-R187)* CMS content per locale, labelled generic, linked from each action.
- [ ] WPM-T333 **Changing manager.** *(WPM-R188)* The "getting to know your team" item; no handover of adjustment words.
      **Part done (2026-10-29), not ticked.** Built: the pure core `src/rules/manager_change.rs` (change detection, the dated 60-day
      window, `GetToKnowItem` with counts, one date and a passport yes/no, and no text field), 6 unit tests including two that fail if
      the payload or the type gains a field or a text type (mutation-checked: window bound, change detection and an added `String`
      field each fail a test). Verified: `cargo test --lib` 866 pass, `cargo clippy --all-targets -- -D warnings` clean. **Not
      built**, because its inputs do not exist yet: the facts come from WPM-T326/T327 (absence plans, review points), T331 (the
      action list the item appears on), T334 (the passport) and T310-series PADR/learning records, none of which is done; so no
      controller, migration, openapi, UI or strings, no request test, and no DPIA or inventory change (nothing is stored or
      exposed). Remaining: wire `items()` into the action list once T331 exists and add the request test there.

**Section R — disability, non-visible disability, neurodivergence and reasonable adjustments**

- [ ] WPM-T334 **The adjustments passport and "working with me".** *(WPM-R190, WPM-R191, WPM-D125, WPM-D126)* Person-owned, keyed by
      `person_ref`, shares by entry and person, withdrawal, re-agreement with a new manager.
- [ ] WPM-T335 **Adjustment reviews, movements, change impact and absence adjustments.** *(WPM-R192, WPM-R193, WPM-D127)* Review dates,
      the movement checklist item, the HR-only count from the floor, `absence_review_points` adjustments, disability leave.
- [ ] WPM-T336 **Candidate adjustments and support schemes.** *(WPM-R194, WPM-R195)* Kept from scorers (a test fails if scoring names the
      table), retention with the application, guaranteed interview opt-in, Access to Work status.
- [ ] WPM-T337 **Disability equality measures.** *(WPM-R196)* From equality monitoring only; `unknown` without it; WDES mapping by number.

**Section S — succession, deputies and acting up**

- [ ] WPM-T338 **Succession on positions and roles.** *(WPM-R197)* Plans linked to a position or role profile, derived incumbent,
      `vacant`, emergency cover, the unlinked free-text plans listed for HR.
- [ ] WPM-T339 **Readiness evidence, review dates, the successor's view.** *(WPM-R198, WPM-R199, WPM-D128, WPM-D129)* Evidence beside
      readiness (no score), development items linked, `not_reviewed`. Notify the successor when they are named, changed or removed
      (role and date only); add `/api/me/succession` showing their own entry without the rank, the other candidates or the incumbent's
      risk; add succession to the subject-access export, with a test of the export round-trip and a test that no setting suppresses the
      notice; interested people from mobility interests.
- [ ] WPM-T340 **Slate equality.** *(WPM-R200)* From equality monitoring only, across critical roles, with the floor.
- [ ] WPM-T341 **Deputies, pure core.** *(WPM-R201, WPM-R202, WPM-D130)* Eligibility, acceptance, when a deputyship is active, routing
      to the first available deputy, the closed scopes, no self-dealing, never-delegated items, and the flexible-working grant (WPM-D133):
      the holder sets it; HR may set it only while the holder is unavailable, and it lapses on their return unless they confirm it;
      the deputy never can; `null`, missing and `false` deny. Tests cover each of these, including HR refused while the holder is
      available.
- [ ] WPM-T342 **Deputies, records, API and screens.** *(WPM-R201, WPM-R202)* Routing in leave, time, expenses (closes the expense
      claims' "approval delegation"), training and return-to-work; `flexible_working:decide` as an ABAC operation with
      `resource.deputy_may_decide_flexible_working`, the holder's grant, HR's grant on their behalf with the notice to the holder and the lapse, confirmation and withdrawal (audited), and an enforcement test that a deputy without the
      grant is refused; "as deputy for" in the audit and on the record; the action list.
- [ ] WPM-T343 **Acting assignments and pay.** *(WPM-R203, WPM-R204, WPM-D131)* Kind `acting`, substantive on hold, acting cover kept
      vacant in the establishment, extensions, pay treatments as configuration, the earnings line and arrears.
- [ ] WPM-T344 **Acting up limits and fairness.** *(WPM-R205, WPM-D132)* Maximum duration on the action lists, expressions of interest,
      open versus direct counts, the equality share from monitoring.
- [x] (2026-10-28) WPM-T345 **Absence categories: paid, unpaid and by policy.** *(WPM-R5, WPM-D73, WPM-D121)* A table of leave categories
      with pay, balance rule and reason rule, replacing the two hard-coded cases; a government holiday kind; a listing by pay.
      **Done:** `rules/absence_category.rs` (`CATEGORIES`: `annual` paid and enforced, `public_holiday` paid with no balance, `sick` by policy,
      may go negative, never a reason, `parental` and `other` by policy and enforced, `unpaid` unpaid with no balance); `rules::leave::check_balance`
      and `validate_reason` read it; `public_holiday` added to `LEAVE_KINDS`; `GET /api/absence-categories?pay=paid|unpaid|policy` (open);
      the time-off picker offers the new kind; its label in all 13 locales (machine-drafted, unreviewed). 6 unit tests, including one that fails if a
      category names a condition or stress, and one that keeps the table in step with `LEAVE_KINDS`; a request test (the listing and each filter, a
      holiday request accepted, a sick request with a reason refused, an unknown kind such as `workplace_stress` refused).
      **Mutation-checked** four ways (a holiday with a balance, an unpaid category marked paid, sick leave allowing a reason, sick leave with an enforced balance),
      all caught. Full `--ignored` suite 170, `cargo test --lib` 948, `pnpm test` 218, clippy and `pnpm check` clean.
      **Not done, on purpose:** a workplace-stress category. It would record a health matter against a person, which WPM-D73 and WPM-D121 rule out;
      stress belongs to the work assessment (absence-and-line-managers.md). If the product owner wants it, that is a decision to reverse D121, not a
      setting. **Also found and fixed:** the earlier T311, T314 and T317 tokens (`site`, `area_of_work`, `job_role`, `staff_group`,
      `occupation_code`, `mandatory`, `performance`, `padr`) had no entry in `values` and the UI unit test failed; labels added in 13 locales,
      two French words that equal the English (`site`, `performance`) allow-listed. **Not verified:** pay by policy is not looked up anywhere (no
      sick pay or entitlement is computed); no entitlement is created for `public_holiday`; a holiday calendar is not built, so a holiday is
      recorded by hand; the Playwright specs, axe and the Keycloak suite were not run.
- [x] (2026-10-28) WPM-T346 **The holiday calendar.** *(WPM-R5, WPM-D116)* Government holidays generated per country or region, not entered one by one.
      **Done:** `rules/holiday.rs` (rule kinds `Fixed`, `Nth`, `Last`, `Easter`; weekend policies `Stays`, `NextWeekday` (a substitute that never lands on another
      holiday, placing weekday holidays first), `Nearest`; Easter by the Gregorian algorithm; `generate`, `is_holiday`, `is_business_day`; packs `GB-ENG`, `US`, `DE`, each
      `verified: false`); migration `000104` (`holidays`, unique per organization, region, natural date and name); `GET /api/holiday-regions`, `GET /api/holidays`
      (open to a signed-in caller in scope), `POST /api/holidays/generate` (HR; idempotent: a row it made is updated, a hand-added row is kept) and `POST /api/holidays` (HR).
      11 unit tests (Easter for six years; England 2026, with the Boxing Day substitute; Christmas on a weekend for 2021, 2022 and 2027 and every year 2015 to 2039 with eight
      distinct weekday days; the US Saturday-to-Friday and Sunday-to-Monday observance, including a New Year observed in the year before; Germany from Easter) and a request test
      (listing, refusals, generation, repetition, regions kept apart, a hand-added day, an update and a kept manual row). **Mutation-checked** five ways (the substitute clash
      check; the weekday-first placement; the Saturday/Sunday direction for `Nearest`; Easter's day; manual rows protected), all caught. `cargo test --lib` 959, clippy clean, `pnpm check` 0/0.
      **Not verified, and why it matters:** **the packs were not checked against any government list**; they are from the author's knowledge and marked unverified. Missing by design:
      Scotland, Northern Ireland, US states, one-off days (a royal funeral, a jubilee), and every other country. **Not done:** a worker's region (so the calendar is per organization and
      region, not yet per person), the leave day count skipping a holiday, a `public_holiday` leave entry created from the calendar, a UI.
- [x] (2026-10-28) WPM-T347 **ESLint and Prettier as one lint gate for the UI.** *(testing and verification)* Prettier checked only `src`; there was no ESLint.
      **Done:** `eslint.config.js` (flat config: `@eslint/js` and `typescript-eslint` recommended, `eslint-plugin-svelte` recommended, Prettier's config last so formatting
      is Prettier's alone); `pnpm lint` is now `eslint .` then `prettier --check` over `src`, `tests`, `scripts` and the config files, so CI's existing `pnpm run lint` step runs both;
      `pnpm lint:fix` and `pnpm format` cover the same files. The first run found **79 problems in 40 files**; 61 were one rule and are off with a recorded reason, 18 were real and
      are fixed (six unused names, two unused assignments, a regex whose character ranges held combining marks, a `let` that was never reassigned, an unneeded `svelte-ignore`).
      56 test and script files were formatted once (formatting only; the 218 unit tests, `pnpm check` 0/0 and the build-free checks are unchanged).
      **Rules turned off, and why:** `svelte/no-navigation-without-resolve` (every link is built by `l()`, which puts the locale in the URL, and `resolve()` knows nothing of it);
      `svelte/prefer-svelte-reactivity` (the three `Map`s are built inside `$derived` as local lookups); `svelte/no-useless-mustaches` (`{" "}` keeps a deliberate space);
      `no-empty-object-type` for `src/app.d.ts` only (SvelteKit's template). `tests/unit/untranslated.allow.json` is left out of Prettier because a script writes it.
      **Verified:** a probe file with an unused variable fails `pnpm lint` at ESLint, and one badly formatted fails it at Prettier; both removed. **Not verified:** the Playwright
      specs were reformatted but not run; CI was not run (the workflow is unchanged because `pnpm run lint` was already a step); the service-side Rust gates are unchanged.
- [x] (2026-10-28) WPM-T348 **Single-use hashed link tokens.** *(auth.md, WPM-D73 spirit)* A token that works once, stored only as a hash.
      **Done:** `rules/link_token.rs` (32 random bytes as 43 URL-safe characters; `well_formed` refuses anything else before it is hashed; `hash` is SHA-256 in hex, checked against two
      standard vectors; a purpose of 1 to 60 snake-case characters; a life of 1 minute to 7 days; `check` with the reason, expiry exclusive); migration `000105` (`link_tokens`, hash
      unique and checked as 64 hex characters, a trigger that refuses any change to `used_at` once set); `models::link_tokens::{issue, consume, purge_expired}` (consume is one atomic
      `UPDATE ... RETURNING`; every refusal looks the same to the caller and the reason goes to the audit trail, never the token). 8 unit tests and a request test (bad purpose or life
      refused; no column for the token and the token text is nowhere in the table; wrong purpose refused without using it up; malformed input; works once; `used_at` cannot be cleared;
      expiry at the minute; reasons in the audit trail and the token not; purge). **Mutation-checked** three ways (the unused test dropped; the purpose test dropped; the expiry test
      dropped), all caught; a fourth (the length test in `well_formed`) is an equivalent mutant because the base64 decode already fixes the length. `cargo test --lib` 967, clippy clean.
      **Not done:** any consumer or endpoint; rate limiting; a scheduled purge; two simultaneous consumes were not tested (the test pool holds one connection), the atomicity rests on the
      single statement. **Not verified:** the full `--ignored` suite was not re-run for this change.
- [x] (2026-10-28) WPM-T349 **Small-cell suppression as one rule, with a guard.** *(WPM-D35)* The floor comparison was written by hand in 17 rule files (about 40 places), with
      different handling of zero, and nothing stopped a table from giving a hidden cell away by subtraction.
      **Done:** `rules/suppression.rs`: `visible(count, floor)` (a zero is shown, a floor of 0 is read as 1) and `with_total(cells, floor)` (hides the cells under the floor and then
      protects the total: if exactly one cell is hidden, the smallest other non-zero cell is hidden too, or the total itself when there is none). An exhaustive test over every table of up
      to five cells with counts 0 to 6 and floors 1 to 4 (about 78,000 tables) checks that a shown count is the real one, that every small cell is hidden, and that a shown total never
      leaves exactly one hidden cell. `assignment::breakdown` and `mandatory_learning::rollup` (the two newest) use `visible`. A guard test fails when any other file under `src/rules/`
      writes its own floor comparison (`< floor`, `>= floor`, `floor.max(1)`, `>= GROUP_FLOOR`) or when a file listed in `OWN_COPY` no longer does, so the list of 17 can only shrink.
      **Verified:** the guard fails for a probe file with `n >= floor` and fails with a stale entry (`appraisal.rs` rewritten); the probe was removed. `cargo test --lib` 976.
      **Not done:** the 17 existing files are not moved to the shared rule (`bargaining_pack` and `pay_range` already do their own complementary suppression and `human_capital`,
      `funding`, `equality_monitoring` and `resignation` do more than a count; each needs its own care and tests); the helper handles counts, not rates or means; it does not defend
      against two overlapping tables read together or against a reader narrowing a hidden count to a range; no controller was changed.
- [x] (2026-10-28) WPM-T350 **No UI page shows API error text.** *(WPM-R94 spirit, WPM-T259, WPM-T261, WPM-T265)* `errorText` added the service's message after the catalogue's words
      for a `403`, `409` or `422`. That text is English whatever the locale, written for developers, and can name a table or a rule.
      **Done:** `errorText` returns the catalogue's words for the status and nothing the service said; `tests/unit/no-api-error-text.test.ts` scans all of `src` (exempting the
      API client, `write.ts`, server code and `+server.ts` routes) and fails when a catch block, a promise `.catch`, or a file that handles `ApiError` reads an error's `message`,
      `body`, `description` or `detail`, or puts the error in a template string; the detector has its own tests. The two Playwright specs that expected the service's words (the payroll `403`,
      the skill-delete `409`) now expect the catalogue's words and that the service's text is absent. **Verified:** the guard fails for a page that reads `cause.message` and for
      a template string of a caught error, and passes on the real source; `pnpm test` 225, `pnpm lint`, `pnpm check` 0/0; the two changed specs pass in Chromium.
      **A cost, stated:** a refusal no longer says *which* field or rule (`"end date is before start date"`); a person sees "That was not accepted". The remedy is a structured error code the
      page maps to its own catalogue sentence, which the service does not return yet. The `invalid` and `conflict` sentences end without a full stop because they used to be followed by a
      colon and the detail; they read as short statements and were not reworded in 12 locales. **Not done:** the service's error bodies are unchanged; server logs still carry the detail.
- [x] (2026-10-28) WPM-T351 **An accessibility (axe) check that covers every route.** *(WPM-R94 spirit, accessibility statement)* Only three specs ran axe and nothing said which pages they missed.
      **Done:** `tests/e2e/axe-routes.ts` (all 56 page routes and the URL visited for each; an `EXCLUDED` table, empty, for any route left out with its reason);
      `tests/e2e/a11y-routes.spec.ts` (axe's WCAG 2 A and AA rules on each route, the report naming the rule, the elements and axe's own explanation with the contrast ratio);
      `tests/unit/axe-covers-every-route.test.ts` (fails when a `src/routes/**/+page.svelte` is in neither table, when a table names a route that is gone, when a route is in both, and when a
      URL does not belong to its route). **It found three real defects, now fixed:** the `ok` chip (4.0:1), the kanban column count (3.98:1, faded with `opacity`) and, by the same
      arithmetic, the `warn` chip (4.18:1); the chip text is darker (`#156a3a`, `#a8480f`, 6.1:1 and 5.3:1) with the border unchanged, and the count uses the muted colour.
      **Verified:** 56 of 56 pass in Chromium; the coverage test passes (5). **Not verified, and what this does not prove:** axe finds some problems, not all (it does not judge reading
      order, focus handling, meaning of text or alternatives that are present but wrong); each route is checked in the state the stubbed API gives it (loading, empty or refused),
      so a page with data on screen is covered only where a specific spec does that (SFIA, bargaining); light theme and the default locale only; no dark theme, no right-to-left locale,
      no phone width; a screen reader was not used; the independent accessibility audit (WPM-T202) remains a person's task.
- [x] (2026-10-28) WPM-T352 **A translation-review register that cannot lie, beside the key-parity test.** *(WPM-R94, WPM-D64)* Key parity (every `-001` locale holds every key, a regional one invents none)
      was already pinned in `wpm.test.ts`; the register (`content/locales/REVIEW.md`) was prose with no test behind it.
      **Done:** `tests/unit/translation-review-register.test.ts` fails when a translated locale has no row or a row names a locale that does not exist, when a locale appears twice, when a row does
      not say who wrote it, when a review is claimed in part (a reviewer without a date, scope and outcome) or with a date that is not a date, and when every row is unreviewed but the page stops
      saying so. The checker has its own tests. The register records today's nine value words as machine-drafted and unreviewed, and names the terms of art a reviewer should check.
      **Verified:** passes on the register; fails with a row removed and with a review claimed in part (restored). **Not done:** no translation was reviewed by a person (WPM-T155 stays open);
      the test checks the register's form, not that a named reviewer exists.
- [x] (2026-10-28) WPM-T353 **A personal-data inventory that is checked, and the gaps it found.** *(WPM-R91, WPM-R92, WPM-D22)* The retention schedule covered the 56 soft-deleting tables; the DPIA and the record of
      processing were prose. Nothing listed every table that holds a person's data or checked that each is exported and erased.
      **Done:** `tests/personal_data_inventory.rs` (no database) reads the migrations for every table with a person column (`worker_pid`, `person_ref` and ten others), and fails when such a table is
      missing from the subject-access export, or is in neither the erasure statements nor `rules::privacy::kind_of`, unless it is named in `NOT_EXPORTED` or `NOT_ERASED_OR_RETAINED` with a reason; it fails
      for an exemption that is no longer needed or names a table that is gone. It writes `spec/governance/personal-data-inventory.md` (66 tables: linked by, export, erasure, retention kind, reason) and fails
      when that is out of date. **It found real gaps, now fixed:** `time_entry_weeks` and `organization_memberships` were in neither the erasure nor the retention schedule; `onboarding_items`,
      `time_entry_weeks` and `organization_memberships` were not exported; an onboarding item's free-text waiver reason survived erasure. Four statements appended to `erasure_statements`
      (weekly totals and memberships deleted, the waiver reason scrubbed, succession entries deleted) and three tables added to the export; the audit counts gained four keys.
      Record of processing rows 18 (assignments) and 19 (link tokens) and DPIA risks R18 and R19 added. **Verified:** a probe table with `worker_pid` and no handling fails the test; removing an
      erasure statement fails it; a request test seeds the four tables, checks the export carries three (and not the succession rating, WPM-D128) and that erasure leaves none of the deleted
      rows and no waiver reason, and fails when the scrub is disabled; full `--ignored` suite 173, `cargo test --lib` 976, clippy clean.
      **Recorded gaps, since closed (2026-10-29):** applicants now have `GET /api/candidates/{pid}/subject-access` (privileged; candidate, applications, interviews with interviewers' notes, assessments and results; audited); the `appraisal_nominations` "not exported" label was wrong (the export carried them as `appraisal_nominations_as_rater`), and the real gap was that the sweep hard-deleted an appraisal and left its nominations and responses, now removed first and the table put on the retention schedule. **Still open:** erasing an applicant on request before consent expires; a redacted view of a person's own succession entry is not built. **Cannot see:** a table linked by a column with another name, personal data in a free-text
      column of a table not keyed to a person, a table built by SeaORM migrations rather than raw SQL, and whether a statement is *right*. The retention schedule's own list of 56 tables was not
      re-derived. **Not verified:** the Keycloak suite; a restore-and-replay drill with the new statements.

## Phase 25 — cross-app bridges (proposed WPM-R206–R211, WPM-D134–D136; [cross-app-bridges.md](cross-app-bridges.md))

- [x] (2026-10-28) WPM-T354 **Spec round.** *(traces to WPM-R206–R211, WPM-D134–D136)* [cross-app-bridges.md](cross-app-bridges.md), the registries and [integrations.md](integrations.md).
      Found while writing it: the agency link already exists in part (`worker_contractor_details.supplier_ref`, validated as an `organization:` URN), and the `worker_hired` event carries no
      `person_ref`, so a consumer cannot tell who was hired. **Not read:** PPM's code is not on this machine, and CRM was looked at only as far as its file list and its use of `person_ref`;
      every PPM contract is an assumed shape and marked unverified (WPM-D136). No code, migration or test changed in this task.
- [x] (2026-10-28) WPM-T355 **Hire and termination events carry the person's ref.** *(WPM-R207)* `worker_hired` and `worker_terminated` gain `data` with `person_ref`, `organization_ref` and `effective_on`, nothing else;
      a test pins the exact keys.
      **Done:** `rules::bridge::handoff_data` (one function, keys pinned as `HANDOFF_KEYS`, no personal word in the payload) used in three places: hiring from an application (`worker_hired`, which had `data: None`),
      **direct creation by HR (which emitted only `created`, so no hire event existed: `worker_hired` is now emitted there too)**, and `worker_terminated` and `worker_retired` (their `from` and `to` kept). A request test
      reads the envelopes and pins the exact key set for hire (3) and for termination (5), and that no name or department is in them. **Mutation-checked** three ways (the direct-creation event not emitted; an extra
      field in the payload; the overlap check's employed filter dropped), all caught. **Not done:** CRM's side; a consumer must be idempotent on `event_id` (the outbox is at-least-once); the in-memory transport
      loses events on restart (the outbox transport does not). **Changed behaviour to know:** webhook subscribers to `worker_hired` now also receive it for directly created workers.
- [x] (2026-10-28) WPM-T356 **The supplier's contact.** *(WPM-R206)* `supplier_contact_ref` (a `person:` URN) on contractor details; validated, exported with the worker, erased with them.
      **Done:** migration `000106` (a nullable column, checked as a `person:` URN), the entity, validation through the existing `ref_opt`, and the response; the table was already in the export and the erasure, so the new
      column goes with them. A request test: an organization URN, a string and a malformed `person:` are refused; a valid one is stored and read back; leaving it out clears it. **Not done:** the name lookup (WPM-T358).
- [x] (2026-10-28) WPM-T357 **The overlap check.** *(WPM-R208, WPM-D135)* `POST /api/bridges/overlap`: up to 500 `person:` refs, a yes or no for each, HR under their own token, audited with the count only.
      **Done:** `rules::bridge::{overlap_problems, overlap}` (1 to 500 well-formed `person:` URNs; one answer per distinct ref in the order first seen; nothing about a person not named) and `POST /api/bridges/overlap`
      (HR of the organization, in scope, under the caller's token; employed today means hired, not left, not deleted; the audit entry holds the counts and no reference, written inside a transaction as the audit guard requires,
      which a first version missed and the guard caught). Request test: yes for a worker, no for a stranger, repeats answered once, no pid in the answer, four refused shapes, no reference in the audit trail, and no longer a
      worker once terminated. **Not done:** a push to CRM; the endpoint is not rate limited; it tells an HR caller whether a person they name is employed by their organization, which is the fact HR reads on the worker list.
      Verified: full `--ignored` suite 175, `cargo test --lib` 979, clippy clean, `personal_data_inventory` passes.
- [x] (2026-10-28) WPM-T358 **Supplier and contact names, best-effort.** *(WPM-R206)* Best-effort, cached, never blocking a write. **Re-scoped 2026-10-28:** CRM was read and has no lookup by reference and no person names, so the source is person-service (see `cross-app-bridges.md`, "What CRM was read to say"). **Built:** `GET /api/workers/{pid}/contractor-details` adds `supplier_name` and `supplier_contact_name` through `clients::display_name` (cache, then the owning service in `http` mode); unresolved is `null`, never an error, nothing is stored, and a person's name stays in the process cache only (not the shared one). The write's answer carries no name. **Not verified:** the live call to person-service or an organization service (stub mode only).
- [x] (2026-10-29) WPM-T359 **Capacity what-if for PPM.** *(WPM-R209)* Is WPM-T158 to T160, all done. **Not done here:** PPM's side of the call (T157, outside this repository) and the product owner's confirmation of WPM-D142.
- [x] (2026-10-30) WPM-T360 **Programme demand in.** *(WPM-R210, WPM-R213)* Idempotent on programme and source event. **Closed 2026-10-30** when the product owner fixed the family contract (`cross-app-bridges.md`, "The family contract"): PPM calls the existing `POST /api/capacity/peer-demand` (`{pool_pid: <WPM pool pid>, programme_ref: "thing:<ppm plan pid>", month: YYYY-MM-01, fte_centi, peer_event_id}`) under the forwarded caller's token, and pulls `GET /api/capacity/peer-supply`. The receive route, its idempotence and its refusals were built and tested in WPM-T364 and T366; **nothing in WPM changed for this task.** **Not verified:** PPM's sender (being built by PPM's owners) has not been run against WPM; no end-to-end call with a real forwarded token has been made; whether PPM retries with the same `peer_event_id` after a timeout (which the route needs to stay idempotent) is PPM's to show; an older event replayed after a newer one still overwrites it (T364).
- [x] (2026-10-30) WPM-T361 **People-impact out.** *(WPM-R211, WPM-D134, WPM-D135)* Counts with the floor, for a programme. A pull, as the family contract fixed it (`cross-app-bridges.md`).
      **Done:** `rules::people_impact` (pure: `workers_committed`, `contingent_ending`, `over_committed`, each a `Sub::Known` or `Sub::Unknown(reason)`; the floor of ten through `rules::suppression::visible`; no total) and `GET /api/capacity/people-impact?programme=thing:<uuid>&ends_on=&months=` (`capacity/people_impact.rs`): the people employed today whose post the programme funds, by the pools they count towards and in no pool; the fixed-term and contract engagements among them ending on or after today and before `ends_on`; the over and unknown pool-months among those the programme holds a counted claim in. Under the caller's scope; audited in a transaction (`people_impact_asked`, no counts); classified `Open` in `rules::access::READS`; in the OpenAPI document. **Derivation:** see the task's section in `cross-app-bridges.md`; excludes people whose funding has ended, people not employed today, other organizations than the caller's, and claims the pool does not count. **Unknown, not zero:** no funded post (the first two parts), no `ends_on` (WPM holds no programme end date; `ends_on` is an optional parameter added to the fixed contract so PPM can give its plan's end), no counted claim. **Verified:** 6 unit tests; a request test (12 in pool A, 3 in pool B hidden, one contractor in no pool with its end nulled as an older record, one worker of another programme left out; without `ends_on` the ending part is unknown with a reason; with it 10 end in pool A and 0 in B; one over pool-month; no name, number, worker pid or `person:` in the answer; a programme WPM knows nothing of is unknown in all three parts; three malformed requests refused; three audit rows); the persona binary `enforcement_what_if` (HR, a planner and a manager get counts only; another organization's HR is told every part is unknown and no pool is named; a partner 403; signed out 401). **Mutation-checked** twice (the floor removed; an empty programme made known), both caught. **Not tested:** a caller in two organizations; a linked pool the peer has sent nothing to (its months are unknown by T366's rule and are counted in `unknown`, not run here); differencing across overlapping answers. **Not verified:** PPM's call (being built by PPM's owners).
- [x] (2026-10-30) WPM-T372 **A hire or an ending tells a webhook subscriber who.** *(WPM-R151, WPM-R207, proposed WPM-D145; family contract point 5)* For `worker_hired`, `worker_terminated` and `worker_retired` the webhook payload gains `data`, exactly `rules::bridge::handoff_data`'s keys (`person_ref`, `organization_ref`, `effective_on`) plus `from` and `to` for the two endings; every other kind has none; the signature is unchanged.
      **Done:** `rules::webhook::{data_keys, payload_fields, delivery_data, HIRED_DATA_KEYS, ENDED_DATA_KEYS, PAYLOAD_FIELDS_WITH_DATA}` (pure; a kind's keys are pinned; an event lacking a key sends no `data` rather than part; an extra event field never leaves); `payload` takes the kept data. Migration `000108` adds `webhook_deliveries.data` (JSONB, null), with a `CHECK` that allows it only for the three kinds. `streaming::enqueue_webhooks` writes `delivery_data` in the change's transaction; `webhook_deliver` sends it; the delivery log does not list it. **Verified:** 4 unit tests in `rules::webhook` (the pinned-fields test now pins the key set per kind: four fields for `worker_activated` and `leave_approved` even when the event carries `from`/`to`, five with `data` for the three, and the exact `data` keys for each); the request test `webhooks` checks a sent termination (five `data` keys, `from` `offboarding`, `to` `terminated`, `person:` and `organization:` references), a sent hire (`data` equal to the worker's `person_ref`, organization and `hired_on`), a sent activation with no `data`, no name, number or department in either, the log without `data`, and the schema refusing `data` on `leave_approved`. **Mutation-checked** twice: `data` kept for a kind with no keys (unit tests fail) and the event's data passed through unfiltered (the activation's delivery is refused by the schema `CHECK` and the request test fails). **Not done:** the product owner's confirmation of WPM-D145; CRM's receiver (CRM's owners build it); `scripts/check-webhook-flow.py` still subscribes to `worker_activated` only and was not run. **Changed behaviour to know:** an existing subscriber to the three kinds now receives a fifth top-level field.
- [x] (2026-10-28) WPM-T362 **MHRA entry in the regulatory register.** *(WPM-R212)* See [mhra.md](mhra.md). The workforce questions stay open until the linked guidance is read.
- [x] (2026-10-28) WPM-T363 **Pool link to a peer's pool.** *(WPM-R213, WPM-D139)* See `cross-app-bridges.md`, "Working alone, and working together".
      **Done (2026-10-28):** migration `000107` adds `skill_pools.peer_pool_ref` (a lower-case hyphenated UUID, unique per organization among live pools) and `PUT /api/skill-pools/{pid}/peer-link` sets or clears it, audited without the id; the pool JSON shows `peer_pool_ref` and `demand_source`.
- [x] (2026-10-28) WPM-T364 **Sourced demand claims.** *(WPM-R213, WPM-D140, WPM-D141)*
      **Done (2026-10-28):** `programme_demands` gains `source` (`local`/`ppm`), `peer_event_id` and `received_at`; the live-claim index now includes the source, so a received and a local claim for the same programme, pool and month sit side by side. `POST /api/capacity/peer-demand` stores a received claim as `proposed` (a start decision still activates it, WPM-D50), refuses an unlinked pool, and is idempotent on the last event id of a claim. `rules::capacity::counted_source` is the one rule: a linked pool counts `ppm` claims, an unlinked pool `local`, never both; the view, the what-if and the start check all use it. **Not done:** the capacity view does not yet show the claims it ignores beside the counted figure (D140's "entered here"; the claim list shows every claim with its source); an older event replayed after a newer one overwrites it (only the latest event id is remembered); the receive route is WPM's own definition and PPM has not been read to call it.
- [x] (2026-10-28) WPM-T365 **Outbound supply for a linked pool.** *(WPM-R213, WPM-D135)*
      **Done (2026-10-28):** `GET /api/capacity/peer-supply?pool=&from=&months=` gives a linked pool's supply by month in centi-FTE, **hidden (`null`, `hidden: true`) in any month made up of fewer than five workers** (`rules::capacity::peer_supply` over the shared floor; the headcount is the distinct workers employed in the month, from the same query as the supply). Names no worker, no total, answered under the caller's own token and scope; an unlinked pool is refused. **Not done:** nothing pushes the figure to the peer; the peer asks. Differencing across two linked pools that share a worker is not defended.
- [x] (2026-10-28) WPM-T366 **Standalone and linked tests.** *(WPM-R213, WPM-D138)*
      **Done (2026-10-28):** a request test answers the view and the what-if with the peer unset, silent and removed. Doing it found a real fault: a linked pool the peer had said nothing to counted **zero** demand and so *fit*, against WPM-D141. It is now `unknown` in every month (`rules::capacity::with_demand_unknown`, and `PoolView::demand_known` so a what-if cannot make it fit), until the peer sends a claim; unlinking returns the local figures. Mutation-checked three ways (the floor, the unknown overlay, the what-if keeping it), all caught. WPM makes no outbound call, so "the peer is slow or down" is the same as "silent"; no timing was tested.
- [x] (2026-10-29) WPM-T367 **Worldwide audit of the spec**: 19 rows, see [worldwide.md](worldwide.md). T368 to T371 remain.
- [x] (2026-10-29) WPM-T368 **Jurisdiction profile model**: a design, proposed WPM-D144, five questions for the product owner; see [worldwide.md](worldwide.md). No code.
- [ ] WPM-T369 and T371 **Worldwide target.** *(WPM-R214, WPM-D143)* Second-profile tests and a locale and convention equity check. See [worldwide.md](worldwide.md).
- [x] (2026-10-30) WPM-T370 **Regulatory register by region.** *(WPM-R214, WPM-D143)* Spec only: [regulatory.md](regulatory.md) ends with the register by region, the UK one region among eleven, the others *not yet read*; the wording fixes of audit rows 10 and 11. See [worldwide.md](worldwide.md).
- [x] (2026-10-30) WPM-T373 and WPM-T376 **Money in the UI is worldwide.** *(WPM-R214, WPM-D143)* One formatter and parser (`src/lib/money.ts`, shared in behaviour with the sibling PPM UI's): the currency's own exponent (JPY 0, KWD 3), the page's locale, a decimal comma read, no starting currency. See [worldwide.md](worldwide.md). Percentages on `/planning` remain (T375).
- [x] (2026-10-30) WPM-T375 **Dates and numbers through the page's locale.** *(WPM-R214, WPM-D143)* One module (`src/lib/locale-format.ts`) on `Intl` in the page's locale for numbers, percentages, basis points, FTE and dates; `tf`/`tp` write number placeholders in the page's format; the decimal inputs read the locale's decimal mark through `money.ts`'s parser; a scan test. See [worldwide.md](worldwide.md). Bare counts in markup done in part.
- [x] (2026-10-30) WPM-T374 **Plural rules.** *(WPM-R214, WPM-D143)* `tn(key, count, params)` picks a CLDR plural form with `Intl.PluralRules` in the page's locale; plural groups (`x.one`/`x.other`, six forms in Arabic and Welsh) for the eight "(s)" strings and the English "s" in `FrameworkRolePanel`; a test that every locale holds its language's categories. See [worldwide.md](worldwide.md). Other count strings remain.
