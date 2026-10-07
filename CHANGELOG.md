# Changelog

Repo-level change summary. **Each subproject's `CHANGELOG.md` is the
authoritative, detailed history for that subproject** (Keep a
Changelog format); this file records repo-wide events only. Milestone
narrative lives in [NEWS.md](NEWS.md).

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

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

### Added — NHS Agenda for Change pay scale for Wales (WPM-T92)

The 2026/27 Wales scale from pay letter AfC(W) 02/2026, with a salary-placement and
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
