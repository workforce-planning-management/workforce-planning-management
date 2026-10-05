//! `cargo loco task rota_reminders [days_ahead:N] [as_of:YYYY-MM-DD]` — tell
//! each person whose on-call turn starts `days_ahead` days from `as_of`
//! (default 1 day from today) that it is about to.
//!
//! Run it daily. Idempotent: a person already reminded for that rota and
//! start date is left alone, so a re-run (or two overlapping schedules)
//! sends nothing twice. A turn only counts as starting when the person was
//! *not* already on call the day before, so someone carrying on is not
//! reminded again.

use loco_rs::prelude::*;
use loco_rs::task::{TaskInfo, Vars};

use crate::controllers::rotas::turns_starting_on;
use crate::models::_entities::notifications;
use crate::models::notifications::Model as Notification;

/// The rota-reminder task.
pub struct RotaReminders;

#[async_trait]
impl Task for RotaReminders {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "rota_reminders".to_string(),
            detail: "Remind people whose on-call turn starts soon (optional days_ahead:N, \
                     as_of:YYYY-MM-DD; idempotent)"
                .to_string(),
        }
    }

    async fn run(&self, ctx: &AppContext, vars: &Vars) -> Result<()> {
        let as_of = match vars.cli_arg("as_of") {
            Ok(text) => text
                .parse::<chrono::NaiveDate>()
                .map_err(|e| Error::string(&format!("as_of must be YYYY-MM-DD: {e}")))?,
            Err(_) => chrono::Utc::now().date_naive(),
        };
        let days_ahead = match vars.cli_arg("days_ahead") {
            Ok(text) => text
                .parse::<i64>()
                .ok()
                .filter(|n| (0..=14).contains(n))
                .ok_or_else(|| Error::string("days_ahead must be a number from 0 to 14"))?,
            Err(_) => 1,
        };
        let sent = send_reminders(ctx, as_of + chrono::Duration::days(days_ahead)).await?;
        tracing::info!(%as_of, days_ahead, sent, "on-call reminders sent");
        Ok(())
    }
}

/// Remind everyone whose on-call turn starts on `starts_on`; returns how many
/// were newly told (people already reminded for that rota and day are skipped).
///
/// # Errors
///
/// A database error.
pub async fn send_reminders(ctx: &AppContext, starts_on: chrono::NaiveDate) -> Result<usize> {
    let mut sent = 0;
    for (rota, worker) in turns_starting_on(ctx, starts_on).await? {
        let already = notifications::Entity::find()
            .filter(notifications::Column::WorkerPid.eq(worker))
            .filter(notifications::Column::Kind.eq("on_call_reminder"))
            .all(&ctx.db)
            .await?
            .iter()
            .any(|n| {
                n.data["rota_pid"] == serde_json::json!(rota.pid)
                    && n.data["starts_on"] == serde_json::json!(starts_on)
            });
        if already {
            continue;
        }
        Notification::push(
            &ctx.db,
            worker,
            "on_call_reminder",
            &format!("Your on-call turn for {} starts on {starts_on}.", rota.name),
            serde_json::json!({ "rota_pid": rota.pid, "starts_on": starts_on }),
        )
        .await?;
        sent += 1;
    }
    Ok(sent)
}
