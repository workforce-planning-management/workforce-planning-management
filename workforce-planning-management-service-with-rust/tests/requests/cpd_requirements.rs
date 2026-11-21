//! A CPD requirement can be corrected or withdrawn, and a worker's progress carries a pace that says
//! what it rests on (WPM-R139, WPM-D92).
use serde_json::{Value, json};
use serial_test::serial;

use super::{a_person, an_org};

async fn hire(request: &axa_test::TestServer) -> String {
    let created: Value = request
        .post("/api/workers")
        .json(&json!({
            "person_ref": a_person(), "organization_ref": an_org(), "worker_number": "1",
            "display_name": "Pace Worker", "employment_type": "permanent", "department": "engineering",
            "job_title": format!("Pace {}", uuid::Uuid::new_v4().simple()), "hired_on": "2020-01-05",
        }))
        .await
        .json();
    created["pid"].as_str().expect("worker pid").to_string()
}

/// The worker's first requirement in their progress.
async fn progress(request: &axa_test::TestServer, worker: &str) -> Value {
    let p: Value = request
        .get(&format!("/api/workers/{worker}/cpd-progress"))
        .await
        .json();
    p["requirements"][0].clone()
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn a_requirement_is_corrected_and_withdrawn_and_entries_stay() {
    crate::requests::request_open(|request, _ctx| async move {
        let worker = hire(&request).await;
        let detail: Value = request.get(&format!("/api/workers/{worker}")).await.json();
        let title = detail["job_title"].as_str().expect("title").to_string();
        let today = workforce_planning_management::dates::today();
        let body = |name: &str, required: f64, unit: &str| {
            json!({
                "name": name, "unit": unit, "required": required, "job_title": title,
                "period_start": (today - jiff::Span::new().days(9)).to_string(),
                "period_end": (today + jiff::Span::new().days(20)).to_string(), // 30 days, today is day 10
            })
        };
        let created: Value = request
            .post("/api/cpd-requirements")
            .json(&body("Annual hours", 30.0, "hours"))
            .await
            .json();
        let pid = created["pid"]
            .as_str()
            .expect("requirement pid")
            .to_string();
        request
            .post(&format!("/api/workers/{worker}/cpd-entries"))
            .json(&json!({
                "entry_date": today.to_string(), "activity": "Course", "category": "course",
                "unit": "hours", "amount": 5.0,
            }))
            .await
            .assert_status_ok();
        // 5 of 30 hours with a third of the period gone: the line gives 10, so behind
        let before = progress(&request, &worker).await;
        assert_eq!(before["name"], "Annual hours");
        assert_eq!(before["pace"]["status"], "behind");
        assert_eq!(before["pace"]["expected_by_today"], 10.0);
        assert_eq!(before["pace"]["days_left"], 21);
        assert!(
            before["pace"]["basis"]
                .as_str()
                .unwrap()
                .contains("straight line")
        );

        // corrected down to 12 hours: the same entry now puts them ahead of the line
        request
            .put(&format!("/api/cpd-requirements/{pid}"))
            .json(&body("Annual hours (corrected)", 12.0, "hours"))
            .await
            .assert_status_ok();
        let after = progress(&request, &worker).await;
        assert_eq!(after["name"], "Annual hours (corrected)");
        assert_eq!(after["required"], 12.0);
        assert_eq!(after["recorded"], 5.0, "the entry was not touched");
        assert_eq!(after["pace"]["status"], "on_track");

        // the unit cannot change, and a bad body is refused
        let changed = request
            .put(&format!("/api/cpd-requirements/{pid}"))
            .json(&body("x", 12.0, "points"))
            .await;
        assert_eq!(changed.status_code(), 422, "{}", changed.text());
        assert_eq!(
            request
                .put(&format!("/api/cpd-requirements/{pid}"))
                .json(&body("x", -1.0, "hours"))
                .await
                .status_code(),
            422
        );

        // withdrawn: it is gone from the list and from progress, and withdrawing twice is a 404
        request
            .delete(&format!("/api/cpd-requirements/{pid}"))
            .await
            .assert_status_ok();
        let listed: Value = request.get("/api/cpd-requirements").await.json();
        assert!(!listed.to_string().contains(&pid));
        let p: Value = request
            .get(&format!("/api/workers/{worker}/cpd-progress"))
            .await
            .json();
        assert_eq!(p["requirements"], json!([]));
        assert_eq!(
            request
                .delete(&format!("/api/cpd-requirements/{pid}"))
                .await
                .status_code(),
            404
        );
        let entries: Value = request
            .get(&format!("/api/workers/{worker}/cpd-entries"))
            .await
            .json();
        assert_eq!(
            entries.as_array().unwrap().len(),
            1,
            "the worker's entry remains"
        );
    })
    .await;
}

/// A live requirement is listed with its figures; an entry's evidence must be a web link; a registration may not expire
/// before it starts; progress lists the registrations with their standing.
#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn evidence_registrations_and_the_live_list_are_checked() {
    crate::requests::request_open(|request, _ctx| async move {
        let worker = hire(&request).await;
        let today = workforce_planning_management::dates::today();
        let name = format!("Live {}", uuid::Uuid::new_v4().simple());
        request
            .post("/api/cpd-requirements")
            .json(&json!({ "name": name, "unit": "hours", "required": 12.5,
                            "period_start": (today - jiff::Span::new().days(5)).to_string(),
                            "period_end": (today + jiff::Span::new().days(5)).to_string() }))
            .await
            .assert_status_ok();
        let listed: Value = request.get("/api/cpd-requirements").await.json();
        let row = listed.as_array().unwrap().iter().find(|r| r["name"] == name.as_str()).unwrap().clone();
        assert_eq!((row["required"].as_f64(), row["required_hundredths"].as_i64()), (Some(12.5), Some(1250)), "{row}");

        let entries = format!("/api/workers/{worker}/cpd-entries");
        let entry = |url: &str| json!({ "entry_date": today.to_string(), "activity": "Reading", "category": "reading",
                                        "unit": "hours", "amount": 1.0, "evidence_note": "Notes kept", "evidence_url": url });
        let ftp = request.post(&entries).json(&entry("ftp://example.org/x")).await;
        assert_eq!(ftp.status_code(), 422);
        assert!(ftp.text().contains("evidence_url"), "{}", ftp.text());
        let kept = request.post(&entries).json(&entry("https://example.org/notes")).await;
        assert_eq!(kept.status_code(), 200, "{}", kept.text());

        let registrations = format!("/api/workers/{worker}/registrations");
        let reversed = json!({ "body": "Example Council", "reference": "R-1", "registered_on": "2026-05-01", "expires_on": "2026-04-01" });
        assert_eq!(request.post(&registrations).json(&reversed).await.status_code(), 422);
        let made = request
            .post(&registrations)
            .json(&json!({ "body": "Example Council", "reference": "R-1", "registered_on": "2026-01-01",
                            "expires_on": (today + jiff::Span::new().days(400)).to_string() }))
            .await;
        assert_eq!(made.status_code(), 200, "{}", made.text());
        let progress: Value = request.get(&format!("/api/workers/{worker}/cpd-progress")).await.json();
        assert!(progress.to_string().contains("Example Council"), "{progress}");
    })
    .await;
}
