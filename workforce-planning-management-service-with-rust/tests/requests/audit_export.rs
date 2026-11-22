//! The audit trail as JSON Lines for a security monitoring system (WPM-R122, WPM-T197): resumable,
//! oldest first, capped, and without snapshots.

use jiff::Timestamp;
use serde_json::Value;
use serial_test::serial;
use workforce_planning_management::absent::AbsentIsEmpty as _;

use super::{a_person, an_org, seed_worker};

fn lines(text: &str) -> Vec<Value> {
    text.lines()
        .map(|l| serde_json::from_str(l).expect("each line is JSON"))
        .collect()
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn the_trail_is_exported_in_order_and_resumes_where_it_stopped() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        let _ = a_person();
        for number in ["e1", "e2", "e3"] {
            seed_worker!(&request, &org, number, None::<i64>).await;
        }
        let first = request.get("/api/audits/export?limit=2").await;
        assert_eq!(first.status_code(), 200);
        let content_type = first
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .or_empty()
            .to_string();
        assert!(
            content_type.starts_with("application/x-ndjson"),
            "{content_type}"
        );
        let next = first
            .headers()
            .get("x-next-after")
            .and_then(|v| v.to_str().ok())
            .expect("a resume point")
            .to_string();
        let page = lines(&first.text());
        assert_eq!(page.len(), 2);
        assert_eq!(page[1]["chain_seq"].to_string(), next);
        assert!(
            page[0]["chain_seq"].as_i64() < page[1]["chain_seq"].as_i64(),
            "oldest first"
        );
        for entry in &page {
            for key in [
                "at",
                "entity",
                "entity_pid",
                "action",
                "prev_hash",
                "entry_hash",
            ] {
                assert!(entry.get(key).is_some(), "{key} missing: {entry}");
            }
            assert!(
                entry.get("snapshot").is_none(),
                "no snapshot leaves: {entry}"
            );
        }

        let rest = request
            .get(&format!("/api/audits/export?after={next}"))
            .await;
        let more = lines(&rest.text());
        assert!(!more.is_empty());
        assert!(
            more[0]["chain_seq"].as_i64() > page[1]["chain_seq"].as_i64(),
            "resumes after, no repeat"
        );

        // nothing new: an empty page that keeps the resume point
        let last = more.last().expect("some")["chain_seq"].to_string();
        let none = request
            .get(&format!("/api/audits/export?after={last}"))
            .await;
        assert_eq!(none.text(), "");
        assert_eq!(
            none.headers()
                .get("x-next-after")
                .and_then(|v| v.to_str().ok()),
            Some(last.as_str())
        );
        assert_eq!(
            request
                .get("/api/audits/export?after=-1")
                .await
                .status_code(),
            422
        );
    })
    .await;
}

/// The department-scoped query (spec `audit.md`): only entries since the time whose snapshot names the department, newest first.
#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test -- --ignored`"]
async fn the_department_query_keeps_to_the_department_and_the_time() {
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let since = workforce_planning_management::dates::rfc3339_seconds(
            Timestamp::now() - jiff::SignedDuration::from_mins(1),
        );
        let first = seed_worker!(&request, &org, "d1", None::<i64>).await;
        let second = seed_worker!(&request, &org, "d2", None::<i64>).await;

        // More newer rows than the old ten-times-cap pre-filter used to read. Filtering after that
        // cap returned an empty answer even though both engineering rows are in the time window.
        sqlx::query(
            "INSERT INTO audit_logs (entity, entity_pid, action, snapshot) \
             SELECT 'noise', $1, 'read', jsonb_build_object('department', 'elsewhere') \
             FROM generate_series(1, 2001)",
        )
        .bind(uuid::Uuid::new_v4())
        .execute(workforce_planning_management::db::pool(&ctx))
        .await
        .expect("newer unrelated audit activity");

        let found = request
            .get(&format!("/api/audits?department=engineering&since={since}"))
            .await;
        assert_eq!(found.status_code(), 200);
        let found: Value = found.json();
        let entries = found.as_array().expect("an array");
        assert!(!entries.is_empty());
        for entry in entries {
            assert_eq!(entry["snapshot"]["department"], "engineering", "{entry}");
        }
        let at = |pid: &str| {
            entries
                .iter()
                .position(|e| e["entity_pid"] == pid)
                .unwrap_or_else(|| panic!("{pid} is in the department's trail"))
        };
        assert!(at(&second) < at(&first), "newest first");
        let ids: Vec<i64> = entries.iter().filter_map(|e| e["id"].as_i64()).collect();
        assert!(ids.windows(2).all(|w| w[0] > w[1]), "newest first: {ids:?}");

        let elsewhere: Value = request
            .get(&format!(
                "/api/audits?department=nowhere-at-all&since={since}"
            ))
            .await
            .json();
        assert_eq!(elsewhere, serde_json::json!([]));

        let later = workforce_planning_management::dates::rfc3339_seconds(
            Timestamp::now() + jiff::SignedDuration::from_hours(1),
        );
        let none: Value = request
            .get(&format!("/api/audits?department=engineering&since={later}"))
            .await
            .json();
        assert_eq!(none, serde_json::json!([]), "nothing before the time");
    })
    .await;
}
