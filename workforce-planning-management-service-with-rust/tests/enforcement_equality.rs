//! Equality and diversity monitoring under enforcement (WPM-R125, WPM-D74).
//!
//! Its own test binary (the auth flag and policy are cached in process-wide `OnceLock`s). The
//! feature is **off** until a deployer records a lawful basis; this proves it does not exist then,
//! that once on only the worker can read their own answers, that the only output is an aggregate
//! with small groups withheld, and that no audit entry carries a value.

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

const CATEGORIES: &str =
    r#"{"sex":["female","male"],"religion_or_belief":["none","other"],"disability":["yes","no"]}"#;

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test --test enforcement_equality -- --ignored`"]
#[allow(clippy::too_many_lines)] // one scenario read top to bottom
async fn monitoring_is_off_until_a_basis_is_recorded_and_then_only_the_worker_reads_their_answers()
{
    use sea_orm::ConnectionTrait;
    let (keys, kid) = keys_and_kid();
    unsafe {
        std::env::set_var("WPM_REQUIRE_AUTH", "1");
        std::env::set_var("WPM_PASETO_KEYS", keys.to_string());
        std::env::set_var("WPM_ABAC_POLICY_FILE", REFERENCE_POLICY_FILE);
        std::env::remove_var("WPM_EQUALITY_MONITORING_BASIS");
        std::env::remove_var("WPM_EQUALITY_CATEGORIES");
        std::env::remove_var("WPM_EQUALITY_FLOOR");
    }
    let people: Vec<uuid::Uuid> = (0..16).map(|_| uuid::Uuid::new_v4()).collect();
    let bearer = |t: String| format!("Bearer {t}");
    let tokens: std::sync::Arc<Vec<String>> = std::sync::Arc::new(
        people
            .iter()
            .map(|p| bearer(sign_as(&kid, &p.to_string(), &[])))
            .collect(),
    );
    let token = {
        let tokens = tokens.clone();
        move |i: usize| tokens[i].clone()
    };
    let svc = bearer(sign_as(&kid, "svc-user", &[("svc", &["true"])]));
    let hr_p = uuid::Uuid::new_v4();
    let hr = bearer(sign_as(&kid, &hr_p.to_string(), &[("hr", &["true"])]));

    request::<App, _, _>(|request, ctx| async move {
        let org = format!("organization:{}", uuid::Uuid::new_v4());
        // Fifteen workers: twelve in engineering, three in operations; and HR.
        let mut pids = Vec::new();
        for (i, person) in people.iter().enumerate().take(15) {
            let dept = if i < 12 { "engineering" } else { "operations" };
            let r = request.post("/api/workers").add_header("authorization", svc.clone()).json(&json!({
                "person_ref": format!("person:{person}"), "organization_ref": org,
                "worker_number": format!("EQ-{i}"), "display_name": format!("Test Worker EQ-{i}"),
                "employment_type": "permanent", "department": dept, "job_title": "Engineer", "hired_on": "2023-01-05",
            })).await;
            assert_eq!(r.status_code(), 200);
            let pid = r.json::<Value>()["pid"].as_str().unwrap().to_string();
            request.post(&format!("/api/workers/{pid}/status")).add_header("authorization", svc.clone())
                .json(&json!({ "to": "active" })).await.assert_status_ok();
            pids.push(pid);
        }
        let hr_w = request.post("/api/workers").add_header("authorization", svc.clone()).json(&json!({
            "person_ref": format!("person:{hr_p}"), "organization_ref": org, "worker_number": "EQ-HR",
            "display_name": "Test Worker EQ-HR", "employment_type": "permanent", "department": "people",
            "job_title": "HR", "hired_on": "2023-01-05" })).await.json::<Value>()["pid"].as_str().unwrap().to_string();
        ctx.db.execute_unprepared(&format!(
            "INSERT INTO organization_memberships (pid, person_ref, organization_ref, worker_pid, role, starts_on) \
             VALUES ('{}', 'person:{hr_p}', '{org}', '{hr_w}', 'hr_admin', '2023-01-05')", uuid::Uuid::new_v4())).await.unwrap();
        let put = |i: usize, body: Value| {
            let (request, t) = (&request, token(i));
            async move { request.put("/api/me/equality-monitoring").add_header("authorization", t).json(&body).await }
        };
        let get = |path: &str, t: String| {
            let request = &request;
            let path = path.to_string();
            async move { request.get(&path).add_header("authorization", t).await }
        };

        // ── Off: no lawful basis recorded ─────────────────────────────────────────────
        let off: Value = get("/api/me/equality-monitoring", token(0)).await.json();
        assert_eq!(off, json!({ "enabled": false }), "nothing about categories is even offered");
        assert_eq!(put(0, json!({ "declarations": { "sex": "female" } })).await.status_code(), 404, "it does not exist");
        assert_eq!(get("/api/equality-monitoring/summary?category=sex", hr.clone()).await.status_code(), 404);
        // Categories alone switch nothing on: the basis is what does.
        unsafe { std::env::set_var("WPM_EQUALITY_CATEGORIES", CATEGORIES); }
        assert_eq!(put(0, json!({ "declarations": { "sex": "female" } })).await.status_code(), 404);

        // ── On: a basis, a floor of five ──────────────────────────────────────────────
        unsafe {
            std::env::set_var("WPM_EQUALITY_MONITORING_BASIS", "Test deployment: impact assessment EQ-1");
            std::env::set_var("WPM_EQUALITY_FLOOR", "5");
        }
        let on: Value = get("/api/me/equality-monitoring", token(0)).await.json();
        assert_eq!(on["enabled"], true);
        assert_eq!(on["lawful_basis"], "Test deployment: impact assessment EQ-1");
        assert_eq!(on["categories"]["sex"], json!(["female", "male", "prefer_not_to_say"]));
        assert!(on["notice"].as_str().unwrap().contains("voluntary"));
        // Bad answers.
        for (why, body) in [
            ("unknown category", json!({ "declarations": { "age": "30" } })),
            ("value not offered", json!({ "declarations": { "sex": "robot" } })),
            ("nothing given", json!({ "declarations": {} })),
        ] {
            assert_eq!(put(0, body).await.status_code(), 422, "{why}");
        }
        assert_eq!(request.put("/api/me/equality-monitoring").json(&json!({ "declarations": { "sex": "female" } })).await.status_code(), 401);

        // Engineering: six female, six male. Operations: two female, one male.
        for i in 0..12 {
            let v = if i < 6 { "female" } else { "male" };
            assert_eq!(put(i, json!({ "declarations": { "sex": v } })).await.status_code(), 200, "worker {i}");
        }
        for (i, v) in [(12, "female"), (13, "female"), (14, "male")] {
            put(i, json!({ "declarations": { "sex": v } })).await.assert_status_ok();
        }
        // A worker can change their mind, and prefer_not_to_say is an answer.
        put(14, json!({ "declarations": { "sex": "prefer_not_to_say" } })).await.assert_status_ok();
        put(14, json!({ "declarations": { "sex": "male" } })).await.assert_status_ok();

        // ── Only the worker reads their own ───────────────────────────────────────────
        let mine: Value = get("/api/me/equality-monitoring", token(0)).await.json();
        assert_eq!(mine["declarations"], json!({ "sex": "female" }));
        let theirs: Value = get("/api/me/equality-monitoring", token(7)).await.json();
        assert_eq!(theirs["declarations"], json!({ "sex": "male" }), "each sees only their own");
        // No route returns anyone else's. The HR and worker routes carry nothing of it.
        for path in [format!("/api/workers/{}", pids[0]), format!("/api/workers/{}/contact-details", pids[0])] {
            let body = get(&path, hr.clone()).await.text();
            assert!(!body.contains("female") && !body.contains("equality"), "{path}: {body}");
        }
        // The export: the worker gets their own answers; anyone else running it gets a note.
        let own_export: Value = get(&format!("/api/workers/{}/subject-access", pids[0]), token(0)).await.json();
        assert_eq!(own_export["equality_monitoring"][0]["value"], "female");
        let admin_export: Value = get(&format!("/api/workers/{}/subject-access", pids[0]), svc.clone()).await.json();
        assert!(admin_export["equality_monitoring"].as_str().unwrap().starts_with("withheld"));
        assert!(!admin_export.to_string().contains("\"female\""), "the answers are not in the other export");

        // ── The aggregate ─────────────────────────────────────────────────────────────
        assert_eq!(get("/api/equality-monitoring/summary?category=sex", token(0)).await.status_code(), 403, "not a worker's to read");
        assert_eq!(get("/api/equality-monitoring/summary?category=age", hr.clone()).await.status_code(), 422);
        let sum: Value = get("/api/equality-monitoring/summary?category=sex", hr.clone()).await.json();
        assert_eq!(sum["small_group_floor"], 5);
        assert_eq!(sum["headcount"], 16, "everyone employed in the organization, HR included");
        assert_eq!(sum["responded"], 15);
        assert_eq!(sum["completeness_percent"], 93, "15 of 16, rounded down; names no value");
        assert_eq!(sum["organization"]["female"], 8);
        assert_eq!(sum["organization"]["male"], 7);
        assert!(sum["by_department"].is_null(), "operations has cells of 2 and 1, so no department breakdown at all");
        // A category where one value is small: the large cell shows, the small one is withheld.
        for i in 0..5 {
            put(i, json!({ "declarations": { "religion_or_belief": "none" } })).await.assert_status_ok();
        }
        for i in 5..7 {
            put(i, json!({ "declarations": { "religion_or_belief": "other" } })).await.assert_status_ok();
        }
        let rel: Value = get("/api/equality-monitoring/summary?category=religion_or_belief", hr.clone()).await.json();
        assert_eq!(rel["responded"], 7);
        assert_eq!(rel["organization"]["none"], 5, "exactly the floor shows");
        assert!(rel["organization"]["other"].is_null(), "two is withheld, not shown as 2");
        // Before anyone else answers, no answers means no figures at all.
        // A category nobody has answered: completeness is zero and nothing shows.
        let none: Value = get("/api/equality-monitoring/summary?category=disability", hr.clone()).await.json();
        assert_eq!((none["completeness_percent"].as_i64(), none["responded"].as_i64()), (Some(0), Some(0)));
        assert!(none["organization"].is_null());
        // The audit says that answers changed and that the aggregate was read; it holds no value.
        let audits = get(&format!("/api/audits/{}", pids[0]), svc.clone()).await.text();
        assert!(audits.contains("equality_monitoring_declared"));
        assert!(!audits.contains("female") && !audits.contains("\"sex\""));
        let summary_audit = get("/api/audits/00000000-0000-0000-0000-000000000000", svc.clone()).await.text();
        assert!(summary_audit.contains("equality_monitoring_summary_read") && !summary_audit.contains("female"));

        // ── Withdrawal and erasure ────────────────────────────────────────────────────
        assert_eq!(request.delete("/api/me/equality-monitoring/disability").add_header("authorization", token(0)).await.status_code(), 404, "nothing to withdraw");
        assert_eq!(request.delete("/api/me/equality-monitoring/sex").add_header("authorization", token(0)).await.status_code(), 200);
        let after: Value = get("/api/me/equality-monitoring", token(0)).await.json();
        assert_eq!(after["declarations"], json!({ "religion_or_belief": "none" }), "only the withdrawn category went");
        assert_eq!(request.delete("/api/me/equality-monitoring").add_header("authorization", token(1)).await.status_code(), 200);
        let count = |pid: &str| {
            let db = ctx.db.clone();
            let pid = pid.to_string();
            async move {
                db.query_one_raw(sea_orm::Statement::from_string(sea_orm::DbBackend::Postgres,
                    format!("SELECT count(*)::bigint AS n FROM equality_declarations WHERE worker_pid = '{pid}'")))
                    .await.unwrap().unwrap().try_get::<i64>("", "n").unwrap()
            }
        };
        assert_eq!(count(&pids[2]).await, 2, "a sex answer and a religion answer");
        for to in ["offboarding", "terminated"] {
            request.post(&format!("/api/workers/{}/status", pids[2])).add_header("authorization", svc.clone())
                .json(&json!({ "to": to })).await.assert_status_ok();
        }
        assert_eq!(request.post(&format!("/api/workers/{}/erase", pids[2])).add_header("authorization", svc.clone()).await.status_code(), 200);
        assert_eq!(count(&pids[2]).await, 0, "erased with the person");
        // Switching the basis off again removes the feature, and the data stays only in the table.
        unsafe { std::env::remove_var("WPM_EQUALITY_MONITORING_BASIS"); }
        assert_eq!(get("/api/me/equality-monitoring", token(3)).await.json::<Value>(), json!({ "enabled": false }));
        assert_eq!(get("/api/equality-monitoring/summary?category=sex", hr.clone()).await.status_code(), 404);
    })
    .await;
}
