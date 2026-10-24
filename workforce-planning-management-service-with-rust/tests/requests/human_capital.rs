//! The human capital report (WPM-R147, WPM-D100): figures from measures WPM already works out, small groups withheld,
//! what it cannot compute unknown, nothing named.

use sea_orm::ConnectionTrait;
use serde_json::{Value, json};
use serial_test::serial;

use super::{a_person, an_org};

async fn hire(
    request: &loco_rs::TestServer,
    ctx: &loco_rs::app::AppContext,
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
            ctx.db
                .execute_unprepared(&format!(
                    "UPDATE workers SET terminated_on = '{day}' WHERE pid = '{pid}'"
                ))
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
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
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
        let rows = ctx
            .db
            .query_all_raw(sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                "SELECT snapshot::text AS snapshot FROM audit_logs WHERE action = 'human_capital_report_read'".to_string(),
            ))
            .await
            .unwrap();
        assert!(!rows.is_empty());
        for row in rows {
            let snapshot: Option<String> = row.try_get("", "snapshot").unwrap();
            assert!(!snapshot.unwrap_or_default().contains("headcount"));
        }
    })
    .await;
}
