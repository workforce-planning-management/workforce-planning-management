//! Pay information requests under enforcement (WPM-R145, WPM-D99).
//!
//! Its own test binary (the auth flag and policy are cached in process-wide `OnceLock`s). Proves that a worker asks for
//! and reads only their own requests with a computed due date and one waiting at a time; that the answer is worked out
//! only for a caller who may read salaries and recorded only by one who may write; and that the answer names no one else.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use ed25519_dalek::SigningKey;
use loco_rs::testing::prelude::*;
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

const CATEGORIES: &str = r#"{"sex":["female","male"]}"#;

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test --test enforcement_pay_information -- --ignored`"]
#[allow(clippy::too_many_lines)] // one scenario read top to bottom
async fn a_worker_asks_for_their_own_pay_information_and_only_those_who_may_answer_it_can() {
    use sea_orm::ConnectionTrait;
    let (keys, kid) = keys_and_kid();
    unsafe {
        std::env::set_var("WPM_REQUIRE_AUTH", "1");
        std::env::set_var("WPM_PASETO_KEYS", keys.to_string());
        std::env::set_var("WPM_ABAC_POLICY_FILE", REFERENCE_POLICY_FILE);
        std::env::set_var(
            "WPM_EQUALITY_MONITORING_BASIS",
            "Test deployment: impact assessment PI-2",
        );
        std::env::set_var("WPM_EQUALITY_CATEGORIES", CATEGORIES);
        std::env::set_var("WPM_EQUALITY_FLOOR", "5");
    }
    let bearer = |t: String| format!("Bearer {t}");
    let svc = bearer(sign_as(&kid, "svc-user", &[("svc", &["true"])]));
    let hr_p = uuid::Uuid::new_v4();
    let hr = bearer(sign_as(&kid, &hr_p.to_string(), &[("hr", &["true"])]));
    let payroll_p = uuid::Uuid::new_v4();
    let payroll = bearer(sign_as(
        &kid,
        &payroll_p.to_string(),
        &[("payroll", &["true"])],
    ));
    // Twelve engineers: six women and six men. The first woman and the first man are the people who ask.
    let people: Vec<uuid::Uuid> = (0..12).map(|_| uuid::Uuid::new_v4()).collect();
    let tokens: Vec<String> = people
        .iter()
        .map(|p| bearer(sign_as(&kid, &p.to_string(), &[])))
        .collect();

    request::<App, _, _>(|request, ctx| async move {
        let org = format!("organization:{}", uuid::Uuid::new_v4());
        let mut pids = Vec::new();
        for (i, person) in people.iter().enumerate() {
            let (sex, salary) = if i < 6 { ("female", 9_000_000) } else { ("male", 10_000_000) };
            let r = request.post("/api/workers").add_header("authorization", svc.clone()).json(&json!({
                "person_ref": format!("person:{person}"), "organization_ref": org,
                "worker_number": format!("PIE-{i}"), "display_name": format!("Pay Info Enforcement {i}"),
                "employment_type": "permanent", "department": "engineering", "job_title": "Engineer",
                "salary_minor": salary, "salary_currency": "GBP", "hired_on": "2020-01-05",
            })).await;
            assert_eq!(r.status_code(), 200, "{}", r.text());
            let pid = r.json::<Value>()["pid"].as_str().unwrap().to_string();
            ctx.db.execute_unprepared(&format!(
                "INSERT INTO equality_declarations (worker_pid, category, value) VALUES ('{pid}', 'sex', '{sex}')")).await.unwrap();
            pids.push(pid);
        }
        for (p, n, role) in [(hr_p, "HR", "hr_admin"), (payroll_p, "PAY", "payroll_admin")] {
            let w = request.post("/api/workers").add_header("authorization", svc.clone()).json(&json!({
                "person_ref": format!("person:{p}"), "organization_ref": org, "worker_number": format!("PIE-{n}"),
                "display_name": format!("Pay Info Enforcement {n}"), "employment_type": "permanent", "department": "people",
                "job_title": "Administrator", "hired_on": "2020-01-05" })).await.json::<Value>()["pid"].as_str().unwrap().to_string();
            ctx.db.execute_unprepared(&format!(
                "INSERT INTO organization_memberships (pid, person_ref, organization_ref, worker_pid, role, starts_on) \
                 VALUES ('{}', 'person:{p}', '{org}', '{w}', '{role}', '2020-01-05')", uuid::Uuid::new_v4())).await.unwrap();
        }
        request.post("/api/pay-categories").add_header("authorization", hr.clone()).json(&json!({
            "organization_ref": org, "name": "Engineering",
            "basis": "Same skills, effort, responsibility and working conditions.", "job_titles": ["Engineer"] }))
            .await.assert_status_ok();
        let get = |path: String, t: String| {
            let request = &request;
            async move { request.get(&path).add_header("authorization", t).await }
        };
        let mine = "/api/me/pay-information-requests".to_string();

        // ── A worker asks; the due date is computed; one waits at a time ───────────────
        assert_eq!(request.post(&mine).await.status_code(), 401, "no token");
        let asked = request.post(&mine).add_header("authorization", tokens[0].clone()).await;
        assert_eq!(asked.status_code(), 200, "{}", asked.text());
        let body: Value = asked.json();
        let today = chrono::Utc::now().date_naive();
        let due = today.checked_add_months(chrono::Months::new(2)).unwrap();
        assert_eq!(body["due_on"], due.to_string(), "two calendar months, computed");
        let request_pid = body["pid"].as_str().unwrap().to_string();
        let again = request.post(&mine).add_header("authorization", tokens[0].clone()).await;
        assert_eq!(again.status_code(), 422, "one at a time");
        assert!(again.text().contains("already"), "{}", again.text());
        let own: Value = get(mine.clone(), tokens[0].clone()).await.json();
        assert_eq!(own.as_array().unwrap().len(), 1);
        assert_eq!(own[0]["standing"], "open");
        assert_eq!(own[0]["due_on"], due.to_string());
        let other: Value = get(mine.clone(), tokens[1].clone()).await.json();
        assert!(other.as_array().unwrap().is_empty(), "a worker sees only their own requests");

        // ── Who reads and answers ──────────────────────────────────────────────────────
        let queue = format!("/api/pay-information-requests?organization={org}");
        let answer = format!("/api/pay-information-requests/{request_pid}/answer");
        let done = format!("/api/pay-information-requests/{request_pid}/answered");
        assert_eq!(get(queue.clone(), tokens[0].clone()).await.status_code(), 403, "the queue is not the worker's");
        assert_eq!(get(answer.clone(), tokens[0].clone()).await.status_code(), 403, "nor the answer");
        assert_eq!(request.post(&done).add_header("authorization", tokens[0].clone()).await.status_code(), 403, "nor recording it");
        let listed: Value = get(queue.clone(), hr.clone()).await.json();
        assert_eq!(listed.as_array().unwrap().len(), 1, "HR sees who asked");
        assert!(listed[0]["display_name"].as_str().unwrap().starts_with("Pay Info Enforcement"));
        let masked = get(answer.clone(), hr.clone()).await;
        assert_eq!(masked.status_code(), 403, "HR reads salaries masked: {}", masked.text());
        let given = get(answer.clone(), payroll.clone()).await;
        assert_eq!(given.status_code(), 200, "{}", given.text());
        let text = given.text();
        let parsed: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(parsed["answer"]["own_pay_minor"], 9_000_000);
        assert_eq!(parsed["answer"]["female_average_minor"], json!({ "status": "shown", "value": 9_000_000 }));
        assert_eq!(parsed["answer"]["male_average_minor"], json!({ "status": "shown", "value": 10_000_000 }));
        for pid in &pids {
            assert!(!text.contains(pid.as_str()), "the answer names no one");
        }
        assert!(!text.contains("Pay Info Enforcement") && !text.contains("PIE-"));
        assert_eq!(request.post(&done).add_header("authorization", payroll.clone()).await.status_code(), 403, "payroll reads; it does not write");
        let recorded = request.post(&done).add_header("authorization", hr.clone()).await;
        assert_eq!(recorded.status_code(), 200, "{}", recorded.text());

        // ── The worker sees it answered and may ask again ─────────────────────────────
        let after: Value = get(mine.clone(), tokens[0].clone()).await.json();
        assert_eq!(after[0]["standing"], "answered");
        assert!(after[0]["answered_on"].is_string());
        assert_eq!(request.post(&mine).add_header("authorization", tokens[0].clone()).await.status_code(), 200, "a new request after an answer");

        // ── It is in the worker's own export, and the audit holds no figure ─────────────
        let export: Value = get(format!("/api/workers/{}/subject-access", pids[0]), tokens[0].clone()).await.json();
        assert_eq!(export["pay_information_requests"].as_array().unwrap().len(), 2);
        let rows = ctx.db.query_all_raw(sea_orm::Statement::from_string(sea_orm::DatabaseBackend::Postgres,
            "SELECT action, snapshot::text AS snapshot FROM audit_logs WHERE action LIKE 'pay_information%' ORDER BY id".to_string()))
            .await.unwrap();
        let actions: Vec<String> = rows.iter().map(|r| r.try_get("", "action").unwrap()).collect();
        for expected in ["pay_information_requested", "pay_information_answer_read", "pay_information_answered"] {
            assert!(actions.iter().any(|a| a == expected), "{expected} in {actions:?}");
        }
        for row in &rows {
            let snapshot: Option<String> = row.try_get("", "snapshot").unwrap();
            assert!(!snapshot.unwrap_or_default().contains("000000"), "a figure in an audit snapshot");
        }
    })
    .await;
}
