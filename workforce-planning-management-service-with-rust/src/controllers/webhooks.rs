//! Outbound webhooks (WPM-R151, WPM-D106): who is told when something happens, and how the telling went. Administrators only.
//!
//! A subscription holds an endpoint, the event kinds it wants and the **name** of the environment variable that holds its signing
//! secret; the secret is never stored, returned or logged. A delivery holds an event id, a kind, the URN of the record and the
//! attempts, never a body. Deliveries are queued in the transaction of the change that caused them (`streaming::emit_on`) and
//! sent by the `webhook_deliver` task. Spec: `spec/integrations.md`.

use axa::extract::{Json, Path};
use axa::prelude::*;
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use super::{ensure_valid, forbidden, row_json};
use crate::auth::MaybeAuthUser;
use crate::db::{self, pool};
use crate::models::_entities::{webhook_deliveries, webhook_subscriptions};
use crate::rules::webhook as rules;

/// Only an administrator may see or change webhooks; with sign-in off (a demo) there is no one to ask.
fn administrators_only(caller: &MaybeAuthUser) -> Result<()> {
    if !crate::auth::require_auth() {
        return Ok(());
    }
    let admin = caller
        .claims()
        .and_then(|c| c.attrs.get("access"))
        .is_some_and(|values| values.iter().any(|v| v == "admin"));
    if admin {
        Ok(())
    } else {
        Err(forbidden("webhooks are for administrators"))
    }
}

fn shown(row: &webhook_subscriptions::Model) -> Result<serde_json::Value> {
    let mut out = row_json(row)?;
    // Whether the secret is there to sign with, never what it is.
    out["secret_set"] =
        json!(crate::compat::env_var(&row.secret_ref).is_some_and(|v| !v.is_empty()));
    Ok(out)
}

/// `GET /api/webhook-subscriptions` — the subscriptions, with how many deliveries wait, were delivered or gave up.
#[debug_handler]
async fn list(State(ctx): State<AppContext>, caller: MaybeAuthUser) -> Result<Response> {
    administrators_only(&caller)?;
    let rows = sqlx::query_as!(
        webhook_subscriptions::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, name, url, kinds, secret_ref, active,
                  created_by
           FROM webhook_subscriptions ORDER BY name"#
    )
    .fetch_all(pool(&ctx))
    .await?;
    let deliveries = sqlx::query!("SELECT subscription_pid, status FROM webhook_deliveries")
        .fetch_all(pool(&ctx))
        .await?;
    let mut out = Vec::with_capacity(rows.len());
    for row in &rows {
        let count = |status: &str| {
            deliveries
                .iter()
                .filter(|d| d.subscription_pid == row.pid && d.status == status)
                .count()
        };
        let mut item = shown(row)?;
        item["deliveries"] = json!({ "pending": count("pending"), "delivered": count("delivered"), "failed": count("failed") });
        out.push(item);
    }
    format::json(json!({ "kinds": rules::kind_names(), "subscriptions": out }))
}

#[derive(Debug, Deserialize)]
struct Payload {
    name: String,
    url: String,
    kinds: Vec<String>,
    secret_ref: String,
    #[serde(default = "yes")]
    active: bool,
}

const fn yes() -> bool {
    true
}

fn problems(payload: &Payload) -> Vec<String> {
    let mut out = rules::check_subscription(&payload.name, &payload.kinds)
        .err()
        .unwrap_or_default();
    if let Err(e) = rules::check_url(&payload.url) {
        out.push(e);
    }
    if let Err(e) = rules::check_secret_ref(&payload.secret_ref) {
        out.push(e);
    }
    out
}

/// `POST /api/webhook-subscriptions` — subscribe an endpoint to event kinds.
#[debug_handler]
async fn create(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Json(payload): Json<Payload>,
) -> Result<Response> {
    administrators_only(&caller)?;
    ensure_valid(&problems(&payload))?;
    let pid = Uuid::new_v4();
    let mut tx = pool(&ctx).begin().await?;
    sqlx::query!(
        "INSERT INTO webhook_subscriptions (pid, name, url, kinds, secret_ref, active, created_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
        pid,
        payload.name.trim(),
        payload.url.trim(),
        &payload.kinds,
        payload.secret_ref,
        payload.active,
        caller.actor(),
    )
    .execute(&mut *tx)
    .await?;
    db::audit(
        &mut tx,
        "webhook_subscription",
        pid,
        "webhook_subscription_created",
        caller.actor(),
        Some(json!({ "kinds": payload.kinds.len() })),
    )
    .await?;
    tx.commit().await?;
    format::json(json!({ "pid": pid }))
}

/// `PUT /api/webhook-subscriptions/{pid}` — replace a subscription's name, endpoint, kinds, secret reference and whether it is on.
#[debug_handler]
async fn replace(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<Uuid>,
    Json(payload): Json<Payload>,
) -> Result<Response> {
    administrators_only(&caller)?;
    ensure_valid(&problems(&payload))?;
    let mut tx = pool(&ctx).begin().await?;
    let updated = sqlx::query!(
        "UPDATE webhook_subscriptions SET name = $2, url = $3, kinds = $4, secret_ref = $5, active = $6 WHERE pid = $1",
        pid,
        payload.name.trim(),
        payload.url.trim(),
        &payload.kinds,
        payload.secret_ref,
        payload.active,
    )
    .execute(&mut *tx)
    .await?;
    if updated.rows_affected() == 0 {
        return Err(Error::NotFound);
    }
    db::audit(
        &mut tx,
        "webhook_subscription",
        pid,
        "webhook_subscription_updated",
        caller.actor(),
        Some(json!({ "kinds": payload.kinds.len(), "active": payload.active })),
    )
    .await?;
    tx.commit().await?;
    format::json(json!({ "pid": pid }))
}

/// `DELETE /api/webhook-subscriptions/{pid}` — remove a subscription and its delivery log.
#[debug_handler]
async fn remove(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<Uuid>,
) -> Result<Response> {
    administrators_only(&caller)?;
    let mut tx = pool(&ctx).begin().await?;
    let removed = sqlx::query!("DELETE FROM webhook_subscriptions WHERE pid = $1", pid)
        .execute(&mut *tx)
        .await?;
    if removed.rows_affected() == 0 {
        return Err(Error::NotFound);
    }
    db::audit(
        &mut tx,
        "webhook_subscription",
        pid,
        "webhook_subscription_deleted",
        caller.actor(),
        None,
    )
    .await?;
    tx.commit().await?;
    format::json(json!({ "deleted": pid }))
}

#[derive(Debug, Deserialize)]
struct LogQuery {
    limit: Option<u64>,
}

/// `GET /api/webhook-subscriptions/{pid}/deliveries?limit=` — the delivery log, newest first: kind, URN, status and attempts.
/// There is no body to show because none is kept.
#[debug_handler]
async fn deliveries(
    State(ctx): State<AppContext>,
    caller: MaybeAuthUser,
    Path(pid): Path<Uuid>,
    axa::extract::Query(query): axa::extract::Query<LogQuery>,
) -> Result<Response> {
    administrators_only(&caller)?;
    sqlx::query_scalar!("SELECT id FROM webhook_subscriptions WHERE pid = $1", pid)
        .fetch_optional(pool(&ctx))
        .await?
        .ok_or(Error::NotFound)?;
    let rows = sqlx::query_as!(
        webhook_deliveries::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, subscription_pid, event_id, kind,
                  subject, data, occurred_at AS "occurred_at: _", status, attempts, last_status, last_error,
                  next_attempt_at AS "next_attempt_at: _", delivered_at AS "delivered_at: _"
           FROM webhook_deliveries WHERE subscription_pid = $1 ORDER BY id DESC LIMIT $2"#,
        pid,
        i64::try_from(query.limit.unwrap_or(100).clamp(1, 500)).unwrap_or(500),
    )
    .fetch_all(pool(&ctx))
    .await?;
    // The log shows how a delivery went, not what it said: the handoff `data` a hire or an ending keeps is not listed.
    let log = rows
        .iter()
        .map(|row| {
            let mut value = row_json(row)?;
            if let Some(object) = value.as_object_mut() {
                object.remove("data");
            }
            Ok(value)
        })
        .collect::<Result<Vec<_>>>()?;
    format::json(log)
}

/// The routes.
pub fn routes() -> Routes {
    Routes::new()
        .prefix("/api")
        .add("/webhook-subscriptions", get(list))
        .add("/webhook-subscriptions", post(create))
        .add("/webhook-subscriptions/{pid}", put(replace))
        .add("/webhook-subscriptions/{pid}", delete(remove))
        .add("/webhook-subscriptions/{pid}/deliveries", get(deliveries))
}
