# Tasks — delivery checklist

Status legend: `[x]` done · `[~]` in progress · `[ ]` not started.
Every task traces to design (WPM-D*) and requirement (WPM-R*) ids.
Three-part rule applies: a behavioural change lands as spec edit +
code + tests in one PR.

## Phase 0 — specification

- [x] WPM-T0 Cross-cutting spec round: topic files + SDD trio, both
      edition doc scaffolds, root AGENTS.md wiring. (all WPM-D*, WPM-R*)
      — landed 2026-07-18. No code.

## Phase 1 — service skeleton & employee core (WPM-R7, WPM-R17)

- [x] WPM-T1 Scaffold `workforce-planning-management-api-with-rust`:
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
> (`workforce-planning-management-api-with-rust`, copy-adapted from
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
      breaches across recent **and planned** assignments (±28 days).
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
      days over requisitions filled in the period (mean, median, count);
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
> (`tests/enforcement.rs`) and the Keycloak test (`tests/keycloak.rs`) are
> still unrun, and the "DB-gated … not run" remarks in individual task
> entries above predate this run. `cargo clippy` is clean on every new
> file.

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
      snapshots, a window under 60 days, or any missing leavers figure
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
      `registration_status` with a 90-day window; 6 tests). Controller
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
      retrospective dates are whole days in UTC; skills merged by `/skills`
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

- [~] **Workforce transformation.** *(the plan / demand / gap / alignment machinery is WPM-T47–T50; a current-vs-target org-shape diff view is not built.)* The umbrella discipline —
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

- [x] WPM-T69 (2026-10-04) **Workforce insights.** Builds on WPM-T44.
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

- [x] WPM-T70 (2026-10-04) **Locale content routes and Sveltia CMS.**
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

- [x] WPM-T71 (2026-10-04) **Regional locale overrides; es-es; localized insights.**
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

- [x] WPM-T72 (2026-10-04) **Localized `/admin/`.** The Sveltia CMS shell is
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

- [x] WPM-T73 (2026-10-04) **Auth suites run; two Keycloak-backend bugs fixed.**
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

- [x] WPM-T74 (2026-10-05) **Employee directory.** From the SOTA scan
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

- [x] WPM-T75 (2026-10-05) **Emergency contacts and backups.** Each person
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

- [x] WPM-T76 (2026-10-05) **Directory shows who covers for someone on leave.**
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

- [x] WPM-T77 (2026-10-05) **On-call rota.** Migration
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
        (schedule as runs over `?from=&to=`, default 28 days, at most 92;
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

- [x] WPM-T78 (2026-10-05) **Directory shows who is on call.** Builds on
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

- [x] WPM-T79 (2026-10-05) **Rota notifications.** Builds on WPM-T77. Two new
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

- [x] WPM-T80 (2026-10-05) **On-call reminders.** Builds on WPM-T77/T79. Loco
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

- [x] WPM-T81 (2026-10-05) **Rota swap requests.** Builds on WPM-T77/T79.
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

- [x] WPM-T82 (2026-10-05) **Announcement feed.** From the SOTA scan (orangehrm
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

- [x] WPM-T83 (2026-10-05) **CEO dashboard.** `/ceo`: the whole picture on
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

- [x] WPM-T84 (2026-10-05) **CEO dashboard: ten tiles → six.** By request. Kept:
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

- [x] WPM-T85 (2026-10-05) **CEO dashboard follow-ups.** Builds on WPM-T83/T84.
      - **Drill-down:** every tile is a link to the page behind it (headcount,
        turnover, the trend and the insights → `/metrics`; vacancies →
        `/requisitions`; succession gaps → `/development`), locale-prefixed,
        with a visible focus ring.
      - **Period control** (30 days · 90 days · 12 months · year to date;
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
      pressed by default, and that choosing 90 days puts `range=90d` in the
      URL and reloads metrics with `from` = 89 days back. svelte-check 0,
      vitest 68/68, Playwright 26/26, build green.

- [x] WPM-T86 (2026-10-05) **Announcement extras.** Builds on WPM-T82. Migration
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

- [x] WPM-T87 (2026-10-06) **Skills gap analysis.** Builds on WPM-T20/T41/T51.
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

- [x] WPM-T88 (2026-10-06) **Training time recommendations.** Builds on WPM-T87.
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

- [x] WPM-T89 (2026-10-06) **People joining and leaving the organisation.**
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
        Playwright 28/28, build green. **Not done:** group-lead handover and
        appraisal/requisition ownership are not in the inventory (only what
        the model holds today); handover *on the last day automatically* (it is
        done by a person; a scheduled reminder would be a follow-up); a leaver's
        pending leave approvals; returning to work.

- [ ] **Employee expense claims — deferred (2026-10-05, by decision).** The one
      table-stakes gap from the SOTA scan (`.sota/last-scan.json`): three of
      four comparators ship it (`frappe/hrms` `expense_claim`, `orangehrm`
      `orangehrmClaimPlugin`, `odoo` `hr_expense`). Not started; revisit when
      expense reimbursement into payroll is wanted. Starting point: a
      `rules/expenses.rs` state machine (draft → submitted → approved/
      rejected → reimbursed) and a migration after `…000040`.
