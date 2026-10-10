//! **Pay positions** (WPM-R54): `/api/workers/{pid}/pay-position` — which band
//! and step of a reference pay scale a worker is on, since when, what that pays,
//! and when they become eligible for the next step (see
//! [`crate::rules::pay_position`]).
//!
//! A band and step *is* a salary, so the audience is the same as the job level:
//! **the worker themself and HR only**. The audit entry (`pay_position_set`,
//! `pay_position_cleared`) names no band, step or amount. The position is exported
//! and erased with the person. Progression is *eligibility by years on the step
//! only*; nothing here says the employer will move anyone.

use chrono::{NaiveDate, Utc};
use loco_rs::prelude::*;
use sea_orm::ActiveValue;
use serde::Deserialize;
use serde_json::json;

use super::{record_rejection, unprocessable};
use crate::auth::{self, MaybeAuthUser};
use crate::models::_entities::{worker_pay_positions, workers};
use crate::models::audit_logs::Model as Audit;
use crate::models::records;
use crate::rules::pay_position as rules;
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

async fn current(
    ctx: &AppContext,
    worker: &workers::Model,
) -> Result<Option<worker_pay_positions::Model>> {
    Ok(worker_pay_positions::Entity::find()
        .filter(worker_pay_positions::Column::WorkerPid.eq(worker.pid))
        .one(&ctx.db)
        .await?)
}

/// The position as the API shows it, with progression worked out for `today`.
fn view(row: &worker_pay_positions::Model, today: NaiveDate) -> serde_json::Value {
    let scale = scale_rules::find(&row.scale_id);
    let step = usize::try_from(row.step).unwrap_or(0);
    let band = scale.as_ref().and_then(|s| s.band(&row.band));
    let standing = band.and_then(|b| rules::standing(b, step, row.step_since, today));
    json!({
        "scale": row.scale_id,
        "scale_name": scale.as_ref().map(|s| s.name),
        "band": row.band,
        "step": row.step,
        "currency": scale.as_ref().map(|s| s.currency),
        "annual_minor": scale.as_ref().and_then(|s| rules::annual_minor(s, &row.band, step)),
        "step_since": row.step_since,
        "on_behalf": row.on_behalf,
        "progression": standing,
    })
}

/// `GET /api/workers/{pid}/pay-position` — the worker's position, or `null`.
#[debug_handler]
async fn get_position(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = writable_worker(&ctx, &caller, &pid).await?;
    let today = Utc::now().date_naive();
    let position = current(&ctx, &worker).await?.map(|row| view(&row, today));
    format::json(json!({ "pay_position": position }))
}

/// `PUT` body.
#[derive(Debug, Deserialize)]
struct PositionPayload {
    scale: String,
    band: String,
    /// 1 is the entry step.
    step: usize,
    /// On this step since; default today. Not in the future.
    #[serde(default)]
    step_since: Option<NaiveDate>,
}

/// `PUT /api/workers/{pid}/pay-position` — set (or change) the worker's position.
#[debug_handler]
async fn set_position(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
    Json(payload): Json<PositionPayload>,
) -> Result<Response> {
    let worker = writable_worker(&ctx, &caller, &pid).await?;
    let (scale, band, step) = rules::validate(&payload.scale, &payload.band, payload.step)
        .map_err(|e| unprocessable(&e))?;
    let today = Utc::now().date_naive();
    let step_since = payload.step_since.unwrap_or(today);
    rules::validate_step_since(step_since, today).map_err(|e| unprocessable(&e))?;
    let step = i32::try_from(step).unwrap_or(i32::MAX);
    let recorded_by = caller.actor().map(ToString::to_string);
    let on_behalf = auth::acting_for_other(&caller, &worker.person_ref);
    let saved = match current(&ctx, &worker).await? {
        Some(row) => {
            let mut active: worker_pay_positions::ActiveModel = row.into();
            active.scale_id = ActiveValue::set(scale.to_string());
            active.band = ActiveValue::set(band.to_string());
            active.step = ActiveValue::set(step);
            active.step_since = ActiveValue::set(step_since);
            active.recorded_by = ActiveValue::set(recorded_by);
            active.on_behalf = ActiveValue::set(on_behalf);
            active.update(&ctx.db).await?
        }
        None => {
            worker_pay_positions::ActiveModel {
                worker_pid: ActiveValue::set(worker.pid),
                scale_id: ActiveValue::set(scale.to_string()),
                band: ActiveValue::set(band.to_string()),
                step: ActiveValue::set(step),
                step_since: ActiveValue::set(step_since),
                recorded_by: ActiveValue::set(recorded_by),
                on_behalf: ActiveValue::set(on_behalf),
                ..Default::default()
            }
            .insert(&ctx.db)
            .await?
        }
    };
    // No band, step or amount in the audit entry: the position is the sensitive thing.
    Audit::record(
        &ctx.db,
        "worker",
        worker.pid,
        "pay_position_set",
        caller.actor(),
        None,
    )
    .await?;
    format::json(json!({ "pay_position": view(&saved, today) }))
}

/// `DELETE /api/workers/{pid}/pay-position` — clear it (none recorded is a 404).
#[debug_handler]
async fn clear_position(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<String>,
) -> Result<Response> {
    let worker = writable_worker(&ctx, &caller, &pid).await?;
    let row = current(&ctx, &worker).await?.ok_or(Error::NotFound)?;
    worker_pay_positions::Entity::delete_by_id(row.id)
        .exec(&ctx.db)
        .await?;
    Audit::record(
        &ctx.db,
        "worker",
        worker.pid,
        "pay_position_cleared",
        caller.actor(),
        None,
    )
    .await?;
    format::empty()
}

/// The pay-position routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/workers/{pid}/pay-position", get(get_position))
        .add("/workers/{pid}/pay-position", put(set_position))
        .add("/workers/{pid}/pay-position", delete(clear_position))
}
