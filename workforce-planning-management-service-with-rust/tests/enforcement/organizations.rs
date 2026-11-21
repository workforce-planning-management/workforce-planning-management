//! Organization memberships and confederations under enforcement: who may grant which role in which organization, who may
//! declare or end a confederation edge, and the reads that need a signed-in caller.

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

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (`DATABASE_URL`); run with `cargo test --test enforcement -- --ignored`"]
#[allow(clippy::too_many_lines)] // one matrix read top to bottom
async fn roles_and_confederation_edges_are_managed_only_by_who_may() {
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
    let (hr_p, owner_p, plain_p, admin_p) = (
        uuid::Uuid::new_v4(),
        uuid::Uuid::new_v4(),
        uuid::Uuid::new_v4(),
        uuid::Uuid::new_v4(),
    );
    let hr = bearer(sign_as(&kid, &hr_p.to_string(), &[("hr", &["true"])]));
    let owner = bearer(sign_as(&kid, &owner_p.to_string(), &[("hr", &["true"])]));
    let plain_hr = bearer(sign_as(&kid, &plain_p.to_string(), &[("hr", &["true"])]));
    let admin = bearer(sign_as(
        &kid,
        &admin_p.to_string(),
        &[("access", &["admin"])],
    ));

    request::<App, _, _>(|request, ctx| async move {
        let org = format!("organization:{}", uuid::Uuid::new_v4());
        let child = format!("organization:{}", uuid::Uuid::new_v4());
        member(&ctx, &hr_p.to_string(), &org, "hr_admin").await;
        member(&ctx, &owner_p.to_string(), &org, "org_admin").await;
        member(&ctx, &plain_p.to_string(), &org, "member").await;
        let post = |path: &'static str, token: String, body: Value| {
            let request = &request;
            async move { request.post(path).add_header("authorization", token).json(&body).await }
        };
        let grant = |role: &str| json!({ "person_ref": format!("person:{}", uuid::Uuid::new_v4()), "organization_ref": org, "role": role, "starts_on": "2026-01-01" });

        // Memberships: an HR admin grants a member, not an organization admin; an organization admin grants an HR admin;
        // a plain member grants nothing; an administrator grants anything; dates in the wrong order are refused.
        assert_eq!(post("/api/organization-memberships", hr.clone(), grant("member")).await.status_code(), 200);
        assert_eq!(post("/api/organization-memberships", hr.clone(), grant("org_admin")).await.status_code(), 403);
        assert_eq!(post("/api/organization-memberships", owner.clone(), grant("hr_admin")).await.status_code(), 200);
        assert_eq!(post("/api/organization-memberships", plain_hr.clone(), grant("member")).await.status_code(), 403);
        let by_admin = post("/api/organization-memberships", admin.clone(), grant("resource_manager")).await;
        assert_eq!(by_admin.status_code(), 200, "{}", by_admin.text());
        let mut reversed = grant("member");
        reversed["ends_on"] = json!("2025-01-01");
        assert_eq!(post("/api/organization-memberships", admin.clone(), reversed).await.status_code(), 422);
        // Revoking: an HR admin may not revoke the HR admin the organization admin granted.
        let granted: Value = post("/api/organization-memberships", owner.clone(), grant("hr_admin")).await.json();
        let pid = granted["pid"].as_str().unwrap().to_string();
        let revoke = |token: String| {
            let (request, pid) = (&request, pid.clone());
            async move { request.delete(&format!("/api/organization-memberships/{pid}")).add_header("authorization", token).await }
        };
        // The reference policy gives `delete` to the service and administrators only, so an HR token is refused before the
        // handler's own rule, even for an organization admin. A deployer who wants organization admins to revoke adds a rule.
        assert_eq!(revoke(hr.clone()).await.status_code(), 403);
        assert_eq!(revoke(owner.clone()).await.status_code(), 403, "an HR token may not delete under the reference policy");
        assert_eq!(revoke(admin.clone()).await.status_code(), 200);

        // A signed-in caller lists their own organizations.
        let mine = request.get("/api/me/organizations").add_header("authorization", hr.clone()).await;
        assert_eq!(mine.status_code(), 200);
        assert!(mine.text().contains(&org));

        // Confederations: the parent's organization admin or an administrator may declare an edge; an HR admin may not.
        let edge = json!({ "parent_organization_ref": org, "child_organization_ref": child, "starts_on": "2026-01-01" });
        assert_eq!(post("/api/organization-confederations", hr.clone(), edge.clone()).await.status_code(), 403);
        let mut bad = edge.clone();
        bad["ends_on"] = json!("2025-01-01");
        assert_eq!(post("/api/organization-confederations", owner.clone(), bad).await.status_code(), 422);
        let made = post("/api/organization-confederations", owner.clone(), edge.clone()).await;
        assert_eq!(made.status_code(), 200, "{}", made.text());
        let edge_pid = made.json::<Value>()["pid"].as_str().unwrap().to_string();
        let listed = request
            .get(&format!("/api/organization-confederations?child_organization_ref={child}"))
            .add_header("authorization", admin.clone())
            .await;
        assert_eq!(listed.status_code(), 200);
        assert!(listed.text().contains(&edge_pid));
        let end = |token: String| {
            let (request, edge_pid) = (&request, edge_pid.clone());
            async move { request.delete(&format!("/api/organization-confederations/{edge_pid}")).add_header("authorization", token).await }
        };
        assert_eq!(end(hr.clone()).await.status_code(), 403);
        assert_eq!(end(admin.clone()).await.status_code(), 200);
    })
    .await;
}
