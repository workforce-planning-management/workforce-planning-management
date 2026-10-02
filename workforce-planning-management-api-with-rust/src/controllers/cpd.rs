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

use loco_rs::prelude::*;
use sea_orm::{ActiveValue, QueryOrder};
use serde::Deserialize;
use std::collections::BTreeMap;
use uuid::Uuid;

use super::{ensure_valid, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::models::_entities::{
    cpd_entries, cpd_requirements, professional_registrations, workers,
};
use crate::models::audit_logs::Model as Audit;
use crate::models::records;
use crate::rules::cpd as rules;
use crate::rules::metrics::is_employed_on;
use crate::rules::talent::ratio;
use crate::validation::Problems;

/// A `{pid}` reference response.
#[derive(serde::Serialize)]
struct PidRef {
    pid: String,
}

/// Render a ratio triple as a terms-carrying object, or `null`.
fn ratio_json(terms: Option<(usize, usize, f64)>) -> serde_json::Value {
    terms.map_or(serde_json::Value::Null, |(numerator, denominator, value)| {
        serde_json::json!({ "numerator": numerator, "denominator": denominator, "value": value })
    })
}

/// Hundredths → units for display.
#[allow(clippy::cast_precision_loss)] // display value; the integer is returned alongside
fn units(hundredths: i64) -> f64 {
    hundredths as f64 / 100.0
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

/// `POST /api/cpd-requirements` — define what must be done in a period.
#[debug_handler]
async fn create_requirement(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<RequirementPayload>,
) -> Result<Response> {
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
    let row = cpd_requirements::ActiveModel {
        pid: ActiveValue::set(Uuid::new_v4()),
        name: ActiveValue::set(payload.name.trim().to_string()),
        unit: ActiveValue::set(payload.unit.clone()),
        required_hundredths: ActiveValue::set(required),
        period_start: ActiveValue::set(payload.period_start),
        period_end: ActiveValue::set(payload.period_end),
        job_title: ActiveValue::set(job_title),
        deleted_at: ActiveValue::set(None),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;
    Audit::record(
        &ctx.db,
        "cpd_requirement",
        row.pid,
        "created",
        caller.actor(),
        None,
    )
    .await?;
    format::json(PidRef {
        pid: row.pid.to_string(),
    })
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
                "required": units(r.required_hundredths),
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
        verified_on: ActiveValue::set(None),
        verified_by: ActiveValue::set(None),
        deleted_at: ActiveValue::set(None),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;
    Audit::record(
        &ctx.db,
        "cpd_entry",
        row.pid,
        "recorded",
        caller.actor(),
        None,
    )
    .await?;
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
        "amount": units(e.amount_hundredths),
        "amount_hundredths": e.amount_hundredths,
        "evidence_note": e.evidence_note,
        "evidence_url": e.evidence_url,
        "source": e.source,
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
    let row = active.update(&ctx.db).await?;
    Audit::record(
        &ctx.db,
        "cpd_entry",
        row.pid,
        "verified",
        caller.actor(),
        None,
    )
    .await?;
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
    let row = active.update(&ctx.db).await?;
    Audit::record(
        &ctx.db,
        "cpd_entry",
        row.pid,
        "withdrawn",
        caller.actor(),
        None,
    )
    .await?;
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
    .insert(&ctx.db)
    .await?;
    Audit::record(
        &ctx.db,
        "professional_registration",
        row.pid,
        "recorded",
        caller.actor(),
        None,
    )
    .await?;
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
                "required": units(p.required),
                "recorded": units(p.recorded),
                "verified": units(p.verified),
                "remaining": units(p.remaining()),
                "met": p.met(),
                "met_verified": p.met_verified(),
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
    let staff: Vec<workers::Model> = workers::Entity::find()
        .filter(workers::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?
        .into_iter()
        .filter(|w| is_employed_on(today, w.hired_on, w.terminated_on))
        .collect();
    let entries = cpd_entries::Entity::find()
        .filter(cpd_entries::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?;
    let mut by_worker: BTreeMap<Uuid, Vec<&cpd_entries::Model>> = BTreeMap::new();
    for entry in &entries {
        by_worker.entry(entry.worker_pid).or_default().push(entry);
    }
    let requirements: Vec<serde_json::Value> = live_requirements(&ctx)
        .await?
        .iter()
        .map(|r| {
            let (mut applicable, mut met, mut met_verified) = (0usize, 0usize, 0usize);
            for worker in staff
                .iter()
                .filter(|w| rules::applies_to(r.job_title.as_deref(), &w.job_title))
            {
                applicable += 1;
                let counted: Vec<(i64, bool)> = by_worker
                    .get(&worker.pid)
                    .map(|list| {
                        list.iter()
                            .filter(|e| {
                                e.unit == r.unit
                                    && rules::in_period(e.entry_date, r.period_start, r.period_end)
                            })
                            .map(|e| (e.amount_hundredths, e.verified_on.is_some()))
                            .collect()
                    })
                    .unwrap_or_default();
                let p = rules::progress(&counted, r.required_hundredths);
                met += usize::from(p.met());
                met_verified += usize::from(p.met_verified());
            }
            serde_json::json!({
                "requirement_pid": r.pid,
                "name": r.name,
                "unit": r.unit,
                "period_start": r.period_start,
                "period_end": r.period_end,
                "applicable_workers": applicable,
                "met_recorded": ratio_json(ratio(met, applicable)),
                "met_verified": ratio_json(ratio(met_verified, applicable)),
            })
        })
        .collect();
    let employed: std::collections::HashSet<Uuid> = staff.iter().map(|w| w.pid).collect();
    let (mut expiring, mut expired) = (0usize, 0usize);
    for registration in professional_registrations::Entity::find()
        .filter(professional_registrations::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?
        .iter()
        .filter(|r| employed.contains(&r.worker_pid))
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
        "headcount": staff.len(),
        "requirements": requirements,
        "registrations": { "expiring": expiring, "expired": expired },
    }))
}

/// The CPD routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/cpd-requirements", post(create_requirement))
        .add("/cpd-requirements", get(list_requirements))
        .add("/workers/{pid}/cpd-entries", post(create_entry))
        .add("/workers/{pid}/cpd-entries", get(list_entries))
        .add("/workers/{pid}/cpd-progress", get(progress))
        .add("/workers/{pid}/registrations", post(create_registration))
        .add("/workers/{pid}/registrations", get(list_registrations))
        .add("/cpd-entries/{pid}/verify", post(verify_entry))
        .add("/cpd-entries/{pid}", delete(delete_entry))
        .add("/cpd/overview", get(overview))
}
