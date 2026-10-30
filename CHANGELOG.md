# Changelog

Repo-level change summary. **Each subproject's `CHANGELOG.md` is the
authoritative, detailed history for that subproject** (Keep a
Changelog format); this file records repo-wide events only. Milestone
narrative lives in [NEWS.md](NEWS.md).

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Added — a tutorial for the family bridges (WPM-T394)

[docs/tutorials/family-bridges.md](docs/tutorials/family-bridges.md) walks through the pool link, peer demand and supply, people
impact, the hire webhook, the overlap check and the lean worker list, and was run against a local service. The READMEs, `index.md`
files, INSTALL and NEWS are current. Running it found that a reused peer event id is answered with another claim, across
organizations (WPM-T397, open).

### Changed — agent files brought up to date (WPM-T393)

`AGENTS.md` (was "through WPM-T97"), the `AGENTS/` files and the two subproject `AGENTS.md` now give the current test counts,
the matcher and `scripts/`, the Dependabot practice, the next free ids and how a task is recorded. Every file is under 9,000 bytes.

### Fixed — the spec matches the code (WPM-T391)

`scripts/status.py` missed every dated task entry, so the generated status said 232 tasks done and the highest WPM-T267; it now
counts 314 and WPM-T396. `spec/cross-app-bridges.md` names every id it holds and describes the routes as built; `plan.md`,
`roadmap.md` and `worldwide.md` are current; every duration says calendar days. A GitHub Pages site is recorded as open (WPM-T392).

### Fixed — CI runs every service test binary (WPM-T396)

Seven of the service's test binaries never ran in CI (three persona tests and four that are not `#[ignore]`d, which neither
`cargo test --lib` nor an `--ignored` run reaches). The `api-db` job now runs them. They pass locally.

### Changed — dependencies, actions and images updated (WPM-T389)

Rust lockfile updates within range (`loco-rs` 1.2.0, `tokio` 1.54, `uuid` 1.29 and `redis` 1.8 now also the declared minimum);
the UI's `svelte` 5.57.3, `@types/node` 26.6.6, and the Lily `svelte-locale-translations` 0.4.0 and `svelte-picker-bar` 0.5.1;
`Swatinem/rust-cache` 2.9.3 (Dependabot #17); new digests for the Postgres 18, Redis 8, Rust and Debian images and Stalwart
0.16.27. The matcher, the migration crate and `entity-ref` declare the N-2 MSRV, Rust 1.97, and a new CI job checks them on it;
the service stays at 1.99 until a matcher declaring 1.97 is published (WPM-T390). **Held back:** TypeScript 7 (SvelteKit 3.1.0
does not load it) and Stalwart 1.0 (a new major, not tried). See the subproject changelogs and WPM-T389.

### Added — a lean worker list for bridges (WPM-T388)

`GET /api/workers?fields=` lets CRM's teams import ask for `pid,department,person_ref` only, and the list now audits each salary it
carries as the single read does (WPM-R216, WPM-D147). **Not verified:** CRM asking for it. See the service changelog.

### Changed — PPM can withdraw a demand figure it sent (WPM-T387)

The family contract's demand route takes `fte_centi: 0` to withdraw a claim and an optional `sent_at` so an older event cannot overwrite
a newer one (WPM-R215, WPM-D146; `spec/cross-app-bridges.md`). **Not verified:** PPM's sender against WPM. See the service changelog.

### Added — plural rules in the UI (WPM-T374)

`tn(key, count)` chooses a CLDR plural form with `Intl.PluralRules` in the page's locale. The eight "(s)" strings and the English "s" in
the skills save button are plural groups, with the forms each language needs (six in Arabic and Welsh, four in Russian, one in Chinese),
machine-translated and unreviewed. **Not done:** about 30 other strings that fill a count. See the UI changelog.

### Changed — the UI's numbers and dates are in the reader's format (WPM-T375)

Numbers, percentages, FTE and dates in the UI go through one module, `src/lib/locale-format.ts` (`Intl` in the page's locale): "30 Oct
2026" or "30.10.2026" rather than "2026-10-30", "12,5 %" in German, Bengali digits on a Bengali page. Decimal inputs on `/planning`,
`/capacity` and `/cpd` read the locale's decimal mark. The API is unchanged. **Not done:** every bare count in markup; ranges still use
"→" (WPM-T377). **Not verified:** screenshots; a live service. See the UI changelog.

### Changed — the UI's money is worldwide (WPM-T373, WPM-T376)

The UI shows and reads money with the currency's own minor unit (JPY 0, KWD 3) and the reader's number format, including a decimal
comma, and no form starts with a currency. One module, `src/lib/money.ts`, shared in behaviour with the sibling PPM UI's. The API is
unchanged. **Not done:** other numbers and percentages (WPM-T375). **Not verified:** screenshots; a live service. See the UI changelog.

### Added — the pure core names no country, and second-profile tests (WPM-T369, done in part)

The service's `cargo test --lib` now fails if `src/rules/` names a country, a region or one jurisdiction's law in code (an allow-list
with a reason per entry; a planted name is caught), and runs each rule that takes its limits as parameters with a second, differently
shaped synthetic profile. **Not done:** no rule was refactored; the working-time limits, the holiday weekend days and the payroll stub
still hard-code one regime (WPM-T380, T379, T383). See the service changelog.

### Added — the locale and convention equity check (WPM-T371)

[`spec/worldwide.md`](spec/worldwide.md) records 39 findings against the UI and the API, each with file and line: 9 right, 9 partial, 21
gaps, filed as WPM-T373 to WPM-T386. The worst: the UI shows and stores money as if every currency had two decimals (a JPY amount is a
hundredth of its value), there are no plural rules (and one string writes an English "s" into every language), and the holiday rules fix
the weekend at Saturday and Sunday. **Read-only:** no behaviour and no wording changed; nothing was run in a browser; the CMS was not
examined.

### Changed — the regulatory register is by region (WPM-T370)

[`spec/regulatory.md`](spec/regulatory.md) ends with a register by region: international standards, the EU, the UK, and nine more
regions marked *not yet read*. The DPIA, adjustments and right-to-work wording names its jurisdiction (worldwide audit rows 10 and 11).
Spec only; no law of any region was read for it.

### Changed — webhook payloads for a hire or an ending name the person by reference (WPM-T372, proposed WPM-D145)

So CRM can tell who was hired or left: `data` with `person_ref`, `organization_ref`, `effective_on` (and `from`, `to` for an ending),
nothing else. Proposed; CRM's receiver is being built and has not been run against it.

### Added — people-impact for a programme, the pull PPM makes (WPM-T361)

`GET /api/capacity/people-impact?programme=thing:<uuid>`: counts only, floored at ten, no total, each part known or unknown with its
reason. See the service changelog. Not run against PPM.

### Changed — the family contract with PPM and CRM recorded; programme demand in closed (WPM-T360)

At the product owner's direction (2026-10-30) [`spec/cross-app-bridges.md`](spec/cross-app-bridges.md) records the contract the three
apps share: a PPM plan is `thing:<plan pid>` here, each side's `peer_pool_ref` holds the other's pool pid, PPM sends demand to the
existing `POST /api/capacity/peer-demand` and pulls `GET /api/capacity/peer-supply` under the caller's token, people-impact is a pull,
and the hire and leaving handoff goes to CRM over webhooks. WPM-T360 is closed: the receive route already existed and no code changed.
**Not verified:** PPM's sender and CRM's receiver are being built by their owners and have not been run against WPM. CRM's team
import now sends one deployer-held token (CRM-R87), not the caller's own; the `worker:` against `person:` question is still open.

### Added — who may ask the capacity what-if, tested (WPM-T160; ticks WPM-T359)

`tests/enforcement_what_if.rs` runs each persona against the what-if and the capacity view: the service, HR and a planner get the same counts-only answer, an ordinary manager and a partner-only caller are refused, and another organization is not found. Mutation-checked. The audience is recorded as WPM-D142, **proposed, needs product-owner confirmation**; nothing was widened. No service code changed.

### Added — SFIA and SFIAplus, from the deployer's own licensed files (WPM-T180 to T185; T186 part)

WPM ships no SFIA or SFIAplus content. `import_sfia` (skills, and `levels_file:` for the seven levels of responsibility) and `import_sfiaplus` (development pointers; a
**provisional** file format until the licence holder agrees it) load the deployer's own CSVs, refuse a missing licence check, record each framework's version,
redistribution status, holder and check date, and keep what a newer file drops, marked retired. A role can start from SFIA skills; a declared skill or a requirement keeps
SFIA's own level, counted on WPM's 1–5 by the stated `linear` rule and shown with the framework's wording; the training plan and skill-gap views carry SFIAplus pointers
(nothing is booked or bought). T186 is open: the scoring register (T166) does not exist and the equality assessments are not made. Details: [`spec/sfia.md`](spec/sfia.md), [`spec/tasks.md`](spec/tasks.md).

### Added — changing manager, pure core (WPM-T333, part)

`rules::manager_change`: the dated "getting to know your team" item for a new report (counts, a PADR date and whether a passport exists; no
adjustment words, no text field), with tests that fail if the payload gains one. Not wired to an endpoint: it waits for the action list (WPM-T331).

### Added — specification: the electronic staff record, absence and line managers, disability and adjustments, succession, deputies and acting up (WPM-T309)

Proposed, not built: four topic files and Phase 24 (WPM-T310–T344). [`spec/staff-records-and-pay.md`](spec/staff-records-and-pay.md)
(assignments, coding, annual service, the person-centric record, statutory and mandatory learning, qualifications, the PADR,
payroll information, deductions, pensions, student loans, my payslips), [`spec/absence-and-line-managers.md`](spec/absence-and-line-managers.md)
(sickness absence without reasons, return-to-work, review points, occupational health, stress assessed in the work, the manager's
action list) and [`spec/disability-and-adjustments.md`](spec/disability-and-adjustments.md) (an adjustments passport the person owns,
absence rules that adjust, candidate adjustments, disability measures from voluntary monitoring only) and [`spec/succession-deputies-and-acting-up.md`](spec/succession-deputies-and-acting-up.md)
(succession on positions with evidence beside readiness, deputies with scoped authority, acting up with its pay, limits and fairness).

### Added — a manager's count of team equipment (WPM-T308)

`GET /api/workers/{pid}/team-equipment-count`: how many direct reports hold equipment, from the floor, never which items or who. A panel on `/me`.

### Added — MCP audience binding (WPM-T307)

With `WPM_MCP_RESOURCE` set, `POST /mcp` refuses a bearer token whose audience is not that URL (`401`, with the metadata pointer). Set `WPM_TOKEN_AUDIENCE`
(or the Keycloak audience) to the same URL. Unset, nothing changes.

### Changed — dependencies and Redis 8 (WPM-T305)

The Lily design system packages are on their latest releases (`svelte-headless` 0.6.1, `svelte-locale-translations` 0.3.0, `themes` 0.6.0), with `vite` 8.3.5; the Rust lockfile took 26 compatible updates (`tokio` 1.54.1, `redis` 1.8.0, `uuid` 1.29.0 and others). The three Dependabot changes are applied (`pnpm/action-setup` 6.1.0; Redis 8 in both compose files, and in CI, which Dependabot does not read). Eleven stale advisory ignores are gone from `deny.toml`. TypeScript 7 was tried and is not adopted: `svelte-check` and SvelteKit do not load it yet.


### Added — request log without personal data, and an audit export (WPM-R122, WPM-T197)

One log line per request with a request id and a route such as `/api/workers/{}`: no query string, header, body or caller. Loco's own logger, which wrote the whole URI, is off. `GET /api/audits/export` returns the audit trail as resumable JSON Lines for a security monitoring system, without snapshots.

### Added — rate-limit guidance, least-privilege database roles, a token lifetime limit (WPM-T198, T199, T200)

Behind several instances rate limits multiply; `deployment.md` says what to add. `postgres-roles/roles.sql` makes a migrator role and a runtime role that cannot change the schema or the audit trail, and `scripts/verify-db-roles.sh` (CI job `api-roles`) runs the request tests as that role. The service refuses a token whose lifetime exceeds `WPM_MAX_TOKEN_LIFETIME_SECONDS` (default 24 hours).

### Added — a signed, attested release image (WPM-R119, WPM-T195)

The release workflow signs the image by digest (cosign, keyless), attaches SLSA build provenance and verifies both before it creates the release; the `Containerfile`'s base images are pinned by digest. Not yet run on a runner: the next tag is its first test. Checking an image: [spec/production-readiness.md](spec/production-readiness.md).

### Added — CI hardening (WPM-R119, WPM-T194)

Every GitHub Action is pinned to a commit SHA (`scripts/check-actions-pinned.py` fails an unpinned one), and a `secrets` job runs gitleaks over the whole history; the seven reviewed findings are test fixtures, listed with reasons in `.gitleaksignore`.

### Added — worker matcher crate (WPM-R134, WPM-D85, WPM-T226)

`workforce-planning-management-matcher-with-rust/`: a pure crate that scores whether two worker records are the same person, with each field's score and what the decision rests on; no national identifiers; a differing date of birth caps a match at review. The service calls it for one read-only question, `POST /api/workers/possible-duplicates` (WPM-T228). Published to crates.io as 0.1.0 (2026-10-19). Details: [its changelog](workforce-planning-management-matcher-with-rust/CHANGELOG.md), [spec](spec/worker-matching.md).

### Added — delivery capacity API (WPM-T104–T107)

Skill pools, programme demand, partner commitments, the capacity view, the start check and recorded
start decisions. API only. See `spec/delivery-capacity.md`.

### Changed — payroll runs exclude contractors (WPM-T136)

A contractor gets no payslip; the spec's employment types now match the code (WPM-T134).

### Proposed — not built (WPM-R95–R113)

Issues register and change requests, contractor and fixed-term conversion plans, an equality impact
template for any scoring, worker unions without individual membership, and SFIA/SFIAplus imported
from the deployer's own licence.
### Added — operations, information governance, OIDC sign-in and every page in every locale (WPM-R89–R94)

Backup, restore and a CI-run restore drill with an erasure ledger; deployment, upgrade and runbook
documents; a retention schedule per record kind; an information governance pack
(`spec/governance/`); sign-in with Microsoft Entra ID, tested against an in-process Entra-compatible
OIDC provider; and all page text in the catalogue in 13 languages (AI-written, unreviewed:
`content/locales/REVIEW.md`). See `spec/operations-and-governance.md`.

### Changed — `GET /api/retention` and the sweep return `horizons` per record kind

`horizon_days` is gone from the report, the sweep response and its audit entry. Each record kind has
its own horizon (`WPM_RETENTION_<KIND>_DAYS`); `WPM_RETENTION_DAYS` still applies to any kind without
its own. New: `GET /api/retention/schedule`.

### Added — production readiness (WPM-R72–R74, R77, R78, R87, R88)

Sign-in is enforced by default; security headers on the API and UI; rate limiting; CORS
allow-list; `/metrics.prom` needs a token. The two shared crates are vendored so the
service builds from a fresh clone. CI (`.github/workflows/ci.yml`, written, not yet run),
Dependabot, a release workflow, a container demo, and license texts in `LICENSE/`.
Documents are checked: links, calendar days, license fields, and a generated
`spec/implementation-status.md`. See `spec/production-readiness.md`.

### Changed — BREAKING: sign-in enforcement is on by default

`WPM_REQUIRE_AUTH` was off unless set; it is now on unless set to `0`/`false`/`no`/`off`.
Run local development with `WPM_REQUIRE_AUTH=0`. `LOCO_ENV=production` refuses to start with
it off or with no token key source.

### Changed — licence files live in `LICENSE/`

`LICENSE.md` moved to `LICENSE/index.md` (with a `README.md` copy); links in README,
AI_STATEMENT, CITATION.cff and the agent files point there.

### Added — scheduling belongs to project-portfolio-management (WPM-D55)

Critical chain, critical path and similar scheduling capabilities live in
project-portfolio-management; WPM supplies workforce capacity. The proposed critical
chain work (WPM-R71, WPM-T120–T122) is withdrawn.

### Changed — durations say "calendar days" (WPM-T119)

Every duration in days in the spec, docs, API messages and UI strings now says "calendar
days", following `spec/calendar-days-or-business-days`.

### Changed — the pay scale is generic (WPM-T102)

The pay scale no longer names a specific health service or government. Its id is now
`national-2026-27`, and migration 50 moves stored ids to it. The figures are unchanged.
See `spec/pay-scales.md`.

### Added — calendar days or business days

`spec/calendar-days-or-business-days`: every duration in days says which kind of day.

### Added — browser language for `/` (WPM-T98)

A bare `/` with no remembered locale redirects by `navigator.languages`; see
`spec/locales-for-global-sharing-with-svelte/index.md`.

### Added — expense claims (WPM-T97)

Claims of dated, categorised items, approved by someone other than the claimant (manager or HR),
then reimbursed; the claimant's own claim is never in their decision queue; draft/submitted claims are
cancelled and free text scrubbed on erasure. The last open deferral. See `spec/expense-claims.md`.

### Added — pay position and eligibility reminders (WPM-T96)

A worker's pay band and step (person and HR only; no figure in the audit entry or the
reminder), the date they become eligible for the next step, and the daily task
`pay_progression_reminders`. See `spec/pay-scales.md`.

### Added — grades (WPM-T94)

A worker's job level (person and HR only; audited without the level; exported and erased
with the person) and a role's job level and pay band, with panels on `/me`, the worker
page and `/roles`. No level-to-band equivalence is derived. See `spec/job-levels.md`.

### Added — Google technical job levels (WPM-T93)

Reference ladder L3–L11 (`/api/job-levels`) and a `/job-levels` page; no pay, unstated
fields are null; an unofficial source, said so. See `spec/job-levels.md`.

### Added — national pay scale (WPM-T92)

The national 2026/27 pay scale from the government's pay circular, with a salary-placement and
progression lookup (`/api/pay-scales`) and a `/pay-scales` page. Stateless; see
`spec/pay-scales.md`.

### Changed — whole-repo lint pass (2026-10-06)

`cargo fmt`, `cargo clippy --all-targets -- -D warnings` (both auth backends) and
`pnpm lint` are now clean across the repository; previously only new files had been
kept clean. Mechanical: formatting, twelve `assert!(….is_empty())` test assertions,
a wildcard import in `auth/paseto.rs`, and one justified `#[allow]`. Full suites
re-run afterwards (297 unit, 54 request, enforcement, 68 vitest, 28 Playwright).

### Added — WPM-T37–T89 (2026-09-28 → 2026-10-06)

Detailed history: each subproject's `CHANGELOG.md` and
[spec/tasks.md](spec/tasks.md). Headlines:

- Strategic workforce planning, the metrics layer and insights, role profiles
  and capability frameworks (UK GDAD PCF, ESCO), career history and aspirations,
  reporting lines, groups, organization memberships and transfers.
- The employee directory, emergency contacts, backups (cover), the on-call rota
  (swaps, swap requests, reminders), announcements, and a one-screen CEO
  dashboard for an iPad (9th gen).
- Skills gap analysis, training time recommendations, joiners and leavers with a
  last-day handover.
- 17 content locales under `/<locale>/` routes with aliases, strings edited
  through Sveltia CMS.
- The Keycloak backend verified against a real Keycloak 26; two defects fixed.
- Spec: new topic files (`people-directory-and-cover`, `skills-and-training`,
  `joiners-and-leavers`, `communication-and-leadership`), requirements
  WPM-R39–R50, design decisions WPM-D29–D37; `llms.txt` and `llms.json`; a root `AGENTS.md` with `AGENTS/*` topic files.

### Changed

- `AI_STATEMENT.md` 1.1.0: authorizes AI to merge a pull request into
  `main` and to judge an already-merged version bump ready to release
  — executing `cargo publish` (service) or `npm publish` (front-end) —
  each against an explicit checklist; `GOVERNANCE.md` and
  `CONTRIBUTING.md` updated to match. Also corrects a contradiction
  inherited from the vendored template: the standing `Co-Authored-By`
  commit trailer is attribution only, not disclosure (§10).
- Made both subprojects publish-eligible: removed `"private": true`
  from the front-end's `package.json` (also fixed a stale
  copy-pasted `description` naming a different sibling project's
  domain) and corrected the service's `Cargo.toml` `repository` field,
  which pointed at the parent monorepo rather than this repository.

### Added

- Root special files per
  [spec/special-files-for-public-repos](spec/special-files-for-public-repos/index.md):
  README.md, LICENSE.md, CITATION.cff, NEWS.md, COMPARISONS.md,
  BENCHMARKS.md, INSTALL.md, CONTRIBUTING.md, RFC.md, CODEOWNERS,
  MAINTAINERS.md, CHANGELOG.md, AI_STATEMENT.md, GOVERNANCE.md,
  SECURITY.md.
