# Changelog

Repo-level change summary. **Each subproject's `CHANGELOG.md` is the
authoritative, detailed history for that subproject** (Keep a
Changelog format); this file records repo-wide events only. Milestone
narrative lives in [NEWS.md](NEWS.md).

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

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
