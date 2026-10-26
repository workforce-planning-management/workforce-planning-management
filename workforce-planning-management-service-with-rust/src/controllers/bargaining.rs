//! Recognised bodies and bargaining units (WPM-R102, WPM-D68): reference data about the **organization**. A body is a union, a works
//! council or a staff forum the organization recognises, with a contact **role**; a unit is the part of the workforce it covers, as
//! rules over department, location, employment type and job title. Who is covered is counted with a floor and never listed or stored,
//! and nothing says who belongs to a body or represents one. HR reads and writes. Collective agreements hang off a unit as dated, sourced,
//! versioned reference data; which agreements apply to a worker is derived from the unit rules, for HR and payroll only (WPM-R103).
//! Spec: `spec/worker-unions.md`.

use chrono::{NaiveDate, Utc};
use loco_rs::prelude::*;
use sea_orm::{ActiveValue, QueryOrder, QuerySelect, TransactionTrait};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use super::pay_transparency::organization_problems;
use super::{ensure_org_in_scope, ensure_valid, row_json, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::models::_entities::{
    bargaining_units, collective_agreements, recognised_bodies, workers,
};
use crate::models::audit_logs::Model as Audit;
use crate::models::records;
use crate::rules::agreement as agreement_rules;
use crate::rules::bargaining as rules;
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
    axum::extract::Query(query): axum::extract::Query<ListQuery>,
) -> Result<Response> {
    ensure_valid(&organization_problems(&query.organization))?;
    ensure_org_in_scope(&ctx, &caller, &query.organization).await?;
    let bodies = recognised_bodies::Entity::find()
        .filter(recognised_bodies::Column::OrganizationRef.eq(&query.organization))
        .order_by_asc(recognised_bodies::Column::Name)
        .all(&ctx.db)
        .await?;
    let mut out = Vec::new();
    for body in &bodies {
        let units = bargaining_units::Entity::find()
            .filter(bargaining_units::Column::BodyPid.eq(body.pid))
            .count(&ctx.db)
            .await?;
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
    let clash = recognised_bodies::Entity::find()
        .filter(recognised_bodies::Column::OrganizationRef.eq(&payload.organization_ref))
        .filter(recognised_bodies::Column::Name.eq(&name))
        .count(&ctx.db)
        .await?;
    if clash > 0 {
        return Err(unprocessable(
            "another body in this organization has that name",
        ));
    }
    let pid = Uuid::new_v4();
    let txn = ctx.db.begin().await?;
    recognised_bodies::ActiveModel {
        pid: ActiveValue::set(pid),
        organization_ref: ActiveValue::set(payload.organization_ref),
        name: ActiveValue::set(name),
        kind: ActiveValue::set(payload.kind.clone()),
        basis: ActiveValue::set(payload.basis),
        recognised_on: ActiveValue::set(payload.recognised_on),
        contact_role: ActiveValue::set(payload.contact_role.trim().to_string()),
        recorded_by: ActiveValue::set(caller.actor().map(ToString::to_string)),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    // The audit entry says a body was recorded and of what kind: not its name.
    Audit::record(
        &txn,
        "recognised_body",
        pid,
        "recognised_body_created",
        caller.actor(),
        Some(json!({ "kind": payload.kind })),
    )
    .await?;
    txn.commit().await?;
    format::json(json!({ "pid": pid }))
}

async fn find_body(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: Uuid,
) -> Result<recognised_bodies::Model> {
    let row = recognised_bodies::Entity::find()
        .filter(recognised_bodies::Column::Pid.eq(pid))
        .one(&ctx.db)
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
    let clash = bargaining_units::Entity::find()
        .filter(bargaining_units::Column::BodyPid.eq(body.pid))
        .filter(bargaining_units::Column::Name.eq(&name))
        .count(&ctx.db)
        .await?;
    if clash > 0 {
        return Err(unprocessable("another unit of this body has that name"));
    }
    let trimmed = |v: &Option<String>| v.as_ref().map(|s| s.trim().to_string());
    let unit_pid = Uuid::new_v4();
    let txn = ctx.db.begin().await?;
    bargaining_units::ActiveModel {
        pid: ActiveValue::set(unit_pid),
        body_pid: ActiveValue::set(body.pid),
        name: ActiveValue::set(name),
        department: ActiveValue::set(trimmed(&payload.department)),
        location: ActiveValue::set(trimmed(&payload.location)),
        employment_type: ActiveValue::set(trimmed(&payload.employment_type)),
        job_title: ActiveValue::set(trimmed(&payload.job_title)),
        recorded_by: ActiveValue::set(caller.actor().map(ToString::to_string)),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    Audit::record(
        &txn,
        "bargaining_unit",
        unit_pid,
        "bargaining_unit_created",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
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
    let rows = bargaining_units::Entity::find()
        .filter(bargaining_units::Column::BodyPid.eq(body.pid))
        .order_by_asc(bargaining_units::Column::Name)
        .all(&ctx.db)
        .await?;
    // Only the four facts a unit is defined over, for the workers in work today: nothing that names anyone is read.
    let facts: Vec<(String, Option<String>, String, String)> = workers::Entity::find()
        .select_only()
        .column(workers::Column::Department)
        .column(workers::Column::Location)
        .column(workers::Column::EmploymentType)
        .column(workers::Column::JobTitle)
        .column(workers::Column::HiredOn)
        .column(workers::Column::TerminatedOn)
        .filter(workers::Column::OrganizationRef.eq(&body.organization_ref))
        .filter(workers::Column::DeletedAt.is_null())
        .into_tuple::<(
            String,
            Option<String>,
            String,
            String,
            NaiveDate,
            Option<NaiveDate>,
        )>()
        .all(&ctx.db)
        .await?
        .into_iter()
        .filter(|(_, _, _, _, hired, ended)| is_employed_on(today, *hired, *ended))
        .map(|(department, location, employment_type, job_title, _, _)| {
            (department, location, employment_type, job_title)
        })
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

async fn find_unit(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: Uuid,
) -> Result<(bargaining_units::Model, recognised_bodies::Model)> {
    let unit = bargaining_units::Entity::find()
        .filter(bargaining_units::Column::Pid.eq(pid))
        .one(&ctx.db)
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
    let newest = collective_agreements::Entity::find()
        .filter(collective_agreements::Column::UnitPid.eq(unit.pid))
        .filter(collective_agreements::Column::Title.eq(&title))
        .order_by_desc(collective_agreements::Column::Version)
        .one(&ctx.db)
        .await?;
    ensure_valid(&agreement_rules::version_problems(
        payload.effective_from,
        newest.as_ref().map(|n| n.effective_from),
    ))?;
    let version = newest.map_or(1, |n| n.version + 1);
    let new_pid = Uuid::new_v4();
    let txn = ctx.db.begin().await?;
    collective_agreements::ActiveModel {
        pid: ActiveValue::set(new_pid),
        unit_pid: ActiveValue::set(unit.pid),
        title: ActiveValue::set(title),
        version: ActiveValue::set(version),
        effective_from: ActiveValue::set(payload.effective_from),
        effective_to: ActiveValue::set(payload.effective_to),
        source: ActiveValue::set(payload.source.trim().to_string()),
        pay_scale_id: ActiveValue::set(payload.pay_scale_id.map(|s| s.trim().to_string())),
        max_weekly_hours: ActiveValue::set(payload.max_weekly_hours),
        notice_weeks: ActiveValue::set(payload.notice_weeks),
        consultation_days: ActiveValue::set(payload.consultation_days),
        recorded_by: ActiveValue::set(caller.actor().map(ToString::to_string)),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    Audit::record(
        &txn,
        "collective_agreement",
        new_pid,
        "collective_agreement_recorded",
        caller.actor(),
        Some(json!({ "version": version })),
    )
    .await?;
    txn.commit().await?;
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
    let rows = collective_agreements::Entity::find()
        .filter(collective_agreements::Column::UnitPid.eq(unit.pid))
        .order_by_asc(collective_agreements::Column::Title)
        .order_by_asc(collective_agreements::Column::Version)
        .all(&ctx.db)
        .await?;
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
    axum::extract::Query(query): axum::extract::Query<OnQuery>,
) -> Result<Response> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    ensure_org_in_scope(&ctx, &caller, &worker.organization_ref).await?;
    let day = query.on.unwrap_or_else(|| Utc::now().date_naive());
    let facts = rules::Facts {
        department: &worker.department,
        location: worker.location.as_deref(),
        employment_type: &worker.employment_type,
        job_title: &worker.job_title,
    };
    let body_pids: Vec<Uuid> = recognised_bodies::Entity::find()
        .filter(recognised_bodies::Column::OrganizationRef.eq(&worker.organization_ref))
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|b| b.pid)
        .collect();
    let covering: Vec<Uuid> = bargaining_units::Entity::find()
        .filter(bargaining_units::Column::BodyPid.is_in(body_pids))
        .all(&ctx.db)
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
    let rows = collective_agreements::Entity::find()
        .filter(collective_agreements::Column::UnitPid.is_in(covering))
        .order_by_asc(collective_agreements::Column::Title)
        .all(&ctx.db)
        .await?;
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
    let txn = ctx.db.begin().await?;
    Audit::record(
        &txn,
        "worker",
        worker.pid,
        "collective_agreements_read",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
    format::json(json!({
        "worker_pid": worker.pid,
        "on": day,
        "derivation": "agreements in force on the day, for bargaining units whose rules cover this worker's department, location, employment type and job title. It says the terms may be set by an agreement; it says nothing about whether the worker belongs to any body.",
        "agreements": out,
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
        .add("/bargaining-units/{pid}/agreements", get(unit_agreements))
        .add("/bargaining-units/{pid}/agreements", post(add_agreement))
        .add("/workers/{pid}/collective-agreements", get(of_worker))
}
