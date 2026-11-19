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
- **Source-text checks** (in `cargo test --lib`; 1,026 unit tests in all as of 2026-10-30): no line ends in a doubled backslash
  (`src/lib.rs`), and **`crates/workforce-planning-management-domain/src/` names no country, region or single-jurisdiction law in an identifier or string literal**
  (`src/jurisdiction_neutrality.rs`, WPM-T369): an allow-list entry per file and word with its reason, a stale entry fails, and a
  planted name is caught. Beside it, `crates/workforce-planning-management-domain/src/second_profile.rs` runs each rule that takes a jurisdiction's limits as parameters with
  a second, synthetic profile of a different shape ([worldwide](worldwide.md)).
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
- **Enforcement binary** (`tests/enforcement/persona_matrix.rs`, in the `enforcement` binary — the OnceLock
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
- **Shared cache** (`tests/requests/shared_cache.rs`, `tests/shared_cache_upstream.rs`): ESCO searches cached until invalidated;
  namespaces separate; against a real Redis (`WPM_TEST_REDIS_URL`) shared between two handles and failing open when down; with a
  password (`WPM_TEST_REDIS_AUTH_URL`, `..._WRONG_PASSWORD_URL`, `..._NO_PASSWORD_URL`); over TLS (`WPM_TEST_REDIS_TLS_URL` plus
  `SSL_CERT_FILE` naming the server's CA, then the same server refused with an empty trust store); and name lookups through a stub
  sibling service (organization names shared, a person's never).
- **Conversion plans binary** (`tests/enforcement_conversion.rs`, own process; the reference policy plus a `decider` rule): the
  worker, a stranger and a manager outside the chain are refused; only HR reads; the proposer cannot approve; approval
  records the engagement decision; done makes the worker permanent in one step; the audit holds no reason.
- **Flexible working binary** (`tests/enforcement_flexible.rs`, own process; the reference policy plus one rule
  that gives a line manager the power to write): the request limit and the decide-by date, who may decide (the
  line manager up the chain or HR, never the requester, never a manager outside the chain), refusal reasons from a
  deployer's list, an appeal decided by someone else, a counter-proposal, overdue, and that no contract changes.
- **Individual goals binary** (`tests/enforcement/individual_goals.rs`, in the `enforcement` binary, two scenarios): who proposes, agrees and reads; a measure and a check-in; a review's outcomes and frozen snapshot; export and erasure; the roll-up.
- **PADR binary** (`tests/enforcement/padr.rs`, in the `enforcement` binary, two scenarios): who opens, reads, writes and signs a PADR; objectives as goals and their frozen snapshot; a correction; the due date; the roll-up's floor; audit with no content; the export; who a waiting goal tells, a counter-proposal, and checked links.
- **Injury record binary** (`tests/enforcement/injury_record.rs`, in the `enforcement` binary, one scenario): HR only, no description in any field, several reports with their standings, the roll-up's floor, audit with no content, the export, and erasure that keeps the row.
- **Course requests binary** (`tests/enforcement/course_requests.rs`, in the `enforcement` binary, one scenario): a recorded zero enrols, a fee and an unknown fee wait, who may decide, one open request, approval enrols once, a decline enrols nobody, withdrawal.
- **Separation record binary** (`tests/enforcement/separation_record.rs`, in the `enforcement` binary, one scenario): HR only, what the leaver record supplies, named gaps, a cancelled leaver, the export and its formula guard, audit with no content.
- **Learning economics binary** (`tests/enforcement/learning_economics.rs`, in the `enforcement` binary, three scenarios): a plan's fees beside its hours, who may read them, what a masked reader gets, a stated benefit, train or hire with every input, and spend against a budget.
- **`/api/me` binary** (`tests/enforcement/me.rs`, in the `enforcement` binary): a plain worker changes their own contact details and
  emergency contacts past a policy that lets only HR write; every other write stays refused; a neighbour, a person
  with no worker record and an unsigned request get nothing of another person's; the time-off view, in fixed
  past and future years.
- **Need-to-know binary** (`tests/enforcement/need_to_know.rs`, in the `enforcement` binary): eleven kinds of caller against
  six classes of route (stranger, colleague, line manager, a manager higher up, the worker, HR by attribute,
  payroll, service, HR of the worker's organization, HR of another, an unknown token). A unit test scans the
  controllers so a new `GET` route fails the build until it is classified.
- **Engagement enforcement binary** (`tests/enforcement/engagements.rs`, in the `enforcement` binary): a contractor's rate
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

`entity-ref` is vendored in `workforce-planning-management-service-with-rust/crates/`
(WPM-T123) and `authentication-verifier` comes from crates.io, so a fresh clone
builds and tests with no scratch workspace. Start a throwaway Postgres 18 and a
Redis 8 under **Podman** (`podman compose -f compose.test.yaml up -d --wait`, which
mounts `postgres-init/`; set `WPM_TEST_REDIS_URL=redis://localhost:6379` so the
Redis-backed shared-cache tests run instead of skipping), then run `cargo test --lib`,
`cargo test --test main -- --ignored --test-threads=1` (serial — the tests share
one database), and the enforcement, `enforcement_expenses` and `security`
binaries, each in its own process. Sign-in enforcement is **on by default**
(WPM-D52): the request suite opts out explicitly through
`tests/requests/mod.rs::request_open`, which also turns the rate limits off; the
`security` binary runs with the defaults and low limits to prove them.

**Load** (`scripts/load-test.sh`, WPM-R120, WPM-D195): seeds one organization at 1k, 10k and 50k workers (or any size given) in a throwaway Postgres 18 and drives the routes that read an organization with `oha`. Scenarios on the same seed: every path alone, a concurrency ramp (`LOAD_RAMP`), reads and writes at once (`LOAD_MIX`), a soak that samples memory and database connections (`LOAD_SOAK`), and `LOAD_CHECK`, which exits 1 when a stream has a failed request or a p95 over `LOAD_BAR_MS` (2,000 by default, the bar of WPM-D95). CI's `load-smoke` job runs `LOAD_MIX` at 1,000 workers and fails on a failed request only; timings are measured by hand, since shared runners are too noisy to compare, and recorded in [`BENCHMARKS.md`](../BENCHMARKS.md). The ceiling it led to is checked by `tests/requests/scale.rs`.

**Benchmarks** (Criterion, WPM-D195): `cargo bench -p workforce-planning-management-domain --bench rules` (the domain crate, from the service's directory; WPM-T538) and `cargo bench --bench matching` in the matcher measure the pure functions a view runs once per worker, at 1,000, 10,000 and 50,000; `cargo bench --bench requests` (WPM-T527) measures what a request pays apart from the database: the read classification, the directory search and its JSON, the CSV import and export, and a link token's hash. `scripts/bench-compare.py` reads Criterion's change estimates after `--baseline <name>` and exits 1 when the whole 95% interval of a benchmark's change is above a threshold (10% by default), for a comparison on one quiet machine; CI does not run it. CI runs each once with `cargo test --bench`, so a benchmark that panics or stops building fails; the timings are in [`BENCHMARKS.md`](../BENCHMARKS.md).

**Coverage** (`cargo-tarpaulin` 0.37.6, `--engine llvm`, WPM-T539). CI's `coverage` job holds the domain crate and the matcher
at 95% of lines or more, from their unit tests alone, and prints the service's unit-test figure without holding it, because the
service's controllers and tasks are tested by the database-backed suites. **Measured on 2026-11-17:** the domain crate 97.21%
(7,984 of 8,213 lines; `-p workforce-planning-management-domain --lib --include-files 'crates/workforce-planning-management-domain/src/*'`),
the matcher 98.75% (317 of 321), the workspace's unit tests 27.41% (10,212 of 37,256: the controllers are 1.4% from unit tests).
**With the database-backed suites** (`--workspace --lib --tests --ignored -- --test-threads=1` on PostgreSQL 18, about 13
minutes, 2,895 tests): **95.61%** (35,630 of 37,266): the controllers 95.6%, the rules 98.5%, the models 99.3%, the top-level
modules 91.3%, the tasks 87.4%, `src/auth` 64.9%. The least covered: `scheduler.rs` (25.5%: the loop that starts jobs),
`tasks/seed.rs` (34.0%), `auth/paseto.rs` (64.9%: the key fetch and refresh), `compat.rs` (65.2%) and `app.rs` (78.8%: the start-up paths).
The `keycloak` feature's code is not in these figures. Coverage counts lines a test ran, not what it checked: mutation testing
answers that.

**Test binaries** (WPM-T534): one file under `tests/` is one binary, and each links the whole service, so tests that can share a process share one. `tests/main.rs` holds the request suites (`requests/`) and the checks (`checks/`: the source and schema guards, the route table, the schedule, the `health` command, the migration adoption); `tests/enforcement/` holds the twelve suites that need only sign-in and the reference policy, signing with one throwaway key. Sign-in, the ABAC policy and the PASETO verifier are each read once into a `OnceLock`, so a suite that needs another policy or setting stays a binary of its own: the eleven `enforcement_*.rs`, `security`, `retention_schedule`, `request_log`, `chaos`, `shared_cache_upstream`, and `keycloak` and `oidc_entra` under their feature. 37 binaries became 20; relinking them after a change to the library took a median of 15.1 s against 18.5 s before, measured back to back on one loaded laptop.

**Mutation testing** (`cargo-mutants` 27.1.0, WPM-T526): a mutant is a small change to the code (a `>` made `>=`, a match arm
deleted, a function made to return a default), and a test suite that still passes against it has a gap. Until WPM-T526 this was
done by hand: a task broke its code on purpose and recorded "mutation-checked". Now each Rust crate has a `.cargo/mutants.toml`.
The **service's** examines only the pure core (`crates/workforce-planning-management-domain/src/`, and `src/security.rs`) against the lib's unit tests (`--lib`),
because a controller is tested by the database-backed suites, which a mutant run cannot afford. The **matcher's** examines the
whole crate, and leaves out one equivalent mutant (a mutant no test can tell from the code), with its reason. CI's `mutants` job runs
`scripts/mutants-diff.sh`: the mutants in the lines a push or pull request changed, failing on any that survives or hangs, and
sampling an even slice when there are more than the crate's cap (15 for the service, 40 for the matcher). Both run in place, because
the service's unit tests read files from `spec/`, outside the crate, so a copy of the crate does not build. A full run is
`cargo mutants --workspace` in a crate's directory (the service's examines the domain crate, WPM-D203); it is not in CI, since the service's 6,177 mutants take hours. **Results (2026-11-17):** the matcher, every mutant: 195 found, one left out as equivalent, 2 unviable, and of the rest 19 survived the first run; nine new tests (presets, an equal review and match threshold, a zero weight, one-sided emails and phones, different given names, the engine's configuration, the letters NFKD leaves alone, Soundex's l, m and n) kill all 19, so all 192 viable mutants are caught. The service, `src/security.rs` (the rest of the pure rules not yet run: at about a minute a mutant on a shared laptop, they take hours): 33 found; 6 survived in the security-header middleware and `is_https`, which only the database-backed `security` binary drove, and two new unit tests through a router kill them; 6 more, in the rate-limit middleware, the posture probe and the route table, need the database and are left out in `.cargo/mutants.toml` with that reason; the other 22 are caught and 5 unviable.

**Chaos** (`tests/chaos.rs`, WPM-R137): a TCP proxy between the app and Postgres resets, swallows or slows connections, and a seeded monkey flips it
at random during writes. It asserts every request ends soon as a plain `5xx` that leaks no connection detail, that a refused write leaves
nothing behind, that every worker present has its one audit row, and that the service answers again once the database does. The seed is printed;
`WPM_CHAOS_SEED=<n>` replays a run. Its own binary, because it points `DATABASE_URL` at the proxy for the process.

## Front-end edition (`workforce-planning-management-ui-with-svelte`)

The WPM-T550 regression in `tests/unit/money.test.ts` reads signed, grouped
numbers across the shipped locales and refuses misplaced separators.
`tests/e2e/expenses.spec.ts` checks a German claim through correction and
inspects the amount submitted in euro cents. [Worldwide decision](worldwide.md)

The WPM-T551 browser tests in `tests/e2e/injury-records.spec.ts` check the
submitted instants across both Berlin clock changes and for a New York
recorder entering a Kolkata incident. They verify that changing an ambiguous
time or its zone clears the chosen offset. In `tests/unit/injury-time.test.ts`,
Samoa's skipped date and Lord Howe's half-hour overlap exercise changes of
other sizes. [Worldwide decision WPM-D205](worldwide.md)

WPM-T552 checks exact decimal conversion, formatting, editing, request JSON,
response JSON and export-shaped nested values at the shared integer boundary.
The cases cover currency exponents 0, 2, 3 and 4, rounding on both sides of a
half, and the first value outside the range. A planning browser test verifies
that the largest CLF budget is sent unchanged and the next minor unit sends no
request. Service unit tests drive the boundary middleware for request and
response JSON and a query parameter. [Worldwide decision WPM-D206](worldwide.md)

- **vitest — 313 tests / 37 files (as of 2026-10-30)**: the API client **path map** (every
  function's exact proxied path), `money()` honesty and the money formatter and parser
  (`tests/unit/money.test.ts`: JPY, EUR, KWD and CLF exponents, a decimal comma, no built-in
  currency and no fixed 100 anywhere in `src/`, WPM-T373), numbers and dates in the page's locale and a scan for hand-rolled
  formatting (`tests/unit/locale-format.test.ts`, WPM-T375), plural selection and the CLDR categories each locale holds
  (`tests/unit/plurals.test.ts`, WPM-T374), a scan for physical left/right styles and literal arrows (`tests/unit/right-to-left.test.ts`,
  WPM-T377), today in the reader's time zone and the CLDR week (`tests/unit/reader-calendar.test.ts`, WPM-T378), the **locale parity** of
  every `-001` locale (and that regional locales hold only keys that exist), that every
  `content/locales/` directory is `<language>-<region>` and matches `LOCALES`, the
  prefix (full code only) / negotiation logic, the **CMS config is up to date**, the
  CEO dashboard helpers, and component tests.
- **Playwright — 335 tests passing and 19 skipped (as of 2026-10-30)** over a `page.route`-stubbed API (contract-mirroring;
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
