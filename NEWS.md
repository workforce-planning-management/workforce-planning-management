# News

Project news and milestones. Detailed change history lives in each
subproject's `CHANGELOG.md`; the repo-level summary is
[CHANGELOG.md](CHANGELOG.md).

## Press and contact

Joel Parker Henderson — <joel@joelparkerhenderson.com>.

## Funding

No dedicated funding channel is set up for this repository at
present. See [CONTRIBUTING.md](CONTRIBUTING.md#funding).

## Milestones

- **2026-09-28** — AI tooling authorized to merge pull requests and
  publish already-decided releases, against explicit checklists
  ([AI_STATEMENT.md](AI_STATEMENT.md) 1.1.0); both subprojects made
  publish-eligible (front-end no longer `private`; service's
  `Cargo.toml` `repository` field corrected).
- **2026-09-28** — Root special files added per
  [spec/special-files-for-public-repos](spec/special-files-for-public-repos/index.md):
  README, LICENSE.md, CITATION.cff, this file, COMPARISONS.md,
  BENCHMARKS.md, INSTALL.md, CONTRIBUTING.md, RFC.md, CODEOWNERS,
  MAINTAINERS.md, CHANGELOG.md, AI_STATEMENT.md, GOVERNANCE.md,
  SECURITY.md.
- **2026-08-02** — Service moved onto loco-rs 1.0.1; every deployable
  service given its own containerized test database.
- **2026-07-25** — **WPM-T18–T36 complete**: both subprojects reach
  "implemented" status — auth activation surface, subject rights &
  retention, 360° appraisals + rater self-service + notifications,
  the anonymous wellbeing pulse, working-time guardrails,
  benefits-awareness, ergonomic (DSE) assessments, cognitive testing,
  and reasonable adjustments all land, followed by a documentation
  harmonization pass, `cargo fmt`, and `clippy::pedantic` across every
  crate, and CI pipelines for both git remotes.
- **2026-07-24** — Renamed from Human Capital Management (HCM) to
  Workforce Planning Management (WPM).
- **2026-07-18** — Project starts: cross-cutting specification round
  (WPM-T0) and the service skeleton.
