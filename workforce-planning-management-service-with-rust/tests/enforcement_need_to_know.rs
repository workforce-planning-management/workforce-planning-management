//! Need-to-know on reads under enforcement (WPM-R114, WPM-D71).
//!
//! Its own test binary (the auth flag and policy are cached in process-wide `OnceLock`s). It runs
//! against the shipped reference policy and checks every kind of caller against every class of
//! route: a stranger, a colleague in the same organization, the line manager, a manager higher
//! up, the worker, HR by attribute, payroll by attribute, HR by membership of the worker's
//! organization, and HR by membership of *another* organization.
//!
//! Before this guard, a signed-in caller with no attributes could read a colleague's sick-leave
//! request with its reason, their time-entry notes and every applicant's e-mail address.

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

use sea_orm::ConnectionTrait;

/// One boot, every persona against every class.
#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test --test enforcement_need_to_know -- --ignored`"]
#[allow(clippy::too_many_lines)] // one matrix read top to bottom
async fn reads_are_need_to_know() {
    let (keys, kid) = keys_and_kid();
    unsafe {
        std::env::set_var("WPM_REQUIRE_AUTH", "1");
        std::env::set_var("WPM_PASETO_KEYS", keys.to_string());
        std::env::set_var("WPM_ABAC_POLICY_FILE", REFERENCE_POLICY_FILE);
    }
    let people: Vec<uuid::Uuid> = (0..8).map(|_| uuid::Uuid::new_v4()).collect();
    let [
        worker_p,
        mgr_p,
        boss_p,
        colleague_p,
        stranger_p,
        hr_member_p,
        hr_other_p,
        _spare,
    ] = [0, 1, 2, 3, 4, 5, 6, 7].map(|i| people[i]);
    let bearer = |t: String| format!("Bearer {t}");
    let as_person = |p: uuid::Uuid| bearer(sign_as(&kid, &p.to_string(), &[]));
    let svc = bearer(sign_as(&kid, "svc-user", &[("svc", &["true"])]));
    let hr_attr = bearer(sign_as(&kid, "hr-user", &[("hr", &["true"])]));
    let payroll_attr = bearer(sign_as(&kid, "payroll-user", &[("payroll", &["true"])]));

    request::<App, _, _>(|request, ctx| async move {
        let org_a = format!("organization:{}", uuid::Uuid::new_v4());
        let org_b = format!("organization:{}", uuid::Uuid::new_v4());
        let make = |person: uuid::Uuid, number: &'static str, org: String, manager: Option<String>| {
            let request = &request;
            let t = svc.clone();
            async move {
                let mut body = json!({
                    "person_ref": format!("person:{person}"), "organization_ref": org,
                    "worker_number": number, "display_name": format!("Test Worker {number}"),
                    "employment_type": "permanent", "department": "engineering",
                    "job_title": "Engineer", "hired_on": "2026-01-05",
                });
                if let Some(m) = manager {
                    body["manager_pid"] = json!(m);
                }
                let r = request.post("/api/workers").add_header("authorization", t).json(&body).await;
                assert_eq!(r.status_code(), 200, "{number}");
                r.json::<Value>()["pid"].as_str().unwrap().to_string()
            }
        };
        let w_boss = make(boss_p, "NK-B", org_a.clone(), None).await;
        let w_mgr = make(mgr_p, "NK-M", org_a.clone(), Some(w_boss.clone())).await;
        let w = make(worker_p, "NK-W", org_a.clone(), Some(w_mgr.clone())).await;
        let w_col = make(colleague_p, "NK-C", org_a.clone(), None).await;
        let w_hr = make(hr_member_p, "NK-H", org_a.clone(), None).await;
        let w_other = make(hr_other_p, "NK-O", org_b.clone(), None).await;
        let _ = (&w_mgr, &w_boss);
        // Everyone employed belongs to their organization; the guard decides who reads whom.
        for (person, org, wp, role) in [
            (worker_p, &org_a, &w, "member"),
            (mgr_p, &org_a, &w_mgr, "member"),
            (boss_p, &org_a, &w_boss, "member"),
            (colleague_p, &org_a, &w_col, "member"),
            (hr_member_p, &org_a, &w_hr, "hr_admin"),
            (hr_other_p, &org_b, &w_other, "hr_admin"),
        ] {
            ctx.db
                .execute_unprepared(&format!(
                    "INSERT INTO organization_memberships (pid, person_ref, organization_ref, worker_pid, role, starts_on) \
                     VALUES ('{}', 'person:{person}', '{org}', '{wp}', '{role}', '2026-01-05')",
                    uuid::Uuid::new_v4()
                ))
                .await
                .unwrap();
        }
        // Something sensitive to read.
        let post = |path: String, body: Value| {
            let request = &request;
            let t = svc.clone();
            async move {
                let r = request.post(&path).add_header("authorization", t).json(&body).await;
                assert_eq!(r.status_code(), 200, "{path}");
                r.json::<Value>()
            }
        };
        post(format!("/api/workers/{w}/leave-entitlements"), json!({ "kind": "sick", "year": 2026, "entitled_days": 30 })).await;
        post(format!("/api/workers/{w}/leave-requests"), json!({ "kind": "sick", "start_on": "2026-03-02", "end_on": "2026-03-06" })).await;
        post("/api/candidates".into(), json!({ "display_name": "Test Applicant 9", "email": "nine@example.com",
                                                 "source": "referral", "person_ref": format!("person:{}", uuid::Uuid::new_v4()) })).await;
        let cycle = post("/api/review-cycles".into(), json!({ "name": "NK cycle", "period_start": "2026-01-01", "period_end": "2026-12-31" })).await;
        let review = post(
            format!("/api/review-cycles/{}/reviews", cycle["pid"].as_str().unwrap()),
            json!({ "worker_pid": w, "reviewer_ref": format!("worker:{}", uuid::Uuid::new_v4()) }),
        )
        .await;
        let r = review["pid"].as_str().unwrap().to_string();

        // The classes, one route each.
        let routes: [(&str, String); 6] = [
            ("self-only", format!("/api/workers/{w}/adjustment-requests")),
            ("with-managers", format!("/api/workers/{w}/leave-requests")),
            ("in-scope", format!("/api/workers/{w}/reports")),
            ("worker record", format!("/api/workers/{w}")),
            ("privileged", "/api/candidates".to_string()),
            ("review record", format!("/api/reviews/{r}")),
        ];
        // Who may read each, in the order above. `true` = 200, `false` = 403.
        let personas: [(&str, String, [bool; 6]); 11] = [
            ("stranger", as_person(stranger_p), [false, false, false, false, false, false]),
            ("colleague in the same organization", as_person(colleague_p), [false, false, true, true, false, false]),
            ("line manager", as_person(mgr_p), [false, true, true, true, false, true]),
            ("manager higher up", as_person(boss_p), [false, true, true, true, false, true]),
            ("the worker", as_person(worker_p), [true, true, true, true, false, true]),
            ("HR by attribute", hr_attr.clone(), [true; 6]),
            ("payroll by attribute", payroll_attr.clone(), [true; 6]),
            ("service", svc.clone(), [true; 6]),
            ("HR by membership of the worker's organization", as_person(hr_member_p), [true; 6]),
            // HR of another organization reads nothing about this worker; the applicant list is
            // not organization-scoped, so any HR role may read it.
            ("HR by membership of another organization", as_person(hr_other_p), [false, false, false, false, true, false]),
            ("unknown token", bearer("v4.public.junk".into()), [false; 6]),
        ];
        for (name, token, expected) in &personas {
            for ((class, path), allowed) in routes.iter().zip(expected) {
                let status = request.get(path).add_header("authorization", token.clone()).await.status_code().as_u16();
                if name == &"unknown token" {
                    assert_eq!(status, 401, "{name} / {class}");
                } else if *allowed && *class == "in-scope" {
                    // Past the guard. The controller then lists a worker's reports within the
                    // caller's own organizations, so a caller with none (HR by attribute) gets
                    // an empty answer or a 404 from it, which is the controller's scope rule.
                    assert!(matches!(status, 200 | 404), "{name} may read the {class} route: {status}");
                } else if *allowed {
                    assert_eq!(status, 200, "{name} may read the {class} route");
                } else {
                    assert_eq!(status, 403, "{name} may not read the {class} route");
                }
            }
        }

        // The refusal says why and carries none of the record.
        let refused = request
            .get(&format!("/api/workers/{w}/leave-requests"))
            .add_header("authorization", as_person(stranger_p))
            .await;
        let body = refused.text();
        assert!(body.contains("need-to-know"));
        assert!(!body.contains("sick"));

        // A worker that does not exist is the controller's 404, not a leak of which pids exist.
        let missing = request
            .get(&format!("/api/workers/{}/leave-requests", uuid::Uuid::new_v4()))
            .add_header("authorization", as_person(stranger_p))
            .await;
        assert_eq!(missing.status_code(), 404, "the controller's answer, same as for any caller");
    })
    .await;
}
