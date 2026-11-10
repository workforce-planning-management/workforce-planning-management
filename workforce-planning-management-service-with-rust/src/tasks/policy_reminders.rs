//! `cargo run -- task policy_reminders [days_ahead:N] [as_of:YYYY-MM-DD]`
//! — tell each worker who has not acknowledged the newest version in force of a policy aimed at them, once the day due is within
//! `days_ahead` calendar days of `as_of` (default 3) or has passed.
//!
//! Run it daily. Idempotent: someone already told for a version is not told again, so a re-run sends nothing twice. Only workers
//! employed on `as_of` and not erased are told. The message names the policy and the day due; it holds no policy text and tells
//! no one else. A reminder never forces anything: an overdue acknowledgement is flagged, not required (WPM-D105).

use axa::prelude::*;
use axa::task::{TaskInfo, Vars};
use chrono::NaiveDate;
use std::collections::BTreeSet;
use uuid::Uuid;

use crate::rules::metrics as metric_rules;
use crate::rules::policy as rules;

/// The policy reminder task.
pub struct PolicyReminders;

#[async_trait]
impl Task for PolicyReminders {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "policy_reminders".to_string(),
            detail: "Remind workers who have not acknowledged a policy aimed at them and whose day due is near or past \
                     (optional days_ahead:N calendar days 0-30, as_of:YYYY-MM-DD; idempotent per version)"
                .to_string(),
        }
    }

    async fn run(&self, ctx: &AppContext, vars: &Vars) -> Result<()> {
        let as_of = match vars.cli_arg("as_of") {
            Ok(text) => text
                .parse::<NaiveDate>()
                .map_err(|e| Error::string(&format!("as_of must be YYYY-MM-DD: {e}")))?,
            Err(_) => chrono::Utc::now().date_naive(),
        };
        let days_ahead = match vars.cli_arg("days_ahead") {
            Ok(text) => text
                .parse::<i64>()
                .ok()
                .filter(|n| (0..=30).contains(n))
                .ok_or_else(|| {
                    Error::string("days_ahead must be a number of calendar days from 0 to 30")
                })?,
            Err(_) => rules::REMIND_DAYS,
        };
        let sent = send_reminders(ctx, as_of, days_ahead).await?;
        tracing::info!(%as_of, days_ahead, sent, "policy reminders sent");
        Ok(())
    }
}

/// Tell every worker who is due a reminder on `as_of`; returns how many were newly told.
///
/// # Errors
///
/// A database error.
pub async fn send_reminders(ctx: &AppContext, as_of: NaiveDate, days_ahead: i64) -> Result<usize> {
    let mut sent = 0;
    // Versions already reminded, per worker: (worker, version).
    let told: BTreeSet<(Uuid, String)> =
        sqlx::query!("SELECT worker_pid, data FROM notifications WHERE kind = 'policy_due'")
            .fetch_all(crate::db::pool(ctx))
            .await?
            .into_iter()
            .filter_map(|n| {
                Some((
                    n.worker_pid,
                    n.data.get("version_pid")?.as_str()?.to_string(),
                ))
            })
            .collect();
    for policy in sqlx::query!("SELECT pid, organization_ref, name FROM policies ORDER BY id")
        .fetch_all(crate::db::pool(ctx))
        .await?
    {
        let versions = sqlx::query!(
            "SELECT pid, version, effective_on, audience_kind, audience_value, acknowledge_within_days
               FROM policy_versions WHERE policy_pid = $1 ORDER BY version",
            policy.pid,
        )
        .fetch_all(crate::db::pool(ctx))
        .await?;
        let facts: Vec<rules::Fact> = versions
            .iter()
            .map(|v| rules::Fact {
                version: v.version,
                effective_on: v.effective_on,
            })
            .collect();
        let Some(i) = rules::current(&facts, as_of) else {
            continue;
        };
        let version = &versions[i];
        let acknowledged: BTreeSet<Uuid> = sqlx::query_scalar!(
            "SELECT worker_pid FROM policy_acknowledgements WHERE version_pid = $1",
            version.pid,
        )
        .fetch_all(crate::db::pool(ctx))
        .await?
        .into_iter()
        .collect();
        let staff = sqlx::query!(
            "SELECT pid, department, job_title, hired_on, terminated_on
               FROM workers WHERE organization_ref = $1 AND deleted_at IS NULL",
            policy.organization_ref,
        )
        .fetch_all(crate::db::pool(ctx))
        .await?;
        for worker in staff {
            if !metric_rules::is_employed_on(as_of, worker.hired_on, worker.terminated_on)
                || !rules::in_audience(
                    &version.audience_kind,
                    version.audience_value.as_deref(),
                    &worker.department,
                    &worker.job_title,
                )
                || told.contains(&(worker.pid, version.pid.to_string()))
            {
                continue;
            }
            let standing = rules::standing(
                version.effective_on,
                version.acknowledge_within_days,
                acknowledged.contains(&worker.pid).then_some(as_of),
                as_of,
            );
            if !rules::remind_now(standing, days_ahead, as_of) {
                continue;
            }
            let due = match standing {
                rules::Standing::Waiting { due_on } | rules::Standing::Overdue { due_on, .. } => {
                    due_on
                }
                rules::Standing::Done { .. } => continue,
            };
            crate::db::notify(
                &mut *crate::db::pool(ctx).acquire().await?,
                worker.pid,
                "policy_due",
                &format!(
                    "Please acknowledge the policy \"{}\" by {due}.",
                    policy.name
                ),
                serde_json::json!({ "version_pid": version.pid, "due_on": due }),
            )
            .await?;
            sent += 1;
        }
    }
    Ok(sent)
}
