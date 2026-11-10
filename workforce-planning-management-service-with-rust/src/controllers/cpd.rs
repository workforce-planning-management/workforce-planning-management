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
use axa::extract::{Json, Path};
use axa::prelude::*;
use serde::Deserialize;
use uuid::Uuid;

use super::{ensure_valid, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::db;
use crate::models::_entities::{cpd_entries, cpd_requirements, professional_registrations};
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
    let pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO cpd_requirements (pid, name, unit, required_hundredths, period_start, period_end, job_title)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
        pid,
        name,
        payload.unit,
        required,
        payload.period_start,
        payload.period_end,
        job_title,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "cpd_requirement",
        pid,
        "created",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(PidRef {
        pid: pid.to_string(),
    })
}

async fn find_requirement(ctx: &AppContext, pid: &str) -> Result<cpd_requirements::Model> {
    sqlx::query_as!(
        cpd_requirements::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, name, unit, required_hundredths,
                  period_start, period_end, job_title, deleted_at AS "deleted_at: _"
             FROM cpd_requirements WHERE pid = $1 AND deleted_at IS NULL"#,
        records::parse_pid(pid)?,
    )
    .fetch_optional(db::pool(ctx))
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
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "UPDATE cpd_requirements SET name = $2, required_hundredths = $3, period_start = $4, period_end = $5, job_title = $6
          WHERE id = $1",
        existing.id,
        name,
        required,
        payload.period_start,
        payload.period_end,
        job_title,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "cpd_requirement",
        existing.pid,
        "updated",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(PidRef {
        pid: existing.pid.to_string(),
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
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "UPDATE cpd_requirements SET deleted_at = now() WHERE id = $1",
        existing.id
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "cpd_requirement",
        existing.pid,
        "withdrawn",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
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
    Ok(sqlx::query_as!(
        cpd_requirements::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, name, unit, required_hundredths,
                  period_start, period_end, job_title, deleted_at AS "deleted_at: _"
             FROM cpd_requirements WHERE deleted_at IS NULL ORDER BY period_start DESC"#,
    )
    .fetch_all(db::pool(ctx))
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
    /// The skill the activity developed. Optional; a credential issued from the entry carries the skill's ESCO or SFIA reference.
    #[serde(default)]
    skill_pid: Option<Uuid>,
}

/// `POST /api/workers/{pid}/cpd-entries` — record a CPD activity.
#[debug_handler]
async fn create_entry(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<EntryPayload>,
) -> Result<Response> {
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
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
    if let Some(skill) = payload.skill_pid {
        let known = sqlx::query_scalar!(
            r#"SELECT 1 AS "one!" FROM skills WHERE pid = $1 AND deleted_at IS NULL LIMIT 1"#,
            skill
        )
        .fetch_optional(db::pool(&ctx))
        .await?;
        if known.is_none() {
            return Err(unprocessable("skill_pid is not a skill in the catalogue"));
        }
    }
    let pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO cpd_entries (pid, worker_pid, skill_pid, entry_date, activity, category, unit, amount_hundredths,
                                  evidence_note, evidence_url, source)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 'manual')",
        pid,
        worker.pid,
        payload.skill_pid,
        payload.entry_date,
        payload.activity.trim(),
        payload.category,
        payload.unit,
        amount,
        payload.evidence_note,
        payload.evidence_url,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(&mut tx, "cpd_entry", pid, "recorded", caller.actor(), None).await?;
    tx.commit().await?;
    format::json(PidRef {
        pid: pid.to_string(),
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
        "skill_pid": e.skill_pid,
    })
}

async fn worker_entries(ctx: &AppContext, worker_pid: Uuid) -> Result<Vec<cpd_entries::Model>> {
    Ok(sqlx::query_as!(
        cpd_entries::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, entry_date, activity,
                  category, unit, amount_hundredths, evidence_note, evidence_url, source, external_ref, course_ref,
                  verified_on, verified_by, skill_pid, deleted_at AS "deleted_at: _"
             FROM cpd_entries WHERE worker_pid = $1 AND deleted_at IS NULL ORDER BY entry_date DESC"#,
        worker_pid,
    )
    .fetch_all(db::pool(ctx))
    .await?)
}

/// `GET /api/workers/{pid}/cpd-entries` — a worker's ledger, newest first.
#[debug_handler]
async fn list_entries(State(ctx): State<AppContext>, Path(pid): Path<String>) -> Result<Response> {
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
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
    let mut tx = db::pool(&ctx).begin().await?;
    let row = sqlx::query_as!(
        cpd_entries::Model,
        r#"UPDATE cpd_entries SET verified_on = $2, verified_by = $3 WHERE id = $1
           RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, entry_date, activity,
                     category, unit, amount_hundredths, evidence_note, evidence_url, source, external_ref, course_ref,
                     verified_on, verified_by, skill_pid, deleted_at AS "deleted_at: _""#,
        entry.id,
        chrono::Utc::now().date_naive(),
        caller.actor(),
    )
    .fetch_one(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "cpd_entry",
        row.pid,
        "verified",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
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
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "UPDATE cpd_entries SET deleted_at = now() WHERE id = $1",
        entry.id
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "cpd_entry",
        entry.pid,
        "withdrawn",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::empty_json()
}

async fn find_entry(ctx: &AppContext, pid: &str) -> Result<cpd_entries::Model> {
    sqlx::query_as!(
        cpd_entries::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, entry_date, activity,
                  category, unit, amount_hundredths, evidence_note, evidence_url, source, external_ref, course_ref,
                  verified_on, verified_by, skill_pid, deleted_at AS "deleted_at: _"
             FROM cpd_entries WHERE pid = $1 AND deleted_at IS NULL"#,
        records::parse_pid(pid)?,
    )
    .fetch_optional(db::pool(ctx))
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
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
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
    let pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO professional_registrations (pid, worker_pid, body, reference, registered_on, expires_on)
         VALUES ($1, $2, $3, $4, $5, $6)",
        pid,
        worker.pid,
        payload.body.trim(),
        payload.reference,
        payload.registered_on,
        payload.expires_on,
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "professional_registration",
        pid,
        "recorded",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(PidRef {
        pid: pid.to_string(),
    })
}

async fn worker_registrations(
    ctx: &AppContext,
    worker_pid: Uuid,
) -> Result<Vec<professional_registrations::Model>> {
    Ok(sqlx::query_as!(
        professional_registrations::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, worker_pid, body, reference,
                  registered_on, expires_on, deleted_at AS "deleted_at: _"
             FROM professional_registrations WHERE worker_pid = $1 AND deleted_at IS NULL ORDER BY expires_on"#,
        worker_pid,
    )
    .fetch_all(db::pool(ctx))
    .await?)
}

/// `GET /api/workers/{pid}/registrations` — with each one's expiry status.
#[debug_handler]
async fn list_registrations(
    State(ctx): State<AppContext>,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
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
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
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
    let headcount = sqlx::query_scalar!(
        r#"SELECT count(*) AS "n!" FROM workers
            WHERE deleted_at IS NULL AND hired_on <= $1 AND (terminated_on IS NULL OR terminated_on > $1)"#,
        today,
    )
    .fetch_one(db::pool(&ctx))
    .await?;
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
    for expires_on in sqlx::query_scalar!(
        "SELECT r.expires_on FROM professional_registrations r
          WHERE r.deleted_at IS NULL AND r.expires_on <= $1
            AND r.worker_pid IN (SELECT w.pid FROM workers w
                                  WHERE w.deleted_at IS NULL AND w.hired_on <= $2
                                    AND (w.terminated_on IS NULL OR w.terminated_on > $2))",
        horizon,
        today,
    )
    .fetch_all(db::pool(&ctx))
    .await?
    {
        match rules::registration_status(expires_on, today, rules::DEFAULT_WARN_DAYS) {
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
    let row = sqlx::query!(
        r#"SELECT count(*) AS "applicable!",
                count(*) FILTER (WHERE recorded >= $1::bigint) AS "met!",
                count(*) FILTER (WHERE verified >= $1::bigint) AS "met_verified!"
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
           ) per_worker"#,
        r.required_hundredths,
        r.unit,
        r.period_start,
        r.period_end,
        today,
        r.job_title,
    )
    .fetch_one(db::pool(ctx))
    .await?;
    let count = |n: i64| usize::try_from(n).unwrap_or(0);
    Ok((
        count(row.applicable),
        count(row.met),
        count(row.met_verified),
    ))
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
