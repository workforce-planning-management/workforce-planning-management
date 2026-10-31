//! Jurisdiction profiles (WPM-D144, WPM-T380): codes, append-only parameter rows, the worker's `work_jurisdiction`, and the
//! working-time view resolving each worker's limits from them. The codes are ISO 3166 user-assigned ones, so no country is named.

use sea_orm::ConnectionTrait;
use serde_json::{Value, json};
use serial_test::serial;

use super::{activate, an_org, seed_worker, working_time_profile};

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn a_profile_is_recorded_append_only_and_refuses_what_does_not_fit() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let code = |code: &str, layer: &str, parent: Option<&str>| {
            json!({ "organization_ref": org, "code": code, "layer": layer, "parent_code": parent })
        };
        for body in [
            code("QZ", "supranational", None),
            code("XA", "national", Some("QZ")),
            code("XA-01", "subnational", Some("XA")),
        ] {
            request.post("/api/jurisdictions").json(&body).await.assert_status_ok();
        }
        for (body, why) in [
            (code("xa", "national", None), "lower case"),
            (code("XA-01", "national", None), "a subdivision is subnational"),
            (code("XB", "subnational", None), "a country is not subnational"),
            (code("XB", "regional", None), "no such layer"),
            (code("XB", "national", Some("XB")), "its own parent"),
            (code("QZ", "supranational", Some("XA-01")), "a bloc under a state"),
            (code("XA", "national", Some("XA")), "a cycle of one"),
        ] {
            assert_eq!(request.post("/api/jurisdictions").json(&body).await.status_code(), 422, "{why}");
        }
        let row = |key: &str, value: Value, status: &str| {
            json!({ "organization_ref": org, "code": "XA", "key": key, "value": value,
                    "effective_from": "2026-01-01", "source": "synthetic", "status": status })
        };
        let added: Value = request
            .post("/api/jurisdiction-parameters")
            .json(&row("holidays.weekend_days", json!([6, 5]), "unverified"))
            .await
            .json();
        assert_eq!(added["value"], json!([5, 6]), "stored sorted");
        for (body, why) in [
            (row("no.such", json!(1), "example"), "not in the registry"),
            (row("working_time.reference_weeks", json!(0), "example"), "out of range"),
            (row("working_time.reference_weeks", json!(17), "read"), "read without checked_on"),
            (row("working_time.reference_weeks", json!(17), "verified"), "no such status"),
        ] {
            assert_eq!(request.post("/api/jurisdiction-parameters").json(&body).await.status_code(), 422, "{why}");
        }
        let mut backwards = row("working_time.reference_weeks", json!(17), "example");
        backwards["effective_to"] = json!("2025-12-31");
        assert_eq!(request.post("/api/jurisdiction-parameters").json(&backwards).await.status_code(), 422);
        let mut unsourced = row("working_time.reference_weeks", json!(17), "example");
        unsourced["source"] = json!("  ");
        assert_eq!(request.post("/api/jurisdiction-parameters").json(&unsourced).await.status_code(), 422);
        let mut read = row("working_time.reference_weeks", json!(17), "read");
        read["checked_on"] = json!("2026-10-31");
        request.post("/api/jurisdiction-parameters").json(&read).await.assert_status_ok();

        let profile: Value = request.get(&format!("/api/jurisdiction-profiles?organization={org}")).await.json();
        assert_eq!(profile["jurisdictions"].as_array().unwrap().len(), 3);
        assert_eq!(profile["parameters"].as_array().unwrap().len(), 2);
        assert!(profile["registry"].as_array().unwrap().iter().any(|e| e["key"] == "working_time.reference_weeks"));
        assert_eq!(request.get("/api/jurisdiction-profiles?organization=nope").await.status_code(), 422);

        // Append-only: the database refuses to change or remove a row.
        let pid = added["pid"].as_str().unwrap();
        for sql in [
            format!("UPDATE jurisdiction_parameters SET source = 'x' WHERE pid = '{pid}'"),
            format!("DELETE FROM jurisdiction_parameters WHERE pid = '{pid}'"),
        ] {
            assert!(ctx.db.execute_unprepared(&sql).await.is_err(), "{sql}");
        }
        // The audit names the code and the key, not the value.
        let audit = serde_json::to_string(&request.get(&format!("/api/audits/{pid}")).await.json::<Value>()).unwrap();
        assert!(audit.contains("jurisdiction_parameter_added") && audit.contains("holidays.weekend_days"), "{audit}");
        assert!(!audit.contains("[5,6]"), "{audit}");

        // Where the work is done: a code, or blank to clear.
        let worker = seed_worker!(&request, &org, "JU-1", None).await;
        let set = |value: &str| json!({ "work_jurisdiction": value });
        assert_eq!(request.put(&format!("/api/workers/{worker}")).json(&set("x-1")).await.status_code(), 422);
        let row: Value = request.put(&format!("/api/workers/{worker}")).json(&set("XA-01")).await.json();
        assert_eq!(row["work_jurisdiction"], "XA-01");
        let row: Value = request.put(&format!("/api/workers/{worker}")).json(&set(" ")).await.json();
        assert_eq!(row["work_jurisdiction"], Value::Null);
    })
    .await;
}

/// Two profiles of a different shape in one organization (D143's equity test), and a worker with none: the same recorded
/// minutes are over one worker's limit and not the other's, and the worker with no place of work is counted as not configured,
/// never flagged and never passed.
#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn working_time_follows_each_workers_profile() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let (short, long, none) = (
            seed_worker!(&request, &org, "JU-S", None).await,
            seed_worker!(&request, &org, "JU-L", None).await,
            seed_worker!(&request, &org, "JU-N", None).await,
        );
        for who in [&short, &long, &none] {
            activate!(&request, who).await;
        }
        // A federal chain: the state's rest floor, the country's window, the bloc's lower ceiling.
        for body in [
            json!({ "organization_ref": org, "code": "QZ", "layer": "supranational" }),
            json!({ "organization_ref": org, "code": "XA", "layer": "national", "parent_code": "QZ" }),
            json!({ "organization_ref": org, "code": "XA-01", "layer": "subnational", "parent_code": "XA" }),
        ] {
            request.post("/api/jurisdictions").json(&body).await.assert_status_ok();
        }
        for (code, key, value) in [
            ("XA", "working_time.reference_weeks", 4),
            ("XA", "working_time.max_average_weekly_minutes", 2_880),
            ("QZ", "working_time.max_average_weekly_minutes", 2_400),
            ("XA-01", "working_time.min_daily_rest_minutes", 720),
        ] {
            request
                .post("/api/jurisdiction-parameters")
                .json(&json!({ "organization_ref": org, "code": code, "key": key, "value": value,
                               "effective_from": "2000-01-01", "source": "synthetic", "status": "example" }))
                .await
                .assert_status_ok();
        }
        request
            .put(&format!("/api/workers/{short}"))
            .json(&json!({ "work_jurisdiction": "XA-01" }))
            .await
            .assert_status_ok();
        working_time_profile(&request, &org, "XB", (17, 2_880, 660), &[&long]).await;

        // 10 000 minutes in the 28 days to 2026-07-10, for each of the three.
        for who in [&short, &long, &none] {
            let mut day = chrono::NaiveDate::from_ymd_opt(2026, 6, 20).unwrap();
            for _ in 0..10 {
                request
                    .post(&format!("/api/workers/{who}/time-entries"))
                    .json(&json!({ "worked_on": day, "minutes": 1000 }))
                    .await
                    .assert_status_ok();
                day += chrono::Duration::days(1);
            }
        }
        let signals: Value = request.get("/api/workforce/working-time?as_of=2026-07-10").await.json();
        let find = |who: &str| signals["flagged"].as_array().unwrap().iter().find(|f| f["worker_pid"] == who).cloned();
        let row = find(&short).expect("10 000 over 4 weeks is over 2 400 a week");
        assert_eq!(row["average_weekly"]["denominator_weeks"], 4);
        assert_eq!(row["limits"]["max_average_weekly_minutes"]["value"], 2_400);
        assert_eq!(row["limits"]["max_average_weekly_minutes"]["from"][0]["code"], "QZ", "the bloc's lower ceiling");
        assert_eq!(row["limits"]["reference_weeks"]["from"][0]["code"], "XA");
        assert_eq!(row["limits"]["min_daily_rest_minutes"]["value"], 720);
        assert!(find(&long).is_none(), "10 000 over 17 weeks is under 2 880 a week");
        assert!(find(&none).is_none(), "no limit configured is not a flag");
        assert!(signals["not_configured"]["average"].as_u64().unwrap() >= 1);
        assert!(signals["not_configured"]["no_work_jurisdiction"].as_u64().unwrap() >= 1);
        assert!(!signals.to_string().contains("over_48h"));
    })
    .await;
}
