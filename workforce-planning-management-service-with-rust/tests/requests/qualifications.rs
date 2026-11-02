//! Qualifications and the qualification gap (WPM-R171, WPM-T316): declared is never met, HR verifies with an evidence reference, a
//! role's requirements are read beside the person's list, and no role recorded reads `role: null`.

use chrono::{Duration, Utc};
use serde_json::{Value, json};
use serial_test::serial;

use super::{activate, an_org, seed_worker};

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn a_declared_qualification_is_not_met_until_hr_verifies_it() {
    use workforce_planning_management::rules::framework::LevelMapping;
    use workforce_planning_management::tasks::import_framework::import_pcf;

    crate::requests::request_open(|request, ctx| async move {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
        import_pcf(&ctx.db, &root.join("pcf-mini"), LevelMapping::Identity, false)
            .await
            .expect("pcf");
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let me = seed_worker!(&request, &org, format!("Q-A-{tag}"), None).await;
        activate!(&request, &me).await;
        let quals = format!("/api/workers/{me}/qualifications");

        // No role recorded: the role is null and the gap is empty, never a clean pass.
        let none: Value = request.get(&quals).await.json();
        assert!(none["role"].is_null());
        assert_eq!(none["gap"].as_array().unwrap().len(), 0);

        let profiles: Value = request.get("/api/role-profiles?framework=uk-gdad-pcf").await.json();
        let tester = profiles.as_array().unwrap().iter().find(|p| p["job_title"] == "Tester").unwrap()["pid"]
            .as_str()
            .unwrap()
            .to_string();
        request
            .put(&format!("/api/workers/{me}/framework-roles/uk-gdad-pcf"))
            .json(&json!({ "role_profile_pid": tester }))
            .await
            .assert_status_ok();
        let reqs = format!("/api/role-profiles/{tester}/qualification-requirements");
        // A framework and its level go together; an unknown kind is refused.
        assert_eq!(
            request.post(&reqs).json(&json!({ "title": "Degree", "framework": "RQF" })).await.status_code(),
            422
        );
        assert_eq!(
            request.post(&reqs).json(&json!({ "title": "Degree", "kind": "hobby" })).await.status_code(),
            422
        );
        request
            .post(&reqs)
            .json(&json!({ "title": "Degree", "kind": "academic", "framework": "RQF", "min_level": 6 }))
            .await
            .assert_status_ok();

        let listed: Value = request.get(&reqs).await.json();
        assert_eq!(listed["requirements"].as_array().unwrap().len(), 1);
        assert_eq!(listed["requirements"][0]["min_level"], 6);

        let standing = |v: &Value| v["gap"][0]["standing"].as_str().unwrap().to_string();
        let seen: Value = request.get(&quals).await.json();
        assert_eq!(standing(&seen), "missing");

        // Declared: shown, not met. A level below the minimum does not match.
        let low = request
            .post(&quals)
            .json(&json!({ "kind": "academic", "title": "Degree", "framework": "RQF", "level": 5 }))
            .await;
        assert_eq!(low.status_code(), 200, "{}", low.text());
        assert_eq!(standing(&request.get(&quals).await.json::<Value>()), "missing");
        let made = request
            .post(&quals)
            .json(&json!({ "kind": "academic", "title": "degree ", "framework": "RQF", "level": 6, "awarded_on": "2015-06-30" }))
            .await;
        assert_eq!(made.status_code(), 200, "{}", made.text());
        let qpid = made.json::<Value>()["pid"].as_str().unwrap().to_string();
        let declared: Value = request.get(&quals).await.json();
        assert_eq!(standing(&declared), "declared_only");
        assert_eq!(declared["role"]["job_title"], "Tester");

        // Refusals: expiry before award, a future award, framework without level.
        for bad in [
            json!({ "kind": "licence", "title": "X", "awarded_on": "2020-01-01", "expires_on": "2019-01-01" }),
            json!({ "kind": "licence", "title": "X", "awarded_on": (Utc::now().date_naive() + Duration::days(5)).to_string() }),
            json!({ "kind": "licence", "title": "X", "framework": "RQF" }),
            json!({ "kind": "nope", "title": "X" }),
        ] {
            assert_eq!(request.post(&quals).json(&bad).await.status_code(), 422, "{bad}");
        }

        // Verification needs an evidence reference; then it is met. Verifying twice changes nothing.
        let verify = format!("/api/qualifications/{qpid}/verify");
        assert_eq!(request.post(&verify).json(&json!({ "evidence_ref": "  " })).await.status_code(), 422);
        request.post(&verify).json(&json!({ "evidence_ref": "HR file 2015/88" })).await.assert_status_ok();
        request.post(&verify).json(&json!({ "evidence_ref": "elsewhere" })).await.assert_status_ok();
        let met: Value = request.get(&quals).await.json();
        assert_eq!(standing(&met), "met");
        let row = met["qualifications"].as_array().unwrap().iter().find(|q| q["pid"] == qpid.as_str()).unwrap().clone();
        assert_eq!(row["status"], "verified");
        assert_eq!(row["evidence_ref"], "HR file 2015/88");

        // Removing the verified one leaves the gap open again.
        request.delete(&format!("/api/qualifications/{qpid}")).await.assert_status_ok();
        assert_eq!(standing(&request.get(&quals).await.json::<Value>()), "missing");

        // The export carries them.
        let export: Value = request.get(&format!("/api/workers/{me}/subject-access")).await.json();
        assert!(export["qualifications"].is_array(), "{export}");
    })
    .await;
}
