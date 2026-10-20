//! Who may read the monthly oversight pack under enforcement (WPM-R69, WPM-T116).
//!
//! Its own test binary (the auth flag and policy are cached in process-wide `OnceLock`s). A caller who
//! holds only the `partner` role reads the frozen packs and nothing else: not the live status, not the
//! capacity view, not the workers, and nothing that writes. Everyone reads the same payload, and it
//! holds no worker, no worker number and no name.

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
        json!({ "effect": "allow", "actions": ["write"], "when": { "decider": ["true"] } }),
    );
    policy.to_string()
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test --test enforcement_oversight_pack -- --ignored`"]
async fn a_partner_reads_the_same_pack_as_the_board_and_nothing_else() {
    use sea_orm::ConnectionTrait;
    let (keys, kid) = keys_and_kid();
    unsafe {
        std::env::set_var("WPM_REQUIRE_AUTH", "1");
        std::env::set_var("WPM_PASETO_KEYS", keys.to_string());
        std::env::remove_var("WPM_ABAC_POLICY_FILE");
        std::env::set_var("WPM_ABAC_POLICY", policy_with_deciders());
    }
    let (partner_p, member_p, both_p, elsewhere_p, admin_p) = (
        uuid::Uuid::new_v4(),
        uuid::Uuid::new_v4(),
        uuid::Uuid::new_v4(),
        uuid::Uuid::new_v4(),
        uuid::Uuid::new_v4(),
    );
    let bearer = |t: String| format!("Bearer {t}");
    let partner = bearer(sign_as(&kid, &partner_p.to_string(), &[]));
    let member = bearer(sign_as(
        &kid,
        &member_p.to_string(),
        &[("decider", &["true"])],
    ));
    let both = bearer(sign_as(
        &kid,
        &both_p.to_string(),
        &[("decider", &["true"])],
    ));
    let elsewhere = bearer(sign_as(&kid, &elsewhere_p.to_string(), &[]));
    let admin = bearer(sign_as(
        &kid,
        &admin_p.to_string(),
        &[("decider", &["true"])],
    ));
    let svc = bearer(sign_as(&kid, "svc-user", &[("svc", &["true"])]));

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
        grant(admin_p.to_string(), &org, "hr_admin").await;
        grant(partner_p.to_string(), &org, "partner").await;
        grant(member_p.to_string(), &org, "member").await;
        grant(both_p.to_string(), &org, "partner").await;
        grant(both_p.to_string(), &org, "member").await;
        grant(elsewhere_p.to_string(), &other_org, "partner").await;
        let get = |path: String, token: String| { let request = &request; async move { request.get(&path).add_header("authorization", token).await } };
        let post = |path: String, token: String, body: Value| { let request = &request; async move { request.post(&path).add_header("authorization", token).json(&body).await } };
        // A named worker and a pool exist, so there is something a leak could carry.
        let w = post("/api/workers".into(), svc.clone(), json!({
            "person_ref": format!("person:{}", uuid::Uuid::new_v4()), "organization_ref": org, "worker_number": "PK-SECRET-1",
            "display_name": "Synthetic Named Worker", "employment_type": "permanent", "department": "engineering",
            "job_title": "Engineer", "hired_on": "2023-01-05" })).await;
        assert_eq!(w.status_code(), 200);
        post("/api/skill-pools".into(), svc.clone(), json!({ "organization_ref": org, "name": "Synthetic pool", "operations_reservation_bp": 0 })).await.assert_status_ok();

        let level = post("/api/oversight/levels".into(), svc.clone(), json!({ "organization_ref": org, "ordinal": 1, "name": "Synthetic level" })).await.json::<Value>();
        let recorded = post("/api/oversight/positions".into(), svc.clone(), json!({ "organization_ref": org, "level_pid": level["pid"], "effective_on": "2026-01-01", "deciding_body": "Synthetic Deciding Body", "reason": "Synthetic published reason" })).await;
        assert_eq!(recorded.status_code(), 200);
        // Freezing: signed in, in scope, and not by a partner.
        let freeze = json!({ "organization_ref": org });
        assert_eq!(request.post("/api/oversight/packs").json(&freeze).await.status_code(), 401, "sign in");
        assert_eq!(post("/api/oversight/packs".into(), partner.clone(), freeze.clone()).await.status_code(), 403, "a partner cannot freeze");
        assert_eq!(post("/api/oversight/packs".into(), member.clone(), freeze.clone()).await.status_code(), 403, "nor can an ordinary member");
        assert_eq!(post("/api/oversight/packs".into(), both.clone(), freeze.clone()).await.status_code(), 403, "a partner who is also a member is still not an administrator");
        let by_admin = post("/api/oversight/packs".into(), admin.clone(), freeze.clone()).await;
        assert_eq!(by_admin.status_code(), 200, "an administrator of the organization can");
        let admin_pack = by_admin.json::<Value>()["pid"].as_str().unwrap().to_string();
        let made = post("/api/oversight/packs".into(), svc.clone(), freeze.clone()).await;
        assert_eq!(made.status_code(), 200);
        let pack_pid = made.json::<Value>()["pid"].as_str().unwrap().to_string();
        let one = format!("/api/oversight/packs/{pack_pid}");
        let list = format!("/api/oversight/packs?organization={org}");

        // The partner reads the list and the pack; the board's member reads the same payload.
        let p_list = get(list.clone(), partner.clone()).await;
        assert_eq!(p_list.status_code(), 200);
        assert_eq!(p_list.json::<Value>()["packs"].as_array().unwrap().len(), 2);
        let p_pack = get(one.clone(), partner.clone()).await;
        assert_eq!(p_pack.status_code(), 200);
        let p_json = p_pack.json::<Value>();
        let m_json = get(one.clone(), member.clone()).await.json::<Value>();
        assert_eq!(p_json, m_json, "one payload for every audience");
        assert_eq!(get(one.clone(), svc.clone()).await.json::<Value>(), m_json);
        let text = serde_json::to_string(&p_json).unwrap();
        for leak in ["PK-SECRET-1", "Synthetic Named Worker", "worker:", "person:", "email", "deciding_body", "Synthetic Deciding Body"] {
            assert!(!text.contains(leak), "the pack must not carry `{leak}`: {text}");
        }
        assert!(text.contains("Synthetic pool"), "pools are named; people are not");
        assert!(text.contains("Synthetic published reason"), "the published reason stays");

        // The pack carries the key-person and governance figures, as counts and totals only.
        assert!(p_json["pack"]["key_person"]["single_points_of_failure"].is_number());
        assert!(p_json["pack"]["governance"]["all_hours_per_month"].is_number());
        assert!(p_json["pack"]["governance"].get("roles").is_none());
        // A partner can find the organizations in their scope, to know which packs to ask for.
        assert_eq!(get("/api/me/organizations/scope".into(), partner.clone()).await.status_code(), 200);
        assert_eq!(get("/api/me".into(), partner.clone()).await.status_code(), 403);
        // Withdrawing: an administrator, with a reason, once; the payload is unchanged and everyone sees the mark.
        let withdraw = format!("/api/oversight/packs/{admin_pack}/withdraw");
        assert_eq!(post(withdraw.clone(), partner.clone(), json!({ "reason": "x" })).await.status_code(), 403);
        assert_eq!(post(withdraw.clone(), member.clone(), json!({ "reason": "x" })).await.status_code(), 403);
        assert_eq!(post(withdraw.clone(), admin.clone(), json!({ "reason": "  " })).await.status_code(), 422);
        let before = get(format!("/api/oversight/packs/{admin_pack}"), member.clone()).await.json::<Value>();
        assert_eq!(post(withdraw.clone(), admin.clone(), json!({ "reason": "Frozen before the month closed" })).await.status_code(), 200);
        assert_eq!(post(withdraw.clone(), admin.clone(), json!({ "reason": "again" })).await.status_code(), 422, "once");
        let after_p = get(format!("/api/oversight/packs/{admin_pack}"), partner.clone()).await.json::<Value>();
        let after_m = get(format!("/api/oversight/packs/{admin_pack}"), member.clone()).await.json::<Value>();
        assert_eq!(after_p, after_m);
        assert_eq!(after_p["withdrawn"]["reason"], "Frozen before the month closed");
        assert_eq!(after_p["pack"], before["pack"], "the frozen payload is untouched");
        assert!(before["withdrawn"].is_null());
        let listed = get(list.clone(), partner.clone()).await.json::<Value>();
        assert_eq!(listed["packs"].as_array().unwrap().iter().filter(|p| !p["withdrawn"].is_null()).count(), 1);
        // A partner reads nothing else and writes nothing.
        for path in [
            format!("/api/oversight/status?organization={org}"),
            format!("/api/capacity?organization={org}"),
            "/api/workers".to_string(),
            format!("/api/skill-pools?organization={org}"),
            format!("/api/early-warnings?organization={org}"),
            "/api/oversight/derivations".to_string(),
        ] {
            assert_eq!(get(path.clone(), partner.clone()).await.status_code(), 403, "partner reading {path}");
            assert_ne!(get(path.clone(), member.clone()).await.status_code(), 403, "a member still reads {path}");
        }
        for (path, body) in [
            ("/api/oversight/levels", json!({ "organization_ref": org, "ordinal": 1, "name": "x" })),
            ("/api/early-warnings/sweep?organization=x", json!({})),
        ] {
            assert_eq!(post(path.into(), partner.clone(), body).await.status_code(), 403, "partner writing {path}");
        }
        // A partner who is also a member is not restricted: the rule is for a caller who is only a partner.
        assert_eq!(get(format!("/api/oversight/status?organization={org}"), both.clone()).await.status_code(), 200);
        // A partner of another organization does not see this organization's pack.
        assert_eq!(get(one.clone(), elsewhere.clone()).await.status_code(), 404);
        assert_eq!(get(list.clone(), elsewhere.clone()).await.status_code(), 404);
        // A frozen pack is never changed or removed, not even by direct SQL.
        assert!(ctx.db.execute_unprepared(&format!("UPDATE oversight_packs SET month = month WHERE pid = '{pack_pid}'")).await.is_err());
        assert!(ctx.db.execute_unprepared(&format!("DELETE FROM oversight_packs WHERE pid = '{pack_pid}'")).await.is_err());
    })
    .await;
}
