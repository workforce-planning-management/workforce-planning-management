//! Pay position analysis (WPM-R154, WPM-D108): compa-ratio and range penetration per group, small groups withheld,
//! a total held back when it would give a withheld group away, nothing named.

use sea_orm::ConnectionTrait;
use serde_json::{Value, json};
use serial_test::serial;

use super::{a_person, an_org};

/// Hire `n` workers in `department`, all on band 6 and paid `salary` minor units (a currency of GBP).
async fn staff(
    request: &loco_rs::TestServer,
    ctx: &loco_rs::app::AppContext,
    org: &str,
    department: &str,
    n: usize,
    salary: Option<i64>,
) {
    for i in 0..n {
        let response = request
            .post("/api/workers")
            .json(&json!({
                "person_ref": a_person(), "organization_ref": org,
                "worker_number": format!("PR-{}-{i}", uuid::Uuid::new_v4().simple()),
                "display_name": format!("Pay Range {department} {i}"), "employment_type": "permanent",
                "department": department, "job_title": "Carer", "hired_on": "2020-01-05",
                "fte_percent": 100, "location": "Leeds",
            }))
            .await;
        assert_eq!(response.status_code(), 200, "{}", response.text());
        let created: Value = response.json();
        let pid = created["pid"].as_str().expect("pid");
        if let Some(pay) = salary {
            ctx.db
                .execute_unprepared(&format!(
                    "UPDATE workers SET salary_minor = {pay}, salary_currency = 'GBP' WHERE pid = '{pid}'"
                ))
                .await
                .unwrap();
        }
        let placed = request
            .put(&format!("/api/workers/{pid}/pay-position"))
            .json(&json!({ "scale": "national-2026-27", "band": "6", "step": 1, "step_since": "2024-01-01" }))
            .await;
        assert_eq!(placed.status_code(), 200, "{}", placed.text());
    }
}

fn group<'a>(body: &'a Value, by: &str, key: &str) -> &'a Value {
    body["report"]["groupings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|g| g["by"] == by)
        .and_then(|g| g["groups"].as_array().unwrap().iter().find(|x| x["key"] == key))
        .unwrap_or_else(|| panic!("{by}/{key}"))
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn pay_is_placed_in_its_band_with_small_groups_withheld_and_a_total_that_gives_none_away() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        // Band 6 runs 4 055 900 to 4 884 100 (midpoint 4 470 000). Twelve are at the midpoint, eleven at the top.
        staff(&request, &ctx, &org, "care", 12, Some(4_470_000)).await;
        staff(&request, &ctx, &org, "legal", 11, Some(4_884_100)).await;
        // Three in admin are paid, and one in care has no pay recorded.
        staff(&request, &ctx, &org, "admin", 3, Some(4_470_000)).await;
        staff(&request, &ctx, &org, "care", 1, None).await;

        let url = format!("/api/reports/pay-range?organization={org}&as_of=2026-10-24&cutoff=3");
        let body: Value = request.get(&url).await.json();

        let care = group(&body, "department", "care");
        assert_eq!(care["figure"]["kind"], "shown");
        assert_eq!(care["figure"]["value"]["workers"], 12, "the worker without pay is not in it");
        assert_eq!(care["figure"]["value"]["compa_mean_bp"], 10_000);
        assert_eq!(care["figure"]["value"]["penetration"]["value"]["mean_bp"], 5_000);

        let legal = group(&body, "department", "legal");
        assert_eq!(legal["figure"]["value"]["compa_mean_bp"], 10_926, "4 884 100 over 4 470 000");
        assert_eq!(legal["figure"]["value"]["penetration"]["value"]["mean_bp"], 10_000, "the top of the band");

        // Three is under the floor of ten: withheld, with no count and no figure.
        let admin = group(&body, "department", "admin");
        assert_eq!(admin["figure"]["kind"], "withheld");
        assert!(admin["figure"].get("value").is_none());

        // The total of 26 less care and legal would give admin away, so it is not shown, and so no outlier is called.
        assert_eq!(body["report"]["overall"]["kind"], "unknown");
        assert_eq!(body["report"]["overall"]["reason"], "would_reveal");
        assert_eq!(care["outlier"]["kind"], "unknown");
        assert_eq!(body["report"]["excluded"]["no_pay"]["kind"], "under_floor");
        assert_eq!(body["currency"], "GBP");

        // Without admin the total shows, and the two departments sit either side of it by more than three points.
        let org2 = an_org();
        staff(&request, &ctx, &org2, "care", 12, Some(4_470_000)).await;
        staff(&request, &ctx, &org2, "legal", 11, Some(4_884_100)).await;
        let url2 = format!("/api/reports/pay-range?organization={org2}&as_of=2026-10-24&cutoff=3");
        let body: Value = request.get(&url2).await.json();
        assert_eq!(body["report"]["overall"]["value"]["workers"], 23);
        assert_eq!(body["report"]["overall"]["value"]["compa_mean_bp"], 10_443);
        assert_eq!(group(&body, "department", "legal")["outlier"]["kind"], "above");
        assert_eq!(group(&body, "department", "care")["outlier"]["kind"], "below");
        let calm: Value = request.get(&format!("/api/reports/pay-range?organization={org2}&as_of=2026-10-24")).await.json();
        assert_eq!(group(&calm, "department", "legal")["outlier"]["kind"], "no", "ten points is the default");

        // Nobody is named, no pid or number is carried, and the cut-off is bounded.
        let text = body.to_string();
        assert!(!text.contains("Pay Range"), "no name");
        assert!(!text.contains("PR-"), "no worker number");
        assert_eq!(request.get(&format!("/api/reports/pay-range?organization={org2}&cutoff=0")).await.status_code(), 422);
        assert_eq!(request.get(&format!("/api/reports/pay-range?organization={org2}&as_of=2999-01-01")).await.status_code(), 422);
        assert_eq!(request.get("/api/reports/pay-range?organization=nonsense").await.status_code(), 422);
    })
    .await;
}
