//! `cargo run -- task qualification_expiry_reminders [days_ahead:N] [as_of:YYYY-MM-DD]` — tell a person, and their manager, that one of
//! the person's qualifications is about to expire or has just expired (WPM-R171, WPM-T435, WPM-D167).
//!
//! Run it daily. The window defaults to 60 calendar days. **Idempotent:** each qualification, expiry date and kind of reminder is
//! recorded, so a re-run, or two overlapping schedules, tell no one twice; a renewal gives the qualification a new expiry date and so a
//! new reminder. An expiry that passed more than the window ago is a past fact and is not reminded.
//!
//! The person is told "Your <title> expires on <date>"; the manager is told the person's name, the title and the date. HR is not told:
//! it can read the queue and the roll-up. Nothing else is in the notification: no level, no awarding body, no evidence reference.

use axa::prelude::*;
use axa::task::{TaskInfo, Vars};
use chrono::NaiveDate;
use sea_orm::{ActiveValue, QueryOrder};
use serde_json::json;

use crate::models::_entities::{qualification_reminders, qualifications, workers};
use crate::models::notifications::Model as Notification;
use crate::rules::qualification::{self as rules, Reminder};

/// The reminder task.
pub struct QualificationExpiryReminders;

#[async_trait]
impl Task for QualificationExpiryReminders {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "qualification_expiry_reminders".to_string(),
            detail: "Tell a person and their manager about a qualification expiring soon or just expired \
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
            Err(_) => rules::DEFAULT_REMINDER_WINDOW_DAYS,
        };
        rules::validate_reminder_window(window).map_err(|e| Error::string(&e))?;
        let report = send_reminders(ctx, as_of, window).await?;
        tracing::info!(%as_of, window, expiring = report.expiring, expired = report.expired, told = report.told, "qualification reminders sent");
        Ok(())
    }
}

/// What a run did.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Report {
    /// Qualifications newly reminded as expiring soon.
    pub expiring: usize,
    /// Qualifications newly reminded as just expired.
    pub expired: usize,
    /// Notifications created (the person, and their manager if they have one).
    pub told: usize,
}

/// Remind about every qualification that needs it on `as_of`.
///
/// # Errors
///
/// A database error.
pub async fn send_reminders(ctx: &AppContext, as_of: NaiveDate, window: i64) -> Result<Report> {
    let mut report = Report::default();
    let held = qualifications::Entity::find()
        .filter(qualifications::Column::ExpiresOn.is_not_null())
        .order_by_asc(qualifications::Column::Id)
        .all(&ctx.db)
        .await?;
    for q in held {
        let Some(expires_on) = q.expires_on else {
            continue;
        };
        let Some(reminder) = rules::reminder_for(expires_on, as_of, window) else {
            continue;
        };
        let Some(worker) = workers::Entity::find()
            .filter(workers::Column::Pid.eq(q.worker_pid))
            .filter(workers::Column::DeletedAt.is_null())
            .filter(workers::Column::TerminatedOn.is_null())
            .one(&ctx.db)
            .await?
        else {
            continue;
        };
        if matches!(worker.status.as_str(), "terminated" | "retired") {
            continue;
        }
        let already = qualification_reminders::Entity::find()
            .filter(qualification_reminders::Column::QualificationPid.eq(q.pid))
            .filter(qualification_reminders::Column::ExpiresOn.eq(expires_on))
            .filter(qualification_reminders::Column::Kind.eq(reminder.as_str()))
            .one(&ctx.db)
            .await?
            .is_some();
        if already {
            continue;
        }
        let (kind, own, theirs) = match reminder {
            Reminder::Expiring => (
                "qualification_expiring",
                format!("Your {} expires on {expires_on}.", q.title),
                format!(
                    "{}'s {} expires on {expires_on}.",
                    worker.display_name, q.title
                ),
            ),
            Reminder::Expired => (
                "qualification_expired",
                format!("Your {} expired on {expires_on}.", q.title),
                format!(
                    "{}'s {} expired on {expires_on}.",
                    worker.display_name, q.title
                ),
            ),
        };
        let data = json!({ "worker_pid": worker.pid, "expires_on": expires_on });
        Notification::push(&ctx.db, worker.pid, kind, &own, data.clone()).await?;
        report.told += 1;
        if let Some(boss) = worker.manager_pid
            && boss != worker.pid
        {
            Notification::push(&ctx.db, boss, kind, &theirs, data).await?;
            report.told += 1;
        }
        qualification_reminders::ActiveModel {
            worker_pid: ActiveValue::set(worker.pid),
            qualification_pid: ActiveValue::set(q.pid),
            expires_on: ActiveValue::set(expires_on),
            kind: ActiveValue::set(reminder.as_str().to_string()),
            reminded_on: ActiveValue::set(as_of),
            ..Default::default()
        }
        .insert(&ctx.db)
        .await?;
        match reminder {
            Reminder::Expiring => report.expiring += 1,
            Reminder::Expired => report.expired += 1,
        }
    }
    Ok(report)
}
