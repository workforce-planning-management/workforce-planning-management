# Testing and verification

**Run it, look at it, and report exactly what you verified.** A claim you did not
run is a defect. Current state and strategy: [`spec/testing.md`](../spec/testing.md).

## What to run

| Where | Command | Expect (2026-10-30) |
| --- | --- | --- |
| Service | `cargo test --lib` | 1,034 unit tests |
| Service | `cargo test --test mod -- --ignored --test-threads=1` | 184 request tests (need Postgres; **serial** — they share a database) |
| Service | `cargo test --test <binary> -- --ignored --test-threads=1` | one test each: `enforcement` (the auth persona matrix) and the fourteen `enforcement_*` binaries, `security`, `retention_schedule`, `request_log`, `no_union_membership`, `personal_data_inventory`, `route_coverage`, `scheduler_config`; each its own process because the auth flags are process-wide (the list CI runs is in `.github/workflows/ci.yml`) |
| Service | `cargo test --test chaos -- --ignored --nocapture` | database faults through a proxy; the seed is printed |
| Service | `cargo test --no-default-features --features keycloak --test keycloak -- --ignored` | real Keycloak 26 via Testcontainers (`oidc_entra` likewise, with no container) |
| Service | `cargo clippy --all-targets -- -D warnings` | clean (pedantic) |
| Service | `cargo +1.97 check --locked -p workforce-planning-management-migration -p entity-ref` | the MSRV (CI's `msrv` job; the matcher likewise in its own directory) |
| UI | `pnpm lint` | ESLint, then Prettier: clean |
| UI | `pnpm check` | svelte-check: 0 errors, 0 warnings |
| UI | `pnpm test` | 298 vitest in 36 files |
| UI | `PW_PORT=<free port> pnpm exec playwright test` | 330 pass, 19 skip, stubbed API (4173 is the default port and is often taken) |
| UI | `pnpm build` | green |
| UI | `pnpm exec playwright test tests/e2e/a11y-routes.spec.ts` | axe (WCAG 2 A and AA) on every page route; add a new page to `tests/e2e/axe-routes.ts` or `pnpm test` fails |
| Repo | `python3 scripts/status.py --check`, `llms.py --check`, `check-links.py`, `check-days.py`, `check-licenses.py`, `check-actions-pinned.py` | each clean; CI's `docs` job runs them |

`CONTRIBUTING.md` also asks for `cargo fmt --check`, `cargo clippy --all-targets
-- -D warnings` and `pnpm lint` (`eslint .` then `prettier --check` over `src`, `tests`,
`scripts` and the config files; `pnpm lint:fix` repairs what it can). **As of 2026-10-06 all
three were clean across the repo; the ESLint half of `pnpm lint` was added 2026-10-28 (WPM-T347)** — the service under both auth backends
(default and `--features keycloak`) — after the whole-repo pass recorded in the
changelog. Keep them clean; if you cannot run one, say so rather than claiming a
green gate. A justified `#[allow(clippy::…)]` carries a reason comment.

**Run what CI runs before you push, not only the tests.** On 2026-10-25 a push of many commits turned CI red on four checks that no
per-task test run had covered: `cargo fmt --check` (run `cargo fmt`, then clippy again: it can expose lints), `pnpm lint` (Prettier can need two
`--write` passes), `gitleaks detect --redact` over the whole history (a reviewed test fixture goes in `.gitleaksignore` as
`<commit>:<file>:<rule>:<line>` with a reason), and `scripts/verify-db-roles.sh` (the request suite as the least-privilege role: a test that
runs DDL must go through `crate::requests::ddl`, which uses the schema owner's `WPM_TEST_DDL_URL` in that drill).

## Running the service tests

The two shared crates (`entity-ref`, `authentication-verifier`) are vendored in
`workforce-planning-management-service-with-rust/crates/`, so the service builds and
tests from a fresh clone; no scratch workspace is needed. To run the
database-backed suites:

1. (Only if you want to keep `target/` out of your checkout, copy the service —
   `rsync -a --delete --exclude target --exclude .git` — and run there.)
2. Start a throwaway Postgres 18 with **Podman** (never Docker) on a free port:
   `podman run -d --rm --name <unique> -e POSTGRES_USER=loco -e POSTGRES_PASSWORD=loco
   -e POSTGRES_DB=workforce_planning_management_test -p 55440:5432 --tmpfs
   /var/lib/postgresql <postgres-18-image>`; wait a few seconds.
3. `DATABASE_URL=postgres://loco:loco@localhost:55440/workforce_planning_management_test
   cargo test --test mod -- --ignored --test-threads=1` in the service directory.
   A local PostgreSQL works as well: `createdb -U postgres -O <you> <throwaway name>`, point
   `DATABASE_URL` at it, and `dropdb -U postgres <name>` afterwards. A database reused across
   runs can time out at boot (seen once in `skill_gaps`, WPM-T388); use a fresh one.
4. For Keycloak: `export DOCKER_HOST="unix://$(podman machine inspect --format
   '{{.ConnectionInfo.PodmanSocket.Path}}')"` and `TESTCONTAINERS_RYUK_DISABLED=true`.
5. `podman stop` your container afterwards. **Do not touch containers you did not
   start.**

## Writing tests

- **Rules**: unit tests beside the code; edge cases, ties, empty, unknown.
- **Request tests**: one journey per capability in `tests/requests/<area>.rs`,
  `#[ignore = "requires PostgreSQL …"]`, `#[serial]`. Use `seed_worker!` and
  `activate!` (in `tests/requests/mod.rs`), `an_org()`, and a per-test tag so
  rows from other tests do not collide. Pin the refusals (422/404) and the
  negative cases (hidden from whom, "no one named"), not just the happy path.
  Data the API cannot create (a PCF role, an organization membership) can be
  inserted with `ctx.db.execute_unprepared`.
- **Date-dependent tests** derive dates from "today" so they do not rot; avoid
  assumptions about weekdays.
- **Front-end**: every client function goes in the path-map test; every new view
  gets a Playwright spec over `page.route` stubs (an unstubbed call is a 404 and
  loud). Check *behaviour* (states, refusals, empty states), not just text.
- **Layout-critical screens** (the CEO dashboard) are fit-tested at the target
  viewport(s) for page scroll, tile bounds and per-tile clipping, and the
  screenshot is **read by a person**.

## Reporting

In the task entry and in your hand-off, say: what you ran (counts), what you did
not (and why), anything you fixed along the way, and anything that failed — with
the output, not a summary.
