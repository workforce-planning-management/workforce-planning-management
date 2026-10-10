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
mod m20261002_000029_plan_budget;
mod m20261002_000030_capability_frameworks;
mod m20261003_000031_skill_external_refs;
mod m20261003_000032_esco;
mod m20261003_000033_worker_framework_roles;
mod m20261003_000034_career_history;
mod m20261003_000035_aspiration_visibility;
mod m20261003_000036_groups;
mod m20261003_000037_dotted_line_reports;
mod m20261003_000038_group_organizations;
mod m20261003_000039_group_scope;
mod m20261005_000040_emergency_contacts_and_backups;
mod m20261005_000041_oncall_rotas;
mod m20261005_000042_rota_swap_requests;
mod m20261005_000043_announcements;
mod m20261005_000044_announcement_extras;
mod m20261006_000045_training_time;
mod m20261006_000046_movements;
mod m20261006_000047_grades;
mod m20261006_000048_pay_positions;
mod m20261006_000049_expense_claims;
mod m20261009_000050_generic_pay_scale_id;
mod m20261010_000051_erasure_ledger;
mod m20261011_000052_delivery_capacity;
mod m20261011_000053_engagements;
mod m20261011_000054_scrub_sick_leave_reasons;
mod m20261011_000055_audit_chain;
mod m20261011_000056_contact_details;
mod m20261011_000057_resignations;
mod m20261011_000058_flexible_working;
mod m20261011_000059_equality_declarations;
mod m20261011_000060_health_requirements;
mod m20261012_000061_engagement_reminders;
mod m20261012_000062_conversion_plans;

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
            Box::new(m20261002_000029_plan_budget::Migration),
            Box::new(m20261002_000030_capability_frameworks::Migration),
            Box::new(m20261003_000031_skill_external_refs::Migration),
            Box::new(m20261003_000032_esco::Migration),
            Box::new(m20261003_000033_worker_framework_roles::Migration),
            Box::new(m20261003_000034_career_history::Migration),
            Box::new(m20261003_000035_aspiration_visibility::Migration),
            Box::new(m20261003_000036_groups::Migration),
            Box::new(m20261003_000037_dotted_line_reports::Migration),
            Box::new(m20261003_000038_group_organizations::Migration),
            Box::new(m20261003_000039_group_scope::Migration),
            Box::new(m20261005_000040_emergency_contacts_and_backups::Migration),
            Box::new(m20261005_000041_oncall_rotas::Migration),
            Box::new(m20261005_000042_rota_swap_requests::Migration),
            Box::new(m20261005_000043_announcements::Migration),
            Box::new(m20261005_000044_announcement_extras::Migration),
            Box::new(m20261006_000045_training_time::Migration),
            Box::new(m20261006_000046_movements::Migration),
            Box::new(m20261006_000047_grades::Migration),
            Box::new(m20261006_000048_pay_positions::Migration),
            Box::new(m20261006_000049_expense_claims::Migration),
            Box::new(m20261009_000050_generic_pay_scale_id::Migration),
            Box::new(m20261010_000051_erasure_ledger::Migration),
            Box::new(m20261011_000052_delivery_capacity::Migration),
            Box::new(m20261011_000053_engagements::Migration),
            Box::new(m20261011_000054_scrub_sick_leave_reasons::Migration),
            Box::new(m20261011_000055_audit_chain::Migration),
            Box::new(m20261011_000056_contact_details::Migration),
            Box::new(m20261011_000057_resignations::Migration),
            Box::new(m20261011_000058_flexible_working::Migration),
            Box::new(m20261011_000059_equality_declarations::Migration),
            Box::new(m20261011_000060_health_requirements::Migration),
            Box::new(m20261012_000061_engagement_reminders::Migration),
            Box::new(m20261012_000062_conversion_plans::Migration),
            // inject-above (do not remove this comment)
        ]
    }
}
