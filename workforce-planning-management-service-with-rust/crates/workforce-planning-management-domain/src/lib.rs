//! `workforce-planning-management-domain` — the **domain logic and business rules** of Workforce Planning Management (WPM-D203).
//!
//! The **pure core** (WPM-D3–D5): every lifecycle state machine, the
//! leave / time / scheduling arithmetic, the org-chart cycle check,
//! payslip arithmetic, and benchmark flags — DB-free, clock-free, and
//! exhaustively unit-tested. Controllers wire these; they never
//! re-implement them.

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
        clippy::lossy_float_literal
    )
)]
#![cfg_attr(
    test,
    expect(
        clippy::string_slice,
        clippy::arithmetic_side_effects,
        clippy::default_numeric_fallback,
        reason = "RCA-EXC-5: tests may use plain operators and slicing"
    )
)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod absence_category;
pub mod access;
pub mod adjustments;
pub mod agreement;
pub mod ai_register;
pub mod allocation;
pub mod announcements;
pub mod appraisal;
pub mod assessment;
pub mod assignment;
pub mod bargaining;
pub mod bargaining_pack;
pub mod benchmark;
pub mod bridge;
pub mod capability;
pub mod capacity;
pub mod career;
pub mod change;
pub mod coding;
pub mod consultation;
pub mod consultation_register;
pub mod contact_details;
pub mod conversion;
pub mod cost;
pub mod course_request;
pub mod cover;
pub mod coverage_evidence;
pub mod cpd;
pub mod credential;
pub mod crosswalk;
pub mod csv;
pub mod currency;
pub mod cutover;
pub mod dates;
pub mod decision_gate;
pub mod derived;
pub mod directory;
pub mod display;
pub mod duplicates;
pub mod early_warning;
pub mod emergency;
pub mod employer_contribution;
pub mod engagement;
pub mod equality_monitoring;
pub mod equipment;
pub mod ergonomics;
pub mod escalation;
pub mod esco;
pub mod exact_float;
pub mod expenses;
pub mod flexible_working;
pub mod framework;
pub mod framework_roles;
pub mod funding;
pub mod gap;
pub mod governance;
pub mod grade;
pub mod groups;
pub mod health_requirements;
pub mod holiday;
pub mod human_capital;
pub mod idempotency;
pub mod individual_goals;
pub mod injury_record;
pub mod insights;
pub mod job_levels;
pub mod joint_assessment;
pub mod jurisdiction;
pub mod key_person;
pub mod learning;
pub mod leave;
pub mod lifecycle;
pub mod link_token;
pub mod lms;
pub mod manager_change;
pub mod mandatory_learning;
pub mod mcp;
pub mod metrics;
pub mod minimum_wage;
pub mod mobility;
pub mod movements;
pub mod my_payslips;
pub mod notify;
pub mod org;
pub mod org_access;
pub mod org_shape;
pub mod oversight;
pub mod padr;
pub mod page_limit;
pub mod pay_category;
pub mod pay_gap;
pub mod pay_information;
pub mod pay_position;
pub mod pay_range;
pub mod pay_scale;
pub mod payroll;
pub mod people_impact;
pub mod phone;
pub mod planning;
pub mod policy;
pub mod position;
pub mod position_history;
pub mod privacy;
pub mod programmes;
pub mod pulse;
pub mod qualification;
pub mod resignation;
pub mod roles;
pub mod rota;
pub mod scale;
pub mod scim;
pub mod scoring;
#[cfg(test)]
mod second_profile;
pub mod self_service;
pub mod separation;
pub mod service;
pub mod sfia;
pub mod skill_gap;
pub mod skill_merge;
pub mod statutory_overtime;
pub mod suppression;
pub mod talent;
pub mod time_off;
pub mod tokens;
pub mod tolerance;
pub mod training;
pub mod training_economics;
pub mod webhook;
pub mod wellbeing;
pub mod worker_fields;
pub mod workforce;
pub mod working_time;
