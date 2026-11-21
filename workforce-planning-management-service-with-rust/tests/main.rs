//! The integration tests that can share one process, in one binary (WPM-T534): the request-level suites under [`requests`]
//! and the source and configuration checks under [`checks`]. One binary links once, where each file under `tests/` would link
//! the whole service again.
//!
//! The request suites boot the app against the `test` environment (PostgreSQL through `DATABASE_URL`) and are `#[ignore]`d by
//! default: `cargo test --test main -- --ignored --test-threads=1`. The checks run without: `cargo test --test main`. Sign-in
//! is off here, as `requests::request_open` sets it; the binaries that turn it on are `tests/enforcement/` (the reference
//! policy and one signing key) and the `tests/enforcement_*.rs` files that need a policy or a setting of their own, each in
//! its own process because `WORKFORCE_PLANNING_MANAGEMENT_REQUIRE_AUTH`, the policy and the verifier are read once into a `OnceLock`.

mod checks;
mod requests;
