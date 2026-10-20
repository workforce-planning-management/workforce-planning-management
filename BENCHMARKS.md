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
| 50,000 | key-person register / working time (before; no critical skill seeded, no time entries) | 7 / 10 | 1,292 / 965 | 1,396 / 1,098 | **6 / 5** |
| 10,000 | key-person register / working time (after; 2,000 holders, 200,000 time entries) | 67 / 31 | 119 / 259 | 139 / 304 | 0 |
| 50,000 | key-person register / working time (after; 10,000 holders, 1,000,000 time entries) | 16 / 9 | 516 / 916 | 561 / 1,021 | 0 / **3** |
| 50,000 | the same, one connection at a time | 15 / 8 | 65 / 117 | 77 / 152 | 0 |
| 50,000 | plan forecast / cost | refused (`422`, over 10,000) | | | |
| 50,000 | plan forecast / cost, ceiling raised to 100,000 | 7 / 8 | 1,227 / 1,057 | 1,277 / 1,071 | **7 of 64, 7 of 77** |

(The 1,000-worker rows are the first of each run and are noisy: an earlier run measured the directory at 285 requests/s,
27 ms.)

What it shows:

- **The workforce overview, metrics and insights** (the dashboard's numbers) counted by reading every worker into memory and failed requests at 50,000. They are now counted by the database (opening, closing, starters and leavers in one query; groups and spans as grouped queries): metrics and insights 655 ms to 51 ms, the overview 561 to 272 ms, no failures. Two new request tests pin the answers and pass against the old code as well as the new.
- **The key-person register and working time** read every worker and failed under load. The register now reads only the holders of a critical skill and working time sums the window's minutes in the database and reads only the flagged workers. The load test now seeds a critical skill (bar 5, a fifth of the workers hold it) and 20 time entries per worker, so the "after" rows do real work that the "before" row did not; on that heavier data one connection sees 65 and 117 ms. At 8 concurrent connections, sharing the laptop with the database, the register takes 516 ms and working time 916 ms and loses 3 requests to a connection-pool timeout: its aggregate scans every entry in the window.

- **The CPD overview** (added to the load test after the second pass, with the workers seeded but no CPD entries, so the entry side is not exercised at scale) counts in the database: 50,000 workers, one requirement, 323 ms and no failure. Before, it read every worker and every entry into memory.

- **The directory now costs the page, not the organization**: 1,229 ms and failures at 50,000 became 159 ms and none. The
  paged worker list also slows with size (11 → 16 → 39 ms) because it counts the scope on every page.
- **The capacity view no longer fails** at 50,000 (it reads in batches and holds only sums), but it is **still linear**: 2.2 s at 50,000
  (the same as before, 2.2 s, once the extra queries per batch were paid back). A later pass selected only the six columns it needs and counted
  available days by interval arithmetic (a test says it equals walking the days) and took it from 3.0 s. A single request spends about 25 ms per
  batch of 2,000 workers, mostly decoding rows and four round trips; the next step is SQL aggregates, which has not been done.
- **The plan views still read every worker of the planned departments**, and above 10,000 of them requests fail
  (`500 … Connection pool timed out`: each request holds the async runtime for about a second). That is why
  `WPM_MAX_ORG_WORKERS` (default **10,000** when this was written; **200,000** since WPM-D95) still guards them and only them.
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

## Baseline at 200,000 workers (WPM-R142, WPM-D95)

The first run at the new supported size, before any change for it (8 connections, 8 s per route, one laptop; the bar is 2 s p95 with no failure):

| Route | p50 ms | p99 ms | Failed | Meets the bar |
|---|---:|---:|---:|---|
| worker list (50) | 84 | 122 | 0 | yes |
| directory | 377 | 418 | 0 | yes |
| capacity | no answer in 8 s | | all | **no** |
| plan forecast / cost | refused (`422`, over 10,000) | | 557 / 581 | **no** |
| CPD overview | 756 | 821 | 5 | **no** |
| workforce overview | 587 | 635 | 0 | yes |
| metrics / insights | 103 / 105 | 152 / 258 | 0 | yes |
| key-person register | 2,145 | 2,244 | 1 | **no** |
| working time | 925 | 1,993 | 61 | **no** |

Five routes fail the bar: capacity, plan forecast and cost, the CPD overview, the key-person register and working time. Several failures are connection-pool timeouts under 8 connections, not slow single requests.

### Correction, and capacity in the database (WPM-T239, first step)

**The load test ran the demo profile, which holds one database connection and waits 500 ms for it.** Any query over half a second made the
other seven connections fail with a pool timeout, so most "failed requests" above (working time, the CPD overview, the key-person register,
the worker list at 200,000) were that setting, not the code. A production deployment holds 20 (`config/production.yaml`); the script now
sets `DB_MAX_CONNECTIONS` (default 20) and `DB_CONNECT_TIMEOUT` (5 s). Slow routes are still slow; the failures were not real.

Capacity now sums supply in the database (one query per pool; the rule's definitions, in SQL). With 20 connections, at 200,000 workers:

| Route | p50 ms | p99 ms | Failed | Meets the bar (2 s p95, none failed) |
|---|---:|---:|---:|---|
| worker list / directory | 24 / 127 | 53 / 271 | 0 | yes |
| capacity | 1,279 | 1,494 | 0 | yes (was no answer in 8 s) |
| CPD overview | 170 | 261 | 0 | yes |
| workforce overview / metrics / insights | 198 / 33 / 32 | 281 / 74 / 61 | 0 | yes |
| key-person register | 535 | 859 | 0 | yes |
| working time | 1,433 | 3,041 | 0 | **no** (p99 over 3 s) |
| plan forecast / cost | refused (`422`) | | all | **no** |

Two routes remain: working time, and the plan forecast and cost (still refused over 10,000 workers).

### The plan forecast and cost in the database (WPM-T239, second step)

The two views counted every employed worker of the planned departments in memory. They now read groups: employed people grouped by
department, type, FTE and engagement end (a few hundred rows), the declared skill levels as counts, the few people named by an open plan to
convert, and the salaries of one currency; the answers are the same (`plan_views_count_every_kind_of_person_once`, **mutation-checked**).
The ceiling `WPM_MAX_ORG_WORKERS` defaults to 200,000. The load test's Postgres now gets 512 MB of shared memory: with the container's
default 64 MB, parallel queries failed with "could not resize shared memory segment", which a deployer's database should also have room for.

At 200,000 workers, 8 connections, 20 database connections:

| Route | p50 ms | p99 ms | Failed | Meets the bar |
|---|---:|---:|---:|---|
| plan forecast / cost | 243 / 134 | 458 / 279 | 0 | yes (were refused) |
| capacity | 1,352 | 1,751 | 0 | yes |
| key-person register | 576 | 901 | 0 | yes |
| working time | 1,917 | 3,904 | 0 | **no**; one request at a time takes 515 ms |

One route is left: working time. Its sum reads every time entry of the 17-week window (4,000,000 rows here, at 20 per worker; a
worker who records daily has about 85), so it is bound by the database's CPU, and a seven-times-heavier ledger would be worse. It needs a
rollup (a weekly total per worker kept as entries change), not a faster query.
