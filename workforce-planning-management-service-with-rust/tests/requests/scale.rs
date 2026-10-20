//! A whole-organization view refuses to load more workers than the ceiling (WPM-R120, WPM-D90).

use serde_json::{Value, json};
use serial_test::serial;

use super::{a_person, an_org};

async fn hire(request: &loco_rs::TestServer, org: &str, number: &str) {
    request
        .post("/api/workers")
        .json(&json!({
            "person_ref": a_person(), "organization_ref": org, "worker_number": number,
            "display_name": format!("Worker {number}"), "employment_type": "permanent",
            "department": "engineering", "job_title": "Engineer", "hired_on": "2026-01-05",
        }))
        .await
        .assert_status_ok();
}

/// Restores the ceiling setting when the test ends, even on a failed assertion.
struct Ceiling;

impl Ceiling {
    fn set(value: &str) -> Self {
        // SAFETY: the tests that read this setting are `#[serial]`.
        unsafe { std::env::set_var("WPM_MAX_ORG_WORKERS", value) };
        Self
    }
}

impl Drop for Ceiling {
    fn drop(&mut self) {
        // SAFETY: as above.
        unsafe { std::env::remove_var("WPM_MAX_ORG_WORKERS") };
    }
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn the_directory_and_capacity_refuse_over_the_ceiling_and_answer_at_it() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        hire(&request, &org, "1").await;
        hire(&request, &org, "2").await;
        let capacity = format!("/api/capacity?organization={org}");

        {
            let _ceiling = Ceiling::set("1");
            let refused = request.get("/api/directory").await;
            assert_eq!(refused.status_code(), 422, "{}", refused.text());
            let body: Value = refused.json();
            assert!(body.to_string().contains("WPM_MAX_ORG_WORKERS"), "{body}");
            assert_eq!(request.get(&capacity).await.status_code(), 422);
        }

        let _ceiling = Ceiling::set("1000000");
        request.get("/api/directory").await.assert_status_ok();
        request.get(&capacity).await.assert_status_ok();
    })
    .await;
}
