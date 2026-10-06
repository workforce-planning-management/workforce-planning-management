# Changelog

Repo-level change summary. **Each subproject's `CHANGELOG.md` is the
authoritative, detailed history for that subproject** (Keep a
Changelog format); this file records repo-wide events only. Milestone
narrative lives in [NEWS.md](NEWS.md).

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

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
