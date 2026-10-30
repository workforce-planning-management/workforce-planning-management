# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed — the API description has `GET /api/statutory-defaults` (WPM-T384)

`src/lib/api/openapi.json` and `schema.d.ts` regenerated from the service for the new route. No screen calls it yet.

### Changed — the workstation checklist in the reader's language (WPM-T386)

A default ergonomic checklist item is shown by its `item_key` as `erg.items.<key>` (`src/lib/ergonomics.ts`), in the worker panel and
the `/workforce` issues table; a custom item, or a key this client does not know, is shown as stored. The eight strings are in all 13
`-001` locales, machine-drafted and unreviewed (`content/locales/REVIEW.md`); `static/admin/config.yml` regenerated. Tests: 4 vitest and
1 Playwright test (`/de-001`).

### Changed — world English address labels; British and US ones as overrides (WPM-T382)

`en-001` says "City or locality", "State, province or region" and "Postal code" (it said "Town or city", "County or region",
"Postcode", which move to `en-gb`); `en-us` gets its first overrides, "City", "State" and "ZIP Code". **Not done:** other `en-001` terms
(`rota`, `redundancy`, `payslip`) were not reviewed; the translations of these labels were not re-read; the country box is still two
letters typed (proposed WPM-D152, question 3).

### Changed — names and labels sort by the page's locale (WPM-T381)

`compareText` in `src/lib/locale-format.ts` compares with `Intl.Collator` in the page's locale (numeric, one collator per locale) and
replaces every `localeCompare` with no locale: the org views (`orgViews.ts`), `/directory`, `/roles`, `/skill-gaps`, `RolePicker` and
`FrameworkRolePanel`. The whole display name is the key; no part is assumed. The locale-format scan now fails on a new `localeCompare`.
Tests: 3 vitest (Swedish, German and Spanish order; Cyrillic and Arabic names; numbers in labels; an invalid locale). **Not tested:**
the sorted lists in a browser (the Playwright specs over them pass, but none sorts in a non-English locale).

### Fixed — money uses the ISO 4217 minor unit, not CLDR's display digits (WPM-T400, WPM-D151)

`src/lib/money.ts` takes a currency's exponent from `MINOR_UNITS`, a copy of the service's `src/rules/currency.rs` table (a vitest test
reads the Rust file and fails when they differ), and falls back to `Intl` only for a code it does not hold. `Intl`'s
`maximumFractionDigits` is CLDR's, which shows 0 places for 16 codes where ISO 4217 has 2 (3 for IQD), so 1,500 forint typed was sent
as 1,500 fillér. Formatting now sets the minimum and maximum fraction digits to the ISO exponent, so a forint amount shows two places.
The header says the family's three `money.ts` files share the rule. Tests: 5 vitest (the tables equal; the 16 codes; the `XAU`
fallback; HUF, IDR, COP, JPY, KWD and CLF round trips in `en-001` and `de-001`, and IQD); forcing `Intl` first fails 3. **Stored amounts that could be wrong (not migrated):** only on a deployment that ran `main` between commit 5c9286f (WPM-T373) and this
fix (T373 is in no release; v0.1.3 predates it): an expense item (`expense_items.amount_minor`, by its claim's `currency`) or a plan
budget (`workforce_plans.budget_minor`, `budget_currency`) typed in the UI in AFN, ALL, COP, HUF, IDR, IRR, KPW, LAK, LBP, MGA, MMK,
PKR, SOS, SYP or YER is 1/100 of what was meant, and in IQD 1/1000. An operator lists those rows from that window and corrects one by
hand only where the person confirms what they typed: an API client that sent ISO minor units then stored the right amount, and the
two cannot be told apart (spec/worldwide.md, WPM-T400).
**Not tested:** no Playwright spec in one of the 16 currencies (the yen and KWD specs pass unchanged); a runtime without `Intl` currency data.

### Changed — today, week start and weekend are the reader's (WPM-T378, WPM-D149)

`src/lib/reader-calendar.ts`: `todayIso()` is today in the reader's time zone (it was the UTC date on five pages and in the calendar),
`isoDateIn()` places a shift on its day in that zone, and `weekInfo()` gives a calendar's first day and weekend from CLDR
(`Intl.Locale#getWeekInfo`) for the page locale's country, else the browser's first country, else the page locale's world data; a
fallback table for browsers without it is checked against the runtime's CLDR data. `CalendarPanel` now marks the weekend and starts the
week on the reader's day (the page locale plus `-u-fw-…`, so formatting is unchanged). API values stay ISO dates. Tests:
`tests/unit/reader-calendar.test.ts` (UTC+14 and UTC−11 zones, a Saturday week start and a Friday–Saturday weekend for Egypt on `ar-001`,
Saudi Arabia, Afghanistan, Iran, India, the table, a scan for the UTC "today") and five Playwright cases in `calendar.spec.ts` (today in
`Pacific/Kiritimati` and UTC; the week for `en-001`, a browser in `ar-EG` and one in plain `ar`). **Not done:** no stored user or
organization time zone or region. **Not tested:** a browser without `getWeekInfo`.

### Changed — right to left (WPM-T377)

Every physical left/right style in `src/` is logical: table cells `text-align: start`, the bed and home cards and calendar events
`border-inline-start` / `padding-inline-start`. A range of dates is `formatDateRange` / `formatDateTimeRange` in
`src/lib/locale-format.ts` (`Intl.DateTimeFormat#formatRange`: "1–30 Sept 2026"; an unknown end is "…"), and an arrow meaning
"becomes" is `arrow()` ("←" on an Arabic or Urdu page); the 33 literal "→" in 22 files are gone, three of them ranges that showed raw ISO
dates. The share button's arrow is mirrored under `dir="rtl"`. The payroll list's period column gets a header (`columns.period`, each
locale's existing `ceo.period` word): without one its sort button had no accessible name, which the new axe test found.
Tests: `tests/unit/right-to-left.test.ts` (a scan that fails on a new physical side or literal arrow, with a reasoned allowlist; range
and arrow unit tests) and `tests/e2e/right-to-left.spec.ts` (ar-001 and ur-001: a table, a period, the menu, axe; an en-001 control).
**Not done:** the Lily theme stylesheets are not scanned; the trend chart's time axis is left to right on every page. **Not tested:**
other pages in Arabic or Urdu, dark mode or phone width in right to left, a native reader.

### Changed — dependencies (WPM-T389)

`pnpm update`: `svelte` 5.57.3 and `@types/node` 26.6.6. The Lily `svelte-locale-translations` 0.3.0 to 0.4.0 and
`svelte-picker-bar` 0.4.0 to 0.5.1 (exact pins, as before); lint, svelte-check, vitest, Playwright and the build stay green.
**Held back:** TypeScript 7.0.2, tried again: `svelte-kit sync` fails to load it under SvelteKit 3.1.0, so it stays on 6.0.3.
`pnpm audit` reports one low advisory with no patched release (`deepmerge` 4.3.1 through `@sveltejs/vite-plugin-svelte`, a build
tool); the existing peer warning (`openapi-typescript` 7.13.0 wants TypeScript 5) is unchanged.

### Added — plural rules (WPM-T374)

`src/lib/i18n.svelte.ts` adds `tn(key, count, params)`: a *plural group* in the catalogue is an object keyed by CLDR category
(`{"one": "…", "other": "…"}` in English; `zero`, `one`, `two`, `few`, `many`, `other` in Arabic and Welsh), and the form is chosen with
`Intl.PluralRules` in the page's locale, falling back to the group's `other` before English. Nine strings converted: the eight "(s)" strings
and the save button in `FrameworkRolePanel`, which appended an English "s" in every language. Every `-001` locale holds exactly the
categories its language uses (ar, cy 6; ru 4; es, fr, pt 3; bn, de, en, hi, ur 2; id, zh 1), machine-translated and unreviewed. The CMS
offers all six categories for a plural group. Tests: `tests/unit/plurals.test.ts` (selection in en, ar, ru, cy, zh, id; categories per
locale; no "(s)" left), the parity test made plural-aware, five Playwright tests (the CPD requirement count in en, ru and ar). **Not done:**
other strings that fill a count still have one form. **Not verified:** no native speaker has read a form; decimal counts; screenshots.

### Changed — numbers, percentages and dates in the page's locale (WPM-T375)

One module, `src/lib/locale-format.ts`, on `Intl` with `i18n.locale`: numbers, fixed decimals, percentages, basis points and centi-FTE
(the point placed exactly), signed changes, dates (a civil date the same day everywhere, a timestamp in the reader's zone), date-times,
times and months; typed numbers through `money.ts`'s `parseAmount`. `format.ts` writes through it; `tf`/`tp` write a number placeholder
in the page's format; `DataTable` formats a number or ISO date in a column with no type of its own (sorting and export unchanged). About
85 raw ISO dates, 17 `toFixed` and hand-made percentages, two `toLocale*` calls in the browser's locale and the English "now" in
`CareerHistory` are gone. Eight decimal inputs (attrition, on-cost and line FTE on `/planning`, reservation and claim FTE on `/capacity`,
three amounts on `/cpd`) are `inputmode="decimal"` text read in the page's format and refused before a request when they are not a number
(`common.badNumber`, `common.badNumberAtLeast`: two new keys in the 13 full locales, machine-translated and unreviewed). Tests:
`tests/unit/locale-format.test.ts` (unit tests and a scan of `src/` for new hand-rolled formatting), three new Playwright tests (German
percentages sent as basis points, a payroll period in German and in Bengali digits), 26 assertions updated from ISO text.
**Not done:** bare counts in markup outside the headline tiles are not all converted and the scan cannot see them; multi-line `tf`/`tp`
calls are outside the scan; ranges still use "→" (WPM-T377); `type="date"` inputs follow the browser. **Not verified:** screenshots were
not looked at; no live service; Arabic-Indic digits typed on an Arabic page.

### Changed — money uses the currency's own minor unit and the reader's number format (WPM-T373, WPM-T376)

One formatter and parser, `src/lib/money.ts`, shared in behaviour with the sibling project-portfolio-management UI's `money.ts`
(its commit 956d9ed): the exponent comes from the currency through `Intl` (JPY 0, EUR 2, KWD 3, CLF 4), not a fixed 100, and the
format is the page's locale. Typed amounts (an expense item, a pay-scale salary, a plan budget) are read in the page's number format,
so "12,50" in German is 12.50; text that is not a number is refused before a request. The expense-claim and plan forms start with no
currency, and a claim or a budget cannot be sent without a three-letter code. The API contract (minor units plus a code) is unchanged.
Four new keys in the 13 full locales, machine-translated and unreviewed. Tests: `tests/unit/money.test.ts` (16), five new
Playwright tests (yen pay scale in German, yen and euro expense claims with axe on the panel, a KWD plan budget, no budget sends no
currency). **Not done:** percentages and other numbers (WPM-T375); the percentage inputs on `/planning` are still `type="number"`.
**Not verified:** screenshots were not looked at; no live service was used (stubbed API only); the CMS was not opened.

### Added — SFIA in roles, declared skills, job levels and the training plan (WPM-T183 to T185)

`/roles`: start from SFIA skills (with the category chosen) and the requirement table shows the native level, the mapping and, under it, the framework's wording. A
**Declared skills** panel on `/me` and the worker page declares a skill on WPM's 1–5 or SFIA's 1–7. `/job-levels` and the level pickers choose among ladders and show
attributes. The training plan lists SFIAplus pointers (source named, never booked, links only for `http(s)`). The `/roles` profile select no longer shows blank after seeding.
Strings in the 12 full locales (machine-drafted, unreviewed).

### Added — bargaining, consultations and decision gates (WPM-T178, part)

`/bargaining` (recognised bodies with a contact role, units as rules, coverage from the floor, the information pack with each reading logged by the service, facility time per body per period), `/consultations` (the register, a timeline that is a suggestion with its evidence, begin and close) and `/decision-gates` (set a gate, check a change: advice, never a block). The AI register shows the decision-gate pointer after a feature is recorded. Nothing names a member or a representative (WPM-D68). `bargaining.*`, `consultations.*`, `gates.*` in the 12 full locales (machine-drafted, unreviewed). Stubbed, axe and live specs.

### Added — team equipment on `/me` (WPM-T308)

A manager sees how many of their direct reports hold equipment, from the floor; `teamEquipment.*` in the 13 locales.

### Changed — Lily packages and vite (WPM-T305)

`@lilydesignsystem/svelte-headless` 0.6.1, `svelte-locale-translations` 0.3.0 and `themes` 0.6.0; `vite` 8.3.5. `svelte-check`, the unit tests and the Playwright suite pass unchanged. TypeScript stays on 6.0.3: version 7 breaks `svelte-check` and the SvelteKit loader. `pnpm audit` reports no known vulnerabilities.


### Security — the content manager's script is pinned and its page has a policy (WPM-T249)

`/admin` loaded Sveltia CMS from unpkg at whatever version was newest, with no integrity hash and no content security policy. It now loads 0.230.0 with its `sha384` hash, and the page sends a policy that lets only that script and its own hashed inline script run. Updating the CMS means changing the version and the hash in `src/lib/server/admin-page.ts`.

### Security — the API proxy forwards an allow-list (WPM-T248)

The proxy used to copy every browser header except four, so with no session a browser's own `authorization` and `x-forwarded-for` reached the API. It now sends only `accept`, `content-type`, `if-none-match`, a plain idempotency key, the version, its own bearer, and the caller's address as the server sees it. A path with an empty, `.` or `..` segment is refused with `400`.

### Added — My details (WPM-T215)

`/me/details`, linked from `/me`: contact details, emergency contacts, time off and leave requests, flexible working requests, a resignation, equality monitoring and workplace health requirements, all the signed-in person's own. 86 new strings in the 13 locales (machine drafts, unreviewed).

### Added — data tables fall back to the Lily label pack (WPM-T227)

The catalogue's `dataTable.*` strings stay first; `@lilydesignsystem/svelte-locale-translations` fills only the grid labels the catalogue does not name, for the page's language, loaded one chunk per language. Unreviewed machine drafts, like the catalogue.

### Added — a demo with a guided tour, the Lily link and search pickers, and two fixes (WPM-T118)

Signing in as `example@example.com`, where `WPM_DEMO_API_URL` names a separate service with fictitious data, opens `/demo`: a tour of
34 pages with a banner (Step n of 34, Previous, Next); `scripts/demo-data.py` fills the demo service. `@lilydesignsystem/*` upgraded; the
header gains the link picker (every page) and the search picker (the directory). Fixed: `/signin/sso` threw a 500; the pickers\' hover
tooltips moved a button from under the pointer; the role picker on `/me` re-asked for an empty list without end.

### Added — a demo with a guided tour, the Lily link and search pickers, and two fixes (WPM-T118)

Signing in as `example@example.com`, where `WPM_DEMO_API_URL` names a separate service with fictitious data, opens `/demo`: a tour of
34 pages with a banner (Step n of 34, Previous, Next); `scripts/demo-data.py` fills the demo service. `@lilydesignsystem/*` upgraded; the
header gains the link picker (every page) and the search picker (the directory). Fixed: `/signin/sso` threw a 500; the pickers' hover
tooltips moved a button from under the pointer; the role picker on `/me` re-asked for an empty list without end.

### Changed — the T118 pages can now write, speak the reader's language and read one organization (WPM-T118 follow-up)

Forms to plan a cutover (with its roles and pools, and remove), create skill pools, add members and set a programme's claim, add a level,
record the position, add criteria and measures and enter readings (a missing value carries no evidence); `/warnings` lists the early warnings
with acknowledge and "Look now"; the cutover reasons are said from the service's data (`details`) in the reader's language, in English
only for a code the page does not know; the "how this is worked out" texts are in the catalogue, with the service's own English beneath; the
key-person list reads the chosen organization; the month count no longer needs a plural. 88 more strings in the 13 `-001` locales; 13 more
Playwright specs (stubbed) and four live ones; refusals (403) are said in words and any other failure shows the service's message.

### Added — capacity, resilience, oversight with the monthly pack, and cutover readiness (WPM-T118)

`/capacity` (the heatmap: supply / demand in FTE by pool and month, each cell a mark and a word as well as a colour, an unknown month dashed
and never a zero), `/resilience` (critical skills with holders, backups and successors; single points of failure; governance hours),
`/oversight` (the level, the exit criteria and why a run stopped; the monthly packs: freeze, open, withdraw with a reason) and `/cutover`
(three checks and the worst, each a mark and a word). Ten client functions and path-map entries; 77 strings and 28 vocabulary values in the
13 `-001` locales, including 20 tokens earlier tasks had left without a value. 21 Playwright specs with light/dark and desktop/phone
screenshots (dark sets the saved theme).

### Fixed — the proxy forwarded to the wrong path (found on the first run against a real service)

`/api/proxy/x` went to `<WPM_API_URL>/x`, but the service serves under `/api/`, so every call was a 404 against a real service (the stubbed suite
never reaches the proxy). It now forwards to `<WPM_API_URL>/api/x`; `WPM_API_URL` is the service root. A unit test pins the target.

### Added — requisitions can be created in the UI, and a live suite

A form on `/requisitions` (organization, department, title, headcount, basis, a duration for a basis that ends, programme); with no
membership to choose from, the organization URN is typed (also on the plan form). `tests/live/` runs seven specs against a real service
(`WPM_LIVE_API=1`, `playwright.live.config.ts`). The Playwright `expect` timeout is 20 s: pages sometimes sat idle for several seconds at the
start of a parallel run.

### Added — contingent workforce, engagement, and plans by basis and programme (WPM-T143)

`/contingent` (HR): the share of FTE for contractors and fixed-term staff, engagements ending in 30, 60 and 90 calendar days, past their end,
extended and long-running, with "Show who" (worker numbers; the read is audited), funding ending by month, the trend and the programme
roll-up. A count of one to nine is shown as "withheld", never 0. A worker's page gains an engagement panel (basis, end, extensions,
decision, funding beside the contract end with a flag, a contractor's masked rate). The plan line form takes basis, FTE and programme, and the
forecast shows demand and supply by basis, the FTE gap, demand by programme and the levers in words (they were raw tokens). Requisition cards
show basis, duration and programme. 62 strings and 64 vocabulary values in the 13 `-001` locales; the `untranslated` test, which was failing,
passes. Phone layout: wide tables scroll inside their card.

### Added — an Idempotency-Key on every request that changes something (WPM-T221)

`api()` sends `Idempotency-Key` (a UUID) with POST, PUT, PATCH and DELETE, new for each action, reusable through `idempotencyKey` when
repeating one; the proxy forwards it only when it is a plain 16 to 64 character token. The API acts on it from WPM-T222 (a repeat of the same request returns the first answer).

### Changed — every page and component takes its text from the catalogue (WPM-T153, T154)

710 new keys (`pages.*`, `comp.*`, `titles.*`, `values.*`) in all 13 languages; new helpers `tf()` (with
values), `tv()` (a closed-vocabulary token) and `pageTitle()`; tests that fail on literal visible text,
on a locale repeating English, and on a server token with no translation. The translations are
AI-written and unreviewed (`content/locales/REVIEW.md`).

### Added — security headers and a content security policy (WPM-T127)

`src/lib/security-headers.ts` applied in `hooks.server.ts`, and `csp` in `vite.config.ts`.
A Playwright spec checks the headers and that the policy blocks nothing on two public
pages.

### Changed — durations say "calendar days" (WPM-T119)

`pr.horizon`, `common.days`, `metrics.days`, the time-to-fill insight, the CEO period
buttons and the pay step eligibility line say "calendar days" in the 13 `-001` locales
(AI-written, unreviewed).

### Changed — generic pay scale (WPM-T102)

Comments, test stubs and the pay-scales e2e spec use the generic scale id
`national-2026-27` and its generic name and source.

### Removed — bare-language forwarding (WPM-T100)

`/en/…` is no longer 301-redirected to `/en-001/…`; only a full content code is a locale prefix.
`LOCALE_ALIASES` is now `LANGUAGE_LOCALE` (tag matching only).

### Added — locale directory rule (WPM-T99)

Spec and test: every `content/locales/` directory is `<language>-<region>`; a bare language is never a
directory. (No two-letter directories existed, so none were deleted.)

### Added — bare `/` follows the browser's language (WPM-T98)

`localeFromNavigator` and a root-layout redirect: with no remembered locale, `/` goes to the locale
route for `navigator.languages` (`cy_GB` → `/cy-001/`), falling back to the server's `Accept-Language`
pick (and a `<noscript>` refresh). Other unprefixed paths are unchanged.

### Added — expense claims

`ExpenseClaims` panel on `/me` and the worker page, `/expenses` decision queue, `ExpenseClaimView`;
strings in the 12 `-001` locales. Fixed while building: the queue stayed open on a claim after the
filter changed; "Add item" wiped what was typed for the next item.

### Added — pay-position panel

`PayPosition` on `/me` and the worker page; strings in the 12 `-001` locales.

### Fixed — visual review of the new pages (2026-10-06)

Looked at `/pay-scales`, `/job-levels`, `/roles` and `/me` in light and dark at desktop, iPad and
phone widths. Fixed: band `8a` shown as `8A`; whole-pound pay shown as `£26,300.00` (now `£26,300`);
the lookup and grade forms' unaligned labels; the job-levels table unreadable on a phone (now scrolls
in its box); the level select overflowing the Grade panel on a phone. Added `PW_PORT` to the
Playwright config.

### Added — grade panels

`JobLevel` on `/me` and the worker page, `RoleGrade` on `/roles`; strings in the 12 `-001` locales.

### Added — job levels page

`/job-levels`: the L3–L11 table with a dash for anything the source does not state, in
the 12 `-001` locales (AI-written, unreviewed).

### Added — pay scales page

`/pay-scales`: the national pay scale table, allowances and a salary-placement
lookup, in all 12 `-001` locales (AI-written, unreviewed).

### Added — sitemap generator

`/sitemap.xml` (every public page in every locale with `hreflang` alternates and
`x-default`) and `/robots.txt`, generated per request from the locale list and the
public pages; optional `WPM_PUBLIC_URL` for a proxied deployment. The list of public
pages is now shared with the sign-in gate (`src/lib/publicPages.ts`).

### Added — joiners and leavers

`/movements`: who is joining and leaving, each with a dated checklist, how far
along it is, and how many items are late. A leaver's record also lists
everything they still hold on their last day — people they manage, groups,
on-call seats, shifts, mentorships, tasks and access — to reassign one by one or
all at once to a successor (access is revoked, not handed over), with an audit
trail of what went where.

### Added — skill gaps and training time

"My skill gaps" (on `/me` and a worker's page): what your role, your own
targets and — for you alone — your aspirations ask for against what you have
declared, ranked; a skill you have not declared shows as "Not declared", never
as a gap of some size. Under it, a **training plan**: the courses and hours
that would close each gap and when you would finish at a weekly pace. `/skill-gaps`
shows the workforce's gaps (counts only), the training hours they add up to,
and a catalogue editor for the courses behind each skill.

### Added — announcement audiences, links and read receipts; CEO period control

An announcement can be aimed at one department, carry up to three https
links, and be marked read; editors see how many have read it, never who.
On `/ceo`, a period control (30 calendar days, 90 calendar days, 12 months, year to date) and
every tile links to the page behind it.

### Added — CEO dashboard

`/ceo`: six tiles — headcount, turnover, vacancies, succession gaps, a
six-month headcount line, and the top insights — on one screen, no
scrolling, sized for an iPad (9th gen) at 2160 × 1620 pixels.

### Added — announcement feed; rota swap requests

`/announcements`: company and organization news, pinned first then newest,
plain text only. Editors (`hr_admin`, `org_admin`) post, retire and see
scheduled or expired posts; the latest three also show on the home page. On
`/rota` and "My on-call": ask a colleague to take your on-call days, and
accept, decline or cancel requests.

### Added — on-call rota

`/rota`: a rotation of workers where the duty passes to the next member
every few calendar days. Shows who is on call now, the next four weeks with why
(their turn, covering for someone away, or a swap) and days-on-call per
member; a member on approved leave is skipped to the next available
person. HR creates rotas and records swaps. `/me` shows your own on-call
stretches.

### Added — employee directory; emergency contacts and backups

`/directory`: a searchable list of who works where (employed workers in
the organizations you can read; no pay, dates or person reference). On
`/me` and `/workers/{pid}`: **Emergency contacts** (only you and HR can see
them) and **Backups** — the colleagues who cover when you are out, with
who is covering today.

### Added — locale content routes, `/en/` aliases, Sveltia CMS; metrics insights

UI strings now live in `content/locales/<locale>/ui.json` (16 locales,
`<language>-001` plus `en-gb`, `en-us`, `de-de`), edited through Sveltia
CMS at `/admin/` (`static/admin/config.yml`, generated by `pnpm
cms-config`). Pages are served under their locale: `/en-001/workers`,
`/cy-001/workers` (a `reroute` hook, `src/hooks.ts`; the route tree is
unchanged). `/en/…` and `/cy/…` redirect (301) to `/en-001/…` and
`/cy-001/…`; an unprefixed URL redirects (302) to the remembered or
`Accept-Language` locale. **Breaking for bookmarks and the
`mxi.wpm.locale` value:** locale codes are now `en-001`, not `en`/`en_US`
(old spellings are still accepted on read). `/metrics` also shows the
service's derived insights.

### Changed — PickerBar now uses Lily's default themes and text sizes, not app-specific lists

`spec/lily-design-system-svelte-with-picker-bar/index.md` calls for
"all Lily default themes (not any application-specific custom
themes)" and "all Lily default text sizes." Removed the hand-maintained
`THEMES` array (45 slugs — coincidentally the same *set* as Lily's own
`DEFAULT_THEMES`, but ordered with the UK/US government themes
interleaved alphabetically rather than grouped at the end) and
`SIZES`/`SIZE_LABELS` (a custom 4-step `small`/`medium`/`large`/
`x-large` scale). `PickerBar` now gets no `themes`/`sizes` prop at
all, so it uses its own `DEFAULT_THEMES` (45, alphabetical, UK/US
government themes grouped at the end, matching the spec) and
`DEFAULT_SIZES` (the 7-step `largest`…`smallest` scale) directly.

`src/app.css`'s `[data-text-size]` rules rewritten for the new 7-step
scale (was 3 rules for `small`/`large`/`x-large`; `normal`, formerly
`medium`, stays unscaled).

### Changed — chrome consolidated onto Lily `PickerBar`; locale picker restored

The top-of-page chrome (theme, text-size, share pickers) previously
wired `ThemePicker`/`TextSizePicker`/`SharePicker` individually in
`+layout.svelte`. Replaced with the single Lily
`lily-design-system-svelte-picker-bar` component, which composes all
four pickers — theme, locale, text-size, share — as one row.

This restores a locale picker to the chrome, which this project never
had: `PickerBar` bundles all four unconditionally with no way to omit
one. Wired with `applyDir={false}` and `onChange={(code) =>
i18n.set(code)}`, since this app's own `i18n.svelte.ts` store already
reflects `dir` onto `<html>` (the `lang` effect now also normalises
`i18n.locale` from `en_US` to `en-US` for BCP 47 validity, which it
previously did not need to do). The locale label reuses the existing
`chrome.language` string key, already present in every locale's
catalog rather than newly added.

`SHARE_TARGETS` gained an `email` (`mailto:`) target and now orders
Email / LinkedIn / Reddit / Bluesky / Mastodon (previously
LinkedIn / Mastodon / Bluesky / Reddit); the Mastodon target now
points at `mastodonshare.com` instead of `mastodon.social/share` (a
generic sharer rather than one specific instance).

New dependencies: `lily-design-system-svelte-locale-picker`,
`lily-design-system-svelte-picker-bar` (both `file:` deps on the
sibling Lily checkout, same pattern as the existing four).

Also added `en_US` ("English (United States)") to
`src/lib/i18n.svelte.ts`'s `LOCALES`/`LOCALE_LABELS`/`STRINGS` (a
duplicate of `en`'s copy) and taught `normaliseLocale` to match a
region variant like `en_US`/`en-US` exactly before falling back to
stripping to its primary subtag (previously `en_US` would have
silently collapsed to `en`, since only the primary subtag was ever
checked). `tests/unit/wpm.test.ts` gained a test pinning that
normalisation directly; the existing locale-parity test already
iterates `LOCALES` dynamically, so it covers `en_US` without change.

### Added — centralised null-not-zero format helpers (WPM-T39)

The training-completion rate, uptake rate, enrolment conversion,
learning-path progress, pulse mean, and appraisal competency mean were
each formatted with their own inline null-check, duplicated across
`routes/learning`, `routes/wellbeing`, and `routes/employees/[pid]` —
no `$lib/format.ts` counterpart to the CMS front-end's. New
`src/lib/format.ts` (`percent`/`workings`/`percentWithWorkings`/
`percentOf`/`mean`) plus a shared `Ratio` type in `src/lib/api/types.ts`
replacing three duplicated anonymous type literals in `src/lib/api/wpm.ts`.

Found and fixed two real inconsistencies while centralising, not just a
refactor: `routes/learning`'s completion-rate rendering always showed
the raw numerator/denominator even when the percentage was dashed
(`"— (0/5)"`), while `routes/wellbeing`'s uptake/conversion rates
collapsed to a bare `"—"` — `percentWithWorkings` now picks the latter
convention consistently; and `routes/wellbeing`'s pulse-mean label
silently rendered the literal string `"undefined"` when a cell's mean
was absent, now a proper `"—"` fallback via `mean()`.

`routes/mentorship` needed no change (it has no ratio/mean rendering,
only presence fallbacks); `routes/benchmarks`' below/above-min flag
chip was deliberately left alone — a single, non-duplicated presence
check with no math to centralise.

New `tests/unit/format.test.ts` (17 cases, each pinning the
null-not-zero direction both ways: a real `0` renders as `0%`/`0.0`,
never as "no data"). `npx vitest run` 30/30 (was 13); `svelte-check` 0;
`npx playwright test` 12/12 unchanged; `npm run build` clean. See
`spec/tasks.md` WPM-T39.

### Fixed — `/verify` crashed with a raw 500 when the authentication service was unreachable (WPM-T37)

`src/routes/verify/+page.server.ts` called `await verifyMagicLink(fetch,
token)` with no `try`/`catch`. A network-level failure (the
authentication service unreachable, timed out, connection reset) makes
`fetch` throw rather than resolve — uncaught, that propagated out of
`load` and SvelteKit rendered its generic 500 error page instead of
this route's own friendly UI. The same bug class was found and fixed
first in `place-front-end-with-svelte` (T-26) and
`thing-front-end-with-svelte` (T-23); ported here: a `try`/`catch`
around the call, a new `"serviceUnavailable"` error variant, and its
message in `+page.svelte`. New `tests/unit/verify.test.ts` unit-tests
the `load` function directly (missing token / service unavailable /
invalid token), verified to fail with the `try`/`catch` reverted and
pass with it restored. See spec §13 WPM-T37.

### Added — root sign-in gate (WPM-T38)

No `+layout.server.ts` existed anywhere under `src/routes`, so a
visitor with no session reached every page — profile, payroll,
wellbeing, all of them — and only discovered they were signed out once
an API call silently failed through the BFF proxy. New root
`src/routes/+layout.server.ts` redirects to `/signin` (303) when
`locals.sessionId` is `null`, excluding `/signin`/`/verify` themselves.
Chosen over person-front-end's narrower per-mutation-page guard: WPM's
pages mix read content with embedded actions (the retention sweep on
`/privacy`, payroll lifecycle actions, …) rather than separating reads
and writes onto dedicated routes, so a root gate fits WPM's actual page
shape better than a small mutation-only subset would.

This app runs SPA mode (`ssr = false`), so the redirect never appears
in the raw document response — it surfaces through SvelteKit's
client-side data fetch during hydration, confirmed empirically via
`curl .../employees/__data.json` returning
`{"type":"redirect","location":"/signin"}`. `tests/e2e/smoke.spec.ts`
gained a `signIn()` helper (injects a fake `__Host-mxi_session` cookie
via Playwright) and a new `"sign-in gate (WPM-T38)"` describe proving
the redirect; every pre-existing smoke test moved under a
`"signed-in smoke coverage"` describe whose `beforeEach` now signs in
first, verified green (all 12 Playwright + 10 vitest tests pass). See
`../spec/tasks.md` WPM-T38.

### Added — reasonable adjustments panel (WPM-T36 / WPM-R33, 2026-07-25)

- Employee profile: a Reasonable-adjustments panel — the
  barrier / impact / change prompts ("You do not need a diagnosis to
  ask"), practical category select, request list with status chips,
  and agree (with practical note) / decline / in-place actions.
  10 i18n keys × 13 locales.

### Added — ergonomics area (WPM-T34 / WPM-R32, 2026-07-25)

- Employee profile: a DSE assessments panel (open with the default
  checklist, answer ok/issue with an equipment note, complete).
  `/workforce`: an ergonomic-issues table (rota-tier). 7 i18n keys ×
  13 locales.

### Added — notifications panel (WPM-T33 / WPM-R31, 2026-07-25)

- Employee profile: an in-app Notifications panel (unread first) with
  mark-read. 2 i18n keys × 13 locales.

### Added — privacy area (WPM-T32 / WPM-R30, 2026-07-25)

- `/privacy`: the retention report (horizon, per-table past-horizon
  counts, expired-consent candidates, derivation) with the sweep
  button and its result counts.
- Employee profile: an **Erase (anonymise)** action for terminated /
  retired employees (confirm-gated; the record 404s afterwards, so
  the page navigates back to the list). 8 i18n keys × 13 locales.

### Added — subject-access download (WPM-T30 / WPM-R30, 2026-07-25)

- Employee profile: a "Download my data" link opening the audited
  subject-access JSON export. 1 i18n key × 13 locales.

### Added — "My 360 requests" (WPM-T29 / WPM-R29, 2026-07-25)

- Employee profile: a rater-side panel listing pending 360 requests
  (subject + group) with inline per-competency scoring and a comment;
  responding clears the request. 1 i18n key × 13 locales.

### Added — 360° appraisals panel (WPM-T28 / WPM-R29, 2026-07-25)

- Employee profile: a 360° panel — create a draft (default
  competencies), nominate raters by group, drive the lifecycle,
  submit per-competency responses for nominated raters, and read the
  group-floored report (withheld cells shown as "Withheld below 3
  responses"). 8 i18n keys × 13 locales (parity green).

### Added — wellbeing pulse (WPM-T27 / WPM-R28, 2026-07-25)

- Employee profile: a pulse card listing open surveys with 1–5 score
  buttons and an anonymous-thanks state. `/wellbeing`: k-floored
  results blocks per survey (overall + per-department cells, the
  suppressed state shown as "Hidden below 5 responses", derivation
  line). 5 i18n keys × 13 locales (parity green).

### Added — working-time panel (WPM-T26 / WPM-R27, 2026-07-25)

- `/workforce` gains a Working-time panel: flagged employees with the
  over-48h average chip (hours/week) and 11-hour rest-gap breach
  chips, the derivation line, and an explicit all-clear state. 4 i18n
  keys × 13 locales (parity green).

### Added — enrolment conversion on the uptake table (WPM-T25 / WPM-R26, 2026-07-25)

- The `/wellbeing` uptake table shows "Enrolled after prompt" for
  plan-linked rules (rate with its terms; em dash when nobody has
  acknowledged). 1 i18n key × 13 locales (parity green).

### Added — benefits awareness (WPM-T24 / WPM-R26, 2026-07-25)

- The wellbeing engine now signposts benefits too: rule kind
  (`health | benefit`) select on the `/wellbeing` create form, kind
  chips on the rules table and the profile prompt card, and the prompt
  type carries the linked benefit plan. 2 i18n keys × 13 locales
  (parity green).

### Added — wellbeing area (WPM-T23 / WPM-R25, 2026-07-24)

- Employee profile: a **Health entitlements** card listing the
  employee's live prompts (with the one multi-dose reminder flagged)
  and the four acknowledgement actions
  (booked / done / declined / dismiss) — informational only.
- `/wellbeing` HR admin page: the configured entitlement rules
  (cohort shown as age band + departments + titles), a create form,
  soft-close, and the **aggregate-only** uptake table (counts by
  response + rate with its terms; no individual appears).
- Six client functions with path pins; nav + 11 keys in 13 locales
  (parity green); one stubbed Playwright spec (suite now 8).

### Added — learning + mentorship areas (WPM-T20, 2026-07-20)

- `/learning` (skills matrix + gap list, per-department training
  analytics, learning-path progress with a path selector) and
  `/mentorship` (mentor load, unmatched employees, stale mentorships).
  Six client functions with path pins; nav + keys in 13 locales; two
  stubbed Playwright specs.

### Added

- 2026-07-19 — SVAR strong fit: the **/requisitions** board upgrades from custom CSS columns to
  the SVAR Kanban: drag between status columns drives the pipeline
  transition endpoint (the service's state machine still refuses
  illegal moves; the reload puts the card back where the truth says
  it belongs).

- 2026-07-19 — SVAR component seams: **@svar-ui/svelte-calendar**,
  **@svar-ui/svelte-kanban**, **@svar-ui/svelte-gantt**, and
  **@svar-ui/svelte-filemanager** are installed (no routes yet —
  candidate features are catalogued per project; see the roadmap).

- 2026-07-19 — SVAR DataGrid + Filter: the **/employees** index upgrades from a plain table to the SVAR
  DataGrid with a FilterBar (number / name / title / department /
  status); masked salaries still render the translated Hidden token;
  row selection opens the profile.

- 2026-07-19 — Lily Design System: the Lily Design System lands in the chrome: the hand-rolled
  locale `<select>` is replaced by **LocaleSelect** (wired to the
  i18n store; `applyDir` off), a **ThemeSelect** offers the full
  45-theme catalogue (stylesheets via the `static/assets/themes`
  symlink; choice persisted to `mxi.wpm.theme`), and the **Lily
  headless** component library is a dependency.

- 2026-07-18 — WPM-T18/T19 implementation round: SvelteKit 2 +
  Svelte 5 runes SPA with same-origin BFF proxy (session → PASETO
  exchange seam; `Accepts-version: 1.0` stamped; no token in browser
  JS), 13-locale i18n (48 keys, parity-tested, RTL ar/ur), typed API
  client + honest `money()`, and the pillar views (dashboard,
  employees + profile, org chart, requisition board + pipeline,
  workforce approvals + rota, development, payroll runs + detail,
  benchmarks). svelte-check clean; 5 vitest + 4 Playwright
  (`page.route`-stubbed) tests green.

- 2026-07-18 — WPM-T0 specification round: cross-cutting spec
  (`../spec/`) and this edition's doc scaffold. No code yet; this
  edition is WPM-T18/T19 in the queue.
