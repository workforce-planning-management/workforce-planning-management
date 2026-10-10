//! Who may read a contractor's rate under enforcement (WPM-R80, WPM-D58).
//!
//! Its own test binary (the auth flag and policy are cached in process-wide `OnceLock`s). It runs
//! against the shipped reference policy, so what a deployment is told to mount is what is
//! verified: the rate is masked like salary. Payroll and the worker themself see it; HR, who may
//! record it, and every other signed-in caller see that a rate exists and not what it is.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ed25519_dalek::SigningKey;
use loco_rs::testing::prelude::*;
use rusty_paseto::core::{Footer, Key, Paseto, PasetoAsymmetricPrivateKey, Payload, Public, V4};
use serde_json::{Value, json};
use serial_test::serial;
use sha2::{Digest, Sha256};
use workforce_planning_management_service::app::App;

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
    let iat: i64 = 1_700_000_000;
    let payload = json!({
        "sub": sub,
        "email": "person@example.com", "name": "Test Person",
        "iss": ISSUER, "aud": AUDIENCE,
        "exp": iat + 10_000_000_000_i64, "iat": iat,
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

/// One boot, every persona.
#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test --test enforcement_engagements -- --ignored`"]
async fn a_contractors_rate_is_masked_like_salary() {
    let (keys, kid) = keys_and_kid();
    unsafe {
        std::env::set_var("WPM_REQUIRE_AUTH", "1");
        std::env::set_var("WPM_PASETO_KEYS", keys.to_string());
        std::env::set_var("WPM_ABAC_POLICY_FILE", REFERENCE_POLICY_FILE);
    }
    let contractor_person = uuid::Uuid::new_v4();
    let bearer = |token: &str| format!("Bearer {token}");
    let own = bearer(&sign_as(&kid, &contractor_person.to_string(), &[]));
    let other = bearer(&sign_as(&kid, &uuid::Uuid::new_v4().to_string(), &[]));
    let hr = bearer(&sign_as(&kid, "hr-user", &[("hr", &["true"])]));
    let payroll = bearer(&sign_as(&kid, "payroll-user", &[("payroll", &["true"])]));
    let machine = bearer(&sign_as(&kid, "svc-user", &[("svc", &["true"])]));

    request::<App, _, _>(|request, _ctx| async move {
        let created = request
            .post("/api/workers")
            .add_header("authorization", machine.clone())
            .json(&json!({
                "person_ref": format!("person:{contractor_person}"),
                "organization_ref": format!("organization:{}", uuid::Uuid::new_v4()),
                "worker_number": "EE-1", "display_name": "Test Worker EE-1",
                "employment_type": "contractor", "department": "engineering",
                "job_title": "Engineer", "hired_on": "2026-01-05",
                "engagement_ends_on": "2026-12-31",
            }))
            .await;
        assert_eq!(created.status_code(), 200);
        let worker = created.json::<Value>()["pid"].as_str().unwrap().to_string();
        let url = format!("/api/workers/{worker}/contractor-details");
        let details = json!({ "route": "agency", "rate_minor": 45_000,
                              "rate_currency": "GBP", "rate_basis": "day" });

        // Writing: HR may; the worker and a stranger may not (the policy allows writes to HR).
        for (name, token) in [
            ("worker", &own),
            ("stranger", &other),
            ("payroll", &payroll),
        ] {
            assert_eq!(
                request
                    .put(&url)
                    .add_header("authorization", token.clone())
                    .json(&details)
                    .await
                    .status_code(),
                403,
                "{name} cannot record a rate"
            );
        }
        assert_eq!(
            request
                .put(&url)
                .add_header("authorization", hr.clone())
                .json(&details)
                .await
                .status_code(),
            200
        );

        // Reading: payroll and the worker see the rate; HR and a stranger see that one exists.
        let read = |token: String| {
            let request = &request;
            let url = url.clone();
            async move {
                let response = request.get(&url).add_header("authorization", token).await;
                assert_eq!(response.status_code(), 200);
                response.json::<Value>()["contractor_details"].clone()
            }
        };
        for (name, token) in [("payroll", payroll.clone()), ("worker", own.clone())] {
            let seen = read(token).await;
            assert_eq!(seen["rate_minor"], 45_000, "{name} sees the rate");
            assert_eq!(seen["rate_masked"], false, "{name}");
        }
        // A stranger reads nothing about the contractor at all (need-to-know, WPM-R114).
        let refused = request
            .get(&url)
            .add_header("authorization", other.clone())
            .await;
        assert_eq!(
            refused.status_code(),
            403,
            "a stranger does not even see that details exist"
        );
        for (name, token) in [("hr", hr.clone())] {
            let seen = read(token).await;
            assert_eq!(seen["rate_minor"], Value::Null, "{name} does not");
            assert_eq!(seen["rate_currency"], Value::Null, "{name}");
            assert_eq!(seen["rate_recorded"], true, "{name} can tell a rate exists");
            assert_eq!(seen["rate_masked"], true, "{name}");
        }

        // The worker row, which many endpoints return, never carries a rate.
        let row = request
            .get(&format!("/api/workers/{worker}"))
            .add_header("authorization", payroll.clone())
            .await
            .json::<Value>();
        assert!(row.get("rate_minor").is_none(), "no rate on the worker row");

        // Only an unmasked read of the rate is audited.
        let audits = request
            .get(&format!("/api/audits/{worker}"))
            .add_header("authorization", machine)
            .await
            .json::<Value>();
        let text = serde_json::to_string(&audits).unwrap();
        assert!(text.contains("rate_read"));
        assert!(!text.contains("45000"));
    })
    .await;
}
