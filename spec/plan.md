# Plan — delivery capacity, oversight readiness and production readiness

Status: **proposed** (2026-10-09). Nothing here is built. Delivery is queued as
Phase 12 (WPM-T103–T118), Phase 13 (WPM-T123–T132, WPM-T144–T145) and Phase 14
(WPM-T133–T143) in [tasks.md](tasks.md). When the work starts, the requirements and decisions
below move into topic files (`delivery-capacity-and-oversight.md` for sections
A to D, `production-readiness.md` for section F,
`programmes-and-contingent-workforce.md` for section G), and the "next free ids" line in
[AGENTS/spec-driven-delivery.md](../AGENTS/spec-driven-delivery.md) moves on.

> Generic by design. This plan uses the public concept of escalation: a
> government or oversight body places an organization at a level of heightened
> oversight and sets exit criteria. It is not modelled on, and carries no data
> from, any real organization, government or framework. Levels, criteria and
> measures are **configuration**, never code.

## 1. Why

Organizations that deliver large change programmes with partner organizations
often fail in a recognizable pattern, and an oversight body may escalate them:

| Pattern | What the workforce data would have shown |
|---|---|
| Programmes started without checking capacity | Demand per skill pool exceeds supply for months ahead. |
| The same few teams sit on every programme | One or two constraint pools are over-committed while the others are not. |
| Partners hold the people the programme needs | Partner capacity is promised, unconfirmed or unknown. |
| Problems surface late | An over-commitment that was visible weeks before anyone reported it. |
| Growth "not matched by delivery maturity" | A rising share of contract staff in core roles, and posts on time-limited funding. |
| Over-reliance on key individuals | Critical skills held by one employed person with no backup or successor. |
| Leaders absorbed by recovery work | Recurring governance commitments that consume senior hours. |
| Exit criteria that are judgement calls | No quantified measure, so no evidence-based route to a lower level. |

WPM already holds most of the inputs: workers, FTE, employment type, skills
and proficiency, role profiles, leave, backups, succession, headcount
snapshots, workforce plans and the pulse. What it lacks is **demand from
programmes**, **partner commitments**, and a way to **track an oversight
position against measurable criteria**. This plan adds those, under the
repo's existing rules: pure core first, aggregates name no one, unknown is
never zero, and every estimate says what it rests on.

## 2. Capabilities

### A. Delivery capacity

- **WPM-R56 Skill pools.** A pool groups role profiles and skills into a
  plannable unit (for example integration engineering, test analysis, data
  migration, safety assurance, training, service management). A worker counts
  toward a pool through their role profile or declared skills at a minimum
  proficiency. Each pool has an **operations reservation**: the share of its
  FTE kept for running live services, so change capacity is not overstated.
- **WPM-R57 Programme demand.** A demand claim says how much FTE a programme
  needs from a pool in each month. The programme is referenced by `EntityRef`
  URN (the project-portfolio-management service owns programmes; WPM does not
  copy them). This delivers the roadmap's "WPM ↔ PPM bridge" at pool level.
- **WPM-R58 Partner commitments.** A partner organization (organization-service
  URN) can record capacity per pool per month as `requested`, `committed` or
  `confirmed`. Commitments are aggregate FTE and never name partner staff.
- **WPM-R59 Capacity view.** A pool × month heatmap of supply (employed FTE,
  less the operations reservation, less approved leave) against demand, with
  partner rows. It flags over-commitment and identifies the **constraint
  pool**: the one most over-committed over the horizon. A missing partner
  commitment shows as *unknown*, not as zero (WPM-D46).
- **WPM-R60 Start check and work-in-progress limit.** Before a programme's
  demand is marked `active`, the service reports whether every pool can carry
  it, which months fail, and the earliest start date that fits. An
  organization can set a limit on the number of active programmes drawing on
  the constraint pool. The result is a **suggestion with its evidence**; a
  person decides and the decision is recorded (WPM-D50). It is a capacity
  answer only; scheduling the work belongs to project-portfolio-management
  (WPM-D55).

### B. Workforce resilience

- **WPM-R61 Key-person risk.** For each critical skill (marked critical on a
  role profile), the service counts employed holders at the minimum
  proficiency, ranked backups and ready successors. One holder with no backup
  and no successor is a **single point of failure**. The aggregate (counts per
  pool and skill) names no one. The named list is visible to HR only, and
  every read is audited (WPM-D47).
- **WPM-R62 Contingent share and funding horizon.** The service reports the
  share of FTE with `employment_type` `contractor` or `fixed_term` per pool,
  each shown separately, with its trend over time from headcount snapshots
  split by basis (WPM-R84). A new optional post funding record (source
  kind `core`, `time_limited` or `external`, plus an end date) gives a
  **funding horizon** view: the FTE whose funding ends each month, per pool.
  Amounts of money are not stored; the record holds kind and end date only.
- **WPM-R63 Conversion lever.** Workforce plans (WPM-R36) gain the lever
  "convert contingent to permanent" beside hire, reskill, promote and
  redeploy. The lever is listed with its evidence: the contract FTE in that
  pool and the funding horizon.
- **WPM-R64 Governance load.** A register of recurring governance
  commitments (meeting, review, return, self-assessment) with frequency and
  estimated hours **per role**, not per person. It totals senior hours per
  month absorbed by oversight, so an organization can show which commitments
  to drop. There are no timesheets (WPM-D45).

### C. Oversight readiness (generic escalation)

- **WPM-R65 Oversight levels and position.** An organization configures a
  level scale (ordered names, for example five levels from routine to the
  highest intervention). It records its position history: the level, the
  effective date, the body that decided it (named as free text by the
  organization) and the published reason. Nothing is pre-loaded.
- **WPM-R66 Exit criteria and measures.** Each level carries exit criteria.
  Each criterion links to one or more measures, each with a threshold,
  direction and period, plus a **sustain rule** (N consecutive periods). A
  measure is either **derived** from WPM data (computed) or **reported**
  (entered with an evidence note and a named source document). The service
  shows, for each criterion, whether the sustain rule is met, by which
  measures, and over which periods. It never changes the level itself
  (WPM-D49).
- **WPM-R67 Derived workforce measures.** Measures come from three places:
  - **existing:** turnover, from the WPM-T44 metrics layer;
  - **new with A to C:** contingent share in constraint pools, single points
    of failure, over-committed pool-months, governance hours, and time to
    report (R68);
  - **candidates that need a definition first:** a sickness absence rate from
    leave records, the anonymous pulse (only within its existing disclosure
    floors), and training completion for courses marked as required.

  Each measure states its derivation, exclusions and period, and is `null`
  when the data is missing.
- **WPM-R68 Early warning.** When a pool becomes over-committed beyond its
  threshold, or a measure crosses into breach, the owner receives an in-app
  notification. The service records the date the condition first became true
  and the date it was acknowledged, so **time to report** becomes a measure
  in its own right.
- **WPM-R69 One pack for every audience.** A monthly snapshot of the oversight
  position, criteria, measures and the capacity view, frozen as of a date. The
  internal board, the oversight body and partner organizations read the same
  payload. Partners get a read-only role that sees aggregates and no named
  lists.

### D. Go-live workforce readiness

- **WPM-R70 Cutover readiness.** For a programme go-live at a site, the
  service checks the people side:
  - trained staff per required role (a completed course linked to the
    programme) against the role's headcount at that site;
  - hypercare cover, reusing the on-call rota for the cutover window;
  - key-person risk in the pools the cutover depends on.

  It reports `ready`, `at_risk` or `unknown` with reasons. Clinical or
  operational go/no-go stays a human decision outside WPM.

### E. Scheduling (withdrawn)

Critical chain buffers were proposed here as WPM-R71 and withdrawn on
2026-10-09 by [WPM-D55](design.md): critical chain, critical path and similar
scheduling capabilities live in project-portfolio-management. WPM supplies the
pool capacity and constraint pool (R59) that those schedules use.

### F. Production readiness

**Status (2026-10-10):** WPM-R72, R73, R74, R77, R78, R87 and R88 are built and
specified in [production-readiness.md](production-readiness.md). WPM-R75 (backup
and restore) and WPM-R76 (data protection impact assessment) are **superseded**
by WPM-R89 and WPM-R92 in [operations-and-governance.md](operations-and-governance.md),
which are built (WPM-D53 became WPM-D63). Where the two differ, production-readiness.md is the built behaviour (for
example, the limiter names callers by network address, not by token or
`X-Forwarded-For`).

These capabilities are prerequisites for any use beyond a demo. They extend
the existing production gates WPM-G1 and WPM-G2 in [tasks.md](tasks.md).

- **WPM-R72 Login enforcement is on by default.** `WPM_REQUIRE_AUTH` defaults
  to on. Running without it needs an explicit `WPM_REQUIRE_AUTH=0`, logs a
  warning at boot and on a timer, and shows a "sign-in not enforced" banner in
  the UI. With enforcement on and no key source or policy configured, the
  service refuses to start rather than starting open (WPM-D52).
- **WPM-R73 Security headers.** The API and the UI send:
  - `Strict-Transport-Security` (when served over TLS);
  - `Content-Security-Policy`, with nonces or hashes from the SvelteKit
    `csp` setting, `frame-ancestors 'none'` and no `unsafe-inline` scripts;
  - `X-Content-Type-Options: nosniff`, `Referrer-Policy:
    strict-origin-when-cross-origin`, a restrictive `Permissions-Policy`, and
    `Cross-Origin-Opener-Policy: same-origin`;
  - `Cache-Control: no-store` on every API response that carries personal data.

  A test asserts each header on representative routes, including error
  responses.
- **WPM-R74 Rate limiting.** Requests are limited per authenticated subject
  and per client address, with stricter limits on token, export, erasure,
  sweep and bulk routes. A refusal is `429` with `Retry-After`. Limits are
  configuration with safe defaults. Client addresses are held only in memory
  for the length of the window and are never logged in full (WPM-D54).
- **WPM-R75 Backup and restore.** A written backup plan, plus the means to
  carry it out:
  - **targets:** a recovery point objective (RPO) and recovery time
    objective (RTO) that a deployment sets, with defaults stated (for
    example RPO 15 minutes with WAL archiving, RTO 4 hours);
  - **method:** PostgreSQL base backups plus WAL archiving for
    point-in-time recovery, and a nightly logical dump for portability;
    backups are encrypted, stored off the host, and hold no plaintext
    secrets;
  - **retention:** backups expire after a stated number of calendar days
    (default 35 calendar days), so erased data does not live on in backups
    indefinitely;
  - **erasure after restore:** restoring a backup replays every erasure made
    after the backup was taken, from an erasure ledger, before the service
    accepts traffic (WPM-D53);
  - **drills:** a scripted restore into a scratch database, run on a
    schedule, that checks row counts, migrations and the erasure replay, and
    records the result;
  - **runbook:** who restores, how, and how the restore is announced.
- **WPM-R76 Data protection impact assessment (DPIA).** A DPIA for the
  processing WPM performs, as UK GDPR Article 35 expects for systematic
  evaluation of employees and special-category data (health-related
  adjustments, wellbeing, assessments). It covers the nature, scope, context
  and purposes; necessity and proportionality; the risks to people; and the
  measures already in the design (the WPM-D17–D25 thread, masking, k-floors,
  retention, erasure, audit). It is a **template completed for the demo**: a
  deployer must complete it for their own processing, with their data
  protection officer's advice and sign-off. It is reviewed whenever a
  requirement adds personal data.
- **WPM-R77 Buildable from its own repository, with automated checks.** A
  fresh clone builds and tests with no sibling directories:
  - the `entity-ref` and `authentication-verifier` crates come from a
    registry or a pinned git revision, not `../../` paths;
  - the test compose file carries its own Postgres init scripts.

  A CI workflow runs on every push and pull request: `cargo fmt`, clippy
  `-D warnings`, unit tests, database request tests against a Postgres
  service, the enforcement and Keycloak suites, svelte-check, vitest,
  Playwright, prettier, a Markdown link check, a check that durations say
  calendar or business days, and dependency audits (`cargo deny` or
  `cargo audit`, and `pnpm audit`). A failing check blocks a merge.
- **WPM-R78 Licence files.** A root `LICENSE.md` (the SPDX summary, linking
  the texts) and a `LICENSE/` directory holding `LICENSE/index.md` (with a
  `README.md` copy)
  and the full text of every licence the subprojects offer, one file each
  (MIT, Apache-2.0, BSD-3-Clause, GPL-2.0-only and GPL-3.0-only). The API
  offers all five; the UI offers MIT and Apache-2.0.

  `LICENSE/index.md` links to each text. The `license` fields in `Cargo.toml`
  and `package.json` and the files agree, and a CI check keeps them in step.

### G. Programme-linked planning and contingent workforce

Added 2026-10-09 in response to a user complaint: "It plans headcount by
department, with no link to programmes and minimal tracking of contractors and
fixed-term staff." Research against the code confirms each part:

| Complaint | What the code does today |
|---|---|
| Plans headcount by department | A plan demand line is department × optional role profile × target date × **integer headcount** (`plan_demand_lines`). The forecast groups by department and target date. FTE is not planned, even though snapshots record it. |
| No link to programmes | No plan, demand line, requisition or worker field references a programme. The only programme link is proposed (WPM-R57, pool-level demand). |
| Minimal tracking of contractors and fixed-term staff | `employment_type` is one of `permanent`, `fixed_term`, `contractor`, `intern`, and only one view uses it (a count per type in workforce intelligence). There is no expected end date, no extension history, no supplier, and no rate. The forecast's opening supply counts every employed worker alike and applies one attrition rate, so a fixed-term contract ending next month counts as supply. Snapshots are not split by type. |

Two further findings:

- **The spec and code disagree.** [domain-model.md](domain-model.md) says
  `employment_type` is `full_time | part_time | contract | intern`; the code
  (`rules/tokens.rs`) enforces `permanent | fixed_term | contractor | intern`.
  Working pattern (full-time or part-time) is already `fte_percent`.
- **Payroll includes contractors.** A payroll run calculates a payslip for
  every active or on-leave worker with a salary, whatever their
  `employment_type`, so a contractor with a recorded rate in the salary field
  gets a payslip. Contractors are usually paid against invoices, not payroll.

Improvements:

- **WPM-R79 Engagement basis and end date.** `employment_type` is the
  engagement basis and stays separate from the working pattern
  (`fte_percent`); the spec is corrected to the code's four values
  (WPM-D56). A new `engagement_ends_on` date is **required** for `fixed_term`
  and `contractor`, optional for `intern`, and refused for `permanent`. An
  **extension** is recorded as a dated entry (previous end, new end, reason),
  so the history shows how many times an engagement was extended.
- **WPM-R80 Contractor details.** For `contractor`: the supplier (an
  organization-service URN for an agency or the contractor's own company), the
  engagement route (`agency`, `own_company`, `statement_of_work`, `direct`), a
  rate with its basis (day or hour), and an optional employment-status
  assessment (outcome, date, reviewer) that a deployment can require for its
  jurisdiction. The rate is masked like salary (payroll, HR and self only).
  **Contractors are excluded from payroll runs** (WPM-D58).
- **WPM-R81 End-of-engagement reminders.** A daily task tells the manager and
  HR when a fixed-term or contractor engagement ends within a configured
  window (default 60 calendar days), once per end date. The reminder asks for
  a decision (extend, convert to permanent, or let it end) and records it.
  An engagement past its end date with no decision is flagged in the
  contingent workforce view, never silently treated as continuing.
- **WPM-R82 Programme link on plans and requisitions.** A demand line gains an
  optional programme (`EntityRef` URN, owned by project-portfolio-management),
  an **FTE** target alongside headcount, and an engagement basis (permanent,
  fixed-term or contractor, with an expected end for the last two). A
  requisition gains the same programme, basis and expected duration. The
  forecast reports by department **and** by programme, and plan levers add
  "fixed-term hire" and "engage a contractor", each with its end date.
- **WPM-R83 Posts funded by a programme.** The post funding record
  (WPM-R62) gains an optional programme URN. A programme roll-up shows, per
  programme and month: funded FTE, the share that is fixed-term or contractor,
  engagements ending, and funding ending. Programmes are linked through posts
  and plans, not through time recording (WPM-D59).
- **WPM-R84 Snapshots by basis.** Headcount snapshots record headcount and FTE
  per engagement basis as well as in total. Snapshots taken before this change
  report the split as `unknown`, not as zero.
- **WPM-R85 Basis-aware supply projection.** The forecast projects each basis
  on its own terms: fixed-term and contractor FTE leaves on its known
  `engagement_ends_on` unless an extension is recorded, and the attrition rate
  applies to permanent staff only, observed from permanent snapshots where
  there is enough history (WPM-D57). The payload names each assumption and
  shows headcount and FTE.
- **WPM-R86 Contingent workforce view.** For HR and planners, per department,
  pool and programme: headcount and FTE by basis; engagements ending in the
  next 30, 60 and 90 calendar days; engagements past their end with no
  decision; engagements extended more times than a configured threshold; and
  contractors engaged continuously for longer than a configured number of
  calendar months (conversion or status-review candidates). Aggregates name no
  one. The named lists are HR-only and audited.

**Added 2026-10-10 — track contractors and fixed-term staff properly.** The four things a
planner needs on every contingent engagement are now each a named requirement: the **contract end
date** (WPM-R79, with its extensions), the **supplier** (WPM-R80), the **funding source** with its
own end (WPM-R62, tied to the engagement by WPM-R98) and a **conversion plan** (WPM-R98):

- **WPM-R98 Funding source and conversion plan.** Every fixed-term or contractor engagement shows
  its **funding source** (kind `core`, `time_limited`, `external` or `programme`, the programme URN
  where there is one, and the funding's own end date) beside its contract end date, taken from the
  post funding record (R62, R83). An engagement whose contract outlasts its funding, or whose
  funding outlasts its contract, is flagged. A **conversion plan** records the intent for an
  engagement (`convert`, `extend`, `end` or `undecided`), a target date, the **permanent post** it
  would become (department and role profile, with that post's funding source), the reason, who
  proposed it and who approved it (never the same person), a status (`proposed`, `approved`,
  `done`, `abandoned`) and a review date. Rules in a pure core: a plan to convert needs a funded
  permanent post, or states why it has none; the decision date falls before the contract end; an
  engagement past its end with no approved plan is flagged (R81); a done conversion closes the old
  engagement and starts the permanent record without losing the history. The workforce plan's
  "convert contingent to permanent" lever (R63) reads these plans, so a planner sees which
  conversions are already agreed and which are only a hope. Employment-status rules for
  contractors, any agency conversion fee and notice duties are the deployer's to check; the
  software records the decision and does not decide them.
- **WPM-D66 A conversion is a decision someone makes and records, never an automatic outcome of
  length of service.** The software lists candidates (R86) and keeps the plan; it never converts,
  extends or ends an engagement by itself.

### H. Issues register and change requests (proposed 2026-10-10)

Requested alongside the project-control work: an **issues register** (the portfolio tool's FR-14) and
**change requests**, each with their **people impact**. They are project-control records, so they
belong to project-portfolio-management, not here (WPM-D55, WPM-D65); the requirement text is in
[ppm-issues-and-changes.md](ppm-issues-and-changes.md) (WPM-R95 issues register, WPM-R96 change
requests, WPM-R97 people impact). WPM's part is small: a **what-if capacity answer** for proposed
demand changes (WPM-T158), which depends on the capacity model in section A.

### I. Equality impact assessment for any scoring (proposed 2026-10-10)

Requested: a template for any scoring, such as succession readiness or risk of loss. The template
and the assessments completed for the scores WPM has today are written
([equality-impact/template.md](equality-impact/template.md),
[equality-impact/register.md](equality-impact/register.md)); they are first drafts that no one has
approved. What is proposed in code:

- **WPM-R99 Scoring register and assessment gate.** Every score derived about a person is listed
  in a register in code (`rules::scoring`: id, kind, inputs, the decision it informs, the
  assessment file, status). A test fails when a registered score has no assessment file with every
  section, when its status is not approved (or approved with conditions), or when its review date has
  passed. A lint test flags a new public function in `rules/` whose name says score, rank, rating,
  band, readiness or risk and that is not in the register.
- **WPM-R100 Adverse-impact check.** A run-time check that takes a **grouping the deployer
  supplies** (from equality data they hold elsewhere), computes favourable-outcome rates and the
  ratio to the best-treated group for a registered score, withholds small groups, and **stores
  nothing**. It produces the evidence for section 6 of the assessment. WPM holds no protected
  characteristic, unless a deployer enables the bounded, off-by-default monitoring data of WPM-D74
  (section M), in which case the grouping can come from it as aggregates only.
- **WPM-R101 Scores say what they are.** Every score payload and screen states its inputs,
  that it is advisory and a person decides, and which assessment covers it; rater-facing forms
  carry the guidance on what not to use (absence, leave, working pattern, adjustments).
- **WPM-D67 A score without an approved equality impact assessment does not ship, and an
  individual risk-of-loss score is not built.** Scores advise; a person decides (WPM-D28, D50).
  Department-level attrition is offered instead.

### J. Worker unions (proposed 2026-10-10)

Researched in [worker-unions.md](worker-unions.md). The finding: a workforce plan is what triggers
collective duties (consultation on redundancies and restructuring, bargaining over pay), so the tool
should model **obligations, bodies, agreements and aggregate information**, and must **not**
store who is a member or a representative (special category data, and a history of misuse).

- **WPM-R102 Recognised bodies and bargaining units** (organization-level reference data; coverage
  as a floored aggregate). **WPM-R103 Collective agreements** as reference data. **WPM-R104
  Consultation obligations calendar** (configurable thresholds, a suggestion with its evidence).
  **WPM-R105 Consultation register** (roles, not names). **WPM-R106 Information pack for
  bargaining** (aggregates only, each disclosure logged). **WPM-R107 Facility time as an
  aggregate.** **WPM-R108 Decision gates** (changes that need consultation, including new scoring).
- **WPM-D68 Union membership and representative status are never stored, and nothing may reveal
  them.** No column, flag, deduction label, time-off kind or attendee list. Check-off stays in the
  deployer's payroll process.

### K. SFIA and SFIAplus (proposed 2026-10-10)

Researched in [sfia.md](sfia.md): SFIA (seven levels of responsibility, five generic attributes,
147 skills in version 9) as a third capability framework beside the UK GDAD PCF and ESCO, and
BCS's SFIAplus training and certification detail on top. **The licence is the design**: the free
corporate licence is for internal use and forbids distribution, so WPM ships no SFIA content and
**imports** the deployer's licensed download.

- **WPM-R109 `import_sfia`** (a documented CSV contract, idempotent, versioned). **WPM-R110
  Levels of responsibility** as an imported job-level framework. **WPM-R111 SFIA skills in role
  profiles and declared skills with the native level kept.** **WPM-R112 SFIAplus development
  resources** (optional import; pointers in training recommendations and development plans).
  **WPM-R113 The licence guard and record** (version, redistribution status, holder, check date,
  attribution; nothing licensed on a public surface).
- **WPM-D69 Licensed content is imported by the licensee, never shipped.** **WPM-D70 The
  framework's own scale is kept and the mapping to 1–5 is shown** (`linear`: 1, 2, 2, 3, 4, 4, 5).

### L. Readiness for large medical and governmental organizations (assessed 2026-10-11)

The assessment is in [readiness-assessment.md](readiness-assessment.md): the verdict is **not ready**, with one
blocker found by probing the running service (any signed-in caller could read a colleague's sickness record,
applicants' details and the audit trail) and a list of gaps with evidence.

- **WPM-R114 Reads are need-to-know.** A table in code classifies every `GET` route as open to any signed-in
  caller, or limited to privileged callers (HR, payroll, service, admin, or an `hr_admin` membership), or to the
  worker's own record: the worker, their line managers up the chain, and privileged callers; the most sensitive
  worker records (health-adjacent, wellbeing, notifications) the worker and privileged callers only. A route not
  in the table fails a test, so a new route cannot be open by omission.
- **WPM-R115 The audit trail is append-only and tamper-evident.** The database refuses `UPDATE` and `DELETE` on
  `audit_logs`, and each entry carries a hash of its content and the previous entry's hash; a task verifies the
  chain and reports the first break.
- **WPM-R116 Production refuses insecure data paths.** Boot refuses when the database connection does not require
  TLS (unless an explicit, logged override is set), and the server applies a request timeout.
- **WPM-R117 Backups are encrypted.** `scripts/backup.sh` encrypts the dump with a key the operator supplies and
  the restore script decrypts it; an unencrypted backup needs an explicit flag and says so.
- **WPM-R118 Health detail is not stored.** A sick-leave request records that it is sick leave and its dates,
  never a reason (WPM-D73).
- **WPM-R119 Supply-chain integrity.** Actions pinned to commit SHAs, secret scanning, signed images with build
  provenance, base images pinned by digest.
- **WPM-R120 Evidence under load.** A repeatable load test, bounded work per request for views that load an
  organization's records, and a recorded result.
- **WPM-R121 Independent assurance.** A penetration test, an accessibility audit, a signed-off impact
  assessment, a translation review: recorded as gates, not claimed.
- **WPM-R122 Observability.** A request identifier on every response and in every log line; a test that logs hold
  no query strings or personal data; an audit export for a security monitoring system.
- **WPM-R123 Least-privilege database roles** and network guidance for the deployer.
- **WPM-D71 Need-to-know is decided by a table in code, not by omission.** A route is classified or a test fails.
- **WPM-D72 The audit trail is protected by the database, not only by convention.**
- **WPM-D73 A diagnosis has no column.** Health detail is not asked for, and where free text could hold it the
  field is refused or discouraged by design.

### M. Self-service, requests and workplace health requirements (proposed 2026-10-11)

Six capabilities requested together; the full text is in
[self-service-and-worker-requests.md](self-service-and-worker-requests.md). Two cut across earlier design
decisions (protected characteristics; health data) and are therefore **bounded exceptions, off by default**,
enabled only with a recorded lawful basis.

- **WPM-R124 Self-service updates** (home address, telephone numbers, personal e-mail, emergency contacts).
  **WPM-R125 Equality and diversity monitoring** (voluntary, self-declared, readable only by the worker,
  aggregate-only). **WPM-R126 Flexible working requests** (a person decides; the software computes the dates).
  **WPM-R127 Resignations** (logged by the worker, accepted by HR; never an automatic termination).
  **WPM-R128 Workplace health requirements** (immunizations as a cleared/not-cleared status, no clinical
  detail). **WPM-R129 My time-off** (allowances, days remaining, history; no sick-leave reason).
  **WPM-R130 The `/api/me` surface**, its self-service write allow-list, and one screen.
- **WPM-D74** equality monitoring data is a bounded exception to "no demographics". **WPM-D75** a health
  requirement records the status, never the reason. **WPM-D76** self-service writes are an explicit allow-list.
  **WPM-D77** a worker's request is decided by a person; the software computes dates and never changes a
  contract.

## 3. Design decisions (proposed)

- **WPM-D45 Capacity is planned by pool, not by person.** Supply comes from
  FTE and leave that WPM already holds. No timesheets and no per-person
  allocation percentages. Team-level planning gets most of the value without
  creating surveillance data.
- **WPM-D46 Unknown capacity is unknown.** A month with no partner commitment,
  or a pool with no declared skills, reports `unknown`, and the heatmap shows
  it. Over-commitment is never computed against an assumed zero.
- **WPM-D47 Key-person risk is about roles, not people.** The register exists
  to fund backups and succession, not to judge anyone. Aggregates carry counts
  only. The named list is HR-only, audited, and not in any pack or partner
  view.
- **WPM-D48 Oversight regimes are configuration.** No level names, criteria,
  thresholds or bodies ship in code or seed data, other than a synthetic demo
  scale clearly labelled synthetic. The product does not represent any real
  framework.
- **WPM-D49 Evidence, not verdicts.** The service reports whether a
  criterion's measures meet its sustain rule. A change of level is recorded by
  a person with the deciding body and date. Reported measures show their
  source, so derived and asserted evidence are never mixed silently.
- **WPM-D65 Issues and change requests are PPM's; WPM answers "what does this do to our people?"**
  See [ppm-issues-and-changes.md](ppm-issues-and-changes.md). WPM stores no issue, change request or
  baseline.
- **WPM-D50 Sequencing is a suggestion.** As with workforce plan levers
  (WPM-D28), the start check proposes; a person decides; the decision and its
  reason are kept.
- **WPM-D51** (withdrawn by WPM-D55; the id is not reused).
- **WPM-D52 Secure by default.** A fresh install enforces sign-in. Turning it
  off is an explicit, logged and visible act, and a missing key source or
  policy stops the service instead of opening it.
- **WPM-D53 A restore does not undo an erasure.** The erasure ledger holds
  only the erased record's pid and the erasure date, which is enough to replay
  the erasure and holds no other personal data. Backups expire, so the
  ledger's entries can expire with them.
- **WPM-D54 Rate limits keep no history.** Limits count in memory per window.
  No address or subject is stored or logged beyond the window, so the limiter
  adds no personal data.
- **WPM-D56 Engagement basis is not working pattern.** Whether someone is
  permanent, fixed-term, a contractor or an intern is one fact; how much they
  work is another (`fte_percent`). Mixing them (`full_time`, `part_time` and
  `contract` in one list) cannot describe a part-time fixed-term worker.
- **WPM-D57 A known end date beats an assumed rate.** Where an engagement has
  an end date, the forecast uses it. A uniform attrition rate applied to
  everyone hides a cliff of contracts ending together.
- **WPM-D58 Contractors are not on payroll.** Contractors are paid against
  invoices through their supplier, so payroll runs exclude them. Their rate is
  still personal and sensitive, and is masked like salary.
- **WPM-D59 Programmes link through posts and plans, not time.** A worker's
  link to a programme is the funding of their post or the plan line they fill,
  both facts HR already holds. There are no per-person time allocations, in
  keeping with WPM-D45.

## 4. Data and privacy summary

| New table | Holds | Personal data? |
|---|---|---|
| `skill_pools`, `skill_pool_members` (role profile or skill) | Pool definitions, operations reservation | No |
| `programme_demands` | Programme URN, pool, month, FTE, status | No |
| `partner_commitments` | Partner organization URN, pool, month, FTE, status | No |
| `post_fundings` | Worker, funding kind, end date | Yes (employment data; HR-only, retention-listed, in subject access export) |
| `governance_commitments` | Role, kind, frequency, hours | No |
| `oversight_levels`, `oversight_positions`, `exit_criteria`, `measures`, `measure_readings` | Configuration and readings | No (readings are aggregates) |
| `capacity_signals` | Condition, first-true date, acknowledged date | No |
| `erasure_ledger` | Erased record pid, erasure date | Pseudonymous (pid only); expires with the oldest backup |
| `workers` (new columns `engagement_ends_on`, supplier, route, rate) | End date, supplier URN, route, rate and basis | Yes (rate masked like salary; in export; erased) |
| `engagement_extensions` | Worker, previous end, new end, reason, decided by | Yes (HR-only; in export; erased) |
| `engagement_status_assessments` | Worker, outcome, date, reviewer | Yes (HR-only; in export; erased) |
| `conversion_plans` | Worker, intent, dates, post funding kind, reason, proposer, approver, status | Yes (HR-only; in export; erased; no pay figure) |
| `plan_demand_lines`, `requisitions` (new columns) | Programme URN, FTE, basis, expected duration | No |
| `headcount_snapshots` (new columns) | Headcount and FTE per basis | No (aggregate) |

Every one of these updates [domain-model.md](domain-model.md). The named
key-person list and `post_fundings` update [auth.md](auth.md) and
[audit.md](audit.md), and `post_fundings` updates
[regulatory.md](regulatory.md).

## 5. Order

Section G (Phase 14) fixes a payroll defect and a spec drift, and needs no
other section: its payroll exclusion (WPM-T136) should land as soon as
possible. Section F (production readiness, Phase 13) is independent of A to E
and should run first among the larger pieces: it gates any use beyond a demo. Within it, R77 (standalone build and
CI) comes first so every later change is checked automatically.

For sections A to D (Phase 12):

1. Spec round (topic file, requirements, decisions, model, auth, audit,
   glossary, index, `llms.txt`).
2. Pure core: pool supply, capacity arithmetic, start check, key-person
   count, sustain-rule evaluation, readiness. Each is DB-free and clock-free,
   with exhaustive unit tests.
3. Migrations and controllers: A, then B, then C, then D.
4. UI: capacity heatmap, resilience panel, oversight position and criteria,
   the monthly pack, and cutover readiness.
5. Verification and documentation.

A comes first because B's contingent share, C's derived measures and D's
readiness all read pools.

## 6. Open questions

- Should oversight readiness (C) live in WPM, or in project-portfolio-management
  with WPM supplying the workforce measures? This plan puts it in WPM because
  most of its measures are workforce measures. Moving it later would mean
  keeping only R67 and R69 here.
- Should partner organizations write their own commitments (they would need a
  partner login and role), or should the planning organization record them?
  This plan assumes the planning organization records them, with a `source`
  note.
- Should the funding horizon carry money? This plan says no (WPM-D45 spirit:
  kind and date are enough to see the cliff).
- Where do the sibling crates come from: crates.io releases, or a pinned git
  revision? Which CI host runs the checks?
- Should rate-limit counts be shared across instances (which needs a shared
  store), or held per instance?
- Who owns the DPIA for the demo: the maintainers, with a named reviewer?
- Should a contractor's rate live in WPM at all, or only the fact of the
  engagement, with rates in a procurement system?
- Which reminder window and extension threshold are sensible defaults, and
  should a jurisdiction setting supply them?

## 7. Out of scope

- Programme schedules, critical chain, critical path and similar scheduling
  techniques, budgets and earned value. These belong to
  project-portfolio-management (WPM-D55); WPM supplies capacity.
- Clinical or operational go/no-go decisions.
- Any real oversight framework, organization, government or board, as data or
  as seed.
- Individual time tracking.
