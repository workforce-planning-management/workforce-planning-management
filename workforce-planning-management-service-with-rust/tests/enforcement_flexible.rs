//! Flexible working requests under enforcement (WPM-R126, WPM-D77).
//!
//! Its own test binary (the auth flag and policy are cached in process-wide `OnceLock`s). The policy
//! is the shipped reference policy **plus one rule a deployer would add**: callers carrying
//! `decider=true` may write, which is how a line manager is given the power to decide a request
//! (the reference policy lets only HR write). The service then decides *which* requests such a caller
//! may decide: their own reports', never their own, and an appeal never by who refused.

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
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test --test enforcement_flexible -- --ignored`"]
#[allow(clippy::too_many_lines)] // one scenario read top to bottom
async fn a_request_is_decided_by_a_person_who_may_and_never_by_the_requester() {
    use sea_orm::ConnectionTrait;
    let (keys, kid) = keys_and_kid();
    unsafe {
        std::env::set_var("WPM_REQUIRE_AUTH", "1");
        std::env::set_var("WPM_PASETO_KEYS", keys.to_string());
        std::env::remove_var("WPM_ABAC_POLICY_FILE");
        std::env::set_var("WPM_ABAC_POLICY", policy_with_deciders());
        std::env::set_var("WPM_FLEXIBLE_REFUSAL_REASONS", "too_costly,cannot_cover");
    }
    let p: Vec<uuid::Uuid> = (0..6).map(|_| uuid::Uuid::new_v4()).collect();
    let [worker_p, mgr_p, other_mgr_p, neighbour_p, hr_p, _spare] =
        [0, 1, 2, 3, 4, 5].map(|i| p[i]);
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
        let make = |person: uuid::Uuid, number: &'static str, manager_pid: Option<String>| {
            let request = &request;
            let (t, org) = (svc.clone(), org.clone());
            async move {
                let mut body = json!({
                    "person_ref": format!("person:{person}"), "organization_ref": org,
                    "worker_number": number, "display_name": format!("Test Worker {number}"),
                    "employment_type": "permanent", "department": "engineering",
                    "job_title": "Engineer", "hired_on": "2023-01-05",
                });
                if let Some(m) = manager_pid { body["manager_pid"] = json!(m); }
                let r = request.post("/api/workers").add_header("authorization", t.clone()).json(&body).await;
                assert_eq!(r.status_code(), 200, "{number}");
                let pid = r.json::<Value>()["pid"].as_str().unwrap().to_string();
                request.post(&format!("/api/workers/{pid}/status")).add_header("authorization", t)
                    .json(&json!({ "to": "active" })).await.assert_status_ok();
                pid
            }
        };
        let w_mgr = make(mgr_p, "FW-M", None).await;
        let w = make(worker_p, "FW-W", Some(w_mgr.clone())).await;
        let w_other = make(other_mgr_p, "FW-O", None).await;
        let w_nb = make(neighbour_p, "FW-N", None).await;
        let w_hr = make(hr_p, "FW-H", None).await;
        for (person, wp, role) in [(worker_p, &w, "member"), (mgr_p, &w_mgr, "member"), (other_mgr_p, &w_other, "member"), (neighbour_p, &w_nb, "member"), (hr_p, &w_hr, "hr_admin")] {
            ctx.db.execute_unprepared(&format!(
                "INSERT INTO organization_memberships (pid, person_ref, organization_ref, worker_pid, role, starts_on) \
                 VALUES ('{}', 'person:{person}', '{org}', '{wp}', '{role}', '2023-01-05')", uuid::Uuid::new_v4())).await.unwrap();
        }
        let today = chrono::Utc::now().date_naive();
        let day = |n: i64| (today + chrono::Duration::days(n)).to_string();
        let post = |path: String, token: String, body: Value| {
            let request = &request;
            async move { request.post(&path).add_header("authorization", token).json(&body).await }
        };
        let get = |path: String, token: String| {
            let request = &request;
            async move { request.get(&path).add_header("authorization", token).await }
        };

        // ── Asking ──────────────────────────────────────────────────────────────────────
        for (why, body) in [
            ("unknown kind", json!({ "kind": "sabbatical", "proposed_start": day(30) })),
            ("start in the past", json!({ "kind": "hours", "proposed_start": day(-1) })),
            ("fte out of range", json!({ "kind": "hours", "proposed_start": day(30), "proposed_fte_percent": 0 })),
        ] {
            assert_eq!(post("/api/me/flexible-working".into(), me.clone(), body).await.status_code(), 422, "{why}");
        }
        assert_eq!(request.post("/api/me/flexible-working").json(&json!({ "kind": "hours", "proposed_start": day(30) })).await.status_code(), 401);
        let first = post("/api/me/flexible-working".into(), me.clone(), json!({
            "kind": "hours", "proposed_start": day(30), "proposed_fte_percent": 60,
            "effect_on_team": "Fridays are quiet", "how_to_manage": "Swap the Friday rota" })).await;
        assert_eq!(first.status_code(), 200);
        let f1 = first.json::<Value>();
        let expected_by = today.checked_add_months(chrono::Months::new(2)).unwrap().to_string();
        assert_eq!(f1["decide_by"], expected_by.as_str(), "two calendar months by default");
        let r1 = f1["pid"].as_str().unwrap().to_string();
        let second = post("/api/me/flexible-working".into(), me.clone(), json!({ "kind": "place_of_work", "proposed_start": day(60) })).await;
        let r2 = second.json::<Value>()["pid"].as_str().unwrap().to_string();
        // A third in the year is over the limit (two by default); withdrawing one makes room.
        assert_eq!(post("/api/me/flexible-working".into(), me.clone(), json!({ "kind": "days", "proposed_start": day(90) })).await.status_code(), 422);
        assert_eq!(post(format!("/api/me/flexible-working/{r2}/withdraw"), me.clone(), json!({})).await.status_code(), 200);
        let third = post("/api/me/flexible-working".into(), me.clone(), json!({ "kind": "days", "proposed_start": day(90) })).await;
        assert_eq!(third.status_code(), 200, "a withdrawn request does not count");
        let r3 = third.json::<Value>()["pid"].as_str().unwrap().to_string();
        assert_eq!(post(format!("/api/me/flexible-working/{r2}/withdraw"), me.clone(), json!({})).await.status_code(), 422, "already withdrawn");
        // Someone else's request cannot be withdrawn through /me.
        assert_eq!(post(format!("/api/me/flexible-working/{r1}/withdraw"), neighbour.clone(), json!({})).await.status_code(), 404);

        // The manager is told; who sees the requests.
        let told = get(format!("/api/workers/{w_mgr}/notifications"), manager.clone()).await.text();
        assert!(told.contains("flexible_working_requested") && !told.contains("Fridays"), "told, with nothing of the text");
        assert_eq!(get(format!("/api/workers/{w}/flexible-working"), neighbour.clone()).await.status_code(), 403);
        assert_eq!(get(format!("/api/workers/{w}/flexible-working"), manager.clone()).await.status_code(), 200);
        assert_eq!(get(format!("/api/workers/{w}/flexible-working"), hr.clone()).await.status_code(), 200);
        let queue: Vec<Value> = get("/api/flexible-working?status=requested".into(), manager.clone()).await.json();
        assert_eq!(queue.len(), 2, "the manager sees their report's two open requests");
        assert!(queue.iter().all(|q| q["overdue"] == false));
        for (name, token) in [("a manager outside the chain", other_manager.clone()), ("a neighbour", neighbour.clone())] {
            let theirs: Vec<Value> = get("/api/flexible-working?status=requested".into(), token).await.json();
            assert!(theirs.is_empty(), "{name} sees none");
        }
        let hr_queue: Vec<Value> = get("/api/flexible-working?status=requested".into(), hr.clone()).await.json();
        assert_eq!(hr_queue.len(), 2, "HR sees them all");

        // ── Who may decide ─────────────────────────────────────────────────────────────
        let decide = |pid: &str| format!("/api/flexible-working/{pid}/decide");
        // A caller with no right to write at all.
        assert_eq!(post(decide(&r1), neighbour.clone(), json!({ "outcome": "approve" })).await.status_code(), 403);
        assert_eq!(post(decide(&r1), me.clone(), json!({ "outcome": "approve" })).await.status_code(), 403, "the worker cannot approve their own");
        // Even a worker who has been given the power to decide cannot decide their own.
        let own = post(decide(&r1), me_as_decider.clone(), json!({ "outcome": "approve" })).await;
        assert_eq!(own.status_code(), 422);
        assert!(own.text().contains("nobody decides their own"));
        // A manager who is not in the chain is refused.
        assert_eq!(post(decide(&r1), other_manager.clone(), json!({ "outcome": "approve" })).await.status_code(), 403);
        // A refusal needs a reason from the deployer's list.
        assert_eq!(post(decide(&r1), manager.clone(), json!({ "outcome": "refuse" })).await.status_code(), 422);
        assert_eq!(post(decide(&r1), manager.clone(), json!({ "outcome": "refuse", "reason": "additional_cost" })).await.status_code(), 422, "not on this deployment's list");
        assert_eq!(post(decide(&r1), manager.clone(), json!({ "outcome": "maybe" })).await.status_code(), 422);
        let refused = post(decide(&r1), manager.clone(), json!({ "outcome": "refuse", "reason": "too_costly", "note": "Cover is not affordable this quarter" })).await;
        assert_eq!(refused.status_code(), 200);
        assert_eq!(refused.json::<Value>()["status"], "refused");
        assert_eq!(post(decide(&r1), manager.clone(), json!({ "outcome": "approve" })).await.status_code(), 422, "already decided");
        // The worker is told, and sees the reason.
        let own_view: Vec<Value> = get("/api/me/flexible-working".into(), me.clone()).await.json();
        let seen = own_view.iter().find(|r| r["pid"] == r1.as_str()).unwrap();
        assert_eq!((seen["decision_reason"].as_str(), seen["can_appeal"].as_bool()), (Some("too_costly"), Some(true)));
        assert!(get(format!("/api/workers/{w}/notifications"), me.clone()).await.text().contains("flexible_working_decided"));

        // ── Appeal ─────────────────────────────────────────────────────────────────────
        assert_eq!(post(format!("/api/me/flexible-working/{r1}/appeal"), neighbour.clone(), json!({})).await.status_code(), 404);
        assert_eq!(post(format!("/api/me/flexible-working/{r3}/appeal"), me.clone(), json!({})).await.status_code(), 422, "only a refusal is appealed");
        assert_eq!(post(format!("/api/me/flexible-working/{r1}/appeal"), me.clone(), json!({ "note": "The quarter ends soon" })).await.status_code(), 200);
        assert_eq!(post(format!("/api/me/flexible-working/{r1}/appeal"), me.clone(), json!({})).await.status_code(), 422, "once");
        let appeal = |outcome: &str| json!({ "outcome": outcome });
        let same = post(format!("/api/flexible-working/{r1}/appeal-decision"), manager.clone(), appeal("uphold")).await;
        assert_eq!(same.status_code(), 422, "the appeal is not decided by who refused");
        assert!(same.text().contains("someone other than who refused"));
        let upheld = post(format!("/api/flexible-working/{r1}/appeal-decision"), hr.clone(), appeal("uphold")).await;
        assert_eq!(upheld.status_code(), 200);
        let up = upheld.json::<Value>();
        assert_eq!(up["status"], "approved");
        assert_eq!(up["hr_to_apply_fte_percent"], 60, "a proposal for HR; the contract is not changed here");
        // The contract is untouched by any of it.
        let contract = get(format!("/api/workers/{w}"), svc.clone()).await.json::<Value>();
        assert_eq!(contract["fte_percent"], 100, "an approved change of hours is HR's to apply");

        // ── A counter-proposal ─────────────────────────────────────────────────────────
        let counter = |note: Option<&str>| json!({ "outcome": "counter", "counter_note": note, "counter_fte_percent": 80 });
        assert_eq!(post(decide(&r3), manager.clone(), counter(None)).await.status_code(), 422, "says what is offered");
        assert_eq!(post(decide(&r3), manager.clone(), json!({ "outcome": "counter", "counter_note": "Four days", "counter_fte_percent": 0 })).await.status_code(), 422);
        assert_eq!(post(decide(&r3), manager.clone(), counter(Some("Four days a week"))).await.status_code(), 200);
        assert_eq!(post(format!("/api/me/flexible-working/{r3}/respond"), me.clone(), json!({ "response": "maybe" })).await.status_code(), 422);
        assert_eq!(post(format!("/api/me/flexible-working/{r3}/respond"), neighbour.clone(), json!({ "response": "accept" })).await.status_code(), 404);
        let accepted = post(format!("/api/me/flexible-working/{r3}/respond"), me.clone(), json!({ "response": "accept" })).await.json::<Value>();
        assert_eq!(accepted["status"], "accepted");
        assert_eq!(accepted["hr_to_apply_fte_percent"], 80, "the counter-proposal's hours, not the first ask");

        // ── Overdue ────────────────────────────────────────────────────────────────────
        let late = post("/api/me/flexible-working".into(), me.clone(), json!({ "kind": "times", "proposed_start": day(120) }));
        assert_eq!(late.await.status_code(), 422, "the limit of two in twelve months is reached again");
        // Make an open request overdue by moving its date back, as time would.
        ctx.db.execute_unprepared(&format!(
            "UPDATE flexible_working_requests SET status = 'requested', decide_by = CURRENT_DATE - 1, decided_on = NULL, decided_by = NULL WHERE pid = '{r2}'")).await.unwrap();
        let overdue: Vec<Value> = get("/api/flexible-working?overdue=true".into(), manager.clone()).await.json();
        assert_eq!(overdue.len(), 1);
        assert_eq!(overdue[0]["pid"], r2.as_str());
        assert_eq!(overdue[0]["overdue"], true);

        // ── Records ────────────────────────────────────────────────────────────────────
        let audits = get(format!("/api/audits/{w}"), svc.clone()).await.text();
        assert!(audits.contains("flexible_working_requested") && audits.contains("flexible_working_refused"));
        assert!(!audits.contains("Fridays") && !audits.contains("Swap the Friday") && !audits.contains("Cover is not affordable"), "no text in the audit");
        let export = get(format!("/api/workers/{w}/subject-access"), me.clone()).await.json::<Value>();
        assert_eq!(export["flexible_working_requests"].as_array().unwrap().len(), 3);
    })
    .await;
}
