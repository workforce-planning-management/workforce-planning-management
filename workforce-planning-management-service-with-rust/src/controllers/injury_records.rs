//! The injury and illness notification record (WPM-R221, WPM-D158, WPM-D179, WPM-T421). HR records an incident and each report it owes;
//! the list says where each report stands against the times HR entered. A row proves a report and points to the statutory record, which
//! stays outside WPM. It takes no description, and a body with any field it does not know is refused. WPM decides no duty and files
//! nothing. HR, the service and an administrator read it, never a line manager or payroll, and each read is audited with no content.
//! The rules are `rules::injury_record`. Spec: `spec/injury-and-separation-duties.md`.

use axa::prelude::*;
use chrono::{DateTime, Datelike, NaiveDate, Utc};
use sea_orm::{ActiveValue, QueryOrder, TransactionTrait};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashMap};
use uuid::Uuid;

use super::{ensure_org_in_scope, ensure_valid, record_rejection, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::models::_entities::{injury_records, injury_reports, workers};
use crate::models::audit_logs::Model as Audit;
use crate::models::records;
use crate::rules::{equality_monitoring, injury_record as rules};

/// HR of the organization, or the service or an administrator: whoever may write its records. Payroll and a manager may not.
async fn ensure_hr(ctx: &AppContext, caller: &MaybeAuthUser, organization: &str) -> Result<()> {
    auth::authorize_record(
        caller,
        authentication_verifier::Action::Write,
        &BTreeMap::from([(
            "organization_ref".to_string(),
            vec![organization.to_string()],
        )]),
    )
    .map_err(record_rejection)?;
    ensure_org_in_scope(ctx, caller, organization).await
}

async fn find_record(ctx: &AppContext, pid: Uuid) -> Result<injury_records::Model> {
    injury_records::Entity::find()
        .filter(injury_records::Column::Pid.eq(pid))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)
}

fn utc(at: DateTimeWithTimeZone) -> DateTime<Utc> {
    at.with_timezone(&Utc)
}

fn report_json(r: &injury_reports::Model, now: DateTime<Utc>) -> Value {
    let standing = rules::standing(r.due_at.map(utc), r.sent_at.map(utc), now);
    json!({
        "pid": r.pid, "recipient": r.recipient, "due_at": r.due_at, "sent_at": r.sent_at,
        "method": r.method, "reference": r.reference, "standing": standing.as_str(),
    })
}

fn record_json(
    r: &injury_records::Model,
    reports: &[&injury_reports::Model],
    names: &HashMap<Uuid, String>,
    now: DateTime<Utc>,
) -> Value {
    json!({
        "pid": r.pid, "organization_ref": r.organization_ref, "jurisdiction": r.jurisdiction, "kind": r.kind,
        "worker_pid": r.worker_pid,
        "worker_name": r.worker_pid.and_then(|p| names.get(&p)),
        "occurred_at": r.occurred_at, "learned_at": r.learned_at, "class": r.class,
        "absence_days": r.absence_days, "people_affected": r.people_affected,
        "statutory_record_ref": r.statutory_record_ref, "keep_until": r.keep_until,
        "reports": reports.iter().map(|x| report_json(x, now)).collect::<Vec<_>>(),
    })
}

/// Query of the list.
#[derive(Debug, Deserialize)]
struct ListQuery {
    organization: String,
}

/// `GET /api/injury-records?organization=` — the organization's incidents, latest first, each with its reports and where they stand.
/// HR only. Audited with the count and no content.
#[debug_handler]
async fn list(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(q): axum::extract::Query<ListQuery>,
) -> Result<Response> {
    ensure_hr(&ctx, &caller, &q.organization).await?;
    let rows = injury_records::Entity::find()
        .filter(injury_records::Column::OrganizationRef.eq(&q.organization))
        .order_by_desc(injury_records::Column::OccurredAt)
        .order_by_desc(injury_records::Column::Id)
        .all(&ctx.db)
        .await?;
    let reports = injury_reports::Entity::find()
        .filter(injury_reports::Column::RecordPid.is_in(rows.iter().map(|r| r.pid)))
        .order_by_asc(injury_reports::Column::Id)
        .all(&ctx.db)
        .await?;
    let names: HashMap<Uuid, String> = workers::Entity::find()
        .filter(workers::Column::Pid.is_in(rows.iter().filter_map(|r| r.worker_pid)))
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|w| (w.pid, w.display_name))
        .collect();
    Audit::record(
        &ctx.db,
        "injury_record",
        Uuid::new_v4(),
        "injury_records_read",
        caller.actor(),
        Some(json!({ "organization": q.organization, "count": rows.len() })),
    )
    .await?;
    let now = Utc::now();
    format::json(
        rows.iter()
            .map(|r| {
                let own: Vec<_> = reports.iter().filter(|x| x.record_pid == r.pid).collect();
                record_json(r, &own, &names, now)
            })
            .collect::<Vec<_>>(),
    )
}

/// The body of a new incident. Any other field is refused, so a description has nowhere to go.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CreatePayload {
    organization: String,
    jurisdiction: String,
    kind: String,
    /// Absent for a dangerous occurrence, or someone who is not a worker.
    worker_pid: Option<Uuid>,
    occurred_at: DateTime<Utc>,
    learned_at: Option<DateTime<Utc>>,
    class: String,
    absence_days: Option<i32>,
    people_affected: Option<i32>,
    /// Where the statutory record is held: a reference, never the record.
    statutory_record_ref: Option<String>,
    keep_until: Option<NaiveDate>,
}

/// `POST /api/injury-records` — HR records an incident.
#[debug_handler]
async fn create(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(p): Json<CreatePayload>,
) -> Result<Response> {
    ensure_hr(&ctx, &caller, &p.organization).await?;
    let people = p.people_affected.unwrap_or(1);
    ensure_valid(&rules::problems(
        &p.jurisdiction,
        &p.kind,
        &p.class,
        p.occurred_at,
        p.learned_at,
        p.absence_days,
        people,
        Utc::now(),
    ))?;
    let reference =
        rules::clean_ref(p.statutory_record_ref.as_deref()).map_err(|e| unprocessable(&e))?;
    if let Some(worker) = p.worker_pid {
        let worker = records::find_worker(&ctx.db, worker)
            .await
            .map_err(|_| unprocessable("that worker is not on record"))?;
        if worker.organization_ref != p.organization {
            return Err(unprocessable("that worker is not in this organization"));
        }
    }
    let pid = Uuid::new_v4();
    let txn = ctx.db.begin().await?;
    injury_records::ActiveModel {
        pid: ActiveValue::set(pid),
        organization_ref: ActiveValue::set(p.organization),
        jurisdiction: ActiveValue::set(p.jurisdiction),
        kind: ActiveValue::set(p.kind),
        worker_pid: ActiveValue::set(p.worker_pid),
        occurred_at: ActiveValue::set(p.occurred_at.fixed_offset()),
        learned_at: ActiveValue::set(p.learned_at.map(|at| at.fixed_offset())),
        class: ActiveValue::set(p.class),
        absence_days: ActiveValue::set(p.absence_days),
        people_affected: ActiveValue::set(people),
        statutory_record_ref: ActiveValue::set(reference),
        keep_until: ActiveValue::set(p.keep_until),
        recorded_by: ActiveValue::set(caller.actor().map(ToString::to_string)),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    Audit::record(
        &txn,
        "injury_record",
        pid,
        "injury_recorded",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(json!({ "pid": pid }))
}

/// The body of a change: what can come to light later. The worker, kind, jurisdiction and time of an incident stand as recorded.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UpdatePayload {
    learned_at: Option<DateTime<Utc>>,
    class: String,
    absence_days: Option<i32>,
    people_affected: i32,
    statutory_record_ref: Option<String>,
    keep_until: Option<NaiveDate>,
}

/// `PUT /api/injury-records/{pid}` — HR brings an incident up to date, because a class and a count of days change as a case goes on.
#[debug_handler]
async fn update(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<Uuid>,
    Json(p): Json<UpdatePayload>,
) -> Result<Response> {
    let record = find_record(&ctx, pid).await?;
    ensure_hr(&ctx, &caller, &record.organization_ref).await?;
    ensure_valid(&rules::problems(
        &record.jurisdiction,
        &record.kind,
        &p.class,
        utc(record.occurred_at),
        p.learned_at,
        p.absence_days,
        p.people_affected,
        Utc::now(),
    ))?;
    let reference =
        rules::clean_ref(p.statutory_record_ref.as_deref()).map_err(|e| unprocessable(&e))?;
    let txn = ctx.db.begin().await?;
    let mut row: injury_records::ActiveModel = record.into();
    row.learned_at = ActiveValue::set(p.learned_at.map(|at| at.fixed_offset()));
    row.class = ActiveValue::set(p.class);
    row.absence_days = ActiveValue::set(p.absence_days);
    row.people_affected = ActiveValue::set(p.people_affected);
    row.statutory_record_ref = ActiveValue::set(reference);
    row.keep_until = ActiveValue::set(p.keep_until);
    row.update(&txn).await?;
    Audit::record(
        &txn,
        "injury_record",
        pid,
        "injury_updated",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(json!({ "pid": pid }))
}

/// The body of a report.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReportPayload {
    recipient: String,
    /// When it is due, as the person filing reads the law. Absent where the law says "immediately".
    due_at: Option<DateTime<Utc>>,
    sent_at: Option<DateTime<Utc>>,
    method: Option<String>,
    reference: Option<String>,
}

/// `POST /api/injury-records/{pid}/reports` — HR adds a report the incident owes, sent or not. One incident can owe several.
#[debug_handler]
async fn add_report(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<Uuid>,
    Json(p): Json<ReportPayload>,
) -> Result<Response> {
    let record = find_record(&ctx, pid).await?;
    ensure_hr(&ctx, &caller, &record.organization_ref).await?;
    ensure_valid(&rules::report_problems(
        &p.recipient,
        p.method.as_deref(),
        p.sent_at,
        Utc::now(),
    ))?;
    let reference = rules::clean_ref(p.reference.as_deref()).map_err(|e| unprocessable(&e))?;
    let report = Uuid::new_v4();
    let txn = ctx.db.begin().await?;
    injury_reports::ActiveModel {
        pid: ActiveValue::set(report),
        record_pid: ActiveValue::set(pid),
        recipient: ActiveValue::set(p.recipient),
        due_at: ActiveValue::set(p.due_at.map(|at| at.fixed_offset())),
        sent_at: ActiveValue::set(p.sent_at.map(|at| at.fixed_offset())),
        method: ActiveValue::set(p.method),
        reference: ActiveValue::set(reference),
        recorded_by: ActiveValue::set(caller.actor().map(ToString::to_string)),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    Audit::record(
        &txn,
        "injury_record",
        pid,
        "injury_report_added",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(json!({ "pid": report }))
}

/// The body that says a report went.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SentPayload {
    sent_at: DateTime<Utc>,
    method: String,
    reference: Option<String>,
}

/// `PUT /api/injury-reports/{pid}` — HR says when and how a report went, and its reference. A report already sent can be corrected.
#[debug_handler]
async fn mark_sent(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<Uuid>,
    Json(p): Json<SentPayload>,
) -> Result<Response> {
    let report = injury_reports::Entity::find()
        .filter(injury_reports::Column::Pid.eq(pid))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    let record = find_record(&ctx, report.record_pid).await?;
    ensure_hr(&ctx, &caller, &record.organization_ref).await?;
    ensure_valid(&rules::report_problems(
        &report.recipient,
        Some(&p.method),
        Some(p.sent_at),
        Utc::now(),
    ))?;
    let reference = rules::clean_ref(p.reference.as_deref()).map_err(|e| unprocessable(&e))?;
    let txn = ctx.db.begin().await?;
    let mut row: injury_reports::ActiveModel = report.into();
    row.sent_at = ActiveValue::set(Some(p.sent_at.fixed_offset()));
    row.method = ActiveValue::set(Some(p.method));
    row.reference = ActiveValue::set(reference);
    row.update(&txn).await?;
    Audit::record(
        &txn,
        "injury_record",
        record.pid,
        "injury_report_sent",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(json!({ "pid": pid }))
}

/// Query of the roll-up.
#[derive(Debug, Deserialize)]
struct RollupQuery {
    organization: String,
    year: i32,
}

/// `GET /api/injury-records/rollup?organization=&year=` — counts by class for the calendar year, with the floor. For HR and payroll,
/// who may hold the report it feeds. Names no one and holds no pid.
#[debug_handler]
async fn rollup(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(q): axum::extract::Query<RollupQuery>,
) -> Result<Response> {
    ensure_org_in_scope(&ctx, &caller, &q.organization).await?;
    if !super::policies::privileged(&ctx, &caller, &q.organization).await? {
        return Err(super::forbidden("this is done by HR"));
    }
    let classes: Vec<String> = injury_records::Entity::find()
        .filter(injury_records::Column::OrganizationRef.eq(&q.organization))
        .all(&ctx.db)
        .await?
        .into_iter()
        .filter(|r| r.occurred_at.year() == q.year)
        .map(|r| r.class)
        .collect();
    let floor = equality_monitoring::DEFAULT_FLOOR;
    let counts = rules::rollup(&classes, floor);
    format::json(json!({
        "year": q.year,
        "floor": floor,
        "classes": counts.cells.iter().map(|(class, count)| json!({ "class": class, "count": count })).collect::<Vec<_>>(),
        "total": counts.total,
        "derivation": "Incidents that happened in the calendar year, by class. A count under the floor is withheld, with a second cell or the total where one could be worked out from the other. People affected are not added up.",
    }))
}

/// The routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/injury-records", get(list))
        .add("/injury-records", post(create))
        .add("/injury-records/rollup", get(rollup))
        .add("/injury-records/{pid}", put(update))
        .add("/injury-records/{pid}/reports", post(add_report))
        .add("/injury-reports/{pid}", put(mark_sent))
}
