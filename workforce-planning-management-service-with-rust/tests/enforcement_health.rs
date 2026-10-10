//! Workplace health requirements under enforcement (WPM-R128, WPM-D75).
//!
//! Its own test binary (the auth flag and policy are cached in process-wide `OnceLock`s). The
//! feature is **off** until a deployer records a lawful basis. Once on, only an
//! occupational-health role records a status; the worker sees their own; a manager and HR see only
//! **cleared** or **not cleared**, never the status; no reason is held; and nothing else reads it.

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

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test --test enforcement_health -- --ignored`"]
#[allow(clippy::too_many_lines)] // one scenario read top to bottom
async fn a_manager_sees_cleared_or_not_and_never_the_status() {
    use sea_orm::ConnectionTrait;
    let (keys, kid) = keys_and_kid();
    unsafe {
        std::env::set_var("WPM_REQUIRE_AUTH", "1");
        std::env::set_var("WPM_PASETO_KEYS", keys.to_string());
        std::env::set_var("WPM_ABAC_POLICY_FILE", REFERENCE_POLICY_FILE);
        std::env::remove_var("WPM_HEALTH_REQUIREMENTS_BASIS");
    }
    let p: Vec<uuid::Uuid> = (0..8).map(|_| uuid::Uuid::new_v4()).collect();
    let [worker_p, office_p, mgr_p, neighbour_p, hr_p, oh_p, _a, _b] =
        [0, 1, 2, 3, 4, 5, 6, 7].map(|i| p[i]);
    let bearer = |t: String| format!("Bearer {t}");
    let tok =
        |who: uuid::Uuid, attrs: &[(&str, &[&str])]| bearer(sign_as(&kid, &who.to_string(), attrs));
    let worker = tok(worker_p, &[]);
    let office_worker = tok(office_p, &[]);
    let manager = tok(mgr_p, &[]);
    let neighbour = tok(neighbour_p, &[]);
    let hr = tok(hr_p, &[("hr", &["true"])]);
    let oh = tok(oh_p, &[("occupational_health", &["true"])]);
    let svc = bearer(sign_as(&kid, "svc-user", &[("svc", &["true"])]));

    request::<App, _, _>(|request, ctx| async move {
        let org = format!("organization:{}", uuid::Uuid::new_v4());
        let make = |person: uuid::Uuid, number: &'static str, department: &'static str, manager_pid: Option<String>| {
            let request = &request;
            let (t, org) = (svc.clone(), org.clone());
            async move {
                let mut body = json!({
                    "person_ref": format!("person:{person}"), "organization_ref": org,
                    "worker_number": number, "display_name": format!("Test Worker {number}"),
                    "employment_type": "permanent", "department": department,
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
        let w_mgr = make(mgr_p, "HR-M", "clinic", None).await;
        let w = make(worker_p, "HR-W", "clinic", Some(w_mgr.clone())).await;
        let w_office = make(office_p, "HR-O", "office", Some(w_mgr.clone())).await;
        let w_nb = make(neighbour_p, "HR-N", "office", None).await;
        let w_hr = make(hr_p, "HR-H", "people", None).await;
        for (person, wp, role) in [(worker_p, &w, "member"), (office_p, &w_office, "member"), (mgr_p, &w_mgr, "member"), (neighbour_p, &w_nb, "member"), (hr_p, &w_hr, "hr_admin")] {
            ctx.db.execute_unprepared(&format!(
                "INSERT INTO organization_memberships (pid, person_ref, organization_ref, worker_pid, role, starts_on) \
                 VALUES ('{}', 'person:{person}', '{org}', '{wp}', '{role}', '2023-01-05')", uuid::Uuid::new_v4())).await.unwrap();
        }
        let today = chrono::Utc::now().date_naive();
        let post = |path: String, t: String, body: Value| { let request = &request; async move { request.post(&path).add_header("authorization", t).json(&body).await } };
        let put = |path: String, t: String, body: Value| { let request = &request; async move { request.put(&path).add_header("authorization", t).json(&body).await } };
        let get = |path: String, t: String| { let request = &request; async move { request.get(&path).add_header("authorization", t).await } };

        // ── Off until a lawful basis is recorded ───────────────────────────────────────
        assert_eq!(get("/api/me/health-requirements".into(), worker.clone()).await.json::<Value>(), json!({ "enabled": false }));
        assert_eq!(post("/api/health-requirements".into(), hr.clone(), json!({ "name": "Hepatitis B" })).await.status_code(), 404);
        assert_eq!(get("/api/health-requirements/clearance".into(), manager.clone()).await.status_code(), 404);
        unsafe { std::env::set_var("WPM_HEALTH_REQUIREMENTS_BASIS", "Test deployment: occupational health policy OH-1"); }

        // ── Requirements: defined by HR; what is required, not why ─────────────────────
        assert_eq!(post("/api/health-requirements".into(), worker.clone(), json!({ "name": "x" })).await.status_code(), 403, "the policy: only HR writes");
        for (why, body) in [
            ("blank name", json!({ "name": "  " })),
            ("zero re-check", json!({ "name": "A", "recheck_calendar_days": 0 })),
            ("too long a description", json!({ "name": "A", "description": "y".repeat(301) })),
        ] {
            assert_eq!(post("/api/health-requirements".into(), hr.clone(), body).await.status_code(), 422, "{why}");
        }
        let hep = post("/api/health-requirements".into(), hr.clone(), json!({ "name": "Hepatitis B", "description": "Three doses", "departments": ["clinic"], "recheck_calendar_days": 365 })).await;
        assert_eq!(hep.status_code(), 200);
        let hep = hep.json::<Value>()["pid"].as_str().unwrap().to_string();
        let induction = post("/api/health-requirements".into(), hr.clone(), json!({ "name": "Safety induction" })).await.json::<Value>()["pid"].as_str().unwrap().to_string();
        assert_eq!(post("/api/health-requirements".into(), hr.clone(), json!({ "name": "hepatitis b" })).await.status_code(), 422, "names are unique");

        // ── What each worker is asked for ──────────────────────────────────────────────
        let mine: Value = get("/api/me/health-requirements".into(), worker.clone()).await.json();
        assert_eq!(mine["enabled"], true);
        assert_eq!(mine["requirements"].as_array().unwrap().len(), 2, "both apply in the clinic");
        assert!(mine["requirements"].as_array().unwrap().iter().all(|r| r["standing"] == "not_recorded" && r["cleared"] == false));
        let office_view: Value = get("/api/me/health-requirements".into(), office_worker.clone()).await.json();
        assert_eq!(office_view["requirements"].as_array().unwrap().len(), 1, "hepatitis B is for the clinic only");

        // ── Only occupational health records ───────────────────────────────────────────
        let record = |who: String, target: &str, req: &str, body: Value| put(format!("/api/workers/{target}/health-requirements/{req}"), who, body);
        for (name, token) in [("the worker", worker.clone()), ("their manager", manager.clone()), ("a neighbour", neighbour.clone()), ("HR", hr.clone()), ("service", svc.clone())] {
            let r = record(token, &w, &hep, json!({ "status": "up_to_date" })).await;
            assert_eq!(r.status_code(), 403, "{name} cannot record a status");
        }
        assert_eq!(record(oh.clone(), &w, &hep, json!({ "status": "vaccinated" })).await.status_code(), 422, "a status from the list");
        assert_eq!(record(oh.clone(), &w, &hep, json!({ "status": "up_to_date", "recorded_on": (today + chrono::Duration::days(1)).to_string() })).await.status_code(), 422);
        assert_eq!(record(oh.clone(), &w_office, &hep, json!({ "status": "up_to_date" })).await.status_code(), 422, "does not apply to the office");
        assert_eq!(record(oh.clone(), &w, &uuid::Uuid::new_v4().to_string(), json!({ "status": "up_to_date" })).await.status_code(), 404);
        let done = record(oh.clone(), &w, &hep, json!({ "status": "up_to_date" })).await;
        assert_eq!(done.status_code(), 200);
        let done_text = done.text();
        assert!(done_text.contains("\"cleared\":true") && !done_text.contains("up_to_date"), "a yes, not an echo of the status: {done_text}");

        // The worker sees their own standing and dates.
        let own: Value = get("/api/me/health-requirements".into(), worker.clone()).await.json();
        let hep_view = own["requirements"].as_array().unwrap().iter().find(|r| r["requirement"]["name"] == "Hepatitis B").unwrap();
        assert_eq!(hep_view["standing"], "up_to_date");
        assert_eq!(hep_view["next_due"], (today + chrono::Duration::days(365)).to_string().as_str(), "the re-check period in calendar days");
        // Overdue: the reminder shows and they are no longer cleared.
        ctx.db.execute_unprepared(&format!("UPDATE worker_health_records SET next_due = CURRENT_DATE - 1 WHERE worker_pid = '{w}'")).await.unwrap();
        let late: Value = get("/api/me/health-requirements".into(), worker.clone()).await.json();
        let late_hep = late["requirements"].as_array().unwrap().iter().find(|r| r["requirement"]["name"] == "Hepatitis B").unwrap();
        assert_eq!((late_hep["standing"].as_str(), late_hep["reminder"].as_bool(), late_hep["cleared"].as_bool()), (Some("overdue"), Some(true), Some(false)));
        record(oh.clone(), &w, &hep, json!({ "status": "up_to_date" })).await.assert_status_ok();
        // An exemption and a refusal, recorded by occupational health: no reason is asked for or held.
        record(oh.clone(), &w, &induction, json!({ "status": "exempt_recorded" })).await.assert_status_ok();
        record(oh.clone(), &w_mgr, &hep, json!({ "status": "declined" })).await.assert_status_ok();

        // ── A manager and HR see cleared or not, and never the status ───────────────────
        let clearance = get("/api/health-requirements/clearance".into(), manager.clone()).await;
        assert_eq!(clearance.status_code(), 200);
        let text = clearance.text();
        for word in ["up_to_date", "exempt", "declined", "overdue", "recorded_on", "next_due", "status"] {
            assert!(!text.contains(word), "the manager must not see `{word}`: {text}");
        }
        let rows: Vec<Value> = serde_json::from_str(&text).unwrap();
        for row in &rows {
            let mut keys: Vec<&str> = row.as_object().unwrap().keys().map(String::as_str).collect();
            keys.sort_unstable();
            assert_eq!(keys, ["cleared", "department", "requirement", "worker_number", "worker_pid"]);
        }
        let find = |worker_number: &str, requirement: &str| rows.iter().find(|r| r["worker_number"] == worker_number && r["requirement"] == requirement).map(|r| r["cleared"].as_bool().unwrap());
        assert_eq!(find("HR-W", "Hepatitis B"), Some(true));
        assert_eq!(find("HR-W", "Safety induction"), Some(true), "an exemption is cleared, and the manager is not told it is one");
        assert_eq!(find("HR-M", "Hepatitis B"), None, "a manager does not see their own row as a manager of themself");
        assert_eq!(find("HR-O", "Safety induction"), Some(false), "not recorded is not cleared");
        let hr_rows: Vec<Value> = get("/api/health-requirements/clearance".into(), hr.clone()).await.json();
        assert!(hr_rows.iter().any(|r| r["worker_number"] == "HR-M" && r["requirement"] == "Hepatitis B" && r["cleared"] == false), "declined is not cleared");
        assert!(!serde_json::to_string(&hr_rows).unwrap().contains("declined"));
        let not_cleared: Vec<Value> = get("/api/health-requirements/clearance?not_cleared=true".into(), hr.clone()).await.json();
        assert!(not_cleared.iter().all(|r| r["cleared"] == false) && !not_cleared.is_empty());
        let theirs: Vec<Value> = get("/api/health-requirements/clearance".into(), neighbour.clone()).await.json();
        assert!(theirs.is_empty(), "a neighbour manages no one");

        // ── The detail: occupational health only, and audited ─────────────────────────
        for (name, token) in [("the worker", worker.clone()), ("their manager", manager.clone()), ("HR", hr.clone()), ("a neighbour", neighbour.clone())] {
            assert_eq!(get(format!("/api/health-requirements/records?worker={w}"), token).await.status_code(), 403, "{name} does not read the detail");
        }
        let detail = get(format!("/api/health-requirements/records?worker={w}"), oh.clone()).await;
        assert_eq!(detail.status_code(), 200);
        assert!(detail.text().contains("exempt_recorded"), "occupational health sees the status");
        let audits = get(format!("/api/audits/{w}"), svc.clone()).await.text();
        assert!(audits.contains("health_requirement_recorded") && audits.contains("health_record_read"));
        assert!(!audits.contains("up_to_date") && !audits.contains("exempt_recorded") && !audits.contains("declined"), "no status in the audit");

        // ── The aggregate: HR and occupational health, small groups withheld ───────────
        assert_eq!(get("/api/health-requirements/compliance".into(), neighbour.clone()).await.status_code(), 403);
        let comp: Value = get("/api/health-requirements/compliance".into(), hr.clone()).await.json();
        assert!(comp["compliance"].as_array().unwrap().iter().all(|c| c["applicable"].is_null()), "a handful of people shows nothing");
        assert_eq!(get("/api/health-requirements/compliance".into(), oh.clone()).await.status_code(), 200);

        // ── Export, correction and erasure ────────────────────────────────────────────
        let own_export: Value = get(format!("/api/workers/{w}/subject-access"), worker.clone()).await.json();
        assert!(own_export["workplace_health_records"].as_array().unwrap().len() >= 2);
        let other_export: Value = get(format!("/api/workers/{w}/subject-access"), svc.clone()).await.json();
        assert!(other_export["workplace_health_records"].as_str().unwrap().starts_with("withheld"));
        assert_eq!(request.delete(&format!("/api/workers/{w}/health-requirements/{induction}")).add_header("authorization", hr.clone()).await.status_code(), 403);
        assert_eq!(request.delete(&format!("/api/workers/{w}/health-requirements/{induction}")).add_header("authorization", oh.clone()).await.status_code(), 200, "removing a mistaken entry");
        assert_eq!(request.delete(&format!("/api/workers/{w}/health-requirements/{induction}")).add_header("authorization", oh.clone()).await.status_code(), 404);
        let count = || {
            let db = ctx.db.clone();
            let w = w.clone();
            async move {
                db.query_one_raw(sea_orm::Statement::from_string(sea_orm::DbBackend::Postgres,
                    format!("SELECT count(*)::bigint AS n FROM worker_health_records WHERE worker_pid = '{w}'")))
                    .await.unwrap().unwrap().try_get::<i64>("", "n").unwrap()
            }
        };
        assert_eq!(count().await, 1);
        for to in ["offboarding", "terminated"] {
            request.post(&format!("/api/workers/{w}/status")).add_header("authorization", svc.clone()).json(&json!({ "to": to })).await.assert_status_ok();
        }
        assert_eq!(request.post(&format!("/api/workers/{w}/erase")).add_header("authorization", svc.clone()).await.status_code(), 200);
        assert_eq!(count().await, 0, "erased with the person");
        // Switched off again: the feature disappears.
        unsafe { std::env::remove_var("WPM_HEALTH_REQUIREMENTS_BASIS"); }
        assert_eq!(get("/api/health-requirements".into(), worker.clone()).await.status_code(), 404);
        let _ = post;
    })
    .await;
}
