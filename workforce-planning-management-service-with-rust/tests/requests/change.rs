//! Change initiatives read back and edited down, and the flat audit and event views:
//! an initiative shows its role impacts and skill shifts by name, removing one that is
//! not there is a 404, a closed initiative takes no removals, and the department and
//! event views answer.

use serde_json::{Value, json};
use serial_test::serial;

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn an_initiative_is_read_back_by_name_and_its_impacts_and_shifts_can_be_removed() {
    crate::requests::request_open(|request, _ctx| async move {
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let skill: Value = request
            .post("/api/skills")
            .json(&json!({ "name": format!("Prompting {tag}"), "category": "technical" }))
            .await
            .json();
        let skill = skill["pid"].as_str().unwrap().to_string();
        let role: Value = request
            .post("/api/role-profiles")
            .json(&json!({ "job_title": format!("Analyst {tag}") }))
            .await
            .json();
        let role = role["pid"].as_str().unwrap().to_string();
        let initiative: Value = request
            .post("/api/change-initiatives")
            .json(&json!({ "name": format!("Rollout {tag}"), "kind": "ai_assistance" }))
            .await
            .json();
        let id = initiative["pid"].as_str().unwrap().to_string();
        let base = format!("/api/change-initiatives/{id}");

        request
            .put(&format!("{base}/role-impacts"))
            .json(&json!({ "role_profile_pid": role, "impact": "reshaped", "timeframe": "within_1y", "note": "pairing" }))
            .await
            .assert_status_ok();
        request
            .put(&format!("{base}/skill-shifts"))
            .json(&json!({ "skill_pid": skill, "direction": "rising" }))
            .await
            .assert_status_ok();

        // Read back with names, not just ids.
        let view: Value = request.get(&base).await.json();
        assert_eq!(view["status"], "draft");
        assert_eq!(view["role_impacts"][0]["job_title"], format!("Analyst {tag}"));
        assert_eq!(view["role_impacts"][0]["note"], "pairing");
        assert_eq!(view["skill_shifts"][0]["skill"], format!("Prompting {tag}"));
        assert_eq!(request.get(&format!("/api/change-initiatives/{}", uuid::Uuid::new_v4())).await.status_code(), 404);

        // Removing something never added is a 404; removing what is there empties the list.
        let stranger = uuid::Uuid::new_v4();
        assert_eq!(request.delete(&format!("{base}/role-impacts/{stranger}")).await.status_code(), 404);
        assert_eq!(request.delete(&format!("{base}/skill-shifts/{stranger}")).await.status_code(), 404);
        request.delete(&format!("{base}/role-impacts/{role}")).await.assert_status_ok();
        request.delete(&format!("{base}/skill-shifts/{skill}")).await.assert_status_ok();
        let emptied: Value = request.get(&base).await.json();
        assert_eq!(emptied["role_impacts"], json!([]));
        assert_eq!(emptied["skill_shifts"], json!([]));

        // A closed initiative takes no removals.
        request
            .put(&format!("{base}/skill-shifts"))
            .json(&json!({ "skill_pid": skill, "direction": "rising" }))
            .await
            .assert_status_ok();
        for to in ["active", "completed"] {
            request.post(&format!("{base}/status")).json(&json!({ "to": to })).await.assert_status_ok();
        }
        assert_eq!(request.delete(&format!("{base}/skill-shifts/{skill}")).await.status_code(), 422);
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn the_department_audit_and_recent_event_views_answer_and_need_their_parameters() {
    crate::requests::request_open(|request, _ctx| async move {
        let since = (chrono::Utc::now() - chrono::Duration::days(1)).to_rfc3339();
        let rows: Value = request
            .get("/api/audits")
            .add_query_params([("department", "engineering"), ("since", since.as_str())])
            .await
            .json();
        assert!(rows.is_array());
        assert!(
            request
                .get("/api/audits")
                .await
                .status_code()
                .is_client_error(),
            "department and since are required"
        );
        let events: Value = request.get("/api/events/recent").await.json();
        assert!(events.is_array());
    })
    .await;
}
