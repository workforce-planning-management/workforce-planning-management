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
  files, auth, audit, requirements WPM-R1–R50, design decisions WPM-D1–D37).
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
cargo run -- db migrate && cargo run -- task seed && cargo run -- start
cargo test                          # 297 DB-free unit tests
cargo test -- --ignored             # 60 request tests in 23 files (Postgres; serial)
cargo test --test enforcement -- --ignored   # persona matrix
cargo run -- task rota_reminders    # schedule daily; also snapshot_headcount and pay_progression_reminders
```

## The task queue

Live delivery checklist: [../spec/tasks.md](../spec/tasks.md)
(WPM-T1–T89 delivered, one deliberate deferral — employee expense claims;
production gates WPM-G1/G2 `[~]` — code complete, operational/legal work
remains).
