# Testing and verification

**Run it, look at it, and report exactly what you verified.** A claim you did not
run is a defect. Current state and strategy: [`spec/testing.md`](../spec/testing.md).

## What to run

| Where | Command | Expect (2026-10-06) |
| --- | --- | --- |
| Service | `cargo test` (or `--lib`) | 317 unit tests |
| Service | `cargo test -- --ignored --test-threads=1` | 62 request tests (need Postgres; **serial** — they share a database) |
| Service | `cargo test --test enforcement -- --ignored` | auth persona matrix |
| Service | `cargo test --test enforcement_expenses -- --ignored` | who may see, write and decide an expense claim (own binary: the auth flag is process-wide) |
| Service | `cargo test --no-default-features --features keycloak --test keycloak -- --ignored` | real Keycloak 26 via Testcontainers |
| Service | `cargo clippy --all-targets` | keep **new** files clean (pedantic) |
| UI | `pnpm check` | svelte-check: 0 errors, 0 warnings |
| UI | `pnpm test` | 80 vitest |
| UI | `pnpm exec playwright test` | 45 specs, stubbed API (`PW_PORT=<free port>` if 4173 is taken) |
| UI | `pnpm build` | green |

`CONTRIBUTING.md` also asks for `cargo fmt --check`, `cargo clippy --all-targets
-- -D warnings` and `pnpm lint` (`prettier --check src`). **As of 2026-10-06 all
three are clean across the repo** — the service under both auth backends
(default and `--features keycloak`) — after the whole-repo pass recorded in the
changelog. Keep them clean; if you cannot run one, say so rather than claiming a
green gate. A justified `#[allow(clippy::…)]` carries a reason comment.

## Running the service tests

The two shared crates (`entity-ref`, `authentication-verifier`) are vendored in
`workforce-planning-management-service-with-rust/crates/`, so the service builds and
tests from a fresh clone; no scratch workspace is needed. To run the
database-backed suites:

1. (Only if you want to keep `target/` out of your checkout, copy the service —
   `rsync -a --delete --exclude target --exclude .git` — and run there.)
2. Start a throwaway Postgres 18 with **Podman** (never Docker) on a free port:
   `podman run -d --rm --name <unique> -e POSTGRES_USER=loco -e POSTGRES_PASSWORD=loco
   -e POSTGRES_DB=workforce_planning_management_service_test -p 55440:5432 --tmpfs
   /var/lib/postgresql <postgres-18-image>`; wait a few seconds.
3. `DATABASE_URL=postgres://loco:loco@localhost:55440/workforce_planning_management_service_test
   cargo test --offline --test mod -- --ignored --test-threads=1` in `<ws>/repo/api`.
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
