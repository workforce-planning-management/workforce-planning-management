//! `workforce-planning-management` — a service on Axum and `axa`
//! for **workforce planning management** across the worker lifecycle:
//! requisitions and the applicant-tracking pipeline, onboarding
//! checklists, worker records with the derived org chart, time &
//! attendance, leave, shift scheduling, benefits, performance reviews,
//! training enrollments, succession planning, payroll runs with
//! derived payslips, and salary benchmarking.
//!
//! WPM **owns the employment relationship and its operational state**
//! (its own tables) and **references identities**: humans are
//! person-service records, professional identities worker-service,
//! employers organization-service, training courses course-service —
//! as `EntityRef` URNs, never duplicated demographics.
//!
//! ## Modules
//!
//! - [`app`] — the wiring: configuration, the command line, routes, truncate.
//! - [`controllers`] — Axum controllers, one per pillar.
//! - [`rules`] — the **pure core**: lifecycle machines, leave / time
//!   arithmetic, org-chart cycle check, payslip arithmetic, benchmark
//!   flags. DB-free and exhaustively unit-tested.
//! - [`models`] — row types (one per table) + record helpers.
//! - [`clients`] — upstream service lookups (stub-first).
//! - [`auth`] — bearer-token verification (PASETO or Keycloak JWT,
//!   `paseto`/`keycloak` Cargo feature) + ABAC + masking.
//! - [`streaming`] — event envelope + in-memory / outbox transports.
//!
//! See `../spec/index.md` for the living specification.

#![warn(clippy::pedantic)]
// No panic in production code: a failure is an error value. Tests may (clippy.toml).
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unreachable,
    clippy::todo,
    clippy::unimplemented,
    clippy::indexing_slicing,
    clippy::string_slice,
    clippy::arithmetic_side_effects,
    // Each exception is `#[expect]`ed, with a reason (RCA-EXC-6).
    clippy::allow_attributes,
    clippy::allow_attributes_without_reason,
    // Every literal names its type, so no figure silently becomes an `i32` or `f64` (RCA-INT).
    clippy::default_numeric_fallback
)]
#![cfg_attr(
    not(test),
    deny(
        // RCA-ENF-1: division and remainder are checked, conversions are `From` or `TryFrom`, floats are finite.
        clippy::integer_division,
        clippy::modulo_arithmetic,
        clippy::as_conversions,
        clippy::cast_lossless,
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        clippy::cast_precision_loss,
        clippy::cast_sign_loss,
        clippy::checked_conversions,
        clippy::float_cmp,
        clippy::lossy_float_literal,
        // The service does no float arithmetic: a ratio is the domain's, checked for finiteness (RCA-FLT-2).
        clippy::float_arithmetic
    )
)]
#![cfg_attr(
    test,
    expect(
        clippy::string_slice,
        clippy::arithmetic_side_effects,
        clippy::default_numeric_fallback,
        reason = "RCA-EXC-5: test code may use plain operators and slice strings"
    )
)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
// The hand-written OpenAPI document is one large nested `json!` literal.
#![recursion_limit = "2048"]

// `sqlx.toml` names the row date types by a global path (`::workforce_planning_management::sql_time::…`), which must
// resolve inside this crate as well as in its tests.
extern crate self as workforce_planning_management;

pub mod absent;
pub mod app;
pub mod audit_export;
pub mod auth;
pub mod clients;
/// Backward-compatibility shims for the 2026-07-23 `HCM` → `WPM` rename.
pub mod compat;
pub mod controllers;
/// The date helpers, from the domain crate (WPM-D203).
pub use workforce_planning_management_domain::dates;
pub mod db;
pub mod hardening;
pub mod idempotency;
pub mod metrics;
pub mod migrations;
pub mod models;
pub mod money_boundary;
pub mod need_to_know;
pub mod openapi;
pub mod problem;
pub mod request_log;
/// The **pure core**: the domain logic and business rules, in their own crate (WPM-D203).
pub use workforce_planning_management_domain as rules;
pub mod scheduler;
pub mod security;
pub mod settings;
pub mod shared_cache;
pub mod sql_time;
pub mod statutory_defaults;
pub mod streaming;
pub mod tasks;
pub mod validation;
/// Header-based API versioning (`Accepts-version`) for the REST surface.
pub mod version;

/// The pure core names no country (WPM-T369): a source-text check over `crates/workforce-planning-management-domain/src/`.
#[cfg(test)]
mod jurisdiction_neutrality;

#[cfg(test)]
mod source_text_guards {
    use std::path::Path;

    fn rust_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
        for entry in std::fs::read_dir(dir).expect("source directory") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                rust_files(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }

    /// A line ending in two backslashes inside a string literal is a literal backslash and a line break in the text a client
    /// reads, not a line continuation (which is one backslash). It showed in a derivation as "in the same \ category".
    #[test]
    fn no_line_of_source_ends_in_a_doubled_backslash() {
        let mut files = Vec::new();
        // The service's own source and the domain crate's (WPM-D203).
        for dir in ["src", "crates/workforce-planning-management-domain/src"] {
            rust_files(&Path::new(env!("CARGO_MANIFEST_DIR")).join(dir), &mut files);
        }
        let mut found = Vec::new();
        for file in files {
            let text = std::fs::read_to_string(&file).expect("readable source");
            for (i, line) in text.lines().enumerate() {
                // This test's own explanation and a lone escaped backslash in a char or short literal are not the fault.
                if line.ends_with("\\\\")
                    && !line.trim_start().starts_with("//")
                    && !line.ends_with("'\\\\'")
                {
                    found.push(format!("{}:{}", file.display(), i + 1));
                }
            }
        }
        assert!(
            found.is_empty(),
            "a doubled backslash ends a line (use one for a continuation): {found:?}"
        );
    }
}
