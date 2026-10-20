//! Who may raise and who may decide an escalation under enforcement (WPM-R141, WPM-D94).
//!
//! Its own test binary (the auth flag and policy are cached in process-wide `OnceLock`s). It runs against
//! the shipped reference policy plus one rule that lets a `decider` write, so a refusal below comes from the
//! escalation's own rule: anyone who may write to the plan can say a breach needs a decision, but only an
//! administrator of the plan's organization decides it, and a caller outside the organization cannot see the plan.

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

/// The reference policy with one extra rule that lets a `decider` write, so a refusal below comes from
/// the escalation's own rule (an administrator of the organization) and not from the policy.
fn policy_with_deciders() -> String {
    let text = std::fs::read_to_string(REFERENCE_POLICY_FILE).expect("the reference policy");
    let mut policy: Value = serde_json::from_str(&text).expect("valid JSON");
    policy["rules"].as_array_mut().unwrap().insert(
        0,
        json!({ "effect": "allow", "actions": ["write", "delete"], "when": { "decider": ["true"] } }),
    );
    policy.to_string()
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test --test enforcement_escalations -- --ignored`"]
async fn anyone_who_writes_may_raise_but_only_an_administrator_decides() {
    use sea_orm::ConnectionTrait;
    let (keys, kid) = keys_and_kid();
    unsafe {
        std::env::set_var("WPM_REQUIRE_AUTH", "1");
        std::env::set_var("WPM_PASETO_KEYS", keys.to_string());
        std::env::remove_var("WPM_ABAC_POLICY_FILE");
        std::env::set_var("WPM_ABAC_POLICY", policy_with_deciders());
    }
    let ids: Vec<uuid::Uuid> = (0..3).map(|_| uuid::Uuid::new_v4()).collect();
    let [member_p, admin_p, outsider_p] = [0, 1, 2].map(|i| ids[i]);
    let bearer = |t: String| format!("Bearer {t}");
    let member = bearer(sign_as(
        &kid,
        &member_p.to_string(),
        &[("decider", &["true"])],
    ));
    let admin = bearer(sign_as(
        &kid,
        &admin_p.to_string(),
        &[("decider", &["true"])],
    ));
    let outsider = bearer(sign_as(
        &kid,
        &outsider_p.to_string(),
        &[("decider", &["true"])],
    ));
    let svc = bearer(sign_as(&kid, "svc-user", &[("svc", &["true"])]));

    request::<App, _, _>(|request, ctx| async move {
        let org = format!("organization:{}", uuid::Uuid::new_v4());
        let other = format!("organization:{}", uuid::Uuid::new_v4());
        let grant = |person: String, organization: &str, role: &'static str| {
            let (db, organization) = (&ctx.db, organization.to_string());
            async move {
                db.execute_unprepared(&format!(
                    "INSERT INTO organization_memberships (pid, person_ref, organization_ref, role, starts_on) \
                     VALUES ('{}', 'person:{person}', '{organization}', '{role}', '2023-01-05')", uuid::Uuid::new_v4())).await.unwrap();
            }
        };
        grant("svc-user".to_string(), &org, "org_admin").await;
        grant(member_p.to_string(), &org, "member").await;
        grant(admin_p.to_string(), &org, "hr_admin").await;
        grant(outsider_p.to_string(), &other, "hr_admin").await;
        let post = |path: String, token: String, body: Value| { let request = &request; async move { request.post(&path).add_header("authorization", token).json(&body).await } };
        let put = |path: String, token: String, body: Value| { let request = &request; async move { request.put(&path).add_header("authorization", token).json(&body).await } };

        // a plan that is badly short: two employed against ten wanted, long past its date
        for n in 0..2 {
            let worker = json!({ "person_ref": format!("person:{}", uuid::Uuid::new_v4()), "organization_ref": org,
                "worker_number": format!("S{n}"), "display_name": format!("Synthetic Worker {n}"), "employment_type": "permanent",
                "department": "engineering", "job_title": "Engineer", "hired_on": "2020-01-05" });
            assert_eq!(post("/api/workers".into(), svc.clone(), worker).await.status_code(), 200);
        }
        let plan = post("/api/workforce-plans".into(), svc.clone(), json!({ "name": "Synthetic plan", "organization_ref": org,
            "horizon_start": "2026-01-01", "horizon_end": "2027-12-31", "attrition_bp": 1000 })).await.json::<Value>();
        let pid = plan["pid"].as_str().unwrap().to_string();
        assert_eq!(put(format!("/api/workforce-plans/{pid}/demand-lines"), svc.clone(),
            json!({ "department": "engineering", "target_on": "2026-06-30", "target_headcount": 10 })).await.status_code(), 200);
        assert_eq!(put(format!("/api/workforce-plans/{pid}/tolerances/resources"), svc.clone(), json!({ "limit": 1000 })).await.status_code(), 200);

        // raising: signed in, able to see the plan, and allowed to write
        let url = format!("/api/workforce-plans/{pid}/escalations");
        let body = json!({ "dimension": "resources", "note": "needs a decision" });
        assert_eq!(request.post(&url).json(&body).await.status_code(), 401, "sign in");
        assert_eq!(post(url.clone(), outsider.clone(), body.clone()).await.status_code(), 404, "another organization cannot see the plan");
        let raised = post(url.clone(), member.clone(), body.clone()).await;
        assert_eq!(raised.status_code(), 200, "a member the policy lets write may say a breach needs a decision");
        let id = raised.json::<Value>()["pid"].as_str().unwrap().to_string();
        assert_eq!(post(format!("/api/escalations/{id}/raise-level"), outsider.clone(), json!({})).await.status_code(), 404);
        assert_eq!(post(format!("/api/escalations/{id}/raise-level"), member.clone(), json!({})).await.status_code(), 200);

        // deciding: an administrator of the plan's organization, not a member, not another organization's administrator
        let decide = format!("/api/escalations/{id}/decide");
        let verdict = json!({ "decision": "accept_deviation", "note": "accepted" });
        assert_eq!(post(decide.clone(), member.clone(), verdict.clone()).await.status_code(), 403, "a member the policy lets write is still not an administrator");
        assert_eq!(post(decide.clone(), outsider.clone(), verdict.clone()).await.status_code(), 404, "an administrator of another organization cannot see this one");
        let done = post(decide.clone(), admin.clone(), verdict.clone()).await;
        assert_eq!(done.status_code(), 200, "an administrator of the organization");
        assert_eq!(done.json::<Value>()["decided_by"], admin_p.to_string(), "who decided is the signed-in person");

        // the inbox is for those who can see the organization
        let inbox = format!("/api/escalations?organization={org}");
        assert_eq!(request.get(&inbox).add_header("authorization", member.clone()).await.status_code(), 200);
        assert_eq!(request.get(&inbox).add_header("authorization", outsider.clone()).await.status_code(), 404);
    })
    .await;
}
