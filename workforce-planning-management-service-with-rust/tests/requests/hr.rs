//! The hire journey end-to-end (WPM-R1–R3, WPM-R7) plus the
//! unknown-pid `404` contract and the org-chart cycle refusal.

use serde_json::{Value, json};
use serial_test::serial;

use super::{a_person, a_worker, activate, an_org, seed_worker};

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
// Requisition → candidate → application → interview → offer → hired
// (creates the worker in-tx) → onboarding gate blocks activation →
// complete/waive → activate → requisition fills.
async fn hire_journey_end_to_end() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        // Requisition: draft → open.
        let requisition: Value = request
            .post("/api/requisitions")
            .json(&json!({
                "organization_ref": org, "department": "engineering",
                "job_title": "Platform Engineer", "headcount": 1,
                "salary_min_minor": 3_000_000, "salary_max_minor": 4_000_000, "salary_currency": "GBP",
            }))
            .await
            .json();
        let req_pid = requisition["pid"].as_str().unwrap().to_string();
        // Cannot fill from draft (state machine).
        let premature = request
            .post(&format!("/api/requisitions/{req_pid}/status"))
            .json(&json!({ "to": "filled" }))
            .await;
        assert_eq!(premature.status_code(), 422);
        for to in ["open", "interviewing"] {
            let response = request
                .post(&format!("/api/requisitions/{req_pid}/status"))
                .json(&json!({ "to": to }))
                .await;
            assert_eq!(response.status_code(), 200, "requisition -> {to}");
        }
        // Candidate + application.
        let candidate: Value = request
            .post("/api/candidates")
            .json(&json!({
                "display_name": "Test Applicant 001",
                "email": "applicant@example.com",
                "source": "referral",
                "person_ref": a_person(),
            }))
            .await
            .json();
        let application: Value = request
            .post(&format!("/api/requisitions/{req_pid}/applications"))
            .json(&json!({ "candidate_pid": candidate["pid"] }))
            .await
            .json();
        let app_pid = application["pid"].as_str().unwrap().to_string();
        // Interview round.
        let interview: Value = request
            .post(&format!("/api/applications/{app_pid}/interviews"))
            .json(&json!({
                "scheduled_at": "2026-07-20T10:00:00Z",
                "interviewer_ref": a_worker(),
            }))
            .await
            .json();
        let outcome = request
            .put(&format!(
                "/api/interviews/{}",
                interview["pid"].as_str().unwrap()
            ))
            .json(&json!({ "outcome": "advance" }))
            .await;
        assert_eq!(outcome.status_code(), 200);
        // Stage machine: cannot jump received → hired.
        let jump = request
            .post(&format!("/api/applications/{app_pid}/stage"))
            .json(&json!({ "to": "hired" }))
            .await;
        assert_eq!(jump.status_code(), 422);
        for to in ["screened", "interviewing", "offer"] {
            let response = request
                .post(&format!("/api/applications/{app_pid}/stage"))
                .json(&json!({ "to": to }))
                .await;
            assert_eq!(response.status_code(), 200, "application -> {to}");
        }
        // Hire: creates the worker (onboarding) in one transaction.
        let hired: Value = request
            .post(&format!("/api/applications/{app_pid}/stage"))
            .json(&json!({
                "to": "hired", "worker_number": "E-9001",
                "salary_minor": 4_000_000, "salary_currency": "GBP",
            }))
            .await
            .json();
        let worker_pid = hired["worker_pid"]
            .as_str()
            .expect("worker created")
            .to_string();
        let worker: Value = request
            .get(&format!("/api/workers/{worker_pid}"))
            .await
            .json();
        assert_eq!(worker["status"], "onboarding");
        assert_eq!(worker["department"], "engineering");
        // Onboarding gate: a pending mandatory item blocks activation.
        let items: Value = request
            .post(&format!("/api/workers/{worker_pid}/onboarding"))
            .json(&json!({ "items": [
                { "name": "Signed contract", "mandatory": true },
                { "name": "Right to work check", "mandatory": true },
                { "name": "Desk preference", "mandatory": false },
            ]}))
            .await
            .json();
        let item_pids: Vec<String> = items["pids"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p.as_str().unwrap().to_string())
            .collect();
        let blocked = request
            .post(&format!("/api/workers/{worker_pid}/status"))
            .json(&json!({ "to": "active" }))
            .await;
        assert_eq!(
            blocked.status_code(),
            422,
            "mandatory items block activation"
        );
        // Complete one, waive the other (reason required).
        let no_reason = request
            .post(&format!("/api/onboarding-items/{}/waive", item_pids[1]))
            .json(&json!({ "reason": "  " }))
            .await;
        assert_eq!(no_reason.status_code(), 422);
        request
            .post(&format!("/api/onboarding-items/{}/complete", item_pids[0]))
            .await
            .assert_status_ok();
        request
            .post(&format!("/api/onboarding-items/{}/waive", item_pids[1]))
            .json(&json!({ "reason": "reference on file from agency" }))
            .await
            .assert_status_ok();
        activate!(&request, &worker_pid).await;
        // Requisition can now fill (offer → filled needs the hired count).
        request
            .post(&format!("/api/requisitions/{req_pid}/status"))
            .json(&json!({ "to": "offer" }))
            .await
            .assert_status_ok();
        request
            .post(&format!("/api/requisitions/{req_pid}/status"))
            .json(&json!({ "to": "filled" }))
            .await
            .assert_status_ok();
        // The audit trail recorded the journey.
        let audits: Value = request
            .get(&format!("/api/audits/{worker_pid}"))
            .await
            .json();
        let actions: Vec<&str> = audits
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|a| a["action"].as_str())
            .collect();
        assert!(actions.contains(&"worker_hired"), "actions: {actions:?}");
        assert!(
            actions.contains(&"worker_activated"),
            "actions: {actions:?}"
        );
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
// Unknown-pid reads are honest 404s (family lesson: loco 0.16 does not
// map ModelError::EntityNotFound); malformed pids too. The org-chart
// cycle check refuses a managerial loop, and the worker-number
// uniqueness holds per organization.
async fn contracts_404_cycle_and_uniqueness() {
    crate::requests::request_open(|request, _ctx| async move {
        let ghost = uuid::Uuid::new_v4();
        for path in [
            format!("/api/workers/{ghost}"),
            format!("/api/requisitions/{ghost}"),
            format!("/api/payroll-runs/{ghost}"),
            "/api/workers/not-a-uuid".to_string(),
        ] {
            assert_eq!(request.get(&path).await.status_code(), 404, "{path}");
        }
        // Cycle refusal: a -> b -> a.
        let org = an_org();
        let a = seed_worker!(&request, &org, "E-0001", None).await;
        let b = seed_worker!(&request, &org, "E-0002", None).await;
        request
            .put(&format!("/api/workers/{b}"))
            .json(&json!({ "manager_pid": a }))
            .await
            .assert_status_ok();
        let cycle = request
            .put(&format!("/api/workers/{a}"))
            .json(&json!({ "manager_pid": b }))
            .await;
        assert_eq!(cycle.status_code(), 422, "managerial cycle refused");
        // Self-management refused too.
        let self_cycle = request
            .put(&format!("/api/workers/{a}"))
            .json(&json!({ "manager_pid": a }))
            .await;
        assert_eq!(self_cycle.status_code(), 422);
        // The org chart renders a forest with b under a.
        let chart: Value = request
            .get(&format!("/api/org-chart?organization={org}"))
            .await
            .json();
        let roots = chart.as_array().unwrap();
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0]["reports"].as_array().unwrap().len(), 1);
        // A limit draws the roots first and a person only under a manager who is drawn; the size says how many
        // there are in all.
        let one: Value = request
            .get(&format!("/api/org-chart?organization={org}&limit=1"))
            .await
            .json();
        assert_eq!(one.as_array().unwrap().len(), 1);
        assert_eq!(
            one[0]["reports"].as_array().unwrap().len(),
            0,
            "the report is cut off"
        );
        let size: Value = request
            .get(&format!("/api/org-chart/size?organization={org}"))
            .await
            .json();
        assert_eq!(size["people"], 2);
        // Worker-number uniqueness per organization (index): the
        // duplicate insert surfaces as an error, not a silent success.
        let duplicate = request
            .post("/api/workers")
            .json(&json!({
                "person_ref": a_person(), "organization_ref": org,
                "worker_number": "E-0001", "display_name": "Dup",
                "employment_type": "permanent", "department": "engineering",
                "job_title": "Engineer", "hired_on": "2026-01-05",
            }))
            .await;
        assert_ne!(
            duplicate.status_code(),
            200,
            "duplicate worker number refused"
        );
        // Validation contract: bad URN, bad token, bad FTE ⇒ 422.
        let invalid = request
            .post("/api/workers")
            .json(&json!({
                "person_ref": "not-a-urn", "organization_ref": org,
                "worker_number": "E-0003", "display_name": "X",
                "employment_type": "gig", "department": "engineering",
                "job_title": "Engineer", "fte_percent": 250,
                "hired_on": "2026-01-05",
            }))
            .await;
        assert_eq!(invalid.status_code(), 422);
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
// WPM-T388: `?fields=` returns only the fields asked for, refuses an unknown name, and the list audits a salary read whenever it carries
// a salary, as the single read does, and not when it does not.
async fn worker_list_returns_the_fields_asked_for_and_audits_salary_it_carries() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let department = format!("lean-{}", uuid::Uuid::new_v4().simple());
        let mut pids = Vec::new();
        for n in 1..=2 {
            let created: Value = request
                .post("/api/workers")
                .json(&json!({
                    "person_ref": a_person(), "organization_ref": org,
                    "worker_number": format!("{department}-{n}"), "display_name": format!("Lean Worker {n}"),
                    "employment_type": "permanent", "department": department, "job_title": "Engineer",
                    "salary_minor": 3_000_000, "salary_currency": "GBP", "hired_on": "2026-01-05",
                }))
                .await
                .json();
            pids.push(created["pid"].as_str().unwrap().to_string());
        }
        let salary_reads = |pid: String| {
            let request = &request;
            async move {
                let audits: Value = request.get(&format!("/api/audits/{pid}")).await.json();
                audits.as_array().unwrap().iter().filter(|a| a["action"] == "salary_read").cloned().collect::<Vec<Value>>()
            }
        };

        // The lean list: three fields per row and nothing else; no salary, so no salary read is recorded. Paging is unchanged.
        let lean = request.get(&format!("/api/workers?department={department}&fields=pid,department,person_ref")).await;
        assert_eq!(lean.status_code(), 200);
        assert_eq!(lean.headers().get("x-total-count").unwrap().to_str().unwrap(), "2");
        let rows: Value = lean.json();
        let rows = rows.as_array().unwrap();
        assert_eq!(rows.len(), 2);
        for row in rows {
            let mut keys: Vec<&str> = row.as_object().unwrap().keys().map(String::as_str).collect();
            keys.sort_unstable();
            assert_eq!(keys, vec!["department", "person_ref", "pid"], "{row}");
            assert!(row["person_ref"].as_str().unwrap().starts_with("person:"));
        }
        for pid in &pids {
            assert_eq!(salary_reads(pid.clone()).await, Vec::<Value>::new(), "no salary in the answer, no salary read");
        }

        // Asking for the salary is a salary read of every row that carries one, audited with the department and no figure.
        let paid: Value = request.get(&format!("/api/workers?department={department}&fields=pid,salary_minor")).await.json();
        assert!(paid.as_array().unwrap().iter().all(|r| r["salary_minor"] == 3_000_000 && r.as_object().unwrap().len() == 2), "{paid}");
        for pid in &pids {
            let reads = salary_reads(pid.clone()).await;
            assert_eq!(reads.len(), 1, "{reads:?}");
            assert_eq!(reads[0]["snapshot"], json!({ "department": department, "via": "list" }));
            assert!(!reads[0].to_string().contains("3000000"));
        }
        // The whole record (no `fields`) is the list as before, and is a salary read too.
        let whole: Value = request.get(&format!("/api/workers?department={department}")).await.json();
        assert!(whole[0].get("display_name").is_some() && whole[0].get("job_title").is_some(), "{whole}");
        assert_eq!(salary_reads(pids[0].clone()).await.len(), 2);

        // Refusals: an unknown name, an empty list, the internal id; nothing is audited for them.
        for bad in ["pid,nmae", "", "id", "pid,deleted_at", "salary"] {
            let response = request.get(&format!("/api/workers?department={department}&fields={bad}")).await;
            assert_eq!(response.status_code(), 422, "{bad:?}");
        }
        assert_eq!(salary_reads(pids[0].clone()).await.len(), 2);
    })
    .await;
}
