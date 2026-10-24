//! Outbound webhooks (WPM-R151, WPM-D106): who is told when something happens, and how the telling went. Administrators only.
//!
//! A subscription holds an endpoint, the event kinds it wants and the **name** of the environment variable that holds its signing
//! secret; the secret is never stored, returned or logged. A delivery holds an event id, a kind, the URN of the record and the
//! attempts, never a body. Deliveries are queued in the transaction of the change that caused them (`streaming::emit_on`) and
//! sent by the `webhook_deliver` task. Spec: `spec/integrations.md`.

use loco_rs::prelude::*;
use sea_orm::{ActiveValue, QueryOrder, QuerySelect, TransactionTrait};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use super::{ensure_valid, forbidden, row_json};
use crate::auth::MaybeAuthUser;
use crate::models::_entities::{webhook_deliveries, webhook_subscriptions};
use crate::models::audit_logs::Model as Audit;
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
    let rows = webhook_subscriptions::Entity::find()
        .order_by_asc(webhook_subscriptions::Column::Name)
        .all(&ctx.db)
        .await?;
    let deliveries = webhook_deliveries::Entity::find().all(&ctx.db).await?;
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
    let txn = ctx.db.begin().await?;
    webhook_subscriptions::ActiveModel {
        pid: ActiveValue::set(pid),
        name: ActiveValue::set(payload.name.trim().to_string()),
        url: ActiveValue::set(payload.url.trim().to_string()),
        kinds: ActiveValue::set(payload.kinds.clone()),
        secret_ref: ActiveValue::set(payload.secret_ref.clone()),
        active: ActiveValue::set(payload.active),
        created_by: ActiveValue::set(caller.actor().map(ToString::to_string)),
        ..Default::default()
    }
    .insert(&txn)
    .await?;
    Audit::record(
        &txn,
        "webhook_subscription",
        pid,
        "webhook_subscription_created",
        caller.actor(),
        Some(json!({ "kinds": payload.kinds.len() })),
    )
    .await?;
    txn.commit().await?;
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
    let txn = ctx.db.begin().await?;
    let row = webhook_subscriptions::Entity::find()
        .filter(webhook_subscriptions::Column::Pid.eq(pid))
        .one(&txn)
        .await?
        .ok_or(Error::NotFound)?;
    let mut active: webhook_subscriptions::ActiveModel = row.into();
    active.name = ActiveValue::set(payload.name.trim().to_string());
    active.url = ActiveValue::set(payload.url.trim().to_string());
    active.kinds = ActiveValue::set(payload.kinds.clone());
    active.secret_ref = ActiveValue::set(payload.secret_ref.clone());
    active.active = ActiveValue::set(payload.active);
    active.update(&txn).await?;
    Audit::record(
        &txn,
        "webhook_subscription",
        pid,
        "webhook_subscription_updated",
        caller.actor(),
        Some(json!({ "kinds": payload.kinds.len(), "active": payload.active })),
    )
    .await?;
    txn.commit().await?;
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
    let txn = ctx.db.begin().await?;
    let row = webhook_subscriptions::Entity::find()
        .filter(webhook_subscriptions::Column::Pid.eq(pid))
        .one(&txn)
        .await?
        .ok_or(Error::NotFound)?;
    row.delete(&txn).await?;
    Audit::record(
        &txn,
        "webhook_subscription",
        pid,
        "webhook_subscription_deleted",
        caller.actor(),
        None,
    )
    .await?;
    txn.commit().await?;
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
    axum::extract::Query(query): axum::extract::Query<LogQuery>,
) -> Result<Response> {
    administrators_only(&caller)?;
    webhook_subscriptions::Entity::find()
        .filter(webhook_subscriptions::Column::Pid.eq(pid))
        .one(&ctx.db)
        .await?
        .ok_or(Error::NotFound)?;
    let rows = webhook_deliveries::Entity::find()
        .filter(webhook_deliveries::Column::SubscriptionPid.eq(pid))
        .order_by_desc(webhook_deliveries::Column::Id)
        .limit(query.limit.unwrap_or(100).clamp(1, 500))
        .all(&ctx.db)
        .await?;
    format::json(rows.iter().map(row_json).collect::<Result<Vec<_>>>()?)
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
