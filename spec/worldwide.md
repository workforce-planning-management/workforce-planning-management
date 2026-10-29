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

## Tasks

- [ ] WPM-T367 **Worldwide audit of the spec.** *(WPM-R214, WPM-D143)* Read the spec and the register for statements that assume the UK
  or US (statutory language without a jurisdiction, UK-only examples treated as the model); list each, scope or reword it, and record
  what is a worked example. Output: a table in this file.
- [ ] WPM-T368 **Jurisdiction profile model.** *(WPM-R214, WPM-D143)* A design for per-organization, per-layer (national, regional,
  supranational) profiles with effective dates, and what each existing rule (working-time guardrails, retention, pay transparency,
  equality categories) reads from it. Design first; no code.
- [ ] WPM-T369 **Second-profile tests.** *(WPM-R214, WPM-D143)* For each rule with UK values, a pure-core test with a non-UK profile of a
  different shape (different currency exponent, weekend, week start, limits), and a lint or test that fails if `src/rules/` names a
  country.
- [ ] WPM-T370 **Regulatory register by region.** *(WPM-R214, WPM-D143)* Re-organize [regulatory.md](regulatory.md) so the UK is one
  section among regions, with the others marked *not yet read* rather than absent.
- [ ] WPM-T371 **Locale and convention equity check.** *(WPM-R214, WPM-D143)* Names, addresses, dates and calendars, numbers and
  currency, identifiers, right-to-left and plural rules: a checklist run against the UI and API, with gaps recorded as tasks.

## Not done

Nothing is implemented here. This is the requirement, the proposed decision and the tasks; the audit (T367) comes first.
