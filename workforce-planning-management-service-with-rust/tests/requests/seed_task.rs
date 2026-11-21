//! The seed loads made-up workers, so it runs only in development or test (WPM-R240). The refusal is `axa`'s, through the
//! same `Tasks` the binary runs; this holds WPM to having marked the task.

use axa::{
    Environment,
    task::{Tasks, Vars},
};
use serial_test::serial;
use workforce_planning_management::app::App;

async fn workers(ctx: &axa::AppContext) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM workers")
        .fetch_one(workforce_planning_management::db::pool(ctx))
        .await
        .unwrap()
}

#[tokio::test]
#[serial]
#[ignore = "needs a database"]
async fn the_seed_creates_only_the_documented_synthetic_demo_shape() {
    crate::requests::request_open(|_server, ctx| async move {
        let mut tasks = Tasks::default();
        App::register_tasks(&mut tasks);
        let before = workers(&ctx).await;
        tasks
            .run("seed", &ctx, &Vars::default())
            .await
            .expect("the seed runs under test");
        let pool = workforce_planning_management::db::pool(&ctx);
        assert_eq!(workers(&ctx).await - before, 46);
        let currencies: Vec<(String, i64)> = sqlx::query_as(
            "SELECT salary_currency, count(*) FROM workers WHERE display_name LIKE 'Test %' AND salary_currency IS NOT NULL GROUP BY salary_currency ORDER BY salary_currency",
        )
        .fetch_all(pool)
        .await
        .unwrap();
        assert_eq!(
            currencies,
            [("GBP".to_string(), 40), ("INR".to_string(), 3), ("JPY".to_string(), 3)]
        );
        let homes: Vec<String> = sqlx::query_scalar(
            "SELECT home_jurisdiction FROM organization_settings ORDER BY home_jurisdiction",
        )
        .fetch_all(pool)
        .await
        .unwrap();
        assert_eq!(homes, ["GB", "IN", "JP"]);
        let non_synthetic: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM workers WHERE display_name NOT LIKE 'Test %' OR person_ref NOT LIKE 'person:%'",
        )
        .fetch_one(pool)
        .await
        .unwrap();
        assert_eq!(non_synthetic, 0);
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "needs a database"]
async fn the_seed_is_refused_outside_development_and_test() {
    crate::requests::request_open(|_server, ctx| async move {
        let mut tasks = Tasks::default();
        App::register_tasks(&mut tasks);
        let before = workers(&ctx).await;
        for environment in [
            Environment::Production,
            Environment::Other("demo".to_string()),
            Environment::Other("staging".to_string()),
        ] {
            let mut chosen = ctx.clone();
            chosen.environment = environment.clone();
            let refusal = tasks
                .run("seed", &chosen, &Vars::default())
                .await
                .expect_err("the seed must be refused");
            assert!(
                refusal
                    .to_string()
                    .contains("runs only in development or test"),
                "{environment:?}: {refusal}"
            );
        }
        assert_eq!(workers(&ctx).await, before, "nothing was written");
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "needs a database"]
// Each reminder task runs with a stated day and window, and refuses a day that is not a date and a window out of range.
async fn the_reminder_tasks_read_their_day_and_window_and_refuse_bad_ones() {
    crate::requests::request_open(|_server, ctx| async move {
        let mut tasks = Tasks::default();
        App::register_tasks(&mut tasks);
        let vars = |pairs: &[(&str, &str)]| {
            Vars::from_cli_args(
                pairs
                    .iter()
                    .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
                    .collect(),
            )
        };
        for name in [
            "pay_progression_reminders",
            "qualification_expiry_reminders",
            "engagement_end_reminders",
            "policy_reminders",
            "coverage_expiry_reminders",
            "rota_reminders",
        ] {
            tasks
                .run(
                    name,
                    &ctx,
                    &vars(&[("as_of", "2026-06-01"), ("days_ahead", "7")]),
                )
                .await
                .unwrap_or_else(|e| panic!("{name} runs: {e}"));
            assert!(
                tasks
                    .run(name, &ctx, &vars(&[("as_of", "June")]))
                    .await
                    .is_err(),
                "{name}: not a date"
            );
            assert!(
                tasks
                    .run(name, &ctx, &vars(&[("days_ahead", "9999")]))
                    .await
                    .is_err(),
                "{name}: window out of range"
            );
        }
        // The tasks that take only a day refuse one that is not a date, before doing anything.
        for name in [
            "early_warning_sweep",
            "oversight_pack_freeze",
            "snapshot_headcount",
        ] {
            assert!(
                tasks
                    .run(name, &ctx, &vars(&[("as_of", "June")]))
                    .await
                    .is_err(),
                "{name}: not a date"
            );
        }
    })
    .await;
}
