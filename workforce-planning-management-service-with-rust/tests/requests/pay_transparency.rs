//! Pay transparency (WPM-R144, WPM-D99): categories of workers, and the pay gap report that is an aggregate, names no
//! one and withholds a small cell. The auth-on rules are in `tests/enforcement_pay_gap.rs`.

use sea_orm::ConnectionTrait;
use serde_json::{Value, json};
use serial_test::serial;

use super::{a_person, an_org};

/// Switch equality monitoring on for the length of a test, and off again however it ends. The setting is read from
/// the environment on every request, and these tests run one at a time.
struct MonitoringOn;

impl MonitoringOn {
    fn new() -> Self {
        unsafe {
            std::env::set_var(
                "WPM_EQUALITY_MONITORING_BASIS",
                "Test deployment: impact assessment PG-1",
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

const BASIS: &str = "Same skills, effort, responsibility and working conditions.";

/// Hire `n` workers with a title, a sex answer and a salary; returns their pids.
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
                "worker_number": format!("PG-{title}-{}-{i}", sex.unwrap_or("none")),
                "display_name": format!("Pay Gap {title} {i}"), "employment_type": "permanent",
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

async fn category(request: &loco_rs::TestServer, org: &str, name: &str, titles: &[&str]) -> Value {
    request
        .post("/api/pay-categories")
        .json(
            &json!({ "organization_ref": org, "name": name, "basis": BASIS, "job_titles": titles }),
        )
        .await
        .json()
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn a_category_states_its_basis_and_a_title_is_in_one_category() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let made = category(&request, &org, "Engineering", &["Engineer", "Senior Engineer"]).await;
        let pid = made["pid"].as_str().expect("pid").to_string();

        let listed: Value = request.get(&format!("/api/pay-categories?organization={org}")).await.json();
        let rows = listed.as_array().unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["basis"], BASIS, "the stated basis comes back");
        assert_eq!(rows[0]["job_titles"], json!(["Engineer", "Senior Engineer"]));
        assert!(rows[0].get("id").is_none() && rows[0].get("deleted_at").is_none());

        // Refused, each with a reason.
        for (why, body) in [
            ("no basis", json!({ "organization_ref": org, "name": "A", "basis": "short", "job_titles": ["X"] })),
            ("no name", json!({ "organization_ref": org, "name": " ", "basis": BASIS, "job_titles": ["X"] })),
            ("no titles", json!({ "organization_ref": org, "name": "A", "basis": BASIS, "job_titles": [] })),
            ("a title twice", json!({ "organization_ref": org, "name": "A", "basis": BASIS, "job_titles": ["x", "X "] })),
            ("not an organization", json!({ "organization_ref": "worker:x", "name": "A", "basis": BASIS, "job_titles": ["X"] })),
            ("the same name", json!({ "organization_ref": org, "name": "engineering", "basis": BASIS, "job_titles": ["Other"] })),
            ("a title already held", json!({ "organization_ref": org, "name": "Other", "basis": BASIS, "job_titles": ["  ENGINEER"] })),
        ] {
            let r = request.post("/api/pay-categories").json(&body).await;
            assert_eq!(r.status_code(), 422, "{why}: {}", r.text());
        }
        let clash = request
            .post("/api/pay-categories")
            .json(&json!({ "organization_ref": org, "name": "Other", "basis": BASIS, "job_titles": ["engineer"] }))
            .await
            .text();
        assert!(clash.contains("Engineering"), "the reply names the category that holds it: {clash}");

        // Another organization may use the same titles and name.
        let elsewhere = an_org();
        assert_eq!(
            request
                .post("/api/pay-categories")
                .json(&json!({ "organization_ref": elsewhere, "name": "Engineering", "basis": BASIS, "job_titles": ["Engineer"] }))
                .await
                .status_code(),
            200
        );

        // Replace: the titles it holds itself do not clash with it.
        let url = format!("/api/pay-categories/{pid}");
        let put = request
            .put(&url)
            .json(&json!({ "organization_ref": org, "name": "Engineering", "basis": BASIS, "job_titles": ["Engineer", "Staff Engineer"] }))
            .await;
        assert_eq!(put.status_code(), 200, "{}", put.text());
        assert_eq!(
            request
                .put(&url)
                .json(&json!({ "organization_ref": an_org(), "name": "Engineering", "basis": BASIS, "job_titles": ["Engineer"] }))
                .await
                .status_code(),
            422,
            "a category does not move organization"
        );

        // Remove: gone from the list, 404 after, and its titles are free again.
        request.delete(&url).await.assert_status_ok();
        assert_eq!(request.delete(&url).await.status_code(), 404);
        assert_eq!(request.put(&url).json(&json!({ "organization_ref": org, "name": "E", "basis": BASIS, "job_titles": ["E"] })).await.status_code(), 404);
        assert_eq!(request.put("/api/pay-categories/not-a-pid").json(&json!({ "organization_ref": org, "name": "E", "basis": BASIS, "job_titles": ["E"] })).await.status_code(), 404);
        let after: Value = request.get(&format!("/api/pay-categories?organization={org}")).await.json();
        assert!(after.as_array().unwrap().is_empty());
        assert!(category(&request, &org, "Engineering again", &["Engineer"]).await["pid"].is_string());
        assert_eq!(request.get("/api/pay-categories?organization=nope").await.status_code(), 422);
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn the_report_does_not_exist_until_equality_monitoring_is_on_and_refuses_bad_input() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let off = request
            .get(&format!(
                "/api/pay-transparency/gap-report?organization={org}"
            ))
            .await;
        assert_eq!(off.status_code(), 422);
        assert!(
            off.text().contains("equality monitoring is off"),
            "{}",
            off.text()
        );
        assert_eq!(
            request
                .get(&format!(
                    "/api/pay-transparency/gap-report/csv?organization={org}"
                ))
                .await
                .status_code(),
            422
        );

        let _on = MonitoringOn::new();
        for (why, query) in [
            ("a bad organization", "organization=nope".to_string()),
            (
                "a bad currency",
                format!("organization={org}&currency=pounds"),
            ),
            (
                "a bad variable basis",
                format!("organization={org}&variable=most"),
            ),
        ] {
            let r = request
                .get(&format!("/api/pay-transparency/gap-report?{query}"))
                .await;
            assert_eq!(r.status_code(), 422, "{why}: {}", r.text());
        }
        // An organization with nobody: every figure says nobody, none is zero.
        let empty: Value = request
            .get(&format!(
                "/api/pay-transparency/gap-report?organization={org}"
            ))
            .await
            .json();
        assert_eq!(empty["report"]["mean_gap"]["status"], "unknown");
        assert_eq!(empty["report"]["mean_gap"]["value"], "no_workers");
        assert!(empty["reporting"].is_null(), "no tier below 100 workers");
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
#[allow(clippy::too_many_lines)] // one scenario read top to bottom
async fn the_report_is_an_aggregate_that_names_no_one_and_withholds_a_small_cell() {
    crate::requests::request_open(|request, ctx| async move {
        let _on = MonitoringOn::new();
        let org = an_org();
        // Engineers: six women on £90,000, six men on £100,000. Porters: two of each, on £30,000.
        // Extra: one worker with no answer, one who answered, and one with no salary.
        let mut everyone = Vec::new();
        everyone.extend(hire(&request, &ctx, &org, "Engineer", Some("female"), 9_000_000, 6).await);
        everyone.extend(hire(&request, &ctx, &org, "Engineer", Some("male"), 10_000_000, 6).await);
        everyone.extend(hire(&request, &ctx, &org, "Porter", Some("female"), 3_000_000, 2).await);
        everyone.extend(hire(&request, &ctx, &org, "Porter", Some("male"), 3_000_000, 2).await);
        everyone.extend(hire(&request, &ctx, &org, "Surgeon", None, 20_000_000, 1).await);
        category(&request, &org, "Engineering", &["Engineer"]).await;
        category(&request, &org, "Facilities", &["Porter"]).await;

        let response = request
            .get(&format!("/api/pay-transparency/gap-report?organization={org}&currency=GBP"))
            .await;
        assert_eq!(response.status_code(), 200, "{}", response.text());
        let text = response.text();
        let body: Value = serde_json::from_str(&text).unwrap();

        // (a) the gap: women's mean 90,000 against men's mean of (6 x 100,000 + 2 x 30,000) / 8... all four groups together.
        // Whole workforce means: women (6 x 9,000,000 + 2 x 3,000,000) / 8; men (6 x 10,000,000 + 2 x 3,000,000) / 8.
        // Total pay needs the variable part, which nobody holds: so the total-pay indicators are unknown.
        assert_eq!(body["report"]["mean_gap"]["status"], "unknown", "no variable pay is held, so total pay is not known");
        assert_eq!(body["report"]["mean_variable_gap"]["status"], "unknown");
        assert_eq!(body["report"]["receiving_variable"]["status"], "unknown");

        // (g) is basic pay, which is held: engineers' gap is 1,000 basis points, shown; porters are four people, under the floor.
        let categories = body["report"]["categories"].as_array().unwrap();
        let by_name = |n: &str| categories.iter().find(|c| c["category"] == n).unwrap();
        let engineering = &by_name("Engineering")["basic"];
        assert_eq!(engineering["status"], "shown");
        assert_eq!(engineering["value"]["female_minor"], 9_000_000);
        assert_eq!(engineering["value"]["male_minor"], 10_000_000);
        assert_eq!(engineering["value"]["gap_bp"], 1_000);
        assert_eq!(by_name("Facilities")["basic"]["status"], "withheld", "two of each is under the floor of five");
        assert_eq!(by_name("Engineering")["variable"]["status"], "unknown");

        // Left out: the surgeon declared nothing (one is under the floor of five, so "under floor").
        assert_eq!(body["report"]["exclusions"]["no_sex_declared"]["status"], "under_floor");
        assert_eq!(body["report"]["exclusions"]["sex_not_female_or_male"], json!({ "status": "exactly", "value": 0 }));

        // Names no one and gives no head count.
        for pid in &everyone {
            assert!(!text.contains(pid.as_str()), "a worker pid is in the report");
        }
        for banned in ["Pay Gap", "PG-", "display_name", "worker_number", "headcount", "\"count\""] {
            assert!(!text.contains(banned), "{banned} in the report");
        }
        assert_eq!(body["currency"], "GBP");
        assert!(body["basis"]["pay"].as_str().unwrap().contains("full-time-equivalent"));
        assert!(body["basis"]["variable"].as_str().unwrap().contains("unknown"));
        assert!(body["reporting"].is_null(), "17 workers: below every tier");

        // Another currency: nobody has pay in it, so basic pay is not recorded for anyone.
        let usd: Value = request
            .get(&format!("/api/pay-transparency/gap-report?organization={org}&currency=USD"))
            .await
            .json();
        let usd_categories = usd["report"]["categories"].as_array().unwrap();
        assert_eq!(usd_categories.len(), 2, "both configured categories are still listed");
        assert!(usd_categories.iter().all(|c| c["basic"] == json!({ "status": "unknown", "value": "no_workers" })));
        assert_eq!(usd["report"]["exclusions"]["no_pay_recorded"], json!({ "status": "exactly", "value": 16 }));
        // The default currency is the most common one.
        let default: Value = request.get(&format!("/api/pay-transparency/gap-report?organization={org}")).await.json();
        assert_eq!(default["currency"], "GBP");

        // The CSV carries the workings, as text, and names no one.
        let csv = request.get(&format!("/api/pay-transparency/gap-report/csv?organization={org}&currency=GBP")).await;
        assert_eq!(csv.status_code(), 200);
        assert!(csv.header("content-type").to_str().unwrap().starts_with("text/csv"));
        assert!(csv.header("content-disposition").to_str().unwrap().contains("attachment"));
        let csv_text = csv.text();
        assert!(csv_text.starts_with("indicator,scope,status,reason,female,male,gap_bp,value,unit\r\n"), "{csv_text}");
        assert!(
            csv_text.contains("(g) gap in basic pay,Engineering,shown,,9000000,10000000,1000,,minor_units_per_year\r\n"),
            "{csv_text}"
        );
        assert!(csv_text.contains("(g) gap in basic pay,Facilities,withheld,,,,,,minor_units_per_year\r\n"));
        assert!(csv_text.contains("(a) mean gender pay gap,organization,unknown,no_workers"));
        for pid in &everyone {
            assert!(!csv_text.contains(pid.as_str()));
        }

        // Audited: the day and how many categories, no figure.
        let rows = ctx
            .db
            .query_all_raw(sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                "SELECT action, snapshot::text AS snapshot FROM audit_logs WHERE entity = 'pay_gap_report' ORDER BY id".to_string(),
            ))
            .await
            .unwrap();
        let actions: Vec<String> = rows.iter().map(|r| r.try_get("", "action").unwrap()).collect();
        assert!(actions.contains(&"pay_gap_report_read".to_string()) && actions.contains(&"pay_gap_report_exported".to_string()), "{actions:?}");
        for row in &rows {
            let snapshot: String = row.try_get("", "snapshot").unwrap();
            assert!(snapshot.contains("categories") && !snapshot.contains("9000000") && !snapshot.contains("gap_bp"), "{snapshot}");
        }
    })
    .await;
}
