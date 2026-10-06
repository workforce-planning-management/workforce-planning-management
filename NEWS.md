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

- **2026-10-07** — **Expense claims** (WPM-T97): the one table-stakes gap from the
  benchmark is closed. A claim of dated, categorised items goes draft → submitted →
  approved or rejected → reimbursed, and **nobody decides their own** — a separate
  enforcement test pins it. Draft and submitted claims are cancelled on erasure; amounts
  stay, words go. With it: the NHS Agenda for Change pay scale for Wales, Google's
  technical levels, grades on roles, a worker's pay band and step with eligibility
  reminders (WPM-T92–T96), a lint pass across the repo, and a visual review of the new
  pages (WPM-T90, T95). No open deferrals remain.
- **2026-10-06** — **Skills gap analysis, training time recommendations, and
  joiners and leavers** (WPM-T87–T89): gaps ranked by importance × levels short
  (an undeclared skill is unknown, never a number), a training plan that says
  whether its hours come from catalogue courses or an estimate, and dated
  joiner/leaver checklists with a last-day handover and audit trail.
- **2026-10-05** — The **people-and-cover wave** (WPM-T74–T86), prompted by a
  benchmark against open-source HR platforms: the employee directory, emergency
  contacts, backups, the on-call rota with swaps and reminders, announcements
  with audiences, links and read counts, and a **one-screen CEO dashboard**
  sized for an iPad (9th generation, 2160 × 1620 px). Employee expense claims
  were deliberately deferred.
- **2026-10-04** — **Localization** (WPM-T70–T72): 17 content locales served
  under `/<locale>/` routes with `/en/`-style aliases, strings as content edited
  through Sveltia CMS; workforce **insights** localized from code + figures. The
  Keycloak backend was run for the first time against a real Keycloak 26 and
  two defects were fixed (WPM-T73).
- **2026-10-02 → 2026-10-03** — **Strategic workforce planning and
  capability frameworks** (WPM-T41–T68): workforce plans and forecast, the
  metrics layer, role profiles, UK GDAD PCF and ESCO imports, career history,
  aspirations, reporting lines, groups and organization transfers.
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
