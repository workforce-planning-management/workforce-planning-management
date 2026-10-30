# Worldwide by design

*Added 2026-10-29 at the product owner's direction.*

WPM's target is the whole world, not the United States or the United Kingdom. Much of the current spec, the seed data and the
regulatory register was written with UK examples (UK GDPR, NHS-style pay scales, MHRA, `en-gb`). Those are **examples and the first
worked cases, not the design.** This file is the rule that stops them becoming the design.

## Requirement

- **WPM-R214** Every standard, protocol, convention, locale and the like that WPM adopts must have **equitable capability for countries,
  regions and federations worldwide**. No jurisdiction is the default, the first-class case or the only case that works; a deployer in
  any country, in a federation of states, or in a region spanning several countries can configure and run WPM without a code change and
  without a lesser experience than a UK or US deployer.

## Decision

- **WPM-D143 Jurisdiction is data, never code; a standard is adopted as a profile, never as the only shape.**
  *Proposed, needs product-owner confirmation* (drafted 2026-10-29 by the AI assistant from the instruction above).
  - **Jurisdictions are configuration.** Statutory rules (working-time limits, notice, leave, retention periods, pay-transparency,
    equality categories, data-protection regime) live in per-organization, per-jurisdiction data with an effective date, as pay scales
    and the SFIA import already do. A rule in `src/rules/` takes the jurisdiction's parameters as an input and names no country.
    UK values may ship as a worked example profile beside at least one non-UK profile of a different shape.
  - **Federations and regions are first-class.** A country is not the only unit: a federal state, province, canton, a supranational
    region (for example the EU) and a bloc with shared law are each representable, and a worker or organization may be under more than
    one layer (national and regional; the stricter or the more specific applies as the profile says).
  - **No Anglophone or Western default shapes.** Names (no first/last assumption; one full-name field plus optional parts), addresses,
    dates and calendars (more than Gregorian; week start and weekend days vary), numbers and currency (minor-unit exponent per ISO 4217,
    not always 2), time zones, phone formats, and identifiers (no assumed national-insurance or SSN shape) are all per-locale or
    per-jurisdiction. Equality and disability categories are configured per jurisdiction; none is hard-coded as the list.
  - **Standards: international first, national as profiles.** Prefer an international or open standard (ISO, IETF, W3C, Unicode CLDR,
    ILO conventions) where one exists, and add national or regional standards as profiles of it. Where the only available standard is
    national, say so in the entry and record what a deployer elsewhere does instead.
  - **Locales are equal.** A locale is complete or it is marked incomplete; no locale gets features another lacks. Right-to-left,
    non-Latin scripts, and plural and gender rules beyond English are supported (the 17 shipped locales include `ar-001`, `ur-001`,
    `hi-001`, `bn-001`, `zh-001`); additions follow [localization](../AGENTS/localization.md).
  - **Claims are scoped.** Where the spec says what a law requires, it names the jurisdiction. No sentence says "the law" or "statutory"
    without one. A jurisdiction not yet read is recorded as *not yet read*, as MHRA is, never assumed to match the UK.
  - **Equity is tested.** A change that adds a jurisdiction-specific rule is not done without the same rule's parameters being
    expressible for a second, differently shaped jurisdiction (a test with two profiles).

- **WPM-D144 Jurisdiction rules are held as dated, sourced profiles by jurisdiction code, resolved by one pure function.**
  *Confirmed in part by the product owner, 2026-10-29: questions 1 to 4 of the design (the demo seed always loads example profiles; HR of the organization edits a profile; ISO 3166 plus a deployer-extended supranational list; a structured `work_jurisdiction` set by HR). Question 5 answered "y" to the assistant's suggestion: the second example profile is India, then Japan, both labelled `example`/unverified.* The design is in "Jurisdiction profile design" below: parameters with no default, layers
  (supranational, national, subnational) chained per organization, a combination rule per parameter, the applicable chain taken from the
  place the work is done, and append-only rows with a source and a status.

## Tasks

- [x] (2026-10-29) WPM-T367 **Worldwide audit of the spec.** *(WPM-R214, WPM-D143)* Done: 19 rows in the audit table above. `spec/` only: code, seed data and UI strings are not audited. Wording fixes (rows 10, 11) are left to T370.
- [x] (2026-10-29) WPM-T368 **Jurisdiction profile model.** *(Done: see the design section below; proposed WPM-D144; questions for the product owner at its end.)* Original text: *(WPM-R214, WPM-D143)* A design for per-organization, per-layer (national, regional,
  supranational) profiles with effective dates, and what each existing rule (working-time guardrails, retention, pay transparency,
  equality categories) reads from it. Design first; no code.
- [~] WPM-T369 **Second-profile tests.** *(WPM-R214, WPM-D143)* For each rule with UK values, a pure-core test with a non-UK profile of a
  different shape (different currency exponent, weekend, week start, limits), and a lint or test that fails if `src/rules/` names a
  country.
  **Done in part (2026-10-30):** (1) `src/jurisdiction_neutrality.rs`, in `cargo test --lib`: a lexical check of every identifier and
  string literal in `src/rules/` (comments excluded, since D143 asks prose to name its regime) against lists of country and region
  names, phrases (for example "Working Time Regulations", "national insurance") and upper-case country codes; 27 allow-list entries,
  each a file, a word and a reason (the consultation and holiday example profiles and packs, the `uk-gdad-pcf` framework id, ESCO URIs
  in fixtures, test inputs for code shapes); an entry that matches nothing fails. Tests: a planted file (a constant, a function, a
  camel-case name, a raw string, a code, and names in comments that must not count), ordinary words ("us", SQL `IN`), and the
  allow-list's scope. Mutation-checked: a `"Japan"` constant named `PLANTED_UK_LIMIT` added to `working_time.rs` failed the check on
  both words, and was removed. **Coarse by design:** a country not on the lists is not caught; `IN` is not checked. (2)
  `src/rules/second_profile.rs`: 10 tests running each rule that really takes its limits as parameters with a synthetic second profile
  of a different shape, named for no country: collective consultation (a percentage-only test, a size floor, a window of 60 calendar days, no
  authority notice), holiday rules (no Easter and no substitute day, so a Saturday holiday stays; the nearest-weekday policy crossing
  the year), retention horizons (pay held 3,650 calendar days, recruitment below the floor raised to 30), joint pay assessment (10% and
  3 months), the pay-information period, pay-gap reporting tiers (one band from 50 workers every two years), flexible working (3
  months, one request a year, 30 calendar days to appeal, its own refusal list), resignation notice (90 and 0 calendar days), and money
  (JPY and KWD codes valid; the API never divides by 100). **Not done, and why:** no rule was refactored. The rules that hard-code one
  regime's values take no profile yet: the working-time limits and the roll-up week (WPM-T380), the weekend days of the holiday rules
  and `is_business_day` (WPM-T379), the payroll stub's allowance, rate, day and month (WPM-T383); the pay-gap tiers are a parameter of
  the rule but not of the controller (WPM-T384). Week start has no rule in the API to test; the UI's money exponent is WPM-T373. These
  follow D144's order of work, whose resolver and parameter registry are not among the parts the product owner confirmed (questions 1
  to 4), so none was built here. Not tested: the deployment-wide `WPM_*` settings that feed these parameters (controllers, not the
  pure core).
- [x] (2026-10-30) WPM-T370 **Regulatory register by region.** *(WPM-R214, WPM-D143)* Re-organize [regulatory.md](regulatory.md) so the UK is one
  section among regions, with the others marked *not yet read* rather than absent.
  **Done (spec only):** [regulatory.md](regulatory.md) now opens by saying the law is recorded by region, and ends with "The register by
  region": international standards (ISO 14971), the European Union (GDPR, the Pay Transparency Directive, the AI Act section moved under
  it), the United Kingdom (UK GDPR and the DPA 2018, the Equality Act 2010, right to work, the worked examples, the MHRA), and the United
  States, Canada, Latin America and the Caribbean, Africa, the Middle East and the Gulf, South Asia (India, the planned example profile),
  East Asia (Japan), South-East Asia and Oceania, each *not yet read*. "Production would additionally require" names the jurisdiction of
  each UK duty. Wording fixes: row 10 ([plan.md](plan.md) WPM-R76 names the GDPR and the UK GDPR by region; [design.md](design.md) says
  the adjustments design follows the barrier, with the UK Equality Act 2010 as the one regime read) and row 11 ([glossary.md](glossary.md)
  "Right to work" scoped to the jurisdiction). A stray duplicated heading line in the AI Act section was removed. **Not done:** no law of
  any region was read for this task, so no new fact was added; rows 7, 8 and 15 are covered only by the register's regions, not by edits
  to [pay-transparency.md](pay-transparency.md), [worker-unions.md](worker-unions.md) or [mhra.md](mhra.md); the regions are a
  coarse list, not a per-country register.
- [x] (2026-10-30) WPM-T371 **Locale and convention equity check.** *(WPM-R214, WPM-D143)* Names, addresses, dates and calendars,
  numbers and currency, identifiers, right-to-left and plural rules: a checklist run against the UI and API, with gaps recorded as tasks.
  **Done (read-only):** 39 findings with file and line in "Locale and convention equity check" below (9 right, 9 partial, 21 gaps); the
  gaps are T373 to T386. Audit row 16 is corrected in part: the API never assumes a currency exponent, the UI assumes 2. **Not done:** no
  behaviour and no wording changed (the wording candidates are API messages and UI strings, left to their tasks); nothing was run in a
  browser; the CMS was not examined.
- [x] (2026-10-30) WPM-T373 **Money uses the currency's own exponent in the UI.** *(WPM-R214, WPM-D143; check rows 21, 24)* `money`, `moneyWhole`
  and the three amount inputs take the minor-unit exponent from the currency (ISO 4217, through `Intl.NumberFormat#resolvedOptions` or a
  table), not 100; `PlanForecast` passes the locale; unit tests with JPY (0) and KWD (3).
  **Done:** one formatter and parser, `src/lib/money.ts` in the UI: `money`, `moneyWhole`, `signedMoney`, `minorToInput`, `parseAmount`,
  `toMinor`, `currencyCode` and `exponentOf`. The exponent is `Intl.NumberFormat(locale, { style: "currency", currency })
  .resolvedOptions().maximumFractionDigits` (JPY 0, EUR 2, KWD 3, CLF 4; 2 only when the runtime does not know the code), and the format
  is `Intl` in the page's locale (`i18n.locale`), which is now the default, so `PlanForecast` and every other caller get the reader's
  locale. **Shared in behaviour with the sibling project-portfolio-management UI's `src/lib/money.ts` (its commit 956d9ed)** — the same
  exponent, formatting and parsing rules — so the two apps stay alike; change both together. WPM's differences are its conventions only
  (an unknown amount or a missing currency is "—"; `moneyWhole` drops a fraction that is zero in the currency's own minor unit).
  `src/lib/api/wpm.ts` re-exports `money` and `moneyWhole` from it; the two hand-rolled `minor / 100` formatters are gone, and the expense
  item, pay-scale salary and plan budget inputs store `toMinor(text, currency)` instead of `Math.round(x * 100)`. The API contract is
  unchanged (integer minor units plus a currency code). Exports (CSV from the data grids) already went through `money()`. Tests:
  `tests/unit/money.test.ts` (16: the four exponents, in every shipped locale for JPY and KWD; German and French separators; the em dash;
  `moneyWhole` in GBP, JPY and KWD; parsing and round trips; and a scan of `src/` that fails on a fixed `* 100`, `/ 100` or `% 100` on a
  money name, or on a `$state("XXX")` currency). Playwright: a yen pay scale on `/de-001/pay-scales` (whole yen, `5.200.000 ¥`, a salary
  sent as `salary_minor=5200000`), a yen expense claim (`JP¥1,500` for 1500 typed) and a KWD plan budget (`1.234,567` typed in German sent
  as 1,234,567 fils). A mutation that forced the exponent to 2 failed the yen and KWD specs. **Not done:** the percentages, decimals and
  `toFixed` of row 24 (WPM-T375); the remaining `/ 100` in the UI are basis points, centi-FTE and percentages, not money. **Not tested:**
  a runtime whose `Intl` lacks currency data (the fallback exponent of 2 is reached only there); screenshots were not looked at.
- [x] (2026-10-30) WPM-T374 **Plural rules.** *(WPM-R214, WPM-D143; rows 26, 27)* A `tn(key, count)` that picks a CLDR plural form with
  `Intl.PluralRules(i18n.locale)`; the 8 "(s)" strings and the English "s" in `FrameworkRolePanel.svelte:315` replaced first; parity and
  CMS config updated for the new keys.
  **Done (the named strings):** a *plural group* is a catalogue object whose keys are CLDR categories (`zero one two few many other`),
  `other` among them, so a key `x` is held as `x.one` / `x.other` in English. `src/lib/i18n.svelte.ts` adds `tn(key, count, params)`,
  `translatePlural`, `pluralCategory`, `pluralCategories` (from `Intl.PluralRules#resolvedOptions`) and `PLURAL_KEYS`; the fallback is
  the locale's category, its `-001` base, the group's `other`, and only then English by English's rules. The nine strings are plural
  groups: the eight "(s)" strings (team aspirations, CPD requirements, withheld skills, uncosted line groups, a role step's new skills,
  unsuggested skills, withheld programmes, the pay-gap reporting interval) and `FrameworkRolePanel`'s "change{changes2}" (the English
  "s" is gone). Each locale holds exactly the categories its language uses: ar and cy six, ru four (one, few, many, other), es, fr and pt
  three (one, many, other: CLDR's "many" is for a million and over, "1 000 000 de …"), bn, de, en, hi and ur two, id and zh one;
  AI-written and unreviewed (`content/locales/REVIEW.md`). The French and Portuguese second counts in two strings became "label: n"
  so they need no form. The CMS offers all six categories for a plural group in every locale. Tests: `tests/unit/plurals.test.ts`
  (selection in en, ar, ru, cy, zh and id, a regional locale using its language's forms, no "(s)" left in English, and every plural
  group in every `-001` locale holding exactly its language's categories); the parity test checks plural groups there instead of key
  for key; Playwright: the CPD requirement count in English (1, 3), Russian (3, 5) and Arabic (2). **Not done:** the other strings that
  fill a count ("{n} people", "{days} calendar days" and about 30 more) still have one form; only the count that chooses the form is
  agreed in a string with two counts. **Not tested:** decimal counts (the "other" form of ru, the "few" of some languages for
  fractions); a native speaker has not read any form.
- [x] (2026-10-30) WPM-T375 **Dates and numbers through the page's locale.** *(WPM-R214, WPM-D143; rows 10, 11, 24)* One date, date-time, number and
  percent helper on `Intl` with `i18n.locale`, used for raw ISO dates, `toFixed`, hand-made percentages, `toLocale*([])` and the numbers
  `tf`/`tp` put into strings; the English "now" in `CareerHistory.svelte` becomes a key.
  **Done (most of it):** one module, `src/lib/locale-format.ts` in the UI: `formatNumber`, `formatInteger`, `formatDecimal` (the
  locale's `toFixed`), `formatPercent` (a fraction), `formatPercentValue` (a value already in percent), `formatBasisPoints` and
  `formatCentiFte` (the point placed exactly, `10456e-4`, not by division), `formatSigned`, `formatDate` (a civil date in UTC so it is the
  same day everywhere; a timestamp in the reader's zone), `formatDateTime`, `formatTime`, `formatMonth`, and for typed numbers
  `parseNumber` and `parseScaled` (on `money.ts`'s `parseAmount`, not a second parser) and `numberToInput`. An unknown is "—". `format.ts`
  (`percent`, `rate`, `mean`, `percentOf`, `basisPoints`, `workings`) now writes through it. `tf` and `tp` write a number placeholder with
  `Intl.NumberFormat` in the page's locale (a year is passed pre-formatted without grouping); the percent sign stays in the catalogue
  string, where each locale already puts it its own way. `DataTable` shows a number or an ISO date in a column with no `type`,
  `format` or `cell` in the page's locale, still sorting and exporting the value as sent. About 85 raw ISO dates in markup and
  placeholders (rows 10's list and the rest the scan found), the 17 `toFixed` and hand-made percentages of row 24, the two `toLocale*`
  calls of row 11 (now the page's locale, not the browser's), the sliced timestamps, the CEO, home, mentorship and privacy headline counts,
  and the English "now" in `CareerHistory` (now `values.now`, and the column sorts on the ISO value) go through it. The eight decimal
  inputs (on `/planning` attrition, on-cost and a line's FTE; on `/capacity` the operations reservation and a claim's FTE; on `/cpd` the
  required and logged amounts) are text inputs with `inputmode="decimal"`, read in the page's format and refused before a request with
  `common.badNumber` / `common.badNumberAtLeast` (two new keys in the 13 `-001` locales, AI-written and unreviewed). The API is unchanged:
  ISO dates, basis points, centi-FTE. Tests: `tests/unit/locale-format.test.ts` (unit tests in en, de, fr, hi, bn, zh and en-us, and a
  scan of `src/` that fails on a new `toFixed`, `toLocale*`, a percent sign glued to a number, a timestamp sliced to a date, or an ISO
  date field put straight into markup, a template string or a `tf`/`tp` placeholder); Playwright: a German plan's "12,5" and "28,3" sent
  as 1250 and 2830 basis points and a non-number refused, a payroll period in German (`01.09.2026`) and in Bengali digits, and 26 existing
  assertions moved from ISO text to the en-001 form ("1 Sept 2026", "200,000"). **Not done:** a bare count put straight into markup
  (`{x.length}`, `{row.score}`) outside the places named above is not all converted, and the scan cannot see one (it has no types); a
  date in a multi-line `tf`/`tp` call is outside the scan; ranges still use a literal "→" (WPM-T377, *since done*); `type="date"` inputs show the
  browser's format; whole-number inputs stay `type="number"`; the calendar's shift key is still the UTC day. **Not tested:** screenshots
  were not looked at; Arabic-Indic digits typed on an Arabic page; a reader in a time zone far from UTC (a timestamp shown as a date can
  be the day before or after its UTC date, which is intended).
- [x] (2026-10-30) WPM-T376 **No default currency; locale decimal input.** *(WPM-R214, WPM-D143; rows 22, 23)* The `"GBP"` defaults in the UI go (the
  organization's currency when it has one, else empty); an amount typed with the locale's decimal separator is read correctly.
  **Done (the money parts):** the new-expense-claim form and the plan form start with no currency (`$state("")`); a claim cannot start, and
  a plan with a budget cannot be created, until a three-letter code is typed ("expenses.currencyNeeded",
  "pages.planning.budget_needs_currency"); a plan with no budget sends no currency. The organization has no currency in the API, so there is
  nothing to take one from; an expense item and a pay-scale salary take the record's own currency. An amount is read by `parseAmount` in the
  page's locale: its decimal mark is the only decimal mark ("12,50" and "1.234,50" in German, "1 234,50" in French, "1,234.50" in English),
  its group mark and spaces are ignored, its own digits are read (Bengali), and anything else is refused before a request
  ("expenses.badAmount", "pages.planning.bad_budget", the existing "payScales.badSalary"). The plan budget is now a text input with
  `inputmode="decimal"`, not `type="number"` (whose decimal mark follows the browser, not the page). The pay-gap and pay-range currency
  filters use the same `currencyCode`. Four new keys in all 13 `-001` locales, AI-written and unreviewed (recorded in
  `content/locales/REVIEW.md`). Playwright: the empty currency refused with nothing sent, a German claim with "1.234,50" and "12,5" sent as
  123450 and 1250 and shown as `1.247,00 €`, a non-number refused, axe (WCAG 2 A and AA) over the expense panel, and the plan budget specs
  above. **Not done:** no organization currency exists to default to. (The percentage inputs on `/planning`, attrition and on-cost,
  were left as `type="number"` here and moved to the locale's decimal input by WPM-T375; only the whole-number headcount input stays
  `type="number"`.) **Not tested:** a person on an Arabic or Urdu page typing Arabic-Indic digits (the parser reads the
  locale's own digits, but `ar-001` formats with ASCII digits, so only those are tested there).
- [x] (2026-10-30) WPM-T377 **Right-to-left finish.** *(WPM-R214, WPM-D143; rows 28 to 30)* Logical CSS for the 11 physical declarations (tables
  first); ranges as a catalogue pattern or `formatRange` instead of a literal "→"; a `ur-001` direction test beside the `ar-001` one.
  **Done:** the 11 physical declarations are logical: `text-align: start` on every table cell, `border-inline-start` (and its colour)
  on the bed cards, the home feature cards and calendar events, `padding-inline-start` on calendar events; no other physical left/right
  property was found in `src/` (four-value `margin`/`padding` shorthands, `float`, `inset` and `style:` directives were checked).
  `src/lib/locale-format.ts` adds `formatDateRange` (`Intl.DateTimeFormat#formatRange` in the page's locale: "1–30 Sept 2026",
  "01.–30.09.2026"; an unknown end is "…" with an en dash, no direction), `formatDateTimeRange` (a shift, in the reader's time zone)
  and `arrow()` ("→", or "←" on an Arabic or Urdu page) for an arrow that means "becomes" (3 → 4, a stage to the next, a requester to a
  taker). The 33 literal arrows in 22 components and pages go through them, among them three ranges that showed raw ISO dates
  (`OnCall`'s stretches, `SkillGaps`' training plan, `Backups`' open ends). The eight catalogue strings already used "←" in `ar-001` and
  `ur-001`. The share button's forward arrow (Lily `SharePicker`) is mirrored under `dir="rtl"`; the other icons (menu, close) have no
  direction, and the menu drawer already opens from the inline start. The axe run found a real fault on the way: the payroll list's
  period column had no header, so its sort button had no name; it is now `columns.period` (each locale's existing `ceo.period` word,
  `REVIEW.md`). **Tests:** `tests/unit/right-to-left.test.ts`: a scan of `src/` that fails on a physical left/right property, alignment,
  float or `style:` directive and on a literal "→"/"←" outside comments (an allowlist with a reason per line, empty today), and unit
  tests of the ranges and the arrow in en, ar and ur. Playwright `tests/e2e/right-to-left.spec.ts` on `ar-001` and `ur-001`: `dir`
  and `lang`, every payroll table cell `text-align: start` with its text at the cell's right edge, the period written as the browser's
  own `formatRange` with no arrow and no ISO date, axe (WCAG 2 A and AA) over the page and the open menu, the share arrow mirrored, the
  menu drawer flush with the right edge and its link navigating; an `en-001` control (left to right, arrow not mirrored). The payroll
  list's assertions moved from "1 Sept 2026 → 30 Sept 2026" to "1–30 Sept 2026" (and "01.–30.09.2026" in German). Screenshots of the
  Arabic payroll page and the Urdu menu were looked at. **Not done:** the theme stylesheets in `static/assets/themes/` (copied from Lily)
  are not scanned; the trend chart (`TrendChart.svelte`) keeps time running left to right on every page; the calendar's own
  toolbar and keyboard come from Lily and were not changed. **Not tested:** every other page in Arabic or Urdu (only the payroll list,
  the menu and the scan); screenshots in dark mode or at phone width; a native reader.
- [ ] WPM-T378 **Today, week start and weekend in the UI.** *(WPM-R214, WPM-D143, WPM-D144; rows 12, 13, 16)* "Today" in a time zone
  (the user's or the organization's), not UTC; week start and weekend days for the calendar from a region setting separate from the
  language. Needs the jurisdiction model (D144) or a user setting decided first.
- [ ] WPM-T379 **Weekend days and non-Gregorian holidays in the holiday rules.** *(WPM-R214, WPM-D143, WPM-D144; rows 15, 18, 19)*
  `rules::holiday` takes the weekend days as a parameter (the D144 `holidays.weekend_days`), so a substitute day and `is_business_day`
  follow it; a rule kind for a holiday fixed in another calendar; an example pack of a different shape. The registry is not confirmed yet
  (only D144 questions 1 to 4 are), so this waits for it.
- [ ] WPM-T380 **Working-time limits as parameters.** *(WPM-R214, WPM-D143, WPM-D144; rows 14, 17)* The three constants become the D144
  parameters `working_time.reference_weeks`, `max_average_weekly_minutes` and `min_daily_rest_minutes`, with `no limit configured` when
  none is set (D144 question 1, confirmed) and the demo seed loading an example profile; the roll-up's week start and its partial index
  (fixed at 2,720 minutes) follow. D144's order of work, step 3, after the resolver (step 1).
- [ ] WPM-T381 **Names without a first and last.** *(WPM-R214, WPM-D143; rows 2, 3)* The duplicate check does not split a name into
  first word and last word (compare the whole name, or parts a person gave); sorting uses `Intl.Collator(i18n.locale)`.
- [ ] WPM-T382 **Addresses, phones and world English wording.** *(WPM-R214, WPM-D143; rows 4 to 7, 32)* Neutral `en-001` labels ("Postal
  code", "State, province or region"), the British ones moved to `en-gb`; a country picker named by `Intl.DisplayNames`; an address order
  per country or one free-text block; phone digits accepted in any script and stored as ASCII.
- [ ] WPM-T383 **Payroll stub is not one country's.** *(WPM-R214, WPM-D143; row 33)* The stub tax allowance and rate, the 7.5-hour day
  and the minutes in a month are parameters (or the stub says whose they are and refuses a currency it was not written for).
- [ ] WPM-T384 **Say whose each statutory default is.** *(WPM-R214, WPM-D143, WPM-D144; rows 34 to 36)* Each `WPM_*` default from one
  regime (flexible working, resignation notice, pay information, joint assessment, retention) is named with its regime in the API's
  answer and the spec, then moved to a profile when the registry exists; the pay-gap tiers become configurable.
- [ ] WPM-T385 **A seed that is not only GBP.** *(WPM-R214, WPM-D143, WPM-D144; row 39)* The seed and `demo-data.py` include a currency
  of another exponent and, once profiles exist, the example profiles D144 question 1 calls for (India, then Japan, labelled `example`).
- [ ] WPM-T386 **The workstation checklist is content, not English code.** *(WPM-R214, WPM-D143; row 37)* The default DSE items become
  keys a locale translates, and are named as one regime's list.

## Audit (WPM-T367, 2026-10-29)

Method: a search of `spec/` for UK and US terms (UK, US, NHS, GDPR, Equality Act, Working Time Regulations, national insurance, £, `en-gb`,
and the like), then a read of each hit and of the places where a UK shape is built into a field or rule without a name. Read-only: nothing
is changed by this audit; each row becomes a follow-up. **Not done:** the code, the seed data and the UI strings were not audited (only
`spec/`); the search finds named terms, so an unnamed assumption (a field shaped for one country) is found only where the row says so.

| # | Where | What it assumes | Verdict | Follow-up |
| --- | --- | --- | --- | --- |
| 1 | [pay-scales.md](pay-scales.md) | A single national public-sector scale, in £, with NHS bands and allowances | **Worked example, but not labelled as one in the title.** The model was made generic (WPM-T102); the data is UK. | Retitle as an example; add a second, differently shaped scale (T369) |
| 2 | [staff-records-and-pay.md](staff-records-and-pay.md) pay progression, pension | NHS Agenda for Change; the NHS Pension Scheme tiers and rates | **Worked example**, already described as a synthetic scheme. | Keep; say "example" at the first mention |
| 3 | staff-records-and-pay.md, [domain-model.md](domain-model.md) student loans | `StudentLoanPlan` with UK plan names (1, 2, 4, 5, postgraduate), a tax year | **Shape assumes the UK.** Other countries have different or no loan-deduction schemes. | Generalize to a *statutory deduction scheme* keyed by jurisdiction (T368) |
| 4 | staff-records-and-pay.md, domain-model.md, [regulatory.md](regulatory.md) payroll details | `tax code`, `NI category`, `starter declaration` are fields | **UK-shaped field names.** | Rename or make them jurisdiction-keyed codes (T368) |
| 5 | [workforce-management.md](workforce-management.md) working-time guardrails (WPM-R27) | The 17-week 48-hour average and 11-hour rest gap, called "the Working Time Regulations" | **UK/EU limits as the only limits.** Other countries set different weekly, daily and rest limits. | The limits become profile parameters; the text names the regime (T368, T369) |
| 6 | [requirements.md](requirements.md) public holidays | Packs for England and Wales, US federal and Germany; a weekend policy of stay, next weekday, or Friday and Monday | **Already data, and already labelled unverified.** The weekend is assumed Saturday and Sunday; calendars other than Gregorian are not covered. | Per-region weekend days and week start; non-Gregorian holidays (T371) |
| 7 | [pay-transparency.md](pay-transparency.md) | The EU Pay Transparency Directive (2023/970) | **Correctly scoped to the EU**, and the threshold is configuration. It is the only pay-transparency regime covered; the UK, US states and others are absent. | Record other regimes as *not yet read* (T370) |
| 8 | [worker-unions.md](worker-unions.md) | "Special category data under the UK GDPR and the EU GDPR" | **Scoped, but two regimes only.** Union membership is treated differently, or protected differently, elsewhere. | Add other regimes as not yet read (T370) |
| 9 | [regulatory.md](regulatory.md) | UK GDPR and DPA 2018 lawful-basis mapping; a section on UK regulators; the EU AI Act | **UK-first register.** | Reorganize by region (T370) |
| 10 | [plan.md](plan.md), [design.md](design.md) | "UK GDPR Article 35" for the DPIA; "the Equality Act's actual shape" for the adjustments duty | **Statutory language without a jurisdiction** (the Equality Act is UK law). | Name the jurisdiction in the sentence; the design is the barrier-based duty in many regimes (T370) |
| 11 | [glossary.md](glossary.md) | "Right to work: statutory employment-eligibility check" | **Unscoped.** The check, and whether there is one, differs by country. | Scope the entry |
| 12 | [equality-impact/template.md](equality-impact/template.md) | Lists the UK Equality Act 2010 and the EU rules as examples | **Already says the deployer's lawyers decide**; examples are two regimes. | Keep; add that the list is not exhaustive |
| 13 | [disability-and-adjustments.md](disability-and-adjustments.md) and WDES | WDES (NHS England's Workforce Disability Equality Standard) mapping by number | **A UK health-service standard**, used as a reporting map. | Mark as one profile (T368) |
| 14 | [uk-gdad-pcf](uk-gdad-pcf/index.md), [oxford-spelling](oxford-spelling/index.md) | UK government digital-and-data framework; British-with-"-ize" spelling for the spec's own prose | **Named as UK on purpose**, and one is a writing convention for the spec, not a product behaviour. | Keep; say the spec's prose convention does not set a locale for the product |
| 15 | [mhra.md](mhra.md) | A UK regulator, and ISO 14971 | **Scoped to the UK by name**, and a non-workforce item. | Keep; the register gets sibling regions (T370) |
| 16 | [expense-claims.md](expense-claims.md), [domain-model.md](domain-model.md) money | One ISO 4217 currency per claim; minor units | **Good in the spec**: money is minor units plus a currency, and no two-decimal assumption was found in the text. | Confirm in code that the exponent comes from the currency (T369) |
| 17 | [locales-for-global-sharing-with-svelte](locales-for-global-sharing-with-svelte/index.md), [locales-x-for-roles](locales-x-for-roles/index.md) | 17 locales across scripts, `en-001` as the world-English default | **Good.** Two English variants carry a country (`en-gb`, `en-us`); the other locales are language or region based. | Keep; the equal-locale rule (D143) is checked in T371 |
| 18 | [sfia.md](sfia.md), [esco](esco/index.md) | SFIA (UK-origin, international use) and ESCO (EU) as skills frameworks | **Both are optional imports from the deployer's own file**; neither is built in. Other national skills frameworks are not covered. | Say so; the import is the extension point (T368) |
| 19 | [purpose.md](purpose.md), [scope.md](scope.md) | "Jurisdiction-complete tax law" and "multi-country labor-law engines" out of scope | **Consistent with D143**: WPM holds parameters, not law. | Keep; say "multi-country" in scope means *no engine*, not *no support* |

### What the audit found, in short

- **The UK is the only worked example in 9 of 19 rows** (1 to 5, 9, 10, 11 and 13: pay scales, pension, student loans, payroll fields, working
  time, the register, the DPIA and equality wording, the right-to-work entry and WDES). Of these, rows 3, 4 and 5 are **field or rule shapes**, not only
  examples: they would not fit another country as written. These are the first for T368.
- **The EU is covered only for pay transparency (row 7)**, the AI Act and GDPR. No other region (for example Africa, Latin America, South and
  South-East Asia, the Gulf, Oceania, Canada, or the US states) has a section, and none is marked *not yet read*. That is the gap T370 fills.
- **What is already right:** money is in minor units with a currency (16), holidays are data (6), locales span scripts (17), and the
  skills frameworks are optional imports (18).
- **Wording to fix at once** is rows 10 and 11, where statutory language has no jurisdiction. These are edits, not designs, and are
  carried in T370.

## Jurisdiction profile design (WPM-T368, 2026-10-29)

*Design only: no code, migration or test changed. WPM-D144; questions 1 to 4 below were answered yes by the product owner on 2026-10-29
(HR edits), question 5 answered: India first, then Japan.*

### What exists today

A jurisdiction's rules reach WPM in four different ways, none of which can hold two jurisdictions at once:

| Way | Examples | Limit |
| --- | --- | --- |
| A **constant in code** | `working_time.rs`: 17 weeks, 48 hours, 11 hours of rest | One value for every deployment |
| A **deployment-wide setting** | `WPM_RETENTION_DAYS`, `WPM_PAY_INFORMATION_MONTHS`, `WPM_JOINT_ASSESSMENT_THRESHOLD_PERCENT`, `WPM_PAY_GAP_SEX_CATEGORY` | One value per deployment, so one country per deployment, and no date |
| **Rows with a region** | `holidays.region` (a code like `GB-ENG` or `US` or `DE`, checked against `^[A-Z0-9]{2,3}(-[A-Z0-9]{1,3})?$`) | Right shape, but only for holidays |
| **Reference data** | pay scales (WPM-T102), student loan thresholds, SFIA and ESCO imports | Dated and sourced, but keyed by nothing common |

### The model

A **profile** is a dated, sourced set of **parameters** for one **jurisdiction code**, held by an organization. Three nouns:

- **Jurisdiction code.** The shape the holidays already use, which is ISO 3166-1 alpha-2 for a country, ISO 3166-2 for a subdivision (a
  state, province, canton, nation within a state) and a registered code for a supranational region (`EU`; others by a short list the
  deployer extends). A code has a **layer**: `supranational`, `national` or `subnational`. A code has a parent (`GB-ENG` → `GB`; `DE` → `EU`),
  so a federation, a bloc or a country of countries is just a chain. The chain is data, kept per organization, because membership changes
  (a state leaves a bloc) and WPM does not assert it.
- **Parameter.** A key from a **parameter registry** (a list in the spec, then in code): `working_time.max_average_weekly_minutes`,
  `working_time.reference_weeks`, `working_time.min_daily_rest_minutes`, `retention.<class>.days`, `pay_transparency.information_months`,
  `pay_transparency.joint_assessment_percent`, `holidays.weekend_days`, `calendar.week_start`, `equality.categories`, `deduction.<scheme>`,
  and so on. Each registry entry fixes its **type and unit**, its **combination rule** (below), and its **validation range**. A parameter
  has **no default value**: not configured is `unknown`, never a number (WPM-D141's rule, and the AGENTS rule that an unknown is never a zero).
- **Profile row.** `(organization, jurisdiction code, parameter key, value, effective_from, effective_to?, source, checked_on, status)`.
  `status` is `read` (the source was read and checked on that date), `example` (a worked case, as the UK values will be) or `unverified`.
  Rows are **append-only**: a change is a new row with a later `effective_from`, so a past decision can be explained by the rule in force
  that day, and the audit trail names the parameter and the code but no person and no figure about a person.

### Which jurisdictions apply to a worker

A worker is under the chain of the **place the work is done**, **not** nationality, citizenship, residence or any characteristic of the
person. This is the usual legal test for employment rules, and it keeps a protected characteristic out of the rules. **The staff record
cannot say this today:** `Worker.location` is free text (≤ 100 characters, [domain-model.md](domain-model.md)), so it cannot be resolved to a
code. The design therefore needs a new, structured, optional **`work_jurisdiction`** (a jurisdiction code) on the worker, set by HR, with the
free-text location left as it is. A remote worker's value is a fact about the assignment, set by HR. Where it is not set, the answer is
`unknown` and the guardrails say so; it is never defaulted to the organization's own country.

### How parameters combine across layers

Each registry entry names one rule, and the resolver is one pure function in `rules::jurisdiction`:

| Rule | Used for | Meaning |
| --- | --- | --- |
| `most_specific` | week start, weekend days, currency exponent | The deepest layer that has a value wins |
| `strictest_min` / `strictest_max` | a weekly-hours ceiling (the lowest value), a rest minimum (the highest value) | The most protective of all layers that apply |
| `union` | public holidays, equality categories | All layers' values together |
| `bounded` | retention (a floor from one law, a ceiling from another) | The intersection; an **empty intersection is refused and shown** to HR, not resolved silently |

`resolve(parameter, chain, as_of) -> Resolved { value, from_code, status, source } | Unknown`. It is DB-free and clock-free, tested with at
least two profiles of a different shape (T369). A result always says **where the value came from**, and every screen or report that
uses one shows it, as the capacity view shows its demand's source (WPM-D140).

### What changes in each existing area

| Area (audit row) | Today | After |
| --- | --- | --- |
| Working-time guardrails (5) | Constants, named for the Working Time Regulations | Three parameters; the UK/EU values are an `example` profile; with none configured the flags say `no limit configured`, not `no breach` |
| Retention (config) | One `WPM_RETENTION_*` per deployment | `bounded` parameters per class; the env var becomes a fallback for a deployment with no profile, until it is removed |
| Pay transparency (7) | `WPM_*` settings with the Directive's values | Parameters under `EU`; another region's regime is another profile, or absent |
| Student loans (3) | `StudentLoanPlan` with UK plan names | A general **statutory deduction scheme**: `(jurisdiction, scheme, code, period, threshold, rate, source)`; the UK plans become example rows |
| Payroll fields (4) | `tax_code`, `ni_category` | A list of `(scheme, code)` pairs per worker, validated against the jurisdiction's scheme list; the two names become `scheme` values. **WPM-D115 generalizes:** no bank detail and **no national identifier of any country** gets a column |
| Public holidays (6) | `holidays.region` | Unchanged; adds `holidays.weekend_days` and week start per code |
| Equality categories (13) | A configured category list | `union` parameter per code; the pay-gap "sex" category stays a named setting |
| Money | Currency per row | Unchanged; the minor-unit exponent comes from the currency (T369 checks the code) |

### What does not change

- **WPM stores parameters, not law** (scope: no labour-law engine). A profile is a deployer's statement with a source, not WPM's claim.
- **Roll-ups, floors and masks are unchanged.** A profile never makes a figure about a person visible; a count under the floor is hidden in
  every jurisdiction.
- **No profile is the default.** The demo seed ships an `example` UK profile **and** a differently shaped example (proposed: one with a
  Sunday-start week, a six-day working week, a different currency exponent and a federal layer) so that nothing in the demo reads as
  "the" shape.

### Order of work

1. A pure `rules::jurisdiction` (registry, chain, `resolve`), with two-profile tests (T369).
2. The tables (`jurisdictions`, `jurisdiction_parameters`), HR-only read and write, audited without figures about people.
3. Move one rule at a time, working time first (it has constants and a clear test), then retention, then pay transparency, then deductions
   and payroll fields.
4. A screen showing, for an organization, each parameter, its value, its source and layer, and its status.

### Needs the product owner

1. **Behaviour change.** **Answered 2026-10-29: yes.** With no profile configured, today's UK/EU working-time flags would become `no limit configured`. Is that right
   for the demo, or does the demo seed always load an example profile (the proposal)?
2. **Who edits a profile.** **Answered 2026-10-29: yes (HR).** HR of the organization (proposed), the deployer only, or both with a review?
3. **Subnational and bloc codes.** **Answered 2026-10-29: yes.** Is ISO 3166 plus a deployer-extended supranational list acceptable, or is a different registry wanted?
4. **Where the work is done.** **Answered 2026-10-29: yes.** Is a new structured `work_jurisdiction` on the worker, set by HR, the right test, and is it HR's to set for a remote worker?
5. **Naming the example profiles.** **Answered 2026-10-29: India first, then Japan** (the UAE as a possible third).

## Locale and convention equity check (WPM-T371, 2026-10-30)

Method: a read of the code, not the spec. The UI is `workforce-planning-management-ui-with-svelte` (`src/`, `content/locales/`); the API is
`workforce-planning-management-service-with-rust` (`src/rules/`, `src/controllers/`, `src/tasks/seed.rs`, the migrations) and
`scripts/demo-data.py`. Paths below are relative to the subproject named in the first column (UI or API) or to the repository root (root).
Line numbers are as of commit `55e4e69`. Each gap is a task (T373 to T386). **No behaviour changed in this check, and no wording was
changed either:** the only candidates were API error messages ("such as GBP") and UI strings, and both are visible behaviour that a test
or a translation depends on, so they are left to their tasks. Locale key counts come from a script over `content/locales/*/ui.json`.
**Not done:** nothing was run in a browser for this check; the CMS was not examined; the matcher crate's name handling was read only
through `rules::duplicates`.

| # | Area | Where | What the code does | Verdict | Task |
| --- | --- | --- | --- | --- | --- |
| 1 | Names | API `src/models/_entities/workers.rs:21`; UI `src/routes/workers/+page.svelte:24` | One `display_name`; no first, last, given or family field, no title, no initials, anywhere in the UI or the worker table | **Right** | — |
| 2 | Names | API `src/rules/duplicates.rs:62-69` | The duplicate check splits `display_name` into given = first word and family = last word for the matcher | **Gap**: wrong for family-name-first orders and for names written without spaces (Chinese, Japanese) | T381 |
| 3 | Names | UI `src/lib/orgViews.ts:45` (and `:41,65,117,141,146`, `src/routes/directory/+page.svelte:44`) | `localeCompare` with no locale, so sorting follows the runtime's collation, not the page's | **Partial** | T381 |
| 4 | Addresses | API `src/rules/contact_details.rs:115-135`; UI `src/lib/components/me/ContactDetailsPanel.svelte:16-22,93` | Two lines, city, region, postcode, country; every field optional; no postcode pattern in the UI | **Partial**: right in what is required, but the order is fixed small-to-large | T382 |
| 5 | Addresses | API `src/rules/contact_details.rs:39-52` | A postcode is up to 12 ASCII letters, digits, spaces or hyphens | **Partial**: fits the codes known to the author; not checked against every country | T382 |
| 6 | Addresses | UI `content/locales/en-001/ui.json:2232-2235` | World English labels are British: "Town or city", "County or region", "Postcode"; the country is a free-text "Country (two letters)" box | **Gap** | T382 |
| 7 | Phone | API `src/rules/contact_details.rs:67-90`; UI `ContactDetailsPanel.svelte:93` (`type="tel"`, no pattern) | 5 to 15 digits, an optional `+`; no country's shape assumed. Only ASCII digits are accepted | **Partial**: no country's shape, but a number typed in Arabic-Indic or Bengali digits is refused | T382 |
| 8 | Identifiers | API `src/models/_entities/workers.rs:11-34`; UI (no field, no pattern) | No national identifier, tax code or insurance number column or field in either; the worker number is the employer's own free text | **Right** (the spec's payroll fields, audit row 4, are not in the code) | — |
| 9 | Dates | UI `src/lib/components/CalendarPanel.svelte:50,54,58,75`, `TrendChart.svelte:28` | `Intl.DateTimeFormat(i18n.locale, …)`; weekday names from `Intl` | **Right** | — |
| 10 | Dates | UI `src/lib/components/me/TimeOffPanel.svelte:90`, `src/routes/groups/+page.svelte:156`, `src/routes/movements/[pid]/+page.svelte:245`, `CareerHistory.svelte:54` (and about 80 more) | Raw ISO dates in markup or placeholders; `CareerHistory` writes an English "now" | **Gap**. **Fixed 2026-10-30** (`src/lib/locale-format.ts`) | T375 |
| 11 | Dates | UI `src/routes/workforce/+page.svelte:69,100` | `toLocaleTimeString([])` / `toLocaleString()`: the browser's locale, not the page's | **Gap**. **Fixed 2026-10-30** (`src/lib/locale-format.ts`) | T375 |
| 12 | Time zone | UI `CalendarPanel.svelte:35`, `src/routes/rota/+page.svelte:46`, `src/lib/ceo.ts:11,87` | "Today" is the UTC date (`toISOString().slice(0, 10)`); there is no user or organization time zone | **Gap**: the wrong day near midnight far from UTC | T378 |
| 13 | Week start | UI `CalendarPanel.svelte:95` (`locale={i18n.locale}` to the calendar, which takes the first day from the locale) | Every `-001` locale resolves to Monday; only `en-us` gives Sunday; no region setting apart from the language | **Partial** | T378 |
| 14 | Week start | API `migration/src/m20261021_000078_time_entry_weeks.rs:32-37`; `src/controllers/workforce.rs:799` | The working-time roll-up groups by `date_trunc('week', …)`, an ISO Monday week, and its partial index is fixed at 2,720 minutes | **Gap** (tied to row 17) | T380 |
| 15 | Weekend | API `src/rules/holiday.rs:128-130,188-192` | `is_weekend` is Saturday and Sunday; a substitute day and `is_business_day` follow it | **Gap**: a Friday and Saturday (or Friday only) weekend cannot be expressed | T379 |
| 16 | Weekend | UI `CalendarPanel.svelte:153` (`td[data-weekend]`, never set: `weekendDays` is not passed) | No weekend is shown in any locale | **Gap** (neutral, but no weekend at all) | T378 |
| 17 | Working time | API `src/rules/working_time.rs:9-23`; `src/controllers/workforce.rs:762,775,895` | 17 weeks, 48 hours and 11 hours of rest are constants; no other limits can be configured | **Gap**: the D144 design's first move; not refactored here (see T369) | T380 |
| 18 | Calendars | API `src/rules/holiday.rs:79-82`; UI (no `calendar` option, no `-u-ca-`) | Gregorian only: Easter by the Gregorian algorithm; no lunar, Hijri, Hindu or other holiday rule; no non-Gregorian display | **Gap** | T379 |
| 19 | Holidays | API `src/rules/holiday.rs:240-290` | Three example packs (GB-ENG, US, DE), labelled unverified; any region can add days by hand | **Right** in shape (data, labelled); the packs are three Western ones | T379 |
| 20 | Currency exponent | API `src/rules/expenses.rs:65-71`, `src/rules/cost.rs:140-145`; `src/rules/display.rs:13-17` (used only for CPD amounts and FTE, not money) | The API holds and returns minor units with a code and never converts to major units, so it needs no exponent | **Right** | — |
| 21 | Currency exponent | UI `src/lib/api/wpm.ts:100,116-117`; input at `ExpenseClaimView.svelte:115`, `src/routes/pay-scales/+page.svelte:93`, `src/routes/planning/+page.svelte:279` | `minor / 100` to show, `Math.round(x * 100)` to store, `moneyWhole` assumes cents | **Gap**: a JPY amount shows at a hundredth of its value, KWD or BHD at ten times; audit row 16 holds for the spec and the API but not the UI. **Fixed 2026-10-30** (`src/lib/money.ts`) | T373 |
| 22 | Currency | UI `src/lib/components/ExpenseClaims.svelte:34`, `src/routes/planning/+page.svelte:74` | `$state("GBP")` as the default currency | **Gap**. **Fixed 2026-10-30**: no starting currency | T376 |
| 23 | Numbers | UI `ExpenseClaimView.svelte:107,211` (`Number(amount)`), `pay-scales/+page.svelte:85` | A decimal comma ("12,50") is not a number | **Gap** for de, fr, es, pt, ru, id and others. **Fixed 2026-10-30** for amounts (`parseAmount`) | T376 |
| 24 | Numbers | UI `src/lib/format.ts:23,66,75,85`, `CapacityHeatmap.svelte:31`, `PlanForecast.svelte:45,58,68,167-172`; `src/lib/i18n.svelte.ts` (`tf`, `tp` use `String(n)`) | Percentages and decimals by hand (`toFixed`, a `.` and `%`); `money()` without a locale; numbers in strings are not localized, so Bengali digits from `Intl` and ASCII digits mix on a page | **Gap**. **Fixed 2026-10-30** (`src/lib/locale-format.ts`), bare counts in markup in part | T375 |
| 25 | Numbers | UI `src/lib/components/LoadMore.svelte:23`, `src/routes/pay-gap/+page.svelte:127` | `Intl.NumberFormat(i18n.locale)` | **Right** | — |
| 26 | Plurals | UI `src/lib/i18n.svelte.ts`; `content/locales/REVIEW.md:50-51` | No `Intl.PluralRules`; 8 en-001 strings use "(s)"; about 40 strings fill a count | **Gap**: Arabic, Russian, Welsh and others have more than two forms. **Fixed 2026-10-30** for the 8 "(s)" strings (`tn`); the other count strings remain | T374 |
| 27 | Plurals | UI `src/lib/components/FrameworkRolePanel.svelte:315` | `changes2: changes.length === 1 ? "" : "s"` appends an English "s" in every locale | **Gap (a bug)**. **Fixed 2026-10-30** (`tn`) | T374 |
| 28 | Right-to-left | UI `src/hooks.server.ts:64-65`, `src/routes/+layout.svelte:200-201`, `src/lib/locales.ts` (`RTL_LANGUAGES = ["ar", "ur"]`) | `dir` set on the server and the client for `ar-001` and `ur-001`; an end-to-end test covers `ar` only | **Right**; `ur` untested. **Tested 2026-10-30** in `ar` and `ur` (`right-to-left.spec.ts`) | T377 |
| 29 | Right-to-left | UI `src/app.css:128,182,191-206`, `CalendarPanel.svelte:203-204`, `src/routes/+page.svelte:369` | 11 physical declarations (against 27 logical); `text-align: left` on every table cell | **Gap**. **Fixed 2026-10-30**: all logical, and a scan fails on a new one | T377 |
| 30 | Right-to-left | UI `TimeOffPanel.svelte:90`, `OnCall.svelte:59`, `metrics/+page.svelte:74` (and about 12 more, plus 8 catalogue strings) | A literal "→" between the ends of a range points the wrong way in `ar` and `ur` | **Gap**. **Fixed 2026-10-30**: `formatDateRange` and `arrow()` | T377 |
| 31 | Locales | UI `content/locales/*/ui.json`, `src/lib/locales.ts` | 17 locales; all 12 translated `-001` files hold all 2,338 keys of `en-001`; `en-001` (not `en-gb`) is the default; all translations are AI-written and unreviewed (`REVIEW.md`) | **Right** in parity; **partial**: no locale is reviewed | — (recorded) |
| 32 | Locales | UI `content/locales/en-gb/ui.json` (5 overrides), `en-us/ui.json` (0) | `en-001` is written in British wording, so `en-gb` barely differs and `en-us` has nothing to override | **Partial** | T382 |
| 33 | Pay arithmetic | API `src/rules/payroll.rs:42-47`, `src/rules/workforce.rs:10` | The stub tax (20% above 104,750 minor units a month, the UK allowance divided by 12), a 7.5-hour day and 9,750 minutes a month, applied to every worker whatever the currency | **Gap** (labelled a stub; production gate WPM-G2) | T383 |
| 34 | Statutory defaults | API `src/rules/flexible_working.rs:13-17`, `src/rules/resignation.rs:13`, `src/rules/pay_information.rs:17`, `src/rules/joint_assessment.rs:17-19`, `src/rules/privacy.rs:85-97` | Each rule takes its limit as a parameter, but the default when no `WPM_*` setting is given is one regime's value (2 months, 2 requests, 14 calendar days; 28 calendar days; 2 months; 5% and 6 months) and is not labelled as whose | **Partial**: parameterized, one deployment-wide value | T384 |
| 35 | Statutory defaults | API `src/controllers/pay_transparency.rs:442`, `src/rules/pay_gap.rs:540-566` | `tier_for` takes the tiers, but the controller always passes the EU Directive's tiers | **Partial** | T384 |
| 36 | Pay gap | UI `src/routes/pay-gap/+page.svelte:132-133`; API `WPM_PAY_GAP_SEX_CATEGORY` | Female and male columns, the shape of the EU and UK reports; the equality categories themselves come from the server (`EqualityPanel.svelte:62`) | **Partial** | T384 |
| 37 | Server content | API `src/rules/ergonomics.rs:6-16`, `src/controllers/ergonomics.rs:52` | The default workstation checklist is eight English sentences stored as data, "UK Display Screen Equipment shape" | **Gap**: not translatable, one regime's list | T386 |
| 38 | Server content | API `src/rules/cutover.rs:49-59`, `src/rules/insights.rs:72-76` | English text for an API reader plus a code the UI translates | **Right** | — |
| 39 | Seed data | API `src/tasks/seed.rs:61,100,150,175,190`; root `scripts/demo-data.py:281` | Every salary, payslip and expense claim is in GBP; names are "Test Worker 001" and "Fictitious … Worker" (neutral); no holiday pack and no example jurisdiction profile is loaded | **Gap**: the demo has one currency (exponent 2) and one country's money; D144 question 1 (confirmed) has the seed load example profiles, which do not exist yet | T385 |

### What the check found, in short

- **39 findings: 9 right, 9 partial, 21 gaps.** The worst are the UI's two-decimal money (row 21), the absent
  plural rules with a bug that writes an English "s" into Arabic (rows 26, 27), and the weekend fixed at Saturday and Sunday in the
  holiday rules (row 15).
- **Audit row 16 is corrected in part:** the API keeps minor units and never assumes an exponent, but the UI does, on display and on input.
- **What is already right:** one display name, no national identifier anywhere, phone and address fields with no country's shape required,
  `Intl` dates in the calendar, `dir` for both right-to-left locales, locale parity, and server text that carries a code to translate.

## Not done

Nothing is implemented here (T367 audit, T368 design, T371 check). This is the requirement, the decisions, the tasks, the audit (T367)
and the equity check (T371). Since 2026-10-30 the UI's money (T373, T376), its numbers and dates (T375), its plural rules (T374) and the pure-core checks of T369 are code; their entries
above say what each did and did not do.
