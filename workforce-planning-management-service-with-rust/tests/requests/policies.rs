//! Policies and acknowledgements (WPM-R156, WPM-D105): versions by reference, an audience, the newest version in force, a worker's own
//! acknowledgement, a manager's counts with the floor, and HR's named record.

use chrono::{Duration, Utc};
use sea_orm::{ConnectionTrait, Statement};
use serde_json::{Value, json};
use serial_test::serial;
use workforce_planning_management::tasks::policy_reminders::send_reminders;

use super::{a_person, an_org};

async fn hire(
    request: &loco_rs::TestServer,
    org: &str,
    department: &str,
    manager: Option<&str>,
) -> String {
    let response = request
        .post("/api/workers")
        .json(&json!({
            "person_ref": a_person(), "organization_ref": org,
            "worker_number": format!("PO-{}", uuid::Uuid::new_v4().simple()),
            "display_name": "Policy Person", "employment_type": "permanent",
            "department": department, "job_title": "Carer", "hired_on": "2020-01-05",
            "fte_percent": 100, "manager_pid": manager,
        }))
        .await;
    assert_eq!(response.status_code(), 200, "{}", response.text());
    response.json::<Value>()["pid"]
        .as_str()
        .unwrap()
        .to_string()
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn a_version_is_acknowledged_by_its_audience_counted_for_managers_and_named_for_hr() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        // An acknowledgement is made through /api/me, which needs a signed-in person (the enforcement test does that);
        // here the rows are written directly so the counts and the named record can be read.
        let acknowledge = |who: String, version: String| {
            let ctx = &ctx;
            async move {
                ctx.db
                    .execute_unprepared(&format!(
                        "INSERT INTO policy_acknowledgements (pid, version_pid, worker_pid, acknowledged_on) VALUES ('{}', '{version}', '{who}', CURRENT_DATE)",
                        uuid::Uuid::new_v4()
                    ))
                    .await
                    .unwrap();
            }
        };
        let boss = hire(&request, &org, "management", None).await;
        let mut care = Vec::new();
        for _ in 0..10 {
            care.push(hire(&request, &org, "care", Some(&boss)).await);
        }
        let legal = hire(&request, &org, "legal", Some(&boss)).await;

        let policy: Value = request.post("/api/policies").json(&json!({ "organization_ref": org, "name": "Code of conduct" })).await.json();
        let policy = policy["pid"].as_str().unwrap().to_string();
        assert_eq!(request.post("/api/policies").json(&json!({ "organization_ref": org, "name": "Code of conduct" })).await.status_code(), 422);

        let versions = format!("/api/policies/{policy}/versions");
        let yesterday = (Utc::now().date_naive() - Duration::days(1)).to_string();
        for bad in [
            json!({ "title": "", "document_ref": "doc:c1", "effective_on": yesterday, "audience": "all" }),
            json!({ "title": "T", "document_ref": "has a space", "effective_on": yesterday, "audience": "all" }),
            json!({ "title": "T", "document_ref": "doc:c1", "effective_on": yesterday, "audience": "department" }),
            json!({ "title": "T", "document_ref": "doc:c1", "effective_on": yesterday, "audience": "all", "audience_value": "care" }),
            json!({ "title": "T", "document_ref": "doc:c1", "effective_on": yesterday, "audience": "all", "acknowledge_within_days": 0 }),
        ] {
            assert_eq!(request.post(&versions).json(&bad).await.status_code(), 422, "{bad}");
        }
        let v1: Value = request
            .post(&versions)
            .json(&json!({ "title": "Code of conduct", "document_ref": "doc:conduct-1", "effective_on": yesterday, "audience": "department", "audience_value": "Care" }))
            .await
            .json();
        assert_eq!(v1["version"], 1);
        let v1 = v1["pid"].as_str().unwrap().to_string();

        // A care worker is asked; a legal worker is not (not in the audience).
        let mine: Value = request.get(&format!("/api/workers/{}/policies", care[0])).await.json();
        assert_eq!(mine["waiting"], 1);
        assert_eq!(mine["policies"][0]["standing"]["kind"], "waiting");
        let theirs: Value = request.get(&format!("/api/workers/{legal}/policies")).await.json();
        assert_eq!(theirs["waiting"], 0);
        for who in &care[..4] {
            acknowledge(who.clone(), v1.clone()).await;
        }
        let counts: Value = request.get(&format!("/api/workers/{boss}/team-policy-counts?version_pid={v1}")).await.json();
        assert_eq!(counts["count"], json!({ "kind": "shown", "acknowledged": 4, "waiting": 6 }));
        assert!(!counts.to_string().contains(&care[0]), "a count names no one");

        // A new version in force replaces the old for the asking; the old one can no longer be acknowledged, and the count starts again.
        let v2: Value = request
            .post(&versions)
            .json(&json!({ "title": "Code of conduct", "document_ref": "doc:conduct-2", "effective_on": yesterday, "audience": "department", "audience_value": "care" }))
            .await
            .json();
        assert_eq!(v2["version"], 2);
        let v2 = v2["pid"].as_str().unwrap().to_string();
        let mine: Value = request.get(&format!("/api/workers/{}/policies", care[0])).await.json();
        assert_eq!(mine["waiting"], 1, "version 2 needs a new acknowledgement");
        assert_eq!(mine["policies"][0]["version"]["version"], 2);
        acknowledge(care[0].clone(), v2.clone()).await;

        // A version aimed at three people shows a manager nothing: the team in it is under the floor.
        let v3: Value = request
            .post(&format!("/api/policies/{policy}/versions"))
            .json(&json!({ "title": "Legal hold", "document_ref": "doc:hold", "effective_on": yesterday, "audience": "department", "audience_value": "legal" }))
            .await
            .json();
        let small: Value = request.get(&format!("/api/workers/{boss}/team-policy-counts?version_pid={}", v3["pid"].as_str().unwrap())).await.json();
        assert_eq!(small["count"]["kind"], "withheld");

        // The list shows each version by reference, with no text.
        let listed: Value = request.get(&format!("/api/policies?organization={org}")).await.json();
        assert_eq!(listed["policies"][0]["versions"].as_array().map(Vec::len), Some(3));
        assert_eq!(listed["policies"][0]["versions"][0]["document_ref"], "doc:conduct-1");

        // HR's named record lists the audience with who acknowledged version 2, and is audited.
        let named: Value = request.get(&format!("/api/policy-versions/{v2}/acknowledgements")).await.json();
        let people = named["people"].as_array().unwrap();
        assert_eq!(people.len(), 10);
        assert_eq!(people.iter().filter(|p| !p["acknowledged_on"].is_null()).count(), 1);
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn a_policy_is_renamed_without_touching_its_versions_and_workers_are_reminded_once() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let today = Utc::now().date_naive();
        let make = |name: &'static str| {
            let request = &request;
            let org = org.clone();
            async move {
                let made: Value = request.post("/api/policies").json(&json!({ "organization_ref": org, "name": name })).await.json();
                made["pid"].as_str().unwrap().to_string()
            }
        };
        let conduct = make("Conduct").await;
        let travel = make("Travel").await;

        // A rename: a clash, an empty name, a name too long and an unknown policy are refused; an unchanged name is not a clash.
        let url = format!("/api/policies/{conduct}");
        assert_eq!(request.patch(&url).json(&json!({ "name": "Travel" })).await.status_code(), 422, "another policy has that name");
        assert_eq!(request.patch(&url).json(&json!({ "name": "  " })).await.status_code(), 422);
        assert_eq!(request.patch(&url).json(&json!({ "name": "x".repeat(121) })).await.status_code(), 422);
        assert_eq!(request.patch(&format!("/api/policies/{}", uuid::Uuid::new_v4())).json(&json!({ "name": "Q" })).await.status_code(), 404);
        assert_eq!(request.patch(&url).json(&json!({ "name": "Conduct" })).await.status_code(), 200, "its own name is not a clash");
        let version = request
            .post(&format!("/api/policies/{conduct}/versions"))
            .json(&json!({ "title": "Conduct", "document_ref": "doc:conduct", "effective_on": (today - Duration::days(1)).to_string(), "audience": "all", "acknowledge_within_days": 5 }))
            .await;
        assert_eq!(version.status_code(), 200, "{}", version.text());
        let renamed = request.patch(&url).json(&json!({ "name": "  Code of conduct " })).await;
        assert_eq!(renamed.status_code(), 200, "{}", renamed.text());
        assert_eq!(renamed.json::<Value>()["name"], "Code of conduct", "trimmed");
        let listed: Value = request.get(&format!("/api/policies?organization={org}")).await.json();
        let row = listed["policies"].as_array().unwrap().iter().find(|p| p["pid"] == conduct.as_str()).unwrap();
        assert_eq!(row["name"], "Code of conduct");
        assert_eq!(row["versions"][0]["title"], "Conduct", "a version keeps its own title");

        // No audit, no write (WPM-D107).
        crate::requests::ddl(&ctx, "CREATE FUNCTION refuse_audit() RETURNS trigger AS $$ BEGIN RAISE EXCEPTION 'audit refused'; END $$ LANGUAGE plpgsql;
                 CREATE TRIGGER refuse_audit BEFORE INSERT ON audit_logs FOR EACH ROW EXECUTE FUNCTION refuse_audit()",).await;
        let refused = request.patch(&url).json(&json!({ "name": "Refused" })).await;
        assert!(refused.status_code().is_server_error(), "refused: {}", refused.status_code());
        crate::requests::ddl(&ctx, "DROP TRIGGER refuse_audit ON audit_logs; DROP FUNCTION refuse_audit()").await;
        let kept: Value = request.get(&format!("/api/policies?organization={org}")).await.json();
        assert!(kept["policies"].as_array().unwrap().iter().any(|p| p["name"] == "Code of conduct"), "nothing was kept");
        let _ = travel;

        // Reminders: due in 4 calendar days (effective yesterday + 5), so a window of 3 sends nothing and a window of 4 sends one each.
        let a = hire(&request, &org, "care", None).await;
        let b = hire(&request, &org, "care", None).await;
        let told = |worker: String| {
            let ctx = &ctx;
            async move {
                let row = ctx
                    .db
                    .query_one_raw(Statement::from_string(
                        ctx.db.get_database_backend(),
                        format!("SELECT count(*) FROM notifications WHERE worker_pid = '{worker}' AND kind = 'policy_due'"),
                    ))
                    .await
                    .unwrap()
                    .unwrap();
                row.try_get_by_index::<i64>(0).unwrap()
            }
        };
        send_reminders(&ctx, today, 3).await.unwrap();
        assert_eq!(told(a.clone()).await, 0, "four calendar days left is outside a window of three");
        send_reminders(&ctx, today, 4).await.unwrap();
        assert_eq!(told(a.clone()).await, 1);
        assert_eq!(told(b.clone()).await, 1);
        assert_eq!(send_reminders(&ctx, today, 4).await.unwrap(), 0, "idempotent: nobody is told twice for a version");

        // Someone who has acknowledged is left alone; a worker hired later is not told as of an earlier day.
        let c = hire(&request, &org, "care", None).await;
        let version_pid = listed_version(&request, &org, &conduct).await;
        ctx.db
            .execute_unprepared(&format!(
                "INSERT INTO policy_acknowledgements (pid, version_pid, worker_pid, acknowledged_on) VALUES ('{}', '{version_pid}', '{c}', CURRENT_DATE)",
                uuid::Uuid::new_v4()
            ))
            .await
            .unwrap();
        send_reminders(&ctx, today, 30).await.unwrap();
        assert_eq!(told(c).await, 0, "already acknowledged");
        let early = send_reminders(&ctx, today - Duration::days(400), 30).await.unwrap();
        assert_eq!(early, 0, "no version was in force 400 days ago, and nobody was employed");
    })
    .await;
}

async fn listed_version(request: &loco_rs::TestServer, org: &str, policy: &str) -> String {
    let listed: Value = request
        .get(&format!("/api/policies?organization={org}"))
        .await
        .json();
    let row = listed["policies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["pid"] == policy)
        .unwrap();
    row["versions"][0]["pid"].as_str().unwrap().to_string()
}
