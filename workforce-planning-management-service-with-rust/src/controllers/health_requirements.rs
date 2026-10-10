//! **Workplace health requirements** (WPM-R128, WPM-D75): whether a worker meets a requirement
//! their area sets, such as a required immunization. Pure rules in
//! [`crate::rules::health_requirements`].
//!
//! This is the narrow exception to "WPM holds no health data", and it is bounded:
//!
//! - **Off** until a deployer records the lawful basis they rely on
//!   (`WPM_HEALTH_REQUIREMENTS_BASIS`). Until then every route answers `404`.
//! - **A status, never a reason.** A record is one of `up_to_date`, `exempt_recorded` or `declined`,
//!   the day it was recorded and the day it falls due again. There is no column for a diagnosis, a
//!   product, a batch, or why someone is exempt.
//! - **Written only by occupational health** (a token attribute), not by HR, a manager or the worker.
//! - **Who sees what:** the worker sees their own; occupational health sees the detail (and each read
//!   is audited); a manager (their reports) and HR see only **cleared** or **not cleared** per
//!   requirement; the only aggregate is a compliance count with small groups withheld.

use chrono::Utc;
use loco_rs::controller::ErrorDetail;
use loco_rs::prelude::*;
use sea_orm::{ActiveValue, QueryOrder};
use serde::Deserialize;
use serde_json::json;

use super::{ensure_valid, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::models::_entities::{health_requirements, worker_health_records, workers};
use crate::models::audit_logs::Model as Audit;
use crate::models::{memberships, records};
use crate::need_to_know::{facts_for, has_privileged_attrs};
use crate::rules::access::Relation;
use crate::rules::equality_monitoring::{DEFAULT_FLOOR, validate_floor};
use crate::rules::health_requirements::{self as rules};
use crate::validation::Problems;

const ATTRIBUTE: &str = "occupational_health";

/// The recorded lawful basis, or `404`: the feature does not exist until one is recorded.
fn on() -> Result<String> {
    rules::gate(crate::compat::env_var("WPM_HEALTH_REQUIREMENTS_BASIS").as_deref())
        .ok_or(Error::NotFound)
}

/// The small-group floor for aggregates: `WPM_HEALTH_FLOOR`, default 10, at least 5.
fn floor() -> usize {
    match crate::compat::env_var("WPM_HEALTH_FLOOR") {
        None => DEFAULT_FLOOR,
        Some(text) => match text.trim().parse::<usize>() {
            Ok(n) if validate_floor(n).is_ok() => n,
            _ => {
                tracing::warn!("WPM_HEALTH_FLOOR is not a number of at least 5; using the default");
                DEFAULT_FLOOR
            }
        },
    }
}

/// Whether the token carries the occupational-health attribute.
fn is_occupational_health(caller: &MaybeAuthUser) -> bool {
    caller
        .claims()
        .and_then(|c| c.attrs.get(ATTRIBUTE))
        .is_some_and(|values| values.iter().any(|v| v == "true"))
}

/// Only occupational health records or reads detail. With sign-in off (development) there is no
/// identity to check.
fn ensure_occupational_health(caller: &MaybeAuthUser) -> Result<()> {
    if !auth::require_auth() || is_occupational_health(caller) {
        Ok(())
    } else {
        Err(Error::CustomError(
            axum::http::StatusCode::FORBIDDEN,
            ErrorDetail::new(
                "forbidden",
                "only occupational health records or reads health requirement detail",
            ),
        ))
    }
}

// ─── Requirements (what an area asks for) ───────────────────────────────────

fn requirement_json(r: &health_requirements::Model) -> serde_json::Value {
    json!({
        "pid": r.pid,
        "name": r.name,
        "description": r.description,
        "departments": r.departments,
        "recheck_calendar_days": r.recheck_calendar_days,
        "reminder_calendar_days": r.reminder_calendar_days,
    })
}

fn departments_of(r: &health_requirements::Model) -> Vec<String> {
    r.departments
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(ToString::to_string))
                .collect()
        })
        .unwrap_or_default()
}

async fn live_requirements(ctx: &AppContext) -> Result<Vec<health_requirements::Model>> {
    Ok(health_requirements::Entity::find()
        .filter(health_requirements::Column::DeletedAt.is_null())
        .order_by_asc(health_requirements::Column::Name)
        .all(&ctx.db)
        .await?)
}

/// `GET /api/health-requirements` — what is required, and of whom. Not personal data.
#[debug_handler]
async fn list_requirements(State(ctx): State<AppContext>) -> Result<Response> {
    on()?;
    format::json(
        live_requirements(&ctx)
            .await?
            .iter()
            .map(requirement_json)
            .collect::<Vec<_>>(),
    )
}

/// `POST /api/health-requirements` and `PUT …/{pid}` body.
#[derive(Debug, Deserialize)]
struct RequirementPayload {
    name: String,
    /// What is required, not why anyone is or is not.
    #[serde(default)]
    description: Option<String>,
    /// The departments it applies to; empty or absent means everyone.
    #[serde(default)]
    departments: Vec<String>,
    /// Calendar days before a record falls due again; absent for a one-off.
    #[serde(default)]
    recheck_calendar_days: Option<u32>,
    #[serde(default)]
    reminder_calendar_days: Option<u32>,
}

fn check_requirement(p: &RequirementPayload) -> Result<()> {
    let mut problems = Problems::new();
    problems.cap_text("name", &p.name);
    ensure_valid(&problems.into_vec())?;
    rules::validate_requirement(
        &p.name,
        p.description.as_deref(),
        &p.departments,
        p.recheck_calendar_days,
        p.reminder_calendar_days
            .unwrap_or(rules::DEFAULT_REMINDER_CALENDAR_DAYS),
    )
    .map_err(|e| unprocessable(&e))
}

/// `POST /api/health-requirements` — define a requirement (HR).
#[debug_handler]
async fn create_requirement(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<RequirementPayload>,
) -> Result<Response> {
    on()?;
    check_requirement(&payload)?;
    let exists = live_requirements(&ctx)
        .await?
        .iter()
        .any(|r| r.name.trim().eq_ignore_ascii_case(payload.name.trim()));
    if exists {
        return Err(unprocessable("a requirement with that name already exists"));
    }
    let row = health_requirements::ActiveModel {
        pid: ActiveValue::set(uuid::Uuid::new_v4()),
        name: ActiveValue::set(payload.name.trim().to_string()),
        description: ActiveValue::set(payload.description.clone().filter(|d| !d.trim().is_empty())),
        departments: ActiveValue::set(json!(payload.departments)),
        recheck_calendar_days: ActiveValue::set(
            payload
                .recheck_calendar_days
                .map(|d| i32::try_from(d).unwrap_or(i32::MAX)),
        ),
        reminder_calendar_days: ActiveValue::set(
            i32::try_from(
                payload
                    .reminder_calendar_days
                    .unwrap_or(rules::DEFAULT_REMINDER_CALENDAR_DAYS),
            )
            .unwrap_or(60),
        ),
        deleted_at: ActiveValue::set(None),
        ..Default::default()
    }
    .insert(&ctx.db)
    .await?;
    Audit::record(
        &ctx.db,
        "health_requirement",
        row.pid,
        "created",
        caller.actor(),
        None,
    )
    .await?;
    format::json(requirement_json(&row))
}

async fn find_requirement(ctx: &AppContext, pid: &str) -> Result<health_requirements::Model> {
    health_requirements::Entity::find()
        .filter(health_requirements::Column::Pid.eq(records::parse_pid(pid)?))
        .filter(health_requirements::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)
}

/// `PUT /api/health-requirements/{pid}` — change one (HR).
#[debug_handler]
async fn update_requirement(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<RequirementPayload>,
) -> Result<Response> {
    on()?;
    check_requirement(&payload)?;
    let row = find_requirement(&ctx, &pid).await?;
    let clash = live_requirements(&ctx)
        .await?
        .iter()
        .any(|r| r.pid != row.pid && r.name.trim().eq_ignore_ascii_case(payload.name.trim()));
    if clash {
        return Err(unprocessable("a requirement with that name already exists"));
    }
    let mut active: health_requirements::ActiveModel = row.into();
    active.name = ActiveValue::set(payload.name.trim().to_string());
    active.description =
        ActiveValue::set(payload.description.clone().filter(|d| !d.trim().is_empty()));
    active.departments = ActiveValue::set(json!(payload.departments));
    active.recheck_calendar_days = ActiveValue::set(
        payload
            .recheck_calendar_days
            .map(|d| i32::try_from(d).unwrap_or(i32::MAX)),
    );
    active.reminder_calendar_days = ActiveValue::set(
        i32::try_from(
            payload
                .reminder_calendar_days
                .unwrap_or(rules::DEFAULT_REMINDER_CALENDAR_DAYS),
        )
        .unwrap_or(60),
    );
    let saved = active.update(&ctx.db).await?;
    Audit::record(
        &ctx.db,
        "health_requirement",
        saved.pid,
        "updated",
        caller.actor(),
        None,
    )
    .await?;
    format::json(requirement_json(&saved))
}

/// `DELETE /api/health-requirements/{pid}` — soft delete (HR).
#[debug_handler]
async fn delete_requirement(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    on()?;
    let row = find_requirement(&ctx, &pid).await?;
    let mut active: health_requirements::ActiveModel = row.into();
    active.deleted_at = ActiveValue::set(Some(Utc::now().into()));
    let saved = active.update(&ctx.db).await?;
    Audit::record(
        &ctx.db,
        "health_requirement",
        saved.pid,
        "deleted",
        caller.actor(),
        None,
    )
    .await?;
    format::json(requirement_json(&saved))
}

// ─── Records (occupational health) ──────────────────────────────────────────

/// `PUT …/health-requirements/{req_pid}` body. A status, never a reason.
#[derive(Debug, Deserialize)]
struct RecordPayload {
    /// `up_to_date`, `exempt_recorded` or `declined`.
    status: String,
    /// The day it was recorded; default today. Not in the future.
    #[serde(default)]
    recorded_on: Option<chrono::NaiveDate>,
    /// The day it falls due again; default the re-check period from the recording, if the
    /// requirement has one.
    #[serde(default)]
    next_due: Option<chrono::NaiveDate>,
}

/// `PUT /api/workers/{pid}/health-requirements/{req_pid}` — record the worker's status against a
/// requirement. **Occupational health only.** The requirement must apply to the worker's area.
/// The audit entry says a status was recorded, never which.
#[debug_handler]
async fn record_status(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path((pid, req_pid)): Path<(String, String)>,
    Json(payload): Json<RecordPayload>,
) -> Result<Response> {
    on()?;
    ensure_occupational_health(&caller)?;
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    let requirement = find_requirement(&ctx, &req_pid).await?;
    if !rules::applies(&departments_of(&requirement), &worker.department) {
        return Err(unprocessable(
            "that requirement does not apply to this worker's area",
        ));
    }
    let today = Utc::now().date_naive();
    let recorded_on = payload.recorded_on.unwrap_or(today);
    let next_due = payload.next_due.or_else(|| {
        (payload.status == "up_to_date")
            .then(|| {
                rules::next_due_after(
                    recorded_on,
                    requirement
                        .recheck_calendar_days
                        .and_then(|d| u32::try_from(d).ok()),
                )
            })
            .flatten()
    });
    rules::validate_record(&payload.status, recorded_on, next_due, today)
        .map_err(|e| unprocessable(&e))?;
    let existing = worker_health_records::Entity::find()
        .filter(worker_health_records::Column::WorkerPid.eq(worker.pid))
        .filter(worker_health_records::Column::RequirementPid.eq(requirement.pid))
        .one(&ctx.db)
        .await?;
    let recorded_by = caller.actor().map(ToString::to_string);
    if let Some(row) = existing {
        let mut active: worker_health_records::ActiveModel = row.into();
        active.status = ActiveValue::set(payload.status.clone());
        active.recorded_on = ActiveValue::set(recorded_on);
        active.next_due = ActiveValue::set(next_due);
        active.recorded_by = ActiveValue::set(recorded_by);
        active.update(&ctx.db).await?;
    } else {
        worker_health_records::ActiveModel {
            worker_pid: ActiveValue::set(worker.pid),
            requirement_pid: ActiveValue::set(requirement.pid),
            status: ActiveValue::set(payload.status.clone()),
            recorded_on: ActiveValue::set(recorded_on),
            next_due: ActiveValue::set(next_due),
            recorded_by: ActiveValue::set(recorded_by),
            ..Default::default()
        }
        .insert(&ctx.db)
        .await?;
    }
    Audit::record(
        &ctx.db,
        "worker",
        worker.pid,
        "health_requirement_recorded",
        caller.actor(),
        None,
    )
    .await?;
    // The answer to the caller is the same yes-or-no a manager would see, not an echo of the status.
    let standing = rules::standing(
        Some(&payload.status),
        next_due,
        today,
        u32::try_from(requirement.reminder_calendar_days).unwrap_or(60),
    );
    format::json(json!({ "requirement": requirement.name, "cleared": rules::is_cleared(standing) }))
}

/// `DELETE /api/workers/{pid}/health-requirements/{req_pid}` — remove a record entered by mistake
/// (occupational health only).
#[debug_handler]
async fn remove_record(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path((pid, req_pid)): Path<(String, String)>,
) -> Result<Response> {
    on()?;
    ensure_occupational_health(&caller)?;
    let worker = records::find_worker(&ctx.db, records::parse_pid(&pid)?).await?;
    let requirement = find_requirement(&ctx, &req_pid).await?;
    let removed = worker_health_records::Entity::delete_many()
        .filter(worker_health_records::Column::WorkerPid.eq(worker.pid))
        .filter(worker_health_records::Column::RequirementPid.eq(requirement.pid))
        .exec(&ctx.db)
        .await?
        .rows_affected;
    if removed == 0 {
        return Err(Error::NotFound);
    }
    Audit::record(
        &ctx.db,
        "worker",
        worker.pid,
        "health_requirement_removed",
        caller.actor(),
        None,
    )
    .await?;
    format::empty()
}

// ─── Reading ────────────────────────────────────────────────────────────────

async fn records_for(
    ctx: &AppContext,
    worker_pid: uuid::Uuid,
) -> Result<Vec<worker_health_records::Model>> {
    Ok(worker_health_records::Entity::find()
        .filter(worker_health_records::Column::WorkerPid.eq(worker_pid))
        .all(&ctx.db)
        .await?)
}

fn standing_of(
    requirement: &health_requirements::Model,
    record: Option<&worker_health_records::Model>,
    today: chrono::NaiveDate,
) -> rules::Standing {
    rules::standing(
        record.map(|r| r.status.as_str()),
        record.and_then(|r| r.next_due),
        today,
        u32::try_from(requirement.reminder_calendar_days).unwrap_or(60),
    )
}

/// What the worker sees about themself: each requirement that applies to them with their own
/// standing and dates. Shared with `/api/me/health-requirements`.
pub(crate) async fn view_for(ctx: &AppContext, worker: &workers::Model) -> Result<Response> {
    let Ok(basis) = on() else {
        return format::json(json!({ "enabled": false }));
    };
    let today = Utc::now().date_naive();
    let held = records_for(ctx, worker.pid).await?;
    let list: Vec<serde_json::Value> = live_requirements(ctx)
        .await?
        .iter()
        .filter(|r| rules::applies(&departments_of(r), &worker.department))
        .map(|r| {
            let record = held.iter().find(|h| h.requirement_pid == r.pid);
            let standing = standing_of(r, record, today);
            json!({
                "requirement": requirement_json(r),
                "standing": standing.as_str(),
                "cleared": rules::is_cleared(standing),
                "recorded_on": record.map(|h| h.recorded_on),
                "next_due": record.and_then(|h| h.next_due),
                "reminder": matches!(standing, rules::Standing::DueSoon | rules::Standing::Overdue),
            })
        })
        .collect();
    format::json(json!({
        "enabled": true,
        "lawful_basis": basis,
        "requirements": list,
        "notice": "Only occupational health records these, and only you and occupational health see the detail. \
                   Your manager and HR are told only whether you are cleared. No reason is held.",
    }))
}

/// Query for the clearance list.
#[derive(Debug, Deserialize)]
struct ClearanceQuery {
    requirement: Option<String>,
    not_cleared: Option<bool>,
}

/// `GET /api/health-requirements/clearance?requirement=&not_cleared=` — for each worker the caller
/// manages (anywhere down the chain), or, for HR, each in their organizations: **cleared or not
/// cleared** per requirement that applies. The status is never shown.
#[debug_handler]
async fn clearance(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(query): axum::extract::Query<ClearanceQuery>,
) -> Result<Response> {
    on()?;
    let today = Utc::now().date_naive();
    let requirements: Vec<health_requirements::Model> = live_requirements(&ctx)
        .await?
        .into_iter()
        .filter(|r| {
            query
                .requirement
                .as_deref()
                .is_none_or(|p| r.pid.to_string() == p)
        })
        .collect();
    let scope = memberships::scope_organization_refs(&ctx.db, caller.claims()).await?;
    let mut out = Vec::new();
    for worker in workers::Entity::find()
        .filter(workers::Column::DeletedAt.is_null())
        .order_by_asc(workers::Column::Id)
        .all(&ctx.db)
        .await?
    {
        if !crate::rules::metrics::is_employed_on(today, worker.hired_on, worker.terminated_on)
            || scope
                .as_ref()
                .is_some_and(|refs| !refs.iter().any(|r| r == &worker.organization_ref))
        {
            continue;
        }
        if auth::require_auth() {
            let Some(claims) = caller.claims() else {
                continue;
            };
            let facts = facts_for(&ctx, claims, &worker).await?;
            if !(facts.privileged || facts.relation == Relation::Manager) {
                continue;
            }
        }
        let held = records_for(&ctx, worker.pid).await?;
        for r in requirements
            .iter()
            .filter(|r| rules::applies(&departments_of(r), &worker.department))
        {
            let cleared = rules::is_cleared(standing_of(
                r,
                held.iter().find(|h| h.requirement_pid == r.pid),
                today,
            ));
            if query.not_cleared == Some(true) && cleared {
                continue;
            }
            out.push(json!({
                "worker_pid": worker.pid, "worker_number": worker.worker_number,
                "department": worker.department, "requirement": r.name, "cleared": cleared,
            }));
        }
    }
    format::json(out)
}

/// Query for the detail.
#[derive(Debug, Deserialize)]
struct DetailQuery {
    worker: String,
}

/// `GET /api/health-requirements/records?worker=` — the detail for one worker. **Occupational
/// health only**; each read is audited.
#[debug_handler]
async fn detail(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    axum::extract::Query(query): axum::extract::Query<DetailQuery>,
) -> Result<Response> {
    on()?;
    ensure_occupational_health(&caller)?;
    let worker = records::find_worker(&ctx.db, records::parse_pid(&query.worker)?).await?;
    let today = Utc::now().date_naive();
    let held = records_for(&ctx, worker.pid).await?;
    let list: Vec<serde_json::Value> = live_requirements(&ctx)
        .await?
        .iter()
        .filter(|r| rules::applies(&departments_of(r), &worker.department))
        .map(|r| {
            let record = held.iter().find(|h| h.requirement_pid == r.pid);
            let standing = standing_of(r, record, today);
            json!({
                "requirement": r.name, "status": record.map(|h| h.status.clone()),
                "standing": standing.as_str(), "recorded_on": record.map(|h| h.recorded_on),
                "next_due": record.and_then(|h| h.next_due),
            })
        })
        .collect();
    Audit::record(
        &ctx.db,
        "worker",
        worker.pid,
        "health_record_read",
        caller.actor(),
        None,
    )
    .await?;
    format::json(list)
}

/// `GET /api/health-requirements/compliance` — per requirement and department, how many it
/// applies to and how many are cleared. Small groups are withheld. HR and occupational health.
#[debug_handler]
async fn compliance(State(ctx): State<AppContext>, caller: MaybeAuthUser) -> Result<Response> {
    on()?;
    if auth::require_auth() {
        let privileged = caller.claims().is_some_and(has_privileged_attrs);
        if !privileged && !is_occupational_health(&caller) {
            return Err(Error::CustomError(
                axum::http::StatusCode::FORBIDDEN,
                ErrorDetail::new(
                    "forbidden",
                    "compliance counts are for HR and occupational health",
                ),
            ));
        }
    }
    let today = Utc::now().date_naive();
    let requirements = live_requirements(&ctx).await?;
    let scope = memberships::scope_organization_refs(&ctx.db, caller.claims()).await?;
    let mut rows: Vec<(String, String, bool)> = Vec::new();
    for worker in workers::Entity::find()
        .filter(workers::Column::DeletedAt.is_null())
        .all(&ctx.db)
        .await?
    {
        if !crate::rules::metrics::is_employed_on(today, worker.hired_on, worker.terminated_on)
            || scope
                .as_ref()
                .is_some_and(|refs| !refs.iter().any(|r| r == &worker.organization_ref))
        {
            continue;
        }
        let held = records_for(&ctx, worker.pid).await?;
        for r in requirements
            .iter()
            .filter(|r| rules::applies(&departments_of(r), &worker.department))
        {
            let cleared = rules::is_cleared(standing_of(
                r,
                held.iter().find(|h| h.requirement_pid == r.pid),
                today,
            ));
            rows.push((r.name.clone(), worker.department.clone(), cleared));
        }
    }
    let f = floor();
    let out: Vec<serde_json::Value> = rules::compliance(&rows, f)
        .into_iter()
        .map(|c| json!({ "requirement": c.requirement, "department": c.department, "applicable": c.applicable, "cleared": c.cleared }))
        .collect();
    format::json(json!({
        "small_group_floor": f,
        "derivation": "People the requirement applies to and how many are cleared, per department. A group below the floor shows nothing.",
        "compliance": out,
    }))
}

/// The routes. The worker's own view is in [`super::me`].
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/health-requirements", get(list_requirements))
        .add("/health-requirements", post(create_requirement))
        .add("/health-requirements/clearance", get(clearance))
        .add("/health-requirements/records", get(detail))
        .add("/health-requirements/compliance", get(compliance))
        .add("/health-requirements/{pid}", put(update_requirement))
        .add("/health-requirements/{pid}", delete(delete_requirement))
        .add(
            "/workers/{pid}/health-requirements/{req_pid}",
            put(record_status),
        )
        .add(
            "/workers/{pid}/health-requirements/{req_pid}",
            delete(remove_record),
        )
}
