# Governance

This project uses **sole-maintainer governance** (sometimes called
BDFL): Joel Parker Henderson (see [MAINTAINERS.md](MAINTAINERS.md))
holds final decision authority over scope, design, releases, and
membership — with two narrow, explicit delegations, both per
[AI_STATEMENT.md](AI_STATEMENT.md) §5/§6: the project's AI tooling may
merge a pull request into `main` once it clears an explicit checklist
(discipline followed, gates green, evidence documented, no unresolved
specification-facing question), and may judge an already-merged,
already-decided version bump ready to publish (`cargo publish` for the
service, `npm publish` for the front-end) and execute that publish.
What a change contains — scope, design, a release's content — stays a
maintainer decision, decided in the pull request itself before either
checklist is ever consulted; the checklists govern only whether an
already-decided change is merged or released. **One boundary on the
merge delegation:** a pull request changing this document,
[AI_STATEMENT.md](AI_STATEMENT.md), or [MAINTAINERS.md](MAINTAINERS.md)
is merged by the maintainer, not by AI — the delegation cannot expand
itself.

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
