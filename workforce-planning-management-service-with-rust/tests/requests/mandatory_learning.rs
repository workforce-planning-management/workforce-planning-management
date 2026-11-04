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

        // Retiring takes it off the list and the roll-up; doing it twice keeps the first day.
        let first: Value = request
            .post(&format!("/api/learning-requirements/{req}/retire"))
            .await
            .json();
        let again: Value = request
            .post(&format!("/api/learning-requirements/{req}/retire"))
            .await
            .json();
        assert_eq!(first["retired_on"], again["retired_on"]);
        let list: Value = request
            .get(&format!("/api/learning-requirements?organization={org}"))
            .await
            .json();
        assert!(!list.as_array().unwrap().iter().any(|r| r["pid"] == req));
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn a_manager_sees_their_team_by_name_with_unknown_kept_unknown() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let boss = seed_worker!(&request, &org, format!("MT-M-{tag}"), None).await;
        activate!(&request, &boss).await;
        let direct = seed_worker!(&request, &org, format!("MT-D-{tag}"), None).await;
        activate!(&request, &direct).await;
        let indirect = seed_worker!(&request, &org, format!("MT-I-{tag}"), None).await;
        activate!(&request, &indirect).await;
        let outsider = seed_worker!(&request, &org, format!("MT-O-{tag}"), None).await;
        activate!(&request, &outsider).await;
        for (who, over) in [(&direct, &boss), (&indirect, &direct)] {
            let res = request
                .put(&format!("/api/workers/{who}"))
                .json(&json!({ "manager_pid": over }))
                .await;
            assert_eq!(res.status_code(), 200, "{}", res.text());
        }
        make_nurse(&request, &direct).await;
        let made = request
            .post("/api/learning-requirements")
            .json(&json!({ "organization": org, "subject": format!("Fire {tag}"), "basis": "mandatory",
                "staff_groups": ["nurse"] }))
            .await;
        assert_eq!(made.status_code(), 200, "{}", made.text());

        let team: Value = request
            .get(&format!("/api/workers/{boss}/team-mandatory-learning"))
            .await
            .json();
        let people = team["people"].as_array().unwrap();
        let find = |pid: &str| people.iter().find(|p| p["worker_pid"] == pid).cloned();
        let d = find(&direct).expect("a direct report is listed");
        assert_eq!(d["report_kind"], "direct");
        assert!(d["display_name"].as_str().unwrap().starts_with("Test Worker"));
        let status_of = |p: &Value| {
            p["requirements"]
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["subject"] == format!("Fire {tag}"))
                .map(|r| r["status"].clone())
        };
        assert_eq!(status_of(&d), Some(json!("never_completed")));
        // The indirect report's staff group is not recorded: unknown, said as unknown.
        let i = find(&indirect).expect("an indirect report is listed");
        assert_eq!(i["report_kind"], "indirect");
        assert_eq!(status_of(&i), Some(json!("unknown")));
        assert!(find(&outsider).is_none(), "someone outside the line is not listed");
        assert!(find(&boss).is_none(), "the manager is not their own report");
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn the_roll_up_by_staff_group_applies_the_floor_to_each_group_on_its_own() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        for i in 0..10 {
            let w = seed_worker!(&request, &org, format!("ML-G{i}-{tag}"), None).await;
            activate!(&request, &w).await;
            make_nurse(&request, &w).await;
        }
        for i in 0..3 {
            let w = seed_worker!(&request, &org, format!("ML-U{i}-{tag}"), None).await;
            activate!(&request, &w).await;
        }
        let made = request
            .post("/api/learning-requirements")
            .json(
                &json!({ "organization": org, "subject": format!("Manual handling {tag}"),
                "basis": "mandatory", "refresher_months": 12 }),
            )
            .await;
        assert_eq!(made.status_code(), 200, "{}", made.text());
        let req = made.json::<Value>()["pid"].as_str().unwrap().to_string();

        let url = |by: &str| {
            format!(
                "/api/learning-requirements/rollup?organization={}{by}",
                org.replace(':', "%3A")
            )
        };
        let plain: Value = request.get(&url("")).await.json();
        let plain_cell = plain["requirements"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["requirement_pid"] == req)
            .unwrap();
        assert!(plain_cell.get("groups").is_none(), "no groups unless asked");

        let by: Value = request.get(&url("&by=staff_group")).await.json();
        assert_eq!(by["by"], "staff_group");
        let cell = by["requirements"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["requirement_pid"] == req)
            .unwrap();
        let groups = cell["groups"].as_array().unwrap();
        let nurse = groups
            .iter()
            .find(|g| g["group"] == "nurse")
            .expect("nurse");
        assert_eq!(nurse["hidden"], false, "ten people meet the floor of ten");
        assert_eq!(nurse["never_completed"], 10);
        let unrecorded = groups
            .iter()
            .find(|g| g["group"].is_null())
            .expect("unrecorded");
        assert_eq!(
            unrecorded["hidden"], true,
            "three people are under the floor"
        );
        assert!(
            unrecorded.get("never_completed").is_none(),
            "a hidden group shows no counts"
        );

        let bad = request.get(&url("&by=salary")).await;
        assert_eq!(bad.status_code(), 400, "{}", bad.text());
        let dept: Value = request.get(&url("&by=department")).await.json();
        assert_eq!(dept["by"], "department");
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn a_requirement_may_carry_its_own_due_soon_window() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let nurse = seed_worker!(&request, &org, format!("ML-W-{tag}"), None).await;
        activate!(&request, &nurse).await;
        make_nurse(&request, &nurse).await;
        let add = |subject: &'static str, window: Value| {
            let request = &request;
            let org = org.clone();
            async move {
                let made = request
                    .post("/api/learning-requirements")
                    .json(&json!({ "organization": org, "subject": subject, "basis": "mandatory",
                        "refresher_months": 12, "staff_groups": ["nurse"], "due_soon_days": window }))
                    .await;
                (made.status_code(), made.json::<Value>())
            }
        };
        // Out of range is refused, not clamped.
        assert_eq!(add("Too long", json!(400)).await.0, 422);
        assert_eq!(add("Zero", json!(0)).await.0, 422);
        let (_, short) = add("Short window", json!(30)).await;
        let (_, plain) = add("Deployment window", Value::Null).await;
        let short = short["pid"].as_str().unwrap().to_string();
        let plain = plain["pid"].as_str().unwrap().to_string();
        // Sixty days left on a twelve month refresher: due soon under the default 90, in date under a window of 30.
        let expires = Utc::now().date_naive() + Duration::days(60);
        let completed = expires - chrono::Months::new(12);
        for req in [&short, &plain] {
            let res = request
                .post(&format!("/api/learning-requirements/{req}/completions"))
                .json(&json!({ "worker_pid": nurse, "completed_on": completed.to_string() }))
                .await;
            assert_eq!(res.status_code(), 200, "{}", res.text());
        }
        let mine: Value = request.get(&format!("/api/workers/{nurse}/mandatory-learning")).await.json();
        let status = |req: &str| {
            mine["requirements"]
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["requirement_pid"] == req)
                .unwrap()["status"]
                .clone()
        };
        assert_eq!(status(&short), "compliant", "60 days left is outside its 30 day window");
        assert_eq!(status(&plain), "due_soon", "and inside the deployment's 90");
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn a_position_audience_matches_the_number_of_the_seat_held_and_otherwise_says_unknown() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let mut people = Vec::new();
        for n in 0..3 {
            let w = seed_worker!(&request, &org, format!("ML-P{n}-{tag}"), None).await;
            activate!(&request, &w).await;
            people.push(w);
        }
        let seat = |number: Option<&'static str>| {
            let request = &request;
            let org = org.clone();
            async move {
                request
                    .post("/api/positions")
                    .json(&json!({ "organization_ref": org, "cost_centre": "CC-ML", "title": "Nurse", "budget_fte_percent": 100,
                        "funding_kind": "core", "number": number }))
                    .await
            }
        };
        let first = seat(Some("N-001")).await;
        assert_eq!(first.status_code(), 200, "{}", first.text());
        let first = first.json::<Value>()["pid"].as_str().unwrap().to_string();
        let second = seat(Some("N-002")).await.json::<Value>()["pid"].as_str().unwrap().to_string();
        // A number names one seat in an organization.
        let again = seat(Some(" N-001 ")).await;
        assert_eq!(again.status_code(), 422, "{}", again.text());
        // Blank is no number, and two seats may both have none.
        assert_eq!(seat(Some("  ")).await.status_code(), 200);
        assert_eq!(seat(None).await.status_code(), 200);

        for (seat_pid, who) in [(&first, &people[0]), (&second, &people[1])] {
            let approved = request
                .post(&format!("/api/positions/{seat_pid}/status"))
                .json(&json!({ "to": "approved" }))
                .await;
            assert_eq!(approved.status_code(), 200, "{}", approved.text());
            let held = request
                .put(&format!("/api/positions/{seat_pid}/holder"))
                .json(&json!({ "worker_pid": who }))
                .await;
            assert_eq!(held.status_code(), 200, "{}", held.text());
        }
        let made = request
            .post("/api/learning-requirements")
            .json(&json!({ "organization": org, "subject": "Ward safety", "basis": "mandatory", "positions": ["N-001"] }))
            .await;
        assert_eq!(made.status_code(), 200, "{}", made.text());
        let req = made.json::<Value>()["pid"].as_str().unwrap().to_string();
        let status_of = |who: &str| {
            let request = &request;
            let url = format!("/api/workers/{who}/mandatory-learning");
            let req = req.clone();
            async move {
                let mine: Value = request.get(&url).await.json();
                mine["requirements"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|r| r["requirement_pid"] == req.as_str())
                    .map(|r| r["status"].clone())
            }
        };
        assert_eq!(status_of(&people[0]).await, Some(json!("never_completed")), "holds N-001");
        // Holding N-002, a seat that is not in the audience, is not applicable (and not listed for the person).
        assert_ne!(status_of(&people[1]).await, Some(json!("never_completed")));
        // Holding no seat: nobody recorded where they are, so the answer is unknown, never "not applicable".
        assert_eq!(status_of(&people[2]).await, Some(json!("unknown")));
    })
    .await;
}
