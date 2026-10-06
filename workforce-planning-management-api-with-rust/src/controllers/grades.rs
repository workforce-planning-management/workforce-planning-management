//! **Grades** (WPM-R53): a worker's current job level, and the job level and pay
//! band of a role profile.
//!
//! - **Worker** (`/api/workers/{pid}/job-level`): readable and writable by the
//!   worker themself and by HR only — seniority is career-sensitive and tracks pay
//!   closely, so a manager cannot see it by default. Edits record who made them and
//!   whether on the person's behalf, and write an audit entry that names **no
//!   level** (as for emergency contacts: the record is the sensitive thing).
//! - **Role profile** (`/api/role-profiles/{pid}/grade`): a role is not a person,
//!   so its level and pay band are ordinary reference data. Setting both links a
//!   level to a pay band *for that role* — a statement by whoever edits the role,
//!   not a derived equivalence (see [`crate::rules::grade`]).

use chrono::Utc;
use loco_rs::prelude::*;
use sea_orm::ActiveValue;
use serde::Deserialize;
use serde_json::json;

use super::{record_rejection, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::models::_entities::{role_profiles, worker_job_levels, workers};
use crate::models::audit_logs::Model as Audit;
use crate::models::records;
use crate::rules::grade as rules;
use crate::rules::job_levels as level_rules;
use crate::rules::pay_scale as scale_rules;

/// A worker, authorized for a write to their record: the worker themself or HR.
async fn writable_worker(
    ctx: &AppContext,
    caller: &MaybeAuthUser,
    pid: &str,
) -> Result<workers::Model> {
    let worker = records::find_worker(&ctx.db, records::parse_pid(pid)?).await?;
    auth::authorize_record(
        caller,
        authentication_verifier::Action::Write,
        &auth::worker_resource_attrs(&worker),
    )
    .map_err(record_rejection)?;
    Ok(worker)
}

/// A level number on a framework, as the API shows it.
fn level_view(framework: &str, number: i32) -> serde_json::Value {
    let found = level_rules::find(framework).and_then(|f| {
        let level = f
            .levels
            .iter()
            .find(|l| i32::from(l.number) == number)
            .cloned();
        level.map(|l| (f.name, l))
    });
    match found {
        Some((name, level)) => {
            json!({ "framework": framework, "framework_name": name, "level": level })
        }
        // A ladder retired from the service: say so rather than drop the fact.
        None => json!({ "framework": framework, "framework_name": null, "level": null,
                        "level_number": number }),
    }
}

/// A pay band on a scale, as the API shows it: entry and top pay, and the steps.
fn band_view(scale: &str, band: &str) -> serde_json::Value {
    let found = scale_rules::find(scale).and_then(|s| {
        let b = s.band(band).cloned();
        b.map(|b| (s.name, s.currency, b))
    });
    match found {
        Some((name, currency, b)) => json!({
            "scale": scale, "scale_name": name, "band": b.code, "closed": b.closed,
            "currency": currency,
            "entry_minor": b.steps.first().map(|s| s.annual_minor),
            "top_minor": b.steps.last().map(|s| s.annual_minor),
            "steps": b.steps,
        }),
        None => json!({ "scale": scale, "scale_name": null, "band": band }),
    }
}

// ─── A worker's level ───────────────────────────────────────────────────────

async fn current(
    ctx: &AppContext,
    worker: &workers::Model,
) -> Result<Option<worker_job_levels::Model>> {
    Ok(worker_job_levels::Entity::find()
        .filter(worker_job_levels::Column::WorkerPid.eq(worker.pid))
        .one(&ctx.db)
        .await?)
}

/// `GET /api/workers/{pid}/job-level` — the worker's level, or `null`.
#[debug_handler]
async fn get_worker_level(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = writable_worker(&ctx, &caller, &pid).await?;
    let view = current(&ctx, &worker).await?.map(|row| {
        let mut v = level_view(&row.framework, row.level_number);
        v["effective_on"] = json!(row.effective_on);
        v["on_behalf"] = json!(row.on_behalf);
        v
    });
    format::json(json!({ "job_level": view }))
}

/// `PUT` body.
#[derive(Debug, Deserialize)]
struct WorkerLevelPayload {
    framework: String,
    level: String,
    /// Held since; default today. Not in the future.
    #[serde(default)]
    effective_on: Option<chrono::NaiveDate>,
}

/// `PUT /api/workers/{pid}/job-level` — set (or change) the worker's level.
#[debug_handler]
async fn set_worker_level(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<WorkerLevelPayload>,
) -> Result<Response> {
    let worker = writable_worker(&ctx, &caller, &pid).await?;
    let (framework, number) =
        rules::resolve_level(&payload.framework, &payload.level).map_err(|e| unprocessable(&e))?;
    let today = Utc::now().date_naive();
    let effective_on = payload.effective_on.unwrap_or(today);
    rules::validate_effective_on(effective_on, today).map_err(|e| unprocessable(&e))?;
    let recorded_by = caller.actor().map(ToString::to_string);
    let on_behalf = auth::acting_for_other(&caller, &worker.person_ref);
    match current(&ctx, &worker).await? {
        Some(row) => {
            let mut active: worker_job_levels::ActiveModel = row.into();
            active.framework = ActiveValue::set(framework.to_string());
            active.level_number = ActiveValue::set(i32::from(number));
            active.effective_on = ActiveValue::set(effective_on);
            active.recorded_by = ActiveValue::set(recorded_by);
            active.on_behalf = ActiveValue::set(on_behalf);
            active.update(&ctx.db).await?;
        }
        None => {
            worker_job_levels::ActiveModel {
                worker_pid: ActiveValue::set(worker.pid),
                framework: ActiveValue::set(framework.to_string()),
                level_number: ActiveValue::set(i32::from(number)),
                effective_on: ActiveValue::set(effective_on),
                recorded_by: ActiveValue::set(recorded_by),
                on_behalf: ActiveValue::set(on_behalf),
                ..Default::default()
            }
            .insert(&ctx.db)
            .await?;
        }
    }
    // No level in the audit entry: the level is the sensitive thing.
    Audit::record(
        &ctx.db,
        "worker",
        worker.pid,
        "job_level_set",
        caller.actor(),
        None,
    )
    .await?;
    let mut view = level_view(framework, i32::from(number));
    view["effective_on"] = json!(effective_on);
    view["on_behalf"] = json!(on_behalf);
    format::json(json!({ "job_level": view }))
}

/// `DELETE /api/workers/{pid}/job-level` — clear it (a level never recorded is a 404).
#[debug_handler]
async fn clear_worker_level(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = writable_worker(&ctx, &caller, &pid).await?;
    let row = current(&ctx, &worker).await?.ok_or(Error::NotFound)?;
    worker_job_levels::Entity::delete_by_id(row.id)
        .exec(&ctx.db)
        .await?;
    Audit::record(
        &ctx.db,
        "worker",
        worker.pid,
        "job_level_cleared",
        caller.actor(),
        None,
    )
    .await?;
    format::empty()
}

// ─── A role profile's grade ─────────────────────────────────────────────────

async fn find_profile(ctx: &AppContext, pid: &str) -> Result<role_profiles::Model> {
    role_profiles::Entity::find()
        .filter(role_profiles::Column::Pid.eq(records::parse_pid(pid)?))
        .filter(role_profiles::Column::DeletedAt.is_null())
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)
}

fn grade_view(profile: &role_profiles::Model) -> serde_json::Value {
    let level = match (&profile.job_level_framework, profile.job_level) {
        (Some(f), Some(n)) => Some(level_view(f, n)),
        _ => None,
    };
    let band = match (&profile.pay_scale_id, &profile.pay_band) {
        (Some(s), Some(b)) => Some(band_view(s, b)),
        _ => None,
    };
    json!({ "role_profile": profile.pid, "job_level": level, "pay_band": band })
}

/// `GET /api/role-profiles/{pid}/grade` — the role's level and pay band (either may be `null`).
#[debug_handler]
async fn get_role_grade(
    State(ctx): State<AppContext>,
    Path(pid): Path<String>,
) -> Result<Response> {
    format::json(grade_view(&find_profile(&ctx, &pid).await?))
}

/// A level for a role.
#[derive(Debug, Deserialize)]
struct RoleLevel {
    framework: String,
    level: String,
}

/// A pay band for a role.
#[derive(Debug, Deserialize)]
struct RoleBand {
    scale: String,
    band: String,
}

/// `PUT` body: the whole grade; what is absent is cleared.
#[derive(Debug, Deserialize)]
struct RoleGradePayload {
    #[serde(default)]
    job_level: Option<RoleLevel>,
    #[serde(default)]
    pay_band: Option<RoleBand>,
}

/// `PUT /api/role-profiles/{pid}/grade` — set the role's job level and/or pay band.
/// Replaces the grade: what is absent is cleared. Both absent is a 422 (use `DELETE`).
#[debug_handler]
async fn set_role_grade(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<RoleGradePayload>,
) -> Result<Response> {
    if payload.job_level.is_none() && payload.pay_band.is_none() {
        return Err(unprocessable(
            "give job_level, pay_band or both (DELETE clears)",
        ));
    }
    let profile = find_profile(&ctx, &pid).await?;
    let level = payload
        .job_level
        .as_ref()
        .map(|l| rules::resolve_level(&l.framework, &l.level))
        .transpose()
        .map_err(|e| unprocessable(&e))?;
    let band = payload
        .pay_band
        .as_ref()
        .map(|b| rules::resolve_band(&b.scale, &b.band))
        .transpose()
        .map_err(|e| unprocessable(&e))?;
    let mut active: role_profiles::ActiveModel = profile.into();
    active.job_level_framework = ActiveValue::set(level.map(|(f, _)| f.to_string()));
    active.job_level = ActiveValue::set(level.map(|(_, n)| i32::from(n)));
    active.pay_scale_id = ActiveValue::set(band.map(|(s, _)| s.to_string()));
    active.pay_band = ActiveValue::set(band.map(|(_, b)| b.to_string()));
    let saved = active.update(&ctx.db).await?;
    Audit::record(
        &ctx.db,
        "role_profile",
        saved.pid,
        "grade_set",
        caller.actor(),
        Some(json!({ "job_level": level.map(|(f, n)| format!("{f}:{n}")),
                     "pay_band": band.map(|(s, b)| format!("{s}:{b}")) })),
    )
    .await?;
    format::json(grade_view(&saved))
}

/// `DELETE /api/role-profiles/{pid}/grade` — clear both.
#[debug_handler]
async fn clear_role_grade(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let profile = find_profile(&ctx, &pid).await?;
    let mut active: role_profiles::ActiveModel = profile.into();
    active.job_level_framework = ActiveValue::set(None);
    active.job_level = ActiveValue::set(None);
    active.pay_scale_id = ActiveValue::set(None);
    active.pay_band = ActiveValue::set(None);
    let saved = active.update(&ctx.db).await?;
    Audit::record(
        &ctx.db,
        "role_profile",
        saved.pid,
        "grade_cleared",
        caller.actor(),
        None,
    )
    .await?;
    format::empty()
}

/// The grade routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/workers/{pid}/job-level", get(get_worker_level))
        .add("/workers/{pid}/job-level", put(set_worker_level))
        .add("/workers/{pid}/job-level", delete(clear_worker_level))
        .add("/role-profiles/{pid}/grade", get(get_role_grade))
        .add("/role-profiles/{pid}/grade", put(set_role_grade))
        .add("/role-profiles/{pid}/grade", delete(clear_role_grade))
}
