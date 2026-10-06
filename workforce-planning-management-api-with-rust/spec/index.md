# Workforce Planning Management service — edition spec

Stack-specific specification for the Loco edition. The
**cross-cutting spec at [`../../spec/`](../../spec/index.md) is the
single source of truth** for the domain (model, pillars, state
machines, auth posture); this file adds only what is specific to
this edition.

<!-- PRO-H8, 2026-08-28: removed the stale "will grow topic files
     ... as implementation phases land" promise — spec/tasks.md shows
     all phases (WPM-T1–T36) landed 2026-07-18 → 2026-07-25 and no
     topic files (routes/api-contract/database/examples) were ever
     added; this index.md stayed the single edition-spec file
     throughout, by outcome rather than by plan. -->

## Stack

Per [architecture](../../spec/architecture.md) and the family
[rust-loco-stack](../../../agents/share/rust-loco-stack.md):
Rust 2024, Loco (Axum + SeaORM), PostgreSQL 18, crate-root
`migration/`, loco-idiomatic `src/controllers/` layout, pure
`src/rules/` core, stub-first upstream clients, offline PASETO +
ABAC, OpenAPI/Swagger, `Accepts-version` header versioning,
tracing + OTLP, Podman.

## Edition-specific decisions (so far)

- **Config**: `config/{development,test,production}.yaml`;
  development and test default upstream clients to `stub` and
  `WPM_EVENT_TRANSPORT=memory`.
- **Env vars**: `WPM_REQUIRE_AUTH`, `WPM_PASETO_KEYS[_URL]`
  (+ `_REFRESH_SECS`), `WPM_ABAC_POLICY[_FILE]`,
  `WPM_EVENT_TRANSPORT`, `WPM_RETENTION_DAYS` (default 365, floor
  30), upstream base URLs (`WPM_PERSON_SERVICE_URL`,
  `WPM_WORKER_SERVICE_URL`, `WPM_ORGANIZATION_SERVICE_URL`,
  `WPM_COURSE_SERVICE_URL`), `WPM_UPSTREAM_MODE` (default `stub`).
  Legacy `HCM_*` spellings still read with a deprecation warning
  (`src/compat.rs`).
- **Identifiers**: public UUID `pid` on every owned record; EntityRef
  URNs for all upstream references; worker number unique per
  organization.
- **Table names all-plural** (the loco `create_table` pluralization
  lesson); `event_outbox` as explicit SQL.

## Edition-specific implementation notes (as landed)

- **Layout**: `src/{app,auth,clients,compat,metrics,openapi,
streaming,validation,version}.rs` (+ `auth/{paseto,keycloak}.rs`, the two
  verification backends, exactly one per build), `src/rules/` (pure core, ~46
  modules — one per subsystem), `src/models/` (+`_entities/`, ~90 entities, the
  notifications `push` helper), ~40 controllers (`src/controllers/`),
  `src/tasks/` (`seed`, `snapshot_headcount`, `rota_reminders`,
  `import_framework`, `import_esco`), crate-root `migration/` (**46** migration
  sets, explicit SQL, named `m<date>_<seq>_<name>`);
  `config/abac-policy.reference.json` is the shipped persona policy
  the enforcement matrix mounts (WPM-G1).
- **Masking**: `mask_worker` clears `salary_minor`+currency;
  `mask_payslip` zeroes amounts and drops the deduction lines;
  payslip reads authorize against the **owning worker's** resource
  attrs, so one policy masks both.
- **Ownership**: `resource.person` carries the bare person uuid so a
  `{"resource.person": ["$sub"]}` rule gives workers their own
  records (deployments map user pid = person uuid).
- **Payroll**: `calculate` replaces the run's payslips inside one
  transaction; only `approved` time entries feed overtime; benefit
  costs join only in the payslip currency; stub tax = 20% over a
  monthly allowance (documented demo stub).
- **Enforcement tests** live in `tests/enforcement.rs` (own process,
  OnceLock lesson) and mount the shipped reference policy file;
  request tests in `tests/requests/` are `#[ignore]`d — run with
  `cargo test -- --ignored`.
- **Privacy mechanics**: erasure = in-place scrub + tombstone
  `person:` URN + soft-delete (raw-SQL statements share one
  transaction, counts in the audit snapshot); the retention sweep
  iterates the pinned `SOFT_DELETED_TABLES` list (55); pulse
  submissions audit **without** an actor; subject access refuses
  masked callers (a full export cannot be "masked").
- **Lifecycle gotcha**: `active → terminated` routes via
  `offboarding` (the erasure tests pin it).
- **Auth backends**: the default `paseto` backend (offline PASETO v4.public
  against the sibling authentication service's keys) and the `keycloak` Cargo
  feature (a Keycloak-issued JWT verified against the realm's JWKS) produce the
  same `Claims`, so ABAC and every controller are unaffected; the Keycloak
  suite runs against a real Keycloak 26 via Testcontainers on Podman. The
  algorithm is taken from the token header **only if it is RS256 or ES256**
  (a mixed validation list is refused by `jsonwebtoken`).
- **Where the newer areas live**: directory (`controllers/directory.rs`),
  emergency contacts + backups (`contacts.rs`), on-call rota (`rotas.rs`,
  `rota_swaps.rs`, `tasks/rota_reminders.rs`), announcements
  (`announcements.rs`), skill gaps (`skill_gaps.rs`), training plans
  (`training_plan.rs`), joiners/leavers (`movements.rs`, `handover.rs`),
  insights and metrics (`intelligence.rs`, `rules/{metrics,insights}.rs`) — each
  specified in the cross-cutting topic files linked from
  [../../spec/index.md](../../spec/index.md).
- **Personal-data wiring**: a person-keyed table joins the subject-access
  export and the erasure statements in `controllers/privacy.rs` (new
  statements go at the end — the audit snapshot indexes them by position); a
  soft-deleting table joins `SOFT_DELETED_TABLES`.
- **Test-pool gotcha**: do not read through `ctx.db` while a transaction is open
  — the test pool is small and the read waits for the connection the transaction
  holds, surfacing as a 500.

## Delivery

The queue is [../../spec/tasks.md](../../spec/tasks.md):
WPM-T1–T19 **delivered 2026-07-18**; the wellbeing / 360 / privacy /
ergonomics / adjustments rounds (WPM-T20–T36) **delivered
2026-07-20 → 2026-07-25**; both production gates' code sides are
done (WPM-G1/G2 `[~]` — operational/legal work remains); WPM-T37–T89
(strategic planning, frameworks, reporting lines, the directory, cover and
on-call, announcements, the CEO dashboard, skills gaps and training time,
joiners and leavers, localization) **delivered 2026-09-28 → 2026-10-06**.
Tests per [../../spec/testing.md](../../spec/testing.md): 302 unit tests,
55 database-backed request tests, the enforcement matrix and the Keycloak
suite.
