//! `sea-orm-migration` schema migrations for
//! `workforce-planning-management-service`.
//!
//! The [`Migrator`] runs the ordered list below at boot (or via
//! `cargo loco db migrate`): the worker core, the acquisition
//! pipeline, the workforce tables, benefits + development, payroll,
//! then the `audit_logs` and `event_outbox` side tables. Employment
//! data is personal data, so the audit trail is the who/what/when
//! record over every change and every sensitive read.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![warn(clippy::pedantic)]
#![allow(elided_lifetimes_in_paths)]
pub use sea_orm_migration::prelude::*;
mod m20220101_000001_workers;
mod m20220101_000002_acquisition;
mod m20220101_000003_workforce;
mod m20220101_000004_development;
mod m20220101_000005_payroll;
mod m20220101_000006_audit_logs;
mod m20220101_000007_event_outbox;
mod m20260720_000008_learning;
mod m20260723_000009_assessments;
mod m20260723_000010_talent;
mod m20260724_000011_wellbeing;
mod m20260725_000012_benefits_awareness;
mod m20260725_000013_pulse;
mod m20260725_000014_appraisals;
mod m20260725_000015_notifications;
mod m20260725_000016_ergonomics;
mod m20260725_000017_adjustments;
mod m20260928_000018_organization_memberships;
mod m20260929_000019_organization_confederations;
mod m20261002_000020_worker_location;
mod m20261002_000021_requisition_filled_on;
mod m20261002_000022_headcount_snapshots;
mod m20261002_000023_role_profiles;
mod m20261002_000024_cpd;
mod m20261002_000025_mobility_interests;
mod m20261002_000026_change_initiatives;
mod m20261002_000027_workforce_plans;
mod m20261002_000028_plan_objectives;

/// The crate's migrator: drives the ordered migration set for the loco
/// CLI / boot-time migration.
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    /// The ordered migration set. Order matters: workers first (most
    /// tables reference them), then the pillar tables, then the side
    /// tables.
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_workers::Migration),
            Box::new(m20220101_000002_acquisition::Migration),
            Box::new(m20220101_000003_workforce::Migration),
            Box::new(m20220101_000004_development::Migration),
            Box::new(m20220101_000005_payroll::Migration),
            Box::new(m20220101_000006_audit_logs::Migration),
            Box::new(m20220101_000007_event_outbox::Migration),
            Box::new(m20260720_000008_learning::Migration),
            Box::new(m20260723_000009_assessments::Migration),
            Box::new(m20260723_000010_talent::Migration),
            Box::new(m20260724_000011_wellbeing::Migration),
            Box::new(m20260725_000012_benefits_awareness::Migration),
            Box::new(m20260725_000013_pulse::Migration),
            Box::new(m20260725_000014_appraisals::Migration),
            Box::new(m20260725_000015_notifications::Migration),
            Box::new(m20260725_000016_ergonomics::Migration),
            Box::new(m20260725_000017_adjustments::Migration),
            Box::new(m20260928_000018_organization_memberships::Migration),
            Box::new(m20260929_000019_organization_confederations::Migration),
            Box::new(m20261002_000020_worker_location::Migration),
            Box::new(m20261002_000021_requisition_filled_on::Migration),
            Box::new(m20261002_000022_headcount_snapshots::Migration),
            Box::new(m20261002_000023_role_profiles::Migration),
            Box::new(m20261002_000024_cpd::Migration),
            Box::new(m20261002_000025_mobility_interests::Migration),
            Box::new(m20261002_000026_change_initiatives::Migration),
            Box::new(m20261002_000027_workforce_plans::Migration),
            Box::new(m20261002_000028_plan_objectives::Migration),
            // inject-above (do not remove this comment)
        ]
    }
}
