//! The human capital report (WPM-R147, WPM-D100): figures from measures WPM already works out, small groups withheld,
//! what it cannot compute unknown, nothing named.

use serde_json::{Value, json};
use serial_test::serial;

use super::{a_person, an_org};
use sqlx::Row;

async fn hire(
    request: &axa_test::TestServer,
    ctx: &axa::AppContext,
    org: &str,
    kind: &str,
    n: usize,
    left_on: Option<&str>,
) {
    for i in 0..n {
        let created: Value = request
            .post("/api/workers")
            .json(&json!({
                "person_ref": a_person(), "organization_ref": org,
                "worker_number": format!("HC-{kind}-{}-{i}", left_on.is_some()),
                "display_name": format!("Human Capital {kind} {i}"), "employment_type": kind,
                "department": "engineering", "job_title": "Engineer", "hired_on": "2020-01-05",
                "fte_percent": 80, "engagement_ends_on": (kind == "contractor").then_some("2030-01-01"),
            }))
            .await
            .json();
        if let Some(day) = left_on {
            let pid = created["pid"].as_str().expect("pid");
            sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
                "UPDATE workers SET terminated_on = '{day}' WHERE pid = '{pid}'"
            )))
            .execute(workforce_planning_management::db::pool(ctx))
            .await
            .unwrap();
        }
    }
}

fn figure<'a>(body: &'a Value, key: &str) -> &'a Value {
    body["report"]["figures"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["key"] == key)
        .unwrap_or_else(|| panic!("{key}"))
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn the_report_counts_withholds_and_says_what_it_cannot_know() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        // Twelve permanent staff at 80 percent still here, three contractors, ten permanent staff who left in March.
        hire(&request, &ctx, &org, "permanent", 12, None).await;
        hire(&request, &ctx, &org, "contractor", 3, None).await;
        hire(&request, &ctx, &org, "permanent", 10, Some("2026-03-01")).await;
        let url = format!("/api/reports/human-capital?organization={org}&as_of=2026-10-24");
        let body: Value = request.get(&url).await.json();

        let headcount = figure(&body, "headcount");
        assert_eq!(headcount["status"], "shown");
        assert_eq!(headcount["value"], 15);
        assert_eq!(headcount["mapping"]["esrs_2026"], "S1-5");
        assert_eq!(headcount["mapping"]["esrs_2023"], "S1-6");
        assert_eq!(figure(&body, "headcount_permanent")["value"], 12);
        assert_eq!(figure(&body, "fte_permanent")["value"], 960);
        assert_eq!(figure(&body, "headcount_contractor")["status"], "withheld", "three is under the floor");
        assert_eq!(figure(&body, "fte_contractor")["status"], "withheld", "FTE would give the count back");

        // Ten left; opening headcount 25, closing 15: 10 / 20 = 5000 bp.
        assert_eq!(figure(&body, "leavers")["value"], 10);
        assert_eq!(figure(&body, "turnover_rate")["value"], 5_000);
        assert_eq!(figure(&body, "turnover_rate")["unit"], "basis_points");
        assert_eq!(figure(&body, "turnover_rate")["period"]["from"], "2025-10-25");

        // Nothing filled, nothing enrolled, no plans: unknown, never zero.
        assert_eq!(figure(&body, "time_to_fill_median")["status"], "unknown");
        assert_eq!(figure(&body, "training_completion")["reason"], "nothing_to_divide");
        assert_eq!(figure(&body, "training_hours")["reason"], "not_recorded");
        assert_eq!(figure(&body, "collective_agreement_coverage")["reason"], "no_record");
        assert_eq!(figure(&body, "pay_gap")["reason"], "read_elsewhere");
        assert_eq!(body["report"]["gaps"].as_array().unwrap().len(), 6);

        // It holds no one's name or id.
        let text = body.to_string();
        assert!(!text.contains("Human Capital") && !text.contains("person:"), "{text}");

        // Another organization sees none of these people.
        let other: Value = request
            .get(&format!("/api/reports/human-capital?organization={}&as_of=2026-10-24", an_org()))
            .await
            .json();
        assert_eq!(figure(&other, "headcount")["value"], 0);

        // The future is refused, and so is a missing organization.
        let future = request.get(&format!("/api/reports/human-capital?organization={org}&as_of=2999-01-01")).await;
        assert_eq!(future.status_code(), 422);
        assert_eq!(request.get("/api/reports/human-capital?organization=nope").await.status_code(), 422);

        // The read is audited, with the date and nothing else.
        let rows = sqlx::query("SELECT snapshot::text AS snapshot FROM audit_logs WHERE action = 'human_capital_report_read'").fetch_all(workforce_planning_management::db::pool(&ctx))
            .await
            .unwrap();
        assert!(!rows.is_empty());
        for row in rows {
            let snapshot: Option<String> = row.try_get("snapshot").unwrap();
            assert!(!snapshot.unwrap_or_default().contains("headcount"));
        }
    })
    .await;
}

// WPM-T425: the report counts incidents by class and the pay periods with a payslip under a legal floor. Monthly floors keep the setup to
// salaries: ten people, of whom one is paid under the floor in the first two of ten months, so two of ten runs are `below`.
#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn the_report_counts_incidents_and_pay_periods_below_a_floor() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        hire(&request, &ctx, &org, "permanent", 10, None).await;
        let url = format!("/api/reports/human-capital?organization={org}&as_of=2026-10-24");

        // Nothing recorded yet: unknown, never zero.
        let before: Value = request.get(&url).await.json();
        assert_eq!(figure(&before, "injuries_total")["reason"], "no_record");
        assert_eq!(figure(&before, "pay_periods_below_minimum_wage")["reason"], "nothing_to_divide");

        let incident = |class: &str, day: &str| {
            json!({ "organization": org, "jurisdiction": "XA", "kind": "accident", "occurred_at": format!("{day}T09:00:00Z"), "class": class })
        };
        for (class, day, n) in [
            ("no_absence", "2026-03-04", 12),
            ("absence", "2026-04-04", 11),
            ("serious", "2026-05-04", 1),
            ("fatal", "2020-05-04", 7),
        ] {
            for _ in 0..n {
                request.post("/api/injury-records").json(&incident(class, day)).await.assert_status_ok();
            }
        }

        sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
                "UPDATE workers SET status = 'active', fte_percent = 100, salary_minor = 3600000, salary_currency = 'GBP', work_jurisdiction = 'XA' WHERE organization_ref = '{org}'"
            ))).execute(workforce_planning_management::db::pool(&ctx))
            .await
            .unwrap();
        request
            .post("/api/jurisdictions")
            .json(&json!({ "organization_ref": org, "code": "XA", "layer": "national" }))
            .await
            .assert_status_ok();
        for (key, value) in [
            ("minimum_wage.currency", json!(["GBP"])),
            ("minimum_wage.monthly_minor.adult", json!(200_000)),
        ] {
            request
                .post("/api/jurisdiction-parameters")
                .json(&json!({ "organization_ref": org, "code": "XA", "key": key, "value": value,
                               "effective_from": "2000-01-01", "source": "synthetic test profile", "status": "example" }))
                .await
                .assert_status_ok();
        }
        let low = |salary: i64| {
            format!("UPDATE workers SET salary_minor = {salary} WHERE pid = (SELECT pid FROM workers WHERE organization_ref = '{org}' ORDER BY id LIMIT 1)")
        };
        sqlx::raw_sql(sqlx::AssertSqlSafe(&*low(1_200_000))).execute(workforce_planning_management::db::pool(&ctx)).await.unwrap();
        // Eleven runs, December to October; the period's first month is November, so all fall inside it.
        for (i, (year, month)) in (0..11).map(|i| if i == 0 { (2025, 12) } else { (2026, i) }).enumerate() {
            if i == 2 {
                sqlx::raw_sql(sqlx::AssertSqlSafe(&*low(3_600_000))).execute(workforce_planning_management::db::pool(&ctx)).await.unwrap();
            }
            let run: Value = request
                .post("/api/payroll-runs")
                .json(&json!({ "organization_ref": org, "period_start": format!("{year}-{month:02}-01"), "period_end": format!("{year}-{month:02}-20") }))
                .await
                .json();
            let pid = run["pid"].as_str().unwrap();
            request.post(&format!("/api/payroll-runs/{pid}/calculate")).await.assert_status_ok();
            // The last run stays calculated: it has not been approved, so it does not count.
            if i < 10 {
                request.post(&format!("/api/payroll-runs/{pid}/approve")).await.assert_status_ok();
            }
        }

        let body: Value = request.get(&url).await.json();
        assert_eq!(figure(&body, "injuries_no_absence")["value"], 12);
        assert_eq!(figure(&body, "injuries_no_absence")["unit"], "incidents");
        assert_eq!(figure(&body, "injuries_serious")["status"], "withheld", "one is under the floor");
        assert_eq!(figure(&body, "injuries_absence")["status"], "withheld", "or the total would give the one back");
        assert_eq!(figure(&body, "injuries_total")["value"], 24, "outside the period is left out");
        assert_eq!(figure(&body, "injuries_fatal")["value"], 0, "the record is in use, so this zero is true");
        assert_eq!(figure(&body, "injuries_total")["mapping"]["esrs_2026"], "S1-13");

        let share = figure(&body, "pay_periods_below_minimum_wage");
        assert_eq!(share["value"], 2_000, "two of ten; the unapproved run is left out");
        assert_eq!(share["unit"], "basis_points");
        assert!(share["mapping"]["esrs_topic"].is_null(), "a legal floor is not the adequate wage");
        assert!(share["exclusions"].to_string().contains("does not show that an employer complies"));
        assert_eq!(body["report"]["gaps"].as_array().unwrap().len(), 6);
        let text = body.to_string();
        assert!(!text.contains("Human Capital") && !text.contains("birth"), "{text}");
    })
    .await;
}
