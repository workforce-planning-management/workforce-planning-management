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

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn the_information_pack_gives_floored_counts_hides_what_could_be_subtracted_and_logs_the_disclosure()
 {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let dept = format!("pk-{tag}");
        let small = format!("pk-small-{tag}");
        let hire = |number: String, department: String, kind: &'static str| {
            let request = &request;
            let org = org.clone();
            async move {
                let mut worker = json!({
                    "person_ref": crate::requests::a_person(), "organization_ref": org, "worker_number": number,
                    "display_name": "Pack Person", "employment_type": kind, "department": department,
                    "job_title": "Carer", "hired_on": "2020-01-05",
                });
                if kind != "permanent" {
                    worker["engagement_ends_on"] = json!("2035-01-01");
                }
                let made = request.post("/api/workers").json(&worker).await;
                assert_eq!(made.status_code(), 200, "{}", made.text());
                made.json::<Value>()["pid"].as_str().unwrap().to_string()
            }
        };
        // 22 permanent and 3 contractors in one department; 4 people in another.
        let mut permanent = Vec::new();
        for n in 0..22 {
            permanent.push(hire(format!("PK-{tag}-P{n}"), dept.clone(), "permanent").await);
        }
        for n in 0..3 {
            hire(format!("PK-{tag}-C{n}"), dept.clone(), "contractor").await;
        }
        for n in 0..4 {
            hire(format!("PK-{tag}-S{n}"), small.clone(), "permanent").await;
        }
        // Twelve of the permanent staff sit in band 5.
        for pid in permanent.iter().take(12) {
            ctx.db
                .execute_unprepared(&format!("INSERT INTO worker_pay_positions (worker_pid, scale_id, band, step, step_since) VALUES ('{pid}', 'national-2026-27', 'B5', 1, '2024-01-01')"))
                .await
                .unwrap();
        }
        // Two open posts in the department.
        for n in 0..2 {
            let made: Value = request
                .post("/api/requisitions")
                .json(&json!({ "organization_ref": org, "department": dept, "job_title": format!("Carer {n}"), "headcount": 2, "salary_min_minor": 2_000_000, "salary_max_minor": 3_000_000, "salary_currency": "GBP" }))
                .await
                .json();
            let status = request.post(&format!("/api/requisitions/{}/status", made["pid"].as_str().unwrap())).json(&json!({ "to": "open" })).await;
            assert_eq!(status.status_code(), 200, "{}", status.text());
        }

        let body: Value = request
            .post("/api/recognised-bodies")
            .json(&json!({ "organization_ref": org, "name": format!("Pack Union {tag}"), "kind": "union", "basis": "statutory", "recognised_on": "2019-03-01", "contact_role": "Head of Employee Relations" }))
            .await
            .json();
        let units = format!("/api/recognised-bodies/{}/units", body["pid"].as_str().unwrap());
        let whole: Value = request.post(&units).json(&json!({ "name": "Whole department", "department": dept })).await.json();
        let tiny: Value = request.post(&units).json(&json!({ "name": "Small department", "department": small })).await.json();
        let located: Value = request.post(&units).json(&json!({ "name": "By location", "department": dept, "location": "Leeds" })).await.json();
        let pack_of = |unit: &Value| format!("/api/bargaining-units/{}/information-pack", unit["pid"].as_str().unwrap());

        let full: Value = request.get(&pack_of(&whole)).await.json();
        assert_eq!(full["pack"]["headcount"], 25);
        let by = |section: &str, group: &str| full["pack"][section].as_array().unwrap().iter().find(|g| g["group"] == group).unwrap().clone();
        assert_eq!(by("by_department", &dept)["headcount"], json!({ "kind": "shown", "count": 25 }));
        assert_eq!(by("by_department", &dept)["fte_hundredths"], 2500);
        // Three contractors are under the floor; the 22 permanent would give the three away, so they are hidden too.
        assert_eq!(by("by_employment_type", "contractor")["headcount"], json!({ "kind": "withheld" }));
        assert_eq!(by("by_employment_type", "permanent")["headcount"], json!({ "kind": "withheld" }), "hidden so the total does not give the three away");
        // Twelve in band 5 and thirteen with none recorded: both shown, and none is called zero.
        assert_eq!(by("by_pay_band", "B5")["headcount"], json!({ "kind": "shown", "count": 12 }));
        assert_eq!(by("by_pay_band", "not_recorded")["headcount"], json!({ "kind": "shown", "count": 13 }));
        assert_eq!(full["vacancies"], json!({ "kind": "shown", "open_posts": 4 }));
        assert_eq!(full["pack"]["starters_last_year"], json!({ "kind": "withheld" }), "nobody started in the last year, which is under the floor, not a nought");

        // A unit of four people gives nothing; a unit defined by location cannot be matched to vacancies and says so.
        let none: Value = request.get(&pack_of(&tiny)).await.json();
        assert!(none["pack"]["headcount"].is_null(), "{none}");
        assert_eq!(none["pack"]["by_department"], json!([]));
        let by_location: Value = request.get(&pack_of(&located)).await.json();
        assert_eq!(by_location["vacancies"]["kind"], "unknown");

        // Nothing names a worker, a number, a pay figure or the body.
        let text = json!({ "pack": full["pack"], "vacancies": full["vacancies"] }).to_string();
        for banned in [format!("PK-{tag}"), "Pack Person".to_string(), "salary".to_string(), format!("Pack Union {tag}")] {
            assert!(!text.contains(&banned), "no {banned}: {text}");
        }
        for who in &permanent {
            assert!(!text.contains(who.as_str()));
        }

        // Each read is a disclosure, logged against the unit with who and when.
        let logged = ctx
            .db
            .query_one_raw(sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                format!("SELECT count(*) AS n FROM audit_logs WHERE action = 'information_pack_disclosed' AND entity_pid = '{}'::uuid", whole["pid"].as_str().unwrap()),
            ))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(logged.try_get::<i64>("", "n").unwrap(), 1);
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn facility_time_is_minutes_per_body_per_period_and_never_a_person() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let body: Value = request
            .post("/api/recognised-bodies")
            .json(&json!({ "organization_ref": org, "name": format!("Facility Union {tag}"), "kind": "union", "basis": "voluntary_agreement", "recognised_on": "2019-03-01", "contact_role": "Head of Employee Relations" }))
            .await
            .json();
        let url = format!("/api/recognised-bodies/{}/facility-time", body["pid"].as_str().unwrap());
        let year = |from: &str, to: &str, minutes: i64, budget: Option<i64>| json!({ "period_start": from, "period_end": to, "minutes": minutes, "budget_minutes": budget });

        for bad in [
            year("2026-04-01", "2026-03-31", 10, None),
            year("2026-01-01", "2027-01-02", 10, None),
            year("2026-04-01", "2026-06-30", -5, None),
            year("2026-04-01", "2026-06-30", 10, Some(7_000_000)),
        ] {
            assert_eq!(request.post(&url).json(&bad).await.status_code(), 422, "{bad}");
        }
        assert_eq!(request.post(&url).json(&year("2025-04-01", "2026-03-31", 120_000, Some(150_000))).await.status_code(), 200);
        assert_eq!(request.post(&url).json(&year("2026-04-01", "2027-03-31", 90_000, Some(60_000))).await.status_code(), 200);
        assert_eq!(request.post(&url).json(&year("2027-04-01", "2028-03-31", 1_000, None)).await.status_code(), 200);
        assert_eq!(request.post(&url).json(&year("2026-03-31", "2026-04-30", 5, None)).await.status_code(), 422, "overlaps both neighbours");
        assert_eq!(request.post("/api/recognised-bodies/00000000-0000-4000-8000-000000000000/facility-time").json(&year("2025-04-01", "2026-03-31", 1, None)).await.status_code(), 404);

        let read: Value = request.get(&url).await.json();
        let periods = read["periods"].as_array().unwrap();
        assert_eq!(periods.len(), 3);
        assert_eq!(periods[0]["period_start"], "2027-04-01", "newest first");
        assert_eq!(periods[0]["over_budget"], Value::Null, "no budget agreed: unknown, not within");
        assert_eq!(periods[1]["over_budget"], true);
        assert_eq!(periods[2]["over_budget"], false);
        assert_eq!(periods[2]["hours"], 2000.0);
        assert_eq!(read["total_minutes"], 211_000);
        let text = read["periods"].to_string().to_lowercase();
        for banned in ["worker", "person", "representative", "name"] {
            assert!(!text.contains(banned), "no {banned} in a period: {text}");
        }
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn a_decision_gate_points_a_change_at_the_body_to_consult_and_asks_for_the_reference_without_blocking()
 {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let body: Value = request
            .post("/api/recognised-bodies")
            .json(&json!({ "organization_ref": org, "name": format!("Gate Council {tag}"), "kind": "works_council", "basis": "statutory", "recognised_on": "2019-03-01", "contact_role": "Head of Employee Relations" }))
            .await
            .json();
        let body_pid = body["pid"].as_str().unwrap().to_string();

        // Bad gates: an unknown change, a body that is not the organization's.
        assert_eq!(request.post("/api/decision-gates").json(&json!({ "organization_ref": org, "change_kind": "coffee", "body_pid": body_pid })).await.status_code(), 422);
        assert_eq!(request.post("/api/decision-gates").json(&json!({ "organization_ref": "organization:00000000-0000-4000-8000-0000000000aa", "change_kind": "new_monitoring", "body_pid": body_pid })).await.status_code(), 422);
        let gate = json!({ "organization_ref": org, "change_kind": "new_or_changed_score", "body_pid": body_pid, "note": "Scores that rank or advise on people" });
        // A previous run may have left this gate in the shared test database; either way one exists after this.
        let first = request.post("/api/decision-gates").json(&gate).await.status_code();
        assert!(first == 200 || first == 422, "{first}");
        assert_eq!(request.post("/api/decision-gates").json(&gate).await.status_code(), 422, "one gate per change and body");

        let listed: Value = request.get(&format!("/api/decision-gates?organization={org}")).await.json();
        assert!(listed["changes"].as_array().unwrap().iter().any(|c| c == "new_or_changed_score"));
        assert!(listed["gates"].as_array().unwrap().iter().any(|g| g["body_pid"] == body_pid.as_str()));

        // A change with no gate is not pointed anywhere; one with a gate asks for the reference.
        let ungated: Value = request.get(&format!("/api/decision-gates/check?organization={org}&change=shift_pattern_change")).await.json();
        assert_eq!(ungated["standing"], json!({ "kind": "no_gate" }));
        let asked: Value = request.get(&format!("/api/decision-gates/check?organization={org}&change=new_or_changed_score")).await.json();
        assert_eq!(asked["standing"]["kind"], "reference_needed");
        assert!(asked["equality_impact_template"].as_str().unwrap().contains("section 11"));
        assert!(asked["consult"].as_array().unwrap().iter().any(|c| c["body_pid"] == body_pid.as_str()));
        assert_eq!(request.get(&format!("/api/decision-gates/check?organization={org}&change=coffee")).await.status_code(), 422);

        // A reference must be a real consultation record; then the standing is "given".
        let stray = "00000000-0000-4000-8000-000000000000";
        assert_eq!(request.get(&format!("/api/decision-gates/check?organization={org}&change=new_or_changed_score&consultation_pid={stray}")).await.status_code(), 422);
        let consultation: Value = request
            .post("/api/consultations")
            .json(&json!({ "body_pid": body_pid, "establishment": "Whole organization", "profile": "great_britain", "establishment_size": 100, "dismissals": 1, "earliest_termination_on": "2030-01-31", "topics": ["other"] }))
            .await
            .json();
        let consultation_pid = consultation["pid"].as_str().unwrap().to_string();
        let given: Value = request.get(&format!("/api/decision-gates/check?organization={org}&change=new_or_changed_score&consultation_pid={consultation_pid}")).await.json();
        assert_eq!(given["standing"], json!({ "kind": "reference_given" }));

        // The change that is gated: recording a scoring feature. It is never refused for want of a reference; it is pointed at the gate.
        let record = |extra: Value| {
            let request = &request;
            async move {
                let mut payload = json!({ "overseer_role": "Head of Resourcing" });
                if let (Some(map), Some(more)) = (payload.as_object_mut(), extra.as_object()) {
                    map.extend(more.clone());
                }
                request.put("/api/ai-register/S4").json(&payload).await
            }
        };
        let without = record(json!({})).await;
        assert_eq!(without.status_code(), 200, "advice, not a block: {}", without.text());
        let pointed = without.json::<Value>()["decision_gate"].clone();
        assert_eq!(pointed["standing"]["kind"], "reference_needed");
        assert_eq!(pointed["change"], "new_or_changed_score", "a score is pointed at the score gate, not another");
        assert!(pointed["consult"].as_array().unwrap().iter().any(|c| c["body_pid"] == body_pid.as_str()), "{pointed}");
        assert!(pointed["equality_impact_template"].as_str().unwrap().contains("section 11"));
        let with = record(json!({ "consultation_pid": consultation_pid })).await;
        assert_eq!(with.status_code(), 200);
        assert_eq!(with.json::<Value>()["decision_gate"]["standing"], json!({ "kind": "reference_given" }));
        assert_eq!(record(json!({ "consultation_pid": stray })).await.status_code(), 422, "a reference that is no record is refused");
    })
    .await;
}
