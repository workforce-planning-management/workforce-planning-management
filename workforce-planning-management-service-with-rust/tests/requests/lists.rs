//! The collection reads: each list route answers 200 with a JSON array (or, for the paged
//! ones, a body that holds one), and the health probes answer without a database round trip
//! being required of the caller.

use serde_json::Value;
use serial_test::serial;

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn every_collection_read_answers_with_a_list() {
    crate::requests::request_open(|request, _ctx| async move {
        for url in [
            "/api/candidates",
            "/api/shifts",
            "/api/requisitions",
            "/api/review-cycles",
            "/api/succession-plans",
            "/api/talent-pipelines",
            "/api/workforce-plans",
            "/api/payroll-runs",
            "/api/benchmarks",
            "/api/change-initiatives",
            "/api/learning-paths",
        ] {
            let response = request.get(url).await;
            assert_eq!(response.status_code(), 200, "{url}");
            let body: Value = response.json();
            assert!(
                body.is_array()
                    || body
                        .as_object()
                        .is_some_and(|o| o.values().any(Value::is_array)),
                "{url} should hold a list: {body}"
            );
        }
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn the_liveness_and_readiness_probes_answer() {
    crate::requests::request_open(|request, _ctx| async move {
        for url in ["/_ping", "/_readiness"] {
            assert_eq!(request.get(url).await.status_code(), 200, "{url}");
        }
    })
    .await;
}
