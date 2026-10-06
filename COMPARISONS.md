# Comparisons

Where Workforce Planning Management (WPM) sits relative to adjacent
kinds of systems. These are orientation notes, not benchmarks or
feature-war tables.

## All-in-one HRIS platforms (Workday, SAP SuccessFactors, BambooHR)

Commercial HRIS suites cover similar ground — ATS, core HR, time &
leave, performance, learning, succession, payroll — as one integrated
platform. WPM follows the same five-pillar shape but differs by being
**a consumer application over a federated identity family**: it does
not own identity itself (person/worker/organization/course are
separate services, referenced by `EntityRef` URN), it is open source,
implemented in Rust end to end, and spec-driven — every behaviour
traces to a numbered requirement, design decision, and task. See
[spec/index.md](spec/index.md).

## Open-source HR platforms (frappe/hrms, OrangeHRM, Horilla, Odoo HR)

A scan on 2026-10-05 (recorded in `.sota/last-scan.json`) checked what these
projects ship, from their code and docs rather than their READMEs:
[frappe/hrms](https://github.com/frappe/hrms) (~8.9k stars),
[OrangeHRM](https://github.com/orangehrm/orangehrm) (~1.1k),
[Horilla](https://github.com/horilla-opensource/horilla) (~1.5k) and the `hr_*`
addons of [Odoo](https://github.com/odoo/odoo) (~55k).

- **Table stakes WPM matches:** the employee record and org chart, leave, time and
  attendance, payroll, recruitment, onboarding and offboarding, performance,
  learning, and headcount/staffing plans (frappe's `staffing_plan` is the
  counterpart of WPM's workforce plans).
- **One table-stakes gap, deliberately deferred:** employee **expense claims** —
  shipped by frappe/hrms (`expense_claim`), OrangeHRM (`orangehrmClaimPlugin`) and
  Odoo (`hr_expense`).
- **Edge features seen elsewhere:** grievances and helpdesk (frappe, Horilla),
  geofenced or biometric check-in (Horilla), a social feed and company directory
  (OrangeHRM `Buzz` / `CorporateDirectory` — WPM now has a directory and an
  announcement feed), job-offer terms and referrals (frappe), a mobile app or PWA
  (OrangeHRM, frappe). See [spec/roadmap.md](spec/roadmap.md).
- **Where WPM differs:** it is a *consumer application over a federated identity
  family* (not its own user registry); sensitive data is **unrepresentable or
  restricted by construction** (no diagnosis column, no pulse author, emergency
  contacts visible only to the person and HR); every roll-up names no one and an
  unknown is never shown as zero; and it ships a skills-gap and training-time
  capability built on role requirements, which this scan did not compare.
  This is orientation, not a feature-war table: those projects cover far more
  integrations, localizations and deployments than a demo.

## Point solutions (ATS-only, payroll-only, LMS-only tools)

Point solutions (Greenhouse/Lever for ATS; Gusto/ADP for payroll;
standalone LMS products) each do one pillar well but leave the
handoffs between them to integration work or spreadsheets — exactly
the scatter [spec/purpose.md](spec/purpose.md) names as the problem
WPM addresses. WPM trades per-pillar depth for one operational system
across the whole employee lifecycle, hire to retire.

## The wider identity/registry family

WPM is one of several **consumer applications** built on the same
federated identity registries (person, worker, organization, course
services), alongside siblings such as case-folder and
patient-flow. What distinguishes WPM within that family is its domain:
the employment relationship and its operational state, not clinical
or case-management state. See
[spec/integrations.md](spec/integrations.md).

## Context

If you know of a system this should be compared against, see
[RFC.md](RFC.md) — that feedback is explicitly wanted.
