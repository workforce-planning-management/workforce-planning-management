//! Emergency contacts and backups: a person provides both about themselves.
//! Contacts are validated, ranked, capped, and exported/erased with the
//! person; backups name colleagues who cover, and `cover` resolves who does
//! on a day (skipping someone outside their window or no longer employed).

use serde_json::{Value, json};
use serial_test::serial;

use super::{activate, an_org, seed_worker};

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn emergency_contacts_are_validated_ranked_capped_and_exported() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let worker = seed_worker!(&request, &org, "EC-1", None).await;
        activate!(&request, &worker).await;
        let url = format!("/api/workers/{worker}/emergency-contacts");

        // An undialable phone and a nameless contact are refused.
        for bad in [
            json!({ "name": "Sam", "relationship": "Partner", "phone": "n/a" }),
            json!({ "name": " ", "relationship": "Partner", "phone": "+44 7700 900123" }),
            json!({ "name": "Sam", "relationship": "Partner", "phone": "12345", "email": "nope" }),
        ] {
            assert_eq!(
                request.post(&url).json(&bad).await.status_code(),
                422,
                "{bad}"
            );
        }

        // Add two: the second is asked second by default; then reorder.
        let first: Value = request
            .post(&url)
            .json(&json!({ "name": "Sam Lee", "relationship": "Partner",
                           "phone": "+44 7700 900123", "email": "sam@example.org" }))
            .await
            .json();
        let second: Value = request
            .post(&url)
            .json(
                &json!({ "name": "Kim Lee", "relationship": "Sibling", "phone": "020 7946 0018" }),
            )
            .await
            .json();
        let listed: Vec<Value> = request.get(&url).await.json();
        assert_eq!(listed.len(), 2);
        assert_eq!(listed[0]["name"], "Sam Lee");
        assert_eq!(listed[0]["priority"], 1);
        assert_eq!(listed[1]["priority"], 2);

        let second_pid = second["pid"].as_str().expect("pid");
        let updated: Value = request
            .put(&format!("/api/emergency-contacts/{second_pid}"))
            .json(&json!({ "priority": 1, "note": "speaks Welsh" }))
            .await
            .json();
        assert_eq!(updated["priority"], 1);
        assert_eq!(updated["note"], "speaks Welsh");
        assert_eq!(
            request
                .put(&format!("/api/emergency-contacts/{second_pid}"))
                .json(&json!({ "phone": "bad" }))
                .await
                .status_code(),
            422
        );

        // The subject-access export carries them (they are the person's data).
        let export: Value = request
            .get(&format!("/api/workers/{worker}/subject-access"))
            .await
            .json();
        assert_eq!(
            export["emergency_contacts"].as_array().map(Vec::len),
            Some(2)
        );

        // Capped at five.
        for n in 3..=5 {
            request
                .post(&url)
                .json(
                    &json!({ "name": format!("C{n}"), "relationship": "Friend", "phone": "12345" }),
                )
                .await
                .assert_status_ok();
        }
        assert_eq!(
            request
                .post(&url)
                .json(&json!({ "name": "C6", "relationship": "Friend", "phone": "12345" }))
                .await
                .status_code(),
            422,
            "at most five"
        );

        // Remove one.
        let first_pid = first["pid"].as_str().expect("pid");
        request
            .delete(&format!("/api/emergency-contacts/{first_pid}"))
            .await
            .assert_status_ok();
        let after: Vec<Value> = request.get(&url).await.json();
        assert_eq!(after.len(), 4);
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn backups_cover_in_rank_order_and_skip_the_unavailable() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let me = seed_worker!(&request, &org, "BK-ME", None).await;
        let first = seed_worker!(&request, &org, "BK-1", None).await;
        let second = seed_worker!(&request, &org, "BK-2", None).await;
        for pid in [&me, &first, &second] {
            activate!(&request, pid).await;
        }
        let url = format!("/api/workers/{me}/backups");

        // Not yourself, not a stranger, not twice.
        assert_eq!(
            request
                .post(&url)
                .json(&json!({ "backup_pid": me }))
                .await
                .status_code(),
            422
        );
        assert_eq!(
            request
                .post(&url)
                .json(&json!({ "backup_pid": uuid::Uuid::new_v4() }))
                .await
                .status_code(),
            422
        );
        let first_row: Value = request
            .post(&url)
            .json(&json!({ "backup_pid": first, "note": "knows the payroll run" }))
            .await
            .json();
        assert_eq!(
            request
                .post(&url)
                .json(&json!({ "backup_pid": first }))
                .await
                .status_code(),
            422,
            "already named"
        );
        request
            .post(&url)
            .json(&json!({ "backup_pid": second }))
            .await
            .assert_status_ok();

        let listed: Vec<Value> = request.get(&url).await.json();
        assert_eq!(listed.len(), 2);
        assert_eq!(listed[0]["backup_pid"], first.as_str());
        assert_eq!(listed[0]["backup_name"], "Test Worker BK-1");

        // The first-ranked backup covers.
        let cover_url = format!("/api/workers/{me}/cover");
        let today: Value = request.get(&cover_url).await.json();
        assert_eq!(today["covered_by"], first.as_str());

        // Restrict the first to a window in the past: the second covers today.
        let first_pid = first_row["pid"].as_str().expect("pid");
        request
            .put(&format!("/api/backups/{first_pid}"))
            .json(&json!({ "starts_on": "2026-01-01", "ends_on": "2026-01-31" }))
            .await
            .assert_status_ok();
        let today: Value = request.get(&cover_url).await.json();
        assert_eq!(today["covered_by"], second.as_str());
        let in_window: Value = request
            .get(&format!("/api/workers/{me}/cover?on=2026-01-15"))
            .await
            .json();
        assert_eq!(in_window["covered_by"], first.as_str());

        // An inverted window is refused.
        assert_eq!(
            request
                .put(&format!("/api/backups/{first_pid}"))
                .json(&json!({ "starts_on": "2026-03-01", "ends_on": "2026-02-01" }))
                .await
                .status_code(),
            422
        );

        // The second leaves: nobody can cover, and the view says so.
        for to in ["offboarding", "terminated"] {
            request
                .post(&format!("/api/workers/{second}/status"))
                .json(&json!({ "to": to }))
                .await
                .assert_status_ok();
        }
        let nobody: Value = request.get(&cover_url).await.json();
        assert!(nobody["covered_by"].is_null());
        assert_eq!(nobody["backups_named"], 2);
    })
    .await;
}
