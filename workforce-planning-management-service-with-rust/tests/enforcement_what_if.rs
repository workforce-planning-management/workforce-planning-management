//! Who may ask the capacity what-if, and what every audience is told (WPM-R97, WPM-T160, WPM-D142).
//!
//! Its own test binary (the auth flag and policy are cached in process-wide `OnceLock`s). The answer
//! is one payload for every caller who may ask: pool-month figures and counts, with no worker, no
//! person reference, no worker number, no name and no list of members. Who may ask is the policy's
//! write rule (a what-if is a POST) plus the organization's scope; a caller who is only a partner
//! reads the frozen packs and nothing else.

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

const REFERENCE_POLICY_FILE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/config/abac-policy.reference.json"
);

/// The reference policy with one extra rule that lets a `planner` write, so a deployer who grants
/// planners the what-if is covered, and an ordinary manager's refusal comes from the policy.
fn policy_with_planners() -> String {
    let text = std::fs::read_to_string(REFERENCE_POLICY_FILE).expect("the reference policy");
    let mut policy: Value = serde_json::from_str(&text).expect("valid JSON");
    policy["rules"].as_array_mut().unwrap().insert(
        0,
        json!({ "effect": "allow", "actions": ["write"], "when": { "planner": ["true"] } }),
    );
    policy.to_string()
}

/// Keys that would name a person or list people; none may appear anywhere in an answer.
const PEOPLE_KEYS: [&str; 9] = [
    "worker_pid",
    "worker_pids",
    "person_ref",
    "display_name",
    "worker_number",
    "workers",
    "people",
    "names",
    "email",
];

/// Walk a JSON value: no people key anywhere, and a `members` field (if present) is a bare count.
fn assert_counts_only(value: &Value, at: &str) {
    match value {
        Value::Object(map) => {
            for (key, inner) in map {
                assert!(
                    !PEOPLE_KEYS.contains(&key.as_str()),
                    "`{key}` names or lists people at {at}"
                );
                if key == "members" {
                    assert!(
                        inner.is_number() || inner.is_null(),
                        "members must be a count at {at}/{key}: {inner}"
                    );
                }
                assert_counts_only(inner, &format!("{at}/{key}"));
            }
        }
        Value::Array(items) => {
            for (i, v) in items.iter().enumerate() {
                assert_counts_only(v, &format!("{at}/{i}"));
            }
        }
        _ => {}
    }
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test --test enforcement_what_if -- --ignored`"]
#[allow(clippy::too_many_lines)] // one scenario read top to bottom
async fn the_what_if_answers_each_persona_with_counts_only_or_refuses_it() {
    use sea_orm::ConnectionTrait;
    let (keys, kid) = keys_and_kid();
    unsafe {
        std::env::set_var("WPM_REQUIRE_AUTH", "1");
        std::env::set_var("WPM_PASETO_KEYS", keys.to_string());
        std::env::remove_var("WPM_ABAC_POLICY_FILE");
        std::env::set_var("WPM_ABAC_POLICY", policy_with_planners());
    }
    let ids: Vec<uuid::Uuid> = (0..6).map(|_| uuid::Uuid::new_v4()).collect();
    let (hr_p, planner_p, manager_p, partner_p, elsewhere_p, loner_p) =
        (ids[0], ids[1], ids[2], ids[3], ids[4], ids[5]);
    let bearer = |t: String| format!("Bearer {t}");
    let svc = bearer(sign_as(&kid, "svc-user", &[("svc", &["true"])]));
    let hr = bearer(sign_as(&kid, &hr_p.to_string(), &[("hr", &["true"])]));
    let planner = bearer(sign_as(
        &kid,
        &planner_p.to_string(),
        &[("planner", &["true"])],
    ));
    let manager = bearer(sign_as(&kid, &manager_p.to_string(), &[]));
    let partner = bearer(sign_as(&kid, &partner_p.to_string(), &[]));
    let elsewhere = bearer(sign_as(
        &kid,
        &elsewhere_p.to_string(),
        &[("hr", &["true"])],
    ));
    let loner = bearer(sign_as(&kid, &loner_p.to_string(), &[("hr", &["true"])]));

    request::<App, _, _>(|request, ctx| async move {
        let org = format!("organization:{}", uuid::Uuid::new_v4());
        let other_org = format!("organization:{}", uuid::Uuid::new_v4());
        let grant = |person: String, organization: &str, role: &'static str| {
            let (db, organization) = (&ctx.db, organization.to_string());
            async move {
                db.execute_unprepared(&format!(
                    "INSERT INTO organization_memberships (pid, person_ref, organization_ref, role, starts_on) \
                     VALUES ('{}', 'person:{person}', '{organization}', '{role}', '2023-01-05')", uuid::Uuid::new_v4())).await.unwrap();
            }
        };
        grant("svc-user".to_string(), &org, "org_admin").await;
        grant(hr_p.to_string(), &org, "hr_admin").await;
        grant(planner_p.to_string(), &org, "member").await;
        grant(manager_p.to_string(), &org, "member").await;
        grant(partner_p.to_string(), &org, "partner").await;
        grant(elsewhere_p.to_string(), &other_org, "hr_admin").await;
        let get = |path: String, token: String| { let request = &request; async move { request.get(&path).add_header("authorization", token).await } };
        let post = |path: String, token: String, body: Value| { let request = &request; async move { request.post(&path).add_header("authorization", token).json(&body).await } };

        // Something a leak could carry: three named workers in one pool, and another programme's claims.
        let tag = uuid::Uuid::new_v4().simple().to_string();
        let title = format!("WhatIf Persona Engineer {tag}");
        for n in 1..=3 {
            let w = post("/api/workers".into(), svc.clone(), json!({
                "person_ref": format!("person:{}", uuid::Uuid::new_v4()), "organization_ref": org,
                "worker_number": format!("WIP-SECRET-{n}"), "display_name": format!("Persona Secret Name {n}"),
                "employment_type": "permanent", "fte_percent": 100, "department": "engineering",
                "job_title": title, "hired_on": "2026-01-05" })).await;
            assert_eq!(w.status_code(), 200);
            let pid = w.json::<Value>()["pid"].as_str().unwrap().to_string();
            assert_eq!(post(format!("/api/workers/{pid}/status"), svc.clone(), json!({ "to": "active" })).await.status_code(), 200);
        }
        let profile = post("/api/role-profiles".into(), svc.clone(), json!({ "job_title": title })).await.json::<Value>();
        let pool = post("/api/skill-pools".into(), svc.clone(), json!({ "organization_ref": org, "name": "Synthetic persona pool", "operations_reservation_bp": 0 })).await.json::<Value>();
        let pool = pool["pid"].as_str().unwrap().to_string();
        post(format!("/api/skill-pools/{pool}/members"), svc.clone(), json!({ "role_profile_pid": profile["pid"] })).await.assert_status_ok();
        let (other, mine) = (format!("thing:{}", uuid::Uuid::new_v4()), format!("thing:{}", uuid::Uuid::new_v4()));
        for (programme, month, fte) in [(&other, "2027-01-01", 200), (&mine, "2027-01-01", 50)] {
            let put = request.put("/api/programme-demands").add_header("authorization", svc.clone())
                .json(&json!({ "programme_ref": programme, "pool_pid": pool, "month": month, "fte_centi": fte })).await;
            assert_eq!(put.status_code(), 200);
        }
        ctx.db.execute_unprepared(&format!("UPDATE programme_demands SET status = 'active' WHERE programme_ref IN ('{other}', '{mine}')")).await.unwrap();

        let path = "/api/capacity/what-if".to_string();
        let ask = json!({ "organization_ref": org, "programme_ref": mine, "programme_new_end": "2027-06-30",
                          "changes": [{ "pool_pid": pool, "month": "2027-01-01", "fte_centi": 100 }] });
        // Signed out: refused.
        assert_eq!(request.post(&path).json(&ask).await.status_code(), 401, "sign in");
        // Who may ask: the policy's write rule (service, HR, and a planner under this deployer's rule) within the organization's scope.
        let mut answers = Vec::new();
        for (who, token) in [("service", &svc), ("hr", &hr), ("planner", &planner)] {
            let response = post(path.clone(), token.clone(), ask.clone()).await;
            assert_eq!(response.status_code(), 200, "{who} may ask");
            answers.push((who, response.json::<Value>()));
        }
        // An ordinary manager is not given write by the policy, so is refused; a partner-only caller is refused outright.
        assert_eq!(post(path.clone(), manager.clone(), ask.clone()).await.status_code(), 403, "a manager without the write rule");
        assert_eq!(post(path.clone(), partner.clone(), ask.clone()).await.status_code(), 403, "a partner reads packs only");
        // Out of scope: another organization's HR, and an HR caller with no membership, are told it does not exist.
        assert_eq!(post(path.clone(), elsewhere.clone(), ask.clone()).await.status_code(), 404, "another organization");
        assert_eq!(post(path.clone(), loner.clone(), ask.clone()).await.status_code(), 404, "no membership");

        // Every audience that may ask gets the same payload, and it holds counts and figures only.
        assert_eq!(answers[0].1["answer"], "breaks", "{}", answers[0].1);
        for (who, body) in &answers {
            assert_eq!(body, &answers[0].1, "{who} is told the same as the service");
            assert_counts_only(body, who);
            let text = body.to_string();
            for leak in ["WIP-SECRET", "Persona Secret Name", "person:", other.as_str(), "worker_pid"] {
                assert!(!text.contains(leak), "{who}: the answer must not carry `{leak}`: {text}");
            }
            assert_eq!(body["stored"], false);
            assert!(body["contingent_ending"].is_array(), "the contingent counts are there for {who}");
        }

        // The capacity view a manager can read is also counts only: `members` is a number, never a list.
        for (who, token) in [("manager", &manager), ("hr", &hr), ("planner", &planner)] {
            let response = get(format!("/api/capacity?organization={org}"), token.clone()).await;
            assert_eq!(response.status_code(), 200, "{who} reads the view");
            let body = response.json::<Value>();
            assert_counts_only(&body, who);
            let text = body.to_string();
            for leak in ["WIP-SECRET", "Persona Secret Name", "person:"] {
                assert!(!text.contains(leak), "{who}: the view must not carry `{leak}`");
            }
        }
        assert_eq!(get(format!("/api/capacity?organization={org}"), partner.clone()).await.status_code(), 403);
        assert_eq!(get(format!("/api/capacity?organization={org}"), elsewhere.clone()).await.status_code(), 404);

        // People-impact (WPM-R211, WPM-T361), under the caller's own token: an in-scope caller is told the programme's counted claim in
        // counts only; another organization's HR is told nothing is known (not zero, and no pool of this organization); a partner is
        // refused; signed out is refused.
        let impact = format!("/api/capacity/people-impact?programme={mine}");
        for (who, token) in [("hr", &hr), ("planner", &planner), ("manager", &manager)] {
            let response = get(impact.clone(), token.clone()).await;
            assert_eq!(response.status_code(), 200, "{who} reads people-impact");
            let body = response.json::<Value>();
            assert_eq!(body["over_committed_pool_months"]["known"], true, "{who}: {body}");
            assert_eq!(body["over_committed_pool_months"]["considered"], 1, "{who}: {body}");
            assert_counts_only(&body, who);
            let text = body.to_string();
            for leak in ["WIP-SECRET", "Persona Secret Name", "person:", other.as_str()] {
                assert!(!text.contains(leak), "{who}: people-impact must not carry `{leak}`");
            }
        }
        let outside = get(impact.clone(), elsewhere.clone()).await;
        assert_eq!(outside.status_code(), 200);
        let outside = outside.json::<Value>();
        for part in ["workers_committed", "contingent_ending_before_end", "over_committed_pool_months"] {
            assert_eq!(outside[part]["known"], false, "another organization learns nothing: {outside}");
        }
        assert!(!outside.to_string().contains(&pool), "no pool of this organization is named to another");
        assert_eq!(get(impact.clone(), partner.clone()).await.status_code(), 403, "a partner reads packs only");
        assert_eq!(request.get(&impact).await.status_code(), 401, "sign in");
    })
    .await;
}
