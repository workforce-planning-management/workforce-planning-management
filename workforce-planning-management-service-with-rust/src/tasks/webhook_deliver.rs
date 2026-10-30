//! `cargo loco task webhook_deliver` — send the queued webhook deliveries (WPM-R151, WPM-D106).
//!
//! Each due delivery is turned into its payload (an id, a kind, a URN and a time, and for a hire or an ending the handoff `data`,
//! see [`crate::rules::webhook`]), signed with
//! the secret named by its subscription, and posted without following redirects. The answer decides what happens next:
//! delivered, retried after a back-off, or given up on. The log keeps the status code and a short reason, never a body.

use chrono::{DateTime, Duration, Utc};
use loco_rs::prelude::*;
use loco_rs::task::{TaskInfo, Vars};
use sea_orm::{ActiveValue, QueryOrder, QuerySelect};
use serde::Serialize;
use serde_json::json;

use crate::models::_entities::{webhook_deliveries, webhook_subscriptions};
use crate::rules::webhook::{self as rules, Next};

/// The most deliveries one run sends.
const BATCH: u64 = 200;

/// What one run did.
#[derive(Debug, Default, Clone, Serialize, PartialEq, Eq)]
pub struct Summary {
    /// Delivered.
    pub delivered: usize,
    /// Will be tried again.
    pub retrying: usize,
    /// Given up on.
    pub failed: usize,
}

/// Send what is due at `now`.
///
/// # Errors
///
/// A database error. A subscriber's failure is recorded and counted, not returned.
pub async fn run(ctx: &AppContext, now: DateTime<Utc>) -> Result<Summary> {
    let due = webhook_deliveries::Entity::find()
        .filter(webhook_deliveries::Column::Status.eq("pending"))
        .filter(webhook_deliveries::Column::NextAttemptAt.lte(now))
        .order_by_asc(webhook_deliveries::Column::Id)
        .limit(BATCH)
        .all(&ctx.db)
        .await?;
    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| super::super::controllers::internal(&e.to_string()))?;
    let mut summary = Summary::default();
    for delivery in due {
        let subscription = webhook_subscriptions::Entity::find()
            .filter(webhook_subscriptions::Column::Pid.eq(delivery.subscription_pid))
            .one(&ctx.db)
            .await?;
        let attempts = delivery.attempts + 1;
        let (status, reason) = match subscription.filter(|s| s.active) {
            None => (None, Some("subscription off".to_string())),
            Some(s) => send(&http, &s, &delivery, now).await,
        };
        let next = if reason.as_deref() == Some("subscription off")
            || reason.as_deref() == Some("secret not set") && attempts >= rules::MAX_ATTEMPTS
        {
            Next::GiveUp
        } else {
            rules::after_attempt(status, attempts)
        };
        let mut row: webhook_deliveries::ActiveModel = delivery.into();
        row.attempts = ActiveValue::set(attempts);
        row.last_status = ActiveValue::set(status.map(i32::from));
        row.last_error = ActiveValue::set(match next {
            Next::Delivered => None,
            _ => reason.or_else(|| status.map(|s| format!("HTTP {s}"))),
        });
        match next {
            Next::Delivered => {
                summary.delivered += 1;
                row.status = ActiveValue::set("delivered".to_string());
                row.delivered_at = ActiveValue::set(Some(now.fixed_offset()));
                row.next_attempt_at = ActiveValue::set(None);
            }
            Next::Retry(wait) => {
                summary.retrying += 1;
                row.next_attempt_at =
                    ActiveValue::set(Some((now + Duration::seconds(wait)).fixed_offset()));
            }
            Next::GiveUp => {
                summary.failed += 1;
                row.status = ActiveValue::set("failed".to_string());
                row.next_attempt_at = ActiveValue::set(None);
            }
        }
        row.update(&ctx.db).await?;
    }
    Ok(summary)
}

/// Post one delivery. Returns the HTTP status, or `None` and a short reason when the request did not complete.
async fn send(
    http: &reqwest::Client,
    subscription: &webhook_subscriptions::Model,
    delivery: &webhook_deliveries::Model,
    now: DateTime<Utc>,
) -> (Option<u16>, Option<String>) {
    let Some(secret) = crate::compat::env_var(&subscription.secret_ref).filter(|s| !s.is_empty())
    else {
        return (None, Some("secret not set".to_string()));
    };
    let body = rules::payload(
        &delivery.event_id.to_string(),
        &delivery.kind,
        &delivery.subject,
        delivery.occurred_at.with_timezone(&Utc),
        delivery.data.as_ref(),
    )
    .to_string();
    let header = rules::signature(secret.as_bytes(), now.timestamp(), body.as_bytes());
    let sent = http
        .post(&subscription.url)
        .header("content-type", "application/json")
        .header("x-wpm-event", &delivery.kind)
        .header("x-wpm-delivery", delivery.pid.to_string())
        .header("x-wpm-signature", header)
        .body(body)
        .send()
        .await;
    match sent {
        Ok(r) => (Some(r.status().as_u16()), None),
        Err(_) => (None, Some("request failed".to_string())),
    }
}

/// The scheduled task.
pub struct WebhookDeliver;

#[async_trait]
impl Task for WebhookDeliver {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "webhook_deliver".to_string(),
            detail: "Send the queued webhook deliveries (references and kinds only, signed); prints a summary".to_string(),
        }
    }

    async fn run(&self, ctx: &AppContext, _vars: &Vars) -> Result<()> {
        let summary = run(ctx, Utc::now()).await?;
        println!("{}", json!(summary));
        Ok(())
    }
}
