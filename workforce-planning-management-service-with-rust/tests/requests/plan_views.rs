//! The plan forecast and cost views count the employed workforce in groups (WPM-D95): the answers for a
//! department with every kind of person in it, pinned so that how they are counted can change.

use sea_orm::ConnectionTrait;
use serde_json::{Value, json};
use serial_test::serial;

use super::an_org;

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
#[allow(clippy::too_many_lines)] // one seeded department, both views
async fn plan_views_count_every_kind_of_person_once() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let dept = format!("ops-{tag}");
        let skill: Value = request
            .post("/api/skills")
            .json(&json!({ "name": format!("Plan-Skill {tag}"), "category": "technical" }))
            .await
            .json();
        let profile: Value = request
            .post("/api/role-profiles")
            .json(&json!({ "job_title": format!("Operator {tag}") }))
            .await
            .json();
        request
            .put(&format!("/api/role-profiles/{}/requirements", profile["pid"].as_str().unwrap()))
            .json(&json!({ "skill_pid": skill["pid"], "min_proficiency": 3, "importance": "critical" }))
            .await
            .assert_status_ok();
        // (type, fte, salary, currency, hired, skill level)
        let people = [
            ("permanent", 100, 3_000_000, "GBP", "2026-01-05", Some(4)),
            ("permanent", 100, 3_200_000, "GBP", "2026-01-05", Some(3)),
            ("permanent", 50, 2_000_000, "GBP", "2026-01-05", Some(2)),
            ("fixed_term", 100, 3_000_000, "GBP", "2026-01-05", Some(5)), // ends 2027-03-31
            ("contractor", 80, 4_000_000, "GBP", "2026-01-05", None), // ends 2027-12-31
            ("permanent", 100, 9_000_000, "USD", "2026-01-05", Some(3)),
            ("permanent", 100, 3_000_000, "GBP", "2026-01-05", Some(5)), // left in June 2026
            ("permanent", 100, 3_000_000, "GBP", "2030-01-01", Some(5)), // not yet started
        ];
        let mut who = Vec::new();
        for (n, (kind, fte, salary, currency, hired, level)) in people.iter().enumerate() {
            let created: Value = request
                .post("/api/workers")
                .json(&json!({
                    "person_ref": super::a_person(), "organization_ref": org,
                    "worker_number": format!("PV-{tag}-{n}"), "display_name": format!("Test Worker PV-{n}"),
                    "employment_type": kind, "fte_percent": fte, "department": dept, "job_title": "Operator",
                    "salary_minor": salary, "salary_currency": currency, "hired_on": hired,
                    "engagement_ends_on": match *kind { "fixed_term" => Some("2027-03-31"), "contractor" => Some("2027-12-31"), _ => None },
                }))
                .await
                .json();
            let pid = created["pid"].as_str().unwrap_or_else(|| panic!("{created}")).to_string();
            super::activate!(&request, &pid).await;
            if let Some(level) = level {
                request
                    .put(&format!("/api/workers/{pid}/skills"))
                    .json(&json!({ "skill_pid": skill["pid"], "proficiency": level }))
                    .await
                    .assert_status_ok();
            }
            who.push(pid);
        }
        ctx.db.execute_unprepared(&format!(
            "UPDATE workers SET terminated_on = '2026-06-30' WHERE pid = '{}';
             INSERT INTO conversion_plans (pid, worker_pid, intent, target_on, status)
               VALUES (gen_random_uuid(), '{}', 'convert', '2027-01-31', 'proposed');
             INSERT INTO conversion_plans (pid, worker_pid, intent, target_on, status)
               VALUES (gen_random_uuid(), '{}', 'convert', '2027-01-31', 'approved');",
            who[6], who[3], who[4])).await.unwrap();

        let plan: Value = request
            .post("/api/workforce-plans")
            .json(&json!({ "name": format!("Views {tag}"), "organization_ref": org,
                           "horizon_start": "2026-01-01", "horizon_end": "2027-12-31",
                           "attrition_bp": 0, "on_cost_bp": 0 }))
            .await
            .json();
        let pid = plan["pid"].as_str().unwrap();
        request
            .put(&format!("/api/workforce-plans/{pid}/demand-lines"))
            .json(&json!({ "department": dept, "role_profile_pid": profile["pid"],
                           "target_on": "2027-06-30", "target_headcount": 9 }))
            .await
            .assert_status_ok();

        let forecast: Value = request.get(&format!("/api/workforce-plans/{pid}/forecast")).await.json();
        let d = &forecast["departments"][0];
        assert_eq!(d["opening_headcount"], 6, "those who left and those not yet started are not employed");
        assert_eq!(d["projected_supply"], 5, "the fixed-term engagement ends before the date");
        assert_eq!(d["projected_supply_fte"], 4.3);
        assert_eq!(d["headcount_gap"], 4);
        let c = &d["competency_gaps"][0];
        assert_eq!((c["proficient_now"].as_u64(), c["reskill_pool"].as_u64(), c["shortfall"].as_u64()), (Some(4), Some(1), Some(5)));
        assert_eq!(d["conversions"]["agreed"], json!({ "headcount": 1, "fte": 0.8 }));
        assert_eq!(d["conversions"]["hoped"], json!({ "headcount": 1, "fte": 1.0 }));
        assert_eq!(d["contract_fte_now"]["contractor"], json!({ "headcount": 1, "fte": 0.8 }));
        let cost: Value = request.get(&format!("/api/workforce-plans/{pid}/cost")).await.json();
        let g = &cost["groups"][0];
        assert_eq!(g["hires_needed"], 3);
        assert_eq!(g["unit_cost_source"], "department_average");
        assert_eq!(g["unit_cost_minor"], 3_040_000, "five GBP salaries; the dollar salary is not mixed in");
        assert_eq!(g["annual_cost_minor"], 9_120_000);
    })
    .await;
}
