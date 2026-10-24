//! The last routes with no request test: removals (a backup, a pool member, a plan's demand line
//! and tolerance), restating a group and a wellbeing rule, and the per-worker and per-application
//! reads. Each pins the happy path, that it is gone afterwards, and that a stranger is a 404.

use serde_json::{Value, json};
use serial_test::serial;

use super::{a_person, activate, an_org, seed_worker};

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn a_backup_a_pool_member_a_demand_line_and_a_tolerance_can_each_be_removed_once() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let ghost = uuid::Uuid::new_v4();

        // A backup: named, then no longer named; a second removal is a 404.
        let me = seed_worker!(&request, &org, format!("RM-{tag}-A"), None).await;
        let cover = seed_worker!(&request, &org, format!("RM-{tag}-B"), None).await;
        for pid in [&me, &cover] {
            activate!(&request, pid).await;
        }
        let backup: Value = request
            .post(&format!("/api/workers/{me}/backups"))
            .json(&json!({ "backup_pid": cover }))
            .await
            .json();
        let backup = backup["pid"].as_str().unwrap().to_string();
        request.delete(&format!("/api/backups/{backup}")).await.assert_status_ok();
        let left: Vec<Value> = request.get(&format!("/api/workers/{me}/backups")).await.json();
        assert!(left.iter().all(|b| b["pid"] != backup.as_str()));
        assert_eq!(request.delete(&format!("/api/backups/{backup}")).await.status_code(), 404);
        assert_eq!(request.delete(&format!("/api/backups/{ghost}")).await.status_code(), 404);

        // A skill-pool member.
        let pool: Value = request
            .post("/api/skill-pools")
            .json(&json!({ "organization_ref": org, "name": format!("Pool {tag}") }))
            .await
            .json();
        let pool = pool["pid"].as_str().unwrap().to_string();
        let skill: Value = request
            .post("/api/skills")
            .json(&json!({ "name": format!("Pool skill {tag}"), "category": "technical" }))
            .await
            .json();
        let member: Value = request
            .post(&format!("/api/skill-pools/{pool}/members"))
            .json(&json!({ "skill_pid": skill["pid"], "min_proficiency": 3 }))
            .await
            .json();
        let member = member["pid"].as_str().unwrap().to_string();
        let url = format!("/api/skill-pools/{pool}/members/{member}");
        request.delete(&url).await.assert_status_ok();
        assert_eq!(request.delete(&url).await.status_code(), 404, "already removed");
        assert_eq!(request.delete(&format!("/api/skill-pools/{pool}/members/{ghost}")).await.status_code(), 404);

        // A plan's demand line and a tolerance.
        let plan: Value = request
            .post("/api/workforce-plans")
            .json(&json!({
                "name": format!("Plan {tag}"), "organization_ref": org,
                "horizon_start": "2026-01-01", "horizon_end": "2027-12-31",
                "attrition_bp": 1000, "budget_minor": 10_000_000, "budget_currency": "GBP",
            }))
            .await
            .json();
        let plan = plan["pid"].as_str().unwrap().to_string();
        let line: Value = request
            .put(&format!("/api/workforce-plans/{plan}/demand-lines"))
            .json(&json!({ "department": "engineering", "target_on": "2026-06-30", "target_headcount": 10 }))
            .await
            .json();
        let line = line["pid"].as_str().expect("the line is answered with its pid").to_string();
        let line_url = format!("/api/workforce-plans/{plan}/demand-lines/{line}");
        request.delete(&line_url).await.assert_status_ok();
        assert_eq!(request.delete(&line_url).await.status_code(), 404);

        request
            .put(&format!("/api/workforce-plans/{plan}/tolerances/cost"))
            .json(&json!({ "limit": 20_000 }))
            .await
            .assert_status_ok();
        let tolerance = format!("/api/workforce-plans/{plan}/tolerances/cost");
        request.delete(&tolerance).await.assert_status_ok();
        assert_eq!(request.delete(&tolerance).await.status_code(), 404, "none left to remove");
        assert_eq!(
            request.delete(&format!("/api/workforce-plans/{plan}/tolerances/mood")).await.status_code(),
            422,
            "an unknown dimension is not a 404"
        );
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn a_group_and_a_wellbeing_rule_can_be_restated() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();

        let make = |name: String| request.post("/api/groups").json(&json!({ "organization_ref": org, "name": name, "kind": "practice", "description": "d" }));
        let first: Value = make(format!("Guild {tag}")).await.json();
        let second: Value = make(format!("Club {tag}")).await.json();
        let (first, second) = (first["pid"].as_str().unwrap(), second["pid"].as_str().unwrap());
        let url = format!("/api/groups/{first}");
        request
            .put(&url)
            .json(&json!({ "name": format!("Renamed {tag}"), "kind": "interest", "description": "new" }))
            .await
            .assert_status_ok();
        let listed: Value = request.get("/api/groups").await.json();
        let row = listed.as_array().unwrap().iter().find(|g| g["pid"] == first).unwrap();
        assert_eq!(row["name"], format!("Renamed {tag}"));
        assert_eq!(row["kind"], "interest");
        assert_eq!(
            request.put(&url).json(&json!({ "name": format!("club {tag}"), "kind": "practice" })).await.status_code(),
            422,
            "another group already has that name"
        );
        assert_eq!(request.put(&format!("/api/groups/{second}")).json(&json!({ "name": "x", "kind": "club" })).await.status_code(), 422);
        assert_eq!(
            request.put(&format!("/api/groups/{}", uuid::Uuid::new_v4())).json(&json!({ "name": "x", "kind": "practice" })).await.status_code(),
            404
        );

        let rule: Value = request
            .post("/api/wellbeing-entitlements")
            .json(&json!({ "name": format!("Flu {tag}"), "description": "Free jab", "doses": 1 }))
            .await
            .json();
        let rule_url = format!("/api/wellbeing-entitlements/{}", rule["pid"].as_str().unwrap());
        request
            .put(&rule_url)
            .json(&json!({ "name": format!("Flu {tag}"), "description": "Free jab, two doses", "doses": 2 }))
            .await
            .assert_status_ok();
        let all: Value = request.get("/api/wellbeing-entitlements").await.json();
        let restated = all.as_array().unwrap().iter().find(|r| r["pid"] == rule["pid"]).unwrap();
        assert_eq!(restated["doses"], 2);
        assert_eq!(request.put(&rule_url).json(&json!({ "name": "Bad", "description": "x", "doses": 0 })).await.status_code(), 422);
        assert_eq!(
            request
                .put(&format!("/api/wellbeing-entitlements/{}", uuid::Uuid::new_v4()))
                .json(&json!({ "name": "Gone", "description": "x" }))
                .await
                .status_code(),
            404
        );
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn a_workers_appraisals_movements_and_onboarding_and_an_applications_reads_answer() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let worker = seed_worker!(&request, &org, format!("RD-{tag}"), None).await;

        for kind in ["appraisals", "movements", "onboarding"] {
            let response = request.get(&format!("/api/workers/{worker}/{kind}")).await;
            assert_eq!(response.status_code(), 200, "{kind}");
            assert!(response.json::<Value>().is_array(), "{kind} is a list");
            assert_eq!(
                request.get(&format!("/api/workers/{}/{kind}", uuid::Uuid::new_v4())).await.status_code(),
                404,
                "{kind} for a stranger"
            );
        }

        let requisition: Value = request
            .post("/api/requisitions")
            .json(&json!({ "organization_ref": org, "department": "engineering", "job_title": format!("Role {tag}"), "headcount": 1, "salary_min_minor": 3_000_000, "salary_max_minor": 4_000_000, "salary_currency": "GBP" }))
            .await
            .json();
        let requisition = requisition["pid"].as_str().unwrap().to_string();
        request.post(&format!("/api/requisitions/{requisition}/status")).json(&json!({ "to": "open" })).await.assert_status_ok();
        let candidate: Value = request
            .post("/api/candidates")
            .json(&json!({ "display_name": format!("Candidate {tag}"), "email": format!("{tag}@example.com"), "source": "web", "person_ref": a_person() }))
            .await
            .json();
        let application: Value = request
            .post(&format!("/api/requisitions/{requisition}/applications"))
            .json(&json!({ "candidate_pid": candidate["pid"] }))
            .await
            .json();
        let application = application["pid"].as_str().unwrap().to_string();

        let listed: Value = request.get(&format!("/api/requisitions/{requisition}/applications")).await.json();
        assert!(listed.as_array().unwrap().iter().any(|a| a["pid"] == application.as_str()));
        for tail in ["interviews", "assessments"] {
            let response = request.get(&format!("/api/applications/{application}/{tail}")).await;
            assert_eq!(response.status_code(), 200, "{tail}");
            assert_eq!(
                request.get(&format!("/api/applications/{}/{tail}", uuid::Uuid::new_v4())).await.status_code(),
                404,
                "{tail} for a stranger"
            );
        }
        let profile = request.get(&format!("/api/candidates/{}/assessment-profile", candidate["pid"].as_str().unwrap())).await;
        assert_eq!(profile.status_code(), 200);
        assert_eq!(
            request.get(&format!("/api/candidates/{}/assessment-profile", uuid::Uuid::new_v4())).await.status_code(),
            404
        );
    })
    .await;
}
