//! Shared record helpers: parsing a path `pid` and reading the active (not soft-deleted) worker, plus the
//! `ActiveModelBehavior` impls the `SeaORM` entities still need to build; they go with the entities (WPM-T466).
//!
//! Both return the **`axa`** `Result` with `Error::NotFound` for an unparsable pid or a missing row, so every
//! `GET /…/{pid}` contract is an honest 404 (family lesson, pinned in the request tests).

use axa::prelude::*;
use uuid::Uuid;

use super::_entities::{
    applications, assessment_instruments, assessment_results, assessments, benchmarks,
    benefit_enrollments, benefit_plans, candidates, development_plan_items, development_plans,
    early_career_programs, feedback_entries, goals, interviews, leave_entitlements, leave_requests,
    onboarding_items, organization_confederations, organization_memberships, payroll_runs,
    payslips, pipeline_members, program_placements, requisitions, review_cycles, reviews,
    shift_assignments, shifts, succession_candidates, succession_plans, talent_pipelines,
    time_entries, training_enrollments, workers,
};

impl ActiveModelBehavior for workers::ActiveModel {}
impl ActiveModelBehavior for super::_entities::audit_logs::ActiveModel {}
impl ActiveModelBehavior for super::_entities::event_outbox::ActiveModel {}
impl ActiveModelBehavior for super::_entities::payslip_queries::ActiveModel {}
impl ActiveModelBehavior for super::_entities::development_plan_benefits::ActiveModel {}
impl ActiveModelBehavior for super::_entities::learning_budgets::ActiveModel {}
impl ActiveModelBehavior for super::_entities::worker_goals::ActiveModel {}
impl ActiveModelBehavior for super::_entities::padr_versions::ActiveModel {}
impl ActiveModelBehavior for super::_entities::separation_records::ActiveModel {}
impl ActiveModelBehavior for super::_entities::course_requests::ActiveModel {}
impl ActiveModelBehavior for super::_entities::injury_records::ActiveModel {}
impl ActiveModelBehavior for super::_entities::injury_reports::ActiveModel {}
impl ActiveModelBehavior for super::_entities::goal_measures::ActiveModel {}
impl ActiveModelBehavior for super::_entities::goal_check_ins::ActiveModel {}
impl ActiveModelBehavior for super::_entities::review_goal_outcomes::ActiveModel {}
impl ActiveModelBehavior for requisitions::ActiveModel {}
impl ActiveModelBehavior for assessment_instruments::ActiveModel {}
impl ActiveModelBehavior for assessments::ActiveModel {}
impl ActiveModelBehavior for assessment_results::ActiveModel {}
impl ActiveModelBehavior for development_plans::ActiveModel {}
impl ActiveModelBehavior for development_plan_items::ActiveModel {}
impl ActiveModelBehavior for talent_pipelines::ActiveModel {}
impl ActiveModelBehavior for pipeline_members::ActiveModel {}
impl ActiveModelBehavior for early_career_programs::ActiveModel {}
impl ActiveModelBehavior for program_placements::ActiveModel {}
impl ActiveModelBehavior for candidates::ActiveModel {}
impl ActiveModelBehavior for applications::ActiveModel {}
impl ActiveModelBehavior for interviews::ActiveModel {}
impl ActiveModelBehavior for onboarding_items::ActiveModel {}
impl ActiveModelBehavior for time_entries::ActiveModel {}
impl ActiveModelBehavior for leave_entitlements::ActiveModel {}
impl ActiveModelBehavior for leave_requests::ActiveModel {}
impl ActiveModelBehavior for shifts::ActiveModel {}
impl ActiveModelBehavior for shift_assignments::ActiveModel {}
impl ActiveModelBehavior for benefit_plans::ActiveModel {}
impl ActiveModelBehavior for benefit_enrollments::ActiveModel {}
impl ActiveModelBehavior for review_cycles::ActiveModel {}
impl ActiveModelBehavior for reviews::ActiveModel {}
impl ActiveModelBehavior for goals::ActiveModel {}
impl ActiveModelBehavior for feedback_entries::ActiveModel {}
impl ActiveModelBehavior for training_enrollments::ActiveModel {}
impl ActiveModelBehavior for succession_plans::ActiveModel {}
impl ActiveModelBehavior for succession_candidates::ActiveModel {}
impl ActiveModelBehavior for payroll_runs::ActiveModel {}
impl ActiveModelBehavior for payslips::ActiveModel {}
impl ActiveModelBehavior for benchmarks::ActiveModel {}
impl ActiveModelBehavior for organization_memberships::ActiveModel {}
impl ActiveModelBehavior for organization_confederations::ActiveModel {}

/// Parse a path `pid` string to a [`Uuid`], mapping failure to `404`
/// (an unparsable pid can never name a record).
///
/// # Errors
///
/// [`Error::NotFound`] when `pid` is not a UUID.
pub fn parse_pid(pid: &str) -> Result<Uuid> {
    Uuid::parse_str(pid).map_err(|_| Error::NotFound)
}

/// The active (not soft-deleted) worker by public id.
///
/// # Errors
///
/// [`Error::NotFound`] when absent or soft-deleted; any query error otherwise.
pub async fn worker_by_pid(ctx: &AppContext, pid: Uuid) -> Result<workers::Model> {
    sqlx::query_as!(
        workers::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, person_ref,
                  upstream_worker_ref, organization_ref, worker_number, display_name, status, employment_type,
                  fte_percent, department, job_title, location, work_jurisdiction, manager_pid, salary_minor,
                  salary_currency, hired_on, terminated_on, engagement_ends_on, deleted_at AS "deleted_at: _"
             FROM workers WHERE pid = $1 AND deleted_at IS NULL"#,
        pid,
    )
    .fetch_optional(crate::db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)
}
