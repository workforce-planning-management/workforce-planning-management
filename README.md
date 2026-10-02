# Workforce Planning Management (WPM)

An **all-in-one HR platform** demonstrating the employee lifecycle
hire to retire: talent acquisition & onboarding, workforce management
(time, leave, scheduling), HR service delivery (employee record, org
chart, self-service, benefits, wellbeing, ergonomics, subject
rights), talent management & development (reviews, 360°s, training,
succession), and payroll & compensation. See
[spec/index.md](spec/index.md) for the full specification.

> ⚠️ **Demo software.** Not a production HR or payroll system;
> statutory calculations are illustrative stubs; no real personal
> data anywhere in the repository. See
> [spec/regulatory.md](spec/regulatory.md).

**Status: implemented (WPM-T1–T36, 2026-07-18 → 2026-07-25).** See
each subproject's own README for its own test counts and gate status.

## Subprojects

| Subproject | Role | Stack |
| --- | --- | --- |
| [workforce-planning-management-service-with-rust](workforce-planning-management-service-with-rust/) | Back-end JSON API | Rust, Loco (Axum + SeaORM), PostgreSQL |
| [workforce-planning-management-ui-with-svelte](workforce-planning-management-ui-with-svelte/) | HR / manager / self-service UI | SvelteKit 2, Svelte 5 runes, TypeScript |

Each subproject is self-contained: it owns its own `README.md`,
`AGENTS.md` (working agreements), `CHANGELOG.md`, and `spec/`
(stack-specific detail; the cross-cutting specification lives at the
repo root in [spec/](spec/)).

WPM is a **consumer application**: it does not register identities
itself. A person is a `person-service` record, their professional
identity a `worker-service` record, the employer an
`organization-service` record; training courses live in
`course-service`. WPM owns only the employment relationship and its
operational state, referencing identities by `EntityRef` URN, never
duplicating them. See [spec/scope.md](spec/scope.md) and
[spec/integrations.md](spec/integrations.md).

## Running

```sh
# Back-end
cd workforce-planning-management-service-with-rust
cargo run -- db migrate && cargo run -- task seed && cargo run -- start

# Front-end (in another shell)
cd workforce-planning-management-ui-with-svelte
pnpm install && pnpm dev
```

See [INSTALL.md](INSTALL.md) for prerequisites and the full build/test
commands.

## Specification-driven delivery (SDD)

Three lock-step files drive delivery: [spec/requirements.md](spec/requirements.md)
(`WPM-R*`), [spec/design.md](spec/design.md) (`WPM-D*`), and
[spec/tasks.md](spec/tasks.md) — the live delivery checklist (`WPM-T*`).
A change starts in `requirements.md`, is shaped in `design.md`, is
queued in `tasks.md`, and only then lands as code. **No code lands
without the spec describing it.**

## Documentation

- [spec/index.md](spec/index.md) — the cross-cutting specification
  (purpose, scope, domain model, the five pillars, auth, audit,
  architecture, testing, regulatory posture, roadmap, glossary).
- [CONTRIBUTING.md](CONTRIBUTING.md) — ground rules and how to
  contribute.
- [AI_STATEMENT.md](AI_STATEMENT.md) — how AI tools are used to
  develop this software.

## License

See [LICENSE.md](LICENSE.md). Each subproject declares its own SPDX
license expression in its manifest.
