//! The `outbox` event transport (WPM-R16; [integrations](../../spec/integrations.md) "Events"): a change writes its event to `event_outbox` in the same transaction, and the
//! flat event view reads it back from there rather than from the in-process ring.
//!
//! Its own test binary because the transport is read once per process. `#[ignore]`d: boots the app (needs PostgreSQL).
//! Run with `cargo test --test event_outbox -- --ignored`.

use axa_test::prelude::*;
use serde_json::{Value, json};
use serial_test::serial;
use workforce_planning_management::app::App;

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test --test event_outbox -- --ignored`"]
async fn a_change_is_written_to_the_outbox_and_read_back_from_it() {
    // `set_var` is `unsafe` in edition 2024; set before the app boots and reads it.
    unsafe {
        std::env::set_var("WORKFORCE_PLANNING_MANAGEMENT_EVENT_TRANSPORT", "outbox");
        std::env::set_var("WORKFORCE_PLANNING_MANAGEMENT_REQUIRE_AUTH", "0");
        std::env::set_var("WORKFORCE_PLANNING_MANAGEMENT_RATE_LIMIT_PER_MINUTE", "0");
    }
    request::<App, _, _>(|request, ctx| async move {
        assert!(workforce_planning_management::streaming::transport().is_outbox());
        let created = request
            .post("/api/workers")
            .json(&json!({
                "person_ref": format!("person:{}", uuid::Uuid::new_v4()),
                "organization_ref": format!("organization:{}", uuid::Uuid::new_v4()),
                "worker_number": format!("OB-{}", uuid::Uuid::new_v4().simple()),
                "display_name": "Outbox Worker", "employment_type": "permanent", "department": "engineering",
                "job_title": "Engineer", "hired_on": "2026-01-05",
            }))
            .await;
        assert_eq!(created.status_code(), 200, "{}", created.text());
        let pid = created.json::<Value>()["pid"].as_str().unwrap().to_string();

        let stored: i64 = sqlx::query_scalar("SELECT count(*) FROM event_outbox WHERE payload::text LIKE '%' || $1 || '%'")
            .bind(&pid)
            .fetch_one(workforce_planning_management::db::pool(&ctx))
            .await
            .unwrap();
        assert!(stored >= 1, "the change's event is in the outbox");
        let recent: Value = request.get("/api/events/recent").await.json();
        assert!(
            recent.as_array().unwrap().iter().any(|e| e["pid"] == pid.as_str()),
            "the flat view reads the outbox: {recent}"
        );
    })
    .await;
}
