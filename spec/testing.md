# Testing

State as of 2026-10-07 (WPM-T97).

## Service edition (`workforce-planning-management-service-with-rust`)

- **Pure-core unit tests** (DB-free, `cargo test --lib` — **317**): every
  lifecycle's legal/illegal transition matrix; leave-balance, overtime, shift
  and payslip arithmetic; org-chart cycle refusal; working-time boundaries;
  wellbeing eligibility; the pulse k-floor and 360 group floor; assessment
  category↔scale exhaustiveness; erasure/retention pins (incl. the
  retention-sweep table list, 56); notification kinds; and, for the newer
  modules, `rules::{metrics, insights, directory, emergency, cover, rota,
  announcements, skill_gap, training, movements, pay_scale, pay_position, job_levels,
  grade, expenses}` — each pinning its edge cases
  (dated windows, ties, "nobody available", "unknown is not zero", completion
  gating). Each new rule module is written and tested before its controller.
- **Request tests** (Postgres, `#[ignore]`d, `cargo test -- --ignored` —
  **62 tests in 24 files**): the hire journey; leave and shift conflicts;
  payroll; L&D; assessments; talent; wellbeing; pulse; 360; ergonomics;
  adjustments; subject rights; and one journey per newer capability — directory
  (employed only, nothing sensitive, away / covered / on call), emergency
  contacts and backups, the on-call rota (rotation, swap, leave skip, reminders,
  swap requests), announcements (ordering, hidden scheduled/expired, department
  audience, links, read counts without identities), skill gaps (merge across
  sources, ranking, unknown, roll-up with no names, no aspirations), training
  plans (cheapest-first, estimate, completed course dropped, schedule dates,
  assess-first) and joiners/leavers (a full leaver journey: dated checklist,
  twelve held items, refusals, one-by-one and bulk handover, audit trail,
  completion gating), the national pay scale (placement, progression, refusals), job levels
  (unknown left null, no pay), grades (a worker's level audited without the level and erased;
  a role's level linked to a pay band by the editor), pay positions (dated eligibility,
  reminders sent once and naming no pay) and expense claims (draft to reimbursed, duplicates
  flagged, no amount in the audit or notification, erasure keeping amounts and dropping
  words). Unknown-pid 404s pinned throughout.
- **Enforcement binary** (`tests/enforcement.rs`, own process — the OnceLock
  lesson): the persona matrix on the shipped reference policy file —
  401/403 splits, masking, `$sub` self-reads, destructive gating.
- **Health requirements binary** (`tests/enforcement_health.rs`, own process): off until a lawful basis is recorded;
  only an occupational-health token records a status (the worker, a manager, HR and the service are each refused);
  a manager and HR get cleared or not cleared and never the status (checked key by key and by word); only
  occupational health reads the detail, and each read is audited; the audit holds no status; the export gives the
  records to the worker alone; erasure removes them. Unit tests scan the source so no other code names the table.
- **Equality monitoring binary** (`tests/enforcement_equality.rs`, own process): the feature does not exist until a
  lawful basis is recorded (categories alone switch nothing on); once on, each worker reads and changes only their
  own answers; the only output is the aggregate, with cells below the floor withheld and the department breakdown
  withheld unless every cell passes; the export gives the answers to the worker and a note to anyone else; the audit
  holds no value; erasure removes them. A unit test scans the source so no other code names the table.
- **Conversion plans binary** (`tests/enforcement_conversion.rs`, own process; the reference policy plus a `decider` rule): the
  worker, a stranger and a manager outside the chain are refused; only HR reads; the proposer cannot approve; approval
  records the engagement decision; done makes the worker permanent in one step; the audit holds no reason.
- **Flexible working binary** (`tests/enforcement_flexible.rs`, own process; the reference policy plus one rule
  that gives a line manager the power to write): the request limit and the decide-by date, who may decide (the
  line manager up the chain or HR, never the requester, never a manager outside the chain), refusal reasons from a
  deployer's list, an appeal decided by someone else, a counter-proposal, overdue, and that no contract changes.
- **`/api/me` binary** (`tests/enforcement_me.rs`, own process): a plain worker changes their own contact details and
  emergency contacts past a policy that lets only HR write; every other write stays refused; a neighbour, a person
  with no worker record and an unsigned request get nothing of another person's; the time-off view, in fixed
  past and future years.
- **Need-to-know binary** (`tests/enforcement_need_to_know.rs`, own process): eleven kinds of caller against
  six classes of route (stranger, colleague, line manager, a manager higher up, the worker, HR by attribute,
  payroll, service, HR of the worker's organization, HR of another, an unknown token). A unit test scans the
  controllers so a new `GET` route fails the build until it is classified.
- **Engagement enforcement binary** (`tests/enforcement_engagements.rs`, own process): a contractor's rate
  is masked like salary against the shipped reference policy.
- **Expense enforcement binary** (`tests/enforcement_expenses.rs`, own process — the auth
  flag is process-wide): with an open blanket policy so the *controller's* rule is what is
  tested — the claimant and HR write a claim, the manager and HR decide it, a peer or
  stranger gets 404/403, the claimant is absent from their own queue, and **HR cannot approve
  their own claim**. **Mutation-checked:** with `may_decide` stripped of its not-the-claimant
  clause the test fails at that assertion.
- **Keycloak binary** (`tests/keycloak.rs`, feature `keycloak`): a **real
  Keycloak 26** started by Testcontainers (Podman), real access tokens, the
  app booted against its JWKS. It found and fixed two defects the unit tests
  could not (a compile error and an `InvalidAlgorithm` on every token).
- Clippy **pedantic** is on (`#![warn(clippy::pedantic)]`) and `cargo clippy
  --all-targets -- -D warnings` is **clean** (default and `--features keycloak`
  builds), as are `cargo fmt --check` and the front-end's `pnpm lint`
  (2026-10-07).

### Running the service tests

The two shared crates (`entity-ref`, `authentication-verifier`) are vendored in
`workforce-planning-management-service-with-rust/crates/` (WPM-T123), so a fresh
clone builds and tests with no scratch workspace. Start a throwaway Postgres 18
under **Podman** (`podman compose -f compose.test.yaml up -d --wait`, which
mounts `postgres-init/`), then run `cargo test --lib`,
`cargo test --test mod -- --ignored --test-threads=1` (serial — the tests share
one database), and the enforcement, `enforcement_expenses` and `security`
binaries, each in its own process. Sign-in enforcement is **on by default**
(WPM-D52): the request suite opts out explicitly through
`tests/requests/mod.rs::request_open`, which also turns the rate limits off; the
`security` binary runs with the defaults and low limits to prove them.

## Front-end edition (`workforce-planning-management-ui-with-svelte`)

- **vitest — 80 tests / 12 files**: the API client **path map** (every
  function's exact proxied path), `money()` honesty, the **locale parity** of
  every `-001` locale (and that regional locales hold only keys that exist), that every
  `content/locales/` directory is `<language>-<region>` and matches `LOCALES`, the
  prefix (full code only) / negotiation logic, the **CMS config is up to date**, the
  CEO dashboard helpers, and component tests.
- **Playwright — 45 specs** over a `page.route`-stubbed API (contract-mirroring;
  an unstubbed call is a 404 and loud): the signed-in smoke journeys, the
  locale redirect / picker / the unforwarded bare language / `/admin/` shell, and one spec per newer
  area. The **CEO dashboard** is checked at **two viewports** — 1080 × 810 @2×
  (an iPad, 9th generation) and a literal 2160 × 1620 @1× — for page scroll,
  every tile inside the screen, **and no tile clipping its own content**
  (tiles hide overflow, so a page-level check alone would miss it); screenshots
  are written to `test-results/` and should be looked at.
- `svelte-check` is clean (0 errors, 0 warnings); `pnpm build` is green.

## Gotchas

- Playwright reuses whatever is listening on its port: another project's preview server on 4173
  made the specs (and the screenshots) run against the wrong app. Use `PW_PORT=<free port>`.
- Playwright prefers the **last registered** route: register a generic `…/*` stub before the
  specific ones, or it swallows them.
- Playwright occasionally fails nearly everything when started straight after
  another build (the preview server races the build); re-run it.
- `pnpm cms-config:check` (and a unit test) fail if `static/admin/config.yml`
  is out of date — regenerate with `pnpm cms-config`.

## Data

- **Seed task**: a synthetic org (~40 employees, requisitions in every stage, a
  payroll run) — synthetic data only. No real personal data anywhere.
