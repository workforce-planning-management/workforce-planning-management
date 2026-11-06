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

use super::writable_worker;
use axa::prelude::*;
use chrono::{NaiveDate, Utc};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use super::unprocessable;
use crate::auth::{self, MaybeAuthUser};
use crate::db::{self, pool};
use crate::models::_entities::worker_pay_positions;
use crate::rules::pay_position as rules;
use crate::rules::pay_scale as scale_rules;

async fn current(ctx: &AppContext, worker: Uuid) -> Result<Option<worker_pay_positions::Model>> {
    Ok(sqlx::query_as!(
        worker_pay_positions::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, worker_pid, scale_id, band, step,
                  step_since, recorded_by, on_behalf
           FROM worker_pay_positions WHERE worker_pid = $1"#,
        worker,
    )
    .fetch_optional(pool(ctx))
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
    let position = current(&ctx, worker.pid)
        .await?
        .map(|row| view(&row, today));
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
    let on_behalf = auth::acting_for_other(&caller, &worker.person_ref);
    let existing = current(&ctx, worker.pid).await?;
    let mut tx = pool(&ctx).begin().await?;
    let saved = match existing {
        Some(row) => {
            sqlx::query_as!(
                worker_pay_positions::Model,
                r#"UPDATE worker_pay_positions
                   SET scale_id = $2, band = $3, step = $4, step_since = $5, recorded_by = $6, on_behalf = $7
                   WHERE id = $1
                   RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, worker_pid, scale_id, band, step,
                             step_since, recorded_by, on_behalf"#,
                row.id,
                scale,
                band,
                step,
                step_since,
                caller.actor(),
                on_behalf,
            )
            .fetch_one(&mut *tx)
            .await?
        }
        None => {
            sqlx::query_as!(
                worker_pay_positions::Model,
                r#"INSERT INTO worker_pay_positions (worker_pid, scale_id, band, step, step_since, recorded_by, on_behalf)
                   VALUES ($1, $2, $3, $4, $5, $6, $7)
                   RETURNING created_at AS "created_at: _", updated_at AS "updated_at: _", id, worker_pid, scale_id, band, step,
                             step_since, recorded_by, on_behalf"#,
                worker.pid,
                scale,
                band,
                step,
                step_since,
                caller.actor(),
                on_behalf,
            )
            .fetch_one(&mut *tx)
            .await?
        }
    };
    // No band, step or amount in the audit entry: the position is the sensitive thing.
    db::audit(
        &mut tx,
        "worker",
        worker.pid,
        "pay_position_set",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
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
    let row = current(&ctx, worker.pid).await?.ok_or(Error::NotFound)?;
    let mut tx = pool(&ctx).begin().await?;
    sqlx::query!("DELETE FROM worker_pay_positions WHERE id = $1", row.id)
        .execute(&mut *tx)
        .await?;
    db::audit(
        &mut tx,
        "worker",
        worker.pid,
        "pay_position_cleared",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
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
