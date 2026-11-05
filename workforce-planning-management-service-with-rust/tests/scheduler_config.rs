//! The scheduler's job list (WPM-T300): every expression parses, every job names a task that exists, and no task is left out
//! without a reason, so a new task is a decision (scheduled, or named here as run by hand) and not something forgotten.

use std::collections::BTreeSet;

use axa::task::Tasks;
use chrono::Utc;
use workforce_planning_management::app::App;
use workforce_planning_management::scheduler::{JOBS, Job, next_after, tagged};

/// Tasks that are run by a person and never on a schedule, and why.
const BY_HAND: &[(&str, &str)] = &[
    (
        "seed",
        "fills a demo database with a synthetic organization",
    ),
    (
        "import_esco",
        "an operator loads the ESCO catalogue once, from a file they chose",
    ),
    (
        "import_sfiaplus",
        "an operator loads the SFIAplus detail they are licensed for, from a file they chose",
    ),
    (
        "import_sfia",
        "an operator loads the SFIA catalogue they are licensed for, from a file they chose",
    ),
    (
        "import_framework",
        "an operator loads a capability framework once, from a file they chose",
    ),
    (
        "replay_erasures",
        "run after a restore, with the ledger file; never on a schedule",
    ),
];

fn job(name: &str) -> &'static Job {
    JOBS.iter()
        .find(|job| job.name == name)
        .unwrap_or_else(|| panic!("{name} is scheduled"))
}

fn names(tasks: &Tasks) -> Vec<String> {
    tasks.list().into_iter().map(|task| task.name).collect()
}

#[test]
fn every_job_names_a_task_and_every_task_is_scheduled_or_run_by_hand() {
    let mut tasks = Tasks::default();
    App::register_tasks(&mut tasks);
    let scheduled: BTreeSet<String> = JOBS.iter().map(|j| j.task.to_string()).collect();
    for job in JOBS {
        assert!(
            names(&tasks).iter().any(|name| name == job.task),
            "{} runs {}, which is not a task",
            job.name,
            job.task
        );
    }
    let by_hand: BTreeSet<&str> = BY_HAND.iter().map(|(name, _)| *name).collect();
    let unplaced: Vec<String> = names(&tasks)
        .into_iter()
        .filter(|name| !scheduled.contains(name) && !by_hand.contains(name.as_str()))
        .collect();
    assert!(
        unplaced.is_empty(),
        "schedule these in src/scheduler.rs, or name them in BY_HAND with a reason: {unplaced:?}"
    );
    for (name, _) in BY_HAND {
        assert!(
            !scheduled.contains(*name),
            "{name} is listed as run by hand and also scheduled"
        );
        assert!(
            names(&tasks).iter().any(|n| n == name),
            "{name} is listed as run by hand but is not a task"
        );
    }
}

#[test]
fn every_cron_has_six_or_seven_fields_and_a_job_that_needs_settings_is_not_in_the_default_tag() {
    for job in JOBS {
        let fields = job.cron.split_whitespace().count();
        assert!(
            (6..=7).contains(&fields),
            "{}: `{}` has {fields} cron fields",
            job.name,
            job.cron
        );
        assert!(
            next_after(job, Utc::now())
                .expect("a valid expression")
                .is_some(),
            "{} never runs",
            job.name
        );
        assert!(
            !job.tags.is_empty(),
            "{} has no tag, so no `--tag` starts it",
            job.name
        );
    }
    assert_eq!(
        tagged("wpm").len() + tagged("scim").len(),
        JOBS.len(),
        "every job has one of the two tags"
    );
    // The tasks that fail unless the deployment has set something up must not start with the default tag.
    assert!(
        !job("scim_push").tags.contains(&"wpm"),
        "scim_push fails without its settings"
    );
    // The ones that must be tried daily or more often are not accidentally weekly or monthly.
    let daily_or_more = [
        "webhook_deliver",
        "snapshot_headcount",
        "early_warning_sweep",
        "engagement_end_reminders",
        "rota_reminders",
        "pay_progression_reminders",
        "policy_reminders",
        "qualification_expiry_reminders",
        "coverage_expiry_reminders",
        "verify_audit_chain",
    ];
    for name in daily_or_more {
        let cron = job(name).cron;
        let parts: Vec<&str> = cron.split_whitespace().collect();
        assert_eq!(
            &parts[3..6],
            ["*", "*", "*"],
            "{name}: day, month and weekday must be open, got `{cron}`"
        );
    }
}
