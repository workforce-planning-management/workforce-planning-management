//! Who may write and read a cutover under enforcement (WPM-R70, WPM-T117).
//!
//! Its own test binary (the auth flag and policy are cached in process-wide `OnceLock`s). It runs against
//! the shipped reference policy plus one rule that lets a `decider` write, so a refusal below comes from the
//! cutover's own rule: planning a go-live is for an administrator of the organization, whoever the policy
//! lets write. Reading the readiness is open to anyone in the organization's scope and to nobody outside it,
//! and a caller who holds only the `partner` role reads nothing here.

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
/// the pack's own rule (an administrator of the organization) and not from the policy.
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
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test --test enforcement_cutover -- --ignored`"]
async fn only_an_administrator_plans_a_cutover_and_only_the_organization_reads_it() {
    use sea_orm::ConnectionTrait;
    let (keys, kid) = keys_and_kid();
    unsafe {
        std::env::set_var("WPM_REQUIRE_AUTH", "1");
        std::env::set_var("WPM_PASETO_KEYS", keys.to_string());
        std::env::remove_var("WPM_ABAC_POLICY_FILE");
        std::env::set_var("WPM_ABAC_POLICY", policy_with_deciders());
    }
    let ids: Vec<uuid::Uuid> = (0..4).map(|_| uuid::Uuid::new_v4()).collect();
    let [member_p, admin_p, partner_p, outsider_p] = [0, 1, 2, 3].map(|i| ids[i]);
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
    let partner = bearer(sign_as(
        &kid,
        &partner_p.to_string(),
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
        grant(partner_p.to_string(), &org, "partner").await;
        grant(outsider_p.to_string(), &other, "hr_admin").await;
        let get = |path: String, token: String| { let request = &request; async move { request.get(&path).add_header("authorization", token).await } };
        let post = |path: String, token: String, body: Value| { let request = &request; async move { request.post(&path).add_header("authorization", token).json(&body).await } };
        let profile = post("/api/role-profiles".into(), svc.clone(), json!({ "job_title": "Synthetic Operator" })).await.json::<Value>();
        ctx.db.execute_unprepared(&format!(
            "INSERT INTO training_enrollments (pid, worker_pid, course_ref, status) VALUES ('{}', '{}', 'SYN-COURSE', 'enrolled')",
            uuid::Uuid::new_v4(), uuid::Uuid::new_v4())).await.unwrap();
        let body = json!({ "organization_ref": org, "programme_ref": format!("thing:{}", uuid::Uuid::new_v4()), "name": "Synthetic go-live",
            "site": "Synthetic Site", "window_start": "2027-03-01", "window_end": "2027-03-03" });
        // Planning: signed in, in scope, and an administrator of the organization.
        assert_eq!(request.post("/api/cutovers").json(&body).await.status_code(), 401, "sign in");
        assert_eq!(post("/api/cutovers".into(), member.clone(), body.clone()).await.status_code(), 403, "a member the policy lets write is still not an administrator");
        assert_eq!(post("/api/cutovers".into(), partner.clone(), body.clone()).await.status_code(), 403, "a partner reads packs and nothing else");
        assert_eq!(post("/api/cutovers".into(), outsider.clone(), body.clone()).await.status_code(), 404, "an administrator of another organization cannot see this one");
        let made = post("/api/cutovers".into(), admin.clone(), body.clone()).await;
        assert_eq!(made.status_code(), 200, "an administrator of the organization");
        let id = made.json::<Value>()["pid"].as_str().unwrap().to_string();
        let role = json!({ "role_profile_pid": profile["pid"], "course_ref": "SYN-COURSE", "required_headcount": 1 });
        assert_eq!(post(format!("/api/cutovers/{id}/roles"), member.clone(), role.clone()).await.status_code(), 403);
        assert_eq!(post(format!("/api/cutovers/{id}/roles"), outsider.clone(), role.clone()).await.status_code(), 404);
        assert_eq!(post(format!("/api/cutovers/{id}/roles"), admin.clone(), role.clone()).await.status_code(), 200);
        let pool = post("/api/skill-pools".into(), svc.clone(), json!({ "organization_ref": org, "name": "Synthetic pool", "operations_reservation_bp": 0 })).await.json::<Value>();
        assert_eq!(post(format!("/api/cutovers/{id}/pools"), member.clone(), json!({ "pool_pid": pool["pid"] })).await.status_code(), 403);
        assert_eq!(post(format!("/api/cutovers/{id}/pools"), admin.clone(), json!({ "pool_pid": pool["pid"] })).await.status_code(), 200);
        // Reading: anyone in the organization's scope, nobody outside it, no partner, no anonymous caller.
        let readiness = format!("/api/cutovers/{id}/readiness");
        let list = format!("/api/cutovers?organization={org}");
        assert_eq!(request.get(&readiness).await.status_code(), 401);
        assert_eq!(get(readiness.clone(), member.clone()).await.status_code(), 200);
        assert_eq!(get(list.clone(), member.clone()).await.status_code(), 200);
        assert_eq!(get(readiness.clone(), outsider.clone()).await.status_code(), 404);
        assert_eq!(get(list.clone(), outsider.clone()).await.status_code(), 404);
        // The key-person register reads one organization when asked, and only one the caller can see.
        let register = format!("/api/workforce-intelligence/key-person?organization={org}");
        assert_eq!(get(register.clone(), member.clone()).await.status_code(), 200);
        assert_eq!(get(register.clone(), outsider.clone()).await.status_code(), 404, "another organization's register is not readable");
        assert_eq!(get(readiness.clone(), partner.clone()).await.status_code(), 403);
        assert_eq!(get(list.clone(), partner.clone()).await.status_code(), 403);
        // Removal is for an administrator too.
        assert_eq!(request.delete(&format!("/api/cutovers/{id}")).add_header("authorization", member.clone()).await.status_code(), 403);
        assert_eq!(request.delete(&format!("/api/cutovers/{id}")).add_header("authorization", outsider.clone()).await.status_code(), 404);
        assert_eq!(request.delete(&format!("/api/cutovers/{id}")).add_header("authorization", admin.clone()).await.status_code(), 200);
    })
    .await;
}
