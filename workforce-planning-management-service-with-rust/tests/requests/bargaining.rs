//! Recognised bodies and bargaining units (WPM-R102, WPM-D68): the rules over department, location, employment type and job title; the
//! count with a floor; and that nothing says who is covered or who belongs.

use sea_orm::ConnectionTrait;
use serde_json::{Value, json};
use serial_test::serial;

use super::an_org;

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn a_unit_is_counted_by_its_rules_from_the_floor_and_names_no_one() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let dept = format!("bu-{tag}");
        let hire = |number: String, department: String, location: Option<&'static str>, title: &'static str, ended: Option<&'static str>| {
            let request = &request;
            let org = org.clone();
            async move {
                let made = request
                    .post("/api/workers")
                    .json(&json!({
                        "person_ref": crate::requests::a_person(), "organization_ref": org, "worker_number": number,
                        "display_name": "Unit Person", "employment_type": "permanent", "department": department,
                        "job_title": title, "location": location, "hired_on": "2020-01-05",
                    }))
                    .await;
                assert_eq!(made.status_code(), 200, "{}", made.text());
                let pid = made.json::<Value>()["pid"].as_str().unwrap().to_string();
                (pid, ended)
            }
        };
        // Twelve carers in Leeds (one of whom leaves), three in York, and one nurse in Leeds.
        let mut leeds = Vec::new();
        for n in 0..12 {
            leeds.push(hire(format!("BU-{tag}-L{n}"), dept.clone(), Some("Leeds"), "Carer", None).await.0);
        }
        for n in 0..3 {
            hire(format!("BU-{tag}-Y{n}"), dept.clone(), Some("York"), "Carer", None).await;
        }
        hire(format!("BU-{tag}-N"), dept.clone(), Some("Leeds"), "Nurse", None).await;

        // Bad bodies: unknown kind, no contact role, a duplicate name.
        let body = |name: &str| json!({ "organization_ref": org, "name": name, "kind": "union", "basis": "statutory", "recognised_on": "2019-03-01", "contact_role": "Head of Employee Relations" });
        for bad in [
            json!({ "organization_ref": org, "name": "X", "kind": "club", "basis": "statutory", "recognised_on": "2019-03-01", "contact_role": "HR" }),
            json!({ "organization_ref": org, "name": "X", "kind": "union", "basis": "statutory", "recognised_on": "2019-03-01", "contact_role": " " }),
        ] {
            assert_eq!(request.post("/api/recognised-bodies").json(&bad).await.status_code(), 422, "{bad}");
        }
        let made = request.post("/api/recognised-bodies").json(&body(&format!("Care Union {tag}"))).await;
        assert_eq!(made.status_code(), 200, "{}", made.text());
        let body_pid = made.json::<Value>()["pid"].as_str().unwrap().to_string();
        assert_eq!(request.post("/api/recognised-bodies").json(&body(&format!("Care Union {tag}"))).await.status_code(), 422, "a name is unique in the organization");

        // Bad units: no rule, a duplicate name.
        let units = format!("/api/recognised-bodies/{body_pid}/units");
        assert_eq!(request.post(&units).json(&json!({ "name": "None" })).await.status_code(), 422, "a unit needs a rule");
        assert_eq!(request.post("/api/recognised-bodies/00000000-0000-4000-8000-000000000000/units").json(&json!({ "name": "U", "department": "x" })).await.status_code(), 404);
        for unit in [
            json!({ "name": "Leeds carers", "department": dept.to_uppercase(), "location": " leeds ", "job_title": "carer" }),
            json!({ "name": "York carers", "department": dept, "location": "York" }),
            json!({ "name": "Nurses", "department": dept, "job_title": "Nurse" }),
            json!({ "name": "Nobody", "department": dept, "location": "Hull" }),
            json!({ "name": "Whole department", "department": dept }),
        ] {
            let added = request.post(&units).json(&unit).await;
            assert_eq!(added.status_code(), 200, "{unit}: {}", added.text());
        }
        assert_eq!(request.post(&units).json(&json!({ "name": "Nurses", "department": dept })).await.status_code(), 422, "a unit name is unique in its body");

        let read = || {
            let request = &request;
            let units = units.clone();
            async move {
                let v: Value = request.get(&units).await.json();
                v["units"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|u| (u["name"].as_str().unwrap().to_string(), u["coverage"].clone()))
                    .collect::<std::collections::BTreeMap<_, _>>()
            }
        };
        let cover = read().await;
        assert_eq!(cover["Leeds carers"], json!({ "kind": "shown", "covered": 12 }), "case and edge spaces do not matter");
        assert_eq!(cover["York carers"], json!({ "kind": "withheld" }), "three is under the floor");
        assert_eq!(cover["Nurses"], json!({ "kind": "withheld" }), "one is under the floor");
        assert_eq!(cover["Nobody"], json!({ "kind": "no_one" }));
        assert_eq!(cover["Whole department"], json!({ "kind": "shown", "covered": 16 }));

        // A leaver is no longer covered; a deleted record is not either.
        ctx.db
            .execute_unprepared(&format!("UPDATE workers SET terminated_on = '2020-06-01' WHERE pid = '{}'::uuid", leeds[0]))
            .await
            .unwrap();
        ctx.db
            .execute_unprepared(&format!("UPDATE workers SET deleted_at = now() WHERE pid = '{}'::uuid", leeds[1]))
            .await
            .unwrap();
        let cover = read().await;
        assert_eq!(cover["Leeds carers"], json!({ "kind": "shown", "covered": 10 }));
        assert_eq!(cover["Whole department"], json!({ "kind": "shown", "covered": 14 }));

        // The answers name no worker and hold nothing about membership.
        let text = request.get(&units).await.text();
        for who in &leeds {
            assert!(!text.contains(who.as_str()), "coverage names no one");
        }
        assert!(!text.contains("Unit Person") && !text.contains(&format!("BU-{tag}")), "no name or number: {text}");
        let list: Value = request.get(&format!("/api/recognised-bodies?organization={org}")).await.json();
        let ours = list["bodies"].as_array().unwrap().iter().find(|b| b["pid"] == body_pid.as_str()).unwrap();
        assert_eq!(ours["units"], 5);
        assert_eq!(ours["contact_role"], "Head of Employee Relations");
        assert!(ours.get("recorded_by").is_none());
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn agreements_are_versioned_and_apply_to_a_worker_by_their_job_facts_without_naming_a_body() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let dept = format!("ag-{tag}");
        let hire = |number: String, title: &'static str| {
            let request = &request;
            let org = org.clone();
            let dept = dept.clone();
            async move {
                let made = request
                    .post("/api/workers")
                    .json(&json!({
                        "person_ref": crate::requests::a_person(), "organization_ref": org, "worker_number": number,
                        "display_name": "Agreement Person", "employment_type": "permanent", "department": dept,
                        "job_title": title, "hired_on": "2020-01-05",
                    }))
                    .await;
                assert_eq!(made.status_code(), 200, "{}", made.text());
                made.json::<Value>()["pid"].as_str().unwrap().to_string()
            }
        };
        let carer = hire(format!("AG-{tag}-C"), "Carer").await;
        let nurse = hire(format!("AG-{tag}-N"), "Nurse").await;

        let body: Value = request
            .post("/api/recognised-bodies")
            .json(&json!({ "organization_ref": org, "name": format!("Agreement Forum {tag}"), "kind": "works_council", "basis": "statutory", "recognised_on": "2019-03-01", "contact_role": "Head of Employee Relations" }))
            .await
            .json();
        let units = format!("/api/recognised-bodies/{}/units", body["pid"].as_str().unwrap());
        let unit: Value = request.post(&units).json(&json!({ "name": "Carers", "department": dept, "job_title": "Carer" })).await.json();
        let agreements = format!("/api/bargaining-units/{}/agreements", unit["pid"].as_str().unwrap());
        assert_eq!(request.post("/api/bargaining-units/00000000-0000-4000-8000-000000000000/agreements").json(&json!({ "title": "T", "source": "s", "effective_from": "2024-04-01" })).await.status_code(), 404);

        // Bad: no source, an unknown pay scale, an hours limit out of range, an end before the start.
        let ok = json!({ "title": "Pay and conditions", "source": "Agreement of 2024-04-01, schedule 2", "effective_from": "2024-04-01", "effective_to": "2026-03-31", "pay_scale_id": "national-2026-27", "max_weekly_hours": 40, "notice_weeks": 4, "consultation_days": 60 });
        for (field, value) in [("source", json!(" ")), ("pay_scale_id", json!("made-up")), ("max_weekly_hours", json!(200)), ("effective_to", json!("2024-03-31"))] {
            let mut bad = ok.clone();
            bad[field] = value;
            assert_eq!(request.post(&agreements).json(&bad).await.status_code(), 422, "{field}");
        }
        let first = request.post(&agreements).json(&ok).await;
        assert_eq!(first.status_code(), 200, "{}", first.text());
        assert_eq!(first.json::<Value>()["version"], 1);
        let mut same_day = ok.clone();
        same_day["effective_to"] = Value::Null;
        assert_eq!(request.post(&agreements).json(&same_day).await.status_code(), 422, "a new version must take effect after the newest");
        let mut second = ok.clone();
        second["effective_from"] = json!("2026-04-01");
        second["effective_to"] = Value::Null;
        second["max_weekly_hours"] = json!(37);
        assert_eq!(request.post(&agreements).json(&second).await.json::<Value>()["version"], 2);
        // A second agreement that has lapsed.
        let lapsed = json!({ "title": "Old allowance", "source": "Side letter 2019", "effective_from": "2019-01-01", "effective_to": "2020-12-31" });
        assert_eq!(request.post(&agreements).json(&lapsed).await.status_code(), 200);

        let listed: Value = request.get(&agreements).await.json();
        let rows = listed["agreements"].as_array().unwrap();
        assert_eq!(rows.len(), 3);
        let state: Vec<(String, i64, bool)> = rows.iter().map(|r| (r["title"].as_str().unwrap().to_string(), r["version"].as_i64().unwrap(), r["in_force"].as_bool().unwrap())).collect();
        assert_eq!(state, [("Old allowance".to_string(), 1, false), ("Pay and conditions".to_string(), 1, false), ("Pay and conditions".to_string(), 2, true)]);

        // The carer's job facts match the unit: the version in force today, with its terms, and no body, unit or membership named.
        let now: Value = request.get(&format!("/api/workers/{carer}/collective-agreements")).await.json();
        let applying = now["agreements"].as_array().unwrap();
        assert_eq!(applying.len(), 1, "{now}");
        assert_eq!((applying[0]["version"].as_i64(), applying[0]["terms"]["max_weekly_hours"].as_i64()), (Some(2), Some(37)));
        let text = now.to_string();
        for named in [format!("Agreement Forum {tag}"), "Carers".to_string(), body["pid"].as_str().unwrap().to_string(), unit["pid"].as_str().unwrap().to_string(), "member".to_string()] {
            assert!(!text.to_lowercase().contains(&named.to_lowercase()), "names no {named}: {text}");
        }
        // On an earlier day it is the earlier version; before any, none; and the nurse is never covered.
        let earlier: Value = request.get(&format!("/api/workers/{carer}/collective-agreements?on=2025-01-01")).await.json();
        assert_eq!(earlier["agreements"][0]["version"], 1);
        let before: Value = request.get(&format!("/api/workers/{carer}/collective-agreements?on=2023-01-01")).await.json();
        assert_eq!(before["agreements"].as_array().unwrap().len(), 0);
        let none: Value = request.get(&format!("/api/workers/{nurse}/collective-agreements")).await.json();
        assert_eq!(none["agreements"].as_array().unwrap().len(), 0);
    })
    .await;
}
