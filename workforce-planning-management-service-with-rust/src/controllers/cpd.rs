//! The **CPD ledger** — continuing professional development as a
//! first-class record: requirements (hours or points in a period), what a
//! worker did (entries, with evidence and optional verification), and
//! professional registrations with an expiry. Pure rules live in
//! [`crate::rules::cpd`].
//!
//! Amounts travel as plain numbers of units (`1.5` hours) and are stored
//! as whole hundredths. A worker with no entries has a real `0` recorded
//! against a *defined* requirement; ratios over an empty population are
//! `null`, never `0`.

use super::ratio_json;
use crate::rules::display;
use loco_rs::prelude::*;
use sea_orm::{ActiveValue, PaginatorTrait, QueryOrder, QuerySelect, QueryTrait, TransactionTrait};
use serde::Deserialize;
use uuid::Uuid;

use super::{ensure_valid, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::models::_entities::{
    cpd_entries, cpd_requirements, professional_registrations, workers,
};
use crate::models::audit_logs::Model as Audit;
use crate::models::records;
use crate::rules::cpd as rules;
use crate::rules::talent::ratio;
use crate::validation::Problems;

/// A `{pid}` reference response.
#[derive(serde::Serialize)]
struct PidRef {
    pid: String,
}

/// A pace as the progress payload shows it: amounts in units, and the basis it rests on.
fn pace_json(pace: &rules::Pace) -> serde_json::Value {
    serde_json::json!({
        "status": pace.status,
        "days_left": pace.days_left,
        "needed_per_30_days": pace.needed_per_30_days_hundredths.map(display::from_hundredths),
        "expected_by_today": pace.expected_by_today_hundredths.map(display::from_hundredths),
        "basis": rules::PACE_BASIS,
    })
}

/// Whether an evidence URL is an http(s) link.
fn is_web_url(url: &str) -> bool {
    url.starts_with("https://") || url.starts_with("http://")
}

// ─── Requirements ───────────────────────────────────────────────────────────

/// `POST /api/cpd-requirements` body.
#[derive(Debug, Deserialize)]
struct RequirementPayload {
    name: String,
    unit: String,
    /// Units required in the period (e.g. `30` hours).
    required: f64,
    period_start: chrono::NaiveDate,
    period_end: chrono::NaiveDate,
    /// Restrict to one job title; absent ⇒ every worker.
    #[serde(default)]
    job_title: Option<String>,
}

/// The checked fields of a requirement body: its name, required hundredths and job title.
fn checked_requirement(payload: &RequirementPayload) -> Result<(String, i64, Option<String>)> {
    let mut problems = Problems::new();
    problems.require_text("name", &payload.name);
    problems.cap_text("name", &payload.name);
    if let Some(title) = &payload.job_title {
        problems.cap_text("job_title", title);
    }
    ensure_valid(&problems.into_vec())?;
    let required = rules::to_hundredths(payload.required)
        .ok_or_else(|| unprocessable("required must be a positive amount"))?;
    rules::validate_requirement(
        &payload.unit,
        required,
        payload.period_start,
        payload.period_end,
    )
    .map_err(|e| unprocessable(&e))?;
    let job_title = payload
        .job_title
        .as_deref()
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(ToString::to_string);
    Ok((payload.name.trim().to_string(), required, job_title))
}

/// `POST /api/cpd-requirements` — define what must be done in a period.
#[debug_handler]
async fn create_requirement(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<RequirementPayload>,
) -> Result<Response> {
    let (name, required, job_title) = checked_requirement(&payload)?;
    let txn = ctx.db.begin().await?;
    let row = cpd_requirements::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        name: ActiveValue::set(name),
        unit: ActiveValue::set(payload.unit.clone()),
        required_hundredths: ActiveValue::set(required),
        period_start: ActiveValue::set(payload.period_start),
        period_end: ActiveValue::set(payload.period_end),
        job_title: ActiveValue::set(job_title),
        deleted_at: ActiveValue::set(None),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    Audit::record(
        &txn,
        "cpd_requirement",
        row.pid,
        "created",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(PidRef {
        pid: row.pid.to_string(),
    })
}

async fn find_requirement(ctx: &AppContext, pid: &str) -> Result<cpd_requirements::Model> {
    cpd_requirements::Entity::find()
        .filter(cpd_requirements::Column::Pid.eq(records::parse_pid(pid)?))
        .filter(cpd_requirements::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)
}

/// `PUT /api/cpd-requirements/{pid}` — correct a requirement (WPM-R139). The unit stays: entries were
/// counted in it. Entries are untouched and are counted afresh against the new terms.
#[debug_handler]
async fn update_requirement(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<RequirementPayload>,
) -> Result<Response> {
    let existing = find_requirement(&ctx, &pid).await?;
    if payload.unit != existing.unit {
        return Err(unprocessable(
            "the unit of a requirement cannot change; withdraw it and define a new one",
        ));
    }
    let (name, required, job_title) = checked_requirement(&payload)?;
    let mut active: cpd_requirements::ActiveModel = existing.into();
    active.name = ActiveValue::set(name);
    active.required_hundredths = ActiveValue::set(required);
    active.period_start = ActiveValue::set(payload.period_start);
    active.period_end = ActiveValue::set(payload.period_end);
    active.job_title = ActiveValue::set(job_title);
    let txn = ctx.db.begin().await?;
    let row = active.update(&txn).await?;
    Audit::record(
        &txn,
        "cpd_requirement",
        row.pid,
        "updated",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(PidRef {
        pid: row.pid.to_string(),
    })
}

/// `DELETE /api/cpd-requirements/{pid}` — withdraw a requirement (soft delete).
#[debug_handler]
async fn delete_requirement(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let existing = find_requirement(&ctx, &pid).await?;
    let mut active: cpd_requirements::ActiveModel = existing.into();
    active.deleted_at = ActiveValue::set(Some(chrono::Utc::now().into()));
    let txn = ctx.db.begin().await?;
    let row = active.update(&txn).await?;
    Audit::record(
        &txn,
        "cpd_requirement",
        row.pid,
        "withdrawn",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::empty_json()
}

/// `GET /api/cpd-requirements` — live requirements, newest period first.
#[debug_handler]
async fn list_requirements(State(ctx): State<AppContext>) -> Result<Response> {
    let rows = live_requirements(&ctx).await?;
    let out: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "pid": r.pid,
                "name": r.name,
                "unit": r.unit,
                "required": display::from_hundredths(r.required_hundredths),
                "required_hundredths": r.required_hundredths,
                "period_start": r.period_start,
                "period_end": r.period_end,
                "job_title": r.job_title,
            })
        })
        .collect();
    format::json(out)
}

async fn live_requirements(ctx: &AppContext) -> Result<Vec<cpd_requirements::Model>> {
    Ok(cpd_requirements::Entity::find()
        .filter(cpd_requirements::Column::DeletedAt.is_null())
        .order_by_desc(cpd_requirements::Column::PeriodStart)
        .all(&ctx.db)
        .await?)
}

// ─── Entries ────────────────────────────────────────────────────────────────

/// `POST /api/workers/{pid}/cpd-entries` body.
#[derive(Debug, Deserialize)]
struct EntryPayload {
    entry_date: chrono::NaiveDate,
    activity: String,
    category: String,
    unit: String,
    /// Units done (e.g. `1.5` hours).
    amount: f64,
    #[serde(default)]
    evidence_note: Option<String>,
    #[serde(default)]
    evidence_url: Option<String>,
}

/// `POST /api/workers/{pid}/cpd-entries` — record a CPD activity.
#[debug_handler]
async fn create_entry(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<EntryPayload>,
) -> Result<Response> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    let mut problems = Problems::new();
    problems.require_text("activity", &payload.activity);
    problems.cap_text("activity", &payload.activity);
    if let Some(note) = &payload.evidence_note {
        problems.cap_text("evidence_note", note);
    }
    if let Some(url) = &payload.evidence_url {
        problems.cap_text("evidence_url", url);
        if !is_web_url(url) {
            problems.push("evidence_url must be an http(s) link");
        }
    }
    ensure_valid(&problems.into_vec())?;
    let amount = rules::to_hundredths(payload.amount)
        .ok_or_else(|| unprocessable("amount must be a positive number of units"))?;
    rules::validate_entry(
        &payload.unit,
        &payload.category,
        amount,
        payload.entry_date,
        chrono::Utc::now().date_naive(),
    )
    .map_err(|e| unprocessable(&e))?;
    let txn = ctx.db.begin().await?;
    let row = cpd_entries::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        worker_pid: ActiveValue::set(worker.pid),
        entry_date: ActiveValue::set(payload.entry_date),
        activity: ActiveValue::set(payload.activity.trim().to_string()),
        category: ActiveValue::set(payload.category.clone()),
        unit: ActiveValue::set(payload.unit.clone()),
        amount_hundredths: ActiveValue::set(amount),
        evidence_note: ActiveValue::set(payload.evidence_note.clone()),
        evidence_url: ActiveValue::set(payload.evidence_url.clone()),
        source: ActiveValue::set("manual".to_string()),
        external_ref: ActiveValue::set(None),
        course_ref: ActiveValue::set(None),
        verified_on: ActiveValue::set(None),
        verified_by: ActiveValue::set(None),
        deleted_at: ActiveValue::set(None),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    Audit::record(&txn, "cpd_entry", row.pid, "recorded", caller.actor(), None).await?;
    txn.commit().await?;
    format::json(PidRef {
        pid: row.pid.to_string(),
    })
}

fn entry_json(e: &cpd_entries::Model) -> serde_json::Value {
    serde_json::json!({
        "pid": e.pid,
        "entry_date": e.entry_date,
        "activity": e.activity,
        "category": e.category,
        "unit": e.unit,
        "amount": display::from_hundredths(e.amount_hundredths),
        "amount_hundredths": e.amount_hundredths,
        "evidence_note": e.evidence_note,
        "evidence_url": e.evidence_url,
        "source": e.source,
        "lms_url": e.course_ref.as_deref().and_then(super::lms::course_url),
        "verified_on": e.verified_on,
        "verified_by": e.verified_by,
    })
}

async fn worker_entries(ctx: &AppContext, worker_pid: Uuid) -> Result<Vec<cpd_entries::Model>> {
    Ok(cpd_entries::Entity::find()
        .filter(cpd_entries::Column::WorkerPid.eq(worker_pid))
        .filter(cpd_entries::Column::DeletedAt.is_null())
        .order_by_desc(cpd_entries::Column::EntryDate)
        .all(&ctx.db)
        .await?)
}

/// `GET /api/workers/{pid}/cpd-entries` — a worker's ledger, newest first.
#[debug_handler]
async fn list_entries(State(ctx): State<AppContext>, Path(pid): Path<String>) -> Result<Response> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    let rows = worker_entries(&ctx, worker.pid).await?;
    format::json(rows.iter().map(entry_json).collect::<Vec<_>>())
}

/// `POST /api/cpd-entries/{pid}/verify` — mark an entry verified (a
/// manager or HR confirming the evidence).
#[debug_handler]
async fn verify_entry(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let entry = find_entry(&ctx, &pid).await?;
    if entry.verified_on.is_some() {
        return Err(unprocessable("entry is already verified"));
    }
    let mut active: cpd_entries::ActiveModel = entry.into();
    active.verified_on = ActiveValue::set(Some(chrono::Utc::now().date_naive()));
    active.verified_by = ActiveValue::set(caller.actor().map(ToString::to_string));
    let txn = ctx.db.begin().await?;
    let row = active.update(&txn).await?;
    Audit::record(&txn, "cpd_entry", row.pid, "verified", caller.actor(), None).await?;
    txn.commit().await?;
    format::json(entry_json(&row))
}

/// `DELETE /api/cpd-entries/{pid}` — withdraw an entry (soft-delete).
#[debug_handler]
async fn delete_entry(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let entry = find_entry(&ctx, &pid).await?;
    let mut active: cpd_entries::ActiveModel = entry.into();
    active.deleted_at = ActiveValue::set(Some(chrono::Utc::now().into()));
    let txn = ctx.db.begin().await?;
    let row = active.update(&txn).await?;
    Audit::record(
        &txn,
        "cpd_entry",
        row.pid,
        "withdrawn",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::empty_json()
}

async fn find_entry(ctx: &AppContext, pid: &str) -> Result<cpd_entries::Model> {
    cpd_entries::Entity::find()
        .filter(cpd_entries::Column::Pid.eq(records::parse_pid(pid)?))
        .filter(cpd_entries::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)
}

// ─── Registrations ──────────────────────────────────────────────────────────

/// `POST /api/workers/{pid}/registrations` body.
#[derive(Debug, Deserialize)]
struct RegistrationPayload {
    body: String,
    #[serde(default)]
    reference: Option<String>,
    #[serde(default)]
    registered_on: Option<chrono::NaiveDate>,
    #[serde(default)]
    expires_on: Option<chrono::NaiveDate>,
}

/// `POST /api/workers/{pid}/registrations` — record a professional
/// registration (e.g. a regulator's register) with its expiry.
#[debug_handler]
async fn create_registration(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<RegistrationPayload>,
) -> Result<Response> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    let mut problems = Problems::new();
    problems.require_text("body", &payload.body);
    problems.cap_text("body", &payload.body);
    if let Some(reference) = &payload.reference {
        problems.cap_text("reference", reference);
    }
    if let (Some(from), Some(to)) = (payload.registered_on, payload.expires_on)
        && to < from
    {
        problems.push("expires_on must not be before registered_on");
    }
    ensure_valid(&problems.into_vec())?;
    let txn = ctx.db.begin().await?;
    let row = professional_registrations::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        worker_pid: ActiveValue::set(worker.pid),
        body: ActiveValue::set(payload.body.trim().to_string()),
        reference: ActiveValue::set(payload.reference.clone()),
        registered_on: ActiveValue::set(payload.registered_on),
        expires_on: ActiveValue::set(payload.expires_on),
        deleted_at: ActiveValue::set(None),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    Audit::record(
        &txn,
        "professional_registration",
        row.pid,
        "recorded",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(PidRef {
        pid: row.pid.to_string(),
    })
}

async fn worker_registrations(
    ctx: &AppContext,
    worker_pid: Uuid,
) -> Result<Vec<professional_registrations::Model>> {
    Ok(professional_registrations::Entity::find()
        .filter(professional_registrations::Column::WorkerPid.eq(worker_pid))
        .filter(professional_registrations::Column::DeletedAt.is_null())
        .order_by_asc(professional_registrations::Column::ExpiresOn)
        .all(&ctx.db)
        .await?)
}

/// `GET /api/workers/{pid}/registrations` — with each one's expiry status.
#[debug_handler]
async fn list_registrations(
    State(ctx): State<AppContext>,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    let today = chrono::Utc::now().date_naive();
    let rows = worker_registrations(&ctx, worker.pid).await?;
    let out: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            serde_json::json!({
                "pid": r.pid,
                "body": r.body,
                "reference": r.reference,
                "registered_on": r.registered_on,
                "expires_on": r.expires_on,
                "status": rules::registration_status(r.expires_on, today, rules::DEFAULT_WARN_DAYS),
            })
        })
        .collect();
    format::json(out)
}

// ─── Progress + overview ────────────────────────────────────────────────────

/// `GET /api/workers/{pid}/cpd-progress` — for every live requirement
/// that applies to this worker: recorded and verified against the
/// requirement (entries in the period, in the requirement's unit), plus
/// the worker's registrations with expiry status.
#[debug_handler]
async fn progress(State(ctx): State<AppContext>, Path(pid): Path<String>) -> Result<Response> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    let today = chrono::Utc::now().date_naive();
    let entries = worker_entries(&ctx, worker.pid).await?;
    let requirements: Vec<serde_json::Value> = live_requirements(&ctx)
        .await?
        .iter()
        .filter(|r| rules::applies_to(r.job_title.as_deref(), &worker.job_title))
        .map(|r| {
            let counted: Vec<(i64, bool)> = entries
                .iter()
                .filter(|e| {
                    e.unit == r.unit && rules::in_period(e.entry_date, r.period_start, r.period_end)
                })
                .map(|e| (e.amount_hundredths, e.verified_on.is_some()))
                .collect();
            let p = rules::progress(&counted, r.required_hundredths);
            serde_json::json!({
                "requirement_pid": r.pid,
                "name": r.name,
                "unit": r.unit,
                "period_start": r.period_start,
                "period_end": r.period_end,
                "required": display::from_hundredths(p.required),
                "recorded": display::from_hundredths(p.recorded),
                "verified": display::from_hundredths(p.verified),
                "remaining": display::from_hundredths(p.remaining()),
                "met": p.met(),
                "met_verified": p.met_verified(),
                "pace": pace_json(&rules::pace(
                    p.recorded,
                    p.required,
                    r.period_start,
                    r.period_end,
                    today,
                )),
            })
        })
        .collect();
    let registrations: Vec<serde_json::Value> = worker_registrations(&ctx, worker.pid)
        .await?
        .iter()
        .map(|r| {
            serde_json::json!({
                "body": r.body,
                "expires_on": r.expires_on,
                "status": rules::registration_status(r.expires_on, today, rules::DEFAULT_WARN_DAYS),
            })
        })
        .collect();
    format::json(serde_json::json!({
        "derivation": "recorded counts the worker's own entries in the requirement's unit and \
                       period; verified is the part a manager or HR has confirmed. A worker with \
                       no entries has a real 0 against a defined requirement.",
        "worker_pid": worker.pid,
        "requirements": requirements,
        "registrations": registrations,
    }))
}

/// `GET /api/cpd/overview` — aggregate, no one named: per requirement,
/// how many **employed** workers it applies to, how many have met it on
/// recorded and on verified entries, and across the whole workforce how
/// many registrations are expiring or expired.
#[debug_handler]
async fn overview(State(ctx): State<AppContext>) -> Result<Response> {
    let today = chrono::Utc::now().date_naive();
    let employed = || {
        workers::Entity::find()
            .filter(workers::Column::DeletedAt.is_null())
            .filter(workers::Column::HiredOn.lte(today))
            .filter(
                Condition::any()
                    .add(workers::Column::TerminatedOn.is_null())
                    .add(workers::Column::TerminatedOn.gt(today)),
            )
    };
    let headcount = employed().count(&ctx.db).await?;
    // One aggregate per requirement, worked out by the database: no worker and no entry is read into memory.
    let mut requirements: Vec<serde_json::Value> = Vec::new();
    for r in live_requirements(&ctx).await? {
        let (applicable, met, met_verified) = requirement_counts(&ctx, &r, today).await?;
        requirements.push(serde_json::json!({
            "requirement_pid": r.pid,
            "name": r.name,
            "unit": r.unit,
            "period_start": r.period_start,
            "period_end": r.period_end,
            "applicable_workers": applicable,
            "met_recorded": ratio_json(ratio(met, applicable)),
            "met_verified": ratio_json(ratio(met_verified, applicable)),
        }));
    }
    // Only the registrations that need attention are read: expired, or expiring inside the warning window.
    let horizon = today + chrono::Days::new(u64::try_from(rules::DEFAULT_WARN_DAYS).unwrap_or(90));
    let (mut expiring, mut expired) = (0usize, 0usize);
    for registration in professional_registrations::Entity::find()
        .filter(professional_registrations::Column::DeletedAt.is_null())
        .filter(professional_registrations::Column::ExpiresOn.lte(horizon))
        .filter(
            professional_registrations::Column::WorkerPid.in_subquery(
                employed()
                    .select_only()
                    .column(workers::Column::Pid)
                    .into_query(),
            ),
        )
        .all(&ctx.db)
        .await?
    {
        match rules::registration_status(registration.expires_on, today, rules::DEFAULT_WARN_DAYS) {
            "expiring" => expiring += 1,
            "expired" => expired += 1,
            _ => {}
        }
    }
    format::json(serde_json::json!({
        "derivation": "counts EMPLOYED workers (the shared employed-on-date definition) the \
                       requirement applies to; met_recorded uses all entries, met_verified only \
                       verified ones; registrations expiring are within 90 days",
        "headcount": headcount,
        "requirements": requirements,
        "registrations": { "expiring": expiring, "expired": expired },
    }))
}

/// For one requirement: how many employed workers it applies to, how many have recorded at least the
/// required amount in its unit and period, and how many have verified at least that much. The same
/// definitions as [`rules::applies_to`], [`rules::in_period`] and [`rules::progress`], done in SQL.
async fn requirement_counts(
    ctx: &AppContext,
    r: &cpd_requirements::Model,
    today: chrono::NaiveDate,
) -> Result<(usize, usize, usize)> {
    use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};
    let statement = Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        "SELECT count(*) AS applicable,
                count(*) FILTER (WHERE recorded >= $1) AS met,
                count(*) FILTER (WHERE verified >= $1) AS met_verified
           FROM (
             SELECT COALESCE(SUM(e.amount_hundredths), 0) AS recorded,
                    COALESCE(SUM(e.amount_hundredths) FILTER (WHERE e.verified_on IS NOT NULL), 0) AS verified
               FROM workers w
               LEFT JOIN cpd_entries e
                      ON e.worker_pid = w.pid AND e.deleted_at IS NULL AND e.unit = $2
                     AND e.entry_date BETWEEN $3 AND $4
              WHERE w.deleted_at IS NULL AND w.hired_on <= $5
                AND (w.terminated_on IS NULL OR w.terminated_on > $5)
                AND ($6::text IS NULL OR lower(btrim(w.job_title)) = lower(btrim($6::text)))
              GROUP BY w.pid
           ) per_worker",
        [
            r.required_hundredths.into(),
            r.unit.clone().into(),
            r.period_start.into(),
            r.period_end.into(),
            today.into(),
            r.job_title.clone().into(),
        ],
    );
    let row = ctx
        .db
        .query_one_raw(statement)
        .await?
        .ok_or_else(|| Error::string("the requirement count returned no row"))?;
    let count = |name: &str| -> Result<usize> {
        let n: i64 = row.try_get("", name)?;
        Ok(usize::try_from(n).unwrap_or(0))
    };
    Ok((count("applicable")?, count("met")?, count("met_verified")?))
}

/// The CPD routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/cpd-requirements", post(create_requirement))
        .add("/cpd-requirements", get(list_requirements))
        .add("/cpd-requirements/{pid}", put(update_requirement))
        .add("/cpd-requirements/{pid}", delete(delete_requirement))
        .add("/workers/{pid}/cpd-entries", post(create_entry))
        .add("/workers/{pid}/cpd-entries", get(list_entries))
        .add("/workers/{pid}/cpd-progress", get(progress))
        .add("/workers/{pid}/registrations", post(create_registration))
        .add("/workers/{pid}/registrations", get(list_registrations))
        .add("/cpd-entries/{pid}/verify", post(verify_entry))
        .add("/cpd-entries/{pid}", delete(delete_entry))
        .add("/cpd/overview", get(overview))
}
