# Changelog

Repo-level change summary. **Each subproject's `CHANGELOG.md` is the
authoritative, detailed history for that subproject** (Keep a
Changelog format); this file records repo-wide events only. Milestone
narrative lives in [NEWS.md](NEWS.md).

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

### Changed — the country picker; WPM-T382 is done (WPM-D152 question 3)

The country of an address is picked by name in the page's language and stored as its ISO 3166-1 code; an organization may add a code
ISO 3166-1 lacks (such as `XK`) as a national jurisdiction code, and the API refuses any other. **Not verified:** the 249-code list.

### Added — telephone numbers kept three ways (WPM-T382, WPM-D152 question 2)

A number is kept as typed, as ASCII digits (search, export) and as E.164 where it can be derived from a `+` or a known country, never
guessed; existing rows are backfilled; HR can find a worker by number. **Not verified:** the calling-code table (91 countries). Also
fixes the stale `spec/implementation-status.md` of the previous push (counted from files not yet committed).

### Changed — addresses follow the country's order, or are one free-text block (WPM-T382, WPM-D152 question 1)

The contact details form shows the address in the chosen country's order and with its labels for ten countries, and offers one
free-text block (a new column) for a country the table lacks. **Not verified:** the ten entries were not read against their source
(Google libaddressinput data). The stored forms of a telephone number and the country picker (D152 questions 2 and 3) follow.

### Added — example India and Japan jurisdiction profiles in the demo (WPM-T385)

The seed and `scripts/demo-data.py` load example profiles for India (a federal chain: a state under the country) and Japan beside the
UK-shaped one, every value labelled `example`, none read against its law. **Not done:** moving the other `WPM_*` defaults into profiles
(WPM-T384 remains open).

### Changed — public holidays follow the organization's weekend, and one rule kind is not Gregorian (WPM-T379)

A substitute holiday follows the weekend days in the organization's jurisdiction profile (or the example pack's own), a holiday can be a
fixed day of the tabular Islamic calendar, and a fourth example pack (Saudi Arabia, unverified) has a Friday and Saturday weekend.
**Not done:** lunisolar calendars. **Not verified:** the new pack.

### Changed — working-time limits come from jurisdiction profiles (WPM-T380)

The working-time guardrails no longer hold 17 weeks, 48 hours and 11 hours in code: each worker's limits come from their organization's
profile for where the work is done, a worker with none configured is counted as "no limit configured" (never as a pass), and the demo
seed loads an `example` profile in the UK's shape. HR records codes and append-only parameter rows through three new endpoints.
**Not done:** a screen for profiles. **Not tested:** the HR-only refusals under sign-in.

### Added — the jurisdiction parameter registry and its one resolver (WPM-T407)

The pure core of WPM-D144 in the service: a registry of five parameters, the chain of jurisdiction codes from the place of work, and
`resolve`, which says where each value came from and answers "unknown", never a number, when nothing is configured. **Not done:** no
table, endpoint or rule uses it yet.

### Changed — the product owner's decisions of 2026-10-31 are recorded (WPM-T406)

WPM-D144 (the jurisdiction profile design as written), WPM-D145, WPM-D149 and WPM-D151 now say they are confirmed; WPM-D152 records
the answers (both address forms, a telephone number kept as typed, as ASCII digits and as E.164 where it can be derived, the country
picker with deployer-added codes); CRM's one deployer-held HR token (CRM-D16) is recorded as the accepted design in
`spec/cross-app-bridges.md`. Spec only. **Not done:** nothing that depends on these decisions is built by this change.

### Added — overtime under a configured contracted day is request-tested (WPM-T405)

A request test sets `WPM_CONTRACTED_DAY_MINUTES` and checks that both the timesheet's derived overtime and the payslip's overtime line
count from the deployer's day, scaled by FTE. No code changed. **Not done:** the day together with configured month minutes. **Not
tested:** any reader of the day other than the timesheet and payroll.

### Fixed — the Pay Transparency Directive's defaults no longer claim to have been read (WPM-T404)

The five defaults that rest on Directive (EU) 2023/970 said `read`, but the Directive was only ever seen as search excerpts of its
EUR-Lex text and the Commission's FAQ (WPM-T268). They now say `unverified` in `GET /api/statutory-defaults`, the gap report and joint
assessment bases, and the `/statutory-defaults` page; each reference keeps its Article and says it came from excerpts. **Not done:** the
Directive was not fetched or read now, so the values themselves are no better checked than before. **Not tested:** the page's
"read" label, which no row uses any more.

### Added — a page that shows whose each default is (WPM-T403)

`/statutory-defaults` in the UI shows, read only, each setting's value in force, whose its default is, whether that source was read, and
whether the deployment set it. Its strings in the 12 other languages are machine drafts, unreviewed. **Not done:** changing a setting
from the page (by design); the jurisdiction shown as a translated name. **Not tested:** right to left and phone width beyond axe; against
a real service.

### Added — whose the AI Act date and the pay gap's sex category are (WPM-T402)

`GET /api/statutory-defaults` now also lists `WPM_AI_ACT_APPLIES_FROM` (the EU's, **unverified**: the date came from the Commission's
AI Act Service Desk, not the amending act) and `WPM_PAY_GAP_SEX_CATEGORY` (a starting point, no regime's), and the pay gap report is
request-tested with the deployer's own reporting tiers. **Not done:** moving the defaults into jurisdiction profiles (waits for the
D144 registry); the Digital Omnibus itself was not read. **Not tested:** the CSV and the AI register's answer with the settings changed.

### Fixed — the payroll stub's settings are tested (WPM-T401)

The payroll stub's five settings are now tested: unit tests for every valid and invalid shape, and a request test that a deployer's
yen stub calculates and refuses a worker paid in pounds. A calculation no longer says its stub was `configured` when a currency was
refused for want of its own allowance and rate and the example stayed in force. **Not done:** a real tax table; more than one stub per
deployment. **Not tested:** the overtime line and the timesheet's overtime with a configured contracted day, in a request.

### Changed — the demo data is not only GBP (WPM-T385, part)

The seed adds an organization paid in yen with a requisition and a benchmark in yen, and a benchmark and a benefit plan in Kuwaiti
dinars; `scripts/demo-data.py` adds expense claims in yen and dinars, each amount in its currency's own minor unit. **Not done:** the
example India and Japan jurisdiction profiles, which wait for the D144 registry; the yen and dinar amounts were not looked at in the UI.
**Missed, then fixed:** the documentation site's copy of the documents was not re-synced before the push, and CI's site job failed.

### Added — whose each statutory default is (WPM-T384, part)

Every `WPM_*` setting whose default is one regime's value (flexible working, resignation notice, pay information, the joint pay
assessment, retention, the payroll stub) now says whose: the EU Directive's Articles (read), Great Britain's flexible working rules
(unverified), an example, or a starting point that is no regime's, in `GET /api/statutory-defaults` and in the spec. The pay-gap
reporting tiers are configurable (`WPM_PAY_GAP_TIERS`, or `none`). **Not done:** moving the defaults into jurisdiction profiles, which
waits for the D144 registry, not yet confirmed; the female and male columns of the pay-gap page.

### Changed — the payroll stub is not one country's (WPM-T383)

The stub tax's allowance and rate, the working minutes in a month and the 7.5-hour contracted day are settings; unset, the stub is an
example in the UK's 2025-26 shape written for GBP, the calculation says so, and a payroll run that would pay anyone in another currency
is refused instead of being taxed with pounds read as another unit. **Not done:** a real tax table; one stub per deployment, so an
organization paying in two currencies cannot run payroll for both. **Not tested:** the new settings themselves.

### Changed — the workstation checklist is translatable content (WPM-T386)

The eight default ergonomic checklist items are stored by key (a new `ergonomic_items.item_key`, existing rows backfilled) with their
English text, shown in the reader's language in all 13 shipped languages (machine drafts, unreviewed), and named as one regime's example
list. **Not done:** a second regime's list.

### Changed — addresses and phones assume no one country (WPM-T382, proposed WPM-D152)

A postcode has no pattern (any text up to 20 characters), a telephone number's digits may be typed in any Unicode decimal digit set or
full width, and both are stored as typed. The world English address labels are neutral ("Postal code", "State, province or region"),
with the British words in `en-gb` and US ones in `en-us`. The address order per country, the stored form of a number and a country
picker are questions for the product owner (proposed WPM-D152, spec/worldwide.md); none was built.

### Changed — names are compared whole and sorted by the page's locale (WPM-T381)

The service's duplicate check no longer takes the first word as the given name and the last as the family name: the one display name is
folded script-aware and compared in any token order, so a family name written first, a one-word name and a name written without spaces
are compared correctly; the answer's breakdown is now `name` and `worker_number`. The UI sorts names and labels with `Intl.Collator` in
the page's locale. **Not done:** transliteration across scripts (spec/worldwide.md, WPM-T381).

### Fixed — a currency's minor unit is ISO 4217's, from one table the UI and the API share (WPM-T400, WPM-D151)

The UI's `money.ts` (WPM-T373) took the exponent from `Intl`, which gives Unicode CLDR's display digits; for 16 codes (HUF, IDR, COP,
PKR, IQD among them) those differ from the ISO 4217 minor unit the API stores, so a forint amount typed in the UI was stored at a
hundredth of its value. The service gains `rules::currency` (the ISO 4217 table, the shape of the sibling apps') and refuses a code
outside it with 422 wherever it takes a currency; the UI uses the same table, held equal by a vitest test, and `Intl` only as the
fallback for a code it does not hold. **Stored amounts that could be wrong (not migrated):** only on a deployment that ran `main` between commit 5c9286f (WPM-T373) and this
fix (T373 is in no release; v0.1.3 predates it): an expense item (`expense_items.amount_minor`, by its claim's `currency`) or a plan
budget (`workforce_plans.budget_minor`, `budget_currency`) typed in the UI in AFN, ALL, COP, HUF, IDR, IRR, KPW, LAK, LBP, MGA, MMK,
PKR, SOS, SYP or YER is 1/100 of what was meant, and in IQD 1/1000. An operator lists those rows from that window and corrects one by
hand only where the person confirms what they typed: an API client that sent ISO minor units then stored the right amount, and the
two cannot be told apart (spec/worldwide.md, WPM-T400).
**Not done:** no migration; codes already stored are not checked.

### Added — a GitHub Pages documentation site, built and not published (WPM-T392, WPM-R217, WPM-D150)

New subproject `workforce-planning-management.github.io/` (SvelteKit `adapter-static`, Svelte 5 runes, Lily pickers): the root
`index.md` (new; the repository had none), README, INSTALL, the family bridges tutorial, the spec index, `spec/worldwide.md` and the
231 role briefs in 21 locales, with a locale switcher that opens the same brief by its `.locale-peer-id`, dates in the page's locale
as the UI's `locale-format.ts` shows them, and Arabic and Urdu laid out from the right. `make github-pages` runs
`bin/make-github-pages` (`git subtree push`); CI job `site` (content copy, svelte-check, build, Playwright with axe). **Not done:** no
`github-pages` remote, no exported repository and no push (WPM-T399, the maintainer's); the exported repository's deploy workflow
has never run; the site's own words are English only; 210 of the 231 briefs are not axe-audited.

### Changed — the UI's today, week start and weekend are the reader's (WPM-T378, WPM-D149)

Today is the reader's date, not the UTC date, and the calendars' first day and weekend follow CLDR for the reader's region. Proposed
WPM-D149 records the choice (the browser, no stored setting yet). See the UI changelog for what was not done.

### Changed — right-to-left pages are laid out from the right (WPM-T377)

Logical CSS throughout the UI, date ranges written by `Intl`'s `formatRange` instead of a literal "→", arrows that point along the
page's direction, and a scan and Playwright tests in Arabic and Urdu. See the UI changelog for what was not tested.

### Fixed — a reused peer event id no longer answers with another organization's claim (WPM-T397)

The repeat check of `POST /api/capacity/peer-demand` is scoped to the event's own pool, programme and month, after the pool and scope
checks, and the unique index matches (WPM-D148). See the service changelog for what was not verified.

### Changed — `llms.json` and `llms.txt` current (WPM-T395)

The status (it said WPM-T1–T226 as of 2026-10-19), the stack and the stale descriptions are current, and every current document is
listed, among them the tutorial and the repository's special files.

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
