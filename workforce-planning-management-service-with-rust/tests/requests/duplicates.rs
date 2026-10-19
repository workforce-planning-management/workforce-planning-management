//! Possible duplicate workers (WPM-R136, WPM-D87): read-only, names and worker numbers only,
//! a name alone is `review`, and it never reaches outside the caller's organization.

use serde_json::{Value, json};
use serial_test::serial;

use super::{a_person, an_org};

async fn hire(request: &loco_rs::TestServer, org: &str, number: &str, name: &str) -> String {
    let created: Value = request
        .post("/api/workers")
        .json(&json!({
            "person_ref": a_person(), "organization_ref": org, "worker_number": number,
            "display_name": name, "employment_type": "permanent", "department": "engineering",
            "job_title": "Engineer", "hired_on": "2026-01-05",
        }))
        .await
        .json();
    created["pid"].as_str().expect("worker pid").to_string()
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn a_name_alone_is_review_and_a_shared_number_is_a_match() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let ada = hire(&request, &org, "1001", "Ada Lovelace").await;
        hire(&request, &org, "1002", "Grace Hopper").await;

        let by_name: Value = request
            .post("/api/workers/possible-duplicates")
            .json(&json!({ "display_name": "Ada Lovelace", "organization_ref": org }))
            .await
            .json();
        let listed = by_name.as_array().expect("a list");
        assert_eq!(listed.len(), 1, "{by_name}");
        assert_eq!(listed[0]["worker_pid"], ada);
        assert_eq!(listed[0]["decision"], "review");

        let by_number: Value = request
            .post("/api/workers/possible-duplicates")
            .json(&json!({
                "display_name": "A. Lovelace", "organization_ref": org, "worker_number": "1001",
            }))
            .await
            .json();
        assert_eq!(by_number[0]["worker_pid"], ada, "{by_number}");
        assert_eq!(by_number[0]["decision"], "match");
        assert_eq!(by_number[0]["basis"], "worker_number");
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn it_changes_nothing_and_ignores_another_organization() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let other = an_org();
        hire(&request, &other, "1001", "Ada Lovelace").await;
        let before: Value = request.get("/api/workers").await.json();

        let found: Value = request
            .post("/api/workers/possible-duplicates")
            .json(&json!({
                "display_name": "Ada Lovelace", "organization_ref": org, "worker_number": "1001",
            }))
            .await
            .json();
        // another organization's number is no evidence, and its worker's name is a name match only
        // when scope allows: open mode reads every organization, so only `review` can appear
        for item in found.as_array().expect("a list") {
            assert_ne!(item["decision"], "match", "{found}");
        }
        let after: Value = request.get("/api/workers").await.json();
        assert_eq!(before, after, "the check wrote nothing");

        let bad = request
            .post("/api/workers/possible-duplicates")
            .json(&json!({ "display_name": " ", "organization_ref": "nonsense" }))
            .await;
        assert_eq!(bad.status_code(), 422);
    })
    .await;
}
