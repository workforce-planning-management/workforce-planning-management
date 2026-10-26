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
