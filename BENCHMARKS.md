# Benchmarks

Neither subproject ships a performance benchmark suite yet (no
`benches/`, no Criterion dependency in
`workforce-planning-management-api-with-rust/Cargo.toml`). This
file exists as the place such results would live once measured.

## What exists today

- Service (2026-10-06): 297 DB-free unit tests, 54 database-backed request tests
  (19 files, serial), the auth enforcement persona matrix and a Keycloak suite
  against a real Keycloak 26, run with `cargo test` and `cargo test -- --ignored`.
  See [workforce-planning-management-api-with-rust/README.md](workforce-planning-management-api-with-rust/README.md)
  and [spec/testing.md](spec/testing.md).
- Front-end: 74 vitest + 29 Playwright specs, run with `pnpm test` /
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
