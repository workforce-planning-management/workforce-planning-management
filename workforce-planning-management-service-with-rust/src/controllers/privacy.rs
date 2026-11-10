//! Subject rights & retention (WPM-R30 / WPM-D22): the subject-access
//! export, erasure-as-anonymisation, and the retention report + sweep.
//! Erasure and the sweep are destructive-classified POSTs (`/erase`,
//! `/sweep` — `access=admin` under enforcement). WPM covers its own
//! store only: what the upstream identity services hold is the
//! deployment's coordination duty, stated in the payloads.

use axa::extract::Path;
use axa::prelude::*;
use uuid::Uuid;

use super::statutory_defaults::{self as statutory, InForce};
use super::{forbidden, record_rejection, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::models::_entities::{
    adjustment_requests, announcement_reads, applications, appraisal_nominations,
    appraisal_responses, appraisals, assessment_results, assessments, asset_issues, assignments,
    benefit_enrollments, candidates, conversion_plans, course_requests, cpd_entries,
    credential_issuances, development_plan_benefits, development_plans, dotted_line_reports,
    emergency_contacts, engagement_decisions, engagement_extensions, engagement_reminders,
    engagement_status_assessments, entitlement_acknowledgements, equality_declarations,
    ergonomic_assessments, expense_claims, expense_items, flexible_working_requests,
    goal_check_ins, goal_measures, group_members, handover_actions, injury_records, interviews,
    learning_completions, leave_entitlements, leave_requests, mentorships, mobility_interests,
    movements, notifications, onboarding_items, organization_memberships, padr_versions,
    path_enrollments, pay_information_requests, payslip_queries, payslips, pipeline_members,
    policy_acknowledgements, positions, post_fundings, professional_registrations,
    program_placements, qualification_reminders, qualifications, resignations,
    review_goal_outcomes, reviews, rota_members, rota_overrides, rota_swap_requests, scim_pushes,
    separation_records, shift_assignments, time_entries, training_enrollments, worker_aspirations,
    worker_backups, worker_contact_details, worker_contractor_details, worker_framework_roles,
    worker_goals, worker_health_records, worker_job_levels, worker_pay_positions,
    worker_skill_history, worker_skills,
};
use crate::models::records;
use crate::rules::privacy as rules;

/// The tombstone URN an erased worker's `person_ref` becomes: a
/// syntactically valid `EntityRef` that resolves to no one.
const TOMBSTONE_PERSON: &str = "person:00000000-0000-0000-0000-000000000000";

/// The scrub placeholder for erased free text and names.
const ERASED: &str = "[erased]";

/// Rows keyed by one subject in `module` (helper macro: run the given statement, which aggregates the table's rows as JSON, on
/// `tx`, and return them as that table's models, so the export has the model's shape, field order and formats).
macro_rules! rows_for {
    ($tx:ident, $module:ident, $sql:tt, $pid:expr) => {
        serde_json::json!(as_models::<$module::Model>(
            sqlx::query_scalar!($sql, $pid).fetch_one(&mut *$tx).await?
        )?)
    };
}

/// Read a JSON array of a table's rows as its models.
fn as_models<T: serde::de::DeserializeOwned>(rows: serde_json::Value) -> Result<Vec<T>> {
    serde_json::from_value(rows).map_err(|_| super::internal("a row does not read as its model"))
}

/// A read transaction whose times are in UTC: the rows are aggregated as JSON in the database, and a time read through a model
/// is in UTC, so the export's times keep the offset they had when each row was read as a model.
async fn utc_reader(ctx: &AppContext) -> Result<sqlx::Transaction<'static, sqlx::Postgres>> {
    let mut tx = crate::db::pool(ctx).begin().await?;
    sqlx::raw_sql("SET LOCAL TIME ZONE 'UTC'")
        .execute(&mut *tx)
        .await?;
    Ok(tx)
}

/// The worker's weekly time totals, as a JSON array. The table has no entity (it is a derived summary keyed by worker and week),
/// so its rows are sent as the database writes them, in the session's own time zone.
async fn weekly_time(ctx: &AppContext, worker: Uuid) -> Result<serde_json::Value> {
    Ok(sqlx::query_scalar!(
        r#"SELECT COALESCE(json_agg(t ORDER BY week_start), '[]'::json) AS "rows!" FROM time_entry_weeks t WHERE worker_pid = $1"#,
        worker,
    )
    .fetch_one(crate::db::pool(ctx))
    .await?)
}

/// `GET /api/workers/{pid}/subject-access` — everything WPM holds
/// keyed to this worker, in one audited JSON document. Exclusions
/// are named, not hidden: pulse responses (no author link exists,
/// WPM-D20) and other raters' 360° content about the subject
/// (third-party data; the shared report aggregate stands in).
#[debug_handler]
#[allow(clippy::too_many_lines)] // one gather over every table
async fn subject_access(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    let obligations = auth::authorize_record(
        &caller,
        authentication_verifier::Action::Read,
        &auth::worker_resource_attrs(&worker),
    )
    .map_err(record_rejection)?;
    // A masked read of a *full export* would be a contradiction — the
    // export exists to disclose everything. Refuse rather than leak:
    // subject access is for the subject (`$sub`) and unmasked-read
    // personas (payroll/admin/svc), never the masked fallback.
    if obligations.iter().any(|o| o == "mask") {
        return Err(forbidden(
            "subject access requires an unmasked read (the export discloses \
             salary and payslips; masked callers cannot receive it)",
        ));
    }
    let epid = worker.pid;
    let weeks = weekly_time(&ctx, epid).await?;
    let mut db = utc_reader(&ctx).await?;
    // 360°: appraisals about them, nominations naming them as rater,
    // and only the responses THEY authored.
    let their_nominations = rows_for!(
        db,
        appraisal_nominations,
        r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM appraisal_nominations t WHERE rater_pid = $1"#,
        epid
    );
    let nomination_pids: Vec<Uuid> = their_nominations
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|n| n["pid"].as_str()?.parse().ok())
        .collect();
    let authored_responses = rows_for!(
        db,
        appraisal_responses,
        r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM appraisal_responses t WHERE nomination_pid = ANY($1)"#,
        &nomination_pids
    );
    let mentorship_rows = rows_for!(
        db,
        mentorships,
        r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM mentorships t WHERE mentor_pid = $1 OR mentee_pid = $1"#,
        epid
    );
    let export = serde_json::json!({
        "as_of": chrono::Utc::now(),
        "worker": worker,
        "time_entries": rows_for!(db, time_entries, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM time_entries t WHERE worker_pid = $1"#, epid),
        "leave_entitlements": rows_for!(db, leave_entitlements, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM leave_entitlements t WHERE worker_pid = $1"#, epid),
        "leave_requests": rows_for!(db, leave_requests, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM leave_requests t WHERE worker_pid = $1"#, epid),
        "shift_assignments": rows_for!(db, shift_assignments, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM shift_assignments t WHERE worker_pid = $1"#, epid),
        "benefit_enrollments": rows_for!(db, benefit_enrollments, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM benefit_enrollments t WHERE worker_pid = $1"#, epid),
        "reviews": rows_for!(db, reviews, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM reviews t WHERE worker_pid = $1"#, epid),
        "training_enrollments": rows_for!(db, training_enrollments, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM training_enrollments t WHERE worker_pid = $1"#, epid),
        "skills": rows_for!(db, worker_skills, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM worker_skills t WHERE worker_pid = $1"#, epid),
        "learning_path_enrollments": rows_for!(db, path_enrollments, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM path_enrollments t WHERE worker_pid = $1"#, epid),
        "development_plans": rows_for!(db, development_plans, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM development_plans t WHERE worker_pid = $1"#, epid),
        "development_plan_benefits": rows_for!(db, development_plan_benefits, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM development_plan_benefits t WHERE worker_pid = $1"#, epid),
        "worker_goals": rows_for!(db, worker_goals, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM worker_goals t WHERE worker_pid = $1"#, epid),
        "padr_versions": rows_for!(db, padr_versions, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM padr_versions t WHERE worker_pid = $1"#, epid),
        "goal_measures": rows_for!(db, goal_measures, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM goal_measures t WHERE worker_pid = $1"#, epid),
        "goal_check_ins": rows_for!(db, goal_check_ins, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM goal_check_ins t WHERE worker_pid = $1"#, epid),
        "review_goal_outcomes": rows_for!(db, review_goal_outcomes, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM review_goal_outcomes t WHERE worker_pid = $1"#, epid),
        "cpd_entries": rows_for!(db, cpd_entries, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM cpd_entries t WHERE worker_pid = $1"#, epid),
        "framework_roles": rows_for!(db, worker_framework_roles, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM worker_framework_roles t WHERE worker_pid = $1"#, epid),
        "skill_history": rows_for!(db, worker_skill_history, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM worker_skill_history t WHERE worker_pid = $1"#, epid),
        "aspirations": rows_for!(db, worker_aspirations, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM worker_aspirations t WHERE worker_pid = $1"#, epid),
        "emergency_contacts": rows_for!(db, emergency_contacts, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM emergency_contacts t WHERE worker_pid = $1"#, epid),
        "job_level": rows_for!(db, worker_job_levels, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM worker_job_levels t WHERE worker_pid = $1"#, epid),
        "pay_position": rows_for!(db, worker_pay_positions, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM worker_pay_positions t WHERE worker_pid = $1"#, epid),
        "contact_details": rows_for!(db, worker_contact_details, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM worker_contact_details t WHERE worker_pid = $1"#, epid),
        "resignations": rows_for!(db, resignations, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM resignations t WHERE worker_pid = $1"#, epid),
        "conversion_plans": rows_for!(db, conversion_plans, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM conversion_plans t WHERE worker_pid = $1"#, epid),
        "post_funding": rows_for!(db, post_fundings, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM post_fundings t WHERE worker_pid = $1"#, epid),
        "engagement_decisions": rows_for!(db, engagement_decisions, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM engagement_decisions t WHERE worker_pid = $1"#, epid),
        "engagement_reminders": rows_for!(db, engagement_reminders, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM engagement_reminders t WHERE worker_pid = $1"#, epid),
        // Health data readable by the worker and occupational health only (WPM-D75): anyone else
        // running the export gets a note.
        "workplace_health_records": if auth::acting_for_other(&caller, &worker.person_ref) {
            serde_json::json!("withheld: only the worker can export their own workplace health records")
        } else {
            rows_for!(db, worker_health_records, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM worker_health_records t WHERE worker_pid = $1"#, epid)
        },
        // Special-category data readable by the worker only (WPM-D74): anyone else running the
        // export gets a note, not the answers.
        "equality_monitoring": if auth::acting_for_other(&caller, &worker.person_ref) {
            serde_json::json!("withheld: only the worker can export their own equality monitoring answers")
        } else {
            rows_for!(db, equality_declarations, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM equality_declarations t WHERE worker_pid = $1"#, epid)
        },
        "flexible_working_requests": rows_for!(db, flexible_working_requests, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM flexible_working_requests t WHERE worker_pid = $1"#, epid),
        "pay_information_requests": rows_for!(db, pay_information_requests, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM pay_information_requests t WHERE worker_pid = $1"#, epid),
        "scim_pushes": rows_for!(db, scim_pushes, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM scim_pushes t WHERE worker_pid = $1"#, epid),
        "asset_issues": rows_for!(db, asset_issues, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM asset_issues t WHERE worker_pid = $1"#, epid),
        "assignments": rows_for!(db, assignments, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM assignments t WHERE worker_pid = $1"#, epid),
        "learning_completions": rows_for!(db, learning_completions, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM learning_completions t WHERE worker_pid = $1"#, epid),
        "qualifications": rows_for!(db, qualifications, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM qualifications t WHERE worker_pid = $1"#, epid),
        "qualification_reminders": rows_for!(db, qualification_reminders, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM qualification_reminders t WHERE worker_pid = $1"#, epid),
        "policy_acknowledgements": rows_for!(db, policy_acknowledgements, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM policy_acknowledgements t WHERE worker_pid = $1"#, epid),
        "credential_issuances": rows_for!(db, credential_issuances, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM credential_issuances t WHERE worker_pid = $1"#, epid),
        "positions_held": rows_for!(db, positions, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM positions t WHERE holder_worker_pid = $1"#, epid),
        "engagement_extensions": rows_for!(db, engagement_extensions, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM engagement_extensions t WHERE worker_pid = $1"#, epid),
        "contractor_details": rows_for!(db, worker_contractor_details, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM worker_contractor_details t WHERE worker_pid = $1"#, epid),
        "employment_status_assessments": rows_for!(db, engagement_status_assessments, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM engagement_status_assessments t WHERE worker_pid = $1"#, epid),
        "expense_claims": rows_for!(db, expense_claims, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM expense_claims t WHERE worker_pid = $1"#, epid),
        "expense_items": rows_for!(db, expense_items, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM expense_items t WHERE worker_pid = $1"#, epid),
        "backups": rows_for!(db, worker_backups, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM worker_backups t WHERE worker_pid = $1"#, epid),
        "named_as_backup_by": rows_for!(db, worker_backups, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM worker_backups t WHERE backup_pid = $1"#, epid),
        "on_call_rota_memberships": rows_for!(db, rota_members, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM rota_members t WHERE worker_pid = $1"#, epid),
        "on_call_swaps": rows_for!(db, rota_overrides, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM rota_overrides t WHERE worker_pid = $1"#, epid),
        "announcements_read": rows_for!(db, announcement_reads, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM announcement_reads t WHERE worker_pid = $1"#, epid),
        "joiner_leaver_records": rows_for!(db, movements, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM movements t WHERE worker_pid = $1"#, epid),
        "separation_records": rows_for!(db, separation_records, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM separation_records t WHERE worker_pid = $1"#, epid),
        "course_requests": rows_for!(db, course_requests, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM course_requests t WHERE worker_pid = $1"#, epid),
        "injury_records": rows_for!(db, injury_records, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM injury_records t WHERE worker_pid = $1"#, epid),
        "handover_actions_from": rows_for!(db, handover_actions, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM handover_actions t WHERE from_worker = $1"#, epid),
        "handover_actions_to": rows_for!(db, handover_actions, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM handover_actions t WHERE to_worker = $1"#, epid),
        "on_call_swap_requests_made": rows_for!(db, rota_swap_requests, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM rota_swap_requests t WHERE requester_pid = $1"#, epid),
        "on_call_swap_requests_received": rows_for!(db, rota_swap_requests, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM rota_swap_requests t WHERE taker_pid = $1"#, epid),
        "group_memberships": rows_for!(db, group_members, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM group_members t WHERE worker_pid = $1"#, epid),
        "dotted_line_as_report": rows_for!(db, dotted_line_reports, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM dotted_line_reports t WHERE report_pid = $1"#, epid),
        "dotted_line_as_manager": rows_for!(db, dotted_line_reports, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM dotted_line_reports t WHERE manager_pid = $1"#, epid),
        "mobility_interests": rows_for!(db, mobility_interests, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM mobility_interests t WHERE worker_pid = $1"#, epid),
        "professional_registrations": rows_for!(db, professional_registrations, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM professional_registrations t WHERE worker_pid = $1"#, epid),
        "program_placements": rows_for!(db, program_placements, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM program_placements t WHERE worker_pid = $1"#, epid),
        "payslips": rows_for!(db, payslips, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM payslips t WHERE worker_pid = $1"#, epid),
        "payslip_queries": rows_for!(db, payslip_queries, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM payslip_queries t WHERE worker_pid = $1"#, epid),
        "onboarding_items": rows_for!(db, onboarding_items, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM onboarding_items t WHERE worker_pid = $1"#, epid),
        "organization_memberships": rows_for!(db, organization_memberships, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM organization_memberships t WHERE worker_pid = $1"#, epid),
        "time_entry_weeks": weeks,
        "wellbeing_acknowledgements":
            rows_for!(db, entitlement_acknowledgements, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM entitlement_acknowledgements t WHERE worker_pid = $1"#, epid),
        "notifications": rows_for!(db, notifications, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM notifications t WHERE worker_pid = $1"#, epid),
        "ergonomic_assessments": rows_for!(db, ergonomic_assessments, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM ergonomic_assessments t WHERE worker_pid = $1"#, epid),
        "adjustment_requests": rows_for!(db, adjustment_requests, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM adjustment_requests t WHERE worker_pid = $1"#, epid),
        "assessments": rows_for!(db, assessments, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM assessments t WHERE subject_pid = $1"#, epid),
        "pipeline_memberships": rows_for!(db, pipeline_members, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM pipeline_members t WHERE subject_pid = $1"#, epid),
        "mentorships": mentorship_rows,
        "appraisals_as_subject": rows_for!(db, appraisals, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM appraisals t WHERE worker_pid = $1"#, epid),
        "appraisal_nominations_as_rater": their_nominations,
        "appraisal_responses_authored": authored_responses,
        "exclusions": [
            "pulse responses: structurally impossible — responses store no author (WPM-D20)",
            "other raters' 360 content about the subject: third-party data; the shared \
             report aggregate stands in (WPM-D21)",
            "upstream person/worker/organization records: held by the identity services, \
             not WPM — coordinate subject access there too (WPM-D22)",
        ],
    });
    db.commit().await?;
    let mut connection = crate::db::pool(&ctx).acquire().await?;
    crate::db::audit(
        &mut connection,
        "worker",
        epid,
        "subject_access_exported",
        caller.actor(),
        None,
    )
    .await?;
    format::json(export)
}

/// `GET /api/candidates/{pid}/subject-access` — everything WPM holds about one applicant, in
/// one audited JSON document (WPM-T353). An applicant is a data subject before they are a worker:
/// the record, their applications, the interviews on them (the interviewers' notes are about the
/// applicant, so they are theirs to see), and their assessments with the results. Privileged
/// callers only: an applicant has no sign-in here, so HR makes the request on their behalf.
#[debug_handler]
async fn candidate_subject_access(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let candidate_pid = sqlx::query_scalar!(
        "SELECT pid FROM candidates WHERE pid = $1 AND deleted_at IS NULL",
        records::parse_pid(&pid)?,
    )
    .fetch_optional(crate::db::pool(&ctx))
    .await?
    .ok_or(Error::NotFound)?;
    let mut db = utc_reader(&ctx).await?;
    let apps = rows_for!(
        db,
        applications,
        r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM applications t WHERE candidate_pid = $1"#,
        candidate_pid
    );
    let app_pids: Vec<Uuid> = apps
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|a| a["pid"].as_str()?.parse().ok())
        .collect();
    let their_interviews = rows_for!(
        db,
        interviews,
        r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM interviews t WHERE application_pid = ANY($1)"#,
        &app_pids
    );
    let their_assessments: Vec<assessments::Model> = as_models(
        sqlx::query_scalar!(
            r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM assessments t WHERE subject_pid = $1"#,
            candidate_pid,
        )
        .fetch_one(&mut *db)
        .await?,
    )?;
    let assessment_pids: Vec<Uuid> = their_assessments.iter().map(|a| a.pid).collect();
    let results = rows_for!(
        db,
        assessment_results,
        r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM assessment_results t WHERE assessment_pid = ANY($1)"#,
        &assessment_pids
    );
    let export = serde_json::json!({
        "as_of": chrono::Utc::now(),
        "candidate": rows_for!(db, candidates, r#"SELECT COALESCE(json_agg(t), '[]') AS "rows!" FROM candidates t WHERE pid = $1"#, candidate_pid),
        "applications": apps,
        "interviews": their_interviews,
        "assessments": their_assessments,
        "assessment_results": results,
        "exclusions": [
            "the requisition an application is for: the employer's record, not the applicant's",
            "interviewers' identities: held as references (WPM-D22), not names; the notes they \
             wrote about the applicant are included",
            "upstream person records: held by the identity services, not WPM",
        ],
    });
    db.commit().await?;
    let mut tx = crate::db::pool(&ctx).begin().await?;
    crate::db::audit(
        &mut tx,
        "candidate",
        candidate_pid,
        "subject_access_exported",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(export)
}

/// The statements that scrub everything a worker authored or owns, and the rows
/// that are about them only (WPM-D22). One list, used by the live erasure and by
/// the replay after a restore (WPM-D63), so the two cannot drift. The order is
/// pinned: the audit snapshot indexes the row counts by position.
#[allow(clippy::too_many_lines)] // one ordered list; splitting it would risk the pinned order
fn erasure_statements(epid: Uuid) -> Vec<String> {
    vec![
        format!("UPDATE time_entries SET notes = NULL WHERE worker_pid = '{epid}'"),
        format!(
            "UPDATE appraisal_responses SET comment = NULL WHERE nomination_pid IN \
             (SELECT pid FROM appraisal_nominations WHERE rater_pid = '{epid}')"
        ),
        format!(
            "UPDATE mentorship_sessions SET notes = '{ERASED}' WHERE mentorship_pid IN \
             (SELECT pid FROM mentorships WHERE mentor_pid = '{epid}' OR mentee_pid = '{epid}')"
        ),
        format!(
            "UPDATE appraisals SET deleted_at = now() WHERE worker_pid = '{epid}' AND deleted_at IS NULL"
        ),
        format!("DELETE FROM entitlement_acknowledgements WHERE worker_pid = '{epid}'"),
        format!("DELETE FROM notifications WHERE worker_pid = '{epid}'"),
        format!(
            "UPDATE ergonomic_items SET note = NULL, deleted_at = now() WHERE assessment_pid IN \
             (SELECT pid FROM ergonomic_assessments WHERE worker_pid = '{epid}')"
        ),
        format!(
            "UPDATE ergonomic_assessments SET deleted_at = now() WHERE worker_pid = '{epid}' AND deleted_at IS NULL"
        ),
        format!(
            "UPDATE adjustment_requests SET barrier = '{ERASED}', impact = '{ERASED}', \
             adjustment = '{ERASED}', decision_note = NULL, deleted_at = now() \
             WHERE worker_pid = '{epid}' AND deleted_at IS NULL"
        ),
        format!(
            "UPDATE cpd_entries SET evidence_note = NULL, evidence_url = NULL, deleted_at = now() \
             WHERE worker_pid = '{epid}' AND deleted_at IS NULL"
        ),
        format!(
            "UPDATE professional_registrations SET reference = NULL, deleted_at = now() \
             WHERE worker_pid = '{epid}' AND deleted_at IS NULL"
        ),
        format!(
            "UPDATE mobility_interests SET note = NULL, deleted_at = now() \
             WHERE worker_pid = '{epid}' AND deleted_at IS NULL"
        ),
        format!("DELETE FROM worker_framework_roles WHERE worker_pid = '{epid}'"),
        format!("DELETE FROM worker_skill_history WHERE worker_pid = '{epid}'"),
        format!("DELETE FROM worker_aspirations WHERE worker_pid = '{epid}'"),
        format!("DELETE FROM group_members WHERE worker_pid = '{epid}'"),
        format!(
            "DELETE FROM dotted_line_reports WHERE report_pid = '{epid}' OR manager_pid = '{epid}'"
        ),
        format!("DELETE FROM emergency_contacts WHERE worker_pid = '{epid}'"),
        format!("DELETE FROM worker_backups WHERE worker_pid = '{epid}' OR backup_pid = '{epid}'"),
        format!("DELETE FROM rota_members WHERE worker_pid = '{epid}'"),
        format!("DELETE FROM rota_overrides WHERE worker_pid = '{epid}'"),
        format!(
            "DELETE FROM rota_swap_requests WHERE requester_pid = '{epid}' OR taker_pid = '{epid}'"
        ),
        format!("DELETE FROM announcement_reads WHERE worker_pid = '{epid}'"),
        format!("UPDATE movements SET notes = NULL, reason = NULL WHERE worker_pid = '{epid}'"),
        format!("UPDATE movement_items SET assignee_pid = NULL WHERE assignee_pid = '{epid}'"),
        format!(
            "UPDATE handover_actions SET note = NULL WHERE from_worker = '{epid}' OR to_worker = '{epid}'"
        ),
        format!("DELETE FROM worker_job_levels WHERE worker_pid = '{epid}'"),
        format!("DELETE FROM worker_pay_positions WHERE worker_pid = '{epid}'"),
        format!("DELETE FROM worker_contact_details WHERE worker_pid = '{epid}'"),
        format!("DELETE FROM resignations WHERE worker_pid = '{epid}'"),
        format!("DELETE FROM conversion_plans WHERE worker_pid = '{epid}'"),
        format!("DELETE FROM post_fundings WHERE worker_pid = '{epid}'"),
        format!("DELETE FROM engagement_decisions WHERE worker_pid = '{epid}'"),
        format!("DELETE FROM engagement_reminders WHERE worker_pid = '{epid}'"),
        format!("DELETE FROM worker_health_records WHERE worker_pid = '{epid}'"),
        format!("DELETE FROM equality_declarations WHERE worker_pid = '{epid}'"),
        format!("DELETE FROM flexible_working_requests WHERE worker_pid = '{epid}'"),
        format!("DELETE FROM pay_information_requests WHERE worker_pid = '{epid}'"),
        format!("DELETE FROM scim_pushes WHERE worker_pid = '{epid}'"),
        // The asset stays (it is the employer's); only who held it is erased.
        format!("DELETE FROM asset_issues WHERE worker_pid = '{epid}'"),
        format!("DELETE FROM assignments WHERE worker_pid = '{epid}'"),
        format!("DELETE FROM policy_acknowledgements WHERE worker_pid = '{epid}'"),
        // A credential the person holds cannot be taken back, so the issuance is revoked and cut from the person (its position in the
        // revocation list stays, or a later issuance would take the same place).
        format!(
            "UPDATE credential_issuances SET worker_pid = '00000000-0000-0000-0000-000000000000', source_pid = '00000000-0000-0000-0000-000000000000', revoked_at = COALESCE(revoked_at, now()), revoked_by = COALESCE(revoked_by, 'erasure') WHERE worker_pid = '{epid}'"
        ),
        // A seat outlives the person who held it (WPM-D104): only the link is cut.
        format!("UPDATE positions SET holder_worker_pid = NULL WHERE holder_worker_pid = '{epid}'"),
        format!("DELETE FROM webhook_deliveries WHERE subject = 'worker:{epid}'"),
        format!("DELETE FROM engagement_extensions WHERE worker_pid = '{epid}'"),
        format!("DELETE FROM worker_contractor_details WHERE worker_pid = '{epid}'"),
        format!("DELETE FROM engagement_status_assessments WHERE worker_pid = '{epid}'"),
        // Expense claims are financial records: draft and submitted ones are closed,
        // the amounts stay (statutory retention), and the free text goes.
        format!(
            "UPDATE expense_claims SET status = 'cancelled', updated_at = now() \
             WHERE worker_pid = '{epid}' AND status IN ('draft', 'submitted')"
        ),
        format!(
            "UPDATE expense_claims SET title = 'erased', description = NULL, decision_note = NULL \
             WHERE worker_pid = '{epid}'"
        ),
        format!(
            "UPDATE expense_items SET description = NULL, receipt_ref = NULL WHERE worker_pid = '{epid}'"
        ),
        // Added with the personal-data inventory (WPM-T353): weekly totals and memberships are about them only; an onboarding
        // item stays (the checklist is the employer's) but its free-text reason goes; a succession entry is a rating about them.
        format!("DELETE FROM time_entry_weeks WHERE worker_pid = '{epid}'"),
        format!("DELETE FROM organization_memberships WHERE worker_pid = '{epid}'"),
        format!("UPDATE onboarding_items SET waived_reason = NULL WHERE worker_pid = '{epid}'"),
        format!("DELETE FROM succession_candidates WHERE worker_pid = '{epid}'"),
        format!("DELETE FROM learning_completions WHERE worker_pid = '{epid}'"),
        format!("DELETE FROM qualifications WHERE worker_pid = '{epid}'"),
        format!("DELETE FROM qualification_reminders WHERE worker_pid = '{epid}'"),
        format!("DELETE FROM payslip_queries WHERE worker_pid = '{epid}'"),
        format!("DELETE FROM development_plan_benefits WHERE worker_pid = '{epid}'"),
        format!("DELETE FROM review_goal_outcomes WHERE worker_pid = '{epid}'"),
        format!("DELETE FROM goal_check_ins WHERE worker_pid = '{epid}'"),
        format!("DELETE FROM goal_measures WHERE worker_pid = '{epid}'"),
        format!("DELETE FROM worker_goals WHERE worker_pid = '{epid}'"),
        format!("DELETE FROM padr_versions WHERE worker_pid = '{epid}'"),
        format!("DELETE FROM separation_records WHERE worker_pid = '{epid}'"),
        format!("DELETE FROM course_requests WHERE worker_pid = '{epid}'"),
        // The row proves a duty was met and names no one without this column (WPM-D179).
        format!("UPDATE injury_records SET worker_pid = NULL WHERE worker_pid = '{epid}'"),
    ]
}

/// Record an erasure in the ledger (WPM-D63): the pid and the time, nothing else.
async fn record_erasure(tx: &mut sqlx::PgConnection, epid: Uuid) -> Result<()> {
    sqlx::query!(
        "INSERT INTO erasure_ledger (worker_pid) VALUES ($1) ON CONFLICT DO NOTHING",
        epid
    )
    .execute(tx)
    .await?;
    Ok(())
}

/// Run one of [`erasure_statements`]: a fixed statement with the worker's pid in it, and the rows it touched.
async fn run_statement(tx: &mut sqlx::PgConnection, statement: String) -> Result<u64> {
    Ok(sqlx::raw_sql(sqlx::AssertSqlSafe(statement))
        .execute(tx)
        .await?
        .rows_affected())
}

/// What a replay did.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ReplayReport {
    /// Ledger entries considered.
    pub considered: usize,
    /// Workers found in the database and erased again.
    pub reapplied: usize,
    /// Entries whose worker is not in this database (the backup predates them).
    pub missing: usize,
}

/// Replay the erasure ledger after a restore (WPM-D63): for every entry (erased
/// on or after `since`, or all of them), scrub the worker again with the same
/// statements as the live erasure. Idempotent, so running it twice changes
/// nothing more. A worker the database does not hold is counted, not an error.
///
/// # Errors
///
/// A database error; nothing is applied for the entry that failed.
///
/// `db` is the `SeaORM` handle the task passes; the replay runs on the `sqlx` pool under it (WPM-T466).
pub async fn replay_erasures(
    db: &sea_orm::DatabaseConnection,
    since: Option<chrono::NaiveDate>,
) -> Result<ReplayReport> {
    let pool = db.get_postgres_connection_pool();
    let rows = sqlx::query_scalar!(
        "SELECT worker_pid FROM erasure_ledger WHERE ($1::date IS NULL OR erased_at >= $1) ORDER BY erased_at",
        since,
    )
    .fetch_all(pool)
    .await?;
    let mut report = ReplayReport::default();
    for epid in rows {
        report.considered += 1;
        let mut tx = pool.begin().await?;
        let found = sqlx::query!(
            "UPDATE workers SET display_name = $2, person_ref = $3, upstream_worker_ref = NULL, salary_minor = NULL,
                                salary_currency = NULL, deleted_at = COALESCE(deleted_at, now())
              WHERE pid = $1",
            epid,
            ERASED,
            TOMBSTONE_PERSON,
        )
        .execute(&mut *tx)
        .await?
        .rows_affected();
        if found == 0 {
            report.missing += 1;
            tx.commit().await?;
            continue;
        }
        for statement in erasure_statements(epid) {
            run_statement(&mut tx, statement).await?;
        }
        crate::db::audit(
            &mut tx,
            "worker",
            epid,
            "erasure_replayed",
            Some("task:replay_erasures"),
            None,
        )
        .await?;
        tx.commit().await?;
        report.reapplied += 1;
    }
    Ok(report)
}

/// `POST /api/workers/{pid}/erase` — anonymise (WPM-D22): scrub the
/// worker's identity fields and soft-delete the row, scrub free text
/// they authored (time-entry notes, 360 comments, mentorship session
/// notes), close their appraisals-as-subject, and delete their
/// wellbeing acknowledgements. Payroll/financial rows remain, keyed to
/// a pid that no longer identifies anyone. Refused while employment is
/// open. Destructive-classified; audited with counts.
#[debug_handler]
#[allow(clippy::too_many_lines)] // one scrub statement per worker-owned table
async fn erase(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    auth::authorize_record(
        &caller,
        authentication_verifier::Action::Destructive,
        &auth::worker_resource_attrs(&worker),
    )
    .map_err(record_rejection)?;
    if !rules::erasable(&worker.status) {
        return Err(unprocessable(
            "erasure requires a terminated or retired employment (the active \
             relationship is the lawful basis for the data)",
        ));
    }
    let epid = worker.pid;
    let mut tx = crate::db::pool(&ctx).begin().await?;
    // Identity fields scrubbed in place; the row soft-deleted.
    sqlx::query!(
        "UPDATE workers SET display_name = $2, person_ref = $3, upstream_worker_ref = NULL, salary_minor = NULL,
                            salary_currency = NULL, deleted_at = now(), updated_at = now()
          WHERE id = $1",
        worker.id,
        ERASED,
        TOMBSTONE_PERSON,
    )
    .execute(&mut *tx)
    .await?;
    // Free text they authored, and rows that are about them only.
    let mut affected = Vec::new();
    for statement in erasure_statements(epid) {
        affected.push(run_statement(&mut tx, statement).await?);
    }
    record_erasure(&mut tx, epid).await?;
    crate::db::audit(
        &mut tx,
        "worker",
        epid,
        "erased",
        caller.actor(),
        Some(serde_json::json!({
            "notes_scrubbed": affected[0],
            "appraisal_comments_scrubbed": affected[1],
            "session_notes_scrubbed": affected[2],
            "appraisals_closed": affected[3],
            "acknowledgements_deleted": affected[4],
            "notifications_deleted": affected[5],
            "ergonomic_items_scrubbed": affected[6],
            "ergonomic_assessments_closed": affected[7],
            "adjustment_requests_scrubbed": affected[8],
            "cpd_entries_scrubbed": affected[9],
            "registrations_scrubbed": affected[10],
            "mobility_interests_withdrawn": affected[11],
            "framework_roles_deleted": affected[12],
            "skill_history_deleted": affected[13],
            "aspirations_deleted": affected[14],
            "group_memberships_deleted": affected[15],
            "dotted_lines_deleted": affected[16],
            "emergency_contacts_deleted": affected[17],
            "backups_deleted": affected[18],
            "rota_memberships_deleted": affected[19],
            "rota_swaps_deleted": affected[20],
            "rota_swap_requests_deleted": affected[21],
            "announcement_reads_deleted": affected[22],
            "movement_notes_scrubbed": affected[23],
            "movement_tasks_unassigned": affected[24],
            "handover_notes_scrubbed": affected[25],
            "job_levels_deleted": affected[26],
            "pay_positions_deleted": affected[27],
            "expense_claims_closed": affected[28],
            "expense_claim_texts_scrubbed": affected[29],
            "expense_item_texts_scrubbed": affected[30],
            "weekly_time_deleted": affected[31],
            "memberships_deleted": affected[32],
            "onboarding_reasons_scrubbed": affected[33],
            "succession_entries_deleted": affected[34],
        })),
    )
    .await?;
    tx.commit().await?;
    format::json(serde_json::json!({
        "erased": epid,
        "note": "anonymised, not deleted: payroll/financial rows remain under statutory \
                 retention, keyed to a pid that no longer identifies anyone (WPM-D22); \
                 coordinate erasure with the upstream identity services",
    }))
}

/// The horizon for every record kind and where it came from: the organization's profile over its
/// home jurisdiction's chain (`retention.<kind>.days`, WPM-D153) when `scope` is given, else a kind's
/// own `WPM_RETENTION_<KIND>_DAYS`, else the legacy `WPM_RETENTION_DAYS`, else the kind's default;
/// floored at 30 calendar days (WPM-R91, WPM-D22, WPM-D60).
pub(crate) fn horizons(
    scope: Option<&super::jurisdictions::Scope>,
    as_of: chrono::NaiveDate,
) -> Vec<(rules::RecordKind, InForce<rules::Horizon>)> {
    let legacy = crate::compat::env_var("WPM_RETENTION_DAYS");
    rules::RecordKind::ALL
        .iter()
        .map(|&kind| {
            let name = kind.env_name();
            let own = crate::compat::env_var(&name);
            let configured = rules::horizon_for(kind, own.as_deref(), legacy.as_deref());
            let mut found = statutory::pick(
                scope,
                &name,
                as_of,
                |value| match value {
                    crate::rules::jurisdiction::Value::Range { min, max } => {
                        rules::horizon_from_range(*min, *max)
                    }
                    _ => None,
                },
                (configured.source != rules::HorizonSource::Default).then_some(configured),
                configured,
            );
            if found.value.source == rules::HorizonSource::Legacy {
                found.came_from["setting"] = serde_json::json!("WPM_RETENTION_DAYS");
            }
            (kind, found)
        })
        .collect()
}

/// The horizons the sweep and the report use. The sweep is one pass over every organization's rows,
/// and a row does not always say whose it is, so for each kind it takes the **longest** of the
/// deployment's horizon and every horizon an organization's profile gives over its home
/// jurisdiction: a profile can make the sweep keep a record longer, never delete one sooner.
async fn sweep_horizons(
    ctx: &AppContext,
) -> Result<Vec<(rules::RecordKind, InForce<rules::Horizon>)>> {
    let as_of = chrono::Utc::now().date_naive();
    let mut all = horizons(None, as_of);
    for organization in
        super::jurisdictions::organizations_with_a_home(crate::db::pool(ctx)).await?
    {
        let scope =
            super::jurisdictions::Scope::for_organization(crate::db::pool(ctx), &organization)
                .await?;
        for (kind, found) in horizons(Some(&scope), as_of) {
            if found.value.source != rules::HorizonSource::Profile {
                continue;
            }
            if let Some((_, held)) = all.iter_mut().find(|(k, _)| *k == kind)
                && found.value.days > held.value.days
            {
                let mut came_from = found.came_from;
                came_from["organization_ref"] = serde_json::json!(organization);
                *held = InForce {
                    value: found.value,
                    came_from,
                };
            }
        }
    }
    Ok(all)
}

/// The horizon in calendar days for one table.
fn days_for(table: &str, all: &[(rules::RecordKind, InForce<rules::Horizon>)]) -> i64 {
    rules::kind_of(table)
        .and_then(|kind| all.iter().find(|(k, _)| *k == kind))
        .map_or(rules::RETENTION_DEFAULT_DAYS, |(_, found)| found.value.days)
}

/// The horizons as `{kind: days}`, for the report, the sweep and its audit row.
fn horizon_map(all: &[(rules::RecordKind, InForce<rules::Horizon>)]) -> serde_json::Value {
    serde_json::Value::Object(
        all.iter()
            .map(|(kind, found)| (kind.key().to_string(), serde_json::json!(found.value.days)))
            .collect(),
    )
}

/// Where each horizon came from, as `{kind: came_from}`.
fn horizon_sources(all: &[(rules::RecordKind, InForce<rules::Horizon>)]) -> serde_json::Value {
    serde_json::Value::Object(
        all.iter()
            .map(|(kind, found)| (kind.key().to_string(), found.came_from.clone()))
            .collect(),
    )
}

#[derive(Debug, serde::Deserialize)]
struct ScheduleQuery {
    organization: Option<String>,
}

/// `GET /api/retention/schedule?organization=` — each record kind, its tables, and the horizon in
/// force with where it came from: the deployment's, or with `organization` that organization's over
/// its home jurisdiction's profile. Configuration, not personal data.
#[debug_handler]
async fn retention_schedule(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axa::extract::Query(q): axa::extract::Query<ScheduleQuery>,
) -> Result<Response> {
    let as_of = chrono::Utc::now().date_naive();
    let scope = match &q.organization {
        Some(org) => {
            super::ensure_valid(&super::pay_transparency::organization_problems(org))?;
            super::ensure_org_in_scope(&ctx, &caller, org).await?;
            Some(super::jurisdictions::Scope::for_organization(crate::db::pool(&ctx), org).await?)
        }
        None => None,
    };
    let kinds: Vec<serde_json::Value> = horizons(scope.as_ref(), as_of)
        .into_iter()
        .map(|(kind, found)| {
            let tables: Vec<&str> = rules::SOFT_DELETED_TABLES
                .iter()
                .copied()
                .filter(|t| rules::kind_of(t) == Some(kind))
                .collect();
            serde_json::json!({
                "kind": kind.key(),
                "days": found.value.days,
                "source": found.value.source.label(),
                "came_from": found.came_from,
                "default_days": kind.default_days(),
                "override_variable": kind.env_name(),
                "profile_key": format!("retention.{}.days", kind.key()),
                "tables": tables,
            })
        })
        .collect();
    format::json(serde_json::json!({
        "organization_ref": q.organization,
        "floor_days": rules::RETENTION_FLOOR_DAYS,
        "kinds": kinds,
        "derivation": "calendar days after a record is soft-deleted before the sweep \
                       hard-deletes it; with an organization, its profile's retention.<kind>.days \
                       over its home jurisdiction (the floor, else the ceiling) wins; then a kind's own \
                       WPM_RETENTION_<KIND>_DAYS, then the legacy WPM_RETENTION_DAYS, then the default; \
                       every value is floored (WPM-D22, WPM-D60). The sweep takes, for each kind, the \
                       longest of the deployment's horizon and every organization's profile, so a \
                       profile never makes it delete sooner. The defaults are starting points a \
                       deployer replaces with their own legal basis, not legal advice.",
    }))
}

/// `GET /api/retention` — the report: per table, soft-deleted rows
/// older than that table's horizon, plus candidates whose consent expired
/// before the recruitment horizon. Read-only; the sweep is the separate
/// destructive POST.
#[debug_handler]
async fn retention_report(State(ctx): State<AppContext>) -> Result<Response> {
    let all = sweep_horizons(&ctx).await?;
    let mut tables = serde_json::Map::new();
    for table in rules::SOFT_DELETED_TABLES {
        let days = days_for(table, &all);
        // The table comes from the fixed list of soft-deleting tables, never from the request.
        let count: i64 = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "SELECT COUNT(*) AS n FROM {table} \
             WHERE deleted_at IS NOT NULL AND deleted_at < now() - interval '{days} days'"
        )))
        .fetch_one(crate::db::pool(&ctx))
        .await?;
        if count > 0 {
            tables.insert((*table).to_string(), serde_json::json!(count));
        }
    }
    let recruitment_days = days_for("candidates", &all);
    let expired_candidates = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "n!" FROM candidates WHERE deleted_at IS NULL AND consent_until < $1"#,
        chrono::Utc::now().date_naive() - chrono::Duration::days(recruitment_days),
    )
    .fetch_one(crate::db::pool(&ctx))
    .await?;
    format::json(serde_json::json!({
        "as_of": chrono::Utc::now(),
        "horizons": horizon_map(&all),
        "horizons_from": horizon_sources(&all),
        "soft_deleted_past_horizon": tables,
        "expired_consent_candidates": expired_candidates,
        "derivation": "soft-deleted rows older than their kind's horizon are hard-deleted \
                       by the sweep; candidates whose consent expired before the \
                       recruitment horizon are scrubbed; every horizon floors at 30 \
                       calendar days (WPM-D22, WPM-D60); see /api/retention/schedule",
    }))
}

/// `POST /api/retention/sweep` — hard-delete soft-deleted rows past
/// the horizon across every soft-deleting table, and scrub expired-
/// consent candidates (name/email erased, `person_ref` dropped, row
/// soft-deleted so the next sweep removes it). Destructive-classified;
/// audited with counts.
#[debug_handler]
async fn retention_sweep(State(ctx): State<AppContext>, caller: MaybeAuthUser) -> Result<Response> {
    let all = sweep_horizons(&ctx).await?;
    let recruitment_days = days_for("candidates", &all);
    let mut tx = crate::db::pool(&ctx).begin().await?;
    let mut deleted = serde_json::Map::new();
    let mut total: u64 = 0;
    // An appraisal's invitations and answers live and die with it: the sweep removes them first, or they would outlive
    // the appraisal for good (a rater's pid and scores with nothing left to attach to).
    let appraisal_days = days_for("appraisals", &all);
    // Each table comes from a fixed list and each horizon is a number, never from the request.
    for table in ["appraisal_responses", "appraisal_nominations"] {
        let affected = run_statement(
            &mut tx,
            format!(
                "DELETE FROM {table} WHERE appraisal_pid IN (SELECT pid FROM appraisals \
                 WHERE deleted_at < now() - interval '{appraisal_days} days')"
            ),
        )
        .await?;
        if affected > 0 {
            deleted.insert(table.to_string(), serde_json::json!(affected));
            total += affected;
        }
    }
    for table in rules::SOFT_DELETED_TABLES {
        let days = days_for(table, &all);
        let affected = run_statement(
            &mut tx,
            format!("DELETE FROM {table} WHERE deleted_at < now() - interval '{days} days'"),
        )
        .await?;
        if affected > 0 {
            deleted.insert((*table).to_string(), serde_json::json!(affected));
            total += affected;
        }
    }
    let scrubbed = run_statement(
        &mut tx,
        format!(
            "UPDATE candidates SET display_name = '{ERASED}', email = '{ERASED}', \
             person_ref = NULL, deleted_at = now() \
             WHERE deleted_at IS NULL \
             AND consent_until < CURRENT_DATE - interval '{recruitment_days} days'"
        ),
    )
    .await?;
    crate::db::audit(
        &mut tx,
        "retention",
        Uuid::nil(),
        "retention_swept",
        caller.actor(),
        Some(serde_json::json!({
            "horizons": horizon_map(&all),
            "rows_deleted": total,
            "candidates_scrubbed": scrubbed,
        })),
    )
    .await?;
    tx.commit().await?;
    format::json(serde_json::json!({
        "horizons": horizon_map(&all),
        "deleted": deleted,
        "rows_deleted": total,
        "candidates_scrubbed": scrubbed,
    }))
}

/// The privacy routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/workers/{pid}/subject-access", get(subject_access))
        .add(
            "/candidates/{pid}/subject-access",
            get(candidate_subject_access),
        )
        .add("/workers/{pid}/erase", post(erase))
        .add("/retention", get(retention_report))
        .add("/retention/schedule", get(retention_schedule))
        .add("/retention/sweep", post(retention_sweep))
}
