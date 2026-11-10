//! `cargo run -- task coverage_expiry_reminders [days_ahead:N] [as_of:YYYY-MM-DD]` — tell an organization's HR that a workers' compensation
//! or employer's liability record is about to end or has just ended (WPM-R220, WPM-T420).
//!
//! Run it daily. The window defaults to 60 calendar days. **Idempotent:** each record, end date and kind of reminder is recorded, so a
//! re-run, or two overlapping schedules, tell no one twice; a renewal gives the record a new end date and so a new reminder. An end that
//! passed more than the window ago is a past fact and is not reminded; a record with no end date has nothing to remind about, and the list
//! says so. A retired record is not reminded. The notification names the jurisdiction, the kind and the date, and nothing else: no
//! insurer, policy reference or certificate. It goes to the organization's `hr_admin` members.

use axa::prelude::*;
use axa::task::{TaskInfo, Vars};
use chrono::NaiveDate;
use serde_json::json;

use crate::rules::qualification::{self as rules, Reminder};

/// The reminder task.
pub struct CoverageExpiryReminders;

#[async_trait]
impl Task for CoverageExpiryReminders {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "coverage_expiry_reminders".to_string(),
            detail: "Tell an organization's HR that workers' compensation or liability cover is ending soon or has ended \
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
        tracing::info!(%as_of, window, expiring = report.expiring, expired = report.expired, told = report.told, "coverage reminders sent");
        Ok(())
    }
}

/// What a run did.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Report {
    /// Records newly reminded as ending soon.
    pub expiring: usize,
    /// Records newly reminded as just ended.
    pub expired: usize,
    /// Notifications created (one per HR member per record).
    pub told: usize,
}

/// Remind about every coverage record that needs it on `as_of`.
///
/// # Errors
///
/// A database error.
pub async fn send_reminders(ctx: &AppContext, as_of: NaiveDate, window: i64) -> Result<Report> {
    let mut report = Report::default();
    let held = sqlx::query!(
        "SELECT pid, organization_ref, kind, jurisdiction, valid_to FROM coverage_evidence
          WHERE valid_to IS NOT NULL AND retired_on IS NULL ORDER BY id"
    )
    .fetch_all(crate::db::pool(ctx))
    .await?;
    for record in held {
        let Some(valid_to) = record.valid_to else {
            continue;
        };
        let Some(reminder) = rules::reminder_for(valid_to, as_of, window) else {
            continue;
        };
        let already = sqlx::query_scalar!(
            r#"SELECT EXISTS (SELECT 1 FROM coverage_reminders WHERE coverage_pid = $1 AND valid_to = $2 AND kind = $3) AS "e!""#,
            record.pid,
            valid_to,
            reminder.as_str(),
        )
        .fetch_one(crate::db::pool(ctx))
        .await?;
        if already {
            continue;
        }
        let (kind, body) = match reminder {
            Reminder::Expiring => (
                "coverage_expiring",
                format!(
                    "The {} cover recorded for {} ends on {valid_to}. Renew it, or record the new cover.",
                    record.kind.replace('_', " "),
                    record.jurisdiction
                ),
            ),
            Reminder::Expired => (
                "coverage_expired",
                format!(
                    "The {} cover recorded for {} ended on {valid_to}. Record its renewal, or retire the record.",
                    record.kind.replace('_', " "),
                    record.jurisdiction
                ),
            ),
        };
        let data = json!({ "coverage_pid": record.pid, "valid_to": valid_to });
        let mut tx = crate::db::pool(ctx).begin().await?;
        for pid in sqlx::query_scalar!(
            "SELECT worker_pid FROM organization_memberships
              WHERE organization_ref = $1 AND role = 'hr_admin' AND deleted_at IS NULL",
            record.organization_ref,
        )
        .fetch_all(&mut *tx)
        .await?
        .into_iter()
        .flatten()
        {
            crate::db::notify(&mut tx, pid, kind, &body, data.clone()).await?;
            report.told += 1;
        }
        sqlx::query!(
            "INSERT INTO coverage_reminders (coverage_pid, valid_to, kind, reminded_on) VALUES ($1, $2, $3, $4)",
            record.pid,
            valid_to,
            reminder.as_str(),
            as_of,
        )
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        match reminder {
            Reminder::Expiring => report.expiring += 1,
            Reminder::Expired => report.expired += 1,
        }
    }
    Ok(report)
}
