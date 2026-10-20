//! Escalations of a breached tolerance (WPM-R141, WPM-D94): raised by a person on an amber or red
//! dimension, up three levels, decided with a note, the measure kept at each step.

use serde_json::{Value, json};
use serial_test::serial;

use super::{a_person, an_org};

async fn plan_with_breaches(request: &loco_rs::TestServer, org: &str) -> String {
    for i in 0..6 {
        request
            .post("/api/workers")
            .json(&json!({
                "person_ref": a_person(), "organization_ref": org, "worker_number": format!("E{i}"),
                "display_name": format!("Escalation Worker {i}"), "employment_type": "permanent",
                "department": "engineering", "job_title": "Engineer", "hired_on": "2020-01-05",
                "salary_minor": 5_000_000, "salary_currency": "GBP",
            }))
            .await
            .assert_status_ok();
    }
    let plan: Value = request
        .post("/api/workforce-plans")
        .json(&json!({
            "name": "escalations", "organization_ref": org,
            "horizon_start": "2026-01-01", "horizon_end": "2027-12-31",
            "attrition_bp": 1000, "budget_minor": 10_000_000, "budget_currency": "GBP",
        }))
        .await
        .json();
    let pid = plan["pid"].as_str().expect("plan pid").to_string();
    request
        .put(&format!("/api/workforce-plans/{pid}/demand-lines"))
        .json(&json!({ "department": "engineering", "target_on": "2026-06-30", "target_headcount": 10 }))
        .await
        .assert_status_ok();
    for (dimension, limit) in [("time", 100_000), ("cost", 20_000), ("resources", 3_000)] {
        request
            .put(&format!(
                "/api/workforce-plans/{pid}/tolerances/{dimension}"
            ))
            .json(&json!({ "limit": limit }))
            .await
            .assert_status_ok();
    }
    pid
}

async fn view(request: &loco_rs::TestServer, pid: &str) -> Value {
    request
        .get(&format!("/api/workforce-plans/{pid}/tolerances"))
        .await
        .json()
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn a_breach_is_escalated_up_three_levels_and_decided_with_the_measure_kept() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let pid = plan_with_breaches(&request, &org).await;
        let url = format!("/api/workforce-plans/{pid}/escalations");
        let resources = |v: &Value| {
            v["dimensions"]
                .as_array()
                .unwrap()
                .iter()
                .find(|d| d["dimension"] == "resources")
                .unwrap()
                .clone()
        };

        // resources is red and nobody has been asked
        let before = view(&request, &pid).await;
        assert_eq!(resources(&before)["rag"], "red");
        assert_eq!(before["breaches_without_escalation"], 1);

        // only an amber or red dimension may be raised, and the refusal says why
        let green = request
            .post(&url)
            .json(&json!({ "dimension": "cost" }))
            .await;
        assert_eq!(green.status_code(), 422);
        assert!(green.text().contains("green"), "{}", green.text());
        let unknown = request
            .post(&url)
            .json(&json!({ "dimension": "risk" }))
            .await;
        assert_eq!(unknown.status_code(), 422);
        assert!(unknown.text().contains("no tolerance") || unknown.text().contains("unknown"));
        assert_eq!(
            request
                .post(&url)
                .json(&json!({ "dimension": "quality" }))
                .await
                .status_code(),
            422
        );
        let long = request
            .post(&url)
            .json(&json!({ "dimension": "resources", "note": "x".repeat(501) }))
            .await;
        assert_eq!(long.status_code(), 422);

        // raise it: level 1, with the measure at that moment
        let raised: Value = request
            .post(&url)
            .json(&json!({ "dimension": "resources", "note": "the gap is wider than we agreed" }))
            .await
            .json();
        let id = raised["pid"].as_str().expect("escalation pid").to_string();
        assert_eq!(raised["level"], "plan_owner");
        assert_eq!(raised["status"], "open");
        assert_eq!(
            raised["at_raise"],
            json!({ "rag": "red", "deviation": 4000, "limit": 3000 })
        );
        assert_eq!(raised["days_open"], 0);
        // one open per dimension
        assert_eq!(
            request
                .post(&url)
                .json(&json!({ "dimension": "resources" }))
                .await
                .status_code(),
            422
        );

        // the tolerances view shows it, and the breach now has an answer coming
        let during = view(&request, &pid).await;
        assert_eq!(resources(&during)["escalation"]["pid"], id.as_str());
        assert_eq!(during["breaches_without_escalation"], 0);

        // up, up, and no further
        for expected in ["board", "executive"] {
            let up: Value = request
                .post(&format!("/api/escalations/{id}/raise-level"))
                .await
                .json();
            assert_eq!(up["level"], expected);
            assert!(up["level_raised_on"].is_string());
        }
        assert_eq!(
            request
                .post(&format!("/api/escalations/{id}/raise-level"))
                .await
                .status_code(),
            422
        );

        // the inbox lists it, oldest first, with the plan's name
        let inbox: Value = request
            .get(&format!("/api/escalations?organization={org}"))
            .await
            .json();
        assert_eq!(inbox.as_array().unwrap().len(), 1);
        assert_eq!(inbox[0]["plan_name"], "escalations");
        assert_eq!(request.get("/api/escalations").await.status_code(), 422);

        // deciding needs a known decision and a note
        let decide = |body: Value| {
            request
                .post(&format!("/api/escalations/{id}/decide"))
                .json(&body)
        };
        assert_eq!(
            decide(json!({ "decision": "ignore", "note": "x" }))
                .await
                .status_code(),
            422
        );
        assert_eq!(
            decide(json!({ "decision": "stop", "note": "  " }))
                .await
                .status_code(),
            422
        );
        let decided: Value = decide(
            json!({ "decision": "accept_deviation", "note": "accepted until the next intake" }),
        )
        .await
        .json();
        assert_eq!(decided["status"], "decided");
        assert_eq!(decided["decision"], "accept_deviation");
        assert_eq!(
            decided["at_decision"],
            json!({ "rag": "red", "deviation": 4000 })
        );
        assert_eq!(
            decided["at_raise"]["limit"], 3000,
            "the raising measure is kept"
        );
        assert_eq!(
            decide(json!({ "decision": "stop", "note": "again" }))
                .await
                .status_code(),
            422,
            "decided is final"
        );
        assert_eq!(
            request
                .post(&format!("/api/escalations/{id}/raise-level"))
                .await
                .status_code(),
            422
        );

        // deciding changes neither the tolerance nor the plan: it is still red, and unanswered again
        let after = view(&request, &pid).await;
        assert_eq!(resources(&after)["rag"], "red");
        assert_eq!(resources(&after)["escalation"], Value::Null);
        assert_eq!(after["breaches_without_escalation"], 1);

        // the history keeps the decided one, and a new one may be raised
        let again: Value = request
            .post(&url)
            .json(&json!({ "dimension": "resources" }))
            .await
            .json();
        let history: Value = request.get(&url).await.json();
        assert_eq!(history.as_array().unwrap().len(), 2);
        assert_eq!(history[0]["pid"], again["pid"], "open first");
        assert_eq!(history[1]["status"], "decided");
        let none: Value = request
            .get(&format!("/api/escalations?organization={}", an_org()))
            .await
            .json();
        assert_eq!(none, json!([]));
    })
    .await;
}
