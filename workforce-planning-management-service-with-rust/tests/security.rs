//! Security headers and rate limiting over the live routes (WPM-R73,
//! WPM-R74).
//!
//! Its own test binary because the limits are read once into a process-wide
//! `OnceLock`, and the other suites turn them off. Enforcement is left at its
//! default (**on**) here, which also proves the default: an unauthenticated
//! read is a `401`, not a `200`.
//!
//! The caller is the address `axa` resolves (WPM-R239, WPM-D187). `axa_test::prelude::request`
//! serves over a real socket, so its peer is `127.0.0.1`, which `TRUSTED_PROXIES` lists here:
//! the test client plays a trusted proxy, and `X-Forwarded-For` is what that proxy reports.
//! Other peers are a request sent straight to the router with the `ConnectInfo` the server
//! would have put on it, which also shows every layer of the real app in order.
//!
//! `#[ignore]`d: boots the app (needs PostgreSQL via
//! `DATABASE_URL`). Run with `cargo test --test security -- --ignored`.

use axa_test::prelude::*;
use axum::{
    Router,
    body::Body,
    extract::ConnectInfo,
    http::{Request, StatusCode},
};
use serial_test::serial;
use std::net::SocketAddr;
use tower::ServiceExt;
use workforce_planning_management::app::App;

const BASELINE: [&str; 6] = [
    "content-security-policy",
    "x-content-type-options",
    "referrer-policy",
    "permissions-policy",
    "cross-origin-opener-policy",
    "cross-origin-resource-policy",
];

/// The ordinary limit these tests run with.
const LIMIT: usize = 8;

/// Read when the router is built: a small ordinary limit, a sensitive limit of one.
fn settings() {
    // `set_var` is `unsafe` in edition 2024; these tests are serial.
    unsafe {
        std::env::remove_var("WPM_REQUIRE_AUTH");
        std::env::set_var("WPM_RATE_LIMIT_PER_MINUTE", LIMIT.to_string());
        std::env::set_var("WPM_RATE_LIMIT_SENSITIVE_PER_MINUTE", "1");
        // Loopback is the test client, and 10.0.0.0/8 the proxies in the direct requests.
        std::env::set_var("TRUSTED_PROXIES", "127.0.0.1/32, 10.0.0.0/8");
        // Retired: it must neither stop the boot nor believe a header.
        std::env::set_var("WPM_TRUST_FORWARDED", "1");
        // Off, so the unauthenticated requests above count only towards the rate limit.
        std::env::remove_var("LOCKOUT_CLIENT_ADDRESS_AFTER_COUNT");
    }
}

/// `GET /api/workers` straight to the router, from `peer` if there is one.
async fn get(router: &Router, peer: Option<&str>, forwarded: &str) -> StatusCode {
    let mut request = Request::get("/api/workers")
        .header("x-forwarded-for", forwarded)
        .body(Body::empty())
        .unwrap();
    if let Some(peer) = peer {
        let peer: SocketAddr = format!("{peer}:443").parse().unwrap();
        request.extensions_mut().insert(ConnectInfo(peer));
    }
    router.clone().oneshot(request).await.unwrap().status()
}

/// How many of `LIMIT + 2` requests got through before the first `429`, where
/// `send` is told which request this is and may forge a fresh header for it.
async fn allowed_before_refusal<F, Fut>(send: F) -> usize
where
    F: Fn(usize) -> Fut,
    Fut: Future<Output = StatusCode>,
{
    for n in 0..LIMIT + 2 {
        if send(n).await == StatusCode::TOO_MANY_REQUESTS {
            return n;
        }
    }
    panic!("the limiter never tripped");
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test --test security -- --ignored`"]
async fn the_header_is_believed_only_from_a_trusted_peer() {
    settings();
    let boot = boot_test::<App>()
        .await
        .expect("the retired setting must not stop the boot");
    let router = boot.router.expect("a router");
    let router = &router;

    // A peer nobody listed is the caller: a fresh header on every request buys nothing.
    let peer = Some("203.0.113.50");
    let allowed =
        allowed_before_refusal(
            |n| async move { get(router, peer, &format!("198.51.100.{n}")).await },
        )
        .await;
    assert_eq!(allowed, LIMIT, "an untrusted peer named its own bucket");
    // It is refused under any header at all, and another peer has its own bucket.
    assert_eq!(
        get(router, peer, "192.0.2.1").await,
        StatusCode::TOO_MANY_REQUESTS
    );
    assert_ne!(
        get(router, Some("203.0.113.51"), "198.51.100.0").await,
        StatusCode::TOO_MANY_REQUESTS
    );

    // Behind a trusted proxy, a fresh left entry from the caller buys nothing: the
    // proxy's own right entry names it, through whichever listed proxy it comes.
    let allowed = allowed_before_refusal(|n| async move {
        let proxy = format!("10.0.0.{}", 1 + n % 2);
        get(router, Some(&proxy), &format!("192.0.2.{n}, 203.0.113.60")).await
    })
    .await;
    assert_eq!(allowed, LIMIT, "a forged left entry bought a fresh bucket");
    // Two real clients behind it are told apart.
    assert_ne!(
        get(router, Some("10.0.0.1"), "203.0.113.61").await,
        StatusCode::TOO_MANY_REQUESTS,
        "two clients behind one proxy shared its bucket"
    );

    // A proxy that sends no address, or none that can be read, is its own caller.
    let allowed = allowed_before_refusal(|_| get(router, Some("10.0.0.9"), "junk")).await;
    assert_eq!(allowed, LIMIT);

    // With no peer there is no address, and the header is not one: one shared bucket.
    let allowed =
        allowed_before_refusal(
            |n| async move { get(router, None, &format!("198.51.100.{n}")).await },
        )
        .await;
    assert_eq!(allowed, LIMIT, "with no peer, the header named a bucket");
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test --test security -- --ignored`"]
async fn headers_everywhere_and_the_limiter_trips() {
    settings();
    request::<App, _, _>(|request, _ctx| async move {
        // Secure by default: no variable set, so sign-in is enforced.
        let posture = request.get("/_posture").await;
        assert_eq!(posture.status_code(), 200);
        assert_eq!(posture.json::<serde_json::Value>()["auth_enforced"], true);
        let unauth = request.get("/api/workers").await;
        assert_eq!(unauth.status_code(), 401, "default is enforced");
        // WPM-R238: the sign-in layer's refusal is a problem, and keeps its challenge.
        assert_eq!(unauth.header("content-type"), "application/problem+json");
        assert!(unauth.headers().contains_key("www-authenticate"));
        assert_eq!(unauth.json::<serde_json::Value>()["status"], 401);

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
            // The framework's `X-Powered-By: loco.rs` names the stack to a scanner (ZAP 10037).
            assert!(
                !response.headers().contains_key("x-powered-by"),
                "x-powered-by present on {path} ({expect})"
            );
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
        assert_eq!(refused.header("content-type"), "application/problem+json");
        assert_eq!(
            refused.json::<serde_json::Value>()["code"],
            "too_many_requests"
        );
        // WPM-D188: `axa::ratelimit` answers, with the limit and the wait.
        assert_eq!(refused.header("ratelimit-limit"), LIMIT.to_string());
        assert_eq!(refused.header("ratelimit-remaining"), "0");
        assert_eq!(
            refused.header("retry-after"),
            refused.header("ratelimit-reset")
        );
        assert!(refused.headers().contains_key("x-content-type-options"));
        // Nor does a left entry the caller wrote, over a real connection from a
        // trusted proxy: this is what an appending proxy forwards.
        let forged = request
            .get("/api/workers")
            .add_header("x-forwarded-for", "192.0.2.77, 198.51.100.1")
            .await;
        assert_eq!(forged.status_code(), 429);
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
        // What is let through says how much is left.
        assert_eq!(other.header("ratelimit-remaining"), (LIMIT - 1).to_string());
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

/// The failed sign-in lockout (WPM-R242, WPM-D190): after `LOCKOUT_CLIENT_ADDRESS_AFTER_COUNT` `401`s, that client
/// address is refused with `429` on every route but the probes; another address is not; and with it unset, the
/// `401`s go on.
#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test --test security -- --ignored`"]
async fn a_client_address_is_locked_out_after_its_401s_and_only_that_one() {
    settings();
    let router = boot_test::<App>()
        .await
        .expect("boot")
        .router
        .expect("a router");
    for _ in 0..5 {
        assert_eq!(
            get(&router, Some("203.0.113.80"), "junk").await,
            StatusCode::UNAUTHORIZED,
            "unset is no lockout"
        );
    }

    // `set_var` is `unsafe` in edition 2024; these tests are serial.
    unsafe { std::env::set_var("LOCKOUT_CLIENT_ADDRESS_AFTER_COUNT", "3") };
    let router = boot_test::<App>()
        .await
        .expect("boot")
        .router
        .expect("a router");
    let peer = Some("203.0.113.81");
    for _ in 0..3 {
        assert_eq!(get(&router, peer, "junk").await, StatusCode::UNAUTHORIZED);
    }
    let mut request = Request::get("/api/workers").body(Body::empty()).unwrap();
    request.extensions_mut().insert(ConnectInfo::<SocketAddr>(
        "203.0.113.81:443".parse().unwrap(),
    ));
    let refused = router.clone().oneshot(request).await.unwrap();
    assert_eq!(refused.status(), StatusCode::TOO_MANY_REQUESTS);
    let wait: u32 = refused.headers()["retry-after"]
        .to_str()
        .unwrap()
        .parse()
        .unwrap();
    assert!((1..=900).contains(&wait), "{wait}");
    assert_eq!(
        refused.headers()["content-type"],
        "application/problem+json"
    );
    // Behind a trusted proxy the client it names is locked out, not the proxy.
    assert_eq!(
        get(&router, Some("10.0.0.1"), "203.0.113.81").await,
        StatusCode::TOO_MANY_REQUESTS
    );
    assert_eq!(
        get(&router, Some("10.0.0.1"), "203.0.113.82").await,
        StatusCode::UNAUTHORIZED,
        "another address is not locked out"
    );
    let mut health = Request::get("/_health").body(Body::empty()).unwrap();
    health.extensions_mut().insert(ConnectInfo::<SocketAddr>(
        "203.0.113.81:443".parse().unwrap(),
    ));
    assert_eq!(
        router.clone().oneshot(health).await.unwrap().status(),
        StatusCode::OK
    );

    unsafe { std::env::set_var("LOCKOUT_CLIENT_ADDRESS_AFTER_COUNT", "several") };
    assert!(
        boot_test::<App>().await.is_err(),
        "an unreadable count stops the start"
    );
    unsafe { std::env::remove_var("LOCKOUT_CLIENT_ADDRESS_AFTER_COUNT") };
}
