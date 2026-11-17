//! Sign-in on, under the reference ABAC policy, in one process (WPM-T534): the persona matrix and the suites that need only
//! that. `WPM_REQUIRE_AUTH`, the policy and the PASETO verifier are each read once into a `OnceLock`, so every module here sets
//! the same three settings before it boots and signs with the same throwaway key (seed 9). A suite that needs a policy or a
//! setting of its own is a `tests/enforcement_*.rs` file in its own process.
//!
//! `#[ignore]`d: it needs PostgreSQL. `cargo test --test enforcement -- --ignored --test-threads=1`.

mod course_requests;
mod duplicates;
mod engagements;
mod individual_goals;
mod injury_record;
mod learning_economics;
mod me;
mod need_to_know;
mod padr;
mod payroll_checks;
mod persona_matrix;
mod separation_record;
