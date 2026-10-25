//! `cargo loco task scim_push` — provision workers to an identity provider over SCIM 2.0 (WPM-R150, WPM-D102).
//!
//! It **reconciles** rather than replaying events: for each worker it works out the SCIM `User` that should exist today
//! ([`crate::rules::scim`]), compares its fingerprint with what was last sent, and creates or replaces the user only when it
//! differs. That makes it idempotent, self-healing after an outage and independent of the event transport (the outbox exists
//! under only one of them). A failure is recorded as a status code and retried with back-off; no body, name or contact detail
//! is kept. Managers are sent first so the Enterprise `manager` can carry the provider's own id.
//!
//! Settings: `WPM_SCIM_BASE_URL` (https, or http to this machine), `WPM_SCIM_TOKEN` (a bearer token, never logged or stored),
//! `WPM_SCIM_ORGANIZATION` (optional: only this organization's workers).

use std::collections::{BTreeMap, HashMap};

use chrono::{DateTime, Duration, NaiveDate, Utc};
use loco_rs::prelude::*;
use loco_rs::task::{TaskInfo, Vars};
use sea_orm::ActiveValue;
use serde::Serialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::models::_entities::{scim_pushes, workers};
use crate::rules::scim::{self, Employment, Step};

/// Where and how to push.
#[derive(Debug, Clone)]
pub struct Config {
    /// The SCIM service base URL, without a trailing slash.
    pub base_url: String,
    /// The bearer token.
    pub token: String,
    /// Only this organization's workers, when set.
    pub organization: Option<String>,
}

/// What one run did.
#[derive(Debug, Default, Clone, Serialize, PartialEq, Eq)]
pub struct Summary {
    /// Users created.
    pub created: usize,
    /// Users replaced.
    pub replaced: usize,
    /// Users already as wanted.
    pub unchanged: usize,
    /// Workers not sent: not yet joined, or deleted and never sent.
    pub skipped: usize,
    /// Workers waiting out a back-off.
    pub waiting: usize,
    /// Pushes that failed this run.
    pub failed: usize,
}

/// The settings from the environment, or why there are none.
///
/// # Errors
///
/// When the base URL or the token is missing or the URL is not acceptable.
pub fn config_from_env() -> std::result::Result<Config, String> {
    let base = crate::compat::env_var("WPM_SCIM_BASE_URL").ok_or("WPM_SCIM_BASE_URL is not set")?;
    let token = crate::compat::env_var("WPM_SCIM_TOKEN").ok_or("WPM_SCIM_TOKEN is not set")?;
    Ok(Config {
        base_url: scim::check_base_url(&base)?,
        token,
        organization: crate::compat::env_var("WPM_SCIM_ORGANIZATION").filter(|o| !o.trim().is_empty()),
    })
}

/// How many managers deep a worker is, so managers go first. A loop in the data stops at a fixed depth.
fn depth(pid: Uuid, manager_of: &HashMap<Uuid, Option<Uuid>>) -> usize {
    let mut at = pid;
    let mut n = 0;
    while let Some(Some(up)) = manager_of.get(&at) {
        n += 1;
        at = *up;
        if n > 64 {
            break;
        }
    }
    n
}

enum Outcome {
    Sent(String),
    Failed(String),
}

fn failure(status: reqwest::StatusCode) -> Outcome {
    Outcome::Failed(format!("HTTP {}", status.as_u16()))
}

fn scim_id_of(body: &Value) -> Option<String> {
    body.get("id").and_then(Value::as_str).map(str::to_string)
}

/// Create the user, or on a conflict find the one the provider already has for this `externalId` and replace it.
async fn create(http: &reqwest::Client, cfg: &Config, body: &Value, external_id: &str) -> Outcome {
    let url = format!("{}/Users", cfg.base_url);
    let sent = http
        .post(&url)
        .bearer_auth(&cfg.token)
        .header("content-type", "application/scim+json")
        .json(body)
        .send()
        .await;
    match sent {
        Ok(r) if r.status().is_success() => match r.json::<Value>().await.ok().as_ref().and_then(scim_id_of) {
            Some(id) => Outcome::Sent(id),
            None => Outcome::Failed("no id in the answer".to_string()),
        },
        Ok(r) if r.status() == reqwest::StatusCode::CONFLICT => {
            let found = http
                .get(format!("{url}?{}", scim::lookup_query(external_id)))
                .bearer_auth(&cfg.token)
                .send()
                .await;
            let id = match found {
                Ok(f) if f.status().is_success() => f
                    .json::<Value>()
                    .await
                    .ok()
                    .and_then(|v| scim_id_of(&v["Resources"][0])),
                _ => None,
            };
            match id {
                Some(id) => replace(http, cfg, &id, body).await,
                None => Outcome::Failed("HTTP 409".to_string()),
            }
        }
        Ok(r) => failure(r.status()),
        Err(_) => Outcome::Failed("request failed".to_string()),
    }
}

async fn replace(http: &reqwest::Client, cfg: &Config, id: &str, body: &Value) -> Outcome {
    let sent = http
        .put(format!("{}/Users/{id}", cfg.base_url))
        .bearer_auth(&cfg.token)
        .header("content-type", "application/scim+json")
        .json(body)
        .send()
        .await;
    match sent {
        Ok(r) if r.status().is_success() => Outcome::Sent(id.to_string()),
        Ok(r) => failure(r.status()),
        Err(_) => Outcome::Failed("request failed".to_string()),
    }
}

/// What to keep after a push: the provider's id, the fingerprint, when it was sent, the failures and the next try.
type Saved = (Option<String>, Option<String>, Option<DateTime<chrono::FixedOffset>>, i32, Option<DateTime<chrono::FixedOffset>>, Option<String>);

async fn save(ctx: &AppContext, worker: Uuid, held: Option<scim_pushes::Model>, saved: Saved) -> Result<scim_pushes::Model> {
    let (scim_id, body_hash, pushed_at, attempts, next_attempt_at, last_error) = saved;
    Ok(if let Some(old) = held {
        let mut active: scim_pushes::ActiveModel = old.into();
        active.scim_id = ActiveValue::set(scim_id);
        active.body_hash = ActiveValue::set(body_hash);
        active.pushed_at = ActiveValue::set(pushed_at);
        active.attempts = ActiveValue::set(attempts);
        active.next_attempt_at = ActiveValue::set(next_attempt_at);
        active.last_error = ActiveValue::set(last_error);
        active.update(&ctx.db).await?
    } else {
        scim_pushes::ActiveModel {
            pid: ActiveValue::set(Uuid::new_v4()),
            worker_pid: ActiveValue::set(worker),
            scim_id: ActiveValue::set(scim_id),
            body_hash: ActiveValue::set(body_hash),
            pushed_at: ActiveValue::set(pushed_at),
            attempts: ActiveValue::set(attempts),
            next_attempt_at: ActiveValue::set(next_attempt_at),
            last_error: ActiveValue::set(last_error),
            ..Default::default()
        }
        .insert(&ctx.db)
        .await?
    })
}

/// One reconciliation. `today` is the date employment is judged on and `now` stamps the state.
///
/// # Errors
///
/// A database error. A provider failure is recorded and counted, not returned.
pub async fn run(ctx: &AppContext, cfg: &Config, today: NaiveDate, now: DateTime<Utc>) -> Result<Summary> {
    let mut query = workers::Entity::find();
    if let Some(org) = &cfg.organization {
        query = query.filter(workers::Column::OrganizationRef.eq(org));
    }
    let everyone = query.all(&ctx.db).await?;
    let manager_of: HashMap<Uuid, Option<Uuid>> = everyone.iter().map(|w| (w.pid, w.manager_pid)).collect();
    let mut ordered: Vec<&workers::Model> = everyone.iter().collect();
    ordered.sort_by_key(|w| (depth(w.pid, &manager_of), w.id));

    let mut state: BTreeMap<Uuid, scim_pushes::Model> = scim_pushes::Entity::find()
        .all(&ctx.db)
        .await?
        .into_iter()
        .map(|p| (p.worker_pid, p))
        .collect();
    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| super::super::controllers::internal(&e.to_string()))?;

    let mut summary = Summary::default();
    for w in ordered {
        let employment = Employment {
            pid: w.pid.to_string(),
            worker_number: w.worker_number.clone(),
            display_name: w.display_name.clone(),
            job_title: w.job_title.clone(),
            department: w.department.clone(),
            hired_on: w.hired_on,
            terminated_on: w.terminated_on,
            deleted: w.deleted_at.is_some(),
        };
        let held = state.get(&w.pid).cloned();
        if held.as_ref().and_then(|h| h.next_attempt_at).is_some_and(|at| at > now) {
            summary.waiting += 1;
            continue;
        }
        let manager_scim = w
            .manager_pid
            .and_then(|m| state.get(&m))
            .and_then(|m| m.scim_id.clone());
        let body = scim::user(&employment, today, manager_scim.as_deref());
        let wanted = scim::fingerprint(&body);
        let known = held.as_ref().and_then(|h| h.scim_id.clone());
        let step = scim::step(
            scim::due(&employment, today),
            employment.deleted,
            known.is_some(),
            held.as_ref().and_then(|h| h.body_hash.as_deref()),
            &wanted,
        );
        let outcome = match (step, &known) {
            (Step::Skip, _) => {
                summary.skipped += 1;
                continue;
            }
            (Step::Unchanged, _) => {
                summary.unchanged += 1;
                continue;
            }
            (Step::Replace, Some(id)) => replace(&http, cfg, id, &body).await,
            (Step::Create | Step::Replace, _) => create(&http, cfg, &body, &scim::external_id(&employment.pid)).await,
        };
        let stamp = now.fixed_offset();
        let saved = match outcome {
            Outcome::Sent(id) => {
                if step == Step::Create {
                    summary.created += 1;
                } else {
                    summary.replaced += 1;
                }
                (Some(id), Some(wanted), Some(stamp), 0, None, None)
            }
            Outcome::Failed(why) => {
                summary.failed += 1;
                let attempts = held.as_ref().map_or(0, |h| h.attempts) + 1;
                // `attempts` already counts this failure; the first retry is one minute away (the webhook task counts the same way).
                let wait = scim::backoff_seconds(u32::try_from(attempts - 1).unwrap_or(0));
                (
                    known,
                    held.as_ref().and_then(|h| h.body_hash.clone()),
                    held.as_ref().and_then(|h| h.pushed_at),
                    attempts,
                    Some((now + Duration::seconds(wait)).fixed_offset()),
                    Some(why),
                )
            }
        };
        let row = save(ctx, w.pid, held, saved).await?;
        state.insert(w.pid, row);
    }
    Ok(summary)
}

/// The scheduled task.
pub struct ScimPush;

#[async_trait]
impl Task for ScimPush {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "scim_push".to_string(),
            detail: "Provision workers to the identity provider over SCIM 2.0 (employment facts only); prints a summary".to_string(),
        }
    }

    async fn run(&self, ctx: &AppContext, _vars: &Vars) -> Result<()> {
        let cfg = config_from_env().map_err(|e| Error::string(&e))?;
        let summary = run(ctx, &cfg, Utc::now().date_naive(), Utc::now()).await?;
        println!("{}", json!(summary));
        if summary.failed > 0 {
            return Err(Error::string(&format!("{} push(es) failed and will be retried", summary.failed)));
        }
        Ok(())
    }
}
