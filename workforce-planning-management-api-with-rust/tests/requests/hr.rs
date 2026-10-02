//! The hire journey end-to-end (WPM-R1–R3, WPM-R7) plus the
//! unknown-pid `404` contract and the org-chart cycle refusal.

use loco_rs::testing::prelude::*;
use serde_json::{Value, json};
use serial_test::serial;
use workforce_planning_management_service::app::App;

use super::{a_person, a_worker, activate, an_org, seed_worker};

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
// Requisition → candidate → application → interview → offer → hired
// (creates the worker in-tx) → onboarding gate blocks activation →
// complete/waive → activate → requisition fills.
async fn hire_journey_end_to_end() {
    request::<App, _, _>(|request, _ctx| async move {
        let org = an_org();
        // Requisition: draft → open.
        let requisition: Value = request
            .post("/api/requisitions")
            .json(&json!({
                "organization_ref": org, "department": "engineering",
                "job_title": "Platform Engineer", "headcount": 1,
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
    request::<App, _, _>(|request, _ctx| async move {
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
