//! Equipment issued and returned (WPM-R155): assets, one holder at a time, returns and write-offs, the leaver's record held open
//! while equipment is out, the subject export, and erasure (the asset stays).

use chrono::{Duration, Utc};
use sea_orm::{ConnectionTrait, Statement};
use serde_json::{Value, json};
use serial_test::serial;

use super::{activate, an_org, seed_worker};

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn equipment_is_issued_to_one_holder_returned_or_written_off_and_holds_a_leaver_open() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let a = seed_worker!(&request, &org, format!("EQ-{tag}-A"), None).await;
        activate!(&request, &a).await;
        let b = seed_worker!(&request, &org, format!("EQ-{tag}-B"), None).await;
        activate!(&request, &b).await;

        // Bad assets: unknown kind, empty tag, no owner role.
        for bad in [
            json!({ "organization_ref": org, "kind": "drone", "tag": "X", "owner_role": "IT" }),
            json!({ "organization_ref": org, "kind": "laptop", "tag": " ", "owner_role": "IT" }),
            json!({ "organization_ref": org, "kind": "laptop", "tag": "X", "owner_role": "" }),
        ] {
            assert_eq!(request.post("/api/assets").json(&bad).await.status_code(), 422, "{bad}");
        }
        let laptop: Value = request
            .post("/api/assets")
            .json(&json!({ "organization_ref": org, "kind": "laptop", "tag": format!("LT-{tag}"), "owner_role": "IT" }))
            .await
            .json();
        let laptop = laptop["pid"].as_str().unwrap().to_string();
        let again = request
            .post("/api/assets")
            .json(&json!({ "organization_ref": org, "kind": "phone", "tag": format!("LT-{tag}"), "owner_role": "IT" }))
            .await;
        assert_eq!(again.status_code(), 422, "a tag is unique in an organization");

        // Issue: not in the future, not to a stranger; one holder at a time.
        let issue_url = format!("/api/assets/{laptop}/issue");
        let future = (Utc::now().date_naive() + Duration::days(2)).to_string();
        assert_eq!(request.post(&issue_url).json(&json!({ "worker_pid": a, "issued_on": future })).await.status_code(), 422);
        assert_eq!(request.post(&issue_url).json(&json!({ "worker_pid": uuid::Uuid::new_v4() })).await.status_code(), 422);
        let issued: Value = request.post(&issue_url).json(&json!({ "worker_pid": a })).await.json();
        let issue = issued["pid"].as_str().unwrap().to_string();
        assert_eq!(request.post(&issue_url).json(&json!({ "worker_pid": b })).await.status_code(), 422, "already out");

        let held: Value = request.get(&format!("/api/workers/{a}/equipment")).await.json();
        assert_eq!(held["out"], 1);
        assert_eq!(held["issues"][0]["tag"], format!("LT-{tag}"));
        let listed: Value = request.get(&format!("/api/assets?organization={org}")).await.json();
        assert_eq!(listed["assets"][0]["out"], true);
        assert!(!listed.to_string().contains(&a), "the list does not say who holds it");

        // A leaver's record cannot be completed while the laptop is out.
        let last_day = (Utc::now().date_naive() + Duration::days(14)).to_string();
        let movement: Value = request
            .post(&format!("/api/workers/{a}/movements"))
            .json(&json!({ "kind": "leaver", "effective_on": last_day, "reason": "resignation" }))
            .await
            .json();
        let mv = movement["pid"].as_str().unwrap().to_string();
        let refused = request.post(&format!("/api/movements/{mv}/complete")).await;
        assert_eq!(refused.status_code(), 422);
        assert!(refused.text().contains("equipment still out"), "{}", refused.text());

        // A write-off needs a reason; a return cannot predate the issue.
        let close = format!("/api/asset-issues/{issue}/return");
        assert_eq!(request.post(&close).json(&json!({ "outcome": "written_off" })).await.status_code(), 422);
        assert_eq!(request.post(&close).json(&json!({ "outcome": "returned", "returned_on": "2000-01-01" })).await.status_code(), 422);
        assert_eq!(request.post(&close).json(&json!({ "outcome": "lost" })).await.status_code(), 422);
        let done = request.post(&close).json(&json!({ "outcome": "written_off", "note": "lost on a train" })).await;
        assert_eq!(done.status_code(), 200, "{}", done.text());
        assert_eq!(request.post(&close).json(&json!({ "outcome": "returned" })).await.status_code(), 422, "already closed");
        let after = request.post(&format!("/api/movements/{mv}/complete")).await;
        assert!(!after.text().contains("equipment still out"), "nothing is out now: {}", after.text());

        // The asset can be issued again once it is back (written off or not, it is the employer's record).
        assert_eq!(request.post(&issue_url).json(&json!({ "worker_pid": b })).await.status_code(), 200);

        // The subject export lists the issues; erasure removes the issues and keeps the asset.
        let export: Value = request.get(&format!("/api/workers/{a}/subject-access")).await.json();
        assert_eq!(export["asset_issues"].as_array().map(Vec::len), Some(1), "{export}");
        for to in ["offboarding", "terminated"] {
            request.post(&format!("/api/workers/{a}/status")).json(&json!({ "to": to })).await.assert_status_ok();
        }
        request.post(&format!("/api/workers/{a}/erase")).await.assert_status_ok();
        let row = ctx
            .db
            .query_one_raw(Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                format!("SELECT (SELECT count(*) FROM asset_issues WHERE worker_pid = '{a}') AS issues, (SELECT count(*) FROM assets WHERE pid = '{laptop}'::uuid) AS assets"),
            ))
            .await
            .unwrap()
            .unwrap();
        let issues: i64 = row.try_get("", "issues").unwrap();
        let assets: i64 = row.try_get("", "assets").unwrap();
        assert_eq!((issues, assets), (0, 1), "the issues are erased and the asset stays");
    })
    .await;
}
