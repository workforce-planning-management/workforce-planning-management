# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added — conversion plans (WPM-T162, T163; part of T164)

What a person intends for a fixed-term, contractor or intern engagement: convert, extend, end or undecided (migration `000062`,
`/api/workers/{pid}/conversion-plans`, `/api/conversion-plans`). Proposed by the line manager or HR and approved by someone else;
one open plan at a time; no pay figure; HR-only reads; audited without the reason; exported and erased with the worker. Approval
records the engagement decision; marking a conversion done makes the worker permanent in one transaction. The end-of-engagement
reminder names the plan's status and review date, never its reason.

### Added — end-of-engagement reminders and decisions (WPM-T138)

The task `engagement_end_reminders [days_ahead:N] [as_of:YYYY-MM-DD]` (default 60 calendar days; run it daily) tells the manager
and HR once about each engagement that is ending, and about one that has ended with no decision; no rate or supplier is in the
notification. `POST /api/workers/{pid}/engagement/decision` records extend, convert or end against the end date it settles.
Migration `000061`. The OpenAPI document again lists the engagement routes of WPM-T137, which a mis-sync had dropped.

### Added — workplace health requirements, off by default (WPM-T214)

Whether a worker meets a requirement their area sets, such as a required immunization (migration `000060`). **Off unless
`WPM_HEALTH_REQUIREMENTS_BASIS` records the lawful basis you rely on.** A status and two dates, never a reason. Written only by a
token carrying `occupational_health=true`; a manager and HR see only cleared or not cleared; the detail is occupational health's.
Health data: do your own impact assessment first.

### Added — equality and diversity monitoring, off by default (WPM-T213)

Voluntary, self-declared monitoring answers (migration `000059`). **Off unless `WPM_EQUALITY_MONITORING_BASIS` records the
lawful basis you rely on**, with `WPM_EQUALITY_CATEGORIES` defining the categories (WPM ships none). Only the worker reads
their own answers; the only output is an aggregate with small groups withheld. Special-category data: do your own impact
assessment first.

### Added — flexible working requests (WPM-T211, T212)

A worker asks for a different working arrangement (`POST /api/me/flexible-working`); the software computes the decide-by
date (calendar months, default 2) and limits requests (default 2 in 12 calendar months); a line manager or HR decides
(approve, refuse with a reason from the deployer's list, or counter-propose), never the worker themself; a refusal can be
appealed once to someone else. An approved change of hours is a proposal for HR to apply; the contract is not changed.
Migration `000058`. Defaults are starting points to confirm for your jurisdiction.

### Added — resignations (WPM-T209, T210)

A worker logs an intent to resign (`POST /api/me/resignation`: a proposed last day that allows the notice, in calendar
days, default 28 and set with `WPM_RESIGNATION_NOTICE_CALENDAR_DAYS`; an optional reason from a closed list) and can
withdraw it until a person accepts. The manager is told it was logged, never why. Accepting (`POST
/api/resignations/{pid}/accept`) records the agreed last day, opens the leaver process and tells the worker; it does
not end employment. Migration `000057`.

### Added — `/api/me`: contact details, emergency contacts and time-off (WPM-T205–T208)

A worker changes their own home address, telephone numbers, personal e-mail and emergency contacts, and reads their own
allowances, days remaining and past absence. The writes are an explicit allow-list that skips the HR-only write policy
for those routes only (a valid token is still required; the handler resolves the person from it). Migration `000056`
adds `worker_contact_details`. HR and payroll read contact details at `GET /api/workers/{pid}/contact-details`
(audited). A worker also requests and cancels their own leave (WPM-T216); approving it is still a decision for someone with authority.

### Security — backups are encrypted by default (WPM-T192)

**Behaviour change.** `scripts/backup.sh` refuses to run without `WPM_BACKUP_KEY_FILE` (a file whose first line is
a passphrase of at least 32 characters) unless `WPM_BACKUP_ALLOW_PLAINTEXT=1`. It writes `wpm-….dump.enc` with a
`.hmac` tag; `scripts/restore.sh` needs the same key for a `.enc` file and checks the tag first. Backups made
before this are plaintext `.dump` files and restore as before.

### Security — production refuses a plaintext database connection and times requests out (WPM-T191)

**Behaviour change in production.** Boot refuses a `DATABASE_URL` that reaches another host without
`sslmode=require`, `verify-ca` or `verify-full` (a loopback address or a Unix socket is exempt); set
`WPM_ALLOW_PLAINTEXT_DATABASE=1` to accept the risk explicitly. A request running longer than
`WPM_REQUEST_TIMEOUT_MS` (default 30000) is answered `408`.

### Security — the audit trail is append-only and hash-chained (WPM-T189)

Migration `000055`: the database refuses `UPDATE` and `DELETE` on `audit_logs`, and each entry carries a hash of
its content and the entry before it. New: `GET /api/audits/verify`, the `verify_audit_chain` task, and the entries
now carry `chain_seq`, `prev_hash` and `entry_hash`. Existing entries are chained by the migration. Record the head
hash outside the database. Audit writes now serialise on a lock until their transaction ends.

### Changed — sick leave takes no reason (WPM-T190)

**Behaviour change, and a one-way migration.** `POST /api/workers/{pid}/leave-requests` with `kind: sick` now refuses
a `reason`: a diagnosis is health data the service has no need to hold. Migration `000054` removes the reason from
sick-leave requests already stored; the text cannot be restored. Other kinds keep their reason.

### Security — reads are need-to-know (WPM-T188)

**Behaviour change.** With sign-in on, a signed-in caller could read a colleague's sick-leave request (with its
reason), time-entry notes, every applicant's details and the audit trail. A guard now classifies every `GET`:
reference data and aggregates stay open to any signed-in caller; applicants, payroll runs, the audit trail,
succession and similar are HR, payroll, service and administrator only; a worker's personal records are for the
worker, their line managers up the chain and privileged callers; the most personal (adjustment requests, benefit
enrolments, payslips, wellbeing, notifications) for the worker and privileged callers only. The reference policy
is unchanged. A deployment whose interface showed a person data outside their need will now see `403`.

### Added — engagement end dates, extensions and contractor details (WPM-T137)

`engagement_ends_on` (the last day; required for fixed-term and contractor, optional for an intern, refused for
permanent), dated extensions, a contractor's supplier, route and rate (masked like salary) and employment-status
assessments. Migration `000053`. **Behaviour change:** creating or hiring a fixed-term or contractor worker now
needs `engagement_ends_on`, and a permanent worker is refused one. Existing rows keep no end date and are listed
by `GET /api/engagements/missing-end-date`.

### Added — engagement rules and basis-aware supply projection, not yet wired in (WPM-T135)

`rules::engagement` (end-date rules per basis, extensions, the end-of-engagement window in calendar days,
standing, flags) and `rules::planning::project_by_basis`. No endpoint uses them yet.

### Added — delivery capacity (WPM-T104–T107)

Skill pools, programme demand and partner commitments (`/api/skill-pools`, `/api/programme-demands`,
`/api/partner-commitments`), the pool × month `GET /api/capacity` view with a constraint pool, the
`POST /api/capacity/start-check` suggestion and recorded `start-decisions`, and the organization's
work-in-progress limit. A month in which a partner has made no commitment is unknown, never zero. Migration
`000052` adds six tables, none holding personal data.

### Changed — payroll runs exclude contractors (WPM-T136)

A contractor is paid against invoices, not through payroll, so `POST /api/payroll-runs/{pid}/calculate`
produces no payslip for an `employment_type` of `contractor`. Permanent, fixed-term and intern workers are
unchanged. The spec's employment types now match the code.

### Added — operations, governance and OIDC (WPM-T147–T152)

Migration 51 `erasure_ledger` and the task `replay_erasures [since:] [file:]`; `RecordKind` and
`horizon_for` in `rules::privacy` with `GET /api/retention/schedule`; the OIDC backend reads Entra's
top-level `roles` claim and, with `WPM_OIDC_SUBJECT_CLAIM=oid`, the object id as the subject; test
binaries `retention_schedule` and `oidc_entra`; `config/demo.yaml` now reads `PORT`.

### Changed — retention is per record kind

See the root changelog. The request and sweep payloads carry `horizons`, not `horizon_days`.

### Added — production readiness (WPM-T123–T128, T144)

`src/security.rs` (security headers, per-address rate limiting with a stricter class,
`GET /_posture`), `auth::startup_check` and `auth::parse_require_auth`, vendored
`crates/entity-ref` and `crates/authentication-verifier`, `postgres-init/`, `deny.toml`,
`Containerfile`, `config/demo.yaml`, a CORS allow-list in `config/production.yaml`
(`WPM_CORS_ORIGIN` required), and a new `security` test binary. New environment:
`WPM_RATE_LIMIT_PER_MINUTE`, `WPM_RATE_LIMIT_SENSITIVE_PER_MINUTE`, `WPM_TRUST_FORWARDED`,
`WPM_HSTS`, `WPM_CORS_ORIGIN`.

### Changed — BREAKING: enforcement on by default; metrics need a token

`WPM_REQUIRE_AUTH` defaults on. `/metrics.prom` is no longer a public path. Request
suites opt out through `tests/requests/mod.rs::request_open`.

### Fixed — `config/production.yaml` could not parse with `SMTP_USER` and `SMTP_PASSWORD` unset

The empty values rendered as YAML null; they are quoted now.

### Changed — durations say "calendar days" (WPM-T119)

Refusal messages (leave span, leave balance, rota and swap windows), the time-to-fill
insight and threshold note, the metrics derivation, two OpenAPI summaries, the pay
reminder task's help and error, and the joiner checklist's "30-calendar-day review held"
item now say "calendar days". Field names are unchanged.

### Changed — generic pay scale (WPM-T102)

The scale is `rules::pay_scale::national_2026_27` with id `national-2026-27`, name
"Public-sector pay scale 2026/27", framework `banded-pay`, nation `national`, and a
generic source. Allowance names lose the place name. Migration 50
(`m20261009_000050_generic_pay_scale_id`) moves every stored scale id in
`worker_pay_positions` and `role_profiles` to the new id; its `down` is a no-op.
Figures, bands, steps and behaviour are unchanged.

### Added — expense claims (WPM-T97)

Migration 49 (`expense_claims`, `expense_items`), `rules::expenses`, 12 routes under
`/api/expense-claims`, notification kinds `expense_submitted` and `expense_decided`, export and
erasure wired, and the `tests/enforcement_expenses.rs` binary (the not-your-own-claim rule).

### Added — pay positions (WPM-T96)

Migration 48 (`worker_pay_positions`), `rules::pay_position`, `/api/workers/{pid}/pay-position`, the
`pay_progression_reminders` task and the `pay_step_due` notification kind; export and erasure wired.

### Added — grades (WPM-T94)

Migration 47 (`worker_job_levels`; grade columns on `role_profiles`), `rules::grade`,
`/api/workers/{pid}/job-level` and `/api/role-profiles/{pid}/grade`; privacy export and
erasure wired.

### Added — job levels (WPM-T93)

`rules::job_levels` (Google technical levels L3–L11) and `GET /api/job-levels`, `/{id}`,
`/{id}/levels/{code}`. Reference data; no pay; unstated fields are null.

### Added — pay scales (WPM-T92)

`rules::pay_scale` (the national 2026/27 pay scale from the government's pay
circular; `locate`, `progression`) and `GET /api/pay-scales`, `/{id}`,
`/{id}/position`. Reference data, stateless, no migration.

### Added — WPM-T41–T89 (2026-10-02 → 2026-10-06)

A summary; the authoritative per-task record, with what was verified and what
was not, is [../spec/tasks.md](../spec/tasks.md).

- **Strategic planning and frameworks:** capability analysis, the shared
  metrics layer, headcount snapshots (`snapshot_headcount` task), workforce
  plans and forecast, role profiles, the UK GDAD PCF and ESCO imports, CPD,
  mobility, change initiatives, career history, aspirations, reporting lines,
  groups, organization memberships, confederations and transfers.
- **People and cover:** `GET /api/directory`; emergency contacts and backups
  (`/api/workers/{pid}/emergency-contacts|backups|cover`); the on-call rota
  (`/api/rotas`, overrides, swap requests, notifications, the `rota_reminders`
  task); announcements with department audiences, https links and read counts.
- **Skills and training:** `/api/workers/{pid}/skill-gaps`,
  `/api/workforce-intelligence/{skill-gaps,training-demand,insights}`,
  `/api/workers/{pid}/training-plan`, and the skill course catalogue.
- **Joiners and leavers:** `/api/movements` with dated checklists and the
  last-day handover (`…/handover`, `…/handover/all`, `…/handover/actions`).
- **Migrations 18–46** (explicit SQL); the retention sweep list now covers 55
  tables; subject access and erasure cover every new person-keyed table.
- **Auth:** the `keycloak` backend now compiles and verifies tokens (an
  `attrs_from_keycloak_claims` borrow error and an `InvalidAlgorithm` on every
  token were fixed, WPM-T73); the enforcement and Keycloak suites run and pass.
- **Tests:** 297 unit tests and 54 database-backed request tests.

### Added — list pagination headers (WPM-T40)

No controller emitted `X-Total-Count`/`X-Limit`/`X-Offset`, and the two
highest-traffic list endpoints capped with a hardcoded `.limit(...)`
rather than accepting `?limit=&offset=` — the family-wide contract in
`agents/share/restful.md`. Added the shared `Page`/`with_page_headers`
pagination helpers (`MAX_LIMIT` 500, `MAX_OFFSET` 10 000) to
`src/controllers/mod.rs`, ported from the sibling entity services.
`GET /api/employees` and `GET /api/benefit-plans` now accept
`?limit=&offset=`, clamp `limit` rather than reject it, bound `offset`
(`400` past `MAX_OFFSET`, SEC-G7), and stamp all three response headers;
the employee list's total reflects its `?department=`/`?status=` filter.
New DB-gated tests in `tests/requests/pagination.rs`. See
spec/tasks.md WPM-T40.

### Added — `require_ref` test coverage for Worker/Organization/Course (WPM-T37)

`src/validation.rs`'s `ref_rules` unit test only ever exercised
`EntityType::Person`, even though `controllers/hr_core.rs`,
`acquisition.rs`, `development.rs`, `payroll.rs`, and `talent.rs` all
call the shared `require_ref` helper with `EntityType::Worker`/
`::Organization`/`::Course`/`::CourseInstance` too. New
`ref_rules_wrong_type_worker_organization_and_course` test pins both
the wrong-type rejection and the matching-type acceptance for `Worker`,
`Organization`, and `Course`. See `../spec/tasks.md` WPM-T37.

### Added — declared MSRV (Rust 1.95)

- `Cargo.toml` now declares `rust-version = "1.95"`, the repository's
  **current stable minus three** floor
  (`spec/rust-msrv-n-minus-3/index.md`). Sourced from `ci/msrv.txt` and
  enforced by `scripts/ci-check.sh msrv`, which asserts the declared
  value matches that file and then compiles the crate — `--all-targets`,
  so benches and tests count — against the 1.95 toolchain. Behaviour is
  unchanged; what changes is that the floor is now a checked claim
  rather than an unstated assumption.
  *(Corrected 2026-09-06: `Cargo.toml` now declares `rust-version =
  "1.96"`, matching `ci/msrv.txt` and the repository's current **N-2**
  policy (`spec/rust-msrv-n-minus-2/index.md`) — the policy tightened
  from N-3 to N-2 after this entry was written. No behaviour change;
  `Cargo.toml`, `ci/msrv.txt`, and `scripts/ci-check.sh msrv` already
  agreed on 1.96 before this correction.)*

### Changed — loco-rs 1.0.1 (2026-08-02)

- **loco-rs 0.16 → 1.0.1**: sea-orm 1.1 → 2.0, sea-orm-migration →
  2.0, sea-query → 1.0. No feature-list changes (default feature set).
- **One raw `Statement` call site** in `src/controllers/privacy.rs`
  (the retention report's per-table soft-delete count) moves to
  `query_one_raw`.
- **`ColType::PkAuto` now generates a 64-bit primary key.** Of this
  crate's ~50 tables, exactly one (`audit_logs`) goes through loco's
  schema DSL and moves from `i32` to `i64`; every other table —
  employees, the acquisition/workforce/development/payroll/talent/
  wellbeing/pulse/notifications/learning/ergonomics/adjustments
  domains, and `event_outbox` — is created with raw SQL and stays
  `i32`, unaffected.
- A `useless_conversion` in `src/models/event_outbox.rs` and a
  pre-existing `needless_borrows_for_generic_args` in
  `src/controllers/mod.rs`, both surfaced by the same clippy run.
- No behavioural change; verified with the full DB-gated suite (20
  tests, unchanged count) against a freshly migrated Postgres 18.

### Added — reasonable adjustments (WPM-T36 / WPM-R33, 2026-07-25)

- Neurodiversity-inclusive adjustment requests (new WPM-D25): the
  **barrier** faced, its **impact** on the work, and the **change**
  that would reduce it — all required, in the requester's own words.
  **No diagnosis, condition, or medical-evidence column exists**: a
  diagnosis is never required to ask, and the schema has nowhere to
  put one. Practical suggestion categories (quieter workspace,
  written instructions, agendas in advance, flexible breaks, clear
  priorities, equipment, schedule, other).
- Lifecycle `requested → agreed|declined|withdrawn`;
  `agreed → in_place|withdrawn`; decisions carry a practical note,
  are audited, and notify the employee in-app (category + state,
  never the words). Content-tier masking: masked reads withhold the
  words; unmasked reads audited. Erasure scrubs; subject access
  includes requests verbatim; the sweep list grows to 41. No
  aggregate reporting surface exists, deliberately.

### Added — ergonomic (DSE) assessments + cognitive testing (WPM-T34/T35, 2026-07-25)

- **Ergonomics (WPM-R32, new WPM-D24)**: workstation assessments with
  the default 8-item DSE checklist (no item names a symptom — the
  workstation, never the body; a test pins it), `ok`/`issue` answers
  with equipment notes, an every-item-answered completion gate, and a
  rota-tier department issues report. Erasure scrubs and closes them;
  subject access includes them; both tables join the sweep (40).
- **Cognitive testing (WPM-R20)**: a fifth assessment category with
  standard index scales (verbal comprehension, working memory,
  processing speed, spatial reasoning, fluid reasoning). No composite
  "IQ number" exists; `selection` instruments refuse cognitive scales
  (no silent hiring use); `psychometric` batteries span them; masking,
  audited reads, and distribution-only analytics apply unchanged.

### Added — 360° notifications (WPM-T33 / WPM-R31, 2026-07-25)

- In-app notifications (new WPM-D23: in-app, reference-only,
  event-born — WPM holds no contact details, so outbound channels are
  a deployment integration, stated not simulated). Moving an
  appraisal to `collecting` notifies every rater (self included);
  `shared` notifies the subject. Bodies carry a neutral line and the
  appraisal reference — never scores or comments.
- `GET /api/employees/{pid}/notifications` (unread first,
  `$sub`-owned) + `POST /api/notifications/{pid}/read` (owner-only).
  Erasure deletes them; the subject-access export includes them.

### Added — auth activation surface (WPM-T31 / WPM-G1, 2026-07-25)

- `config/abac-policy.reference.json` — the `auth.md` personas as a
  shipped, verified policy: svc/admin everything, payroll unmasked
  read, HR write + masked read, `$sub` self-read, masked fallback.
  The enforcement binary now mounts **this file** via
  `WPM_ABAC_POLICY_FILE`, so the runbook and the verification cannot
  drift. Activation runbook added to spec `auth.md` (with the engine's
  known limits stated).
- **Fixed** two masking gaps found during verification:
  `GET …/subject-access` refused (`403`) to masked callers — a full
  export cannot be "masked"; the 360 report withholds comments
  (review-content tier) from masked callers while keeping the numeric
  aggregates (`comments_withheld` flag).
- Matrix extensions: payroll vs HR read masking, subject-access
  self/masked split, `/erase` + `/sweep` destructive gating, and
  svc-erase of an active employment still refused (the lawful basis
  holds regardless of privilege).

### Added — subject rights & retention (WPM-T30 / WPM-R30, 2026-07-25)

- `GET /api/employees/{pid}/subject-access` — everything WPM holds for
  one employee in a single audited JSON document, with exclusions
  named rather than hidden (pulse responses are structurally
  impossible to attribute; other raters' 360 content is third-party;
  upstream identity records are the deployment's coordination duty).
- `POST /api/employees/{pid}/erase` — erasure **as anonymisation**
  (new WPM-D22): identity fields scrubbed (tombstone `person:` URN),
  authored free text scrubbed, appraisals-as-subject closed,
  acknowledgements deleted, row soft-deleted; payroll/financial rows
  remain under statutory retention keyed to a pid that identifies no
  one. Refused while employment is open. Audited with counts.
- `GET /api/retention` + `POST /api/retention/sweep` — soft-deleted
  rows past the horizon are hard-deleted across all 38 soft-deleting
  tables and expired-consent candidates are scrubbed;
  `WPM_RETENTION_DAYS` defaults 365 and **floors at 30** (a zero
  horizon would turn soft-delete into hard-delete).
- `/erase` and `/sweep` are destructive-classified
  (`DESTRUCTIVE_POST_SUFFIXES` grew to 5) ⇒ `access=admin` under
  enforcement. Closes the code side of gate WPM-G2.

### Added — rater self-service for 360s (WPM-T29 / WPM-R29, 2026-07-25)

- `GET /api/employees/{pid}/appraisal-requests` — the rater's own
  pending requests (`collecting`, nominated, not yet responded) with
  subject, group, and competencies; `$sub`-owned; responding clears
  the request. Discloses only what the rater already knows.

### Added — 360° appraisals (WPM-T28 / WPM-R29, 2026-07-25)

- Multi-rater appraisals around a subject employee: declared
  competencies, nominations by group (`self | manager | peer |
  report`; self automatic; ≤ 12 raters; ≥ 3 non-self to start
  collecting), a one-way lifecycle (draft → collecting → shared;
  nominations freeze, responses close at shared), and `$sub`-owned
  once-per-rater responses (every declared competency scored 1–5).
- Rater anonymity is **procedural** (new WPM-D21): the store links a
  response to its nomination (once-per-rater + completion tracking
  need it) but no endpoint serves rater-level content — the detail
  view shows who responded, the shared-only report shows group ×
  competency count + mean and group-pooled alphabetised comments,
  with `peer`/`report` cells withheld below 3 responses (count
  included); `manager`/`self` disclose at 1 by convention. Report
  reads audited; development-facing, not a payroll input.

### Added — anonymous wellbeing pulse (WPM-T27 / WPM-R28, 2026-07-25)

- Surveys (`/api/pulse-surveys`: name, one question, active window)
  and anonymous 1–5 responses. **Anonymous by construction**
  (WPM-D20): the stored row is survey + department + score + date —
  no author column exists; the submission audit row is actor-less; no
  handle is returned to the submitter. Submitting is `$sub`-owned and
  window-gated (`422` on a closed survey).
- `GET /api/pulse-surveys/{pid}/results` — k-floored aggregate
  (k = 5, a pure-rules constant): per-department + overall cells,
  each suppressed (count withheld) or disclosed (count, distribution,
  mean); counts are responses, never respondents, and the derivation
  says so.

### Added — working-time guardrails (WPM-T26 / WPM-R27, 2026-07-25)

- `GET /api/workforce/working-time?department=&as_of=` — advisory
  Working Time Regulations signals derived entirely from data WPM
  already holds: the 17-week average of **recorded** (not merely
  approved) minutes with WPM-D16 terms and the 48-hour flag (integer
  boundary comparison), plus 11-hour rest-gap breaches across recent
  and planned shift assignments (±28 calendar days). Flags only — nothing is
  refused (new WPM-D19); visibility equals the rota's.
- `rules/working_time.rs` (pure): panic-free average/boundary/rest-gap
  arithmetic; overlaps clamp to 0, malformed intervals are skipped.

### Added — enrolment conversion in the uptake view (WPM-T25 / WPM-R26, 2026-07-25)

- `GET /api/wellbeing/uptake` rows for plan-linked rules gain
  `enrolment_conversion`: distinct acknowledgers now live-enrolled in
  the linked plan / distinct acknowledgers, with WPM-D16 terms
  (`null`, never `0`); `null` for rules with no linked plan; derived
  per request, never stored (WPM-D18); still aggregate-only.

### Added — benefits-awareness engine (WPM-T24 / WPM-R26, 2026-07-25)

- `wellbeing_entitlements` generalises with a closed `kind`
  (`health | benefit`; existing rows default `health`) and an optional
  `benefit_plan_pid` (must name a live plan; refused on a `health`
  rule). Predicate + acknowledgement vocabularies unchanged (WPM-D17).
- A plan-linked prompt carries the plan reference and goes quiet
  automatically for an employee with a live enrolment in that plan —
  derived per request from `benefit_enrollments`, never stored
  (WPM-D18); enrolment remains `POST …/benefit-enrollments`.
- `GET /api/wellbeing-entitlements?kind=` filters (unknown kind
  `422`); uptake rows carry the kind.
- Tests: kind-vocabulary pin + the DB-gated
  `benefits_awareness_round_trip` (kind gate, dead-plan 404,
  enrolment-quietens, filter, null-not-zero rate).

### Added — wellbeing health-entitlement prompts (WPM-T23 / WPM-R25, 2026-07-24)

- Migration `m20260724_000011_wellbeing`: `wellbeing_entitlements`
  (configurable rules — name, description, info URL, age band,
  department / job-title lists, dose count, active window; there is
  deliberately no column a health-status cohort could be expressed in,
  per WPM-D17) + `entitlement_acknowledgements` (one row per
  employee + entitlement, `booked | done | declined | dismissed`).
- `rules/wellbeing.rs` (pure): panic-free whole-year age arithmetic
  (leap-day birthdays included), age-band / department / job-title /
  active-window eligibility where an **unknown age fails a banded
  rule**, and the prompt machine (unacknowledged ⇒ prompt;
  `booked`/`done` on a multi-dose course ⇒ exactly one reminder;
  declining is final).
- `controllers/wellbeing.rs`: rule CRUD
  (`/api/wellbeing-entitlements`), the self-service prompt view
  (`GET /api/employees/{pid}/wellbeing-prompts`, employee-owned via
  the `$sub` ownership attrs; serving a reminder stamps it), the
  acknowledgement upsert (audited), and the HR
  `GET /api/wellbeing/uptake` view — **aggregate counts only** with
  WPM-D16 `{numerator, denominator, value}` terms; no individual
  appears and no manager view exists.
- `clients.rs`: best-effort `birth_date` lookup for `person:` refs
  (stub-first, cached, never stored in a WPM table) so age bands can
  evaluate; `prime_birth_date` for tests.
- Tests: 12 pure pins + the DB-gated `wellbeing_round_trip`
  (cohort scoping, unknown-age honesty, primed-DOB age match,
  closed response vocabulary, decline-is-final, the single reminder,
  aggregate-only uptake, audit row, soft-close). OpenAPI covers the
  five new paths (`spec_shape` extended).

### Changed — renamed to workforce planning management (`HCM` → `WPM`, 2026-07-23)

The project, its crate, its env prefix, its ABAC entity, and its
database names moved from *human capital management* / `HCM` to
**workforce planning management** / `WPM`. Compatibility shims cover
everything a running deployment cannot change atomically:

- **Env prefix.** `WPM_*` is read first, falling back to the legacy
  `HCM_*` spelling with a one-off deprecation warning naming the
  replacement (`src/compat.rs::env_var`, wired through every read in
  `auth.rs` / `streaming.rs` / `clients.rs`). This is a safety fix as
  much as a convenience: an `HCM_REQUIRE_AUTH=1` that stopped being
  read would turn **authentication off**.
- **ABAC entity** `"hcm"` → `"wpm"`. A mounted policy whose rules key on
  `entity: "hcm"` is rewritten at load and warn-logged
  (`compat::migrate_policy_entity`). Same reasoning: a stale entity
  condition fails *silently* — the rule stops matching and the decision
  falls through to the default, so a policy that used to deny could
  start allowing with nothing in the logs. Only the `entity` key is
  touched; a subject attribute whose value happens to be `"hcm"` is left
  alone.
- **Front-end storage.** `mxi.hcm.theme` / `mxi.hcm.locale` are adopted
  under the `mxi.wpm.*` keys on a returning user's next visit, so nobody
  loses their theme or language.
- **Database names** changed with no automatic migration — see
  "Upgrading across the 2026-07-23 rename" in the README for the
  `ALTER DATABASE` statements. A deployment that sets `DATABASE_URL`
  explicitly is unaffected.

Both shims are transitional and documented with their removal
condition. Pinned by 5 unit tests in `compat.rs` (prefix mapping,
entity migration in both JSON shapes, narrowness, idempotence,
malformed-policy safety) and 5 front-end tests
(`tests/unit/rename-migration.test.ts`).

### Added — talent strategy: succession, upskilling, reskilling, pipelines, apprenticeships, internships, workforce intelligence (WPM-T22, 2026-07-23)

- **Development plans** (`development_plans` + `development_plan_items`):
  `upskill` (deepen the current role — no target role) vs `reskill`
  (build toward a named different role), enforced rather than
  conventional. Items pair a catalog skill with a
  `current_level -> target_level` step (1-5, strictly increasing), a
  method, and a due date. Plans report **declared** progress (items
  marked achieved) *and* **verified** progress (declared proficiency
  actually reaching the target) — a claim never stands in for the
  outcome.
- **Talent pipelines** (`talent_pipelines` + `pipeline_members`) for
  succession / hiring / early careers / internal mobility. Stages
  `identified -> assessing -> developing -> ready -> placed`, `exited`
  from any open stage, and a deliberate `ready -> developing`
  regression so the bench cannot be overstated. Health counts the live
  pool only.
- **Early careers** (`early_career_programs` + `program_placements`):
  apprenticeships, internships, and graduate schemes. An
  apprenticeship must declare its off-the-job training hours, only an
  `active` placement accrues them, and **a placement cannot be
  completed below the minimum** — the refusal names both numbers.
  Withdrawal forces the `withdrawn` outcome, so it can never be
  counted as a conversion; conversion rate divides by *completed*
  placements only.
- **Succession planning** deepened: `risk_of_loss` and
  `vacancy_expected_on` on the plan, `PUT /api/succession-plans/{pid}`
  and `PUT /api/succession-candidates/{pid}` (readiness may go
  **down**), bench-coverage classification, and the
  single-point-of-failure rule (uncovered at criticality >= 4, or >= 3
  when the incumbent is a high flight risk).
- **Workforce intelligence** (`/api/workforce-intelligence/*`):
  `overview` (headcount, FTE, tenure buckets, spans of control),
  `capability` (declared skill coverage + gaps, plans in flight,
  assessment coverage), `succession` (bench strength + single points
  of failure), `pipelines` (funnel + early-career conversion). Every
  rate carries `{numerator, denominator, value}` and is `null` — never
  `0` — when there is nothing to divide; nothing is imputed; every
  payload names its derivation; no individual's sensitive data
  appears.
- Pure core `rules/talent.rs` (14 tests) + migration
  `m20260723_000010_talent`; DB-gated request suites
  `development_plans_track_claimed_and_verified_progress` and
  `pipelines_apprenticeships_and_intelligence`.

### Added — assessments: aptitude, personality, psychometric, selection (WPM-T21, 2026-07-23)

- **Instrument catalog** (`assessment_instruments`): a named test, its
  category, the scales it reports, its duration and validity. A scale
  outside its category is a `422` — except `psychometric`, which spans
  aptitude **and** personality by definition.
- **Sittings** (`assessments`) against a candidate or an employee,
  optionally tied to an application (a mismatched application /
  candidate pair is refused). Lifecycle
  `scheduled -> in_progress -> completed -> expired` (+ `cancelled`);
  completing requires at least one result and derives `expires_on`
  from the instrument's validity.
- **Per-scale results** (`assessment_results`): whole-number raw /
  max / percentile (0-100) with the band derived
  (`low` < 10, `below_average` < 30, `average` < 70,
  `above_average` < 90, `high` >= 90). Scores are integers — the same
  discipline as money.
- **Derived views**: per-subject profile (current reading per scale,
  the scales *not* assessed, selection-suitability mean), the hiring
  view `GET /api/applications/{pid}/assessments`, and aggregate
  analytics with band distributions but no individual score.
- **Sensitive**: the ABAC `mask` obligation is honoured on every read
  path (scale and band survive; raw scores, percentiles, and
  narratives do not) and unmasked reads of scored results are audited.
- Pure core `rules/assessment.rs` + migration
  `m20260723_000009_assessments`; DB-gated `assessment_round_trip`.

### Added — learning & development (WPM-T20, 2026-07-20)

- Skills framework (catalog + declared employee proficiency 1-5 with
  optional target), learning paths (ordered course steps +
  per-employee enrolment with honest progress from completed training
  enrolments), and mentorships (proposed->active->completed lifecycle
  + session log). Derived views: the per-department skills matrix +
  gaps, training analytics (completion ratio + cert expiry), and the
  mentorship overview (load, unmatched, stale). Migration
  `m20260720_000008_learning`.

### Added

- 2026-07-18 — WPM-T1–T17 implementation round: full Loco service
  (copy-adapted from patient-flow). 7 migrations / 25 tables, pure
  `rules/` core (lifecycle machines for employee / requisition /
  application / leave / review / payroll; leave balances; overtime;
  shift conflicts; org-chart cycle check; payslip arithmetic with
  the `net = gross − Σ deductions` persist gate and overflow
  refusal; benchmark flags), five pillar controllers (hr_core /
  acquisition / workforce / development / payroll) + audits / docs /
  metrics, offline PASETO + ABAC with `resource.person` `$sub`
  ownership and salary/payslip `mask` obligations, sensitive-read
  audits, event seam (memory/outbox), OpenAPI (57 paths) + Swagger,
  `Accepts-version` negotiation, Prometheus gauges, seed task
  (synthetic 40-employee org). 71 unit + 7 request + 1 enforcement
  tests green against Postgres 18; clippy-pedantic clean.

- 2026-07-18 — WPM-T0 specification round: cross-cutting spec
  (`../spec/`) with the five-pillar domain, SDD trio
  (requirements WPM-R1–R17, design WPM-D1–D12, tasks WPM-T*), and
  this edition's doc scaffold. No code yet.
