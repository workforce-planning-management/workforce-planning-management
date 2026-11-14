//! The collection reads: each list route answers 200 with a JSON array (or, for the paged
//! ones, a body that holds one), and the health probes answer without a database round trip
//! being required of the caller.

use serde_json::Value;
use serial_test::serial;

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
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
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn the_liveness_and_readiness_probes_answer() {
    crate::requests::request_open(|request, _ctx| async move {
        for url in ["/_health/live", "/_health/ready", "/_health/startup"] {
            assert_eq!(request.get(url).await.status_code(), 200, "{url}");
        }
        // axa 0.11 moved the probes (WPM-T482): a probe left on an old path sees a 404 and restarts the service, so say so here.
        for url in ["/_health", "/_ping", "/_readiness"] {
            assert_eq!(request.get(url).await.status_code(), 404, "{url}");
        }
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn a_write_from_another_site_is_refused_before_the_api_sees_it() {
    crate::requests::request_open(|request, _ctx| async move {
        // axa 0.13 refuses, in its standard stack, a write a browser marks cross-site, or one whose `Origin` is not this
        // host when the browser sent no `sec-fetch-site` (WPM-T490). The body is empty, so a write that gets through is
        // the route's own refusal of it, not a 403.
        let write = || request.post("/api/candidates").json(&serde_json::json!({}));
        let cross_site = write().add_header("sec-fetch-site", "cross-site").await;
        assert_eq!(cross_site.status_code(), 403);
        assert!(
            cross_site.text().contains("cross_site_write_refused"),
            "{}",
            cross_site.text()
        );
        let other_origin = write()
            .add_header("origin", "https://elsewhere.example")
            .await;
        assert_eq!(other_origin.status_code(), 403);
        assert!(
            other_origin.text().contains("origin_mismatch"),
            "{}",
            other_origin.text()
        );
        // The same origin, and a caller that is no browser (the UI's server proxy sends neither header), get through.
        let same_origin = write().add_header("sec-fetch-site", "same-origin").await;
        let no_browser = write().await;
        assert_ne!(same_origin.status_code(), 403, "{}", same_origin.text());
        assert_eq!(same_origin.status_code(), no_browser.status_code());
        // A read is never refused for its origin.
        let read = request
            .get("/api/candidates")
            .add_header("sec-fetch-site", "cross-site")
            .await;
        assert_eq!(read.status_code(), 200);
    })
    .await;
}
