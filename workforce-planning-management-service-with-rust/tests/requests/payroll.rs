//! Payroll derivation (WPM-R13, WPM-R14): draft → calculate →
//! approve → paid with reconciled payslips, the approved-run
//! immutability, and the benchmark comparison flags.

use serde_json::{Value, json};
use serial_test::serial;
use workforce_planning_management::rules::payroll as rules;

use super::{activate, an_org, seed_worker};

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
// Two salaried workers (one with a pension enrolment and approved
// overtime) get reconciled payslips; approve freezes the run.
async fn payroll_run_derives_reconciled_payslips() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let plain = seed_worker!(&request, &org, "E-2001", Some(3_600_000)).await;
        let enrolled = seed_worker!(&request, &org, "E-2002", Some(4_800_000)).await;
        activate!(&request, &plain).await;
        activate!(&request, &enrolled).await;
        // A pension plan + enrolment for the second worker.
        let plan: Value = request
            .post("/api/benefit-plans")
            .json(&json!({
                "name": "Pension 5%", "kind": "pension", "provider": "Demo Provider",
                "worker_cost_minor": 15_000, "employer_cost_minor": 30_000,
                "currency": "GBP",
            }))
            .await
            .json();
        request
            .post(&format!("/api/workers/{enrolled}/benefit-enrollments"))
            .json(&json!({ "plan_pid": plan["pid"], "starts_on": "2026-01-01" }))
            .await
            .assert_status_ok();
        // Approved overtime inside the period: a full extra contracted
        // day (450 regular + 450 more = 450 overtime minutes).
        let entry: Value = request
            .post(&format!("/api/workers/{enrolled}/time-entries"))
            .json(&json!({ "worked_on": "2026-07-06", "minutes": 900 }))
            .await
            .json();
        request
            .post(&format!(
                "/api/time-entries/{}/approve",
                entry["pid"].as_str().unwrap()
            ))
            .await
            .assert_status_ok();
        // Unapproved time must NOT count: another 900-minute day left
        // in `recorded`.
        request
            .post(&format!("/api/workers/{enrolled}/time-entries"))
            .json(&json!({ "worked_on": "2026-07-07", "minutes": 900 }))
            .await
            .assert_status_ok();
        // Draft run over July.
        let run: Value = request
            .post("/api/payroll-runs")
            .json(&json!({
                "organization_ref": org,
                "period_start": "2026-07-01", "period_end": "2026-07-31",
            }))
            .await
            .json();
        let run_pid = run["pid"].as_str().unwrap().to_string();
        // Approve from draft is an illegal transition.
        let premature = request
            .post(&format!("/api/payroll-runs/{run_pid}/approve"))
            .await;
        assert_eq!(premature.status_code(), 422);
        // Calculate.
        let calculated: Value = request
            .post(&format!("/api/payroll-runs/{run_pid}/calculate"))
            .await
            .json();
        assert_eq!(calculated["payslips"], 2);
        // The answer says whose figures the tax line used (WPM-T383).
        assert_eq!(calculated["tax_stub"]["source"], "example");
        assert_eq!(calculated["tax_stub"]["currency"], "GBP");
        assert_eq!(
            calculated["tax_stub"]["allowance_minor"],
            rules::TAX_FREE_MINOR
        );
        assert_eq!(calculated["tax_stub"]["contracted_day_minutes"], 450);
        let payslips: Value = request
            .get(&format!("/api/payroll-runs/{run_pid}/payslips"))
            .await
            .json();
        let payslips = payslips.as_array().unwrap();
        assert_eq!(payslips.len(), 2);
        for slip in payslips {
            let gross = slip["gross_minor"].as_i64().unwrap();
            let net = slip["net_minor"].as_i64().unwrap();
            let deductions: i64 = slip["deductions"]
                .as_array()
                .unwrap()
                .iter()
                .map(|d| d["amount_minor"].as_i64().unwrap())
                .sum();
            assert_eq!(net, gross - deductions, "payslip reconciles");
        }
        // The enrolled worker's slip: base 400000 + overtime
        // (450 min × 400000 / 9750) + pension deduction present.
        let enrolled_slip = payslips
            .iter()
            .find(|s| s["worker_pid"].as_str() == Some(enrolled.as_str()))
            .unwrap();
        let expected_base = 400_000;
        let expected_overtime =
            rules::overtime_pay_minor(expected_base, 450, rules::MONTH_MINUTES).unwrap();
        assert_eq!(
            enrolled_slip["gross_minor"].as_i64().unwrap(),
            expected_base + expected_overtime,
            "only approved overtime feeds the gross"
        );
        assert!(
            enrolled_slip["deductions"]
                .as_array()
                .unwrap()
                .iter()
                .any(|d| d["label"] == "Pension 5%"),
            "benefit worker-cost is a deduction line"
        );
        // Approve → immutable (no recalculate, no reopen); then paid.
        request
            .post(&format!("/api/payroll-runs/{run_pid}/approve"))
            .await
            .assert_status_ok();
        assert_eq!(
            request
                .post(&format!("/api/payroll-runs/{run_pid}/calculate"))
                .await
                .status_code(),
            422,
            "approved runs are immutable"
        );
        assert_eq!(
            request
                .post(&format!("/api/payroll-runs/{run_pid}/reopen"))
                .await
                .status_code(),
            422
        );
        request
            .post(&format!("/api/payroll-runs/{run_pid}/pay"))
            .await
            .assert_status_ok();
        // Self-service payslips list for the worker.
        let mine: Value = request
            .get(&format!("/api/workers/{enrolled}/payslips"))
            .await
            .json();
        assert_eq!(mine.as_array().unwrap().len(), 1);
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
// Benchmarks: band validation, and the comparison flags workers
// below/within/above without echoing salary amounts.
async fn benchmark_comparison_flags() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let low = seed_worker!(&request, &org, "E-3001", Some(2_000_000)).await;
        let high = seed_worker!(&request, &org, "E-3002", Some(9_900_000)).await;
        let bad_band = request
            .post("/api/benchmarks")
            .json(&json!({
                "job_title": "Engineer", "currency": "GBP",
                "min_minor": 500, "median_minor": 400, "max_minor": 600,
                "source": "survey", "as_of": "2026-04-01",
            }))
            .await;
        assert_eq!(bad_band.status_code(), 422, "min<=median<=max enforced");
        request
            .post("/api/benchmarks")
            .json(&json!({
                "job_title": "Engineer", "currency": "GBP",
                "min_minor": 3_000_000, "median_minor": 3_800_000, "max_minor": 4_600_000,
                "source": "survey", "as_of": "2026-04-01",
            }))
            .await
            .assert_status_ok();
        let comparison: Value = request
            .get(&format!("/api/benchmarks/comparison?organization={org}"))
            .await
            .json();
        let rows = comparison["rows"].as_array().unwrap();
        let flag_of = |pid: &str| {
            rows.iter()
                .find(|r| r["worker_pid"].as_str() == Some(pid))
                .and_then(|r| r["flag"].as_str().map(ToString::to_string))
        };
        assert_eq!(flag_of(&low).as_deref(), Some("below_min"));
        assert_eq!(flag_of(&high).as_deref(), Some("above_max"));
        // No salary amounts in the comparison payload.
        assert!(
            !comparison.to_string().contains("salary_minor"),
            "comparison echoes flags, not amounts"
        );
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
// A contractor is paid against an invoice, not through payroll (WPM-D58): with a rate recorded in
// the salary field they still get no payslip, while a permanent and a fixed-term worker do.
async fn a_contractor_gets_no_payslip() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let mut pids = Vec::new();
        for (number, kind, ends_on) in [
            ("E-3001", "permanent", None),
            ("E-3002", "fixed_term", Some("2027-12-31")),
            ("E-3003", "contractor", Some("2027-12-31")),
        ] {
            let created: Value = request
                .post("/api/workers")
                .json(&json!({
                    "person_ref": crate::requests::a_person(),
                    "organization_ref": org,
                    "worker_number": number,
                    "display_name": format!("Test Worker {number}"),
                    "employment_type": kind,
                    "department": "engineering",
                    "job_title": "Engineer",
                    "salary_minor": 3_600_000,
                    "salary_currency": "GBP",
                    "hired_on": "2026-01-05",
                    "engagement_ends_on": ends_on,
                }))
                .await
                .json();
            let pid = created["pid"].as_str().expect("worker pid").to_string();
            activate!(&request, &pid).await;
            pids.push(pid);
        }
        let run: Value = request
            .post("/api/payroll-runs")
            .json(&json!({
                "organization_ref": org,
                "period_start": "2026-07-01", "period_end": "2026-07-31",
            }))
            .await
            .json();
        let run_pid = run["pid"].as_str().unwrap().to_string();
        let calculated: Value = request
            .post(&format!("/api/payroll-runs/{run_pid}/calculate"))
            .await
            .json();
        assert_eq!(
            calculated["payslips"], 2,
            "the contractor is not paid through payroll"
        );
        let slips: Value = request
            .get(&format!("/api/payroll-runs/{run_pid}/payslips"))
            .await
            .json();
        let paid: Vec<&str> = slips
            .as_array()
            .unwrap()
            .iter()
            .map(|slip| slip["worker_pid"].as_str().unwrap())
            .collect();
        assert!(paid.contains(&pids[0].as_str()), "permanent is paid");
        assert!(paid.contains(&pids[1].as_str()), "fixed-term is paid");
        assert!(!paid.contains(&pids[2].as_str()), "contractor is not");
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
// WPM-T383: the stub is written for one currency. A run that would pay a worker in another
// currency is refused, names the worker and both currencies, and stays a draft with no payslip.
async fn the_payroll_stub_refuses_a_currency_it_was_not_written_for() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let pounds = seed_worker!(&request, &org, "E-2101", Some(3_600_000)).await;
        activate!(&request, &pounds).await;
        let yen: Value = request
            .post("/api/workers")
            .json(&json!({
                "person_ref": crate::requests::a_person(),
                "organization_ref": org,
                "worker_number": "E-2102",
                "display_name": "Test Worker E-2102",
                "employment_type": "permanent",
                "department": "engineering",
                "job_title": "Engineer",
                "salary_minor": 6_000_000,
                "salary_currency": "JPY",
                "hired_on": "2026-01-05",
            }))
            .await
            .json();
        let yen = yen["pid"].as_str().unwrap().to_string();
        activate!(&request, &yen).await;
        let run: Value = request
            .post("/api/payroll-runs")
            .json(&json!({
                "organization_ref": org,
                "period_start": "2026-07-01", "period_end": "2026-07-31",
            }))
            .await
            .json();
        let run_pid = run["pid"].as_str().unwrap().to_string();
        let refused = request
            .post(&format!("/api/payroll-runs/{run_pid}/calculate"))
            .await;
        assert_eq!(refused.status_code(), 422);
        let text = refused.text();
        assert!(
            text.contains("E-2102") && text.contains("GBP") && text.contains("JPY"),
            "{text}"
        );
        let after: Value = request
            .get(&format!("/api/payroll-runs/{run_pid}"))
            .await
            .json();
        assert_eq!(after["status"], "draft", "a refused run is not calculated");
        let payslips: Value = request
            .get(&format!("/api/payroll-runs/{run_pid}/payslips"))
            .await
            .json();
        assert_eq!(
            payslips.as_array().unwrap().len(),
            0,
            "no payslip is written"
        );
    })
    .await;
}

/// A deployer's yen stub for the length of a test, removed again however it ends (WPM-T401). The
/// settings are read on every calculation, and these tests run one at a time.
struct YenStub;

const YEN_STUB: [(&str, &str); 5] = [
    ("WPM_PAYROLL_STUB_CURRENCY", "JPY"),
    ("WPM_PAYROLL_STUB_ALLOWANCE_MINOR", "100000"),
    ("WPM_PAYROLL_STUB_RATE_PERCENT", "10"),
    ("WPM_PAYROLL_MONTH_MINUTES", "9600"),
    ("WPM_CONTRACTED_DAY_MINUTES", "480"),
];

impl YenStub {
    fn new() -> Self {
        for (name, value) in YEN_STUB {
            unsafe { std::env::set_var(name, value) };
        }
        Self
    }
}

impl Drop for YenStub {
    fn drop(&mut self) {
        for (name, _) in YEN_STUB {
            unsafe { std::env::remove_var(name) };
        }
    }
}

/// Open and calculate a July run for `org`: the run, the status code and the body.
async fn calculate_july(request: &loco_rs::TestServer, org: &str) -> (String, u16, String) {
    let run: Value = request
        .post("/api/payroll-runs")
        .json(&json!({
            "organization_ref": org,
            "period_start": "2026-07-01", "period_end": "2026-07-31",
        }))
        .await
        .json();
    let run_pid = run["pid"].as_str().unwrap().to_string();
    let response = request
        .post(&format!("/api/payroll-runs/{run_pid}/calculate"))
        .await;
    (run_pid, response.status_code().as_u16(), response.text())
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
// WPM-T401: a deployer's stub in another currency (yen, exponent 0) calculates, the answer says it is
// the deployer's, and a run that would pay someone in pounds under it is refused.
async fn a_configured_yen_stub_calculates_and_refuses_pounds() {
    crate::requests::request_open(|request, _ctx| async move {
        let _stub = YenStub::new();
        let org = an_org();
        let yen: Value = request
            .post("/api/workers")
            .json(&json!({
                "person_ref": crate::requests::a_person(),
                "organization_ref": org,
                "worker_number": "E-2201",
                "display_name": "Test Worker E-2201",
                "employment_type": "permanent",
                "department": "engineering",
                "job_title": "Engineer",
                "salary_minor": 6_000_000,
                "salary_currency": "JPY",
                "hired_on": "2026-01-05",
            }))
            .await
            .json();
        let yen = yen["pid"].as_str().unwrap().to_string();
        activate!(&request, &yen).await;
        let (run_pid, status, body) = calculate_july(&request, &org).await;
        assert_eq!(status, 200, "{body}");
        let calculated: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(calculated["payslips"], 1);
        assert_eq!(calculated["tax_stub"]["source"], "configured");
        assert_eq!(calculated["tax_stub"]["currency"], "JPY");
        assert_eq!(calculated["tax_stub"]["allowance_minor"], 100_000);
        assert_eq!(calculated["tax_stub"]["rate_percent"], 10);
        assert_eq!(calculated["tax_stub"]["month_minutes"], 9_600);
        assert_eq!(calculated["tax_stub"]["contracted_day_minutes"], 480);
        let payslips: Value = request
            .get(&format!("/api/payroll-runs/{run_pid}/payslips"))
            .await
            .json();
        let slip = &payslips.as_array().unwrap()[0];
        // 6,000,000 yen a year is 500,000 a month; tax is 10% of what is above 100,000.
        assert_eq!(slip["currency"], "JPY");
        assert_eq!(slip["gross_minor"], 500_000);
        let tax = slip["deductions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|d| d["label"] == "tax")
            .unwrap();
        assert_eq!(tax["amount_minor"], 40_000);
        assert_eq!(slip["net_minor"], 460_000);

        // Under the yen stub a worker paid in pounds is refused, naming both currencies; nothing written.
        let other = an_org();
        let pounds = seed_worker!(&request, &other, "E-2202", Some(3_600_000)).await;
        activate!(&request, &pounds).await;
        let (refused_pid, status, text) = calculate_july(&request, &other).await;
        assert_eq!(status, 422);
        assert!(
            text.contains("E-2202") && text.contains("JPY") && text.contains("GBP"),
            "{text}"
        );
        let after: Value = request
            .get(&format!("/api/payroll-runs/{refused_pid}"))
            .await
            .json();
        assert_eq!(after["status"], "draft");
    })
    .await;
}
