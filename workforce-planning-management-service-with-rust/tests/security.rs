//! Security headers and rate limiting over the live routes (WPM-R73,
//! WPM-R74).
//!
//! Its own test binary because the limits are read once into a process-wide
//! `OnceLock`, and the other suites turn them off. Enforcement is left at its
//! default (**on**) here, which also proves the default: an unauthenticated
//! read is a `401`, not a `200`.
//!
//! `#[ignore]`d: boots the app (needs PostgreSQL via `config/test.yaml` /
//! `DATABASE_URL`). Run with `cargo test --test security -- --ignored`.

use loco_rs::testing::prelude::*;
use serial_test::serial;
use workforce_planning_management_service::app::App;

const BASELINE: [&str; 6] = [
    "content-security-policy",
    "x-content-type-options",
    "referrer-policy",
    "permissions-policy",
    "cross-origin-opener-policy",
    "cross-origin-resource-policy",
];

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test --test security -- --ignored`"]
async fn headers_everywhere_and_the_limiter_trips() {
    // Read on first use: a small ordinary limit, a sensitive limit of one.
    // `set_var` is `unsafe` in edition 2024; single-threaded setup.
    unsafe {
        std::env::remove_var("WPM_REQUIRE_AUTH");
        std::env::set_var("WPM_RATE_LIMIT_PER_MINUTE", "8");
        // The test transport has no peer address; name callers by the header,
        // as a deployment behind a proxy that overwrites it would.
        std::env::set_var("WPM_TRUST_FORWARDED", "1");
        std::env::set_var("WPM_RATE_LIMIT_SENSITIVE_PER_MINUTE", "1");
    }
    request::<App, _, _>(|request, _ctx| async move {
        // Secure by default: no variable set, so sign-in is enforced.
        let posture = request.get("/_posture").await;
        assert_eq!(posture.status_code(), 200);
        assert_eq!(posture.json::<serde_json::Value>()["auth_enforced"], true);
        let unauth = request.get("/api/workers").await;
        assert_eq!(unauth.status_code(), 401, "default is enforced");

        // Headers are on a 200 and on refusals, and the API is never cached.
        for (path, expect) in [
            ("/_health", 200),
            ("/api/workers", 401),
            // Deny unless public: even an unknown path is a 401, not a 404.
            ("/no/such/path", 401),
        ] {
            let response = request.get(path).await;
            assert_eq!(response.status_code(), expect, "{path}");
            for name in BASELINE {
                assert!(
                    response.headers().contains_key(name),
                    "{name} missing on {path} ({expect})"
                );
            }
        }
        let api = request.get("/api/workers").await;
        assert_eq!(api.headers()["cache-control"], "no-store");
        assert_eq!(api.headers()["x-content-type-options"], "nosniff");
        assert!(
            api.headers()["content-security-policy"]
                .to_str()
                .unwrap()
                .contains("frame-ancestors 'none'")
        );
        // Over TLS (as a proxy reports it) HSTS is added; otherwise it is not.
        assert!(!api.headers().contains_key("strict-transport-security"));
        let tls = request
            .get("/api/workers")
            .add_header("x-forwarded-proto", "https")
            .await;
        assert!(tls.headers().contains_key("strict-transport-security"));

        // The ordinary limit is 8 per window per caller: health is never
        // counted, and the ninth counted request is refused with Retry-After
        // and still carries the headers.
        let mut refused = None;
        for _ in 0..20 {
            let response = request
                .get("/api/workers")
                .add_header("x-forwarded-for", "198.51.100.1")
                .await;
            if response.status_code() == 429 {
                refused = Some(response);
                break;
            }
        }
        let refused = refused.expect("the limiter never tripped");
        assert!(refused.headers().contains_key("retry-after"));
        assert!(refused.headers().contains_key("x-content-type-options"));
        // Varying the token does not buy a fresh bucket: the caller is named
        // by its address, not by a header it controls.
        let varied = request
            .get("/api/workers")
            .add_header("x-forwarded-for", "198.51.100.1")
            .add_header("authorization", "Bearer a-different-token")
            .await;
        assert_eq!(varied.status_code(), 429);
        // Another caller has its own bucket, and health is exempt.
        let other = request
            .get("/api/workers")
            .add_header("x-forwarded-for", "198.51.100.2")
            .await;
        assert_ne!(other.status_code(), 429);
        assert_eq!(request.get("/_health").await.status_code(), 200);

        // The sensitive class is stricter: the second erase from one caller
        // is refused before it reaches the (unauthenticated) route.
        let first = request
            .post("/api/workers/00000000-0000-0000-0000-000000000000/erase")
            .add_header("x-forwarded-for", "198.51.100.3")
            .await;
        assert_ne!(first.status_code(), 429);
        let second = request
            .post("/api/workers/00000000-0000-0000-0000-000000000000/erase")
            .add_header("x-forwarded-for", "198.51.100.3")
            .await;
        assert_eq!(second.status_code(), 429);
    })
    .await;
}
