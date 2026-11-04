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

/// A deployer's contracted day of six hours for the length of a test, removed again however it ends
/// (WPM-T405). Read on every calculation and every timesheet read; these tests run one at a time.
struct SixHourDay;

impl SixHourDay {
    fn new() -> Self {
        unsafe { std::env::set_var("WPM_CONTRACTED_DAY_MINUTES", "360") };
        Self
    }
}

impl Drop for SixHourDay {
    fn drop(&mut self) {
        unsafe { std::env::remove_var("WPM_CONTRACTED_DAY_MINUTES") };
    }
}

/// Record a time entry and approve it.
async fn approved_time(request: &loco_rs::TestServer, worker: &str, body: Value) {
    let entry: Value = request
        .post(&format!("/api/workers/{worker}/time-entries"))
        .json(&body)
        .await
        .json();
    request
        .post(&format!(
            "/api/time-entries/{}/approve",
            entry["pid"].as_str().expect("time entry pid")
        ))
        .await
        .assert_status_ok();
}

/// The derived overtime minutes per day in a worker's timesheet.
async fn timesheet_overtime(request: &loco_rs::TestServer, worker: &str) -> Vec<(String, i64)> {
    let listed: Value = request
        .get(&format!(
            "/api/workers/{worker}/time-entries?from=2026-07-01&to=2026-07-31"
        ))
        .await
        .json();
    listed["overtime"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| {
            (
                d["worked_on"].as_str().unwrap().to_string(),
                d["overtime_minutes"].as_i64().unwrap(),
            )
        })
        .collect()
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
// WPM-T405 (follows WPM-T401's "Not tested"): with WPM_CONTRACTED_DAY_MINUTES set to 360, the
// timesheet's derived overtime and the payslip's overtime line both count from the deployer's day,
// scaled by FTE, not from the 450-minute example.
async fn overtime_counts_from_a_configured_contracted_day() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let full = seed_worker!(&request, &org, "E-2301", Some(3_600_000)).await;
        let half: Value = request
            .post("/api/workers")
            .json(&json!({
                "person_ref": crate::requests::a_person(),
                "organization_ref": org,
                "worker_number": "E-2302",
                "display_name": "Test Worker E-2302",
                "employment_type": "permanent",
                "department": "engineering",
                "job_title": "Engineer",
                "salary_minor": 3_600_000,
                "salary_currency": "GBP",
                "fte_percent": 50,
                "hired_on": "2026-01-05",
            }))
            .await
            .json();
        let half = half["pid"].as_str().unwrap().to_string();
        activate!(&request, &full).await;
        activate!(&request, &half).await;
        // Full time: 600 regular minutes one day, 30 explicit overtime minutes the next.
        approved_time(
            &request,
            &full,
            json!({ "worked_on": "2026-07-06", "minutes": 600 }),
        )
        .await;
        approved_time(
            &request,
            &full,
            json!({ "worked_on": "2026-07-07", "minutes": 30, "kind": "overtime" }),
        )
        .await;
        // Half time: 300 regular minutes in a day.
        approved_time(
            &request,
            &half,
            json!({ "worked_on": "2026-07-06", "minutes": 300 }),
        )
        .await;

        // The example day first, so the configured day is seen to change the answer.
        assert_eq!(
            timesheet_overtime(&request, &full).await,
            vec![
                ("2026-07-06".to_string(), 150),
                ("2026-07-07".to_string(), 30)
            ]
        );
        assert_eq!(
            timesheet_overtime(&request, &half).await,
            vec![("2026-07-06".to_string(), 75)]
        );

        let _day = SixHourDay::new();
        // Timesheet: 600 - 360 = 240, the explicit 30 as recorded; half time 300 - 180 = 120.
        assert_eq!(
            timesheet_overtime(&request, &full).await,
            vec![
                ("2026-07-06".to_string(), 240),
                ("2026-07-07".to_string(), 30)
            ]
        );
        assert_eq!(
            timesheet_overtime(&request, &half).await,
            vec![("2026-07-06".to_string(), 120)]
        );

        // Payroll: the same minutes feed the overtime line, and the answer names the day in force.
        let (run_pid, status, body) = calculate_july(&request, &org).await;
        assert_eq!(status, 200, "{body}");
        let calculated: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(calculated["payslips"], 2);
        assert_eq!(calculated["tax_stub"]["contracted_day_minutes"], 360);
        // The contracted day is its own setting: the tax stub is still the example.
        assert_eq!(calculated["tax_stub"]["source"], "example");
        assert_eq!(
            calculated["tax_stub"]["month_minutes"],
            rules::MONTH_MINUTES
        );
        let payslips: Value = request
            .get(&format!("/api/payroll-runs/{run_pid}/payslips"))
            .await
            .json();
        let gross = |worker: &str| {
            payslips
                .as_array()
                .unwrap()
                .iter()
                .find(|s| s["worker_pid"].as_str() == Some(worker))
                .unwrap()["gross_minor"]
                .as_i64()
                .unwrap()
        };
        let full_base = rules::monthly_base_minor(3_600_000, 100).unwrap();
        let half_base = rules::monthly_base_minor(3_600_000, 50).unwrap();
        let full_expected =
            full_base + rules::overtime_pay_minor(full_base, 270, rules::MONTH_MINUTES).unwrap();
        let half_expected =
            half_base + rules::overtime_pay_minor(half_base, 120, rules::MONTH_MINUTES).unwrap();
        assert_eq!(gross(&full), full_expected);
        assert_eq!(gross(&half), half_expected);
        // Not what the example day would have paid (150 + 30 and 75 minutes).
        assert_ne!(
            gross(&full),
            full_base + rules::overtime_pay_minor(full_base, 180, rules::MONTH_MINUTES).unwrap()
        );
        assert_ne!(
            gross(&half),
            half_base + rules::overtime_pay_minor(half_base, 75, rules::MONTH_MINUTES).unwrap()
        );
    })
    .await;
}

// WPM-R218, WPM-D156, WPM-D160 (WPM-T415): the advisory minimum-wage check follows each worker's place of work. Six payslips for July,
// each with 160 recorded hours: pay well above, just above (inside the margin), and below the floor; a worker whose place of work is
// not recorded; a jurisdiction recorded as setting no floor; and one that is recorded but configures nothing. Unknown is never ok,
// "none by law" is its own answer, nothing is refused, and the audit entry holds counts and no worker.
#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn the_minimum_wage_check_is_advisory_and_follows_the_place_of_work() {
    use sea_orm::{ConnectionTrait, Statement};
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        // Annual salaries of 36 000.00, 24 960.00 and 12 000.00 pay 3 000.00, 2 080.00 and 1 000.00 for a month: 18.75, 13.00 and
        // 6.25 an hour over 160 hours. The floor below is 12.71.
        let mut slips = Vec::new();
        for (number, salary) in [
            ("MW-ABOVE", 3_600_000),
            ("MW-NEAR", 2_496_000),
            ("MW-BELOW", 1_200_000),
            ("MW-NOWHERE", 3_600_000),
            ("MW-NOFLOOR", 3_600_000),
            ("MW-BLANK", 3_600_000),
        ] {
            let pid = seed_worker!(&request, &org, number, Some(salary)).await;
            activate!(&request, &pid).await;
            let mut day = chrono::NaiveDate::from_ymd_opt(2026, 7, 1).unwrap();
            for _ in 0..20 {
                request
                    .post(&format!("/api/workers/{pid}/time-entries"))
                    .json(&json!({ "worked_on": day, "minutes": 480 }))
                    .await
                    .assert_status_ok();
                day += chrono::Duration::days(1);
            }
            slips.push((number, pid));
        }
        let pid_of = |number: &str| slips.iter().find(|(n, _)| *n == number).unwrap().1.clone();

        for body in [
            json!({ "organization_ref": org, "code": "XA", "layer": "national" }),
            json!({ "organization_ref": org, "code": "XB", "layer": "national" }),
            json!({ "organization_ref": org, "code": "XC", "layer": "national" }),
        ] {
            request.post("/api/jurisdictions").json(&body).await.assert_status_ok();
        }
        for (code, key, value) in [
            ("XA", "minimum_wage.currency", json!(["GBP"])),
            ("XA", "minimum_wage.hourly_minor.adult", json!(1271)),
            ("XA", "minimum_wage.near_margin_bp", json!(500)),
            ("XB", "minimum_wage.none_by_law", json!(true)),
        ] {
            request
                .post("/api/jurisdiction-parameters")
                .json(&json!({ "organization_ref": org, "code": code, "key": key, "value": value,
                               "effective_from": "2000-01-01", "source": "synthetic test profile", "status": "example" }))
                .await
                .assert_status_ok();
        }
        for (number, code) in [
            ("MW-ABOVE", "XA"),
            ("MW-NEAR", "XA"),
            ("MW-BELOW", "XA"),
            ("MW-NOFLOOR", "XB"),
            ("MW-BLANK", "XC"),
        ] {
            request
                .put(&format!("/api/workers/{}", pid_of(number)))
                .json(&json!({ "work_jurisdiction": code }))
                .await
                .assert_status_ok();
        }

        let run: Value = request
            .post("/api/payroll-runs")
            .json(&json!({ "organization_ref": org, "period_start": "2026-07-01", "period_end": "2026-07-31" }))
            .await
            .json();
        let run_pid = run["pid"].as_str().unwrap().to_string();
        request.post(&format!("/api/payroll-runs/{run_pid}/calculate")).await.assert_status_ok();

        let check: Value = request
            .get(&format!("/api/payroll-runs/{run_pid}/minimum-wage-check"))
            .await
            .json();
        assert_eq!(check["indicative"], true);
        assert!(check["note"].as_str().unwrap().contains("not a statement that an employer complies"));
        assert_eq!(check["payslips"], 6);
        assert_eq!(check["counts"]["ok"], 1);
        assert_eq!(check["counts"]["near"], 1);
        assert_eq!(check["counts"]["below"], 1);
        assert_eq!(check["counts"]["none_by_law"], 1);
        assert_eq!(check["counts"]["unknown"], 2);
        assert_eq!(check["unknown_reasons"]["no_work_jurisdiction"], 1);
        assert_eq!(check["unknown_reasons"]["no_floor_configured"], 1);
        let outcome = |number: &str| -> (String, Value) {
            let row = check["workers"]
                .as_array()
                .unwrap()
                .iter()
                .find(|w| w["worker_pid"] == pid_of(number).as_str())
                .unwrap();
            (row["outcome"].as_str().unwrap().to_string(), row["reason"].clone())
        };
        assert_eq!(outcome("MW-ABOVE").0, "ok");
        assert_eq!(outcome("MW-NEAR").0, "near");
        assert_eq!(outcome("MW-BELOW").0, "below");
        assert_eq!(outcome("MW-NOFLOOR").0, "none_by_law");
        assert_eq!(outcome("MW-NOWHERE"), ("unknown".to_string(), json!("no_work_jurisdiction")));
        assert_eq!(outcome("MW-BLANK"), ("unknown".to_string(), json!("no_floor_configured")));
        // One floor for everyone: no age bands, so no birth date was needed and none is returned.
        assert!(check["workers"][0]["band"] == "adult");
        let text = check.to_string();
        assert!(!text.contains("birth") && !text.contains("\"age\""), "no birth date or age in the answer: {text}");

        // Advisory: the run is unchanged and can still be approved.
        request.post(&format!("/api/payroll-runs/{run_pid}/approve")).await.assert_status_ok();

        // The audit entry holds the counts and no worker.
        let counted: i64 = ctx
            .db
            .query_one_raw(Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                format!("SELECT count(*) AS n FROM audit_logs WHERE entity_pid = '{run_pid}' AND action = 'minimum_wage_checked' AND snapshot->'counts'->>'below' = '1'"),
            ))
            .await
            .unwrap()
            .unwrap()
            .try_get("", "n")
            .unwrap();
        assert_eq!(counted, 1);
        for (_, pid) in &slips {
            let leaked: i64 = ctx
                .db
                .query_one_raw(Statement::from_string(
                    sea_orm::DatabaseBackend::Postgres,
                    format!("SELECT count(*) AS n FROM audit_logs WHERE action = 'minimum_wage_checked' AND snapshot::text LIKE '%{pid}%'"),
                ))
                .await
                .unwrap()
                .unwrap()
                .try_get("", "n")
                .unwrap();
            assert_eq!(leaked, 0, "a worker is named in the audit entry");
        }
    })
    .await;
}

// WPM-R219, WPM-D157, WPM-D160, WPM-D162 (WPM-T418): the advisory overtime check. Weekly bands follow the worker's place of work and the
// status a person recorded on the seat they hold; the premium is the extra over the base rate and no payslip changes. An exempt seat
// is `exempt` (with the weekly-floor flag), a seat nobody recorded and a worker with no seat are `unknown`, a place that sets none by
// law says so, and a place with no bands says `no_limit_configured`. Never a silent zero.
#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
#[allow(clippy::too_many_lines)] // one scenario read top to bottom
async fn the_overtime_premium_is_in_gross_and_the_check_follows_the_seat_and_the_place() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let mut pids = std::collections::BTreeMap::new();
        for (number, salary, code, status) in [
            ("OT-NONEX", 3_600_000, "XA", Some("non_exempt")),
            ("OT-EXEMPT", 2_400_000, "XA", Some("exempt")),
            ("OT-UNREC", 3_600_000, "XA", Some("unknown")),
            ("OT-NOSEAT", 3_600_000, "XA", None),
            ("OT-NONE", 3_600_000, "XB", Some("non_exempt")),
            ("OT-BLANK", 3_600_000, "XC", Some("non_exempt")),
        ] {
            let pid = seed_worker!(&request, &org, number, Some(salary)).await;
            activate!(&request, &pid).await;
            request
                .put(&format!("/api/workers/{pid}"))
                .json(&json!({ "work_jurisdiction": code }))
                .await
                .assert_status_ok();
            if let Some(status) = status {
                let seat: Value = request
                    .post("/api/positions")
                    .json(&json!({ "organization_ref": org, "cost_centre": "CC-OT", "title": number,
                                   "budget_fte_percent": 100, "funding_kind": "core", "overtime_status": status }))
                    .await
                    .json();
                let seat = seat["pid"].as_str().unwrap().to_string();
                request
                    .post(&format!("/api/positions/{seat}/status"))
                    .json(&json!({ "to": "approved" }))
                    .await
                    .assert_status_ok();
                request
                    .put(&format!("/api/positions/{seat}/holder"))
                    .json(&json!({ "worker_pid": pid }))
                    .await
                    .assert_status_ok();
            }
            pids.insert(number, pid);
        }
        // Monday 2026-07-06 to Friday 2026-07-10: nine hours a day, forty-five in the week.
        for day in 6..=10 {
            approved_time(
                &request,
                &pids["OT-NONEX"],
                json!({ "worked_on": format!("2026-07-{day:02}"), "minutes": 540 }),
            )
            .await;
        }
        for body in [
            json!({ "organization_ref": org, "code": "XA", "layer": "national" }),
            json!({ "organization_ref": org, "code": "XB", "layer": "national" }),
            json!({ "organization_ref": org, "code": "XC", "layer": "national" }),
        ] {
            request.post("/api/jurisdictions").json(&body).await.assert_status_ok();
        }
        for (code, key, value) in [
            ("XA", "overtime.bands", json!(["week:2400:15000"])),
            ("XA", "overtime.exempt_salary_floor_minor_weekly", json!(100_000)),
            ("XB", "overtime.none_by_law", json!(true)),
        ] {
            request
                .post("/api/jurisdiction-parameters")
                .json(&json!({ "organization_ref": org, "code": code, "key": key, "value": value,
                               "effective_from": "2000-01-01", "source": "synthetic test profile", "status": "example" }))
                .await
                .assert_status_ok();
        }
        let run: Value = request
            .post("/api/payroll-runs")
            .json(&json!({ "organization_ref": org, "period_start": "2026-07-01", "period_end": "2026-07-31" }))
            .await
            .json();
        let run_pid = run["pid"].as_str().unwrap().to_string();
        request.post(&format!("/api/payroll-runs/{run_pid}/calculate")).await.assert_status_ok();
        let before: Value = request.get(&format!("/api/payroll-runs/{run_pid}/payslips")).await.json();

        let check: Value = request
            .get(&format!("/api/payroll-runs/{run_pid}/overtime-check"))
            .await
            .json();
        assert_eq!(check["indicative"], true);
        let of = |number: &str| {
            check["workers"]
                .as_array()
                .unwrap()
                .iter()
                .find(|w| w["worker_pid"].as_str() == Some(pids[number].as_str()))
                .cloned()
                .unwrap()
        };
        let non_exempt = of("OT-NONEX");
        assert_eq!(non_exempt["outcome"], "derived", "{non_exempt}");
        assert_eq!(non_exempt["overtime_minutes"], 300, "five hours past forty");
        let base = rules::monthly_base_minor(3_600_000, 100).unwrap();
        let expected = rules::overtime_pay_minor(base, 300, rules::MONTH_MINUTES).unwrap() / 2;
        let premium = non_exempt["premium_minor"].as_i64().unwrap();
        assert!((premium - expected).abs() <= 1, "half the base rate on 300 minutes: {premium} vs {expected}");
        let exempt = of("OT-EXEMPT");
        assert_eq!(exempt["outcome"], "exempt");
        assert_eq!(exempt["below_exempt_floor"], true, "461.54 a week is under 1000.00");
        assert_eq!(of("OT-UNREC")["outcome"], "unknown");
        assert_eq!(of("OT-UNREC")["reason"], "status_not_recorded");
        assert_eq!(of("OT-NOSEAT")["reason"], "no_position");
        assert_eq!(of("OT-NONE")["outcome"], "none_by_law");
        assert_eq!(of("OT-BLANK")["outcome"], "no_limit_configured");
        assert_eq!(check["counts"]["derived"], 1);
        assert!(!check.to_string().contains("OT-NONEX"), "no worker number or name");

        // The check moves no payslip, and the calculation that came first already holds the premium (WPM-D169): in gross, shown as its
        // own line, and for nobody who could not be derived.
        let after: Value = request.get(&format!("/api/payroll-runs/{run_pid}/payslips")).await.json();
        assert_eq!(before, after);
        let slip = |number: &str| {
            before
                .as_array()
                .unwrap()
                .iter()
                .find(|s| s["worker_pid"].as_str() == Some(pids[number].as_str()))
                .cloned()
                .unwrap()
        };
        let paid = slip("OT-NONEX");
        assert_eq!(paid["statutory_overtime_minor"].as_i64(), Some(premium), "the same premium the check shows");
        // Nine hours a day against the example 7.5 hour contracted day: 450 minutes of contractual overtime, paid at the base rate. The
        // statutory premium is the extra over that rate, so the two add and never pay the same minute twice.
        let contractual = rules::overtime_pay_minor(base, 450, rules::MONTH_MINUTES).unwrap();
        assert_eq!(
            paid["gross_minor"].as_i64(),
            Some(base + contractual + premium),
            "the premium is in gross, beside the contractual overtime"
        );
        for other in ["OT-EXEMPT", "OT-UNREC", "OT-NOSEAT", "OT-NONE", "OT-BLANK"] {
            assert_eq!(slip(other)["statutory_overtime_minor"], 0, "{other} adds nothing");
        }
        assert_eq!(slip("OT-UNREC")["gross_minor"].as_i64(), Some(base), "an unrecorded post is never paid a guess");
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn employer_cost_lines_follow_the_place_the_class_and_the_wage_base_and_never_touch_net() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let mut pids = std::collections::BTreeMap::new();
        for (number, code) in [("EC-XA", Some("XA")), ("EC-XB", Some("XB")), ("EC-NOPLACE", None)] {
            let pid = seed_worker!(&request, &org, number, Some(3_600_000)).await;
            activate!(&request, &pid).await;
            if let Some(code) = code {
                request
                    .put(&format!("/api/workers/{pid}"))
                    .json(&json!({ "work_jurisdiction": code }))
                    .await
                    .assert_status_ok();
            }
            pids.insert(number, pid);
        }
        for code in ["XA", "XB"] {
            request
                .post("/api/jurisdictions")
                .json(&json!({ "organization_ref": org, "code": code, "layer": "national" }))
                .await
                .assert_status_ok();
        }
        let set = |code: &'static str, key: &'static str, value: Value, from: &'static str| {
            let request = &request;
            let org = org.clone();
            async move {
                request
                    .post("/api/jurisdiction-parameters")
                    .json(&json!({ "organization_ref": org, "code": code, "key": key, "value": value,
                                   "effective_from": from, "source": "synthetic test profile", "status": "example" }))
                    .await
            }
        };
        // XA has three lines: a capped rate, a class-keyed rate with no class set yet, and a flat rate. XB configures nothing.
        let lines = json!(["unemployment:100:450000", "injury@low:50:0", "injury@high:400:0", "pension:300:0"]);
        set("XA", "employer_contribution.lines", lines, "2000-01-01").await.assert_status_ok();
        // A line that does not read is refused when it is set, not skipped later.
        assert_eq!(set("XA", "employer_contribution.lines", json!(["tax:10001:0"]), "2000-02-01").await.status_code(), 422);

        let calc = |start: &'static str, end: &'static str| {
            let request = &request;
            let org = org.clone();
            async move {
                let run: Value = request
                    .post("/api/payroll-runs")
                    .json(&json!({ "organization_ref": org, "period_start": start, "period_end": end }))
                    .await
                    .json();
                let pid = run["pid"].as_str().unwrap().to_string();
                request.post(&format!("/api/payroll-runs/{pid}/calculate")).await.assert_status_ok();
                let slips: Value = request.get(&format!("/api/payroll-runs/{pid}/payslips")).await.json();
                (pid, slips)
            }
        };
        let slip_of = |slips: &Value, number: &str| {
            slips
                .as_array()
                .unwrap()
                .iter()
                .find(|s| s["worker_pid"].as_str() == Some(pids[number].as_str()))
                .cloned()
                .unwrap()
        };
        let line = |slip: &Value, name: &str| {
            slip["employer_costs"].as_array().unwrap().iter().find(|c| c["name"] == name).cloned()
        };
        let base = rules::monthly_base_minor(3_600_000, 100).unwrap();

        // May: no class is set, so the injury line is said to be unresolved and costs nothing; it is never the nearest rate.
        let (may, slips) = calc("2026-05-01", "2026-05-31").await;
        let a = slip_of(&slips, "EC-XA");
        assert_eq!(line(&a, "unemployment").unwrap()["amount_minor"], base / 100, "one percent, under the wage base");
        assert_eq!(line(&a, "pension").unwrap()["amount_minor"], base * 3 / 100);
        assert_eq!(line(&a, "injury").unwrap()["unresolved"], true);
        assert_eq!(a["employer_cost_minor"].as_i64(), Some(base / 100 + base * 3 / 100));
        // Never deducted from net: no deduction carries an employer line, and gross less deductions is still net.
        let deducted: i64 = a["deductions"].as_array().unwrap().iter().map(|d| d["amount_minor"].as_i64().unwrap()).sum();
        assert_eq!(a["gross_minor"].as_i64().unwrap() - deducted, a["net_minor"].as_i64().unwrap());
        assert!(!a["deductions"].to_string().contains("unemployment") && !a["deductions"].to_string().contains("pension"));
        // A place with nothing configured, and a worker with no place, have no lines: nothing is made up.
        for number in ["EC-XB", "EC-NOPLACE"] {
            let s = slip_of(&slips, number);
            assert_eq!((s["employer_cost_minor"].as_i64(), s["employer_costs"].clone()), (Some(0), json!([])), "{number}");
        }
        request.post(&format!("/api/payroll-runs/{may}/approve")).await.assert_status_ok();

        // June: the deployer sets a class; the injury rate follows it. The wage base of 4 500.00 had 3 000.00 used in May, so only
        // 1 500.00 of June's pay is costed for the capped line.
        set("XA", "employer_contribution.class", json!(["low"]), "2026-06-01").await.assert_status_ok();
        let (_, slips) = calc("2026-06-01", "2026-06-30").await;
        let a = slip_of(&slips, "EC-XA");
        let injury = line(&a, "injury").unwrap();
        assert_eq!((injury["class"].clone(), injury["amount_minor"].as_i64()), (json!("low"), Some(base / 200)));
        let capped = line(&a, "unemployment").unwrap();
        assert_eq!((capped["base_minor"].as_i64(), capped["amount_minor"].as_i64()), (Some(150_000), Some(1_500)));
    })
    .await;
}
