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
