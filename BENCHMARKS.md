# Benchmarks

One measurement exists: the load test below (WPM-T196, 2026-10-20). There is no Criterion suite
(no `benches/`); this file is where results live.

## Load test (WPM-R120, WPM-T196)

`scripts/load-test.sh [sizes...]` starts a throwaway Postgres 17 in Podman, applies the migrations, seeds
**one organization** at each size (workers, one skill each, approved leave for a third), starts the release
binary on it and drives four routes with [`oha`](https://github.com/hatoo/oha) for 10 seconds each at 8 connections.
It tears everything down and refuses to run if the port is already answering (a leftover server would be measured
instead). `LOAD_SECONDS`, `LOAD_CONCURRENCY` and `LOAD_LOG` (the service's log) tune it.

Result, one run, 2026-10-20, Apple M-series laptop (16 cores), Postgres in a container on the same machine,
`cargo build --release`, sign-in enforcement **off** and rate limits **off** (so this measures the service and the
database, not the token check or the 429s):

| workers | route | requests/s | p50 ms | p99 ms | failed |
|---:|---|---:|---:|---:|---:|
| 1,000 | `/_ping` | 52,504 | 0.1 | 0.2 | 0 |
| 1,000 | `/api/workers?limit=50` | 704 | 10.3 | 35.1 | 0 |
| 1,000 | `/api/directory?organization=…` | 175 | 43.9 | 68.6 | 0 |
| 1,000 | `/api/capacity?organization=…` | 88 | 87.6 | 134.7 | 0 |
| 10,000 | `/api/workers?limit=50` | 474 | 16.0 | 31.5 | 0 |
| 10,000 | `/api/directory?organization=…` | 30 | 266 | 370 | 0 |
| 10,000 | `/api/capacity?organization=…` | 16 | 518 | 594 | 0 |
| 50,000 | `/api/workers?limit=50` | 180 | 37.9 | 93.1 | **5** |
| 50,000 | `/api/directory?organization=…` | 7 | 1,229 | 1,364 | **19** |
| 50,000 | `/api/capacity?organization=…` | 5 | 2,202 | 2,489 | **19** |

What it shows:

- **The views that load a whole organization cost in proportion to its size** (directory 44 → 266 → 1,229 ms; capacity
  88 → 518 → 2,202 ms for 1k → 10k → 50k workers). The paged worker list also slows (10 → 16 → 38 ms) because it counts the
  scope on every page.
- **At 50,000 workers requests fail**: `500 Failed to acquire connection from pool: timed out`. Each heavy request holds the
  async runtime for one to two seconds, so even the cheap paged list misses its 500 ms slot to take a connection.
  This is why the default ceiling is **10,000 workers** (`WPM_MAX_ORG_WORKERS`, WPM-D90): the largest size run with no failed request.
- The service does about 50,000 requests/s on a route that touches nothing (`/_ping`); the rest is database and work per request.

What it does **not** show: any other route; mixed traffic or writes; more than 8 connections; a database on another
host or a managed one; sign-in enforcement (a token check adds work to every request); a cold cache; a long run (10 seconds
per route, once, no repeat to give a spread). One run on one laptop is a sketch of the shape, not a capacity plan: repeat it on your hardware before
relying on any figure, and raise `WPM_MAX_ORG_WORKERS` only after doing so. It is **not** a CI job, because shared runners are too noisy to compare.
Other whole-table reads exist (about fifty queries load a `workers` or related table with `.all()`); only the directory, the capacity view and the
plan forecast are bounded.

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
