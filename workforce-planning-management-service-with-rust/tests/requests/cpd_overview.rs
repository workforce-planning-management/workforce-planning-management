//! The CPD overview counts in the database and gives the answers it always did (WPM-D90): who a
//! requirement applies to, who met it on recorded and on verified entries, and which registrations need
//! attention, for employed workers only.

use chrono::{Days, Utc};
use serde_json::{Value, json};
use serial_test::serial;

use super::{a_person, an_org};

async fn hire(
    request: &loco_rs::TestServer,
    org: &str,
    n: usize,
    title: &str,
    hired_on: &str,
) -> String {
    let created: Value = request
        .post("/api/workers")
        .json(&json!({
            "person_ref": a_person(), "organization_ref": org, "worker_number": format!("O{n}"),
            "display_name": format!("Overview Worker {n}"), "employment_type": "permanent",
            "department": "engineering", "job_title": title, "hired_on": hired_on,
        }))
        .await
        .json();
    created["pid"].as_str().expect("worker pid").to_string()
}

async fn record(
    request: &loco_rs::TestServer,
    worker: &str,
    date: &str,
    unit: &str,
    amount: f64,
) -> String {
    let created: Value = request
        .post(&format!("/api/workers/{worker}/cpd-entries"))
        .json(&json!({ "entry_date": date, "activity": "Activity", "category": "course", "unit": unit, "amount": amount }))
        .await
        .json();
    created["pid"].as_str().expect("entry pid").to_string()
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn the_overview_counts_employed_workers_against_each_requirement_and_what_needs_attention() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let title = format!("Overview {}", uuid::Uuid::new_v4().simple());
        let today = Utc::now().date_naive();
        let period = |extra: Value| {
            let mut body = json!({
                "unit": "hours", "required": 10.0,
                "period_start": (today - Days::new(30)).to_string(), "period_end": (today + Days::new(30)).to_string(),
            });
            body.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
            body
        };
        let in_period = (today - Days::new(5)).to_string();

        // four workers with the title, one of another title, one not yet employed
        let met_verified = hire(&request, &org, 1, &title, "2020-01-05").await;
        let met_unverified = hire(&request, &org, 2, &format!("  {}  ", title.to_uppercase()), "2020-01-05").await;
        let short = hire(&request, &org, 3, &title, "2020-01-05").await;
        let none = hire(&request, &org, 4, &title, "2020-01-05").await;
        let other_title = hire(&request, &org, 5, "Somebody Else", "2020-01-05").await;
        let not_yet = hire(&request, &org, 6, &title, "2999-01-01").await;

        let entry = record(&request, &met_verified, &in_period, "hours", 10.0).await;
        request.post(&format!("/api/cpd-entries/{entry}/verify")).await.assert_status_ok();
        record(&request, &met_unverified, &in_period, "hours", 6.0).await;
        record(&request, &met_unverified, &in_period, "hours", 4.0).await;
        record(&request, &short, &in_period, "hours", 9.99).await;
        // entries that must not count: another unit, outside the period, a withdrawn one
        record(&request, &short, &in_period, "points", 50.0).await;
        record(&request, &short, &(today - Days::new(60)).to_string(), "hours", 50.0).await;
        let withdrawn = record(&request, &short, &in_period, "hours", 50.0).await;
        request.delete(&format!("/api/cpd-entries/{withdrawn}")).await.assert_status_ok();
        record(&request, &other_title, &in_period, "hours", 99.0).await;
        let _ = &none;

        request
            .post("/api/cpd-requirements")
            .json(&period(json!({ "name": "For the title", "job_title": title })))
            .await
            .assert_status_ok();

        // registrations: employed workers only, and only those expired or inside the 90 calendar days
        let register = |worker: String, days: i64| {
            let body = json!({ "body": "Regulator", "expires_on": (today + chrono::Duration::days(days)).to_string() });
            let request = &request;
            async move { request.post(&format!("/api/workers/{worker}/registrations")).json(&body).await.assert_status_ok() }
        };
        register(met_verified.clone(), -3).await; // expired
        register(met_unverified.clone(), 30).await; // expiring
        register(short.clone(), 91).await; // valid
        register(not_yet.clone(), -3).await; // not employed: not counted

        let overview: Value = request.get("/api/cpd/overview").await.json();
        let row = overview["requirements"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["name"] == "For the title")
            .expect("the requirement");
        // applies to the four employed workers with the title (case and spaces ignored), not the fifth or the not-yet-hired
        assert_eq!(row["applicable_workers"], 4, "{row}");
        // recorded: 10.00, 6+4, short of 9.99, none: two of four; verified: only the first
        assert_eq!(row["met_recorded"], json!({ "numerator": 2, "denominator": 4, "value": 0.5 }));
        assert_eq!(row["met_verified"], json!({ "numerator": 1, "denominator": 4, "value": 0.25 }));
        assert_eq!(overview["registrations"], json!({ "expiring": 1, "expired": 1 }));
        assert!(overview["headcount"].as_u64().unwrap() >= 5);
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn an_entry_may_name_the_skill_it_developed_and_only_a_skill_in_the_catalogue() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let worker = hire(&request, &org, 91, "Carer", "2020-01-05").await;
        let skill: Value = request
            .post("/api/skills")
            .json(&json!({ "name": format!("CPD skill {}", uuid::Uuid::new_v4().simple()), "category": "technical" }))
            .await
            .json();
        let skill = skill["pid"].as_str().unwrap().to_string();
        let url = format!("/api/workers/{worker}/cpd-entries");
        let body = |skill: &str| json!({ "entry_date": "2026-09-01", "activity": "Refresher", "category": "course", "unit": "hours", "amount": 2.0, "skill_pid": skill });
        assert_eq!(request.post(&url).json(&body(&uuid::Uuid::new_v4().to_string())).await.status_code(), 422, "not in the catalogue");
        assert_eq!(request.post(&url).json(&body(&skill)).await.status_code(), 200);
        let ledger: Value = request.get(&url).await.json();
        assert_eq!(ledger[0]["skill_pid"], skill.as_str());
        let plain = request
            .post(&url)
            .json(&json!({ "entry_date": "2026-09-02", "activity": "Other", "category": "course", "unit": "hours", "amount": 1.0 }))
            .await;
        assert_eq!(plain.status_code(), 200, "a skill is optional");
    })
    .await;
}
