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

use clap::Parser;
use croner::{
    Cron,
    parser::{CronParser, Seconds, Year},
};
use jiff::{Timestamp, tz::TimeZone};

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
pub fn next_after(job: &Job, now: Timestamp) -> Result<Option<Timestamp>, String> {
    let schedule = parse(job)?;
    // Times are UTC; a search that finds no later time (a year that has passed) is `None`, as before.
    Ok(schedule
        .find_next_occurrence(&now.to_zoned(TimeZone::UTC), false)
        .ok()
        .map(|at| at.timestamp()))
}

/// A job's expression, as the schedule has always written them: seconds first and required, an optional year after the
/// day of the week (six or seven fields).
fn parse(job: &Job) -> Result<Cron, String> {
    CronParser::builder()
        .seconds(Seconds::Required)
        .year(Year::Optional)
        .build()
        .parse(job.cron)
        .map_err(|cause| format!("{}: {cause}", job.name))
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

/// Split the jobs due on this turn into starts and overlaps. Keeping this decision separate from
/// process creation makes the scheduler's no-overlap promise directly testable.
fn due_jobs(
    upcoming: &[(Timestamp, Job)],
    soonest: Timestamp,
    is_running: impl Fn(&str) -> bool,
) -> (Vec<Job>, Vec<Job>) {
    upcoming
        .iter()
        .filter(|(at, _)| *at == soonest)
        .map(|(_, job)| *job)
        .partition(|job| !is_running(job.name))
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
        let now = Timestamp::now();
        let mut upcoming = Vec::with_capacity(jobs.len());
        for job in &jobs {
            if let Some(at) = next_after(job, now).map_err(|cause| axa::Error::string(&cause))? {
                upcoming.push((at, *job));
            }
        }
        let Some(soonest) = upcoming.iter().map(|(at, _)| *at).min() else {
            return Ok(());
        };
        let wait = std::time::Duration::try_from(soonest.duration_since(now)).unwrap_or_default();
        tokio::select! {
            () = tokio::time::sleep(wait) => {}
            _ = tokio::signal::ctrl_c() => return Ok(()),
        }
        running.retain(|_, child| matches!(child.try_wait(), Ok(None)));
        let (starts, overlaps) = due_jobs(&upcoming, soonest, |name| running.contains_key(name));
        for job in overlaps {
            tracing::warn!(job = job.name, "still running, so this turn is skipped");
        }
        for job in starts {
            if let Some(child) = start(&job, &exe) {
                running.insert(job.name, child);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIRST: Job = Job {
        name: "first",
        task: "first_task",
        cron: "0 * * * * *",
        tags: &["test"],
    };
    const SECOND: Job = Job {
        name: "second",
        task: "second_task",
        cron: "0 * * * * *",
        tags: &["test"],
    };

    #[test]
    fn due_jobs_start_once_and_skip_an_overlap() {
        let due: Timestamp = "2026-11-19T12:00:00Z".parse().unwrap();
        let later: Timestamp = "2026-11-19T12:01:00Z".parse().unwrap();
        let upcoming = [(due, FIRST), (due, SECOND), (later, FIRST)];
        let (starts, overlaps) = due_jobs(&upcoming, due, |name| name == "second");
        assert_eq!(starts, [FIRST]);
        assert_eq!(overlaps, [SECOND]);
    }

    #[test]
    fn an_invalid_expression_names_its_job() {
        let invalid = Job {
            cron: "not cron",
            ..FIRST
        };
        assert!(
            next_after(&invalid, Timestamp::now())
                .unwrap_err()
                .starts_with("first:")
        );
    }

    #[tokio::test]
    async fn a_child_starts_and_a_missing_executable_is_reported() {
        let mut child = start(&FIRST, std::path::Path::new("/usr/bin/true")).expect("starts");
        assert!(child.wait().await.expect("wait").success());
        assert!(start(&FIRST, std::path::Path::new("/definitely/not/wpm")).is_none());
    }

    #[tokio::test]
    async fn finite_command_lines_list_or_explain_the_error() {
        run(&["--list".to_string()]).await.unwrap();
        assert!(
            run(&[])
                .await
                .unwrap_err()
                .to_string()
                .contains("say which jobs")
        );
        assert!(
            run(&["--tag".to_string(), "missing".to_string()])
                .await
                .unwrap_err()
                .to_string()
                .contains("no job has the tag missing")
        );
        assert!(run(&["--unknown".to_string()]).await.is_err());
    }
}
