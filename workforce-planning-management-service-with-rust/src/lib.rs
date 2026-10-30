//! `workforce-planning-management` — a loco.rs consumer application
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
//! - [`app`] — loco `Hooks` wiring (routes, truncate, seed).
//! - [`controllers`] — Axum controllers, one per pillar.
//! - [`rules`] — the **pure core**: lifecycle machines, leave / time
//!   arithmetic, org-chart cycle check, payslip arithmetic, benchmark
//!   flags. DB-free and exhaustively unit-tested.
//! - [`models`] — `SeaORM` entities + CRUD helpers.
//! - [`clients`] — upstream service lookups (stub-first).
//! - [`auth`] — bearer-token verification (PASETO or Keycloak JWT,
//!   `paseto`/`keycloak` Cargo feature) + ABAC + masking.
//! - [`streaming`] — event envelope + in-memory / outbox transports.
//!
//! See `../spec/index.md` for the living specification.

#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
// The hand-written OpenAPI document is one large nested `json!` literal.
#![recursion_limit = "2048"]

pub mod app;
pub mod auth;
pub mod clients;
/// Backward-compatibility shims for the 2026-07-23 `HCM` → `WPM` rename.
pub mod compat;
pub mod controllers;
pub mod hardening;
pub mod idempotency;
pub mod metrics;
pub mod models;
pub mod need_to_know;
pub mod openapi;
pub mod request_log;
pub mod rules;
pub mod security;
pub mod shared_cache;
pub mod streaming;
pub mod tasks;
pub mod validation;
/// Header-based API versioning (`Accepts-version`) for the REST surface.
pub mod version;

/// The pure core names no country (WPM-T369): a source-text check over `src/rules/`.
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
        rust_files(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
            &mut files,
        );
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
