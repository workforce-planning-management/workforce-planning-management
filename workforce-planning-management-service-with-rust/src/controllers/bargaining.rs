//! Recognised bodies and bargaining units (WPM-R102, WPM-D68): reference data about the **organization**. A body is a union, a works
//! council or a staff forum the organization recognises, with a contact **role**; a unit is the part of the workforce it covers, as
//! rules over department, location, employment type and job title. Who is covered is counted with a floor and never listed or stored,
//! and nothing says who belongs to a body or represents one. HR reads and writes. Collective agreements hang off a unit as dated, sourced,
//! versioned reference data; which agreements apply to a worker is derived from the unit rules, for HR and payroll only (WPM-R103).
//! Spec: `spec/worker-unions.md`.

use axa::extract::{Json, Path};
use axa::prelude::*;
use chrono::{NaiveDate, Utc};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use super::pay_transparency::organization_problems;
use super::{ensure_org_in_scope, ensure_valid, row_json, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::db;
use crate::models::_entities::{bargaining_units, collective_agreements, recognised_bodies};
use crate::models::records;
use crate::rules::agreement as agreement_rules;
use crate::rules::bargaining as rules;
use crate::rules::bargaining_pack as pack_rules;
use crate::rules::decision_gate as gate_rules;
use crate::rules::equality_monitoring::DEFAULT_FLOOR;
use crate::rules::metrics::is_employed_on;

#[derive(Debug, Deserialize)]
pub(crate) struct ListQuery {
    organization: String,
}

/// `GET /api/recognised-bodies?organization=` — the bodies the organization recognises, and how many units each has. HR.
#[debug_handler]
async fn list(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axa::extract::Query(query): axa::extract::Query<ListQuery>,
) -> Result<Response> {
    ensure_valid(&organization_problems(&query.organization))?;
    ensure_org_in_scope(&ctx, &caller, &query.organization).await?;
    let bodies = sqlx::query_as!(
        recognised_bodies::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, organization_ref, name, kind,
                  basis, recognised_on, contact_role, recorded_by
             FROM recognised_bodies WHERE organization_ref = $1 ORDER BY name, id"#,
        query.organization,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let pids: Vec<Uuid> = bodies.iter().map(|b| b.pid).collect();
    let counts: std::collections::HashMap<Uuid, i64> = sqlx::query!(
        r#"SELECT body_pid, COUNT(*) AS "units!" FROM bargaining_units WHERE body_pid = ANY($1) GROUP BY body_pid"#,
        &pids,
    )
    .fetch_all(db::pool(&ctx))
    .await?
    .into_iter()
    .map(|r| (r.body_pid, r.units))
    .collect();
    let mut out = Vec::new();
    for body in &bodies {
        let units = counts.get(&body.pid).copied().unwrap_or(0);
        let mut v = row_json(body)?;
        if let Some(o) = v.as_object_mut() {
            o.remove("recorded_by");
            o.insert("units".into(), json!(units));
        }
        out.push(v);
    }
    format::json(json!({ "bodies": out }))
}

#[derive(Debug, Deserialize)]
struct BodyPayload {
    organization_ref: String,
    name: String,
    kind: String,
    basis: String,
    recognised_on: NaiveDate,
    contact_role: String,
}

/// `POST /api/recognised-bodies` — record a body. A name is unique within the organization; the contact is a role, never a person.
#[debug_handler]
async fn create(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<BodyPayload>,
) -> Result<Response> {
    ensure_valid(&organization_problems(&payload.organization_ref))?;
    ensure_org_in_scope(&ctx, &caller, &payload.organization_ref).await?;
    ensure_valid(&rules::body_problems(
        &payload.name,
        &payload.kind,
        &payload.basis,
        &payload.contact_role,
    ))?;
    let name = payload.name.trim().to_string();
    let clash = sqlx::query_scalar!(
        r#"SELECT 1 AS "one!" FROM recognised_bodies WHERE organization_ref = $1 AND name = $2 LIMIT 1"#,
        payload.organization_ref,
        name,
    )
    .fetch_optional(db::pool(&ctx))
    .await?;
    if clash.is_some() {
        return Err(unprocessable(
            "another body in this organization has that name",
        ));
    }
    let pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO recognised_bodies (pid, organization_ref, name, kind, basis, recognised_on, contact_role, recorded_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        pid,
        payload.organization_ref,
        name,
        payload.kind,
        payload.basis,
        payload.recognised_on,
        payload.contact_role.trim(),
        caller.actor(),
    )
    .execute(&mut *tx)
    .await?;
    // The audit entry says a body was recorded and of what kind: not its name.
    db::audit(
        &mut tx,
        "recognised_body",
        pid,
        "recognised_body_created",
        caller.actor(),
        Some(json!({ "kind": payload.kind })),
    )
    .await?;
    tx.commit().await?;
    format::json(json!({ "pid": pid }))
}

pub(super) async fn find_body(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: Uuid,
) -> Result<recognised_bodies::Model> {
    let row = sqlx::query_as!(
        recognised_bodies::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, organization_ref, name, kind,
                  basis, recognised_on, contact_role, recorded_by
             FROM recognised_bodies WHERE pid = $1"#,
        pid,
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)?;
    ensure_org_in_scope(ctx, caller, &row.organization_ref).await?;
    Ok(row)
}

#[derive(Debug, Deserialize)]
struct UnitPayload {
    name: String,
    department: Option<String>,
    location: Option<String>,
    employment_type: Option<String>,
    job_title: Option<String>,
}

fn rule_of(p: &UnitPayload) -> rules::UnitRule<'_> {
    rules::UnitRule {
        department: p.department.as_deref(),
        location: p.location.as_deref(),
        employment_type: p.employment_type.as_deref(),
        job_title: p.job_title.as_deref(),
    }
}

/// `POST /api/recognised-bodies/{pid}/units` — add a bargaining unit: a name and rules over department, location, employment type
/// and job title (at least one). Nothing is recorded about who is covered.
#[debug_handler]
async fn add_unit(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<Uuid>,
    Json(payload): Json<UnitPayload>,
) -> Result<Response> {
    let body = find_body(&ctx, &caller, pid).await?;
    ensure_valid(&rules::unit_problems(&payload.name, &rule_of(&payload)))?;
    let name = payload.name.trim().to_string();
    let clash = sqlx::query_scalar!(
        r#"SELECT 1 AS "one!" FROM bargaining_units WHERE body_pid = $1 AND name = $2 LIMIT 1"#,
        body.pid,
        name,
    )
    .fetch_optional(db::pool(&ctx))
    .await?;
    if clash.is_some() {
        return Err(unprocessable("another unit of this body has that name"));
    }
    let trimmed = |v: &Option<String>| v.as_ref().map(|s| s.trim().to_string());
    let unit_pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO bargaining_units (pid, body_pid, name, department, location, employment_type, job_title, recorded_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        unit_pid,
        body.pid,
        name,
        trimmed(&payload.department),
        trimmed(&payload.location),
        trimmed(&payload.employment_type),
        trimmed(&payload.job_title),
        caller.actor(),
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "bargaining_unit",
        unit_pid,
        "bargaining_unit_created",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(json!({ "pid": unit_pid }))
}

/// `GET /api/recognised-bodies/{pid}/units` — the body's units, each with its rules and how many workers in work today they cover:
/// a count from the floor, nothing under it, and never a list of who. Coverage says nothing about who belongs to the body.
#[debug_handler]
async fn units(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<Uuid>,
) -> Result<Response> {
    let body = find_body(&ctx, &caller, pid).await?;
    let today = Utc::now().date_naive();
    let rows = units_of(&ctx, &[body.pid], true).await?;
    // Only the four facts a unit is defined over, for the workers in work today: nothing that names anyone is read.
    let facts: Vec<(String, Option<String>, String, String)> = sqlx::query!(
        "SELECT department, location, employment_type, job_title, hired_on, terminated_on FROM workers
          WHERE organization_ref = $1 AND deleted_at IS NULL",
        body.organization_ref,
    )
    .fetch_all(db::pool(&ctx))
    .await?
    .into_iter()
    .filter(|w| is_employed_on(today, w.hired_on, w.terminated_on))
    .map(|w| (w.department, w.location, w.employment_type, w.job_title))
    .collect();
    let mut out = Vec::new();
    for unit in &rows {
        let rule = rules::UnitRule {
            department: unit.department.as_deref(),
            location: unit.location.as_deref(),
            employment_type: unit.employment_type.as_deref(),
            job_title: unit.job_title.as_deref(),
        };
        let covered = facts
            .iter()
            .filter(|(department, location, employment_type, job_title)| {
                rules::covers(
                    &rule,
                    &rules::Facts {
                        department,
                        location: location.as_deref(),
                        employment_type,
                        job_title,
                    },
                )
            })
            .count();
        let mut v = row_json(unit)?;
        if let Some(o) = v.as_object_mut() {
            o.remove("recorded_by");
            o.insert(
                "coverage".into(),
                json!(rules::coverage(covered, DEFAULT_FLOOR)),
            );
        }
        out.push(v);
    }
    format::json(json!({
        "body_pid": body.pid,
        "floor": DEFAULT_FLOOR,
        "derivation": "workers in work today whom the unit's rules cover, counted from the floor; under it, nothing. Who is covered is not listed or stored, and coverage says nothing about who belongs to the body.",
        "units": out,
    }))
}

/// The units of these bodies, by name when `by_name`.
async fn units_of(
    ctx: &AppContext,
    bodies: &[Uuid],
    by_name: bool,
) -> Result<Vec<bargaining_units::Model>> {
    Ok(sqlx::query_as!(
        bargaining_units::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, body_pid, name, department,
                  location, employment_type, job_title, recorded_by
             FROM bargaining_units WHERE body_pid = ANY($1)
            ORDER BY CASE WHEN $2 THEN name END, id"#,
        bodies,
        by_name,
    )
    .fetch_all(db::pool(ctx))
    .await?)
}

/// A unit's agreements, or those of several units, by title and then version.
async fn agreements_of(
    ctx: &AppContext,
    units: &[Uuid],
) -> Result<Vec<collective_agreements::Model>> {
    Ok(sqlx::query_as!(
        collective_agreements::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, unit_pid, title, version,
                  effective_from, effective_to, source, pay_scale_id, max_weekly_hours, notice_weeks, consultation_days,
                  recorded_by
             FROM collective_agreements WHERE unit_pid = ANY($1)
            ORDER BY title, version, id"#,
        units,
    )
    .fetch_all(db::pool(ctx))
    .await?)
}

async fn find_unit(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: Uuid,
) -> Result<(bargaining_units::Model, recognised_bodies::Model)> {
    let unit = sqlx::query_as!(
        bargaining_units::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, body_pid, name, department,
                  location, employment_type, job_title, recorded_by
             FROM bargaining_units WHERE pid = $1"#,
        pid,
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)?;
    let body = find_body(ctx, caller, unit.body_pid).await?;
    Ok((unit, body))
}

fn agreement_json(a: &collective_agreements::Model, in_force: bool) -> serde_json::Value {
    json!({
        "pid": a.pid, "title": a.title, "version": a.version,
        "effective_from": a.effective_from, "effective_to": a.effective_to, "in_force": in_force,
        "source": a.source,
        "terms": {
            "pay_scale_id": a.pay_scale_id, "max_weekly_hours": a.max_weekly_hours,
            "notice_weeks": a.notice_weeks, "consultation_days": a.consultation_days,
        },
    })
}

#[derive(Debug, Deserialize)]
struct AgreementPayload {
    title: String,
    source: String,
    effective_from: NaiveDate,
    effective_to: Option<NaiveDate>,
    pay_scale_id: Option<String>,
    max_weekly_hours: Option<i32>,
    notice_weeks: Option<i32>,
    consultation_days: Option<i32>,
}

/// `POST /api/bargaining-units/{pid}/agreements` — record an agreement for a unit, or a **new version** of one by giving the same title.
/// The version is the next number; it must take effect after the newest. Dated, sourced and transcribed, not interpreted.
#[debug_handler]
async fn add_agreement(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<Uuid>,
    Json(payload): Json<AgreementPayload>,
) -> Result<Response> {
    let (unit, _body) = find_unit(&ctx, &caller, pid).await?;
    let terms = agreement_rules::Terms {
        pay_scale_id: payload.pay_scale_id.as_deref(),
        max_weekly_hours: payload.max_weekly_hours,
        notice_weeks: payload.notice_weeks,
        consultation_days: payload.consultation_days,
    };
    ensure_valid(&agreement_rules::agreement_problems(
        &payload.title,
        &payload.source,
        payload.effective_from,
        payload.effective_to,
        &terms,
        |id| crate::rules::pay_scale::find(id).is_some(),
    ))?;
    let title = payload.title.trim().to_string();
    let newest = sqlx::query!(
        "SELECT version, effective_from FROM collective_agreements WHERE unit_pid = $1 AND title = $2
          ORDER BY version DESC LIMIT 1",
        unit.pid,
        title,
    )
    .fetch_optional(db::pool(&ctx))
    .await?;
    ensure_valid(&agreement_rules::version_problems(
        payload.effective_from,
        newest.as_ref().map(|n| n.effective_from),
    ))?;
    let version = newest.map_or(1, |n| n.version + 1);
    let new_pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO collective_agreements
                (pid, unit_pid, title, version, effective_from, effective_to, source, pay_scale_id, max_weekly_hours,
                 notice_weeks, consultation_days, recorded_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
        new_pid,
        unit.pid,
        title,
        version,
        payload.effective_from,
        payload.effective_to,
        payload.source.trim(),
        payload.pay_scale_id.map(|s| s.trim().to_string()),
        payload.max_weekly_hours,
        payload.notice_weeks,
        payload.consultation_days,
        caller.actor(),
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "collective_agreement",
        new_pid,
        "collective_agreement_recorded",
        caller.actor(),
        Some(json!({ "version": version })),
    )
    .await?;
    tx.commit().await?;
    format::json(json!({ "pid": new_pid, "version": version }))
}

/// `GET /api/bargaining-units/{pid}/agreements` — every version of every agreement of the unit, oldest first, saying which is in force today. HR.
#[debug_handler]
async fn unit_agreements(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<Uuid>,
) -> Result<Response> {
    let (unit, _body) = find_unit(&ctx, &caller, pid).await?;
    let today = Utc::now().date_naive();
    let rows = agreements_of(&ctx, &[unit.pid]).await?;
    let out: Vec<serde_json::Value> = rows
        .iter()
        .map(|a| {
            let same: Vec<agreement_rules::Version> = rows
                .iter()
                .filter(|o| o.title == a.title)
                .map(|o| agreement_rules::Version {
                    version: o.version,
                    from: o.effective_from,
                    to: o.effective_to,
                })
                .collect();
            agreement_json(
                a,
                agreement_rules::in_force(&same, today) == Some(a.version),
            )
        })
        .collect();
    format::json(json!({ "unit_pid": unit.pid, "agreements": out }))
}

#[derive(Debug, Deserialize)]
struct OnQuery {
    on: Option<NaiveDate>,
}

/// `GET /api/workers/{pid}/collective-agreements?on=` — the agreements in force on a day (default today) for the units whose rules cover the
/// worker. HR and payroll only, and audited. It is **derived from the worker's job facts**, so it says the worker's terms may be set by
/// an agreement and says nothing about who belongs to a body: no body, unit or membership is named.
#[debug_handler]
async fn of_worker(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    axa::extract::Query(query): axa::extract::Query<OnQuery>,
) -> Result<Response> {
    let worker = records::worker_by_pid(&ctx, records::parse_pid(&pid)?).await?;
    ensure_org_in_scope(&ctx, &caller, &worker.organization_ref).await?;
    let day = query.on.unwrap_or_else(|| Utc::now().date_naive());
    let facts = rules::Facts {
        department: &worker.department,
        location: worker.location.as_deref(),
        employment_type: &worker.employment_type,
        job_title: &worker.job_title,
    };
    let body_pids: Vec<Uuid> = sqlx::query_scalar!(
        "SELECT pid FROM recognised_bodies WHERE organization_ref = $1",
        worker.organization_ref,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let covering: Vec<Uuid> = units_of(&ctx, &body_pids, false)
        .await?
        .into_iter()
        .filter(|u| {
            rules::covers(
                &rules::UnitRule {
                    department: u.department.as_deref(),
                    location: u.location.as_deref(),
                    employment_type: u.employment_type.as_deref(),
                    job_title: u.job_title.as_deref(),
                },
                &facts,
            )
        })
        .map(|u| u.pid)
        .collect();
    let rows = agreements_of(&ctx, &covering).await?;
    let mut out = Vec::new();
    for a in &rows {
        let same: Vec<agreement_rules::Version> = rows
            .iter()
            .filter(|o| o.unit_pid == a.unit_pid && o.title == a.title)
            .map(|o| agreement_rules::Version {
                version: o.version,
                from: o.effective_from,
                to: o.effective_to,
            })
            .collect();
        if agreement_rules::in_force(&same, day) == Some(a.version) {
            let mut v = agreement_json(a, true);
            if let Some(o) = v.as_object_mut() {
                o.remove("pid");
            }
            out.push(v);
        }
    }
    let mut tx = db::pool(&ctx).begin().await?;
    db::audit(
        &mut tx,
        "worker",
        worker.pid,
        "collective_agreements_read",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(json!({
        "worker_pid": worker.pid,
        "on": day,
        "derivation": "agreements in force on the day, for bargaining units whose rules cover this worker's department, location, employment type and job title. It says the terms may be set by an agreement; it says nothing about whether the worker belongs to any body.",
        "agreements": out,
    }))
}

/// `GET /api/bargaining-units/{pid}/information-pack` — the aggregates a body is given to bargain on, for the people the unit's rules cover:
/// headcount and FTE by department and by employment type, the pay bands people sit in, starters and leavers in the last year, those on a
/// basis other than permanent, and the open vacancies. Counts from the floor; when exactly one group is withheld another is withheld too, so
/// none can be worked out; a unit under the floor gives nothing. No name, no salary, no one's band. **Reading it is the disclosure, and it
/// is logged** (who asked, which unit, when) in the same transaction. HR.
#[debug_handler]
#[allow(clippy::too_many_lines)] // facts → who is covered → bands → vacancies → the pack, linearly
async fn information_pack(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<Uuid>,
) -> Result<Response> {
    let (unit, body) = find_unit(&ctx, &caller, pid).await?;
    let today = Utc::now().date_naive();
    let rule = rules::UnitRule {
        department: unit.department.as_deref(),
        location: unit.location.as_deref(),
        employment_type: unit.employment_type.as_deref(),
        job_title: unit.job_title.as_deref(),
    };
    // The facts of every worker of the organization, nothing that names or pays them; who the rules cover is decided here and not kept.
    let everyone = sqlx::query!(
        "SELECT pid, department, location, employment_type, job_title, fte_percent, hired_on, terminated_on FROM workers
          WHERE organization_ref = $1 AND deleted_at IS NULL",
        body.organization_ref,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let covered: Vec<_> = everyone
        .into_iter()
        .filter(|w| {
            rules::covers(
                &rule,
                &rules::Facts {
                    department: &w.department,
                    location: w.location.as_deref(),
                    employment_type: &w.employment_type,
                    job_title: &w.job_title,
                },
            )
        })
        .collect();
    let pids: Vec<Uuid> = covered.iter().map(|c| c.pid).collect();
    let bands: std::collections::BTreeMap<Uuid, String> = sqlx::query!(
        "SELECT worker_pid, band FROM worker_pay_positions WHERE worker_pid = ANY($1)",
        &pids,
    )
    .fetch_all(db::pool(&ctx))
    .await?
    .into_iter()
    .map(|p| (p.worker_pid, p.band))
    .collect();
    let pack_rows: Vec<pack_rules::Row> = covered
        .into_iter()
        .map(|w| pack_rules::Row {
            band: bands.get(&w.pid).cloned(),
            department: w.department,
            employment_type: w.employment_type,
            fte_percent: w.fte_percent,
            hired_on: w.hired_on,
            terminated_on: w.terminated_on,
        })
        .collect();
    let pack = pack_rules::pack(&pack_rows, today, DEFAULT_FLOOR);
    // Vacancies are posts, not people. A requisition carries a department and a job title only, so a unit defined by anything else cannot
    // be matched to them: that is said, not shown as nought.
    let vacancies = if unit.location.is_some() || unit.employment_type.is_some() {
        json!({ "kind": "unknown", "reason": "this unit is defined by location or employment type, which a vacancy does not carry" })
    } else {
        let open = sqlx::query!(
            "SELECT department, job_title, headcount FROM requisitions
              WHERE organization_ref = $1 AND deleted_at IS NULL AND status IN ('open', 'interviewing', 'offer')",
            body.organization_ref,
        )
        .fetch_all(db::pool(&ctx))
        .await?;
        let posts: i64 = open
            .iter()
            .filter(|r| {
                unit.department
                    .as_deref()
                    .is_none_or(|d| d.trim().eq_ignore_ascii_case(r.department.trim()))
                    && unit
                        .job_title
                        .as_deref()
                        .is_none_or(|t| t.trim().eq_ignore_ascii_case(r.job_title.trim()))
            })
            .map(|r| i64::from(r.headcount))
            .sum();
        json!({ "kind": "shown", "open_posts": posts })
    };
    // Reading it is the disclosure: who asked, which unit and when, written with the answer.
    let mut tx = db::pool(&ctx).begin().await?;
    db::audit(
        &mut tx,
        "bargaining_unit",
        unit.pid,
        "information_pack_disclosed",
        caller.actor(),
        Some(json!({ "as_of": today })),
    )
    .await?;
    tx.commit().await?;
    format::json(json!({
        "unit_pid": unit.pid,
        "floor": DEFAULT_FLOOR,
        "derivation": "counts of the people the unit's rules cover, from the floor; when exactly one group is withheld another is withheld too, so none can be worked out. A unit under the floor gives nothing. No name, no salary and no one's band. Reading it is logged. It says nothing about who belongs to the body.",
        "pack": pack,
        "vacancies": vacancies,
    }))
}

#[derive(Debug, Deserialize)]
struct FacilityPayload {
    period_start: NaiveDate,
    period_end: NaiveDate,
    minutes: i32,
    budget_minutes: Option<i32>,
}

/// `POST /api/recognised-bodies/{pid}/facility-time` — record the time released for representative duties **for the body** over a period, in
/// minutes, with an optional budget. Never a person, and there is no kind of time entry for it. Periods of one body do not overlap.
#[debug_handler]
async fn add_facility_time(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<Uuid>,
    Json(payload): Json<FacilityPayload>,
) -> Result<Response> {
    let body = find_body(&ctx, &caller, pid).await?;
    let held: Vec<(NaiveDate, NaiveDate)> = sqlx::query!(
        "SELECT period_start, period_end FROM facility_time_periods WHERE body_pid = $1 ORDER BY id",
        body.pid,
    )
    .fetch_all(db::pool(&ctx))
    .await?
    .into_iter()
    .map(|p| (p.period_start, p.period_end))
    .collect();
    ensure_valid(&gate_rules::facility_problems(
        payload.period_start,
        payload.period_end,
        payload.minutes,
        payload.budget_minutes,
        &held,
    ))?;
    let new_pid = Uuid::new_v4();
    let mut tx = db::pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO facility_time_periods (pid, body_pid, period_start, period_end, minutes, budget_minutes, recorded_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
        new_pid,
        body.pid,
        payload.period_start,
        payload.period_end,
        payload.minutes,
        payload.budget_minutes,
        caller.actor(),
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "facility_time",
        new_pid,
        "facility_time_recorded",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(json!({ "pid": new_pid }))
}

/// `GET /api/recognised-bodies/{pid}/facility-time` — the periods, newest first, with the budget and whether it was exceeded (unknown when no
/// budget was agreed), and the total. Per body: it says nothing about who took the time.
#[debug_handler]
async fn facility_time(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<Uuid>,
) -> Result<Response> {
    let body = find_body(&ctx, &caller, pid).await?;
    let rows = sqlx::query!(
        "SELECT pid, period_start, period_end, minutes, budget_minutes FROM facility_time_periods
          WHERE body_pid = $1 ORDER BY period_start DESC, id DESC",
        body.pid,
    )
    .fetch_all(db::pool(&ctx))
    .await?;
    let total: i64 = rows.iter().map(|r| i64::from(r.minutes)).sum();
    format::json(json!({
        "body_pid": body.pid,
        "derivation": "minutes released for representative duties, per body per period. Not per person: no one is named and no person's time is held.",
        "periods": rows.iter().map(|r| json!({
            "pid": r.pid, "period_start": r.period_start, "period_end": r.period_end,
            "minutes": r.minutes, "hours": crate::rules::display::ratio(usize::try_from(r.minutes).unwrap_or(0), 60),
            "budget_minutes": r.budget_minutes, "over_budget": gate_rules::over_budget(r.minutes, r.budget_minutes),
        })).collect::<Vec<_>>(),
        "total_minutes": total,
    }))
}

/// The routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/recognised-bodies", get(list))
        .add("/recognised-bodies", post(create))
        .add("/recognised-bodies/{pid}/units", get(units))
        .add("/recognised-bodies/{pid}/units", post(add_unit))
        .add("/recognised-bodies/{pid}/facility-time", get(facility_time))
        .add(
            "/recognised-bodies/{pid}/facility-time",
            post(add_facility_time),
        )
        .add("/bargaining-units/{pid}/agreements", get(unit_agreements))
        .add("/bargaining-units/{pid}/agreements", post(add_agreement))
        .add(
            "/bargaining-units/{pid}/information-pack",
            get(information_pack),
        )
        .add("/workers/{pid}/collective-agreements", get(of_worker))
}
