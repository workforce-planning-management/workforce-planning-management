//! Aspirations under enforcement: someone in the person's management chain sees what was shared with managers or with
//! everyone; anyone else sees only what was shared with everyone; a private one is seen by no one but the person.

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
const REFERENCE_POLICY_FILE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/config/abac-policy.reference.json"
);

fn keys_and_kid() -> (Value, String) {
    let public = SigningKey::from_bytes(&SEED).verifying_key().to_bytes();
    let kid = URL_SAFE_NO_PAD.encode(Sha256::digest(public));
    let keys = json!({
        "keys": [{ "kty": "OKP", "crv": "Ed25519", "use": "sig",
                   "kid": kid, "x": URL_SAFE_NO_PAD.encode(public) }]
    });
    (keys, kid)
}

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
        "sub": sub, "email": "person@example.com", "name": "Test Person",
        "iss": ISSUER, "aud": AUDIENCE, "exp": iat + 3_600, "iat": iat,
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

fn bearer(token: String) -> String {
    format!("Bearer {token}")
}

async fn member(ctx: &axa::AppContext, person: &str, org: &str, role: &str) {
    sqlx::raw_sql(sqlx::AssertSqlSafe(format!(
        "INSERT INTO organization_memberships (pid, person_ref, organization_ref, role, starts_on) \
             VALUES ('{}', 'person:{person}', '{org}', '{role}', '2023-01-05')",
        uuid::Uuid::new_v4()
    )))
    .execute(workforce_planning_management::db::pool(ctx))
    .await
    .unwrap();
}

async fn hire(
    request: &axa_test::TestServer,
    svc: &str,
    org: &str,
    person: uuid::Uuid,
    department: &str,
) -> String {
    let created: Value = request
        .post("/api/workers")
        .add_header("authorization", svc.to_string())
        .json(&json!({
            "person_ref": format!("person:{person}"), "organization_ref": org, "worker_number": person.simple().to_string(),
            "display_name": "Person", "employment_type": "permanent", "department": department,
            "job_title": "Engineer", "hired_on": "2023-01-05",
        }))
        .await
        .json();
    let pid = created["pid"].as_str().unwrap().to_string();
    request
        .post(&format!("/api/workers/{pid}/status"))
        .add_header("authorization", svc.to_string())
        .json(&json!({ "to": "active" }))
        .await
        .assert_status_ok();
    pid
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test --test enforcement -- --ignored`"]
async fn a_manager_sees_what_was_shared_with_managers_and_a_stranger_only_what_was_shared_with_everyone()
 {
    let (keys, kid) = keys_and_kid();
    unsafe {
        std::env::set_var("WORKFORCE_PLANNING_MANAGEMENT_REQUIRE_AUTH", "1");
        std::env::set_var(
            "WORKFORCE_PLANNING_MANAGEMENT_PASETO_KEYS",
            keys.to_string(),
        );
        std::env::set_var(
            "WORKFORCE_PLANNING_MANAGEMENT_ABAC_POLICY_FILE",
            REFERENCE_POLICY_FILE,
        );
    }
    let (report_p, boss_p, stranger_p) = (
        uuid::Uuid::new_v4(),
        uuid::Uuid::new_v4(),
        uuid::Uuid::new_v4(),
    );
    let boss = bearer(sign_as(&kid, &boss_p.to_string(), &[("hr", &["true"])]));
    let stranger = bearer(sign_as(&kid, &stranger_p.to_string(), &[("hr", &["true"])]));
    let svc = bearer(sign_as(&kid, "svc-user", &[("svc", &["true"])]));

    request::<App, _, _>(|request, ctx| async move {
        let org = format!("organization:{}", uuid::Uuid::new_v4());
        for person in [report_p, boss_p, stranger_p] {
            member(&ctx, &person.to_string(), &org, "hr_admin").await;
        }
        let report = hire(&request, &svc, &org, report_p, "engineering").await;
        let manager = hire(&request, &svc, &org, boss_p, "engineering").await;
        hire(&request, &svc, &org, stranger_p, "engineering").await;
        let set = request
            .put(&format!("/api/workers/{report}"))
            .add_header("authorization", svc.clone())
            .json(&json!({ "manager_pid": manager }))
            .await;
        assert_eq!(set.status_code(), 200, "{}", set.text());
        let skill: Value = request.post("/api/skills").add_header("authorization", svc.clone())
            .json(&json!({ "name": format!("Skill {}", uuid::Uuid::new_v4().simple()), "category": "technical" }))
            .await
            .json();
        let declared = request.put(&format!("/api/workers/{report}/skills")).add_header("authorization", svc.clone())
            .json(&json!({ "skill_pid": skill["pid"], "proficiency": 2 }))
            .await;
        assert_eq!(declared.status_code(), 200, "{}", declared.text());
        for (note, visibility) in [("Kept to myself", "private"), ("For my manager", "manager"), ("For all", "everyone")] {
            let made = request.post(&format!("/api/workers/{report}/aspirations")).add_header("authorization", svc.clone())
                .json(&json!({ "kind": "skill", "skill_pid": skill["pid"], "target_level": 4, "horizon": "within_1y",
                               "note": note, "visibility": visibility }))
                .await;
            assert_eq!(made.status_code(), 200, "{}", made.text());
        }
        let read = |token: String, path: String| {
            let request = &request;
            async move {
                let r = request.get(&path).add_header("authorization", token).await;
                assert_eq!(r.status_code(), 200, "{path}: {}", r.text());
                r.json::<Value>()
            }
        };
        let labels = |v: &Value| -> Vec<String> {
            v["aspirations"].as_array().unwrap().iter().map(|a| a["note"].as_str().unwrap().to_string())
                .collect::<std::collections::BTreeSet<_>>()
                .into_iter().collect()
        };

        let by_boss = read(boss.clone(), format!("/api/workers/{report}/aspirations")).await;
        assert_eq!(by_boss["viewer"], "manager");
        assert_eq!(labels(&by_boss), ["For all", "For my manager"]);
        let by_stranger = read(stranger.clone(), format!("/api/workers/{report}/aspirations")).await;
        assert_eq!(by_stranger["viewer"], "other");
        assert_eq!(labels(&by_stranger), ["For all"]);

        // The manager's own downline shows what was shared with managers; read by a stranger, only what was shared with
        // everyone. The private one, and how many there are, never shows.
        let downline = read(boss.clone(), format!("/api/workers/{manager}/downline-aspirations")).await.to_string();
        assert!(downline.contains("For my manager") && downline.contains("For all") && !downline.contains("Kept to myself"), "{downline}");
        let seen_by_other = read(stranger.clone(), format!("/api/workers/{manager}/downline-aspirations")).await.to_string();
        assert!(!seen_by_other.contains("For my manager") && seen_by_other.contains("For all"), "{seen_by_other}");
    })
    .await;
}
