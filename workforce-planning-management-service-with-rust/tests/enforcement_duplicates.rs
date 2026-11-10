//! The duplicate check under enforcement (WPM-R136, WPM-D87): an HR caller with a membership is
//! checked only against the organizations they can read, and is refused one they cannot.
//!
//! Its own test binary (the auth flag and policy are cached in process-wide `OnceLock`s). It runs
//! against the shipped reference policy.

use axa_test::prelude::*;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ed25519_dalek::SigningKey;
use rusty_paseto::core::{Footer, Key, Paseto, PasetoAsymmetricPrivateKey, Payload, Public, V4};
use serde_json::{Value, json};
use serial_test::serial;
use sha2::{Digest, Sha256};
use workforce_planning_management::app::App;

const ISSUER: &str = "authentication-service";
const AUDIENCE: &str = "main-x-service";
/// Throwaway Ed25519 seed — mints test tokens only, never a secret.
const SEED: [u8; 32] = [9; 32];

/// The published-key-set JSON + `kid` for the test key.
fn keys_and_kid() -> (Value, String) {
    let public = SigningKey::from_bytes(&SEED).verifying_key().to_bytes();
    let kid = URL_SAFE_NO_PAD.encode(Sha256::digest(public));
    let keys = json!({
        "keys": [{ "kty": "OKP", "crv": "Ed25519", "use": "sig",
                   "kid": kid, "x": URL_SAFE_NO_PAD.encode(public) }]
    });
    (keys, kid)
}

/// Mint a PASETO `v4.public` with a given `sub` and ABAC attributes.
fn sign_as(kid: &str, sub: &str, attrs: &[(&str, &[&str])]) -> String {
    let attrs_map: serde_json::Map<String, Value> = attrs
        .iter()
        .map(|(key, values)| ((*key).to_string(), json!(values)))
        .collect();
    let iat: i64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .and_then(|d| i64::try_from(d.as_secs()).ok())
        .expect("the clock is after 1970");
    let payload = json!({
        "sub": sub,
        "email": "person@example.com", "name": "Test Person",
        "iss": ISSUER, "aud": AUDIENCE,
        "exp": iat + 3_600, "iat": iat,
        "sid": "test-sid", "attrs": attrs_map,
    })
    .to_string();
    let keypair = SigningKey::from_bytes(&SEED).to_keypair_bytes();
    let key = Key::<64>::from(keypair);
    let private = PasetoAsymmetricPrivateKey::<V4, Public>::from(&key);
    let footer = format!(r#"{{"kid":"{kid}"}}"#);
    let mut builder = Paseto::<V4, Public>::builder();
    builder.set_payload(Payload::from(payload.as_str()));
    builder.set_footer(Footer::from(footer.as_str()));
    builder.try_sign(&private).expect("sign")
}

/// The matrix runs against **the shipped reference policy file**
/// (`config/abac-policy.reference.json`, WPM-G1) — so what the
/// runbook tells a deployment to mount is exactly what is verified:
/// svc/admin everything; `payroll=true` unmasked read; `hr=true`
/// write + masked read; `$sub` self-read unmasked; masked-read
/// fallback for every other authenticated caller.
const REFERENCE_POLICY_FILE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/config/abac-policy.reference.json"
);

/// One boot: a member of one organization, a namesake in another.
#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test --test enforcement_duplicates -- --ignored`"]
async fn a_member_is_checked_only_against_their_organizations() {
    let (keys, kid) = keys_and_kid();
    unsafe {
        std::env::set_var("WPM_REQUIRE_AUTH", "1");
        std::env::set_var("WPM_PASETO_KEYS", keys.to_string());
        std::env::set_var("WPM_ABAC_POLICY_FILE", REFERENCE_POLICY_FILE);
    }
    let member = uuid::Uuid::new_v4();
    let bearer = |token: &str| format!("Bearer {token}");
    let as_member = bearer(&sign_as(&kid, &member.to_string(), &[("hr", &["true"])]));
    let machine = bearer(&sign_as(&kid, "svc-user", &[("svc", &["true"])]));
    let mine = format!("organization:{}", uuid::Uuid::new_v4());
    let theirs = format!("organization:{}", uuid::Uuid::new_v4());

    request::<App, _, _>(|request, _ctx| async move {
        let mut pids = Vec::new();
        for (org, number) in [(&mine, "DUP-1"), (&theirs, "DUP-2")] {
            let created = request
                .post("/api/workers")
                .add_header("authorization", machine.clone())
                .json(&json!({
                    "person_ref": format!("person:{}", uuid::Uuid::new_v4()),
                    "organization_ref": org, "worker_number": number,
                    "display_name": "Ada Lovelace", "employment_type": "permanent",
                    "department": "engineering", "job_title": "Engineer",
                    "hired_on": "2026-01-05",
                }))
                .await;
            assert_eq!(created.status_code(), 200, "{}", created.text());
            pids.push(created.json::<Value>()["pid"].as_str().unwrap().to_string());
        }
        let granted = request
            .post("/api/organization-memberships")
            .add_header("authorization", machine.clone())
            .json(&json!({
                "person_ref": format!("person:{member}"), "organization_ref": mine,
                "role": "member", "starts_on": "2026-01-01",
            }))
            .await;
        assert_eq!(granted.status_code(), 200, "{}", granted.text());

        let check = |org: String| {
            let request = &request;
            let token = as_member.clone();
            async move {
                request
                    .post("/api/workers/possible-duplicates")
                    .add_header("authorization", token)
                    .json(&json!({ "display_name": "Ada Lovelace", "organization_ref": org }))
                    .await
            }
        };
        let found = check(mine.clone()).await;
        assert_eq!(found.status_code(), 200, "{}", found.text());
        let found: Value = found.json();
        let seen: Vec<&str> = found
            .as_array()
            .expect("a list")
            .iter()
            .filter_map(|item| item["worker_pid"].as_str())
            .collect();
        assert_eq!(
            seen,
            [pids[0].as_str()],
            "only the member's organization is searched: {found}"
        );

        assert_eq!(check(theirs.clone()).await.status_code(), 403);
    })
    .await;
}
