//! Coverage evidence (WPM-R220, WPM-D154, WPM-T420): a record per organization and jurisdiction, checked for what its kind needs, with a
//! standing today. No end date is said and never read as valid; a retired record leaves the list.

use chrono::{Duration, Utc};
use serde_json::{Value, json};
use serial_test::serial;

use super::an_org;

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn coverage_is_recorded_checked_stood_and_retired() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let today = Utc::now().date_naive();
        let post = |body: Value| {
            let request = &request;
            async move { request.post("/api/coverage-evidence").json(&body).await }
        };
        // A private policy without its insurer and reference is refused; so is a cover with no currency, and a bad code.
        let policy = json!({ "organization": org, "jurisdiction": "XA", "kind": "private_insurance" });
        assert_eq!(post(policy.clone()).await.status_code(), 422);
        let mut cover = json!({ "organization": org, "jurisdiction": "XA", "kind": "self_insured", "minimum_cover_minor": 500_000_000 });
        assert_eq!(post(cover.clone()).await.status_code(), 422, "a cover needs a currency");
        cover["currency"] = json!("GBP");
        assert_eq!(post(json!({ "organization": org, "jurisdiction": "x", "kind": "self_insured" })).await.status_code(), 422);
        assert_eq!(post(json!({ "organization": org, "jurisdiction": "XA", "kind": "mystery" })).await.status_code(), 422);

        let mut ending = policy.clone();
        ending["insurer"] = json!("Acme Mutual");
        ending["policy_reference"] = json!("EL-12345");
        ending["minimum_cover_minor"] = json!(500_000_000);
        ending["currency"] = json!("GBP");
        ending["valid_to"] = json!((today + Duration::days(20)).to_string());
        ending["certificate_ref"] = json!("dms://certificates/el-2026");
        let made = post(ending).await;
        assert_eq!(made.status_code(), 200, "{}", made.text());
        let ending_pid = made.json::<Value>()["pid"].as_str().unwrap().to_string();
        let open_ended = post(json!({ "organization": org, "jurisdiction": "XB", "kind": "statutory_scheme" })).await;
        assert_eq!(open_ended.status_code(), 200, "{}", open_ended.text());
        let expired = post(json!({ "organization": org, "jurisdiction": "XC", "kind": "self_insured",
            "valid_to": (today - Duration::days(1)).to_string() })).await;
        assert_eq!(expired.status_code(), 200);

        let list: Value = request.get(&format!("/api/coverage-evidence?organization={org}")).await.json();
        let of = |code: &str| list.as_array().unwrap().iter().find(|r| r["jurisdiction"] == code).cloned().unwrap();
        assert_eq!(of("XA")["standing"], "ending_soon");
        assert_eq!(of("XA")["days_left"], 20);
        assert_eq!(of("XA")["certificate_ref"], "dms://certificates/el-2026");
        assert_eq!(of("XB")["standing"], "no_end_date", "no end date is not valid");
        assert_eq!(of("XC")["standing"], "expired");
        // A narrower window moves the same record out of "ending soon".
        let narrow: Value = request.get(&format!("/api/coverage-evidence?organization={org}&days=10")).await.json();
        assert_eq!(narrow.as_array().unwrap().iter().find(|r| r["jurisdiction"] == "XA").unwrap()["standing"], "in_force");

        let retired = request.post(&format!("/api/coverage-evidence/{ending_pid}/retire")).await;
        assert_eq!(retired.status_code(), 200);
        let again = request.post(&format!("/api/coverage-evidence/{ending_pid}/retire")).await;
        assert_eq!(again.json::<Value>()["retired_on"], retired.json::<Value>()["retired_on"], "the first date is kept");
        let after: Value = request.get(&format!("/api/coverage-evidence?organization={org}")).await.json();
        assert!(after.as_array().unwrap().iter().all(|r| r["pid"] != ending_pid.as_str()));
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn expiry_reminders_tell_hr_once_and_name_no_policy() {
    use workforce_planning_management::tasks::coverage_expiry_reminders::send_reminders;

    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let today = Utc::now().date_naive();
        let day = |n: i64| (today + Duration::days(n)).to_string();
        let hr = super::seed_worker!(&request, &org, format!("CV-H-{tag}"), None).await;
        super::activate!(&request, &hr).await;
        sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
                "INSERT INTO organization_memberships (pid, person_ref, organization_ref, worker_pid, role, starts_on) \
                 VALUES ('{}', 'person:{}', '{org}', '{hr}', 'hr_admin', '2023-01-05')",
                uuid::Uuid::new_v4(),
                uuid::Uuid::new_v4()
            ))).execute(workforce_planning_management::db::pool(&ctx))
            .await
            .unwrap();
        let add = |code: &str, to: Option<String>| {
            let (request, org, code) = (&request, org.clone(), code.to_string());
            async move {
                let mut body = json!({ "organization": org, "jurisdiction": code, "kind": "private_insurance",
                    "insurer": "SecretInsurer", "policy_reference": "POLICY-SECRET" });
                if let Some(to) = to {
                    body["valid_to"] = json!(to);
                }
                let r = request.post("/api/coverage-evidence").json(&body).await;
                assert_eq!(r.status_code(), 200, "{}", r.text());
                r.json::<Value>()["pid"].as_str().unwrap().to_string()
            }
        };
        let soon = add("RA", Some(day(30))).await;
        let _far = add("RB", Some(day(120))).await;
        let _lapsed = add("RC", Some(day(-10))).await;
        let _old = add("RD", Some(day(-200))).await;
        let _open = add("RE", None).await;

        send_reminders(&ctx, today, 60).await.unwrap();
        let notes = || {
            let (request, hr) = (&request, hr.clone());
            async move {
                serde_json::to_string(&request.get(&format!("/api/workers/{hr}/notifications")).await.json::<Value>()).unwrap()
            }
        };
        let text = notes().await;
        assert!(text.contains(&format!("for RA ends on {}", day(30))), "{text}");
        assert!(text.contains(&format!("for RC ended on {}", day(-10))), "{text}");
        for absent in ["for RB", "for RD", "for RE"] {
            assert!(!text.contains(absent), "{absent}: outside the window, a past fact, or no end date");
        }
        assert!(!text.contains("SecretInsurer") && !text.contains("POLICY-SECRET"), "no insurer or policy in a notification");
        assert_eq!(text.matches("coverage_expiring").count(), 1);

        // Idempotent, and a retired record is not reminded.
        send_reminders(&ctx, today, 60).await.unwrap();
        assert_eq!(notes().await.matches("coverage_expiring").count(), 1);
        request.post(&format!("/api/coverage-evidence/{soon}/retire")).await.assert_status_ok();
        sqlx::raw_sql(sqlx::AssertSqlSafe(format!("DELETE FROM coverage_reminders WHERE coverage_pid = '{soon}'"))).execute(workforce_planning_management::db::pool(&ctx))
            .await
            .unwrap();
        send_reminders(&ctx, today, 60).await.unwrap();
        assert_eq!(notes().await.matches("coverage_expiring").count(), 1, "retired: no new reminder");
    })
    .await;
}
