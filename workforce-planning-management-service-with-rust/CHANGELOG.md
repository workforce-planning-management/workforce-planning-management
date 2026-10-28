# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added — SFIA and SFIAplus imports, native levels and pointers (WPM-T180 to T185)

Migrations `000096` to `000100`: framework licence record (`version`, `redistribution` open/internal_only/unknown, holder, reference, check date; `rules::framework::publishable`),
`sfia_skills` and `sfia_skill_levels`, `job_level_frameworks` and `job_level_levels`, native level columns on `worker_skills` and `source_framework` on requirements, and
`sfiaplus_resources`. Tasks `import_sfia` (`file:`, `levels_file:`) and `import_sfiaplus`, each refusing a missing `licence_checked_on`. `PUT /api/workers/{pid}/skills` and
`PUT /api/role-profiles/{pid}/requirements` take `framework` + `framework_level`; reads show `mapping`, `native_description` and `source_framework_name`. New:
`GET /api/sfia/skills`, `GET /api/sfia/skills/{code}`, `POST /api/role-profiles/from-sfia`; the job-level routes read stored ladders beside the coded one; the training
plan and skill gaps carry `sfiaplus_pointers`. `GET /api/capability-frameworks` shows version, redistribution, holder and check date.

### Added — the information pack for bargaining (WPM-T176), the consultation register and its timeline (WPM-T175), collective agreements (WPM-T174), recognised bodies and bargaining units (WPM-T173), consultation rules (WPM-T172), a guard (WPM-T171)

`GET/POST /api/recognised-bodies` and `GET/POST /api/recognised-bodies/{pid}/units` (HR): a union, works council or staff forum the organization recognises, with a contact **role**, and the units it covers as rules over department, location, employment type and job title. Coverage is a count from the floor and never a list; no member or representative is held (WPM-D68). Collective agreements hang off a unit as dated, sourced, versioned reference data (`/api/bargaining-units/{pid}/agreements`); which apply to a worker is derived from the unit rules, for HR and payroll, and names no body (`/api/workers/{pid}/collective-agreements`). `GET /api/bargaining-units/{pid}/information-pack` gives a body the aggregates for its unit (counts from the floor, complementary suppression, a unit under the floor gives nothing, no name or salary); reading it is logged. `/api/consultations` is the register (a proposal as counts and dates, the body, topics, a document reference, dates and outcome; no attendee or minute) and its timeline reads `rules::consultation` (pure, GB and EU example profiles, the deployer's to confirm) with any longer period an agreement sets. `tests/no_union_membership.rs` fails the build if anything names membership or representative status.

### Added — a manager's count of team equipment (WPM-T308)

`GET /api/workers/{pid}/team-equipment-count`: how many direct reports hold equipment, from the floor, never which items or who. A panel on `/me`.

### Added — MCP audience binding (WPM-T307)

With `WPM_MCP_RESOURCE` set, `POST /mcp` refuses a bearer token whose audience is not that URL (`401`, with the metadata pointer). Set `WPM_TOKEN_AUDIENCE`
(or the Keycloak audience) to the same URL. Unset, nothing changes.

### Changed — lockfile and advisory policy (WPM-T305)

`cargo update` (26 compatible updates, including `tokio`, `redis` and `uuid`); `compose.test.yaml` starts Redis 8. `deny.toml` keeps one accepted advisory (`unsafe-libyaml`, unmaintained, via Loco) and drops eleven that no longer match anything, so they fail the build again if they return. `cargo deny check` and `cargo audit` pass.


### Changed — the wellbeing reminder stays until it is answered (WPM-T247)

`GET /api/workers/{pid}/wellbeing-prompts` no longer writes: it used to mark the one multi-dose reminder as sent while serving it, so a prefetch or retry used it up. The reminder now stays on the list until the worker answers it, and answering it marks it sent.

### Fixed — course-catalogue changes are audited (WPM-T246)

`POST /api/skills/{pid}/courses`, `DELETE /api/skill-courses/{pid}` and `PUT /api/skills/{pid}/training-hours` now record who made the change, in the same transaction as the change.

### Changed — upline, downline, reports and dotted-line read only the people they need (WPM-T244)

`GET /api/workers/{pid}/upline`, `/downline`, `/reports` and `/dotted-line` gave the same answers after reading every worker; at 200,000 workers that took 0.55 s a request. They now follow the chain, so the cost grows with the team, not the organization. `downline` and `reports` list at most `limit` people (default 5,000, at most 20,000), nearest first; the summary and `total` count everyone and `truncated` says whether the list is whole.

## [0.1.3] - 2026-10-21

One organization of 200,000 workers now answers every measured view within 2 seconds at the 95th percentile with eight connections and no failed request (WPM-R142; method and numbers in `BENCHMARKS.md`).

**Upgrade notes:**

- Database: migrations `000078` (`time_entry_weeks`, kept by a trigger on `time_entries`, backfilled from existing entries) and `000079` (a partial index on `deleted_at` for every soft-deleting table) run on start. On a large database the backfill and the indexes take time and the backfill holds writes to `time_entries`; run them in a quiet window.
- `GET /api/learning/mentorship-overview` names the newest 200 unpaired workers (`unmatched_limit`, at most 1,000) and always gives `unmatched_total`; it is no longer a complete list.
- `GET /api/org-chart` returns at most 5,000 people, roots first (`limit`, at most 20,000); `GET /api/org-chart/size` gives the total.

### Changed — every view holds 200,000 workers (WPM-T243)

Skill gaps, training demand, contingent share, programmes, capability, capability analysis, the skills matrix, training analytics, ergonomics and the retention report count in the database or read ids in batches of 5,000. Four of them failed outright at 200,000 workers (over Postgres's 65,535-parameter limit). Answers are unchanged apart from the two bounded lists above.

### Added — a weekly rollup for working time (WPM-T240)

`GET /api/workforce/working-time` reads `time_entry_weeks` to find the weeks that could breach the limit, then sums the exact window; answers are unchanged.

### Added — escalations on the CEO dashboard (WPM-T242) and CPD requirements can be corrected or withdrawn in the UI (WPM-T241)

### Changed — the workforce overview and metrics are counted by the database (WPM-T237)

`GET /api/workforce-intelligence/overview`, `/metrics` and `/insights` give the same answers; at 50,000 workers they went from about 600 ms with failed requests to 50 to 270 ms with none.

### Changed — the CPD overview counts in the database (WPM-T236)

`GET /api/cpd/overview` gives the same answers without reading every worker and entry into memory.

### Added — escalations of a breached tolerance (WPM-T235)

`POST /api/workforce-plans/{pid}/escalations` takes an amber or red dimension to level 1 (`plan_owner`); `POST /api/escalations/{pid}/raise-level` goes up to `board` then `executive`; `POST …/decide` (an administrator of the organization) closes it with a decision and a note. `GET /api/escalations?organization=` lists the open ones. The tolerances view gains `escalation` per dimension and `breaches_without_escalation`. Migration `000077`.

## [0.1.2] - 2026-10-20

This release is signed (cosign, keyless) and carries SLSA build provenance; both are checked by the release workflow before the GitHub release is created, and the commands to check them yourself are in `spec/production-readiness.md`. `workforce-planning-management-migration` is released at the same version; `workforce-planning-management-matcher` stays at 0.1.0.

**Upgrade notes** (each is detailed below):

- Database: migrations `000075` (`cpd_entries.course_ref`) and `000076` (`plan_tolerances`) run on start; both only add.
- A bearer token whose lifetime (`exp` minus `iat`) is over `WPM_MAX_TOKEN_LIFETIME_SECONDS` (default 86400) is refused. A provider that issues longer tokens needs the setting raised.
- The request log format changed: loco's `http-request` span with `http.uri` and `http.user_agent` is gone, replaced by a line with `request_id`, `method`, `route`, `status` and `latency_ms`. A log search or alert on the old fields needs updating.
- The plan forecast and cost views answer `422` when they would read more than `WPM_MAX_ORG_WORKERS` workers (default 10,000).
- Every container image used by CI, the compose files and the Stalwart image is pinned by digest.

### Added — tolerances on a workforce plan (WPM-T234)

`GET /api/workforce-plans/{pid}/tolerances` and `PUT|DELETE …/tolerances/{dimension}`: the largest deviation allowed in `time` (calendar days), `cost` and `resources` (basis points) and `risk` (roles), and a red, amber or green status for each and overall, with the deviation and its basis. An unknown deviation is `unknown`, never green. The alignment view gains `succession_plans_in_plan_departments`. Migration `000076`.

### Added — CPD requirements can be corrected; progress has a pace (WPM-T233)

`PUT|DELETE /api/cpd-requirements/{pid}`. Each requirement in `GET /api/workers/{pid}/cpd-progress` gains `pace` (`status`, `days_left`, `needed_per_30_days`, `expected_by_today`, `basis`), a straight-line comparison that says it is one.

### Added — links to the learning management system (WPM-T232)

`WPM_LMS_COURSE_URL` (a `https` template with `{course}`) turns a course URN into a link: `lms_url` on CPD entries from the learning system, training enrolments, learning-path steps, skill courses and cutover roles. Unset means `null`. Migration `000075` adds `cpd_entries.course_ref`.

### Changed — views read in pages and batches (WPM-T196, WPM-T230)

`GET /api/directory` searches, orders and pages in the database; `GET /api/capacity` reads workers in batches and adds up the supply; the plan forecast and cost views read only the departments the plan names. Answers are unchanged. The plan views answer `422` when they would read more than `WPM_MAX_ORG_WORKERS` workers (default 10,000). **Upgrade note:** a plan over 10,000 workers loses its forecast and cost until the setting is raised; measure first with `scripts/load-test.sh`. `controllers/capacity.rs` and `controllers/planning.rs` are split into directories (no route changes).

### Changed — request log (WPM-T197)

**Log format change:** loco's request logger (`http-request` span with `http.uri`, `http.user_agent`) is replaced by a line with `request_id`, `method`, `route`, `status` and `latency_ms`. A log search or alert keyed on the old fields needs updating. New `GET /api/audits/export` (JSON Lines, privileged).

### Added — token lifetime limit and least-privilege database roles (WPM-T199, WPM-T200)

A bearer token whose `exp - iat` is longer than `WPM_MAX_TOKEN_LIFETIME_SECONDS` (default 86400; `0` or `off` for none), or that has no `iat`, is `401`. **Upgrade note:** a provider that issues tokens longer than 24 hours needs the setting raised. `postgres-roles/roles.sql` creates `wpm_migrator` and `wpm_app`; the service needs nothing new to run as `wpm_app`. The test profile reads `WPM_TEST_FRESH_SCHEMA` and `WPM_TEST_TRUNCATE` (default on).

### Added — chaos testing (WPM-T229)

`tests/chaos.rs` puts a fault-injecting proxy between the app and Postgres (reset, silent, slow) and runs a seeded random monkey during writes. It checks that failures are quick, plain `5xx`s that leak nothing, that writes are all-or-nothing, and that the service heals. Seed printed; `WPM_CHAOS_SEED` replays. New CI step.

### Added — possible-duplicate check (WPM-T228)

`POST /api/workers/possible-duplicates` lists existing workers a new hire might already be, using the published `workforce-planning-management-matcher`. Read-only; names and worker numbers only; a name alone is `review`, only a shared worker number is `match`.

## [0.1.1] - 2026-10-19

First release with the fixes below: 0.1.0 still sent `X-Powered-By`, and its container image had no OCI labels. `workforce-planning-management-migration` is released at the same version.

### Removed — the unused vendored `crates/authentication-verifier`

- The build has used crates.io 0.9.1 since the switch; the directory was dead weight. `entity-ref` stays vendored. Current-state docs that said both shared crates were vendored now say what is true (history entries are unchanged).

### Added — OCI labels on the image

- The `Containerfile` takes `VERSION` and `REVISION` build arguments and sets the `org.opencontainers.image.{title,description,source,licenses,version,revision}` labels. The release workflow passes the crate version and the commit, and fails before pushing if the image's labels disagree with the crate version, the tag or the commit. The builder stage never had git, so nothing was dropped.

### Fixed — no `X-Powered-By` header

- `server.ident: ""` in every config removes loco's `X-Powered-By: loco.rs`, the one OWASP ZAP baseline finding (rule 10037). `tests/security.rs` now asserts the header is absent on every checked route (it failed before the config change).

### Added — Redis for the local test run

- `compose.test.yaml` starts Redis 7 beside Postgres under Podman (port override `TEST_REDIS_PORT`), matching CI. With `WPM_TEST_REDIS_URL` set, the Redis shared-cache tests run (5 pass, including password and TLS) instead of skipping. Docs that called `authentication-verifier` vendored, or the MSRV 1.96, are corrected.

### Changed — ready to package for crates.io

- `migration` is now `workforce-planning-management-migration` (library name still `migration`) with a README, and is publishable; the
  `migration` name on crates.io belongs to another project. `entity-ref` (0.2) carries a version next to its path; `authentication-verifier`
  now comes from crates.io (0.9.1; the vendored copy under `crates/` was identical and is no longer used by the build). The package now ships `config/` and the changelog
  beside `src/` (the service reads `config/<environment>.yaml` at start). Not yet published.

### Changed — dependencies

- `sha2` 0.10 → 0.11 and `prometheus` 0.13 → 0.14, and the lockfile brought up to date within the declared ranges (127 packages). The ZAP
  baseline action is 0.15.0. Verified: all targets compile; 562 unit tests, the 95 request tests and every enforcement binary pass against
  PostgreSQL. Not run: the Keycloak and Entra suites (they need containers).

### Changed — the crate, the binary and the databases are named `workforce-planning-management`

- The package, the `[[bin]]` and `default-run` are now `workforce-planning-management` (was `…-service`); the Rust crate path is
  `workforce_planning_management`. The default database names drop `_service`: `workforce_planning_management_development`, `_test`
  and `_demo`. An existing local database is renamed with `ALTER DATABASE workforce_planning_management_service_development RENAME TO
  workforce_planning_management_development;` (and the same for `_test` and `_demo`), or `DATABASE_URL` is set to the old name.
  The directory `workforce-planning-management-service-with-rust` is unchanged. Container releases build and push with Podman.

### Changed — the planner shows conversion plans (WPM-T164)

- Each department of `GET /api/workforce-plans/{pid}/forecast` carries `conversions`: `agreed` (approved plans to convert) and `hoped`
  (proposed), each a headcount and FTE. Aggregate, naming no one; neither changes the projected supply.

### Changed — cutover reasons as data, key-person for one organization (WPM-T118 follow-up)

- Each cutover check now also carries `details`: its reasons as `{code, params}`, in the same order as the English `reasons`, so a client can
  say them in its own language. `GET /api/workforce-intelligence/key-person` and `/named` take `?organization=` (outside the caller's scope
  is `404`).

### Added — cutover readiness (WPM-T117)

- `cutovers`, `cutover_roles`, `cutover_pools` (migration `000073`) and `GET /api/cutovers/{pid}/readiness`: trained staff per required
  role at the site, hypercare cover from the on-call rota over the window, and key-person risk in the pools the cutover depends on;
  each `ready`, `at_risk` or `unknown` with reasons, the worst overall, an unknown never ready, no one named, and the go/no-go left to a
  human. 558 unit and 95 request tests.

### Changed — cutover readiness follow-up (WPM-T117)

- Key-person risk is read at the site; a site is matched by name and aliases (`site_aliases`, migration `000074`) without case or
  punctuation; a role refuses an unknown course and a renamed one makes the check unknown; planning is for an administrator; an
  at-risk cutover opens a `cutover_at_risk` early warning. New enforcement binary `enforcement_cutover`. 561 unit and 95 request tests.

### Added — monthly oversight pack and the partner role (WPM-T116)

- `oversight_packs` (migration `000071`, append-only by trigger): `POST /api/oversight/packs` freezes the oversight status and a
  capacity view with no person in it; `GET /api/oversight/packs` and `/{pid}` return the stored payload, identical for every
  audience. New membership role `partner`: a caller who holds only that role may read packs in their scope and nothing else, and
  write nothing. Follow-up: the pack carries key-person counts and governance totals; `oversight_pack_freeze` task (idempotent per month); withdrawal of a mistaken pack (migration `000072`, a separate mark); a partner can read `/api/me/organizations/scope`; the free-text deciding body is left out; freezing and withdrawing are for an administrator. 548 unit, 94 request tests and a new enforcement binary.

### Added — early warning and time to report (WPM-T115)

- `early_warnings` (migration `000070`): a pool over-committed in the next six months, or an oversight measure in breach for the last
  complete month, opens one warning dated by the first sweep that saw it, closes when it stops, and tells the HR administrators with
  a reference-only `early_warning` notification. `POST /api/early-warnings/sweep`, `GET /api/early-warnings`, `POST
  /api/early-warnings/{pid}/acknowledge`, and the daily task `early_warning_sweep`. Time to report (the longest calendar days to
  acknowledge; null while any is open) is a fourth derived oversight measure. 546 unit and 92 request tests.

### Added — derived workforce measures for the oversight criteria (WPM-T114)

- A measure may be `derived`: it names `turnover_rate`, `contingent_share` or `overcommitted_pools` and takes that computation's unit.
  The status computes the months a sustain rule needs and returns them as `derived_values`; a month the data cannot speak for is
  `null` and is never a pass. A derived measure takes no entered reading. `GET /api/oversight/derivations` lists the definitions.
  Migration `000069` adds `derivation`, paired with `origin` by a CHECK. Single points of failure and governance hours are not
  derived (neither is dated); the candidate measures are defined in `spec/workforce-measures.md` and not built. 541 unit and 91 request tests.

### Added — oversight levels, positions, criteria, measures and readings (WPM-T113)

Migration `000068`. `POST /api/oversight/levels|positions|criteria|measures`, `PUT /api/oversight/measures/{pid}/readings`, `DELETE`
for levels, criteria and measures, and `GET /api/oversight/status?organization=`: the organization's own scale, its recorded
positions, and for the criteria of the level it is at, whether each sustain rule is met as of the last complete month. A reported
value needs an evidence note and a source document; the service never changes a level. The seed task adds a scale labelled synthetic.

### Added — the sustain rule for oversight criteria (WPM-T112)

`rules::oversight` (pure, no route yet): a measure meets its threshold, in a stated direction, for N consecutive periods; a criterion
is met when every linked measure is. A `null` or missing reading breaks the run and the verdict says which period and why; the rules
never output a level change.

### Added — key-person risk (WPM-T108)

`GET /api/workforce-intelligence/key-person`: for each skill marked critical on a role profile, the employed holders at the highest
critical minimum, their qualified backups and ready successors, and a status (`no_holder`, `single_point_of_failure`,
`covered_single_holder`, `shared`), overall and per department. Counts only. `GET .../key-person/named` gives the sole holder of each
single point of failure by worker number: HR only, audited.

### Added — the governance load register (WPM-T111)

Migration `000067`. `GET/POST /api/governance-commitments`, `PUT/DELETE /api/governance-commitments/{pid}` and
`GET /api/governance-commitments/load`: recurring commitments of a **role** (meeting, review, return, self-assessment; frequency;
minutes; people who take part), and the hours a month they absorb per role and for senior roles. An estimate, not a timesheet; no
person is recorded; scoped to the organization; every write is audited.

### Added — the conversion lever in workforce plans (WPM-T110)

A department with a shortfall and contract FTE today is offered `convert_contingent`; each forecast row carries `contract_fte_now`
(contractors and fixed-term staff apart). The funding horizon stays in the HR-only contingent view.

### Added — a mail server for the demo (WPM-T224)

`compose.yaml` runs Stalwart (`deploy/stalwart`) as a catch-all sink for `wpm.test` that refuses to relay; the demo configuration reads
`SMTP_HOST` and `SMTP_PORT` (defaults unchanged). `scripts/demo-mailbox.py` sends a test message and reads the inbox.

### Added — programme, FTE and basis on demand lines and requisitions (WPM-T140)

Migration `000066`. A demand line carries a programme, an FTE target (unknown when not stated) and a basis with an end date; a
requisition carries a programme, a basis and an expected duration; a hire carries the basis and the end date onto the worker. The forecast
projects each basis on its own terms and reports demand and supply by basis, FTE, and demand by programme; shortfall levers add a
fixed-term hire and a contractor. With no permanent staff, supply no longer waits for an attrition rate.

### Added — programme roll-up (WPM-T141)

`GET /api/workforce-intelligence/programmes` (HR only, audited): per programme and month, FTE funded, the contractor and fixed-term share,
engagements ending and funding ending. A programme under the floor is left out, a figure for one to nine people is withheld, and no total
across programmes is given.

### Added — snapshots by basis, the contingent trend and the named lists (WPM-T139, WPM-T109, WPM-T142)

Migration `000065` adds contractor and fixed-term headcount and FTE to the headcount snapshots (older rows stay unknown). The contingent
view gains a `trend` (insufficient_history until three usable dates span 60 calendar days) and `GET /api/engagements/contingent-list?list=`
names the people behind each engagement figure by worker number (HR only, audited).

### Added — contingent workforce view, engagement figures (WPM-T142)

`GET /api/workforce-intelligence/contingent` gains an `engagements` object: engagements ending in 0–30, 31–60 and 61–90 calendar days
(separate bands, never running totals), past their end with no decision, extended more than twice, and contractors engaged longer than
24 calendar months. A count of one to nine is withheld, zero is shown, and no total is given. The two thresholds are constants.

### Added — idempotency keys (WPM-T222)

A request that changes something and carries an `Idempotency-Key` runs once per caller: the response of a success is kept for one calendar
day and returned again (with `Idempotent-Replayed: true`) for the same request; the same key with a different request is `422`; a repeat
while the first runs is `409`; a failure is not kept. Migration `000064`. The OpenAPI document gains `components` (the key parameter, the
replay header, the `409` and `422` answers and schemas for the funding and contingent answers) and the key on every operation that changes something.

### Added — contingent share and funding horizon (WPM-T109, part)

`GET /api/workforce-intelligence/contingent` (HR only, audited): the share of FTE that is contractor and that is fixed-term, each
apart, and for the next twelve months the people whose funding ends in each month (and those whose funding has already ended).
Aggregate only. The department breakdown shows only when every department reaches the small-group floor; a month with some but
fewer than the floor is withheld, and no total is given, so a withheld figure cannot be worked out.

### Changed — the cache is Redis in production (WPM-T217)

`cache: kind: Redis` with `REDIS_URL` in production (optional: unset, it is in memory with a warning) and the demo; `rediss://` works (TLS added to the Redis client, the rustls provider chosen at boot); development and tests stay in memory. Only non-personal lookups
are cached (organization and course names, ESCO searches), every call fails open, and a plaintext `redis://` to another host
stops a production boot unless `WPM_ALLOW_PLAINTEXT_REDIS=1`. **Fixed:** a database host such as `127.0.0.1.example.com` was
treated as local and skipped the TLS requirement.

### Added — post funding beside the contract end (WPM-T161)

`/api/workers/{pid}/funding` (migration `000063`): the funding kind, programme and end date, compared with the contract end; HR only,
checked by the handler itself. Conversion-plan changes are now taken under a row lock, and a lost race on a second open plan answers 422.

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
