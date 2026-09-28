# Governance

This project uses **sole-maintainer governance** (sometimes called
BDFL): Joel Parker Henderson (see [MAINTAINERS.md](MAINTAINERS.md))
holds final decision authority over scope, design, releases, and
membership. AI tooling is enabled to generate code, tests, and
documentation under maintainer direction (see
[AI_STATEMENT.md](AI_STATEMENT.md) §5–§6) but does not merge, decide
a specification-facing question, or make a release decision — those
stay the maintainer's.

## How decisions are made

- **Design decisions** are recorded in the specification —
  [spec/design.md](spec/design.md) (`WPM-D*`), lock-step with
  [spec/requirements.md](spec/requirements.md) (`WPM-R*`) and the live
  delivery checklist [spec/tasks.md](spec/tasks.md) (`WPM-T*`). A
  decision doc supersedes discussion; re-litigating a recorded
  decision requires new evidence.
- **Regulatory and scope adjudications** — e.g. what the demo-software
  boundary means for a given feature — are recorded in
  [spec/regulatory.md](spec/regulatory.md) and
  [spec/scope.md](spec/scope.md).
- **Working agreements** for contributors and agents are in each
  subproject's `AGENTS.md` and in [CONTRIBUTING.md](CONTRIBUTING.md).

## Changing this document

By pull request, decided by the maintainer.
