//! Pay positions: a worker's band and step (person and HR only), the eligibility
//! date worked out from the years on the step, audited without the band or step,
//! exported and erased with the person, and a reminder that names no pay.

use chrono::{Duration, Months, Utc};
use loco_rs::testing::prelude::*;
use sea_orm::{ConnectionTrait, Statement};
use serde_json::{Value, json};
use serial_test::serial;
use workforce_planning_management_service::app::App;
use workforce_planning_management_service::tasks::pay_progression_reminders::send_reminders;

use super::{activate, an_org, seed_worker};

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn a_pay_position_is_validated_dated_audited_without_pay_and_erased() {
    request::<App, _, _>(|request, ctx| async move {
        let org = an_org();
        let worker = seed_worker!(&request, &org, "PP-1", None).await;
        activate!(&request, &worker).await;
        let url = format!("/api/workers/{worker}/pay-position");
        let today = Utc::now().date_naive();

        let none: Value = request.get(&url).await.json();
        assert_eq!(none["pay_position"], Value::Null);

        // Unknown scale, band, a step off the band, a step on a single rate,
        // and a future date: all refused.
        for bad in [
            json!({ "scale": "afc-england-2026-27", "band": "6", "step": 1 }),
            json!({ "scale": "afc-wales-2026-27", "band": "14", "step": 1 }),
            json!({ "scale": "afc-wales-2026-27", "band": "6", "step": 4 }),
            json!({ "scale": "afc-wales-2026-27", "band": "6", "step": 0 }),
            json!({ "scale": "afc-wales-2026-27", "band": "2", "step": 2 }),
            json!({ "scale": "afc-wales-2026-27", "band": "6", "step": 1, "step_since": "2999-01-01" }),
        ] {
            assert_eq!(request.put(&url).json(&bad).await.status_code(), 422, "{bad}");
        }

        // Band 6, entry step, reached ten days short of two years ago: not yet
        // eligible, and the date is exactly two years after step_since.
        let since = today - Months::new(24) + Duration::days(10);
        let set: Value = request
            .put(&url)
            .json(&json!({ "scale": "afc-wales-2026-27", "band": "6", "step": 1,
                           "step_since": since.to_string() }))
            .await
            .json();
        let p = &set["pay_position"];
        assert_eq!(p["band"], "6");
        assert_eq!(p["annual_minor"], 4_055_900);
        assert_eq!(p["currency"], "GBP");
        assert_eq!(p["progression"]["kind"], "not_yet");
        assert_eq!(p["progression"]["days_remaining"], 10);
        assert_eq!(
            p["progression"]["eligible_on"],
            (since + Months::new(24)).to_string()
        );
        assert_eq!(p["progression"]["next_annual_minor"], 4_280_500);

        // Moving to the top step: nowhere to go.
        let top: Value = request
            .put(&url)
            .json(&json!({ "scale": "afc-wales-2026-27", "band": "6", "step": 3 }))
            .await
            .json();
        assert_eq!(top["pay_position"]["progression"]["kind"], "at_top");
        assert_eq!(top["pay_position"]["annual_minor"], 4_884_100);

        // The audit says it was set, never to what.
        let audit = serde_json::to_string(
            &request.get(&format!("/api/audits/{worker}")).await.json::<Value>(),
        )
        .unwrap();
        assert!(audit.contains("pay_position_set"), "{audit}");
        assert!(!audit.contains("afc-wales") && !audit.contains("4884100"), "{audit}");

        // It travels with the person.
        let export: Value = request
            .get(&format!("/api/workers/{worker}/subject-access"))
            .await
            .json();
        assert_eq!(export["pay_position"][0]["band"], "6");

        // Clearing removes it; clearing nothing is a 404.
        request.delete(&url).await.assert_status_success();
        assert_eq!(request.delete(&url).await.status_code(), 404);

        // Set again, then erase with the person: the row is gone.
        request
            .put(&url)
            .json(&json!({ "scale": "afc-wales-2026-27", "band": "5", "step": 2 }))
            .await
            .assert_status_ok();
        for to in ["offboarding", "terminated"] {
            request
                .post(&format!("/api/workers/{worker}/status"))
                .json(&json!({ "to": to }))
                .await
                .assert_status_ok();
        }
        assert_eq!(
            request.post(&format!("/api/workers/{worker}/erase")).await.status_code(),
            200
        );
        let row = ctx
            .db
            .query_one_raw(Statement::from_string(
                ctx.db.get_database_backend(),
                format!("SELECT count(*) FROM worker_pay_positions WHERE worker_pid = '{worker}'"),
            ))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(row.try_get_by_index::<i64>(0).unwrap(), 0);
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn eligibility_reminders_name_no_pay_and_are_sent_once() {
    request::<App, _, _>(|request, ctx| async move {
        let org = an_org();
        let soon = seed_worker!(&request, &org, "PP-2", None).await;
        let later = seed_worker!(&request, &org, "PP-3", None).await;
        for w in [&soon, &later] {
            activate!(&request, w).await;
        }
        let today = Utc::now().date_naive();
        // `soon` becomes eligible in 10 days; `later` in about a year.
        for (who, since) in [
            (&soon, today - Months::new(24) + Duration::days(10)),
            (&later, today - Months::new(12)),
        ] {
            request
                .put(&format!("/api/workers/{who}/pay-position"))
                .json(
                    &json!({ "scale": "afc-wales-2026-27", "band": "6", "step": 1,
                               "step_since": since.to_string() }),
                )
                .await
                .assert_status_ok();
        }
        let told = |who: String| {
            let request = &request;
            async move {
                let n: Value = request
                    .get(&format!("/api/workers/{who}/notifications"))
                    .await
                    .json();
                n.as_array()
                    .unwrap()
                    .iter()
                    .filter(|n| n["kind"] == "pay_step_due")
                    .cloned()
                    .collect::<Vec<_>>()
            }
        };

        // A 30-day window catches `soon` only, once.
        let to = today + Duration::days(30);
        assert_eq!(send_reminders(&ctx, today, to).await.unwrap(), 1);
        assert_eq!(told(soon.clone()).await.len(), 1);
        assert_eq!(told(later.clone()).await.len(), 0);
        assert_eq!(
            send_reminders(&ctx, today, to).await.unwrap(),
            0,
            "idempotent"
        );
        assert_eq!(told(soon.clone()).await.len(), 1);

        // The message says *eligible* and a date — no band, step or amount.
        let note = &told(soon).await[0];
        let text = note["body"].as_str().unwrap();
        assert!(text.contains("eligible"), "{text}");
        let all = serde_json::to_string(note).unwrap();
        for leak in ["afc-wales", "band", "40559", "42805", "£"] {
            assert!(!all.contains(leak), "{leak} leaked: {all}");
        }
        // A window that excludes the date tells nobody new.
        assert_eq!(
            send_reminders(&ctx, today + Duration::days(40), today + Duration::days(60))
                .await
                .unwrap(),
            0
        );
    })
    .await;
}
