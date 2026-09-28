# Benchmarks

Neither subproject ships a performance benchmark suite yet (no
`benches/`, no Criterion dependency in
`workforce-planning-management-service-with-rust/Cargo.toml`). This
file exists as the place such results would live once measured.

## What exists today

- Service: 139 DB-free unit tests + 19 request suites + the
  enforcement persona matrix, run with `cargo test`. See
  [workforce-planning-management-service-with-rust/README.md](workforce-planning-management-service-with-rust/README.md)
  for current counts.
- Front-end: 10 vitest + 9 Playwright specs, run with `pnpm test` /
  `pnpm exec playwright test`.

These are correctness suites, not benchmarks — they prove behaviour,
not throughput or latency.

## If you add a benchmark

Prefer Criterion (`cargo bench`) for the service; assert the property
you care about in a test where possible — a number nobody compares is
not a gate. Record the method and any measured results here, with the
task id from [spec/tasks.md](spec/tasks.md) that motivated it.
