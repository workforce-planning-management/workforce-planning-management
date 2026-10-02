//! The `keycloak` auth backend against a **real Keycloak** started by
//! Testcontainers (`spec/testcontainers-keycloak/index.md`).
//!
//! A throwaway Keycloak container imports `tests/keycloak/realm-wpm.json`
//! (realm `wpm`, public client `wpm-api` with the audience / realm-role /
//! group / `organization_ref` mappers `spec/auth.md` documents, plus the
//! users `hr-user` and `plain-user`). The test fetches real access tokens
//! with the password grant, points the service at the container's issuer
//! and JWKS, boots the app, and hits secured routes with those tokens.
//!
//! Its own test binary (not part of `tests/mod.rs`) because the auth
//! `OnceLock`s are process-wide and must be set before the app boots.
//! Built only with `--no-default-features --features keycloak`.
//!
//! `#[ignore]`d: needs PostgreSQL (`config/test.yaml`) **and** a container
//! runtime. This workspace uses Podman, not Docker — point Testcontainers
//! at the Podman socket and skip Ryuk (rootless Podman cannot run it):
//!
//! ```text
//! export DOCKER_HOST="unix://$(podman machine inspect \
//!     --format '{{.ConnectionInfo.PodmanSocket.Path}}')"
//! export TESTCONTAINERS_RYUK_DISABLED=true
//! cargo test --no-default-features --features keycloak \
//!     --test keycloak -- --ignored
//! ```

use serde_json::{Value, json};
use serial_test::serial;
use testcontainers::core::{IntoContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage, ImageExt};
use workforce_planning_management_service::app::App;

use loco_rs::testing::prelude::*;

/// Fully qualified (Podman refuses short names).
const KEYCLOAK_IMAGE: &str = "quay.io/keycloak/keycloak";
const KEYCLOAK_TAG: &str = "26.0";
const REALM: &str = "wpm";
const CLIENT_ID: &str = "wpm-api";

const REALM_FILE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/keycloak/realm-wpm.json");
const REFERENCE_POLICY_FILE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/config/abac-policy.reference.json"
);

/// Start Keycloak with the test realm imported; returns the container
/// (keep it alive for the test) and the realm's issuer URL.
async fn start_keycloak() -> (ContainerAsync<GenericImage>, String) {
    let realm = std::fs::read(REALM_FILE).expect("realm file");
    let container = GenericImage::new(KEYCLOAK_IMAGE, KEYCLOAK_TAG)
        .with_exposed_port(8080.tcp())
        .with_wait_for(WaitFor::message_on_stdout("Listening on:"))
        .with_env_var("KC_BOOTSTRAP_ADMIN_USERNAME", "admin")
        .with_env_var("KC_BOOTSTRAP_ADMIN_PASSWORD", "admin")
        .with_copy_to("/opt/keycloak/data/import/realm-wpm.json", realm)
        .with_cmd(["start-dev", "--import-realm"])
        .start()
        .await
        .expect("start Keycloak (is the container runtime reachable?)");
    let host = container.get_host().await.expect("host");
    let port = container.get_host_port_ipv4(8080).await.expect("port");
    (container, format!("http://{host}:{port}/realms/{REALM}"))
}

/// A real access token for `username` (password grant, public client).
async fn token_for(issuer: &str, username: &str) -> String {
    let response: Value = reqwest::Client::new()
        .post(format!("{issuer}/protocol/openid-connect/token"))
        .form(&[
            ("grant_type", "password"),
            ("client_id", CLIENT_ID),
            ("username", username),
            ("password", "password"),
        ])
        .send()
        .await
        .expect("token request")
        .json()
        .await
        .expect("token json");
    response["access_token"]
        .as_str()
        .unwrap_or_else(|| panic!("no access_token for {username}: {response}"))
        .to_string()
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml) and a container runtime (Podman)"]
async fn keycloak_tokens_gate_and_authorise() {
    let (_keycloak, issuer) = start_keycloak().await;
    let hr = token_for(&issuer, "hr-user").await;
    let plain = token_for(&issuer, "plain-user").await;

    // The auth OnceLocks read these on first use — set before boot.
    // `set_var` is `unsafe` in edition 2024; single-threaded setup step.
    unsafe {
        std::env::set_var("WPM_REQUIRE_AUTH", "1");
        std::env::set_var("WPM_KEYCLOAK_ISSUER", &issuer);
        std::env::set_var("WPM_KEYCLOAK_AUDIENCE", CLIENT_ID);
        std::env::set_var(
            "WPM_KEYCLOAK_JWKS_URL",
            format!("{issuer}/protocol/openid-connect/certs"),
        );
        std::env::set_var("WPM_ABAC_POLICY_FILE", REFERENCE_POLICY_FILE);
    }

    request::<App, _, _>(|request, _ctx| async move {
        let bearer = |token: &str| format!("Bearer {token}");

        // Public allow-list stays open; protected routes need a token.
        assert_eq!(request.get("/_health").await.status_code(), 200);
        assert_eq!(request.get("/api/workers").await.status_code(), 401);
        assert_eq!(
            request
                .get("/api/workers")
                .add_header("authorization", "Bearer not.a.jwt")
                .await
                .status_code(),
            401,
            "a junk token is refused"
        );

        // A plain realm user (no `wpm-hr` role): reads allowed, writes 403.
        assert_eq!(
            request
                .get("/api/workers")
                .add_header("authorization", bearer(&plain))
                .await
                .status_code(),
            200
        );
        assert_eq!(
            request
                .post("/api/workers")
                .add_header("authorization", bearer(&plain))
                .json(&json!({ "person_ref": "person:x" }))
                .await
                .status_code(),
            403,
            "no hr role ⇒ no write"
        );

        // The `wpm-hr` realm role maps to `attrs.hr = ["true"]`: the
        // write passes the policy gate (the body is then judged on its
        // own merits — 422 for the incomplete payload, never 401/403).
        let status = request
            .post("/api/workers")
            .add_header("authorization", bearer(&hr))
            .json(&json!({ "person_ref": "person:x" }))
            .await
            .status_code();
        assert_ne!(status, 401, "a valid Keycloak token authenticates");
        assert_ne!(status, 403, "wpm-hr role authorises the write");
    })
    .await;
}
