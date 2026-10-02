# Contributing

Thank you for your interest. Contributions of code, documentation,
issue reports, and feedback are welcome.

## Ground rules

- **Spec-driven development.** [spec/](spec/index.md) at the repo root
  is the single source of truth, shared by both subprojects; each
  subproject's own `spec/` adds stack-specific detail only. A
  behavioural change is a **three-part change** — spec edit + code
  edit + test edit — landed together, with the subproject's
  `CHANGELOG.md` updated in the same pull request. See each
  subproject's `AGENTS.md` for the full working agreements.
- **Green gate.** Before submitting, on any crate/package you touched:
  - Service (`workforce-planning-management-api-with-rust`):
    `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
    `cargo test` (DB-free unit tests), and `cargo test -- --ignored`
    where you have Postgres available.
  - Front-end (`workforce-planning-management-ui-with-svelte`):
    `pnpm check` (svelte-check), `pnpm lint`, `pnpm test`, and
    `pnpm exec playwright test` where practical.
- **Branch per change.** Branch from `main`, merge back with
  `--no-ff`; never commit directly to `main`.
- **No real personal data, ever** — fixtures and examples are
  synthetic. See [spec/regulatory.md](spec/regulatory.md).
- **Unrepresentability is load-bearing** (WPM-D17/D20/D24/D25): if a
  spec says a field must not be stored or an aggregate must not be
  disclosed, a PR does not add a column or an endpoint that would.

## How to contribute

1. Open an issue or email <joel@joelparkerhenderson.com> describing
   the change, especially before large work.
2. Find the owning subproject's task queue —
   [spec/tasks.md](spec/tasks.md) (`WPM-T*`, traced to `WPM-D*`/`WPM-R*`)
   — most welcome work is already enumerated there.
3. Submit the change with the three parts and the green gate evidence
   in the pull-request description.

## Contributor expectations for AI tooling

See [AI_STATEMENT.md](AI_STATEMENT.md) — it binds contributors as well
as the maintainer. That includes who may merge a change and who may
decide a release is ready and publish it: AI tooling may merge a pull
request once it clears an explicit checklist (the discipline for its
kind of change is followed, the green gate above passes, the evidence
is documented, no unresolved specification-facing question is raised),
and may judge an already-merged version bump ready and run the
registry publish directly — neither without a maintainer-decided *what*
behind it ([AI_STATEMENT.md](AI_STATEMENT.md) §5/§6,
[GOVERNANCE.md](GOVERNANCE.md)). A contributor's own change still lands
the same way as any other: spec + code + test, the same green gate,
merged once that checklist is met — by the maintainer, or by AI acting
on it. A contribution with AI-generated content shall say so in the
pull-request description: which tool, and what it did. The contributor
remains responsible for their submission in full: understood,
explained on request, tested, and honest.

## Funding

There is no dedicated funding channel for this repository at present.
The most useful contribution remains code, review, or a well-written
issue.
