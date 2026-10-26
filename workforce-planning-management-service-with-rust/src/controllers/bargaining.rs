//! Recognised bodies and bargaining units (WPM-R102, WPM-D68): reference data about the **organization**. A body is a union, a works
//! council or a staff forum the organization recognises, with a contact **role**; a unit is the part of the workforce it covers, as
//! rules over department, location, employment type and job title. Who is covered is counted with a floor and never listed or stored,
//! and nothing says who belongs to a body or represents one. HR reads and writes. Spec: `spec/worker-unions.md`.

use chrono::{NaiveDate, Utc};
use loco_rs::prelude::*;
use sea_orm::{ActiveValue, QueryOrder, QuerySelect, TransactionTrait};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use super::pay_transparency::organization_problems;
use super::{ensure_org_in_scope, ensure_valid, row_json, unprocessable};
use crate::auth::MaybeAuthUser;
use crate::models::_entities::{bargaining_units, recognised_bodies, workers};
use crate::models::audit_logs::Model as Audit;
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

/// The routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/recognised-bodies", get(list))
        .add("/recognised-bodies", post(create))
        .add("/recognised-bodies/{pid}/units", get(units))
        .add("/recognised-bodies/{pid}/units", post(add_unit))
}
