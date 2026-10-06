//! `cargo loco task pay_progression_reminders [days_ahead:N] [as_of:YYYY-MM-DD]`
//! — tell each person who becomes eligible for their next pay step within
//! `days_ahead` days of `as_of` (default 30 days from today).
//!
//! Run it daily. Idempotent: someone already told for an eligibility date is left
//! alone, so a re-run sends nothing twice. Only people employed on the eligibility
//! date are told. The message says that they become **eligible** on a date and
//! nothing else — no band, step or amount — and promises no move: eligibility is
//! only the circular's years on the step (WPM-D41).

use chrono::NaiveDate;
use loco_rs::prelude::*;
use loco_rs::task::{TaskInfo, Vars};

use crate::models::_entities::{notifications, worker_pay_positions, workers};
use crate::models::notifications::Model as Notification;
use crate::rules::metrics as metric_rules;
use crate::rules::pay_position as rules;
use crate::rules::pay_scale as scale_rules;

/// The pay-progression reminder task.
pub struct PayProgressionReminders;

#[async_trait]
impl Task for PayProgressionReminders {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "pay_progression_reminders".to_string(),
            detail: "Tell people who become eligible for their next pay step soon (optional \
                     days_ahead:N 0-90, as_of:YYYY-MM-DD; idempotent)"
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
                .filter(|n| (0..=90).contains(n))
                .ok_or_else(|| Error::string("days_ahead must be a number from 0 to 90"))?,
            Err(_) => 30,
        };
        let sent = send_reminders(ctx, as_of, as_of + chrono::Duration::days(days_ahead)).await?;
        tracing::info!(%as_of, days_ahead, sent, "pay-progression reminders sent");
        Ok(())
    }
}

/// Tell everyone who becomes eligible between `from` and `to` (inclusive); returns
/// how many were newly told.
///
/// # Errors
///
/// A database error.
pub async fn send_reminders(ctx: &AppContext, from: NaiveDate, to: NaiveDate) -> Result<usize> {
    let mut sent = 0;
    let positions = worker_pay_positions::Entity::find().all(&ctx.db).await?;
    for row in positions {
        let Some(scale) = scale_rules::find(&row.scale_id) else {
            continue;
        };
        let step = usize::try_from(row.step).unwrap_or(0);
        let Some(on) = scale
            .band(&row.band)
            .and_then(|b| rules::eligible_on(b, step, row.step_since))
        else {
            continue;
        };
        if on < from || on > to {
            continue;
        }
        // Only someone still employed on the day, and not erased.
        let Some(worker) = workers::Entity::find()
            .filter(workers::Column::Pid.eq(row.worker_pid))
            .filter(workers::Column::DeletedAt.is_null())
            .one(&ctx.db)
            .await?
        else {
            continue;
        };
        if !metric_rules::is_employed_on(on, worker.hired_on, worker.terminated_on) {
            continue;
        }
        let already = notifications::Entity::find()
            .filter(notifications::Column::WorkerPid.eq(row.worker_pid))
            .filter(notifications::Column::Kind.eq("pay_step_due"))
            .all(&ctx.db)
            .await?
            .iter()
            .any(|n| n.data["eligible_on"] == serde_json::json!(on));
        if already {
            continue;
        }
        Notification::push(
            &ctx.db,
            row.worker_pid,
            "pay_step_due",
            &format!("You become eligible to move up a pay step on {on}."),
            serde_json::json!({ "eligible_on": on }),
        )
        .await?;
        sent += 1;
    }
    Ok(sent)
}
