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
- [ ] WPM-T369 **Second-profile tests.** *(WPM-R214, WPM-D143)* For each rule with UK values, a pure-core test with a non-UK profile of a
  different shape (different currency exponent, weekend, week start, limits), and a lint or test that fails if `src/rules/` names a
  country.
- [ ] WPM-T370 **Regulatory register by region.** *(WPM-R214, WPM-D143)* Re-organize [regulatory.md](regulatory.md) so the UK is one
  section among regions, with the others marked *not yet read* rather than absent.
- [ ] WPM-T371 **Locale and convention equity check.** *(WPM-R214, WPM-D143)* Names, addresses, dates and calendars, numbers and
  currency, identifiers, right-to-left and plural rules: a checklist run against the UI and API, with gaps recorded as tasks.

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

## Not done

Nothing is implemented here (T367 audit, T368 design). This is the requirement, the proposed decision, the tasks and the audit (T367).
