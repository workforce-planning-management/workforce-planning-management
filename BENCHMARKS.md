# Benchmarks

One measurement exists: the load test below (WPM-T196, 2026-10-20). There is no Criterion suite
(no `benches/`); this file is where results live.

## Load test (WPM-R120, WPM-T196)

`scripts/load-test.sh [sizes...]` starts a throwaway Postgres 17 in Podman, applies the migrations, seeds
**one organization** at each size (workers, one skill each, approved leave for a third), starts the release
binary on it and drives six routes (the plan views on a plan with a demand line in every department) with [`oha`](https://github.com/hatoo/oha) for 10 seconds each at 8 connections.
It tears everything down and refuses to run if the port is already answering (a leftover server would be measured
instead). `LOAD_SECONDS`, `LOAD_CONCURRENCY` and `LOAD_LOG` (the service's log) tune it.

Two passes, 2026-10-20, Apple M-series laptop (16 cores), Postgres in a container on the same machine,
`cargo build --release`, sign-in enforcement **off** and rate limits **off** (so this measures the service and the
database, not the token check or the 429s). Median latency in ms; **failed** is any request that did not get a `2xx`.

**First pass** (before WPM-T230): every view loaded the whole organization into memory.

| workers | `/api/workers?limit=50` | directory | capacity | failed |
|---:|---:|---:|---:|---:|
| 1,000 | 10 | 44 | 88 | 0 |
| 10,000 | 16 | 266 | 518 | 0 |
| 50,000 | 38 | 1,229 | 2,202 | **5, 19, 19** |

**Second pass** (after WPM-T230: the directory searches, orders and pages in SQL; the capacity view reads workers in
batches of 2,000 and adds up the supply; the plan views read only the planned departments' workers).
Requests/s, p50 ms, p99 ms, 10 s at 8 connections:

| workers | route | requests/s | p50 ms | p99 ms | failed |
|---:|---|---:|---:|---:|---:|
| 1,000 | `/api/workers?limit=50` | 477 | 11.7 | 94.9 | 0 |
| 1,000 | directory | 146 | 40.4 | 219.9 | 0 |
| 1,000 | capacity | 78 | 89.6 | 248.9 | 0 |
| 1,000 | plan forecast / cost | 127 / 172 | 61 / 46 | 106 / 62 | 0 |
| 10,000 | `/api/workers?limit=50` | 488 | 15.9 | 24.8 | 0 |
| 10,000 | directory | 115 | 68.5 | 117 | 0 |
| 10,000 | capacity | 19 | 437 | 480 | 0 |
| 10,000 | plan forecast / cost | 27 / 34 | 282 / 235 | 482 / 317 | 0 |
| 50,000 | `/api/workers?limit=50` | 201 | 39.3 | 60.2 | 0 |
| 50,000 | directory | 50 | 158.5 | 171 | 0 |
| 50,000 | capacity | 4 | 2,245 | 2,251 | 0 |
| 10,000 | CPD overview (one requirement) | 158 | 50.5 | 60.7 | 0 |
| 50,000 | CPD overview (one requirement) | 25 | 323 | 369 | 0 |
| 10,000 | workforce overview / metrics / insights (before) | 40 / 38 / 40 | 208 / 203 / 203 | 229 / 303 / 217 | 0 |
| 10,000 | workforce overview / metrics / insights (after) | 91 / 409 / 417 | 87 / 19 / 19 | 118 / 28 / 23 | 0 |
| 50,000 | workforce overview / metrics / insights (before) | 16 / 12 / 13 | 561 / 655 / 587 | 629 / 945 / 968 | **30 / 16 / 19** |
| 50,000 | workforce overview / metrics / insights (after) | 30 / 155 / 155 | 272 / 51 / 51 | 295 / 67 / 64 | 0 |
| 10,000 | key-person register / working time | 29 / 38 | 286 / 211 | 354 / 228 | 0 |
| 50,000 | key-person register / working time | 7 / 10 | 1,292 / 965 | 1,396 / 1,098 | **6 / 5** |
| 50,000 | plan forecast / cost | refused (`422`, over 10,000) | | | |
| 50,000 | plan forecast / cost, ceiling raised to 100,000 | 7 / 8 | 1,227 / 1,057 | 1,277 / 1,071 | **7 of 64, 7 of 77** |

(The 1,000-worker rows are the first of each run and are noisy: an earlier run measured the directory at 285 requests/s,
27 ms.)

What it shows:

- **The workforce overview, metrics and insights** (the dashboard's numbers) counted by reading every worker into memory and failed requests at 50,000. They are now counted by the database (opening, closing, starters and leavers in one query; groups and spans as grouped queries): metrics and insights 655 ms to 51 ms, the overview 561 to 272 ms, no failures. Two new request tests pin the answers and pass against the old code as well as the new.
- **Still failing at 50,000:** the key-person register (`/api/workforce-intelligence/key-person`) and working time (`/api/workforce/working-time`) read every worker and fail under load.

- **The CPD overview** (added to the load test after the second pass, with the workers seeded but no CPD entries, so the entry side is not exercised at scale) counts in the database: 50,000 workers, one requirement, 323 ms and no failure. Before, it read every worker and every entry into memory.

- **The directory now costs the page, not the organization**: 1,229 ms and failures at 50,000 became 159 ms and none. The
  paged worker list also slows with size (11 → 16 → 39 ms) because it counts the scope on every page.
- **The capacity view no longer fails** at 50,000 (it reads in batches and holds only sums), but it is **still linear**: 2.2 s at 50,000
  (the same as before, 2.2 s, once the extra queries per batch were paid back). A later pass selected only the six columns it needs and counted
  available days by interval arithmetic (a test says it equals walking the days) and took it from 3.0 s. A single request spends about 25 ms per
  batch of 2,000 workers, mostly decoding rows and four round trips; the next step is SQL aggregates, which has not been done.
- **The plan views still read every worker of the planned departments**, and above 10,000 of them requests fail
  (`500 … Connection pool timed out`: each request holds the async runtime for about a second). That is why
  `WPM_MAX_ORG_WORKERS` (default **10,000**, WPM-D90) still guards them and only them.
- The service does about 50,000 requests/s on a route that touches nothing (`/_ping`); the rest is database and work per request.

What it does **not** show: any other route; mixed traffic or writes; more than 8 connections; a database on another
host or a managed one; sign-in enforcement (a token check adds work to every request); a cold cache; a long run (10 seconds
per route, once, no repeat to give a spread). One run on one laptop is a sketch of the shape, not a capacity plan: repeat it on your hardware before
relying on any figure, and raise `WPM_MAX_ORG_WORKERS` only after doing so. It is **not** a CI job, because shared runners are too noisy to compare.
Other whole-table reads exist (about fifty queries load a `workers` or related table with `.all()`); of those, the directory, the capacity view and the
plan forecast and cost views are bounded or paged, and the rest are not measured.

## What exists today

- Service (2026-10-11): 561 DB-free unit tests, 95 database-backed request tests
  (serial), the auth enforcement persona matrix and a Keycloak suite
  against a real Keycloak 26, run with `cargo test` and `cargo test -- --ignored`.
  See [workforce-planning-management-service-with-rust/README.md](workforce-planning-management-service-with-rust/README.md)
  and [spec/testing.md](spec/testing.md).
- Front-end: 145 vitest + 130 Playwright specs (+ 11 live), run with `pnpm test` /
  `pnpm exec playwright test`. The CEO dashboard is **fit-tested** (page scroll
  and per-tile clipping) at 1080 × 810 @2× and 2160 × 1620 @1× — a layout
  budget, not a speed benchmark.
- Rough timings seen while developing (not measurements): the full
  database-backed request suite runs in about 15–20 seconds serially; the
  Playwright suite in about 8 seconds.

These are correctness suites, not benchmarks — they prove behaviour,
not throughput or latency.

## If you add a benchmark

Prefer Criterion (`cargo bench`) for the service; assert the property
you care about in a test where possible — a number nobody compares is
not a gate. Record the method and any measured results here, with the
task id from [spec/tasks.md](spec/tasks.md) that motivated it.
