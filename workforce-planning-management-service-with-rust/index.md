# Documentation index

Workforce Planning Management — Loco edition. JSON-only back-end API
for the employment lifecycle: hiring, workforce (incl. working-time
guardrails), HR core (incl. wellbeing, pulse, ergonomics,
adjustments, notifications, subject rights), talent development
(incl. 360°s and five assessment categories), payroll.

## Start here

- **[README.md](README.md)** — what this is, status, the surface, a
  worked curl tutorial, and the auth-activation pointer.
- **[../spec/](../spec/index.md)** — the cross-cutting specification
  (single source of truth: domain model, the five pillars, the newer topic
  files, auth, audit, requirements WPM-R1–R216, design decisions WPM-D1–D147).
- **[spec/](spec/index.md)** — this edition's stack-specific spec
  (layout, env vars, masking/ownership/privacy mechanics, gotchas).
- **[AGENTS.md](AGENTS.md)** — working agreements for contributors
  (incl. the unrepresentability ground rule).
- **[CHANGELOG.md](CHANGELOG.md)** — Keep a Changelog format.
- **[config/abac-policy.reference.json](config/abac-policy.reference.json)**
  — the shipped, matrix-verified persona policy; runbook in
  [../spec/auth.md](../spec/auth.md).

## Running

```bash
cargo run -- db migrate && cargo run -- task seed && WPM_REQUIRE_AUTH=0 cargo run -- start
cargo test --lib                    # 1,034 DB-free unit tests (2026-10-30)
cargo test --test mod -- --ignored --test-threads=1   # 184 request tests (Postgres; serial)
cargo test --test enforcement -- --ignored --test-threads=1   # persona matrix
cargo loco scheduler --config config/scheduler.yaml --tag wpm   # the scheduled tasks (INSTALL.md)
```

## The task queue

Live delivery checklist: [../spec/tasks.md](../spec/tasks.md)
(WPM-T1 to WPM-T396 as of 2026-10-30, with the open and part-done ones
listed in [../spec/implementation-status.md](../spec/implementation-status.md)'s
counts; production gates WPM-G1/G2 `[~]` — code complete, operational/legal
work remains). The bridges to the sibling apps:
[../docs/tutorials/family-bridges.md](../docs/tutorials/family-bridges.md).
