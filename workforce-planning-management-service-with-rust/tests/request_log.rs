//! What the service writes to its log for a request (WPM-R122, WPM-T197): a request id and a route, and
//! never the query string or an identifier.
//!
//! Its own test binary because it installs the process-wide log subscriber.
//! `#[ignore]`d: boots the app (needs PostgreSQL). Run with `cargo test --test request_log -- --ignored`.

use std::io;
use std::sync::{Arc, Mutex};

use loco_rs::testing::prelude::*;
use serial_test::serial;
use tracing_subscriber::fmt::MakeWriter;
use workforce_planning_management::app::App;

#[derive(Clone, Default)]
struct Captured(Arc<Mutex<Vec<u8>>>);

impl io::Write for Captured {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.lock().expect("log buffer").extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for Captured {
    type Writer = Self;
    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

#[tokio::test(flavor = "multi_thread")]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test --test request_log -- --ignored`"]
async fn a_request_leaves_an_id_and_a_route_and_nothing_personal() {
    // `set_var` is `unsafe` in edition 2024; set before the app boots and reads it.
    unsafe {
        std::env::set_var("WPM_REQUIRE_AUTH", "0");
        std::env::set_var("WPM_RATE_LIMIT_PER_MINUTE", "0");
        std::env::set_var("WPM_RATE_LIMIT_SENSITIVE_PER_MINUTE", "0");
    }
    let captured = Captured::default();
    tracing::subscriber::set_global_default(
        tracing_subscriber::fmt()
            .with_writer(captured.clone())
            .with_ansi(false)
            .with_max_level(tracing::Level::INFO)
            .finish(),
    )
    .expect("no other subscriber in this binary");

    request::<App, _, _>(|request, _ctx| async move {
        let worker = "7b1c2f0e-5d3a-4c1b-9e8f-0a1b2c3d4e5f";
        // a name in the query, an identifier in the path, a caller-chosen id, and a bearer token
        let searched = request
            .get("/api/directory?q=Zebediah%20Quarrelsome&department=Underwater")
            .add_header("x-request-id", "caller-chosen-id-42")
            .add_header("authorization", "Bearer SECRET-TOKEN-VALUE")
            .add_header("user-agent", "SecretAgent/9")
            .await;
        assert_eq!(
            searched
                .headers()
                .get("x-request-id")
                .and_then(|v| v.to_str().ok()),
            Some("caller-chosen-id-42"),
            "the response carries the caller's plain id"
        );
        let by_pid = request.get(&format!("/api/workers/{worker}")).await;
        let made = by_pid
            .headers()
            .get("x-request-id")
            .and_then(|v| v.to_str().ok())
            .expect("the response carries an id when the caller sent none")
            .to_string();
        assert!(uuid::Uuid::parse_str(&made).is_ok(), "a made id is a UUID");
        let hostile = request
            .get("/api/directory")
            .add_header("x-request-id", "bad id with spaces")
            .await;
        assert_ne!(
            hostile
                .headers()
                .get("x-request-id")
                .and_then(|v| v.to_str().ok()),
            Some("bad id with spaces")
        );

        let log = String::from_utf8(captured.0.lock().expect("log buffer").clone()).expect("utf8");
        assert!(
            log.contains("request finished"),
            "no completion line:\n{log}"
        );
        assert!(
            log.contains("caller-chosen-id-42"),
            "the id is in the log:\n{log}"
        );
        assert!(log.contains(&made), "the made id is in the log:\n{log}");
        assert!(log.contains("route=/api/directory"), "route:\n{log}");
        assert!(
            log.contains("route=/api/workers/{}"),
            "route template:\n{log}"
        );
        for secret in [
            "Zebediah",
            "Quarrelsome",
            "Underwater",
            "q=",
            worker,
            "SECRET-TOKEN-VALUE",
            "SecretAgent",
            "bad id with spaces",
            "http.uri",
        ] {
            assert!(!log.contains(secret), "the log holds `{secret}`:\n{log}");
        }
    })
    .await;
}
