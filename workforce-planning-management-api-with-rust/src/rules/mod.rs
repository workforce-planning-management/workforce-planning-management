//! The **pure core** (WPM-D3–D5): every lifecycle state machine, the
//! leave / time / scheduling arithmetic, the org-chart cycle check,
//! payslip arithmetic, and benchmark flags — DB-free, clock-free, and
//! exhaustively unit-tested. Controllers wire these; they never
//! re-implement them.

pub mod adjustments;
pub mod announcements;
pub mod appraisal;
pub mod assessment;
pub mod benchmark;
pub mod capability;
pub mod career;
pub mod change;
pub mod cost;
pub mod cover;
pub mod cpd;
pub mod csv;
pub mod directory;
pub mod emergency;
pub mod ergonomics;
pub mod esco;
pub mod framework;
pub mod framework_roles;
pub mod gap;
pub mod groups;
pub mod insights;
pub mod job_levels;
pub mod learning;
pub mod leave;
pub mod lifecycle;
pub mod lms;
pub mod metrics;
pub mod mobility;
pub mod movements;
pub mod notify;
pub mod org;
pub mod org_access;
pub mod pay_scale;
pub mod payroll;
pub mod planning;
pub mod privacy;
pub mod pulse;
pub mod roles;
pub mod rota;
pub mod skill_gap;
pub mod skill_merge;
pub mod talent;
pub mod tokens;
pub mod training;
pub mod wellbeing;
pub mod workforce;
pub mod working_time;
