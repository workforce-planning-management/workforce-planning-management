//! The retention schedule per record kind (WPM-R91, WPM-D60).
//!
//! Its own test binary because the horizons are read from the environment and
//! the request suite does not set a per-kind override. One override is set
//! here: the `planning` kind keeps deleted rows for 100 calendar days, while
//! `recruitment` keeps its default of 180, so the same age is swept for one
//! kind and kept for the other.
//!
//! `#[ignore]`d: boots the app (needs PostgreSQL via `config/test.yaml` /
//! `DATABASE_URL`). Run with `cargo test --test retention_schedule -- --ignored`.

use loco_rs::testing::prelude::*;
use sea_orm::ConnectionTrait;
use serde_json::{Value, json};
use serial_test::serial;
use workforce_planning_management_service::app::App;

/// How many rows of `table` still exist (soft-deleted or not).
async fn count(ctx: &loco_rs::app::AppContext, table: &str) -> i64 {
    ctx.db
        .query_one_raw(sea_orm::Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            format!("SELECT COUNT(*) AS n FROM {table}"),
        ))
        .await
        .unwrap()
        .and_then(|row| row.try_get::<i64>("", "n").ok())
        .unwrap()
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test --test retention_schedule -- --ignored`"]
async fn each_kind_is_swept_at_its_own_horizon() {
    // `set_var` is `unsafe` in edition 2024; single-threaded setup before boot.
    unsafe {
        std::env::set_var("WPM_REQUIRE_AUTH", "0");
        std::env::set_var("WPM_RATE_LIMIT_PER_MINUTE", "0");
        std::env::set_var("WPM_RATE_LIMIT_SENSITIVE_PER_MINUTE", "0");
        std::env::remove_var("WPM_RETENTION_DAYS");
        std::env::set_var("WPM_RETENTION_PLANNING_DAYS", "100");
        // A junk value is ignored, not trusted.
        std::env::set_var("WPM_RETENTION_PAY_DAYS", "junk");
    }
    request::<App, _, _>(|request, ctx| async move {
        // ── The schedule states each kind, its tables and where its horizon came from.
        let schedule: Value = request.get("/api/retention/schedule").await.json();
        assert_eq!(schedule["floor_days"], 30);
        let kinds = schedule["kinds"].as_array().unwrap();
        assert_eq!(kinds.len(), 8);
        let kind = |name: &str| kinds.iter().find(|k| k["kind"] == name).unwrap();
        assert_eq!(kind("planning")["days"], 100);
        assert_eq!(kind("planning")["source"], "override");
        assert_eq!(kind("planning")["default_days"], 365);
        assert_eq!(kind("recruitment")["days"], 180);
        assert_eq!(kind("recruitment")["source"], "default");
        assert_eq!(kind("pay")["days"], 2190, "junk override ignored");
        assert_eq!(kind("pay")["source"], "default");
        let tables: usize = kinds
            .iter()
            .map(|k| k["tables"].as_array().unwrap().len())
            .sum();
        assert_eq!(tables, 56, "every swept table is in exactly one kind");

        // ── Two candidates (recruitment, 180) and two announcements (planning,
        // 100), each pair deleted 150 and 200 calendar days ago.
        let org = format!("organization:{}", uuid::Uuid::new_v4());
        let candidate = |name: &str| {
            let request = &request;
            let name = name.to_string();
            async move {
                request
                    .post("/api/candidates")
                    .json(&json!({ "display_name": name, "email": "a@example.com",
                                   "source": "referral",
                                   "person_ref": format!("person:{}", uuid::Uuid::new_v4()) }))
                    .await
                    .json::<Value>()["pid"]
                    .as_str()
                    .unwrap()
                    .to_string()
            }
        };
        let young_candidate = candidate("Young Candidate").await;
        let old_candidate = candidate("Old Candidate").await;
        let announcement = |title: &str| {
            let request = &request;
            let org = org.clone();
            let title = title.to_string();
            async move {
                request
                    .post("/api/announcements")
                    .json(&json!({ "organization_ref": org, "title": title, "body": "x" }))
                    .await
                    .json::<Value>()["pid"]
                    .as_str()
                    .unwrap()
                    .to_string()
            }
        };
        let young_post = announcement("Young post").await;
        let old_post = announcement("Old post").await;
        for (table, pid, age) in [
            ("candidates", &young_candidate, 150),
            ("candidates", &old_candidate, 200),
            ("announcements", &young_post, 150),
            ("announcements", &old_post, 200),
        ] {
            ctx.db
                .execute_unprepared(&format!(
                    "UPDATE {table} SET deleted_at = now() - interval '{age} days' \
                     WHERE pid = '{pid}'"
                ))
                .await
                .unwrap();
        }

        // ── The report counts only what is past its own table's horizon:
        // candidates at 200 (past 180), announcements at 150 and 200 (past 100).
        let report: Value = request.get("/api/retention").await.json();
        let past = &report["soft_deleted_past_horizon"];
        assert_eq!(past["candidates"], 1, "{report}");
        assert_eq!(past["announcements"], 2, "{report}");
        assert_eq!(report["horizons"]["planning"], 100);
        assert_eq!(report["horizons"]["recruitment"], 180);

        // ── The sweep deletes the same, and keeps the 150-day candidate.
        let sweep: Value = request.post("/api/retention/sweep").await.json();
        assert_eq!(sweep["deleted"]["candidates"], 1, "{sweep}");
        assert_eq!(sweep["deleted"]["announcements"], 2, "{sweep}");
        assert_eq!(count(&ctx, "announcements").await, 0);
        let remaining = ctx
            .db
            .query_one_raw(sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                format!("SELECT COUNT(*) AS n FROM candidates WHERE pid = '{young_candidate}'"),
            ))
            .await
            .unwrap()
            .and_then(|row| row.try_get::<i64>("", "n").ok())
            .unwrap();
        assert_eq!(remaining, 1, "inside the recruitment horizon, so kept");
    })
    .await;
}
