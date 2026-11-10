//! `cargo run -- task engagement_end_reminders [days_ahead:N] [as_of:YYYY-MM-DD]` — tell each
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

use axa::prelude::*;
use axa::task::{TaskInfo, Vars};
use chrono::NaiveDate;
use serde_json::json;

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
    let staff = sqlx::query!(
        "SELECT pid, status, display_name, organization_ref, manager_pid, engagement_ends_on FROM workers
          WHERE deleted_at IS NULL AND terminated_on IS NULL AND engagement_ends_on IS NOT NULL ORDER BY id"
    )
    .fetch_all(crate::db::pool(ctx))
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
        let decided = sqlx::query_scalar!(
            r#"SELECT EXISTS (SELECT 1 FROM engagement_decisions WHERE worker_pid = $1 AND ends_on = $2) AS "e!""#,
            worker.pid,
            ends_on,
        )
        .fetch_one(crate::db::pool(ctx))
        .await?;
        if decided {
            continue;
        }
        let already = sqlx::query_scalar!(
            r#"SELECT EXISTS (SELECT 1 FROM engagement_reminders
                               WHERE worker_pid = $1 AND ends_on = $2 AND kind = $3) AS "e!""#,
            worker.pid,
            ends_on,
            reminder.as_str(),
        )
        .fetch_one(crate::db::pool(ctx))
        .await?;
        if already {
            continue;
        }
        let recipients = recipients(
            ctx,
            worker.pid,
            worker.manager_pid,
            &worker.organization_ref,
        )
        .await?;
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
        let mut tx = crate::db::pool(ctx).begin().await?;
        for to in &recipients {
            crate::db::notify(
                &mut tx,
                *to,
                kind,
                &body,
                json!({ "worker_pid": worker.pid, "ends_on": ends_on }),
            )
            .await?;
            report.told += 1;
        }
        sqlx::query!(
            "INSERT INTO engagement_reminders (worker_pid, ends_on, kind, reminded_on) VALUES ($1, $2, $3, $4)",
            worker.pid,
            ends_on,
            reminder.as_str(),
            as_of,
        )
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        match reminder {
            Reminder::Ending => report.ending += 1,
            Reminder::EndedUndecided => report.ended_undecided += 1,
        }
    }
    Ok(report)
}

/// The open conversion plan, named by its status and review date, never its reason; empty when
/// there is none.
/// Who is told: the manager, then the organization's HR administrators, never the worker, each once.
async fn recipients(
    ctx: &AppContext,
    worker: uuid::Uuid,
    manager: Option<uuid::Uuid>,
    organization: &str,
) -> Result<Vec<uuid::Uuid>> {
    let mut recipients: Vec<uuid::Uuid> = manager.into_iter().collect();
    for m in sqlx::query_scalar!(
        "SELECT worker_pid FROM organization_memberships
          WHERE organization_ref = $1 AND role = 'hr_admin' AND deleted_at IS NULL",
        organization,
    )
    .fetch_all(crate::db::pool(ctx))
    .await?
    {
        if let Some(pid) = m
            && pid != worker
            && !recipients.contains(&pid)
        {
            recipients.push(pid);
        }
    }
    Ok(recipients)
}

async fn plan_note(ctx: &AppContext, worker_pid: uuid::Uuid) -> Result<String> {
    let plan = sqlx::query!(
        "SELECT status, review_on FROM conversion_plans
          WHERE worker_pid = $1 AND status IN ('proposed', 'approved') ORDER BY id LIMIT 1",
        worker_pid,
    )
    .fetch_optional(crate::db::pool(ctx))
    .await?;
    Ok(plan.map_or_else(String::new, |p| {
        let review = p
            .review_on
            .map_or_else(String::new, |d| format!(", to be reviewed on {d}"));
        format!(" A conversion plan is {}{review}.", p.status)
    }))
}
