//! The injury and illness notification record (WPM-R221, WPM-D158, WPM-D179, WPM-T421). HR records an incident and each report it owes;
//! the list says where each report stands against the times HR entered. A row proves a report and points to the statutory record, which
//! stays outside WPM. It takes no description, and a body with any field it does not know is refused. WPM decides no duty and files
//! nothing. HR, the service and an administrator read it, never a line manager or payroll, and each read is audited with no content.
//! The rules are `rules::injury_record`. Spec: `spec/injury-and-separation-duties.md`.

use axa::extract::{Json, Path};
use axa::prelude::*;
use chrono::{DateTime, NaiveDate, Utc};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashMap};
use uuid::Uuid;

use super::{ensure_org_in_scope, ensure_valid, record_rejection, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::db::{self, pool};
use crate::models::_entities::{injury_records, injury_reports};
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
    sqlx::query_as!(
        injury_records::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _",
                  id, pid, organization_ref, jurisdiction, kind, worker_pid,
                  occurred_at AS "occurred_at: _", learned_at AS "learned_at: _", class,
                  absence_days, people_affected, statutory_record_ref, keep_until, recorded_by
           FROM injury_records WHERE pid = $1"#,
        pid,
    )
    .fetch_optional(pool(ctx))
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
    axa::extract::Query(q): axa::extract::Query<ListQuery>,
) -> Result<Response> {
    ensure_hr(&ctx, &caller, &q.organization).await?;
    let rows = sqlx::query_as!(
        injury_records::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _",
                  id, pid, organization_ref, jurisdiction, kind, worker_pid,
                  occurred_at AS "occurred_at: _", learned_at AS "learned_at: _", class,
                  absence_days, people_affected, statutory_record_ref, keep_until, recorded_by
           FROM injury_records
           WHERE organization_ref = $1
           ORDER BY occurred_at DESC, id DESC"#,
        q.organization,
    )
    .fetch_all(pool(&ctx))
    .await?;
    let record_pids: Vec<Uuid> = rows.iter().map(|r| r.pid).collect();
    let reports = sqlx::query_as!(
        injury_reports::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _",
                  id, pid, record_pid, recipient, due_at AS "due_at: _", sent_at AS "sent_at: _",
                  method, reference, recorded_by
           FROM injury_reports
           WHERE record_pid = ANY($1)
           ORDER BY id"#,
        &record_pids,
    )
    .fetch_all(pool(&ctx))
    .await?;
    let worker_pids: Vec<Uuid> = rows.iter().filter_map(|r| r.worker_pid).collect();
    let names: HashMap<Uuid, String> = sqlx::query!(
        "SELECT pid, display_name FROM workers WHERE pid = ANY($1)",
        &worker_pids,
    )
    .fetch_all(pool(&ctx))
    .await?
    .into_iter()
    .map(|w| (w.pid, w.display_name))
    .collect();
    db::audit(
        &mut *pool(&ctx).acquire().await?,
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
        let worker = records::worker_by_pid(&ctx, worker)
            .await
            .map_err(|_| unprocessable("that worker is not on record"))?;
        if worker.organization_ref != p.organization {
            return Err(unprocessable("that worker is not in this organization"));
        }
    }
    let pid = Uuid::new_v4();
    let mut tx = pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO injury_records
             (pid, organization_ref, jurisdiction, kind, worker_pid, occurred_at, learned_at,
              class, absence_days, people_affected, statutory_record_ref, keep_until, recorded_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)",
        pid,
        p.organization,
        p.jurisdiction,
        p.kind,
        p.worker_pid,
        p.occurred_at,
        p.learned_at,
        p.class,
        p.absence_days,
        people,
        reference,
        p.keep_until,
        caller.actor(),
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "injury_record",
        pid,
        "injury_recorded",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
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
    let mut tx = pool(&ctx).begin().await?;
    sqlx::query!(
        "UPDATE injury_records
         SET learned_at = $2, class = $3, absence_days = $4, people_affected = $5,
             statutory_record_ref = $6, keep_until = $7
         WHERE pid = $1",
        pid,
        p.learned_at,
        p.class,
        p.absence_days,
        p.people_affected,
        reference,
        p.keep_until,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "injury_record",
        pid,
        "injury_updated",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
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
    let mut tx = pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO injury_reports
             (pid, record_pid, recipient, due_at, sent_at, method, reference, recorded_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        report,
        pid,
        p.recipient,
        p.due_at,
        p.sent_at,
        p.method,
        reference,
        caller.actor(),
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "injury_record",
        pid,
        "injury_report_added",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
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
    let report = sqlx::query!(
        "SELECT record_pid, recipient FROM injury_reports WHERE pid = $1",
        pid,
    )
    .fetch_optional(pool(&ctx))
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
    let mut tx = pool(&ctx).begin().await?;
    sqlx::query!(
        "UPDATE injury_reports SET sent_at = $2, method = $3, reference = $4 WHERE pid = $1",
        pid,
        p.sent_at,
        p.method,
        reference,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "injury_record",
        record.pid,
        "injury_report_sent",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
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
    axa::extract::Query(q): axa::extract::Query<RollupQuery>,
) -> Result<Response> {
    ensure_org_in_scope(&ctx, &caller, &q.organization).await?;
    if !super::policies::privileged(&ctx, &caller, &q.organization).await? {
        return Err(super::forbidden("this is done by HR"));
    }
    // The calendar year in UTC, as an incident's time is held.
    let classes: Vec<String> = sqlx::query_scalar!(
        "SELECT class FROM injury_records
         WHERE organization_ref = $1
           AND EXTRACT(YEAR FROM occurred_at AT TIME ZONE 'UTC')::int = $2",
        q.organization,
        q.year,
    )
    .fetch_all(pool(&ctx))
    .await?;
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
