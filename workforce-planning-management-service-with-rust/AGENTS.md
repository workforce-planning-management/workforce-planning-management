# AGENTS.md — working agreements

A pocket guide for human and AI collaborators working in this
subproject. Read this **before** opening a PR. (Updated 2026-10-30,
through WPM-T396.)

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
- **Delivery capacity and oversight** — skill pools, programme demand, the
  what-if, early warnings, the oversight pack, cutover readiness.
- **Bridges to the siblings** — the pool link, peer demand and supply and
  people impact for project-portfolio-management; the hire and leave webhooks,
  the overlap check and the lean worker list (`?fields=`) for
  contact-relationship-management. References and counts only, under the
  caller's token ([cross-app-bridges](../spec/cross-app-bridges.md)).

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
   cache. See rust-loco-stack.
3. **Pure core first.** Rules live in DB-free, clock-free `src/rules/<area>.rs`
   with exhaustive unit tests (dates and "today" are always supplied);
   controllers only load data, call the rule and write. Write and test the rule
   module **before** its controller. There are 115 entries in `src/rules/`
   (2026-10-30), from `lifecycle`, `leave` and `working_time` to `capacity`,
   `bridge`, `people_impact`, `webhook` and `worker_fields`; look there before
   writing a new one.
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
cargo run -- db migrate && cargo run -- task seed
WPM_REQUIRE_AUTH=0 cargo run -- start   # sign-in is enforced by default (WPM-D52)
cargo test --lib                    # DB-free unit tests (1,034 on 2026-10-30)
cargo test --test mod -- --ignored --test-threads=1    # request tests (Postgres; serial, they share a database)
cargo test --test enforcement -- --ignored             # auth persona matrix
cargo test --test security -- --ignored                # headers and rate limits
cargo test --test chaos -- --ignored --nocapture       # database faults (reset, blackhole, slow, a seeded monkey)
cargo test --no-default-features --features keycloak \
  --test keycloak -- --ignored      # real Keycloak via Testcontainers (Podman)
cargo clippy --all-targets          # pedantic; keep new files clean
```

Loco tasks (`cargo run -- task <name>`), sixteen as of 2026-10-30: `seed`; the scheduled ones in
`config/scheduler.yaml` (`webhook_deliver` every minute, `snapshot_headcount`, `rota_reminders`,
`pay_progression_reminders`, `policy_reminders`, `engagement_end_reminders`, `early_warning_sweep`,
`oversight_pack_freeze`, `verify_audit_chain`, and `scim_push` where SCIM is configured); `replay_erasures`
after a restore; and the imports `import_framework`, `import_esco`, `import_sfia`, `import_sfiaplus`. See
[testing](../spec/testing.md) for running the suites (`entity-ref` is vendored in `crates/`;
`authentication-verifier` comes from crates.io).

**MSRV.** The policy is stable minus two (1.97 while stable is 1.99), checked by CI's `msrv` job for the
migration crate and `entity-ref`. This crate still declares 1.99 because the published matcher 0.1.0
does; lower it when a matcher declaring 1.97 is published (WPM-T390).
