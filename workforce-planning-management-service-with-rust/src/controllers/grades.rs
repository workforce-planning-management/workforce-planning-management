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

use super::writable_worker;
use axa::extract::{Json, Path};
use axa::prelude::*;
use chrono::Utc;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use super::unprocessable;
use crate::auth::{self, MaybeAuthUser};
use crate::db;
use crate::models::_entities::workers;
use crate::models::records;
use crate::rules::grade as rules;
use crate::rules::pay_scale as scale_rules;

/// A level number on a framework, as the API shows it.
async fn level_view(ctx: &AppContext, framework: &str, number: i32) -> Result<serde_json::Value> {
    let found = super::job_levels::find_framework(crate::db::pool(ctx), framework)
        .await?
        .and_then(|f| {
            let level = f
                .levels
                .iter()
                .find(|l| i32::from(l.number) == number)
                .cloned();
            level.map(|l| (f.name, l))
        });
    Ok(match found {
        Some((name, level)) => {
            json!({ "framework": framework, "framework_name": name, "level": level })
        }
        // A ladder retired from the service: say so rather than drop the fact.
        None => json!({ "framework": framework, "framework_name": null, "level": null,
                        "level_number": number }),
    })
}

/// A level named by a client, resolved against the coded and stored ladders: the canonical framework id and the number.
async fn resolve_level(ctx: &AppContext, framework: &str, level: &str) -> Result<(String, u8)> {
    let found = super::job_levels::find_framework(crate::db::pool(ctx), framework.trim())
        .await?
        .ok_or_else(|| {
            unprocessable(&format!(
                "unknown job-level framework {:?}",
                framework.trim()
            ))
        })?;
    let number = rules::resolve_level_in(&found, level).map_err(|e| unprocessable(&e))?;
    Ok((found.id, number))
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

/// A worker's recorded level.
struct Held {
    framework: String,
    level_number: i32,
    effective_on: chrono::NaiveDate,
    on_behalf: bool,
}

async fn current(ctx: &AppContext, worker: &workers::Model) -> Result<Option<Held>> {
    Ok(sqlx::query_as!(
        Held,
        "SELECT framework, level_number, effective_on, on_behalf FROM worker_job_levels WHERE worker_pid = $1",
        worker.pid,
    )
    .fetch_optional(db::pool(ctx))
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
    let view = match current(&ctx, &worker).await? {
        Some(row) => {
            let mut v = level_view(&ctx, &row.framework, row.level_number).await?;
            v["effective_on"] = json!(row.effective_on);
            v["on_behalf"] = json!(row.on_behalf);
            Some(v)
        }
        None => None,
    };
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
    let (framework, number) = resolve_level(&ctx, &payload.framework, &payload.level).await?;
    let today = Utc::now().date_naive();
    let effective_on = payload.effective_on.unwrap_or(today);
    rules::validate_effective_on(effective_on, today).map_err(|e| unprocessable(&e))?;
    let recorded_by = caller.actor().map(ToString::to_string);
    let on_behalf = auth::acting_for_other(&caller, &worker.person_ref);
    let mut tx = db::pool(&ctx).begin().await?;
    // One row per worker: one upsert on its unique key rather than a read and a write.
    sqlx::query!(
        "INSERT INTO worker_job_levels (worker_pid, framework, level_number, effective_on, recorded_by, on_behalf)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (worker_pid) DO UPDATE SET framework = EXCLUDED.framework, level_number = EXCLUDED.level_number,
             effective_on = EXCLUDED.effective_on, recorded_by = EXCLUDED.recorded_by, on_behalf = EXCLUDED.on_behalf",
        worker.pid,
        framework,
        i32::from(number),
        effective_on,
        recorded_by,
        on_behalf,
    )
    .execute(&mut *tx)
    .await?;
    // No level in the audit entry: the level is the sensitive thing.
    db::audit(
        &mut tx,
        "worker",
        worker.pid,
        "job_level_set",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    let mut view = level_view(&ctx, &framework, i32::from(number)).await?;
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
    let mut tx = db::pool(&ctx).begin().await?;
    let cleared = sqlx::query!(
        "DELETE FROM worker_job_levels WHERE worker_pid = $1",
        worker.pid
    )
    .execute(&mut *tx)
    .await?;
    if cleared.rows_affected() == 0 {
        return Err(Error::NotFound);
    }
    db::audit(
        &mut tx,
        "worker",
        worker.pid,
        "job_level_cleared",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::empty()
}

// ─── A role profile's grade ─────────────────────────────────────────────────

/// A role profile's grade.
struct Grade {
    pid: Uuid,
    job_level_framework: Option<String>,
    job_level: Option<i32>,
    pay_scale_id: Option<String>,
    pay_band: Option<String>,
}

async fn find_profile(ctx: &AppContext, pid: &str) -> Result<Grade> {
    sqlx::query_as!(
        Grade,
        "SELECT pid, job_level_framework, job_level, pay_scale_id, pay_band FROM role_profiles
          WHERE pid = $1 AND deleted_at IS NULL",
        records::parse_pid(pid)?,
    )
    .fetch_optional(db::pool(ctx))
    .await?
    .ok_or(Error::NotFound)
}

/// Set a role profile's grade (all `None` clears it) and return it, or a 404 when the profile is gone.
async fn write_grade(
    tx: &mut sqlx::PgConnection,
    pid: Uuid,
    framework: Option<String>,
    level: Option<i32>,
    scale: Option<String>,
    band: Option<String>,
) -> Result<Grade> {
    sqlx::query_as!(
        Grade,
        "UPDATE role_profiles SET job_level_framework = $2, job_level = $3, pay_scale_id = $4, pay_band = $5
          WHERE pid = $1 AND deleted_at IS NULL
          RETURNING pid, job_level_framework, job_level, pay_scale_id, pay_band",
        pid,
        framework,
        level,
        scale,
        band,
    )
    .fetch_optional(tx)
    .await?
    .ok_or(Error::NotFound)
}

async fn grade_view(ctx: &AppContext, profile: &Grade) -> Result<serde_json::Value> {
    let level = match (&profile.job_level_framework, profile.job_level) {
        (Some(f), Some(n)) => Some(level_view(ctx, f, n).await?),
        _ => None,
    };
    let band = match (&profile.pay_scale_id, &profile.pay_band) {
        (Some(s), Some(b)) => Some(band_view(s, b)),
        _ => None,
    };
    Ok(json!({ "role_profile": profile.pid, "job_level": level, "pay_band": band }))
}

/// `GET /api/role-profiles/{pid}/grade` — the role's level and pay band (either may be `null`).
#[debug_handler]
async fn get_role_grade(
    State(ctx): State<AppContext>,
    Path(pid): Path<String>,
) -> Result<Response> {
    format::json(grade_view(&ctx, &find_profile(&ctx, &pid).await?).await?)
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
    let level = match payload.job_level.as_ref() {
        Some(l) => Some(resolve_level(&ctx, &l.framework, &l.level).await?),
        None => None,
    };
    let band = payload
        .pay_band
        .as_ref()
        .map(|b| rules::resolve_band(&b.scale, &b.band))
        .transpose()
        .map_err(|e| unprocessable(&e))?;
    let mut tx = db::pool(&ctx).begin().await?;
    let saved = write_grade(
        &mut tx,
        profile.pid,
        level.as_ref().map(|(f, _)| f.clone()),
        level.as_ref().map(|(_, n)| i32::from(*n)),
        band.map(|(s, _)| s.to_string()),
        band.map(|(_, b)| b.to_string()),
    )
    .await?;
    db::audit(
        &mut tx,
        "role_profile",
        saved.pid,
        "grade_set",
        caller.actor(),
        Some(
            json!({ "job_level": level.as_ref().map(|(f, n)| format!("{f}:{n}")),
                     "pay_band": band.map(|(s, b)| format!("{s}:{b}")) }),
        ),
    )
    .await?;
    tx.commit().await?;
    format::json(grade_view(&ctx, &saved).await?)
}

/// `DELETE /api/role-profiles/{pid}/grade` — clear both.
#[debug_handler]
async fn clear_role_grade(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let profile = find_profile(&ctx, &pid).await?;
    let mut tx = db::pool(&ctx).begin().await?;
    let saved = write_grade(&mut tx, profile.pid, None, None, None, None).await?;
    db::audit(
        &mut tx,
        "role_profile",
        saved.pid,
        "grade_cleared",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
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
