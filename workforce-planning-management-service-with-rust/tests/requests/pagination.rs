//! `?limit=&offset=` on `GET /api/workers` and `GET /api/benefit-plans`
//! (WPM-T40; `agents/share/restful.md`): `limit` clamps to `MAX_LIMIT`
//! rather than erroring, `offset` past `MAX_OFFSET` is a `400`, and every
//! response carries `X-Total-Count`/`X-Limit`/`X-Offset`.
//!
//! `#[ignore]`d: needs PostgreSQL; run with `cargo test -- --ignored`.

use serde_json::Value;
use serial_test::serial;

use super::{an_org, seed_worker};

macro_rules! header {
    ($r:expr, $name:expr) => {
        $r.headers()
            .get($name)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .to_string()
    };
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn worker_list_paginates_and_clamps() {
    crate::requests::request_open(|request, _ctx| async move {
        let org = an_org();
        for i in 0..5 {
            seed_worker!(&request, org.clone(), format!("EMP-PAGE-{i}"), None).await;
        }

        // Default page: no `limit`/`offset` ⇒ the old behaviour (a bare
        // array), now additionally carrying the total/limit/offset headers.
        let all = request.get("/api/workers").await;
        assert_eq!(all.status_code(), 200);
        let total: u64 = header!(all, "x-total-count").parse().expect("total");
        assert!(total >= 5, "expected at least 5 workers, got {total}");

        // `limit=2&offset=1` returns exactly 2 rows and reports the true
        // total (not the page size).
        let page = request.get("/api/workers?limit=2&offset=1").await;
        assert_eq!(page.status_code(), 200);
        let body: Value = page.json();
        assert_eq!(body.as_array().expect("array").len(), 2);
        assert_eq!(header!(page, "x-total-count"), total.to_string());
        assert_eq!(header!(page, "x-limit"), "2");
        assert_eq!(header!(page, "x-offset"), "1");

        // A `limit` past `MAX_LIMIT` (500) is clamped, not rejected.
        let huge = request.get("/api/workers?limit=100000").await;
        assert_eq!(huge.status_code(), 200);
        assert_eq!(
            header!(huge, "x-limit"),
            "500",
            "limit should clamp to MAX_LIMIT"
        );

        // An `offset` past `MAX_OFFSET` (10 000) is a 400 (SEC-G7).
        let deep = request.get("/api/workers?offset=10001").await;
        assert_eq!(
            deep.status_code(),
            400,
            "offset past MAX_OFFSET should be 400"
        );
    })
    .await;
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
async fn benefit_plan_list_paginates() {
    crate::requests::request_open(|request, _ctx| async move {
        for i in 0..3 {
            let created = request
                .post("/api/benefit-plans")
                .json(&serde_json::json!({
                    "name": format!("Test Plan {i}"),
                    "kind": "pension",
                    "provider": "Test Provider",
                    "worker_cost_minor": 0,
                    "employer_cost_minor": 0,
                    "currency": "GBP",
                }))
                .await;
            assert_eq!(created.status_code(), 200, "plan {i} should create");
        }

        let page = request.get("/api/benefit-plans?limit=1&offset=0").await;
        assert_eq!(page.status_code(), 200);
        let body: Value = page.json();
        assert_eq!(body.as_array().expect("array").len(), 1);
        let total: u64 = header!(page, "x-total-count").parse().expect("total");
        assert!(total >= 3, "expected at least 3 plans, got {total}");
        assert_eq!(header!(page, "x-limit"), "1");
        assert_eq!(header!(page, "x-offset"), "0");

        let deep = request.get("/api/benefit-plans?offset=10001").await;
        assert_eq!(
            deep.status_code(),
            400,
            "offset past MAX_OFFSET should be 400"
        );
    })
    .await;
}
