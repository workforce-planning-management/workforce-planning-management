//! The joint pay assessment signal (WPM-R146, WPM-D99): it reports whether an assessment is due from a category's gap and
//! what a person recorded, and a person records; it never decides.

use sea_orm::ConnectionTrait;
use serde_json::{Value, json};
use serial_test::serial;

use super::{a_person, an_org};

struct MonitoringOn;

impl MonitoringOn {
    fn new() -> Self {
        unsafe {
            std::env::set_var(
                "WPM_EQUALITY_MONITORING_BASIS",
                "Test deployment: impact assessment JA-1",
            );
            std::env::set_var("WPM_EQUALITY_CATEGORIES", r#"{"sex":["female","male"]}"#);
            std::env::set_var("WPM_EQUALITY_FLOOR", "5");
        }
        Self
    }
}

impl Drop for MonitoringOn {
    fn drop(&mut self) {
        unsafe {
            std::env::remove_var("WPM_EQUALITY_MONITORING_BASIS");
            std::env::remove_var("WPM_EQUALITY_CATEGORIES");
            std::env::remove_var("WPM_EQUALITY_FLOOR");
            std::env::remove_var("WPM_JOINT_ASSESSMENT_THRESHOLD_PERCENT");
            std::env::remove_var("WPM_JOINT_ASSESSMENT_MONTHS");
        }
    }
}

async fn hire(
    request: &loco_rs::TestServer,
    ctx: &loco_rs::app::AppContext,
    org: &str,
    title: &str,
    sex: &str,
    salary: i64,
    n: usize,
) {
    for i in 0..n {
        let created: Value = request
            .post("/api/workers")
            .json(&json!({
                "person_ref": a_person(), "organization_ref": org,
                "worker_number": format!("JA-{title}-{sex}-{i}"),
                "display_name": format!("Joint Assessment {title} {i}"), "employment_type": "permanent",
                "department": "engineering", "job_title": title, "hired_on": "2020-01-05",
                "salary_minor": salary, "salary_currency": "GBP",
            }))
            .await
            .json();
        let pid = created["pid"].as_str().expect("pid").to_string();
        ctx.db
            .execute_unprepared(&format!(
                "INSERT INTO equality_declarations (worker_pid, category, value) VALUES ('{pid}', 'sex', '{sex}')"
            ))
            .await
            .unwrap();
    }
}

fn category<'a>(body: &'a Value, name: &str) -> &'a Value {
    body["categories"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["category"] == name)
        .unwrap()
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
#[allow(clippy::too_many_lines)] // one scenario read top to bottom
async fn the_signal_follows_the_gap_the_clock_and_what_a_person_recorded() {
    crate::requests::request_open(|request, ctx| async move {
        let _on = MonitoringOn::new();
        let org = an_org();
        // Engineering: women on 90,000, men on 100,000 (a ten percent gap). Facilities: no gap.
        hire(&request, &ctx, &org, "Engineer", "female", 9_000_000, 6).await;
        hire(&request, &ctx, &org, "Engineer", "male", 10_000_000, 6).await;
        hire(&request, &ctx, &org, "Porter", "female", 3_000_000, 5).await;
        hire(&request, &ctx, &org, "Porter", "male", 3_000_000, 5).await;
        for (name, titles) in [("Engineering", json!(["Engineer"])), ("Facilities", json!(["Porter"]))] {
            request
                .post("/api/pay-categories")
                .json(&json!({ "organization_ref": org, "name": name, "basis": "Same skills, effort, responsibility and conditions.", "job_titles": titles }))
                .await
                .assert_status_ok();
        }
        let url = |extra: &str| format!("/api/pay-transparency/joint-assessment?organization={org}&currency=GBP{extra}");

        // Not yet reported: the clock has not started. No gap: nothing due.
        let first: Value = request.get(&url("&as_of=2026-10-24")).await.json();
        assert_eq!(first["threshold_bp"], 500);
        assert_eq!(first["remedy_months"], 6);
        assert_eq!(category(&first, "Engineering")["gap"]["value"]["gap_bp"], 1_000);
        assert_eq!(category(&first, "Engineering")["signal"], json!({ "status": "pending", "remedy_by": null }));
        assert_eq!(category(&first, "Facilities")["signal"], json!({ "status": "not_due", "why": "below_threshold" }));

        // Reported long enough ago: six months have run out.
        let due: Value = request.get(&url("&as_of=2026-10-24&reported_on=2026-01-01")).await.json();
        assert_eq!(category(&due, "Engineering")["signal"], json!({ "status": "due", "remedy_by": "2026-07-01" }));
        // Reported recently: still within the time.
        let pending: Value = request.get(&url("&as_of=2026-10-24&reported_on=2026-09-01")).await.json();
        assert_eq!(category(&pending, "Engineering")["signal"], json!({ "status": "pending", "remedy_by": "2027-03-01" }));

        // A person records a remedy that came too late: still due. In time: not due.
        let put = |body: Value| {
            let request = &request;
            async move { request.put("/api/pay-transparency/assessment-records").json(&body).await }
        };
        let base = |extra: Value| {
            let mut b = json!({ "organization_ref": org, "category": "Engineering", "reported_on": "2026-01-01" });
            b.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
            b
        };
        assert_eq!(put(base(json!({ "remedied_on": "2026-08-01" }))).await.status_code(), 200);
        let late: Value = request.get(&url("&as_of=2026-10-24&reported_on=2026-01-01")).await.json();
        assert_eq!(category(&late, "Engineering")["signal"]["status"], "due", "a late remedy does not undo it");
        assert_eq!(put(base(json!({ "remedied_on": "2026-06-30" }))).await.status_code(), 200, "replaces");
        let remedied: Value = request.get(&url("&as_of=2026-10-24&reported_on=2026-01-01")).await.json();
        assert_eq!(category(&remedied, "Engineering")["signal"], json!({ "status": "not_due", "why": "remedied" }));

        // Justified: not due. The record comes back with its words.
        assert_eq!(put(base(json!({ "justified": true, "justification": "Different grades under the published job evaluation scheme." }))).await.status_code(), 200);
        let justified: Value = request.get(&url("&as_of=2026-10-24")).await.json();
        let e = category(&justified, "Engineering");
        assert_eq!(e["signal"], json!({ "status": "not_due", "why": "justified" }));
        assert_eq!(e["reported_on"], "2026-01-01", "the latest recorded report date is used when none is given");
        assert!(e["record"]["justification"].as_str().unwrap().contains("job evaluation"));
        assert!(e["record"].get("id").is_none());

        // Assessed with the representatives: not due, with the outcome.
        assert_eq!(
            put(base(json!({ "assessed_on": "2026-09-15", "outcome": "difference_found_remedy_planned", "representatives": "Staff side, engineering" }))).await.status_code(),
            200
        );
        let assessed: Value = request.get(&url("&as_of=2026-10-24")).await.json();
        assert_eq!(category(&assessed, "Engineering")["signal"], json!({ "status": "not_due", "why": "assessed" }));
        let records: Value = request.get(&format!("/api/pay-transparency/assessment-records?organization={org}")).await.json();
        assert_eq!(records.as_array().unwrap().len(), 1, "one record per category and report date, replaced each time");
        assert_eq!(records[0]["outcome"], "difference_found_remedy_planned");
        assert_eq!(records[0]["representatives"], "Staff side, engineering");
        assert!(records[0]["justification"].is_null(), "a replaced record keeps only what was last said");

        // Refused, with the reason.
        for (why, body) in [
            ("justified without words", base(json!({ "justified": true }))),
            ("words without justified", base(json!({ "justification": "A reason that nobody asked for here." }))),
            ("an outcome with no date", base(json!({ "outcome": "difference_remedied" }))),
            ("a date with no outcome", base(json!({ "assessed_on": "2026-09-15" }))),
            ("an outcome off the list", base(json!({ "assessed_on": "2026-09-15", "outcome": "unfair" }))),
            ("a remedy before the report", base(json!({ "remedied_on": "2025-12-31" }))),
            ("no organization", json!({ "organization_ref": "nope", "category": "E", "reported_on": "2026-01-01" })),
            ("no category", base(json!({ "category": " " }))),
            ("representatives too long", base(json!({ "representatives": "x".repeat(201) }))),
        ] {
            let r = put(body).await;
            assert_eq!(r.status_code(), 422, "{why}: {}", r.text());
        }

        // Threshold and period are configuration.
        unsafe {
            std::env::set_var("WPM_JOINT_ASSESSMENT_THRESHOLD_PERCENT", "12");
            std::env::set_var("WPM_JOINT_ASSESSMENT_MONTHS", "3");
        }
        let strict: Value = request.get(&url("&as_of=2026-10-24&reported_on=2026-09-01")).await.json();
        assert_eq!(strict["threshold_bp"], 1_200);
        assert_eq!(category(&strict, "Engineering")["signal"], json!({ "status": "not_due", "why": "below_threshold" }), "ten percent is under twelve");

        // Audited, with no figure and no words.
        let rows = ctx
            .db
            .query_all_raw(sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                "SELECT action, snapshot::text AS snapshot FROM audit_logs WHERE action IN ('joint_assessment_read', 'pay_assessment_recorded')".to_string(),
            ))
            .await
            .unwrap();
        let actions: Vec<String> = rows.iter().map(|r| r.try_get("", "action").unwrap()).collect();
        assert!(actions.contains(&"joint_assessment_read".to_string()) && actions.contains(&"pay_assessment_recorded".to_string()), "{actions:?}");
        for r in &rows {
            let snapshot: String = r.try_get::<Option<String>>("", "snapshot").unwrap().unwrap_or_default();
            assert!(!snapshot.contains("job evaluation") && !snapshot.contains("gap_bp") && !snapshot.contains("Staff side"), "{snapshot}");
        }
        let text = request.get(&url("&as_of=2026-10-24")).await.text();
        assert!(!text.contains("Joint Assessment Engineer"), "no worker is named");
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn a_gap_that_is_hidden_is_unknown_and_a_missing_setup_says_so() {
    crate::requests::request_open(|request, ctx| async move {
        let off = request.get(&format!("/api/pay-transparency/joint-assessment?organization={}", an_org())).await;
        assert_eq!(off.status_code(), 422, "needs equality monitoring on, like the report");
        let _on = MonitoringOn::new();
        let org = an_org();
        // Three of each: under the floor of five, so the gap is withheld and the signal unknown, never "not due".
        hire(&request, &ctx, &org, "Nurse", "female", 9_000_000, 3).await;
        hire(&request, &ctx, &org, "Nurse", "male", 10_000_000, 3).await;
        request
            .post("/api/pay-categories")
            .json(&json!({ "organization_ref": org, "name": "Nursing", "basis": "Same skills, effort, responsibility and conditions.", "job_titles": ["Nurse"] }))
            .await
            .assert_status_ok();
        let body: Value = request.get(&format!("/api/pay-transparency/joint-assessment?organization={org}&currency=GBP&reported_on=2020-01-01")).await.json();
        assert_eq!(category(&body, "Nursing")["signal"], json!({ "status": "unknown", "why": "gap_withheld" }));
        assert_eq!(request.get("/api/pay-transparency/assessment-records?organization=nope").await.status_code(), 422);
    })
    .await;
}

/// No audit, no write (WPM-D107): when the audit row cannot be written the change is refused and nothing of it is kept.
#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn when_the_audit_row_cannot_be_written_the_write_is_refused_and_not_kept() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let record = json!({ "organization_ref": org, "category": "Engineering", "reported_on": "2026-01-01", "remedied_on": "2026-02-01" });
        let category = json!({ "organization_ref": org, "name": "Engineering", "basis": "Same skills, effort, responsibility and conditions.", "job_titles": ["Engineer"] });
        // An audit table that refuses every insert.
        ctx.db
            .execute_unprepared(
                "CREATE OR REPLACE FUNCTION wpm_refuse_audit() RETURNS trigger AS $$
                 BEGIN RAISE EXCEPTION 'the audit trail is unavailable'; END; $$ LANGUAGE plpgsql;
                 CREATE TRIGGER aa_refuse_audit BEFORE INSERT ON audit_logs FOR EACH ROW EXECUTE FUNCTION wpm_refuse_audit();",
            )
            .await
            .unwrap();
        let put = request.put("/api/pay-transparency/assessment-records").json(&record).await;
        let post = request.post("/api/pay-categories").json(&category).await;
        ctx.db
            .execute_unprepared("DROP TRIGGER IF EXISTS aa_refuse_audit ON audit_logs; DROP FUNCTION IF EXISTS wpm_refuse_audit();")
            .await
            .unwrap();
        assert!(put.status_code().is_server_error(), "refused, not accepted: {}", put.status_code());
        assert!(post.status_code().is_server_error(), "refused, not accepted: {}", post.status_code());
        // Nothing of either change was kept.
        let records: Value = request.get(&format!("/api/pay-transparency/assessment-records?organization={org}")).await.json();
        assert!(records.as_array().unwrap().is_empty(), "the record was rolled back: {records}");
        let categories: Value = request.get(&format!("/api/pay-categories?organization={org}")).await.json();
        assert!(categories.as_array().unwrap().is_empty(), "the category was rolled back: {categories}");
        // And with the audit trail back, the same writes go through.
        assert_eq!(request.put("/api/pay-transparency/assessment-records").json(&record).await.status_code(), 200);
        assert_eq!(request.post("/api/pay-categories").json(&category).await.status_code(), 200);
    })
    .await;
}
