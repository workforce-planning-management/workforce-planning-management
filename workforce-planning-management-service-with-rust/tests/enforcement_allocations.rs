//! Per-person allocation under enforcement (WPM-R275, WPM-R276, WPM-D211).
//!
//! Its own test binary (the auth flag and policy are cached in process-wide `OnceLock`s), against the shipped reference policy
//! with nothing added: a resource manager's writes pass on the token role alone (`self_service::ATTRIBUTE_WRITES`). Who reads a
//! person's allocations by name (the person, their line managers, HR, resource managers), who writes them (HR and resource
//! managers, never a resource manager for themselves), and what everyone else sees (counts with the floor).

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
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test --test enforcement_allocations -- --ignored`"]
#[allow(clippy::too_many_lines)] // one matrix read top to bottom
async fn allocations_are_named_to_those_who_plan_and_counted_for_everyone_else() {
    let (keys, kid) = keys_and_kid();
    unsafe {
        std::env::set_var("WORKFORCE_PLANNING_MANAGEMENT_REQUIRE_AUTH", "1");
        std::env::set_var(
            "WORKFORCE_PLANNING_MANAGEMENT_PASETO_KEYS",
            keys.to_string(),
        );
        std::env::remove_var("WORKFORCE_PLANNING_MANAGEMENT_ABAC_POLICY");
        std::env::set_var(
            "WORKFORCE_PLANNING_MANAGEMENT_ABAC_POLICY_FILE",
            REFERENCE_POLICY_FILE,
        );
    }
    let p: Vec<uuid::Uuid> = (0..7).map(|_| uuid::Uuid::new_v4()).collect();
    let [
        worker_p,
        mgr_p,
        neighbour_p,
        hr_p,
        rm_p,
        rm_member_p,
        payroll_p,
    ] = [0, 1, 2, 3, 4, 5, 6].map(|i| p[i]);
    let bearer = |t: String| format!("Bearer {t}");
    let me = bearer(sign_as(&kid, &worker_p.to_string(), &[]));
    let manager = bearer(sign_as(&kid, &mgr_p.to_string(), &[]));
    let neighbour = bearer(sign_as(&kid, &neighbour_p.to_string(), &[]));
    let hr = bearer(sign_as(&kid, &hr_p.to_string(), &[("hr", &["true"])]));
    let rm = bearer(sign_as(
        &kid,
        &rm_p.to_string(),
        &[("resource_manager", &["true"])],
    ));
    let rm_member = bearer(sign_as(&kid, &rm_member_p.to_string(), &[]));
    let payroll = bearer(sign_as(
        &kid,
        &payroll_p.to_string(),
        &[("payroll", &["true"])],
    ));
    let svc = bearer(sign_as(&kid, "svc-user", &[("svc", &["true"])]));
    let admin_p = uuid::Uuid::new_v4();
    let admin = bearer(sign_as(
        &kid,
        &admin_p.to_string(),
        &[("access", &["admin"])],
    ));

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
        let w_mgr = make(mgr_p, "AE-M", None).await;
        let w = make(worker_p, "AE-W", Some(w_mgr.clone())).await;
        let w_rm = make(rm_p, "AE-R", None).await;
        let mut people = vec![(worker_p, Some(w.clone()), "member"), (mgr_p, Some(w_mgr.clone()), "member"), (rm_p, Some(w_rm.clone()), "member")];
        for person in [neighbour_p, hr_p, rm_member_p, payroll_p, admin_p] {
            people.push((person, None, "member"));
        }
        people.push((rm_member_p, None, "resource_manager"));
        for (person, worker, role) in people {
            let worker = worker.map_or("NULL".to_string(), |w| format!("'{w}'"));
            sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
                "INSERT INTO organization_memberships (pid, person_ref, organization_ref, worker_pid, role, starts_on) \
                 VALUES ('{}', 'person:{person}', '{org}', {worker}, '{role}', '2023-01-05')", uuid::Uuid::new_v4())))
                .execute(workforce_planning_management::db::pool(&ctx)).await.unwrap();
        }
        let status = |method: &'static str, path: String, token: String, body: Value| {
            let request = &request;
            async move {
                let r = match method {
                    "GET" => request.get(&path).add_header("authorization", token).await,
                    "POST" => request.post(&path).add_header("authorization", token).json(&body).await,
                    "PUT" => request.put(&path).add_header("authorization", token).json(&body).await,
                    _ => request.delete(&path).add_header("authorization", token).await,
                };
                (r.status_code().as_u16(), r.text())
            }
        };
        let assignment_of = |worker: String| {
            let (request, t) = (&request, svc.clone());
            async move {
                let a: Value = request.get(&format!("/api/workers/{worker}/assignments")).add_header("authorization", t).await.json();
                a["assignments"][0]["pid"].as_str().unwrap().to_string()
            }
        };
        let a_w = assignment_of(w.clone()).await;
        let a_rm = assignment_of(w_rm.clone()).await;
        let body = |assignment: &str| json!({
            "assignment_pid": assignment, "source_ref": "thing:programme-a",
            "from_month": "2027-01-01", "to_month": "2027-02-01", "fte_centi": 40, "status": "confirmed",
        });
        let url = format!("/api/workers/{w}/allocations");

        // ── Writing ──
        for (name, token) in [("the person", &me), ("their manager", &manager), ("a colleague", &neighbour),
                              ("payroll", &payroll), ("a resource manager by membership only", &rm_member)] {
            let (code, text) = status("POST", url.clone(), token.clone(), body(&a_w)).await;
            assert_eq!(code, 403, "{name} does not write allocations: {text}");
        }
        let (code, text) = status("POST", url.clone(), hr.clone(), body(&a_w)).await;
        assert_eq!(code, 200, "HR writes: {text}");
        let (code, text) = status("POST", url.clone(), rm.clone(), body(&a_w)).await;
        assert_eq!(code, 200, "a resource manager writes: {text}");
        let by_rm = serde_json::from_str::<Value>(&text).unwrap()["pid"].as_str().unwrap().to_string();
        let (code, text) = status("POST", format!("/api/workers/{w_rm}/allocations"), rm.clone(), body(&a_rm)).await;
        assert_eq!(code, 403, "a resource manager does not write their own: {text}");
        let (code, _) = status("POST", format!("/api/workers/{w_rm}/allocations"), hr.clone(), body(&a_rm)).await;
        assert_eq!(code, 200, "HR writes the resource manager's");
        for (method, path, b) in [
            ("PUT", format!("/api/allocations/{by_rm}"), json!({ "fte_centi": 30 })),
            ("POST", format!("/api/allocations/{by_rm}/status"), json!({ "to": "tentative" })),
        ] {
            let (code, text) = status(method, path.clone(), manager.clone(), b.clone()).await;
            assert_eq!(code, 403, "the manager may not {method} {path}: {text}");
            let (code, text) = status(method, path.clone(), rm.clone(), b).await;
            assert_eq!(code, 200, "the resource manager may {method} {path}: {text}");
        }
        let (code, _) = status("DELETE", format!("/api/allocations/{by_rm}"), neighbour.clone(), json!({})).await;
        assert_eq!(code, 403);
        let (code, text) = status("DELETE", format!("/api/allocations/{by_rm}"), rm.clone(), json!({})).await;
        assert_eq!(code, 200, "{text}");

        // ── Reading one person ──
        let audited = || {
            let ctx = &ctx;
            let w = w.clone();
            async move {
                sqlx::query_scalar::<_, i64>(sqlx::AssertSqlSafe(format!(
                    "SELECT count(*) FROM audit_logs WHERE action = 'allocations_read' AND entity_pid = '{w}'")))
                    .fetch_one(workforce_planning_management::db::pool(ctx)).await.unwrap()
            }
        };
        let before = audited().await;
        let (code, _) = status("GET", url.clone(), me.clone(), json!({})).await;
        assert_eq!(code, 200, "the person reads their own");
        assert_eq!(audited().await, before, "reading your own is not audited");
        for (name, token) in [("their manager", &manager), ("HR", &hr), ("a resource manager", &rm), ("a resource manager by membership", &rm_member)] {
            let (code, text) = status("GET", url.clone(), token.clone(), json!({})).await;
            assert_eq!(code, 200, "{name} reads: {text}");
        }
        assert_eq!(audited().await, before + 4, "each read of another person's allocations is audited");
        for (name, token) in [("a colleague", &neighbour), ("payroll", &payroll)] {
            let (code, text) = status("GET", url.clone(), token.clone(), json!({})).await;
            assert_eq!(code, 403, "{name} does not read them by name: {text}");
        }

        // An administrator reads as HR does.
        let (code, text) = status("GET", url.clone(), admin.clone(), json!({})).await;
        assert_eq!(code, 200, "an administrator reads: {text}");

        // ── The named list and the summary ──
        let list = format!("/api/allocations?organization={org}&from=2027-01-01&months=2");
        for (name, token, expected) in [("HR", &hr, 200), ("a resource manager", &rm, 200), ("a resource manager by membership", &rm_member, 200),
                                         ("a manager", &manager, 403), ("a colleague", &neighbour, 403), ("payroll", &payroll, 403)] {
            let (code, text) = status("GET", list.clone(), token.clone(), json!({})).await;
            assert_eq!(code, expected, "{name}: {text}");
        }
        sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
            "INSERT INTO capacity_settings (pid, organization_ref, allocation_tracked) VALUES ('{}', '{org}', TRUE)", uuid::Uuid::new_v4())))
            .execute(workforce_planning_management::db::pool(&ctx)).await.unwrap();
        let summary = format!("/api/allocations/summary?organization={org}&from=2027-01-01&months=1");
        let (code, text) = status("GET", summary.clone(), neighbour.clone(), json!({})).await;
        assert_eq!(code, 200, "{text}");
        let cells: Value = serde_json::from_str(&text).unwrap();
        let cell = &cells["cells"][0];
        assert_eq!((cell["staffing"].as_str(), cell["people"].is_null(), cell["allocated_centi"].is_null()), (Some("withheld"), true, true),
                   "under the floor a colleague sees nothing that could name someone: {cells}");
        let (_, text) = status("GET", summary, rm.clone(), json!({})).await;
        let cells: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(cells["cells"][0]["people"], 2, "the resource manager sees the count: {cells}");
        let (code, _) = status("GET", format!("/api/allocations/summary?organization=organization:{}", uuid::Uuid::new_v4()), rm.clone(), json!({})).await;
        assert_eq!(code, 404, "another organization is out of scope");
    })
    .await;
}
