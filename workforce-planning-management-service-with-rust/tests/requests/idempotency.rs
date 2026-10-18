//! Idempotency keys (WPM-R132, WPM-D80): a repeat of the same request under the same key returns the
//! first response and runs nothing again; the same key with a different request is refused; a failure
//! is not kept; a request with no key is untouched; a stored response expires.

use serde_json::{Value, json};
use serial_test::serial;

use super::an_org;

const KEY: &str = "0123456789abcdef-test-key";

async fn count(request: &loco_rs::TestServer, org: &str, title: &str) -> usize {
    let feed: Value = request
        .get(&format!("/api/announcements?organization={org}"))
        .await
        .json();
    feed.as_array()
        .unwrap()
        .iter()
        .filter(|a| a["title"] == title)
        .count()
}

fn sql(text: &str) -> sea_orm::Statement {
    sea_orm::Statement::from_string(sea_orm::DatabaseBackend::Postgres, text.to_string())
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
#[allow(clippy::too_many_lines)] // one scenario read top to bottom
async fn a_repeated_request_under_one_key_runs_once_and_gets_the_first_answer() {
    use sea_orm::ConnectionTrait;
    crate::requests::request_open(|request, ctx| async move {
        let org = an_org();
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let title = format!("Idem {tag}");
        let body = json!({ "organization_ref": org, "title": title, "body": "once" });
        let key = |k: &str| axum::http::HeaderValue::from_str(k).unwrap();
        let name = axum::http::HeaderName::from_static("idempotency-key");

        // The first request runs; the repeat is answered from the stored response and creates nothing.
        let first = request
            .post("/api/announcements")
            .add_header(name.clone(), key(KEY))
            .json(&body)
            .await;
        assert_eq!(first.status_code(), 200, "{}", first.text());
        assert!(first.maybe_header("idempotent-replayed").is_none());
        let again = request
            .post("/api/announcements")
            .add_header(name.clone(), key(KEY))
            .json(&body)
            .await;
        assert_eq!(again.status_code(), 200);
        assert_eq!(again.header("idempotent-replayed"), "true");
        assert_eq!(again.text(), first.text(), "the same answer");
        assert_eq!(count(&request, &org, &title).await, 1, "and nothing was created twice");

        // The same key with a different request is refused, and nothing runs.
        let other = json!({ "organization_ref": org, "title": title, "body": "different" });
        let clash = request
            .post("/api/announcements")
            .add_header(name.clone(), key(KEY))
            .json(&other)
            .await;
        assert_eq!(clash.status_code(), 422);
        assert!(clash.text().contains("different request"));
        assert_eq!(count(&request, &org, &title).await, 1);

        // The same body on a different path is a different request too.
        let elsewhere = request
            .put("/api/announcements/00000000-0000-0000-0000-000000000000")
            .add_header(name.clone(), key(KEY))
            .json(&body)
            .await;
        assert_eq!(elsewhere.status_code(), 422);

        // With no key, a repeat runs again.
        for _ in 0..2 {
            request.post("/api/announcements").json(&body).await.assert_status_ok();
        }
        assert_eq!(count(&request, &org, &title).await, 3);

        // A key on a read is ignored.
        let read = request
            .get(&format!("/api/announcements?organization={org}"))
            .add_header(name.clone(), key("short"))
            .await;
        assert_eq!(read.status_code(), 200);

        // A key that is not a plain token is refused before anything runs.
        for bad in ["short", "has space in the key 123", &"x".repeat(65)] {
            let r = request
                .post("/api/announcements")
                .add_header(name.clone(), key(bad))
                .json(&body)
                .await;
            assert_eq!(r.status_code(), 422, "{bad}");
        }
        assert_eq!(count(&request, &org, &title).await, 3);

        // A failure is not kept: fix the request and retry under the same key.
        let retry = "retry-after-a-failure-0001";
        let bad_body = json!({ "organization_ref": org, "title": " ", "body": "x" });
        let failed = request
            .post("/api/announcements")
            .add_header(name.clone(), key(retry))
            .json(&bad_body)
            .await;
        assert_eq!(failed.status_code(), 422);
        let kept: i64 = ctx
            .db
            .query_one_raw(sql("SELECT count(*) AS n FROM idempotency_keys WHERE idem_key = 'retry-after-a-failure-0001'"))
            .await
            .unwrap()
            .unwrap()
            .try_get("", "n")
            .unwrap();
        assert_eq!(kept, 0, "a failure leaves no claim behind");
        // A request that differs but is valid now runs under that key.
        let fixed = request
            .post("/api/announcements")
            .add_header(name.clone(), key(retry))
            .json(&body)
            .await;
        assert_eq!(fixed.status_code(), 200);
        assert_eq!(count(&request, &org, &title).await, 4);

        // A request still running makes a repeat wait; one abandoned long ago is taken over.
        let fingerprint = workforce_planning_management::rules::idempotency::fingerprint(
            "POST",
            "/api/announcements",
            body.to_string().as_bytes(),
        );
        ctx.db
            .execute_raw(sql(&format!("INSERT INTO idempotency_keys (actor, idem_key, fingerprint) VALUES ('anonymous', 'stuck-request-key-00001', '{fingerprint}')")))
            .await
            .unwrap();
        let busy = request
            .post("/api/announcements")
            .add_header(name.clone(), key("stuck-request-key-00001"))
            .json(&body)
            .await;
        assert_eq!(busy.status_code(), 409, "{}", busy.text());
        assert_eq!(busy.header("retry-after"), "1");
        assert_eq!(count(&request, &org, &title).await, 4, "and it did not run");
        ctx.db
            .execute_raw(sql("UPDATE idempotency_keys SET created_at = now() - interval '10 minutes' WHERE idem_key = 'stuck-request-key-00001'"))
            .await
            .unwrap();
        let taken = request
            .post("/api/announcements")
            .add_header(name.clone(), key("stuck-request-key-00001"))
            .json(&body)
            .await;
        assert_eq!(taken.status_code(), 200, "{}", taken.text());
        assert_eq!(count(&request, &org, &title).await, 5, "an abandoned claim is taken over");

        // Expiry: a stored response older than the keep period is gone, so the same key runs again.
        ctx.db
            .execute_raw(sql("UPDATE idempotency_keys SET created_at = now() - interval '2 days' WHERE idem_key = '0123456789abcdef-test-key'"))
            .await
            .unwrap();
        let after = request
            .post("/api/announcements")
            .add_header(name.clone(), key(KEY))
            .json(&body)
            .await;
        assert_eq!(after.status_code(), 200);
        assert!(after.maybe_header("idempotent-replayed").is_none());
        assert_eq!(count(&request, &org, &title).await, 6, "it ran again after the keep period");
    })
    .await;
}
