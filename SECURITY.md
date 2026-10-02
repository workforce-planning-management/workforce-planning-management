# Security policy

## Reporting a vulnerability

Email <joel@joelparkerhenderson.com> with a description of the issue,
the affected subproject, and reproduction steps. Please do not open a
public issue for a vulnerability. You should receive an
acknowledgement within a few business days.

## Scope

Both subprojects: `workforce-planning-management-service-with-rust`
(the back-end JSON API) and
`workforce-planning-management-ui-with-svelte` (the browser
client).

## What deployers must know

**The shipped default is wide open.** Authentication and authorization
in the service are gated by `WPM_REQUIRE_AUTH`, which defaults **off**
(family convention). A deployment exposed to untrusted callers MUST
set `WPM_REQUIRE_AUTH=1`, configure `WPM_PASETO_KEYS[_URL]`, and mount
a real ABAC policy before it is reachable. See
[spec/auth.md](spec/auth.md) for the activation steps and
[spec/regulatory.md](spec/regulatory.md) for what production would
additionally require (this is demo software; see that file's "Demo
software" note).

This is **synthetic-data demo software**: no real personal data
belongs in the repository, its seeds, or any deployment used for
demonstration. See [spec/regulatory.md](spec/regulatory.md).

## Hardening in place

- No `unsafe` in the service crate (`#![forbid(unsafe_code)]`).
- Salary, payslips, review content, 360 reports, assessment scores,
  adjustment words, and succession plans are masked under ABAC and
  their reads are audited — never logged. See
  [spec/auth.md](spec/auth.md) and [spec/audit.md](spec/audit.md).
- What must not be stored gets no column: no health cohort, symptom,
  diagnosis, or pulse-author column exists (WPM-D17/D20/D24/D25).
- The BFF front-end holds the session cookie server-side and exchanges
  it for short-lived PASETO tokens — no token or credential reaches
  browser JS or localStorage.

## Publishing

Neither subproject has a released version on a registry yet, but both
are publish-eligible: the service crate declares `publish = true` in
`Cargo.toml`, and the front-end package no longer sets `"private": true`
in `package.json`. Per [AI_STATEMENT.md](AI_STATEMENT.md) §5/§6, AI
tooling may run `cargo publish` / `npm publish` for an already-merged,
already-decided version bump once the §6 checklist is met. Whichever
runs it — maintainer or AI — does so against a locally stored
credential (a crates.io API token, an npm auth token): the same class
of long-lived secret this policy's "no secret in logs, no usable
default outside dev" posture exists to minimize. There is no CI-driven
publish step today, so no registry token is held in CI configuration.
