# Roadmap

Beyond the delivered queue ([tasks.md](tasks.md), WPM-T1–T89). Items are
grouped by whether a decision, a dependency or just time stands in the way.

## Deferred by decision

- **Employee expense claims** — the one table-stakes gap in the open-source HR
  comparison (frappe/hrms, OrangeHRM and Odoo all ship it). Deferred
  2026-10-05; a draft → submitted → approved/rejected → reimbursed state
  machine in `rules/expenses.rs` plus a migration is the natural start.

## Next candidates (from the benchmark scan, `.sota/last-scan.json`)

- Grievance / helpdesk ticketing; job-offer terms (the requisition has an
  `offer` state but no offer document); employee referrals; a mobile app or
  PWA with push; geofenced / biometric clock-in; swap-style requests for other
  rota-like bookings.

## Follow-ups on delivered work

- **Notifications:** outbound delivery (email/push over the upstream person
  service's contact details — in-app is delivered; WPM-D23 keeps contact details
  out of WPM); a reminder for unread announcements; a reminder when a leaver's
  last day is near.
- **Skills and training:** gaps against a *next* role (promotion readiness);
  ESCO role requirements; trends over time; course prerequisites, cost and
  availability; the upstream course catalogue; booking a place.
- **Joiners and leavers:** requisition and appraisal ownership in the handover;
  automatic start of the handover on the last day; returning to work.
- **Dashboards:** drill-down to the underlying rows; a date-range control on the
  trend line; other screens sized for other devices.
- **Localization:** a native-speaker review of every translation; exercising the
  Sveltia CMS end to end with a GitHub login; a `<repo>.github.io` site (see
  [monorepo-github-pages](monorepo-github-pages/index.md) — not yet created).
- **Auth:** the Keycloak and enforcement suites are not yet in CI.

## Still on the original list

- **`employed_by` edge emission** — write the registry's cross-service link on
  hire/termination ([integrations.md](integrations.md)).
- **Payroll payment execution** — bank-file (BACS/SEPA) export; real statutory
  tax tables per jurisdiction.
- **Vendor integrations** — e-signature, background-check and job-board
  providers behind the onboarding/requisition checklists.
- **Compensation reviews** — salary-change workflows with approval chains.
- **Multi-org / group payroll** — consolidated reporting for a corporate group.
- **WPM ↔ PPM bridge** — allocations in project-portfolio-management
  referencing WPM employees for capacity-vs-contract checks.
- **Adjustment review cadence**, **DSE re-assessment scheduling**,
  **360 → development-plan linking**.

## Delivered since the original roadmap

People analytics over the arithmetic dashboards (WPM-T44 metrics, T69
insights, T83–T85 CEO dashboard), the on-call rota, the employee directory,
skills gap analysis and training time, joiners and leavers, and the
announcement feed.
