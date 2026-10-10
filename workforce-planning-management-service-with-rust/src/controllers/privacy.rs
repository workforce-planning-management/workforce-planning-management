//! Subject rights & retention (WPM-R30 / WPM-D22): the subject-access
//! export, erasure-as-anonymisation, and the retention report + sweep.
//! Erasure and the sweep are destructive-classified POSTs (`/erase`,
//! `/sweep` — `access=admin` under enforcement). WPM covers its own
//! store only: what the upstream identity services hold is the
//! deployment's coordination duty, stated in the payloads.

use loco_rs::prelude::*;
use sea_orm::{ActiveValue, ConnectionTrait};
use uuid::Uuid;

use super::{record_rejection, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::models::_entities::{
    adjustment_requests, announcement_reads, appraisal_nominations, appraisal_responses,
    appraisals, assessments, benefit_enrollments, candidates, cpd_entries, development_plans,
    dotted_line_reports, emergency_contacts, engagement_extensions, engagement_status_assessments,
    entitlement_acknowledgements, equality_declarations, ergonomic_assessments, expense_claims,
    expense_items, flexible_working_requests, group_members, handover_actions, leave_entitlements,
    leave_requests, mentorships, mobility_interests, movements, notifications, path_enrollments,
    payslips, pipeline_members, professional_registrations, program_placements, resignations,
    reviews, rota_members, rota_overrides, rota_swap_requests, shift_assignments, time_entries,
    training_enrollments, worker_aspirations, worker_backups, worker_contact_details,
    worker_contractor_details, worker_framework_roles, worker_health_records, worker_job_levels,
    worker_pay_positions, worker_skill_history, worker_skills, workers,
};
use crate::models::audit_logs::Model as Audit;
use crate::models::records;
use crate::rules::privacy as rules;

/// The tombstone URN an erased worker's `person_ref` becomes: a
/// syntactically valid `EntityRef` that resolves to no one.
const TOMBSTONE_PERSON: &str = "person:00000000-0000-0000-0000-000000000000";

/// The scrub placeholder for erased free text and names.
const ERASED: &str = "[erased]";

/// Rows keyed by one worker in `module` (helper macro: filter on the
/// given column, return the models as JSON).
macro_rules! rows_for {
    ($db:expr, $module:ident, $column:ident, $pid:expr) => {
        serde_json::json!(
            $module::Entity::find()
                .filter($module::Column::$column.eq($pid))
                .all($db)
                .await?
        )
    };
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
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
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
        return Err(record_rejection((
            axum::http::StatusCode::FORBIDDEN,
            "subject access requires an unmasked read (the export discloses \
             salary and payslips; masked callers cannot receive it)"
                .to_string(),
        )));
    }
    let db = &ctx.db;
    let epid = worker.pid;
    // 360°: appraisals about them, nominations naming them as rater,
    // and only the responses THEY authored.
    let their_nominations = appraisal_nominations::Entity::find()
        .filter(appraisal_nominations::Column::RaterPid.eq(epid))
        .all(db)
        .await?;
    let nomination_pids: Vec<Uuid> = their_nominations.iter().map(|n| n.pid).collect();
    let authored_responses = appraisal_responses::Entity::find()
        .filter(appraisal_responses::Column::NominationPid.is_in(nomination_pids))
        .all(db)
        .await?;
    let mentorship_rows = mentorships::Entity::find()
        .filter(
            sea_orm::Condition::any()
                .add(mentorships::Column::MentorPid.eq(epid))
                .add(mentorships::Column::MenteePid.eq(epid)),
        )
        .all(db)
        .await?;
    let export = serde_json::json!({
        "as_of": chrono::Utc::now(),
        "worker": worker,
        "time_entries": rows_for!(db, time_entries, WorkerPid, epid),
        "leave_entitlements": rows_for!(db, leave_entitlements, WorkerPid, epid),
        "leave_requests": rows_for!(db, leave_requests, WorkerPid, epid),
        "shift_assignments": rows_for!(db, shift_assignments, WorkerPid, epid),
        "benefit_enrollments": rows_for!(db, benefit_enrollments, WorkerPid, epid),
        "reviews": rows_for!(db, reviews, WorkerPid, epid),
        "training_enrollments": rows_for!(db, training_enrollments, WorkerPid, epid),
        "skills": rows_for!(db, worker_skills, WorkerPid, epid),
        "learning_path_enrollments": rows_for!(db, path_enrollments, WorkerPid, epid),
        "development_plans": rows_for!(db, development_plans, WorkerPid, epid),
        "cpd_entries": rows_for!(db, cpd_entries, WorkerPid, epid),
        "framework_roles": rows_for!(db, worker_framework_roles, WorkerPid, epid),
        "skill_history": rows_for!(db, worker_skill_history, WorkerPid, epid),
        "aspirations": rows_for!(db, worker_aspirations, WorkerPid, epid),
        "emergency_contacts": rows_for!(db, emergency_contacts, WorkerPid, epid),
        "job_level": rows_for!(db, worker_job_levels, WorkerPid, epid),
        "pay_position": rows_for!(db, worker_pay_positions, WorkerPid, epid),
        "contact_details": rows_for!(db, worker_contact_details, WorkerPid, epid),
        "resignations": rows_for!(db, resignations, WorkerPid, epid),
        // Health data readable by the worker and occupational health only (WPM-D75): anyone else
        // running the export gets a note.
        "workplace_health_records": if auth::acting_for_other(&caller, &worker.person_ref) {
            serde_json::json!("withheld: only the worker can export their own workplace health records")
        } else {
            rows_for!(db, worker_health_records, WorkerPid, epid)
        },
        // Special-category data readable by the worker only (WPM-D74): anyone else running the
        // export gets a note, not the answers.
        "equality_monitoring": if auth::acting_for_other(&caller, &worker.person_ref) {
            serde_json::json!("withheld: only the worker can export their own equality monitoring answers")
        } else {
            rows_for!(db, equality_declarations, WorkerPid, epid)
        },
        "flexible_working_requests": rows_for!(db, flexible_working_requests, WorkerPid, epid),
        "engagement_extensions": rows_for!(db, engagement_extensions, WorkerPid, epid),
        "contractor_details": rows_for!(db, worker_contractor_details, WorkerPid, epid),
        "employment_status_assessments": rows_for!(db, engagement_status_assessments, WorkerPid, epid),
        "expense_claims": rows_for!(db, expense_claims, WorkerPid, epid),
        "expense_items": rows_for!(db, expense_items, WorkerPid, epid),
        "backups": rows_for!(db, worker_backups, WorkerPid, epid),
        "named_as_backup_by": rows_for!(db, worker_backups, BackupPid, epid),
        "on_call_rota_memberships": rows_for!(db, rota_members, WorkerPid, epid),
        "on_call_swaps": rows_for!(db, rota_overrides, WorkerPid, epid),
        "announcements_read": rows_for!(db, announcement_reads, WorkerPid, epid),
        "joiner_leaver_records": rows_for!(db, movements, WorkerPid, epid),
        "handover_actions_from": rows_for!(db, handover_actions, FromWorker, epid),
        "handover_actions_to": rows_for!(db, handover_actions, ToWorker, epid),
        "on_call_swap_requests_made": rows_for!(db, rota_swap_requests, RequesterPid, epid),
        "on_call_swap_requests_received": rows_for!(db, rota_swap_requests, TakerPid, epid),
        "group_memberships": rows_for!(db, group_members, WorkerPid, epid),
        "dotted_line_as_report": rows_for!(db, dotted_line_reports, ReportPid, epid),
        "dotted_line_as_manager": rows_for!(db, dotted_line_reports, ManagerPid, epid),
        "mobility_interests": rows_for!(db, mobility_interests, WorkerPid, epid),
        "professional_registrations": rows_for!(db, professional_registrations, WorkerPid, epid),
        "program_placements": rows_for!(db, program_placements, WorkerPid, epid),
        "payslips": rows_for!(db, payslips, WorkerPid, epid),
        "wellbeing_acknowledgements":
            rows_for!(db, entitlement_acknowledgements, WorkerPid, epid),
        "notifications": rows_for!(db, notifications, WorkerPid, epid),
        "ergonomic_assessments": rows_for!(db, ergonomic_assessments, WorkerPid, epid),
        "adjustment_requests": rows_for!(db, adjustment_requests, WorkerPid, epid),
        "assessments": rows_for!(db, assessments, SubjectPid, epid),
        "pipeline_memberships": rows_for!(db, pipeline_members, SubjectPid, epid),
        "mentorships": mentorship_rows,
        "appraisals_as_subject": rows_for!(db, appraisals, WorkerPid, epid),
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
    Audit::record(
        &ctx.db,
        "worker",
        epid,
        "subject_access_exported",
        caller.actor(),
        None,
    )
    .await?;
    format::json(export)
}

/// The statements that scrub everything a worker authored or owns, and the rows
/// that are about them only (WPM-D22). One list, used by the live erasure and by
/// the replay after a restore (WPM-D63), so the two cannot drift. The order is
/// pinned: the audit snapshot indexes the row counts by position.
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
        format!("UPDATE movements SET notes = NULL WHERE worker_pid = '{epid}'"),
        format!("UPDATE movement_items SET assignee_pid = NULL WHERE assignee_pid = '{epid}'"),
        format!(
            "UPDATE handover_actions SET note = NULL WHERE from_worker = '{epid}' OR to_worker = '{epid}'"
        ),
        format!("DELETE FROM worker_job_levels WHERE worker_pid = '{epid}'"),
        format!("DELETE FROM worker_pay_positions WHERE worker_pid = '{epid}'"),
        format!("DELETE FROM worker_contact_details WHERE worker_pid = '{epid}'"),
        format!("DELETE FROM resignations WHERE worker_pid = '{epid}'"),
        format!("DELETE FROM worker_health_records WHERE worker_pid = '{epid}'"),
        format!("DELETE FROM equality_declarations WHERE worker_pid = '{epid}'"),
        format!("DELETE FROM flexible_working_requests WHERE worker_pid = '{epid}'"),
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
    ]
}

/// Record an erasure in the ledger (WPM-D63): the pid and the time, nothing else.
async fn record_erasure<C: ConnectionTrait>(db: &C, epid: Uuid) -> Result<()> {
    db.execute_unprepared(&format!(
        "INSERT INTO erasure_ledger (worker_pid) VALUES ('{epid}') ON CONFLICT DO NOTHING"
    ))
    .await?;
    Ok(())
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
pub async fn replay_erasures(
    db: &sea_orm::DatabaseConnection,
    since: Option<chrono::NaiveDate>,
) -> Result<ReplayReport> {
    use sea_orm::TransactionTrait;
    let filter = since.map_or_else(String::new, |d| format!("WHERE erased_at >= '{d}'"));
    let rows = db
        .query_all_raw(sea_orm::Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            format!("SELECT worker_pid FROM erasure_ledger {filter} ORDER BY erased_at"),
        ))
        .await?;
    let mut report = ReplayReport::default();
    for row in rows {
        let Ok(epid) = row.try_get::<Uuid>("", "worker_pid") else {
            continue;
        };
        report.considered += 1;
        let txn = db.begin().await?;
        let found = txn
            .execute_unprepared(&format!(
                "UPDATE workers SET display_name = '{ERASED}', person_ref = '{TOMBSTONE_PERSON}', \
                 upstream_worker_ref = NULL, salary_minor = NULL, salary_currency = NULL, \
                 deleted_at = COALESCE(deleted_at, now()) WHERE pid = '{epid}'"
            ))
            .await?
            .rows_affected();
        if found == 0 {
            report.missing += 1;
            txn.commit().await?;
            continue;
        }
        for statement in erasure_statements(epid) {
            txn.execute_unprepared(&statement).await?;
        }
        Audit::record(
            &txn,
            "worker",
            epid,
            "erasure_replayed",
            Some("task:replay_erasures"),
            None,
        )
        .await?;
        txn.commit().await?;
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
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
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
    let txn = ctx.db.begin().await?;
    // Identity fields scrubbed in place; the row soft-deleted.
    let mut scrubbed: workers::ActiveModel = worker.into();
    scrubbed.display_name = ActiveValue::set(ERASED.to_string());
    scrubbed.person_ref = ActiveValue::set(TOMBSTONE_PERSON.to_string());
    scrubbed.upstream_worker_ref = ActiveValue::set(None);
    scrubbed.salary_minor = ActiveValue::set(None);
    scrubbed.salary_currency = ActiveValue::set(None);
    scrubbed.deleted_at = ActiveValue::set(Some(chrono::Utc::now().into()));
    scrubbed.update(&txn).await?;
    // Free text they authored, and rows that are about them only.
    let statements = erasure_statements(epid);
    let mut affected = Vec::new();
    for statement in &statements {
        let result = txn.execute_unprepared(statement).await?;
        affected.push(result.rows_affected());
    }
    record_erasure(&txn, epid).await?;
    Audit::record(
        &txn,
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
        })),
    )
    .await?;
    txn.commit().await?;
    format::json(serde_json::json!({
        "erased": epid,
        "note": "anonymised, not deleted: payroll/financial rows remain under statutory \
                 retention, keyed to a pid that no longer identifies anyone (WPM-D22); \
                 coordinate erasure with the upstream identity services",
    }))
}

/// The horizon for every record kind, read from the environment: a kind's own
/// `WPM_RETENTION_<KIND>_DAYS`, else the legacy `WPM_RETENTION_DAYS`, else the
/// kind's default; floored at 30 calendar days (WPM-R91, WPM-D22, WPM-D60).
fn horizons() -> Vec<(rules::RecordKind, rules::Horizon)> {
    let legacy = crate::compat::env_var("WPM_RETENTION_DAYS");
    rules::RecordKind::ALL
        .iter()
        .map(|&kind| {
            let own = crate::compat::env_var(&kind.env_name());
            (
                kind,
                rules::horizon_for(kind, own.as_deref(), legacy.as_deref()),
            )
        })
        .collect()
}

/// The horizon in calendar days for one table.
fn days_for(table: &str, all: &[(rules::RecordKind, rules::Horizon)]) -> i64 {
    rules::kind_of(table)
        .and_then(|kind| all.iter().find(|(k, _)| *k == kind))
        .map_or(rules::RETENTION_DEFAULT_DAYS, |(_, horizon)| horizon.days)
}

/// The horizons as `{kind: days}`, for the report, the sweep and its audit row.
fn horizon_map(all: &[(rules::RecordKind, rules::Horizon)]) -> serde_json::Value {
    serde_json::Value::Object(
        all.iter()
            .map(|(kind, horizon)| (kind.key().to_string(), serde_json::json!(horizon.days)))
            .collect(),
    )
}

/// `GET /api/retention/schedule` — each record kind, its tables, and the
/// horizon in force with where it came from. Configuration, not personal data.
#[debug_handler]
async fn retention_schedule() -> Result<Response> {
    let kinds: Vec<serde_json::Value> = horizons()
        .into_iter()
        .map(|(kind, horizon)| {
            let tables: Vec<&str> = rules::SOFT_DELETED_TABLES
                .iter()
                .copied()
                .filter(|t| rules::kind_of(t) == Some(kind))
                .collect();
            serde_json::json!({
                "kind": kind.key(),
                "days": horizon.days,
                "source": horizon.source.label(),
                "default_days": kind.default_days(),
                "override_variable": kind.env_name(),
                "tables": tables,
            })
        })
        .collect();
    format::json(serde_json::json!({
        "floor_days": rules::RETENTION_FLOOR_DAYS,
        "kinds": kinds,
        "derivation": "calendar days after a record is soft-deleted before the sweep \
                       hard-deletes it; a kind's own WPM_RETENTION_<KIND>_DAYS wins, \
                       then the legacy WPM_RETENTION_DAYS, then the default; every value \
                       is floored (WPM-D22, WPM-D60). The defaults are starting points a \
                       deployer replaces with their own legal basis, not legal advice.",
    }))
}

/// `GET /api/retention` — the report: per table, soft-deleted rows
/// older than that table's horizon, plus candidates whose consent expired
/// before the recruitment horizon. Read-only; the sweep is the separate
/// destructive POST.
#[debug_handler]
async fn retention_report(State(ctx): State<AppContext>) -> Result<Response> {
    let all = horizons();
    let mut tables = serde_json::Map::new();
    for table in rules::SOFT_DELETED_TABLES {
        let days = days_for(table, &all);
        let count = ctx
            .db
            .query_one_raw(sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                format!(
                    "SELECT COUNT(*) AS n FROM {table} \
                     WHERE deleted_at < now() - interval '{days} days'"
                ),
            ))
            .await?
            .and_then(|row| row.try_get::<i64>("", "n").ok())
            .unwrap_or(0);
        if count > 0 {
            tables.insert((*table).to_string(), serde_json::json!(count));
        }
    }
    let recruitment_days = days_for("candidates", &all);
    let expired_candidates = candidates::Entity::find()
        .filter(candidates::Column::DeletedAt.is_null())
        .filter(
            candidates::Column::ConsentUntil
                .lt(chrono::Utc::now().date_naive() - chrono::Duration::days(recruitment_days)),
        )
        .all(&ctx.db)
        .await?
        .len();
    format::json(serde_json::json!({
        "as_of": chrono::Utc::now(),
        "horizons": horizon_map(&all),
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
    let all = horizons();
    let recruitment_days = days_for("candidates", &all);
    let txn = ctx.db.begin().await?;
    let mut deleted = serde_json::Map::new();
    let mut total: u64 = 0;
    for table in rules::SOFT_DELETED_TABLES {
        let days = days_for(table, &all);
        let result = txn
            .execute_unprepared(&format!(
                "DELETE FROM {table} WHERE deleted_at < now() - interval '{days} days'"
            ))
            .await?;
        if result.rows_affected() > 0 {
            deleted.insert(
                (*table).to_string(),
                serde_json::json!(result.rows_affected()),
            );
            total += result.rows_affected();
        }
    }
    let scrubbed = txn
        .execute_unprepared(&format!(
            "UPDATE candidates SET display_name = '{ERASED}', email = '{ERASED}', \
             person_ref = NULL, deleted_at = now() \
             WHERE deleted_at IS NULL \
             AND consent_until < CURRENT_DATE - interval '{recruitment_days} days'"
        ))
        .await?
        .rows_affected();
    Audit::record(
        &txn,
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
    txn.commit().await?;
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
        .add("/workers/{pid}/erase", post(erase))
        .add("/retention", get(retention_report))
        .add("/retention/schedule", get(retention_schedule))
        .add("/retention/sweep", post(retention_sweep))
}
