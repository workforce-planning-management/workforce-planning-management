//! The audit trail as JSON Lines for a security monitoring system (WPM-R122, WPM-T197): resumable,
//! oldest first, capped, and without snapshots.

use serde_json::Value;
use serial_test::serial;

use super::{a_person, an_org, seed_worker};

fn lines(text: &str) -> Vec<Value> {
    text.lines()
        .map(|l| serde_json::from_str(l).expect("each line is JSON"))
        .collect()
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
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
            .unwrap_or_default()
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
