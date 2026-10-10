//! Who may propose, read, approve and carry out a conversion plan under enforcement (WPM-R98, WPM-D66).
//!
//! Its own test binary (the auth flag and policy are cached in process-wide `OnceLock`s). It runs
//! against the shipped reference policy plus one extra rule that lets a `decider` write, so the
//! refusals come from the handlers' own rules: the worker never proposes or approves for
//! themself, a stranger and a manager outside the chain are refused, only HR reads, nobody
//! approves their own proposal, and marking a conversion done changes the worker in one step.

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

/// The reference policy with the one extra rule, as inline JSON.
fn policy_with_deciders() -> String {
    let text = std::fs::read_to_string(REFERENCE_POLICY_FILE).expect("the reference policy");
    let mut policy: Value = serde_json::from_str(&text).expect("valid JSON");
    policy["rules"].as_array_mut().unwrap().insert(
        0,
        json!({ "effect": "allow", "actions": ["write"], "when": { "decider": ["true"] } }),
    );
    policy.to_string()
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test --test enforcement_conversion -- --ignored`"]
#[allow(clippy::too_many_lines)] // one scenario read top to bottom
async fn a_conversion_is_proposed_by_a_manager_approved_by_someone_else_and_read_by_hr() {
    use sea_orm::ConnectionTrait;
    let (keys, kid) = keys_and_kid();
    unsafe {
        std::env::set_var("WPM_REQUIRE_AUTH", "1");
        std::env::set_var("WPM_PASETO_KEYS", keys.to_string());
        std::env::remove_var("WPM_ABAC_POLICY_FILE");
        std::env::set_var("WPM_ABAC_POLICY", policy_with_deciders());
    }
    let p: Vec<uuid::Uuid> = (0..5).map(|_| uuid::Uuid::new_v4()).collect();
    let [worker_p, mgr_p, other_mgr_p, neighbour_p, hr_p] = [0, 1, 2, 3, 4].map(|i| p[i]);
    let bearer = |t: String| format!("Bearer {t}");
    let me = bearer(sign_as(&kid, &worker_p.to_string(), &[]));
    let me_as_decider = bearer(sign_as(
        &kid,
        &worker_p.to_string(),
        &[("decider", &["true"])],
    ));
    let manager = bearer(sign_as(&kid, &mgr_p.to_string(), &[("decider", &["true"])]));
    let other_manager = bearer(sign_as(
        &kid,
        &other_mgr_p.to_string(),
        &[("decider", &["true"])],
    ));
    let neighbour = bearer(sign_as(&kid, &neighbour_p.to_string(), &[]));
    let hr = bearer(sign_as(&kid, &hr_p.to_string(), &[("hr", &["true"])]));
    let svc = bearer(sign_as(&kid, "svc-user", &[("svc", &["true"])]));

    request::<App, _, _>(|request, ctx| async move {
        let org = format!("organization:{}", uuid::Uuid::new_v4());
        let today = chrono::Utc::now().date_naive();
        let day = |n: i64| (today + chrono::Duration::days(n)).to_string();
        let make = |person: uuid::Uuid, number: &'static str, basis: &'static str, manager_pid: Option<String>| {
            let request = &request;
            let (t, org, end) = (svc.clone(), org.clone(), day(90));
            async move {
                let mut body = json!({
                    "person_ref": format!("person:{person}"), "organization_ref": org,
                    "worker_number": number, "display_name": format!("Test Worker {number}"),
                    "employment_type": basis, "department": "engineering",
                    "job_title": "Engineer", "hired_on": "2023-01-05",
                });
                if basis == "contractor" { body["engagement_ends_on"] = json!(end); }
                if let Some(m) = manager_pid { body["manager_pid"] = json!(m); }
                let r = request.post("/api/workers").add_header("authorization", t.clone()).json(&body).await;
                assert_eq!(r.status_code(), 200, "{number}");
                let pid = r.json::<Value>()["pid"].as_str().unwrap().to_string();
                request.post(&format!("/api/workers/{pid}/status")).add_header("authorization", t)
                    .json(&json!({ "to": "active" })).await.assert_status_ok();
                pid
            }
        };
        let w_mgr = make(mgr_p, "CV-M", "permanent", None).await;
        let w = make(worker_p, "CV-W", "contractor", Some(w_mgr.clone())).await;
        let w_other = make(other_mgr_p, "CV-O", "permanent", None).await;
        let w_nb = make(neighbour_p, "CV-N", "permanent", None).await;
        let w_hr = make(hr_p, "CV-H", "permanent", None).await;
        for (person, wp, role) in [(worker_p, &w, "member"), (mgr_p, &w_mgr, "member"), (other_mgr_p, &w_other, "member"), (neighbour_p, &w_nb, "member"), (hr_p, &w_hr, "hr_admin")] {
            ctx.db.execute_unprepared(&format!(
                "INSERT INTO organization_memberships (pid, person_ref, organization_ref, worker_pid, role, starts_on) \
                 VALUES ('{}', 'person:{person}', '{org}', '{wp}', '{role}', '2023-01-05')", uuid::Uuid::new_v4())).await.unwrap();
        }
        let post = |path: String, token: String, body: Value| {
            let request = &request;
            async move { request.post(&path).add_header("authorization", token).json(&body).await }
        };
        let get = |path: String, token: String| {
            let request = &request;
            async move { request.get(&path).add_header("authorization", token).await }
        };
        let plans = format!("/api/workers/{w}/conversion-plans");
        let convert = json!({ "intent": "convert", "target_on": day(30), "department": "platform",
                              "post_funding_kind": "core", "reason": "A standing need", "review_on": day(20) });

        // ── Proposing ──────────────────────────────────────────────────────────────────
        assert_eq!(request.post(&plans).json(&convert).await.status_code(), 401, "sign in");
        assert_eq!(post(plans.clone(), me.clone(), convert.clone()).await.status_code(), 403, "the worker has no right to write");
        let own = post(plans.clone(), me_as_decider.clone(), convert.clone()).await;
        assert_eq!(own.status_code(), 422, "nobody decides their own");
        assert_eq!(post(plans.clone(), neighbour.clone(), convert.clone()).await.status_code(), 403);
        assert_eq!(post(plans.clone(), other_manager.clone(), convert.clone()).await.status_code(), 403, "a manager outside the chain");
        for (why, body) in [
            ("no post funding and no reason", json!({ "intent": "convert", "target_on": day(30) })),
            ("after the contract ends", json!({ "intent": "convert", "target_on": day(120), "post_funding_kind": "core" })),
            ("unknown intent", json!({ "intent": "promote", "target_on": day(30) })),
        ] {
            assert_eq!(post(plans.clone(), manager.clone(), body).await.status_code(), 422, "{why}");
        }
        let proposed = post(plans.clone(), manager.clone(), convert.clone()).await;
        assert_eq!(proposed.status_code(), 200);
        let plan = proposed.json::<Value>();
        assert_eq!(plan["status"], "proposed");
        let pid = plan["pid"].as_str().unwrap().to_string();
        assert_eq!(post(plans.clone(), manager.clone(), convert.clone()).await.status_code(), 422, "one open plan at a time");

        // ── Reading: HR only ──────────────────────────────────────────────────────────
        for (name, token) in [("the worker", &me), ("the proposing manager", &manager), ("a neighbour", &neighbour)] {
            assert_eq!(get(plans.clone(), token.clone()).await.status_code(), 403, "{name} does not read plans");
            assert_eq!(get("/api/conversion-plans".into(), token.clone()).await.status_code(), 403, "{name}");
        }
        let seen: Vec<Value> = get(plans.clone(), hr.clone()).await.json();
        assert_eq!(seen.len(), 1);
        assert_eq!(seen[0]["reason"], "A standing need");
        let listed: Vec<Value> = get("/api/conversion-plans".into(), hr.clone()).await.json();
        assert!(listed.iter().any(|e| e["worker_pid"] == w.as_str() && e["plan"]["status"] == "proposed"));

        // ── Approving ──────────────────────────────────────────────────────────────────
        let approve = format!("/api/conversion-plans/{pid}/approve");
        let proposer = post(approve.clone(), manager.clone(), json!({})).await;
        assert_eq!(proposer.status_code(), 422);
        assert!(proposer.text().contains("nobody approves their own"));
        assert_eq!(post(approve.clone(), me_as_decider.clone(), json!({})).await.status_code(), 422, "not the worker");
        assert_eq!(post(approve.clone(), other_manager.clone(), json!({})).await.status_code(), 403);
        assert_eq!(post(format!("/api/conversion-plans/{}/approve", uuid::Uuid::new_v4()), hr.clone(), json!({})).await.status_code(), 404);
        let approved = post(approve.clone(), hr.clone(), json!({})).await;
        assert_eq!(approved.status_code(), 200);
        assert_eq!(approved.json::<Value>()["status"], "approved");
        assert_eq!(post(approve.clone(), hr.clone(), json!({})).await.status_code(), 422, "already approved");
        let engagement = get(format!("/api/workers/{w}/engagement"), hr.clone()).await.json::<Value>();
        assert_eq!(engagement["decision"], "convert", "approval settles the end date with a recorded decision");

        // ── Carrying it out ───────────────────────────────────────────────────────────
        let done = format!("/api/conversion-plans/{pid}/done");
        assert_eq!(post(done.clone(), neighbour.clone(), json!({})).await.status_code(), 403);
        assert_eq!(post(done.clone(), me_as_decider.clone(), json!({})).await.status_code(), 422);
        let before = get(format!("/api/workers/{w}"), svc.clone()).await.json::<Value>();
        assert_eq!(before["employment_type"], "contractor");
        assert_eq!(post(done.clone(), hr.clone(), json!({})).await.status_code(), 200);
        let after = get(format!("/api/workers/{w}"), svc.clone()).await.json::<Value>();
        assert_eq!(after["employment_type"], "permanent");
        assert_eq!(after["engagement_ends_on"], Value::Null);
        assert_eq!(after["department"], "platform", "the post the plan names");
        assert_eq!(post(done.clone(), hr.clone(), json!({})).await.status_code(), 422, "once");
        assert_eq!(post(format!("/api/conversion-plans/{pid}/abandon"), hr.clone(), json!({})).await.status_code(), 422, "a done plan stays done");
        let history: Vec<Value> = get(plans.clone(), hr.clone()).await.json();
        assert_eq!(history[0]["status"], "done", "the plan is kept as history");

        // A permanent worker has nothing to convert.
        assert_eq!(post(plans.clone(), manager.clone(), convert.clone()).await.status_code(), 422);

        // The audit trail names the events and never the reason.
        let audits = get(format!("/api/audits/{w}"), svc.clone()).await.text();
        assert!(audits.contains("conversion_plan_proposed") && audits.contains("conversion_plan_done"));
        assert!(!audits.contains("A standing need"));
    })
    .await;
}
