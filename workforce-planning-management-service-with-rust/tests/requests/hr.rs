//! The hire journey end-to-end (WPM-R1–R3, WPM-R7) plus the
//! unknown-pid `404` contract and the org-chart cycle refusal.

use serde_json::{Value, json};
use serial_test::serial;

use super::{a_person, a_worker, activate, an_org, seed_worker};

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
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
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
// Unknown-pid reads are honest 404s (family lesson: a framework's
// not-found error is not a 404 unless the handler says so); malformed pids too. The org-chart
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
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
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

        // The lean list: three fields per row and nothing else; no salary, so no salary read is recorded. Paging is unchanged: one page, so no `Link`.
        let lean = request.get(&format!("/api/workers?department={department}&fields=pid,department,person_ref")).await;
        assert_eq!(lean.status_code(), 200);
        assert!(lean.headers().get("link").is_none());
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

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
#[allow(clippy::too_many_lines)] // one hiring path, read top to bottom
// Each refusal on the way to a hire: a requisition, a candidate, an application, the hire itself, filling, and onboarding.
async fn the_hiring_path_refuses_what_cannot_be() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let requisition = |extra: Value| {
            let mut body = json!({ "organization_ref": org, "department": "engineering", "job_title": "Analyst", "headcount": 1,
                                   "salary_min_minor": 3_000_000, "salary_max_minor": 4_000_000, "salary_currency": "GBP" });
            for (k, v) in extra.as_object().unwrap() {
                body[k] = v.clone();
            }
            body
        };
        assert_eq!(request.post("/api/requisitions").json(&requisition(json!({ "headcount": 0 }))).await.status_code(), 422);
        assert_eq!(
            request.post("/api/requisitions").json(&requisition(json!({ "salary_min_minor": 5_000_000 }))).await.status_code(),
            422
        );
        let draft: Value = request.post("/api/requisitions").json(&requisition(json!({}))).await.json();
        let draft = draft["pid"].as_str().unwrap().to_string();
        let open: Value = request.post("/api/requisitions").json(&requisition(json!({}))).await.json();
        let open = open["pid"].as_str().unwrap().to_string();
        request.post(&format!("/api/requisitions/{open}/status")).json(&json!({ "to": "open" })).await.assert_status_ok();
        let unfilled = request.post(&format!("/api/requisitions/{open}/status")).json(&json!({ "to": "filled" })).await;
        assert_eq!(unfilled.status_code(), 422, "nobody hired yet: {}", unfilled.text());

        // Candidates: an address with no @ is refused; one without a person reference, and one whose consent has lapsed.
        let candidate = |body: Value| {
            let request = &request;
            async move { request.post("/api/candidates").json(&body).await }
        };
        assert_eq!(candidate(json!({ "display_name": "No at", "email": "nowhere", "source": "web" })).await.status_code(), 422);
        let anonymous: Value = candidate(json!({ "display_name": "No person", "email": "np@example.com", "source": "web" })).await.json();
        let lapsed: Value = candidate(json!({ "display_name": "Lapsed", "email": "lapsed@example.com", "source": "web",
                                                "person_ref": a_person(), "consent_until": "2020-01-01" })).await.json();
        let expired: Value = request.get("/api/candidates?expired=true").await.json();
        assert!(expired.to_string().contains(lapsed["pid"].as_str().unwrap()), "{expired}");

        // Applications: not to a draft requisition, and not from a candidate whose consent has lapsed.
        let apply = |requisition: &str, candidate: &Value| {
            let (request, url) = (&request, format!("/api/requisitions/{requisition}/applications"));
            let body = json!({ "candidate_pid": candidate["pid"] });
            async move { request.post(&url).json(&body).await }
        };
        assert_eq!(apply(&draft, &anonymous).await.status_code(), 422, "a draft takes no applications");
        assert_eq!(apply(&open, &lapsed).await.status_code(), 422, "consent has lapsed");

        // A hire needs a person reference and a worker number.
        let application: Value = apply(&open, &anonymous).await.json();
        let stage = format!("/api/applications/{}/stage", application["pid"].as_str().unwrap());
        for to in ["screened", "interviewing", "offer"] {
            request.post(&stage).json(&json!({ "to": to })).await.assert_status_ok();
        }
        assert_eq!(request.post(&stage).json(&json!({ "to": "hired", "worker_number": "H-1" })).await.status_code(), 422);
        assert_eq!(request.post(&stage).json(&json!({ "to": "hired", "person_ref": a_person() })).await.status_code(), 422);

        // Onboarding: an empty list and one over 64 items are refused; an item is finished once.
        let worker = seed_worker!(&request, &org, format!("OB-{}", uuid::Uuid::new_v4().simple()), None).await;
        let onboarding = format!("/api/workers/{worker}/onboarding");
        assert_eq!(request.post(&onboarding).json(&json!({ "items": [] })).await.status_code(), 422);
        let many: Vec<Value> = (0..65).map(|i| json!({ "name": format!("Item {i}"), "mandatory": false })).collect();
        assert_eq!(request.post(&onboarding).json(&json!({ "items": many })).await.status_code(), 422);
        let items: Value = request.post(&onboarding).json(&json!({ "items": [{ "name": "Badge", "mandatory": true }] })).await.json();
        let item = items["pids"][0].as_str().unwrap().to_string();
        request.post(&format!("/api/onboarding-items/{item}/complete")).await.assert_status_ok();
        let again = request.post(&format!("/api/onboarding-items/{item}/complete")).await;
        assert_eq!(again.status_code(), 422, "{}", again.text());
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
#[allow(clippy::too_many_lines)] // one worker record, read top to bottom
// The worker record's refusals on hire and on restating, a retirement and a leave, and the benefit plan refusals.
async fn the_worker_record_refuses_what_cannot_be() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let long = "x".repeat(500);
        let hire = |extra: Value| {
            let mut body = json!({ "person_ref": a_person(), "organization_ref": org, "worker_number": format!("WR-{tag}"),
                                   "display_name": "Record", "employment_type": "permanent", "department": "engineering",
                                   "job_title": "Engineer", "hired_on": "2026-01-05" });
            for (k, v) in extra.as_object().unwrap() {
                body[k] = v.clone();
            }
            body
        };
        for (extra, why) in [
            (json!({ "location": long }), "location too long"),
            (json!({ "salary_minor": -1, "salary_currency": "GBP" }), "negative salary"),
            (json!({ "salary_minor": 100 }), "salary with no currency"),
        ] {
            assert_eq!(request.post("/api/workers").json(&hire(extra)).await.status_code(), 422, "{why}");
        }

        // Restating: a blank name, a share outside 1 to 100, a long location and a negative salary are refused; a location
        // is set and the manager cleared.
        let worker = seed_worker!(&request, &org, format!("WR-{tag}-A"), None).await;
        let url = format!("/api/workers/{worker}");
        for (body, why) in [
            (json!({ "display_name": " " }), "blank name"),
            (json!({ "fte_percent": 0 }), "no share"),
            (json!({ "location": long }), "long location"),
            (json!({ "salary_minor": -5 }), "negative salary"),
        ] {
            assert_eq!(request.put(&url).json(&body).await.status_code(), 422, "{why}");
        }
        let set: Value = request.put(&url).json(&json!({ "location": "Cardiff", "clear_manager": true })).await.json();
        assert_eq!(set["location"], "Cardiff", "{set}");

        // Status: on leave and back, then offboarding into retirement.
        activate!(&request, &worker).await;
        for to in ["on_leave", "active", "offboarding", "retired"] {
            let r = request.post(&format!("{url}/status")).json(&json!({ "to": to })).await;
            assert_eq!(r.status_code(), 200, "{to}: {}", r.text());
        }

        // Benefits: a negative cost is refused; an enrolment ending before it starts, and a second one, are refused.
        let plan = |worker_cost: i64| {
            json!({ "name": format!("Plan {tag}"), "kind": "health", "provider": "Insurer",
                    "worker_cost_minor": worker_cost, "employer_cost_minor": 100, "currency": "GBP" })
        };
        assert_eq!(request.post("/api/benefit-plans").json(&plan(-1)).await.status_code(), 422);
        let made: Value = request.post("/api/benefit-plans").json(&plan(50)).await.json();
        let member = seed_worker!(&request, &org, format!("WR-{tag}-B"), None).await;
        let enroll = format!("/api/workers/{member}/benefit-enrollments");
        let reversed = json!({ "plan_pid": made["pid"], "starts_on": "2026-06-01", "ends_on": "2026-01-01" });
        assert_eq!(request.post(&enroll).json(&reversed).await.status_code(), 422);
        let good = json!({ "plan_pid": made["pid"], "starts_on": "2026-06-01" });
        let first = request.post(&enroll).json(&good).await;
        assert_eq!(first.status_code(), 200, "{}", first.text());
        assert_eq!(request.post(&enroll).json(&good).await.status_code(), 422, "already enrolled");
    })
    .await;
}
