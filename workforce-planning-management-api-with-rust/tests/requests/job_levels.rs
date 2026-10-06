//! Job levels: Google's L3–L11 ladder is served as reference data, with what the
//! source does not state left null, and an unknown framework or level is a 404.

use loco_rs::testing::prelude::*;
use serde_json::Value;
use serial_test::serial;
use workforce_planning_management_service::app::App;

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn google_levels_are_served_with_unknowns_left_null() {
    request::<App, _, _>(|request, _ctx| async move {
        let list: Value = request.get("/api/job-levels").await.json();
        let google = &list.as_array().unwrap()[0];
        assert_eq!(google["id"], "google-levels");
        assert_eq!(google["organization"], "Google");
        assert_eq!(google["levels"].as_array().unwrap().len(), 9);
        assert!(
            google["source"]
                .as_str()
                .unwrap()
                .contains("Not an official")
        );

        let full: Value = request.get("/api/job-levels/google-levels").await.json();
        let levels = full["levels"].as_array().unwrap();
        assert_eq!(levels[0]["code"], "L3");
        assert_eq!(levels[8]["code"], "L11");
        // No pay anywhere on a level.
        assert!(levels.iter().all(|l| l.get("salary_minor").is_none()));

        // L7 states no experience; L8 is a director equivalent.
        let l7: Value = request
            .get("/api/job-levels/google-levels/levels/L7")
            .await
            .json();
        assert_eq!(l7["level"]["experience"], Value::Null);
        assert_eq!(l7["next_level"]["code"], "L8");
        let l8: Value = request
            .get("/api/job-levels/google-levels/levels/l8")
            .await
            .json();
        assert_eq!(l8["level"]["management_equivalent"], "Director");
        assert_eq!(l8["level"]["experience"], "15+ years");
        // The top has nothing above it; a bare number works.
        let top: Value = request
            .get("/api/job-levels/google-levels/levels/11")
            .await
            .json();
        assert_eq!(top["level"]["code"], "L11");
        assert_eq!(top["next_level"], Value::Null);

        for url in [
            "/api/job-levels/meta-levels",
            "/api/job-levels/google-levels/levels/L2",
            "/api/job-levels/google-levels/levels/L12",
            "/api/job-levels/google-levels/levels/banana",
        ] {
            assert_eq!(request.get(url).await.status_code(), 404, "{url}");
        }
    })
    .await;
}
