//! Announcements under enforcement: only an editor (HR or organization admin of the organization) writes; a reader sees
//! their own department's posts and the organization-wide ones; a post that is not live is the editor's to see.

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
) {
    let created: Value = request
        .post("/api/workers")
        .add_header("authorization", svc.to_string())
        .json(&json!({
            "person_ref": format!("person:{person}"), "organization_ref": org, "worker_number": person.simple().to_string(),
            "display_name": "Reader", "employment_type": "permanent", "department": department,
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
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test --test enforcement -- --ignored`"]
#[allow(clippy::too_many_lines)] // one feed read top to bottom
async fn only_editors_write_and_readers_see_their_departments() {
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
    let (editor_p, hr_other_p, eng_p, sales_p) = (
        uuid::Uuid::new_v4(),
        uuid::Uuid::new_v4(),
        uuid::Uuid::new_v4(),
        uuid::Uuid::new_v4(),
    );
    let editor = bearer(sign_as(&kid, &editor_p.to_string(), &[("hr", &["true"])]));
    let not_editor = bearer(sign_as(&kid, &hr_other_p.to_string(), &[("hr", &["true"])]));
    let engineer = bearer(sign_as(&kid, &eng_p.to_string(), &[]));
    let seller = bearer(sign_as(&kid, &sales_p.to_string(), &[]));
    let svc = bearer(sign_as(&kid, "svc-user", &[("svc", &["true"])]));

    request::<App, _, _>(|request, ctx| async move {
        let org = format!("organization:{}", uuid::Uuid::new_v4());
        member(&ctx, &editor_p.to_string(), &org, "hr_admin").await;
        member(&ctx, &hr_other_p.to_string(), &org, "member").await;
        for (person, department) in [(eng_p, "engineering"), (sales_p, "sales")] {
            member(&ctx, &person.to_string(), &org, "member").await;
            hire(&request, &svc, &org, person, department).await;
        }
        let today = workforce_planning_management::dates::today();
        let post = |token: String, body: Value| {
            let request = &request;
            async move { request.post("/api/announcements").add_header("authorization", token).json(&body).await }
        };
        let item = |title: &str, department: Option<&str>, publish_on: String| {
            json!({ "organization_ref": org, "title": title, "body": "Details.", "department": department, "publish_on": publish_on })
        };

        // Only an editor of the organization writes.
        let refused = post(not_editor.clone(), item("Not mine", None, today.to_string())).await;
        assert_eq!(refused.status_code(), 403, "{}", refused.text());
        let eng_only: Value = post(editor.clone(), item("Engineering only", Some("engineering"), today.to_string())).await.json();
        let eng_only = eng_only["pid"].as_str().unwrap().to_string();
        post(editor.clone(), item("Everyone", None, today.to_string())).await.assert_status_ok();
        let later: Value = post(editor.clone(), item("Not yet", None, (today + jiff::Span::new().days(30)).to_string())).await.json();
        let later = later["pid"].as_str().unwrap().to_string();

        // A reader sees their own department's posts and the organization-wide ones, not another department's.
        let feed = |token: String| {
            let request = &request;
            async move { request.get("/api/announcements").add_header("authorization", token).await.text() }
        };
        let engineers = feed(engineer.clone()).await;
        assert!(engineers.contains("Engineering only") && engineers.contains("Everyone"), "{engineers}");
        let sellers = feed(seller.clone()).await;
        assert!(!sellers.contains("Engineering only") && sellers.contains("Everyone"), "{sellers}");

        // A post that is not live yet is hidden from a reader and shown to its editor.
        let one = |token: String, pid: String| {
            let request = &request;
            async move { request.get(&format!("/api/announcements/{pid}")).add_header("authorization", token).await.status_code() }
        };
        assert_eq!(one(engineer.clone(), later.clone()).await, 404);
        assert_eq!(one(editor.clone(), later.clone()).await, 200);
        assert_eq!(one(engineer.clone(), eng_only.clone()).await, 200);

        // Restating the department: a blank keeps it, a value changes it, and clearing makes it organization-wide.
        let put = |body: Value| {
            let (request, editor, eng_only) = (&request, editor.clone(), eng_only.clone());
            async move {
                let r = request.put(&format!("/api/announcements/{eng_only}")).add_header("authorization", editor).json(&body).await;
                assert_eq!(r.status_code(), 200, "{}", r.text());
                r.json::<Value>()
            }
        };
        assert_eq!(put(json!({ "department": " " })).await["department"], "engineering");
        assert_eq!(put(json!({ "department": "sales" })).await["department"], "sales");
        assert!(put(json!({ "clear_department": true })).await["department"].is_null());
    })
    .await;
}
