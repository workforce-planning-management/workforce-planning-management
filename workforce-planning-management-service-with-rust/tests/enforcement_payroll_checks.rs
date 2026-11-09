//! The payroll run's advisory checks under enforcement (WPM-R218, WPM-R219, WPM-T415).
//!
//! Its own test binary (the auth flag and policy are cached in process-wide `OnceLock`s). The
//! minimum-wage and overtime checks read pay, so a caller who is not HR or payroll is refused, and a
//! payroll caller who holds no seat in the run's organization is told the run does not exist. (The run's own routes are not
//! scoped by membership: an HR or payroll attribute in a token is deployment-wide, as the personas test in `enforcement.rs` has it.) the answer for one who may read names no age.

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

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test --test enforcement_payroll_checks -- --ignored`"]
async fn the_checks_read_pay_so_only_payroll_in_the_organization_may_run_them() {
    use sea_orm::ConnectionTrait;
    let (keys, kid) = keys_and_kid();
    unsafe {
        std::env::set_var("WPM_REQUIRE_AUTH", "1");
        std::env::set_var("WPM_PASETO_KEYS", keys.to_string());
        std::env::set_var("WPM_ABAC_POLICY_FILE", REFERENCE_POLICY_FILE);
    }
    let bearer = |t: String| format!("Bearer {t}");
    let svc = bearer(sign_as(&kid, "svc-user", &[("svc", &["true"])]));
    let plain_p = uuid::Uuid::new_v4();
    let plain = bearer(sign_as(&kid, &plain_p.to_string(), &[]));
    let payroll_p = uuid::Uuid::new_v4();
    let payroll = bearer(sign_as(
        &kid,
        &payroll_p.to_string(),
        &[("payroll", &["true"]), ("hr", &["true"])],
    ));
    let outsider_p = uuid::Uuid::new_v4();
    let outsider = bearer(sign_as(
        &kid,
        &outsider_p.to_string(),
        &[("payroll", &["true"]), ("hr", &["true"])],
    ));

    request::<App, _, _>(|request, ctx| async move {
        let org = format!("organization:{}", uuid::Uuid::new_v4());
        let other = format!("organization:{}", uuid::Uuid::new_v4());
        let mut workers = Vec::new();
        for (n, person, home) in [("PC-1", uuid::Uuid::new_v4(), &org), ("PC-PAY", payroll_p, &org), ("PC-OUT", outsider_p, &other)] {
            let r = request.post("/api/workers").add_header("authorization", svc.clone()).json(&json!({
                "person_ref": format!("person:{person}"), "organization_ref": home, "worker_number": n,
                "display_name": format!("Test Worker {n}"), "employment_type": "permanent", "department": "finance",
                "job_title": "Analyst", "hired_on": "2023-01-05", "salary_minor": 3_600_000, "salary_currency": "GBP",
            })).await;
            assert_eq!(r.status_code(), 200, "{}", r.text());
            let pid = r.json::<Value>()["pid"].as_str().unwrap().to_string();
            request.post(&format!("/api/workers/{pid}/status")).add_header("authorization", svc.clone())
                .json(&json!({ "to": "active" })).await.assert_status_ok();
            workers.push(pid);
        }
        // Payroll in each organization holds a seat there; the plain caller holds none.
        for (person, home, w) in [(payroll_p, &org, &workers[1]), (outsider_p, &other, &workers[2])] {
            ctx.db.execute_unprepared(&format!(
                "INSERT INTO organization_memberships (pid, person_ref, organization_ref, worker_pid, role, starts_on) \
                 VALUES ('{}', 'person:{person}', '{home}', '{w}', 'payroll_admin', '2023-01-05')", uuid::Uuid::new_v4())).await.unwrap();
        }
        let run: Value = request.post("/api/payroll-runs").add_header("authorization", payroll.clone())
            .json(&json!({ "organization_ref": org, "period_start": "2026-07-01", "period_end": "2026-07-31" })).await.json();
        let run_pid = run["pid"].as_str().unwrap().to_string();
        request.post(&format!("/api/payroll-runs/{run_pid}/calculate")).add_header("authorization", payroll.clone()).await.assert_status_ok();

        for check in ["minimum-wage-check", "overtime-check"] {
            let url = format!("/api/payroll-runs/{run_pid}/{check}");
            let get = |t: String| { let (request, url) = (&request, url.clone()); async move { request.get(&url).add_header("authorization", t).await } };
            assert_eq!(request.get(&url).await.status_code(), 401, "{check}: no sign-in");
            let plain_code = get(plain.clone()).await.status_code();
            let outsider_code = get(outsider.clone()).await.status_code();
            let ok = get(payroll.clone()).await;
            assert_eq!(plain_code, 403, "{check}: someone who is not payroll");
            assert_eq!(outsider_code, 404, "{check}: payroll of another organization is told the run does not exist, not that it does");
            assert_eq!(ok.status_code(), 200, "{check}: payroll in this organization");
        }
    })
    .await;
}
