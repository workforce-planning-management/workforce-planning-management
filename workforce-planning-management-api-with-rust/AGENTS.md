# AGENTS.md — working agreements

A pocket guide for human and AI collaborators working in this
subproject. Read this **before** opening a PR. (Updated 2026-10-06,
through WPM-T97.)

## What this project is

A **back-end JSON API**, written in Rust on [Loco](https://loco.rs)
(Axum + SeaORM + PostgreSQL), for workforce planning management across
the whole employment lifecycle:

- **Hiring and the worker record** — requisitions, the applicant pipeline,
  onboarding, worker records, org chart, reporting lines (solid and dotted),
  groups, organization memberships and transfers.
- **Workforce** — time & attendance, leave, shifts with advisory working-time
  guardrails, the **on-call rota** (swaps, swap requests, reminders), backups
  (cover), emergency contacts, the **employee directory**.
- **People development** — reviews, 360° appraisals, assessments, skills and
  learning paths, mentorships, CPD, role profiles and capability frameworks
  (UK GDAD PCF, ESCO), **skills gap analysis** and **training time
  recommendations**, succession, talent pipelines.
- **Strategy and leadership** — workforce plans, forecast and gap analysis,
  the metrics layer, **insights**, headcount snapshots, announcements, the data
  behind the **CEO dashboard**.
- **Joiners and leavers** — dated checklists and a last-day **handover** with an
  audit trail.
- **Wellbeing and rights** — wellbeing prompts, the anonymous pulse, ergonomics,
  reasonable adjustments, subject access / erasure / retention.
- **Payroll** — runs, payslips, benchmarking.

There is no built-in UI — the
[Svelte sibling](../workforce-planning-management-ui-with-svelte/) is the HR /
manager / self-service client. The full picture is the cross-cutting spec,
[`../spec/index.md`](../spec/index.md).

**Domain ownership.** WPM **owns the employment relationship and its
operational state** (its own tables) but **references identities**: humans are
person-service records, professional identities worker-service, employers
organization-service, training courses course-service — always as EntityRef
URNs (`person:<uuid>`), never duplicated demographics
([scope boundary](../spec/scope.md)).

> ⚠️ Demo software, not a production HR/payroll system. Synthetic data
> only. See [regulatory](../spec/regulatory.md).

## Ground rules

1. **Spec first.** [`../spec/`](../spec/index.md) is the single source of
   truth; this subproject's `spec/` adds stack detail only. A behavioural
   change is **requirement (`WPM-R*`) + design decision (`WPM-D*`) + task
   (`WPM-T*`) + code + tests**; the task entry records what was verified and
   what was *not*. The live queue is [`../spec/tasks.md`](../spec/tasks.md).
2. **Family conventions.** Loco layout (`src/controllers/`),
   `#![forbid(unsafe_code)]`, `#![warn(clippy::pedantic)]`, tracing + OTLP,
   OpenAPI/Swagger (`src/openapi.rs` — add every new route), header API
   versioning, **Podman not Docker**, **PostgreSQL not SQLite**, in-memory
   cache. See [rust-loco-stack](../../agents/share/rust-loco-stack.md).
3. **Pure core first.** Rules live in DB-free, clock-free `src/rules/<area>.rs`
   with exhaustive unit tests (dates and "today" are always supplied);
   controllers only load data, call the rule and write. Write and test the rule
   module **before** its controller. Existing modules: `lifecycle`, `leave`,
   `working_time`, `org`, `metrics`, `insights`, `directory`, `emergency`,
   `cover`, `rota`, `announcements`, `skill_gap`, `training`, `movements`, …
4. **Money discipline.** Minor units (`i64`) + ISO-4217; overflow refused;
   `net = gross − Σ deductions` before persist. No floats for money.
5. **Sensitive data.** Salary, payslips, review content, 360 reports,
   assessment scores, adjustment words, succession plans and emergency contacts
   are masked or restricted (ABAC `mask`, or person-and-HR-only) and their reads
   audited ([auth](../spec/auth.md), [audit](../spec/audit.md)). Never log
   them, and never put a sensitive detail in an audit snapshot.
6. **Unrepresentability and honesty are load-bearing** (WPM-D17/D20/D24/D25,
   extended by D29–D36): no health cohort, symptom, diagnosis or pulse-author
   column; no aggregate over adjustment requests; **unknown is not a number**
   (an undeclared skill has no shortfall — return `null`/a distinct status,
   never `0`); workforce roll-ups return **counts and name no one**; a view
   that cannot answer says so ("nobody available") instead of guessing; every
   estimate states what it rests on.
7. **A new table that holds personal data must be wired into rights.**
   Soft-deleting → add to `rules::privacy::SOFT_DELETED_TABLES` (sorted; the
   count pin in its test fails otherwise); person-keyed → add to the
   subject-access export and the erasure statements in
   `controllers/privacy.rs` (append statements at the **end** — the audit
   snapshot indexes them by position).
8. **Notifications and audit.** Notification kinds are a closed list
   (`rules::notify::KINDS`, pinned by a test): in-app, reference-only (names and
   dates, never availability or leave reasons). Every mutation writes an audit
   row; the actor comes from `caller.actor()`.
9. **Authorization patterns.** Record-level: `auth::authorize_record(&caller,
   Action::{Read|Write}, &auth::worker_resource_attrs(&worker))` — *Write* is
   "the person themself or HR". Organization scope:
   `memberships::scope_organization_refs` (`None` = auth off = unrestricted) and
   `memberships::has_role_in` for editor roles. A thing outside the caller's
   organizations is a **404**, not a 403.
10. **Known gotchas.** loco `create_table` pluralizes names (use plural names /
    explicit SQL; migrations here are explicit SQL, `m<date>_<seq>_<name>.rs`);
    `ModelError::EntityNotFound` is not mapped to 404 (return `Error::NotFound`);
    enforcement tests need their own binary (OnceLock); `active → terminated`
    routes via `offboarding`; **never read through `ctx.db` while a transaction
    is open** — the test pool is small and the read waits for a connection the
    transaction holds (a 500 — read before `begin()`); `jsonwebtoken` refuses a
    validation list that mixes key families (see `src/auth/keycloak.rs`).

## Running

```bash
cargo run -- db migrate && cargo run -- task seed && cargo run -- start
cargo test                          # DB-free unit tests (297)
cargo test -- --ignored             # request tests (Postgres; 54, run serially)
cargo test --test enforcement -- --ignored             # auth persona matrix
cargo test --no-default-features --features keycloak \
  --test keycloak -- --ignored      # real Keycloak via Testcontainers (Podman)
cargo clippy --all-targets          # pedantic; keep new files clean
```

Loco tasks (`cargo run -- task <name>`): `seed`, `snapshot_headcount`
(**schedule daily**), `rota_reminders [days_ahead:N]` (**schedule daily**),
`pay_progression_reminders [days_ahead:N]` (**schedule daily**),
`import_framework`, `import_esco`. See [testing](../spec/testing.md) for running
the suite without the sibling crates.
