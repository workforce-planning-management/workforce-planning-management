//! `cargo run -- task webhook_deliver` — send the queued webhook deliveries (WPM-R151, WPM-D106).
//!
//! Each due delivery is turned into its payload (an id, a kind, a URN and a time, and for a hire or an ending the handoff `data`,
//! see [`crate::rules::webhook`]), signed with
//! the secret named by its subscription, and posted without following redirects. The answer decides what happens next:
//! delivered, retried after a back-off, or given up on. The log keeps the status code and a short reason, never a body.

use axa::prelude::*;
use axa::task::{TaskInfo, Vars};
use jiff::Timestamp;
use serde::Serialize;
use serde_json::json;

use crate::models::_entities::{webhook_deliveries, webhook_subscriptions};
use crate::rules::webhook::{self as rules, Next};
use crate::sql_time::SqlTimestamp;

/// The most deliveries one run sends.
const BATCH: i64 = 200;

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
#[allow(
    clippy::arithmetic_side_effects,
    reason = "each summary counter rises by one per delivery handled in this run, so it is at most the batch size"
)]
pub async fn run(ctx: &AppContext, now: Timestamp) -> Result<Summary> {
    let due = sqlx::query_as!(
        webhook_deliveries::Model,
        r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, subscription_pid, event_id,
                  kind, subject, data, occurred_at AS "occurred_at: _", status, attempts, last_status, last_error,
                  next_attempt_at AS "next_attempt_at: _", delivered_at AS "delivered_at: _"
             FROM webhook_deliveries WHERE status = 'pending' AND next_attempt_at <= $1 ORDER BY id LIMIT $2"#,
        SqlTimestamp(now),
        BATCH,
    )
    .fetch_all(crate::db::pool(ctx))
    .await?;
    // reqwest has no rustls provider of its own: choose `ring` before building a client, whatever ran first (idempotent).
    crate::shared_cache::install_tls_provider();
    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| super::super::controllers::internal(&e.to_string()))?;
    let mut summary = Summary::default();
    for delivery in due {
        let subscription = sqlx::query_as!(
            webhook_subscriptions::Model,
            r#"SELECT created_at AS "created_at: _", updated_at AS "updated_at: _", id, pid, name, url, kinds, secret_ref,
                      active, created_by
                 FROM webhook_subscriptions WHERE pid = $1"#,
            delivery.subscription_pid,
        )
        .fetch_optional(crate::db::pool(ctx))
        .await?;
        let attempts = next_attempt(delivery.attempts)?;
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
        let last_error = match next {
            Next::Delivered => None,
            _ => reason.or_else(|| status.map(|s| format!("HTTP {s}"))),
        };
        let (state, delivered_at, next_attempt_at) = match next {
            Next::Delivered => {
                summary.delivered += 1;
                ("delivered".to_string(), Some(now), None)
            }
            Next::Retry(wait) => {
                summary.retrying += 1;
                (
                    delivery.status.clone(),
                    delivery.delivered_at.map(SqlTimestamp::get),
                    Some(retry_time(now, wait)?),
                )
            }
            Next::GiveUp => {
                summary.failed += 1;
                (
                    "failed".to_string(),
                    delivery.delivered_at.map(SqlTimestamp::get),
                    None,
                )
            }
        };
        sqlx::query!(
            "UPDATE webhook_deliveries
                SET attempts = $2, last_status = $3, last_error = $4, status = $5, delivered_at = $6, next_attempt_at = $7
              WHERE id = $1",
            delivery.id,
            attempts,
            status.map(i32::from),
            last_error,
            state,
            delivered_at.map(SqlTimestamp),
            next_attempt_at.map(SqlTimestamp),
        )
        .execute(crate::db::pool(ctx))
        .await?;
    }
    Ok(summary)
}

/// The attempt number after `previous` attempts.
///
/// # Errors
///
/// When the count does not fit.
fn next_attempt(previous: i32) -> Result<i32> {
    previous
        .checked_add(1)
        .ok_or_else(|| Error::string("the delivery attempt count exceeds the supported range"))
}

/// The time `wait` seconds after `now`.
///
/// # Errors
///
/// When it is outside the supported calendar.
fn retry_time(now: Timestamp, wait: i64) -> Result<Timestamp> {
    now.checked_add(jiff::SignedDuration::from_secs(wait))
        .map_err(|e| Error::string(&format!("the retry time leaves the calendar: {e}")))
}

/// Post one delivery. Returns the HTTP status, or `None` and a short reason when the request did not complete.
async fn send(
    http: &reqwest::Client,
    subscription: &webhook_subscriptions::Model,
    delivery: &webhook_deliveries::Model,
    now: Timestamp,
) -> (Option<u16>, Option<String>) {
    let Some(secret) = crate::compat::env_var(&subscription.secret_ref).filter(|s| !s.is_empty())
    else {
        return (None, Some("secret not set".to_string()));
    };
    let body = rules::payload(
        &delivery.event_id.to_string(),
        &delivery.kind,
        &delivery.subject,
        delivery.occurred_at.get(),
        delivery.data.as_ref(),
    )
    .to_string();
    let header = rules::signature(secret.as_bytes(), now.as_second(), body.as_bytes());
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
        let summary = run(ctx, Timestamp::now()).await?;
        println!("{}", json!(summary));
        Ok(())
    }
}

#[cfg(test)]
mod arithmetic_tests {
    use super::*;

    #[test]
    fn attempts_and_retry_times_refuse_overflow() {
        assert_eq!(next_attempt(2).ok(), Some(3));
        assert!(next_attempt(i32::MAX).is_err());
        let now = Timestamp::UNIX_EPOCH;
        assert_eq!(
            retry_time(now, 60).ok(),
            Some(Timestamp::from_second(60).expect("second"))
        );
        assert!(retry_time(Timestamp::MAX, 1).is_err());
        assert!(retry_time(now, i64::MAX).is_err());
    }
}
