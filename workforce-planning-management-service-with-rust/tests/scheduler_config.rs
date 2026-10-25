//! The scheduler's job list (WPM-T300): `config/scheduler.yaml` loads, every job names a task that exists, and no task is left out
//! without a reason, so a new task is a decision (scheduled, or named here as run by hand) and not something forgotten.

use std::collections::BTreeSet;
use std::path::Path;

use loco_rs::app::Hooks;
use loco_rs::environment::Environment;
use loco_rs::scheduler::Scheduler;
use loco_rs::task::Tasks;
use workforce_planning_management::app::App;

/// Tasks that are run by a person and never on a schedule, and why.
const BY_HAND: &[(&str, &str)] = &[
    ("seed", "fills a demo database with a synthetic organization"),
    ("import_esco", "an operator loads the ESCO catalogue once, from a file they chose"),
    ("import_framework", "an operator loads a capability framework once, from a file they chose"),
    ("replay_erasures", "run after a restore, with the ledger file; never on a schedule"),
];

fn path() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("config/scheduler.yaml")
}

fn scheduler() -> Scheduler {
    Scheduler::from_config::<App>(&path(), &Environment::Test).expect("config/scheduler.yaml loads and names real tasks")
}

#[test]
fn every_job_names_a_task_and_every_task_is_scheduled_or_run_by_hand() {
    let scheduler = scheduler();
    let mut tasks = Tasks::default();
    App::register_tasks(&mut tasks);
    let scheduled: BTreeSet<String> = scheduler
        .jobs
        .values()
        .filter_map(|j| j.run.split_whitespace().next().map(ToString::to_string))
        .collect();
    let by_hand: BTreeSet<&str> = BY_HAND.iter().map(|(name, _)| *name).collect();
    let unplaced: Vec<String> = tasks
        .names()
        .into_iter()
        .filter(|name| !scheduled.contains(name) && !by_hand.contains(name.as_str()))
        .collect();
    assert!(unplaced.is_empty(), "schedule these in config/scheduler.yaml, or name them in BY_HAND with a reason: {unplaced:?}");
    for (name, _) in BY_HAND {
        assert!(!scheduled.contains(*name), "{name} is listed as run by hand and also scheduled");
        assert!(tasks.names().iter().any(|n| n == name), "{name} is listed as run by hand but is not a task");
    }
}

#[test]
fn every_cron_has_six_or_seven_fields_and_a_job_that_needs_settings_is_not_in_the_default_tag() {
    let scheduler = scheduler();
    for (name, job) in &scheduler.jobs {
        let fields = job.cron.split_whitespace().count();
        assert!((6..=7).contains(&fields), "{name}: `{}` has {fields} cron fields", job.cron);
        assert!(job.tags.as_ref().is_some_and(|t| !t.is_empty()), "{name} has no tag, so no `--tag` starts it");
    }
    // The tasks that fail unless the deployment has set something up must not start with the default tag.
    for needs_settings in ["scim_push"] {
        let job = scheduler.jobs.get(needs_settings).expect("scim_push is scheduled");
        assert!(!job.tags.as_ref().expect("tags").iter().any(|t| t == "wpm"), "{needs_settings} fails without its settings");
    }
    // The ones that must be tried daily or more often are not accidentally weekly or monthly.
    let daily_or_more = ["webhook_deliver", "snapshot_headcount", "early_warning_sweep", "engagement_end_reminders", "rota_reminders", "pay_progression_reminders", "policy_reminders", "verify_audit_chain"];
    for name in daily_or_more {
        let cron = &scheduler.jobs.get(name).unwrap_or_else(|| panic!("{name} is scheduled")).cron;
        let parts: Vec<&str> = cron.split_whitespace().collect();
        assert_eq!(&parts[3..6], ["*", "*", "*"], "{name}: day, month and weekday must be open, got `{cron}`");
    }
}
