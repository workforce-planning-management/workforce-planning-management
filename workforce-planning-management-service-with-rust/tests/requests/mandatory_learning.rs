//! Statutory and mandatory learning (WPM-R170, WPM-D117, WPM-T315): requirements HR adds, completions HR records, the person's status,
//! `unknown` for a category nobody recorded, a future date refused, and a roll-up that hides a small group and names no one.

use chrono::{Duration, Utc};
use serde_json::{Value, json};
use serial_test::serial;

use super::{activate, an_org, seed_worker};

async fn make_nurse(request: &loco_rs::TestServer, worker: &str) {
    let list: Value = request
        .get(&format!("/api/workers/{worker}/assignments"))
        .await
        .json();
    let primary = list["assignments"][0]["pid"].as_str().unwrap().to_string();
    let res = request
        .post(&format!("/api/assignments/{primary}/change"))
        .json(&json!({ "staff_group": "nurse" }))
        .await;
    assert_eq!(res.status_code(), 200, "{}", res.text());
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn a_requirement_is_checked_against_recorded_completions_and_unknown_is_never_compliant() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let nurse = seed_worker!(&request, &org, format!("ML-A-{tag}"), None).await;
        activate!(&request, &nurse).await;
        make_nurse(&request, &nurse).await;
        let unrecorded = seed_worker!(&request, &org, format!("ML-B-{tag}"), None).await;
        activate!(&request, &unrecorded).await;

        // A statutory requirement must name its source; a refresher of zero is refused.
        let base = json!({ "organization": org, "subject": "Fire safety", "basis": "statutory",
            "refresher_months": 12, "staff_groups": ["nurse"] });
        let mut no_source = base.clone();
        no_source["source"] = Value::Null;
        assert_eq!(
            request.post("/api/learning-requirements").json(&no_source).await.status_code(),
            422
        );
        let mut zero = base.clone();
        zero["source"] = json!("Regulatory Reform (Fire Safety) Order 2005");
        zero["refresher_months"] = json!(0);
        assert_eq!(
            request.post("/api/learning-requirements").json(&zero).await.status_code(),
            422
        );
        let mut ok = base.clone();
        ok["source"] = json!("Regulatory Reform (Fire Safety) Order 2005");
        let made = request.post("/api/learning-requirements").json(&ok).await;
        assert_eq!(made.status_code(), 200, "{}", made.text());
        let req = made.json::<Value>()["pid"].as_str().unwrap().to_string();

        let status_of = |who: String| {
            let request = &request;
            let req = req.clone();
            async move {
                let v: Value = request
                    .get(&format!("/api/workers/{who}/mandatory-learning"))
                    .await
                    .json();
                v["requirements"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|r| r["requirement_pid"] == req)
                    .cloned()
                    .unwrap()
            }
        };
        assert_eq!(status_of(nurse.clone()).await["status"], "never_completed");
        // The second worker's staff group is not recorded, so the audience cannot be matched: unknown, not compliant, not missing.
        assert_eq!(status_of(unrecorded.clone()).await["status"], "unknown");

        let completions = format!("/api/learning-requirements/{req}/completions");
        let today = Utc::now().date_naive();
        let future = request
            .post(&completions)
            .json(&json!({ "worker_pid": nurse, "completed_on": today + Duration::days(1) }))
            .await;
        assert_eq!(future.status_code(), 422);
        // 400 days ago is past a 12-month refresher; the latest completion is the one that counts.
        let old = request
            .post(&completions)
            .json(&json!({ "worker_pid": nurse, "completed_on": today - Duration::days(400), "source": "Provider" }))
            .await;
        assert_eq!(old.status_code(), 200, "{}", old.text());
        assert_eq!(status_of(nurse.clone()).await["status"], "expired");
        let recent = request
            .post(&completions)
            .json(&json!({ "worker_pid": nurse, "completed_on": today - Duration::days(30) }))
            .await;
        assert_eq!(recent.status_code(), 200);
        let now = status_of(nurse.clone()).await;
        assert_eq!(now["status"], "compliant");
        assert_eq!(now["last_completed_on"], json!(today - Duration::days(30)));
        assert!(now["expires_on"].is_string());

        // The requirement is listed, and the roll-up for a group under the floor shows nothing and names no one.
        let list: Value = request
            .get(&format!("/api/learning-requirements?organization={org}"))
            .await
            .json();
        assert!(list.as_array().unwrap().iter().any(|r| r["pid"] == req));
        let roll = request
            .get(&format!("/api/learning-requirements/rollup?organization={org}"))
            .await;
        assert_eq!(roll.status_code(), 200);
        let roll: Value = roll.json();
        let cell = roll["requirements"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["requirement_pid"] == req)
            .unwrap();
        assert_eq!(cell["hidden"], true, "{cell}");
        assert!(!roll.to_string().contains(&nurse), "a roll-up names no one");
    })
    .await;
}
