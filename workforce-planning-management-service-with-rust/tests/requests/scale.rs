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
async fn the_plan_views_refuse_over_the_ceiling_and_the_paged_views_have_none() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        hire(&request, &org, "1").await;
        hire(&request, &org, "2").await;
        let plan: Value = request
            .post("/api/workforce-plans")
            .json(&json!({
                "name": "scale", "organization_ref": org,
                "horizon_start": "2026-01-01", "horizon_end": "2027-12-31", "attrition_bp": 1000,
            }))
            .await
            .json();
        let pid = plan["pid"].as_str().expect("plan pid").to_string();
        request
            .put(&format!("/api/workforce-plans/{pid}/demand-lines"))
            .json(&json!({ "department": "engineering", "target_on": "2027-06-30", "target_headcount": 5 }))
            .await
            .assert_status_ok();
        let forecast = format!("/api/workforce-plans/{pid}/forecast");
        let cost = format!("/api/workforce-plans/{pid}/cost");
        let capacity = format!("/api/capacity?organization={org}");

        {
            let _ceiling = Ceiling::set("1");
            for route in [&forecast, &cost] {
                let refused = request.get(route).await;
                assert_eq!(refused.status_code(), 422, "{route}: {}", refused.text());
                let body: Value = refused.json();
                assert!(body.to_string().contains("WPM_MAX_ORG_WORKERS"), "{body}");
            }
            // the directory pages in SQL and the capacity view reads in batches: neither has a ceiling
            request.get("/api/directory").await.assert_status_ok();
            request.get(&capacity).await.assert_status_ok();
        }

        let _ceiling = Ceiling::set("1000000");
        request.get(&forecast).await.assert_status_ok();
        request.get(&cost).await.assert_status_ok();
    })
    .await;
}
