//! `cargo loco task engagement_end_reminders [days_ahead:N] [as_of:YYYY-MM-DD]` — tell each
//! manager, and HR, about a fixed-term or contractor engagement that is about to end, or has ended
//! with no decision (WPM-R81).
//!
//! Run it daily. The window defaults to 60 calendar days. **Idempotent:** each worker, end date and
//! kind of reminder is recorded, so a re-run, or two overlapping schedules, tell no one twice; an
//! extension gives the engagement a new end date and so a new reminder. An engagement already past
//! its end with no recorded decision is reminded once, never silently treated as continuing.
//!
//! The notification names the worker and the date. **It carries no rate**, no supplier and no reason.
//! HR is the workers who hold an `hr_admin` membership in the worker's organization.

use chrono::NaiveDate;
use loco_rs::prelude::*;
use loco_rs::task::{TaskInfo, Vars};
use sea_orm::{ActiveValue, QueryOrder};
use serde_json::json;

use crate::models::_entities::{
    conversion_plans, engagement_decisions, engagement_reminders, organization_memberships, workers,
};
use crate::models::notifications::Model as Notification;
use crate::rules::engagement::{self as rules, Reminder};

/// The reminder task.
pub struct EngagementEndReminders;

#[async_trait]
impl Task for EngagementEndReminders {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "engagement_end_reminders".to_string(),
            detail: "Tell managers and HR about engagements ending soon or ended with no decision \
                     (optional days_ahead:N calendar days, default 60; as_of:YYYY-MM-DD; idempotent)"
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
        let window = match vars.cli_arg("days_ahead") {
            Ok(text) => text
                .parse::<i64>()
                .map_err(|_| Error::string("days_ahead must be a whole number of calendar days"))?,
            Err(_) => rules::DEFAULT_WINDOW_CALENDAR_DAYS,
        };
        rules::validate_window(window).map_err(|e| Error::string(&e))?;
        let report = send_reminders(ctx, as_of, window).await?;
        tracing::info!(%as_of, window, ending = report.ending, ended_undecided = report.ended_undecided, told = report.told, "engagement reminders sent");
        Ok(())
    }
}

/// What a run did.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Report {
    /// Engagements newly reminded as ending soon.
    pub ending: usize,
    /// Engagements newly reminded as ended with no decision.
    pub ended_undecided: usize,
    /// Notifications created (a manager and each HR person per engagement).
    pub told: usize,
}

/// Remind about every engagement that needs it on `as_of`.
///
/// # Errors
///
/// A database error.
pub async fn send_reminders(ctx: &AppContext, as_of: NaiveDate, window: i64) -> Result<Report> {
    let mut report = Report::default();
    let staff = workers::Entity::find()
        .filter(workers::Column::DeletedAt.is_null())
        .filter(workers::Column::TerminatedOn.is_null())
        .filter(workers::Column::EngagementEndsOn.is_not_null())
        .order_by_asc(workers::Column::Id)
        .all(&ctx.db)
        .await?;
    for worker in staff {
        let Some(ends_on) = worker.engagement_ends_on else {
            continue;
        };
        if matches!(worker.status.as_str(), "terminated" | "retired") {
            continue;
        }
        let Some(reminder) = rules::reminder_for(ends_on, as_of, window) else {
            continue;
        };
        // A decision already recorded for this end date settles it: nothing more to ask.
        let decided = engagement_decisions::Entity::find()
            .filter(engagement_decisions::Column::WorkerPid.eq(worker.pid))
            .filter(engagement_decisions::Column::EndsOn.eq(ends_on))
            .one(&ctx.db)
            .await?
            .is_some();
        if decided {
            continue;
        }
        let already = engagement_reminders::Entity::find()
            .filter(engagement_reminders::Column::WorkerPid.eq(worker.pid))
            .filter(engagement_reminders::Column::EndsOn.eq(ends_on))
            .filter(engagement_reminders::Column::Kind.eq(reminder.as_str()))
            .one(&ctx.db)
            .await?
            .is_some();
        if already {
            continue;
        }
        let mut recipients: Vec<uuid::Uuid> = worker.manager_pid.into_iter().collect();
        for m in organization_memberships::Entity::find()
            .filter(organization_memberships::Column::OrganizationRef.eq(&worker.organization_ref))
            .filter(organization_memberships::Column::Role.eq("hr_admin"))
            .filter(organization_memberships::Column::DeletedAt.is_null())
            .all(&ctx.db)
            .await?
        {
            if let Some(pid) = m.worker_pid
                && pid != worker.pid
                && !recipients.contains(&pid)
            {
                recipients.push(pid);
            }
        }
        let plan_note = plan_note(ctx, worker.pid).await?;
        let (kind, body) = match reminder {
            Reminder::Ending => (
                "engagement_ending",
                format!(
                    "{}'s engagement ends on {ends_on}. Decide whether to extend it, convert it or let it end.{plan_note}",
                    worker.display_name
                ),
            ),
            Reminder::EndedUndecided => (
                "engagement_ended_undecided",
                format!(
                    "{}'s engagement ended on {ends_on} and no decision is recorded.{plan_note}",
                    worker.display_name
                ),
            ),
        };
        for to in &recipients {
            Notification::push(
                &ctx.db,
                *to,
                kind,
                &body,
                json!({ "worker_pid": worker.pid, "ends_on": ends_on }),
            )
            .await?;
            report.told += 1;
        }
        engagement_reminders::ActiveModel {
            worker_pid: ActiveValue::set(worker.pid),
            ends_on: ActiveValue::set(ends_on),
            kind: ActiveValue::set(reminder.as_str().to_string()),
            reminded_on: ActiveValue::set(as_of),
            ..Default::default()
        }
        .insert(&ctx.db)
        .await?;
        match reminder {
            Reminder::Ending => report.ending += 1,
            Reminder::EndedUndecided => report.ended_undecided += 1,
        }
    }
    Ok(report)
}

/// The open conversion plan, named by its status and review date, never its reason; empty when
/// there is none.
async fn plan_note(ctx: &AppContext, worker_pid: uuid::Uuid) -> Result<String> {
    let plan = conversion_plans::Entity::find()
        .filter(conversion_plans::Column::WorkerPid.eq(worker_pid))
        .filter(conversion_plans::Column::Status.is_in(["proposed", "approved"]))
        .one(&ctx.db)
        .await?;
    Ok(plan.map_or_else(String::new, |p| {
        let review = p
            .review_on
            .map_or_else(String::new, |d| format!(", to be reviewed on {d}"));
        format!(" A conversion plan is {}{review}.", p.status)
    }))
}
