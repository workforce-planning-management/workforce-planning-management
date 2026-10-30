//! The ergonomic (DSE) round-trip (WPM-R32 / WPM-D24): the default
//! checklist instantiates, completion is gated on every answer, a
//! completed assessment freezes, and issues surface by department.

use sea_orm::ConnectionTrait;
use serde_json::{Value, json};
use serial_test::serial;

use super::{activate, an_org, seed_worker};

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn ergonomics_round_trip() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let worker = seed_worker!(&request, &org, "DSE-1", None).await;
        activate!(&request, &worker).await;

        // Blank workstation refused; default checklist instantiates.
        assert_eq!(
            request
                .post(&format!("/api/workers/{worker}/ergonomic-assessments"))
                .json(&json!({ "workstation": "  " }))
                .await
                .status_code(),
            422
        );
        let created: Value = request
            .post(&format!("/api/workers/{worker}/ergonomic-assessments"))
            .json(&json!({ "workstation": "Desk 4.12" }))
            .await
            .json();
        let a_pid = created["pid"].as_str().unwrap().to_string();
        let listed: Value = request
            .get(&format!("/api/workers/{worker}/ergonomic-assessments"))
            .await
            .json();
        let assessment = &listed.as_array().unwrap()[0];
        let items = assessment["items"].as_array().unwrap().clone();
        assert_eq!(items.len(), 8, "default DSE checklist");
        // stored by key, with the English text beside it for an API reader (WPM-T386)
        let keys: Vec<&str> = items.iter().map(|i| i["item_key"].as_str().unwrap()).collect();
        assert_eq!(
            keys,
            ["screen", "chair", "keyboard_mouse", "desk_space", "lighting", "leg_room", "breaks", "software"]
        );
        assert_eq!(items[1]["name"], "Chair adjustable, with lower-back support");
        assert_eq!(assessment["status"], "open");

        // Completion is gated until every item is answered.
        assert_eq!(
            request
                .post(&format!("/api/ergonomic-assessments/{a_pid}/complete"))
                .await
                .status_code(),
            422,
            "unanswered items block completion"
        );
        // Answer all items; flag one issue with an equipment note.
        for (index, item) in items.iter().enumerate() {
            let item_pid = item["pid"].as_str().unwrap();
            let body = if index == 1 {
                json!({ "ok": false, "note": "chair height lever broken" })
            } else {
                json!({ "ok": true })
            };
            request
                .put(&format!("/api/ergonomic-items/{item_pid}"))
                .json(&body)
                .await
                .assert_status_ok();
        }
        let completed: Value = request
            .post(&format!("/api/ergonomic-assessments/{a_pid}/complete"))
            .await
            .json();
        assert_eq!(completed["status"], "completed");
        assert!(completed["assessed_on"].is_string());
        // A completed assessment is a record: answers freeze.
        let frozen_item = items[0]["pid"].as_str().unwrap();
        assert_eq!(
            request
                .put(&format!("/api/ergonomic-items/{frozen_item}"))
                .json(&json!({ "ok": false }))
                .await
                .status_code(),
            422,
            "completed assessments freeze"
        );

        // The issues report: one engineering issue, with the note.
        let issues: Value = request.get("/api/ergonomics/issues").await.json();
        assert_eq!(issues["by_department"]["engineering"], 1);
        assert_eq!(issues["issues"][0]["workstation"], "Desk 4.12");
        assert_eq!(issues["issues"][0]["note"], "chair height lever broken");
        assert_eq!(issues["issues"][0]["item_key"], "chair");

        // A custom checklist is honoured.
        let custom: Value = request
            .post(&format!("/api/workers/{worker}/ergonomic-assessments"))
            .json(&json!({ "workstation": "home office",
                           "items": ["Laptop stand present", "External keyboard present"] }))
            .await
            .json();
        assert!(custom["pid"].is_string());
        let listed: Value = request
            .get(&format!("/api/workers/{worker}/ergonomic-assessments"))
            .await
            .json();
        let newest = &listed.as_array().unwrap()[0];
        assert_eq!(newest["items"].as_array().unwrap().len(), 2);
        // a custom item has no key and is shown as typed
        assert!(newest["items"][0]["item_key"].is_null(), "{newest}");
        assert_eq!(newest["items"][0]["name"], "Laptop stand present");

        // A row written before keys existed, holding a default sentence and no key, is shown by that
        // sentence's key; one holding other words stays keyless.
        let old_pid = custom["pid"].as_str().unwrap();
        ctx.db
            .execute_unprepared(&format!(
                "INSERT INTO ergonomic_items (pid, assessment_pid, name, created_at, updated_at) VALUES \
                 (gen_random_uuid(), '{old_pid}', 'Leg room clear; feet supported', now(), now())"
            ))
            .await
            .expect("an old-style row");
        let listed: Value = request
            .get(&format!("/api/workers/{worker}/ergonomic-assessments"))
            .await
            .json();
        let items = listed.as_array().unwrap()[0]["items"].as_array().unwrap().clone();
        assert_eq!(items.len(), 3);
        assert_eq!(items[2]["item_key"], "leg_room", "{listed}");
        assert!(items[0]["item_key"].is_null());
    })
    .await;
}
