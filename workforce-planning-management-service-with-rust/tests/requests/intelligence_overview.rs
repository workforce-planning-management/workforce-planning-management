//! The workforce overview and period metrics are counted by the database (WPM-D90) and give the answers
//! the definitions say: employed on the date, grouped, with a manager named only if employed too.

use serde_json::{Value, json};
use serial_test::serial;

use super::{a_person, an_org};

async fn hire(
    request: &loco_rs::TestServer,
    org: &str,
    n: usize,
    department: &str,
    hired_on: &str,
    fte: i64,
    manager: Option<&str>,
) -> String {
    let created: Value = request
        .post("/api/workers")
        .json(&json!({
            "person_ref": a_person(), "organization_ref": org, "worker_number": format!("IO{n}"),
            "display_name": format!("Overview Person {n}"), "employment_type": "permanent",
            "department": department, "job_title": "Engineer", "hired_on": hired_on,
            "fte_percent": fte, "manager_pid": manager,
        }))
        .await
        .json();
    created["pid"].as_str().expect("worker pid").to_string()
}

fn number(v: &Value, key: &str) -> i64 {
    v[key].as_i64().unwrap_or(0)
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn the_overview_counts_employed_workers_by_group_with_managers_named_only_if_employed() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let department = format!("dept-{}", uuid::Uuid::new_v4().simple());
        let before: Value = request
            .get("/api/workforce-intelligence/overview")
            .await
            .json();

        // a manager with three reports (one at half time), and a manager not yet employed with two
        let manager = hire(&request, &org, 1, &department, "2020-01-05", 100, None).await;
        hire(
            &request,
            &org,
            2,
            &department,
            "2020-01-05",
            100,
            Some(&manager),
        )
        .await;
        hire(
            &request,
            &org,
            3,
            &department,
            "2020-01-05",
            50,
            Some(&manager),
        )
        .await;
        hire(
            &request,
            &org,
            4,
            &department,
            "2020-01-05",
            100,
            Some(&manager),
        )
        .await;
        let future_manager = hire(&request, &org, 5, &department, "2999-01-01", 100, None).await;
        hire(
            &request,
            &org,
            6,
            &department,
            "2020-01-05",
            100,
            Some(&future_manager),
        )
        .await;
        hire(
            &request,
            &org,
            7,
            &department,
            "2020-01-05",
            100,
            Some(&future_manager),
        )
        .await;

        let after: Value = request
            .get("/api/workforce-intelligence/overview")
            .await
            .json();
        // six employed today; the one hired in 2999 is not
        assert_eq!(
            number(&after, "headcount") - number(&before, "headcount"),
            6
        );
        assert_eq!(after["by_department"][&department], 6);
        // 100 + 100 + 50 + 100 + 100 + 100
        assert_eq!(
            number(&after, "fte_percent_total") - number(&before, "fte_percent_total"),
            550
        );
        assert_eq!(
            number(&after["by_employment_type"], "permanent")
                - number(&before["by_employment_type"], "permanent"),
            6
        );
        // all hired on the same day: exactly one tenure bucket gained all six
        let gained: Vec<(String, i64)> = after["by_tenure"]
            .as_object()
            .unwrap()
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    v.as_i64().unwrap() - before["by_tenure"][k].as_i64().unwrap_or(0),
                )
            })
            .filter(|(_, d)| *d != 0)
            .collect();
        assert_eq!(gained.len(), 1, "{gained:?}");
        assert_eq!(gained[0].1, 6);

        // spans: the employed manager is named, the one not yet employed has reports but no name
        let span = |pid: &str| {
            after["spans_of_control"]
                .as_array()
                .unwrap()
                .iter()
                .find(|s| s["manager_pid"] == pid)
                .cloned()
        };
        let named = span(&manager).expect("the manager's span");
        assert_eq!(named["direct_reports"], 3);
        assert_eq!(named["display_name"], "Overview Person 1");
        let unnamed = span(&future_manager).expect("the future manager's span");
        assert_eq!(unnamed["direct_reports"], 2);
        assert_eq!(unnamed["display_name"], Value::Null);

        // on an earlier date nobody here was employed
        let earlier: Value = request
            .get("/api/workforce-intelligence/overview?as_of=2019-01-01")
            .await
            .json();
        assert_eq!(earlier["by_department"].get(&department), None);
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn period_metrics_count_opening_closing_starters_and_leavers_from_the_dates() {
    crate::requests::request_open(|request, ctx| async move {
        use sea_orm::ConnectionTrait;
        let org = an_org();
        let department = format!("dept-{}", uuid::Uuid::new_v4().simple());
        let metrics = || async {
            request
                .get("/api/workforce-intelligence/metrics?from=2021-01-01&to=2021-12-31")
                .await
                .json::<Value>()
        };
        let before = metrics().await;
        // employed before the period and still there; hired in it; left in it; left before it
        hire(&request, &org, 1, &department, "2019-03-01", 100, None).await;
        hire(&request, &org, 2, &department, "2021-06-01", 100, None).await;
        let leaves = hire(&request, &org, 3, &department, "2019-03-01", 100, None).await;
        let left_before = hire(&request, &org, 4, &department, "2019-03-01", 100, None).await;
        for (worker, on) in [(&leaves, "2021-09-30"), (&left_before, "2020-05-31")] {
            ctx.db
                .execute_unprepared(&format!(
                    "UPDATE workers SET terminated_on = '{on}' WHERE pid = '{worker}'"
                ))
                .await
                .unwrap();
        }
        let after = metrics().await;
        let delta = |path: &[&str]| {
            let get = |v: &Value| {
                path.iter()
                    .fold(v.clone(), |acc, k| acc[*k].clone())
                    .as_i64()
                    .unwrap_or(0)
            };
            get(&after) - get(&before)
        };
        // 2020-12-31 opening: workers 1 and 3 (4 had left); closing 2021-12-31: workers 1 and 2
        assert_eq!(delta(&["headcount", "opening"]), 2, "{after}");
        assert_eq!(delta(&["headcount", "closing"]), 2);
        assert_eq!(delta(&["starters"]), 1);
        assert_eq!(delta(&["leavers"]), 1);
    })
    .await;
}
