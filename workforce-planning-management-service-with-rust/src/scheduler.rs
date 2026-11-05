//! **The recurring tasks** (WPM-T300): the schedule, and the small loop that keeps it, run as
//!
//! ```text
//! workforce-planning-management scheduler --tag wpm    # the tasks that need nothing configured
//! workforce-planning-management scheduler --tag scim   # only where WPM_SCIM_BASE_URL and WPM_SCIM_TOKEN are set
//! workforce-planning-management scheduler --list       # what is scheduled
//! ```
//!
//! Run ONE scheduler per deployment: two would run every task twice. Every task here is idempotent (a second run in the same window
//! sends and records nothing new), so a double run is untidy, not harmful. A job runs as a child process of the scheduler, so it needs
//! the same environment as the server (`DATABASE_URL`, `WPM_*`), and a job already running is not started again. Times are UTC. Cron
//! fields: second minute hour day-of-month month day-of-week. A job's output goes to the scheduler's own output, so a failure is
//! visible.
//!
//! Not here, because they are not tasks: `POST /api/retention/sweep` (an admin action), `scripts/backup.sh` and
//! `scripts/export-erasure-ledger.sh`. `replay_erasures` is run by hand after a restore, never on a schedule.

use std::collections::HashMap;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use clap::Parser;
use cron::Schedule;

/// One recurring task.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Job {
    /// The job's name.
    pub name: &'static str,
    /// The task it runs, as `task <name>` would.
    pub task: &'static str,
    /// When: second minute hour day-of-month month day-of-week.
    pub cron: &'static str,
    /// The tags that start it.
    pub tags: &'static [&'static str],
}

/// Every recurring task.
pub const JOBS: &[Job] = &[
    // Webhooks: send what is queued. The delivery endpoint is rate-limited, so a minute is the finest sensible interval.
    Job {
        name: "webhook_deliver",
        task: "webhook_deliver",
        cron: "0 * * * * *",
        tags: &["wpm"],
    },
    // Headcount history cannot be backfilled: a day missed is a day with no figure. Early in the morning, before the reminders.
    Job {
        name: "snapshot_headcount",
        task: "snapshot_headcount",
        cron: "0 30 1 * * *",
        tags: &["wpm"],
    },
    // Open and close warnings; the date a condition is first true is the date of the first sweep that saw it.
    Job {
        name: "early_warning_sweep",
        task: "early_warning_sweep",
        cron: "0 0 2 * * *",
        tags: &["wpm"],
    },
    // Freeze last month's oversight pack. Idempotent: an organization that already has a pack for the month is skipped, so the
    // first days of the month are all tried and only the first one writes.
    Job {
        name: "oversight_pack_freeze",
        task: "oversight_pack_freeze",
        cron: "0 0 3 1-3 * *",
        tags: &["wpm"],
    },
    // Reminders, once a day, in the morning. Each is idempotent per person and date or version.
    Job {
        name: "engagement_end_reminders",
        task: "engagement_end_reminders",
        cron: "0 0 7 * * *",
        tags: &["wpm"],
    },
    Job {
        name: "rota_reminders",
        task: "rota_reminders",
        cron: "0 5 7 * * *",
        tags: &["wpm"],
    },
    Job {
        name: "pay_progression_reminders",
        task: "pay_progression_reminders",
        cron: "0 10 7 * * *",
        tags: &["wpm"],
    },
    Job {
        name: "qualification_expiry_reminders",
        task: "qualification_expiry_reminders",
        cron: "0 20 7 * * *",
        tags: &["wpm"],
    },
    Job {
        name: "coverage_expiry_reminders",
        task: "coverage_expiry_reminders",
        cron: "0 25 7 * * *",
        tags: &["wpm"],
    },
    Job {
        name: "policy_reminders",
        task: "policy_reminders",
        cron: "0 15 7 * * *",
        tags: &["wpm"],
    },
    // The audit chain: prints whether it holds and its head hash, and exits with an error when it is broken. Keep the output
    // somewhere the database cannot reach (see backup-and-restore.md); this job only runs the check, it cannot keep that record.
    Job {
        name: "verify_audit_chain",
        task: "verify_audit_chain",
        cron: "0 0 4 * * *",
        tags: &["wpm"],
    },
    // SCIM provisioning: needs WPM_SCIM_BASE_URL and WPM_SCIM_TOKEN, and fails every run without them, so it is not tagged `wpm`.
    Job {
        name: "scim_push",
        task: "scim_push",
        cron: "0 */15 * * * *",
        tags: &["scim"],
    },
];

/// The scheduler's command line.
#[derive(Debug, Parser)]
#[command(name = "scheduler", about = "Run the recurring tasks")]
struct Args {
    /// Run the jobs with this tag.
    #[arg(long)]
    tag: Option<String>,
    /// List what is scheduled, and stop.
    #[arg(long)]
    list: bool,
}

/// The jobs with `tag`.
#[must_use]
pub fn tagged(tag: &str) -> Vec<Job> {
    JOBS.iter()
        .filter(|job| job.tags.contains(&tag))
        .copied()
        .collect()
}

/// The next time `job` runs after `now`, or `None` when it never does.
///
/// # Errors
/// A sentence saying the expression is not valid.
pub fn next_after(job: &Job, now: DateTime<Utc>) -> Result<Option<DateTime<Utc>>, String> {
    let schedule =
        Schedule::from_str(job.cron).map_err(|cause| format!("{}: {cause}", job.name))?;
    Ok(schedule.after(&now).next())
}

fn start(job: &Job, exe: &std::path::Path) -> Option<tokio::process::Child> {
    match tokio::process::Command::new(exe)
        .args(["task", job.task])
        .kill_on_drop(false)
        .spawn()
    {
        Ok(child) => Some(child),
        Err(cause) => {
            tracing::error!(job = job.name, %cause, "the job did not start");
            None
        }
    }
}

/// `scheduler ...`: list the schedule, or keep it until the process is stopped.
///
/// # Errors
/// The arguments are wrong, a cron expression is invalid or this program's own path cannot be read.
pub async fn run(arguments: &[String]) -> axa::Result<()> {
    let args = Args::try_parse_from(
        std::iter::once("scheduler").chain(arguments.iter().map(String::as_str)),
    )
    .map_err(|cause| axa::Error::string(&cause.to_string()))?;
    if args.list {
        for job in JOBS {
            println!("{}\t{}\t{}", job.name, job.cron, job.tags.join(","));
        }
        return Ok(());
    }
    let Some(tag) = args.tag else {
        return Err(axa::Error::string(
            "say which jobs to run: --tag wpm, --tag scim, or --list",
        ));
    };
    let jobs = tagged(&tag);
    if jobs.is_empty() {
        return Err(axa::Error::string(&format!("no job has the tag {tag}")));
    }
    let exe = std::env::current_exe()?;
    let mut running: HashMap<&'static str, tokio::process::Child> = HashMap::new();
    loop {
        let now = Utc::now();
        let mut upcoming = Vec::with_capacity(jobs.len());
        for job in &jobs {
            if let Some(at) = next_after(job, now).map_err(|cause| axa::Error::string(&cause))? {
                upcoming.push((at, *job));
            }
        }
        let Some(soonest) = upcoming.iter().map(|(at, _)| *at).min() else {
            return Ok(());
        };
        let wait = (soonest - now).to_std().unwrap_or_default();
        tokio::select! {
            () = tokio::time::sleep(wait) => {}
            _ = tokio::signal::ctrl_c() => return Ok(()),
        }
        running.retain(|_, child| matches!(child.try_wait(), Ok(None)));
        for (_, job) in upcoming.iter().filter(|(at, _)| *at == soonest) {
            if running.contains_key(job.name) {
                tracing::warn!(job = job.name, "still running, so this turn is skipped");
            } else if let Some(child) = start(job, &exe) {
                running.insert(job.name, child);
            }
        }
    }
}
