//! A diagnosis has no column (WPM-R118, WPM-D73): a sick-leave request records that it is sick
//! leave and its dates, never why.

use serde_json::{Value, json};
use serial_test::serial;

use super::{activate, an_org, seed_worker};

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn sick_leave_takes_no_reason_and_other_kinds_keep_theirs() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let worker = seed_worker!(&request, &org, "HD-1", Some(3_600_000)).await;
        activate!(&request, &worker).await;
        for kind in ["sick", "annual"] {
            request
                .post(&format!("/api/workers/{worker}/leave-entitlements"))
                .json(&json!({ "kind": kind, "year": 2027, "entitled_days": 30 }))
                .await
                .assert_status_ok();
        }
        let url = format!("/api/workers/{worker}/leave-requests");
        // A reason on sick leave is refused, in any wording.
        for reason in ["chemotherapy", "migraine", " flu "] {
            let refused = request
                .post(&url)
                .json(&json!({ "kind": "sick", "start_on": "2027-03-01", "end_on": "2027-03-02", "reason": reason }))
                .await;
            assert_eq!(refused.status_code(), 422, "{reason}");
            assert!(refused.text().contains("never why"));
        }
        // Without one, or with a blank one, it is accepted and stores none.
        for body in [
            json!({ "kind": "sick", "start_on": "2027-03-01", "end_on": "2027-03-02" }),
            json!({ "kind": "sick", "start_on": "2027-03-08", "end_on": "2027-03-09", "reason": "  " }),
        ] {
            request.post(&url).json(&body).await.assert_status_ok();
        }
        // Another kind keeps its reason.
        request
            .post(&url)
            .json(&json!({ "kind": "annual", "start_on": "2027-04-01", "end_on": "2027-04-02", "reason": "family holiday" }))
            .await
            .assert_status_ok();
        let listed: Vec<Value> = request.get(&url).await.json();
        let text = serde_json::to_string(&listed).unwrap();
        assert!(!text.contains("chemotherapy") && !text.contains("migraine") && !text.contains("flu"));
        assert!(text.contains("family holiday"));
        let sick: Vec<&Value> = listed.iter().filter(|r| r["kind"] == "sick").collect();
        assert_eq!(sick.len(), 2);
        assert!(sick.iter().all(|r| r["reason"].as_str().is_none_or(|s| s.trim().is_empty())));
    })
    .await;
}
