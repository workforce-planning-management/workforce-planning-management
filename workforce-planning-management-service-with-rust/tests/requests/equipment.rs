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

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn a_manager_sees_how_many_of_their_team_hold_equipment_only_from_the_floor() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let hire = |number: String, manager: Option<String>| {
            let request = &request;
            let org = org.clone();
            async move {
                let made = request
                    .post("/api/workers")
                    .json(&json!({
                        "person_ref": crate::requests::a_person(), "organization_ref": org, "worker_number": number,
                        "display_name": "Team Person", "employment_type": "permanent", "department": "care",
                        "job_title": "Carer", "hired_on": "2020-01-05", "manager_pid": manager,
                    }))
                    .await;
                assert_eq!(made.status_code(), 200, "{}", made.text());
                made.json::<Value>()["pid"].as_str().unwrap().to_string()
            }
        };
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let boss = hire(format!("TE-{tag}-M"), None).await;
        let mut team = Vec::new();
        for n in 0..10 {
            team.push(hire(format!("TE-{tag}-{n}"), Some(boss.clone())).await);
        }
        let small_boss = hire(format!("TE-{tag}-S"), None).await;
        for n in 0..3 {
            hire(format!("TE-{tag}-S{n}"), Some(small_boss.clone())).await;
        }

        let count = |who: String| {
            let request = &request;
            async move { request.get(&format!("/api/workers/{who}/team-equipment-count")).await.json::<Value>() }
        };
        assert_eq!(count(boss.clone()).await["count"], json!({ "kind": "shown", "holding": 0, "none": 10 }));

        // Three people hold a laptop; one of them also holds a phone, which does not make them two people.
        let mut issues = Vec::new();
        for (n, (who, kind)) in [(0, "laptop"), (1, "laptop"), (2, "laptop"), (2, "phone")].iter().map(|(i, k)| (&team[*i], *k)).enumerate() {
            let asset: Value = request
                .post("/api/assets")
                .json(&json!({ "organization_ref": org, "kind": kind, "tag": format!("TE-{tag}-{n}"), "owner_role": "IT" }))
                .await
                .json();
            let issued: Value = request
                .post(&format!("/api/assets/{}/issue", asset["pid"].as_str().unwrap()))
                .json(&json!({ "worker_pid": who }))
                .await
                .json();
            issues.push(issued["pid"].as_str().unwrap().to_string());
        }
        let shown = count(boss.clone()).await;
        assert_eq!(shown["count"], json!({ "kind": "shown", "holding": 3, "none": 7 }), "{shown}");
        let text = shown.to_string();
        assert!(!text.contains("laptop") && !text.contains("phone") && !text.contains(&format!("TE-{tag}")), "a count names no item: {text}");
        for who in &team {
            assert!(!text.contains(who.as_str()), "a count names no one");
        }

        // Returned equipment stops counting; a person who returns one of two items still holds the other.
        let closed = request
            .post(&format!("/api/asset-issues/{}/return", issues[3]))
            .json(&json!({ "outcome": "returned" }))
            .await;
        assert_eq!(closed.status_code(), 200, "{}", closed.text());
        assert_eq!(count(boss.clone()).await["count"]["holding"], 3);
        let closed = request
            .post(&format!("/api/asset-issues/{}/return", issues[0]))
            .json(&json!({ "outcome": "returned" }))
            .await;
        assert_eq!(closed.status_code(), 200, "{}", closed.text());
        assert_eq!(count(boss.clone()).await["count"]["holding"], 2);

        // Three reports are under the floor: nothing, whoever holds what; and a worker with no team has no one.
        assert_eq!(count(small_boss).await["count"]["kind"], "withheld");
        assert_eq!(count(team[5].clone()).await["count"]["kind"], "no_one");

        // A report who has left, or whose record is deleted, is not on the team: twelve reports, one holds, then two go.
        let lead = hire(format!("TE-{tag}-L"), None).await;
        let mut crew = Vec::new();
        for n in 0..12 {
            crew.push(hire(format!("TE-{tag}-C{n}"), Some(lead.clone())).await);
        }
        let asset: Value = request
            .post("/api/assets")
            .json(&json!({ "organization_ref": org, "kind": "laptop", "tag": format!("TE-{tag}-C"), "owner_role": "IT" }))
            .await
            .json();
        request
            .post(&format!("/api/assets/{}/issue", asset["pid"].as_str().unwrap()))
            .json(&json!({ "worker_pid": crew[0] }))
            .await
            .assert_status_ok();
        assert_eq!(count(lead.clone()).await["count"], json!({ "kind": "shown", "holding": 1, "none": 11 }));
        ctx.db
            .execute_unprepared(&format!("UPDATE workers SET terminated_on = '2020-06-01' WHERE pid = '{}'::uuid", crew[1]))
            .await
            .unwrap();
        ctx.db
            .execute_unprepared(&format!("UPDATE workers SET deleted_at = now() WHERE pid = '{}'::uuid", crew[2]))
            .await
            .unwrap();
        assert_eq!(count(lead).await["count"], json!({ "kind": "shown", "holding": 1, "none": 9 }));
    })
    .await;
}
