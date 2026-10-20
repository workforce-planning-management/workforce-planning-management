//! Tolerances on a workforce plan (WPM-R140, WPM-D93): the deviation, the limit and a red, amber or
//! green status, with an unknown never green.

use chrono::Utc;
use serde_json::{Value, json};
use serial_test::serial;

use super::{a_person, an_org};

async fn hire_salaried(request: &loco_rs::TestServer, org: &str, n: usize) {
    for i in 0..n {
        request
            .post("/api/workers")
            .json(&json!({
                "person_ref": a_person(), "organization_ref": org, "worker_number": format!("T{i}"),
                "display_name": format!("Tolerance Worker {i}"), "employment_type": "permanent",
                "department": "engineering", "job_title": "Engineer", "hired_on": "2020-01-05",
                "salary_minor": 5_000_000, "salary_currency": "GBP",
            }))
            .await
            .assert_status_ok();
    }
}

fn status<'a>(view: &'a Value, dimension: &str) -> &'a Value {
    view["dimensions"]
        .as_array()
        .expect("dimensions")
        .iter()
        .find(|d| d["dimension"] == dimension)
        .expect("dimension")
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn a_plan_shows_red_amber_green_and_an_unknown_is_never_green() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        hire_salaried(&request, &org, 6).await;
        let plan: Value = request
            .post("/api/workforce-plans")
            .json(&json!({
                "name": "tolerances", "organization_ref": org,
                "horizon_start": "2026-01-01", "horizon_end": "2027-12-31",
                "attrition_bp": 1000, "budget_minor": 10_000_000, "budget_currency": "GBP",
            }))
            .await
            .json();
        let pid = plan["pid"].as_str().expect("plan pid").to_string();
        // 10 wanted by 30 June 2026 (already past), 6 employed: a gap of 4
        request
            .put(&format!("/api/workforce-plans/{pid}/demand-lines"))
            .json(&json!({ "department": "engineering", "target_on": "2026-06-30", "target_headcount": 10 }))
            .await
            .assert_status_ok();
        let url = format!("/api/workforce-plans/{pid}/tolerances");
        let days_past = (Utc::now().date_naive() - chrono::NaiveDate::from_ymd_opt(2026, 6, 30).unwrap()).num_days();

        // nothing set: every dimension has a measured deviation and says not_set; the plan too
        let none: Value = request.get(&url).await.json();
        assert_eq!(none["overall"], "not_set");
        assert_eq!(status(&none, "time")["rag"], "not_set");
        assert_eq!(status(&none, "time")["deviation"], days_past);
        assert_eq!(status(&none, "resources")["deviation"], 4_000); // 4 of 10
        assert_eq!(status(&none, "cost")["deviation"], 10_000); // 20,000,000 against 10,000,000
        assert_eq!(status(&none, "risk")["deviation"], Value::Null, "no succession plans: unknown, not 0");

        let set = |dimension: &str, body: Value| {
            request.put(&format!("{url}/{dimension}")).json(&body)
        };
        set("time", json!({ "limit": days_past + 10 })).await.assert_status_ok(); // ~91% used: amber
        set("resources", json!({ "limit": 3_000 })).await.assert_status_ok(); // 4,000 over 3,000: red
        set("cost", json!({ "limit": 20_000 })).await.assert_status_ok(); // 50% used: green
        let view: Value = set("risk", json!({ "limit": 0 })).await.json(); // nothing to look at: unknown
        assert_eq!(status(&view, "time")["rag"], "amber");
        assert_eq!(status(&view, "resources")["rag"], "red");
        assert_eq!(status(&view, "cost")["rag"], "green");
        assert_eq!(status(&view, "risk")["rag"], "unknown");
        assert!(status(&view, "risk")["reason"].as_str().unwrap().contains("succession"));
        assert_eq!(view["overall"], "red");

        // loosen resources: 4,000 of 5,000 is 80%, amber; the unknown still keeps the overall from green
        let view: Value = set("resources", json!({ "limit": 5_000 })).await.json();
        assert_eq!(status(&view, "resources")["rag"], "amber");
        assert_eq!(view["overall"], "amber");
        // a stricter amber point turns the green cost amber
        let view: Value = set("cost", json!({ "limit": 20_000, "amber_percent": 50 })).await.json();
        assert_eq!(status(&view, "cost")["rag"], "amber");

        // with only green and unknown set, the plan is unknown and not green
        for dimension in ["time", "resources"] {
            request.delete(&format!("{url}/{dimension}")).await.assert_status_ok();
        }
        let view: Value = set("cost", json!({ "limit": 20_000 })).await.json();
        assert_eq!(view["overall"], "unknown");

        // refusals
        assert_eq!(set("time", json!({ "limit": -1 })).await.status_code(), 422);
        assert_eq!(set("time", json!({ "limit": 5, "amber_percent": 100 })).await.status_code(), 422);
        assert_eq!(set("quality", json!({ "limit": 5 })).await.status_code(), 422);
        assert_eq!(request.delete(&format!("{url}/time")).await.status_code(), 404);
    })
    .await;
}
