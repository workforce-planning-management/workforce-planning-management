//! Pay information (WPM-R145, WPM-D99): what HR answers a worker, the pay range a vacancy states before it opens, and
//! that no candidate table has a pay-history column. The worker's own requests are in `tests/enforcement_pay_information.rs`
//! (they need a token).

use sea_orm::ConnectionTrait;
use serde_json::{Value, json};
use serial_test::serial;

use super::{a_person, an_org};

struct MonitoringOn;

impl MonitoringOn {
    fn new() -> Self {
        unsafe {
            std::env::set_var(
                "WPM_EQUALITY_MONITORING_BASIS",
                "Test deployment: impact assessment PI-1",
            );
            std::env::set_var("WPM_EQUALITY_CATEGORIES", r#"{"sex":["female","male"]}"#);
            std::env::set_var("WPM_EQUALITY_FLOOR", "5");
        }
        Self
    }
}

impl Drop for MonitoringOn {
    fn drop(&mut self) {
        unsafe {
            std::env::remove_var("WPM_EQUALITY_MONITORING_BASIS");
            std::env::remove_var("WPM_EQUALITY_CATEGORIES");
            std::env::remove_var("WPM_EQUALITY_FLOOR");
        }
    }
}

async fn hire(
    request: &loco_rs::TestServer,
    ctx: &loco_rs::app::AppContext,
    org: &str,
    title: &str,
    sex: Option<&str>,
    salary: i64,
    n: usize,
) -> Vec<String> {
    let mut pids = Vec::new();
    for i in 0..n {
        let created: Value = request
            .post("/api/workers")
            .json(&json!({
                "person_ref": a_person(), "organization_ref": org,
                "worker_number": format!("PI-{title}-{}-{i}", sex.unwrap_or("none")),
                "display_name": format!("Pay Info {title} {i}"), "employment_type": "permanent",
                "department": "engineering", "job_title": title, "hired_on": "2020-01-05",
                "salary_minor": salary, "salary_currency": "GBP",
            }))
            .await
            .json();
        let pid = created["pid"].as_str().expect("pid").to_string();
        if let Some(sex) = sex {
            ctx.db
                .execute_unprepared(&format!(
                    "INSERT INTO equality_declarations (worker_pid, category, value) VALUES ('{pid}', 'sex', '{sex}')"
                ))
                .await
                .unwrap();
        }
        pids.push(pid);
    }
    pids
}

async fn ask(ctx: &loco_rs::app::AppContext, worker: &str, asked: &str, due: &str) -> String {
    let pid = uuid::Uuid::new_v4().to_string();
    ctx.db
        .execute_unprepared(&format!(
            "INSERT INTO pay_information_requests (pid, worker_pid, asked_on, due_on) VALUES ('{pid}', '{worker}', '{asked}', '{due}')"
        ))
        .await
        .unwrap();
    pid
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
#[allow(clippy::too_many_lines)] // one scenario read top to bottom
async fn hr_answers_with_the_workers_pay_and_their_categorys_average_and_the_figures_are_not_stored()
 {
    crate::requests::request_open(|request, ctx| async move {
        let _on = MonitoringOn::new();
        let org = an_org();
        let women = hire(&request, &ctx, &org, "Engineer", Some("female"), 9_000_000, 6).await;
        hire(&request, &ctx, &org, "Engineer", Some("male"), 10_000_000, 6).await;
        let porter_women = hire(&request, &ctx, &org, "Porter", Some("female"), 3_000_000, 2).await;
        hire(&request, &ctx, &org, "Porter", Some("male"), 3_200_000, 2).await;
        let surgeon = hire(&request, &ctx, &org, "Surgeon", None, 20_000_000, 1).await;
        for (name, titles) in [("Engineering", json!(["Engineer"])), ("Facilities", json!(["Porter"]))] {
            request
                .post("/api/pay-categories")
                .json(&json!({ "organization_ref": org, "name": name, "basis": "Same skills, effort, responsibility and conditions.", "job_titles": titles }))
                .await
                .assert_status_ok();
        }
        let first = ask(&ctx, &women[0], "2026-09-01", "2999-01-01").await;
        let second = ask(&ctx, &porter_women[0], "2026-06-01", "2026-08-01").await;
        let third = ask(&ctx, &surgeon[0], "2026-10-01", "2999-01-01").await;

        // The list: oldest due first, with who asked, no figure; filtered by standing.
        let listed: Value = request.get(&format!("/api/pay-information-requests?organization={org}")).await.json();
        let rows = listed.as_array().unwrap();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0]["pid"], second.as_str(), "overdue first");
        assert_eq!(rows[0]["standing"], "overdue");
        assert!(rows[0]["display_name"].as_str().unwrap().starts_with("Pay Info Porter"));
        assert!(!listed.to_string().contains("9000000"), "no figure in the list");
        let overdue: Value = request.get(&format!("/api/pay-information-requests?organization={org}&status=overdue")).await.json();
        assert_eq!(overdue.as_array().unwrap().len(), 1);
        assert_eq!(request.get("/api/pay-information-requests?status=sometimes").await.status_code(), 422);
        assert_eq!(request.get(&format!("/api/pay-information-requests?organization={}", an_org())).await.json::<Value>().as_array().unwrap().len(), 0);

        // An engineer: her own pay and both averages, each from six people.
        let a: Value = request.get(&format!("/api/pay-information-requests/{first}/answer")).await.json();
        assert_eq!(a["answer"]["own_pay_minor"], 9_000_000);
        assert_eq!(a["answer"]["female_average_minor"], json!({ "status": "shown", "value": 9_000_000 }));
        assert_eq!(a["answer"]["male_average_minor"], json!({ "status": "shown", "value": 10_000_000 }));
        assert_eq!(a["category"]["name"], "Engineering");
        assert!(a["category"]["basis"].as_str().unwrap().contains("responsibility"));
        assert_eq!(a["small_group_floor"], 5);
        let text = a.to_string();
        for pid in &women {
            assert!(!text.contains(pid.as_str()), "no other worker is named");
        }
        assert!(!text.contains("Pay Info"), "no name in the answer");

        // A porter: two of each is under the floor, so neither average is given; her own pay still is.
        let b: Value = request.get(&format!("/api/pay-information-requests/{second}/answer")).await.json();
        assert_eq!(b["answer"]["own_pay_minor"], 3_000_000);
        assert_eq!(b["answer"]["female_average_minor"], json!({ "status": "withheld" }));
        assert_eq!(b["answer"]["male_average_minor"], json!({ "status": "withheld" }));

        // A surgeon is in no category: no average, said so; own pay still given.
        let c: Value = request.get(&format!("/api/pay-information-requests/{third}/answer")).await.json();
        assert_eq!(c["answer"]["own_pay_minor"], 20_000_000);
        assert_eq!(c["answer"]["female_average_minor"], json!({ "status": "unknown", "value": "no_category" }));
        assert!(c["category"].is_null());

        // Record that it was answered: once. The row keeps dates, never the figures.
        let done = request.post(&format!("/api/pay-information-requests/{first}/answered")).await;
        assert_eq!(done.status_code(), 200, "{}", done.text());
        assert_eq!(done.json::<Value>()["standing"], "answered");
        assert_eq!(request.post(&format!("/api/pay-information-requests/{first}/answered")).await.status_code(), 422);
        let stored = ctx
            .db
            .query_all_raw(sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                format!("SELECT row_to_json(r)::text AS t FROM pay_information_requests r WHERE pid = '{first}'"),
            ))
            .await
            .unwrap();
        let row: String = stored[0].try_get("", "t").unwrap();
        assert!(!row.contains("9000000") && !row.contains("10000000"), "the figures are not stored: {row}");
        for missing in [format!("/api/pay-information-requests/{}/answer", uuid::Uuid::new_v4()), "/api/pay-information-requests/nope/answer".to_string()] {
            assert_eq!(request.get(&missing).await.status_code(), 404, "{missing}");
        }
        assert_eq!(request.post(&format!("/api/pay-information-requests/{}/answered", uuid::Uuid::new_v4())).await.status_code(), 404);

        // Audited, with no figure.
        let audits = ctx
            .db
            .query_all_raw(sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                "SELECT action, snapshot::text AS snapshot FROM audit_logs WHERE entity = 'pay_information_request'".to_string(),
            ))
            .await
            .unwrap();
        let actions: Vec<String> = audits.iter().map(|r| r.try_get("", "action").unwrap()).collect();
        assert!(actions.contains(&"pay_information_answer_read".to_string()) && actions.contains(&"pay_information_answered".to_string()), "{actions:?}");
        for r in &audits {
            let snapshot: Option<String> = r.try_get("", "snapshot").unwrap();
            assert!(!snapshot.unwrap_or_default().contains("000000"));
        }
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn without_equality_monitoring_the_worker_still_gets_their_own_pay_and_the_averages_say_why_not()
 {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let workers = hire(&request, &ctx, &org, "Engineer", None, 9_000_000, 6).await;
        request
            .post("/api/pay-categories")
            .json(&json!({ "organization_ref": org, "name": "Engineering", "basis": "Same skills, effort, responsibility and conditions.", "job_titles": ["Engineer"] }))
            .await
            .assert_status_ok();
        let pid = ask(&ctx, &workers[0], "2026-10-01", "2999-01-01").await;
        let a: Value = request.get(&format!("/api/pay-information-requests/{pid}/answer")).await.json();
        assert_eq!(a["answer"]["own_pay_minor"], 9_000_000);
        assert_eq!(a["answer"]["female_average_minor"], json!({ "status": "unknown", "value": "no_sex_recorded" }));
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn a_vacancy_states_its_pay_range_before_it_is_opened() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let make = |extra: Value| {
            let request = &request;
            let org = org.clone();
            async move {
                let mut body = json!({ "organization_ref": org, "department": "engineering", "job_title": "Engineer", "headcount": 1 });
                body.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
                request.post("/api/requisitions").json(&body).await.json::<Value>()["pid"].as_str().unwrap().to_string()
            }
        };
        let open = |pid: String| {
            let request = &request;
            async move { request.post(&format!("/api/requisitions/{pid}/status")).json(&json!({ "to": "open" })).await }
        };
        // No range, one end of one, and no pay at all: refused, with what to do.
        for (why, extra) in [
            ("no range", json!({})),
            ("only a minimum", json!({ "salary_min_minor": 3_000_000, "salary_currency": "GBP" })),
            ("only a maximum", json!({ "salary_max_minor": 4_000_000, "salary_currency": "GBP" })),
            ("a zero minimum", json!({ "salary_min_minor": 0, "salary_max_minor": 4_000_000, "salary_currency": "GBP" })),
        ] {
            let r = open(make(extra).await).await;
            assert_eq!(r.status_code(), 422, "{why}: {}", r.text());
            assert!(r.text().contains("pay range"), "{why}: {}", r.text());
        }
        // A range, or one stated figure, opens.
        assert_eq!(open(make(json!({ "salary_min_minor": 3_000_000, "salary_max_minor": 4_000_000, "salary_currency": "GBP" })).await).await.status_code(), 200);
        assert_eq!(open(make(json!({ "salary_min_minor": 3_500_000, "salary_max_minor": 3_500_000, "salary_currency": "GBP" })).await).await.status_code(), 200);
        // The gate is a setting: a deployment outside the Directive's reach may turn it off.
        unsafe { std::env::set_var("WPM_REQUIRE_PAY_RANGE", "off") };
        let off = open(make(json!({})).await).await.status_code();
        unsafe { std::env::remove_var("WPM_REQUIRE_PAY_RANGE") };
        assert_eq!(off, 200, "off: opens without a range");
        // A draft can still be cancelled or left alone without a range: only opening is gated.
        let draft = make(json!({})).await;
        let cancelled = request.post(&format!("/api/requisitions/{draft}/status")).json(&json!({ "to": "cancelled" })).await;
        assert_eq!(cancelled.status_code(), 200, "{}", cancelled.text());
    })
    .await;
}

/// A candidate is owed the pay for the job and is never asked what they were paid before (Article 5): so no table of
/// candidate data has a column that could hold a pay history.
#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn no_candidate_table_has_a_pay_history_column() {
    crate::requests::request_open(|_request, ctx| async move {
        let rows = ctx
            .db
            .query_all_raw(sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                "SELECT table_name, column_name FROM information_schema.columns
                  WHERE table_schema = current_schema()
                    AND (table_name LIKE '%candidate%' OR table_name IN ('applications', 'interviews'))
                  ORDER BY table_name, column_name"
                    .to_string(),
            ))
            .await
            .unwrap();
        assert!(rows.len() > 5, "the guard reads the candidate tables");
        let words = ["salary", "pay", "wage", "remuneration", "compensation", "earnings", "bonus", "ctc"];
        let offenders: Vec<String> = rows
            .iter()
            .map(|r| (r.try_get::<String>("", "table_name").unwrap(), r.try_get::<String>("", "column_name").unwrap()))
            .filter(|(_, column)| words.iter().any(|w| column.to_lowercase().contains(w)))
            .map(|(t, c)| format!("{t}.{c}"))
            .collect();
        assert_eq!(offenders, Vec::<String>::new(), "a candidate record must not hold a pay history");
    })
    .await;
}
