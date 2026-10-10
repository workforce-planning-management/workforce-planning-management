//! Sign in with Microsoft Entra ID, end to end (WPM-R93, WPM-D62).
//!
//! An in-process, **Entra-compatible OIDC provider** serves what a real tenant does:
//! a discovery document, the tenant's signing keys, an authorization endpoint and a
//! token endpoint that checks PKCE. The test performs the **authorization-code
//! exchange with PKCE** as a client would, receives an RS256 v2 access token carrying
//! Entra's claims (`oid`, `tid`, `ver`, a top-level `roles`), points the service at
//! the provider exactly as `spec/operations/entra-sign-in.md` says, boots it, and calls
//! the API with the token it received.
//!
//! What this proves: discovery and key fetching, the issuer / audience / signature /
//! expiry checks, the app-role to persona mapping, the `oid` subject, and that sign-in is
//! enforced. What it does **not** prove: the interactive browser redirect and consent,
//! which belong to the identity provider and the broker (the authentication service or
//! Keycloak), not to this repository (WPM-D62); and real Entra specifics beyond the
//! documented token shape.
//!
//! Its own test binary: the auth `OnceLock`s are process-wide. Built only with
//! `--no-default-features --features keycloak`; `#[ignore]`d because it needs PostgreSQL
//! (`config/test.yaml`). It needs no container runtime.
//!
//! `cargo test --no-default-features --features keycloak --test oidc_entra -- --ignored`

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::extract::{Form, Path, Query, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Redirect};
use axum::routing::{get, post};
use axum::{Json, Router};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use loco_rs::testing::prelude::*;
use sea_orm::ConnectionTrait;
use serde_json::{Value, json};
use serial_test::serial;
use sha2::{Digest, Sha256};
use workforce_planning_management_service::app::App;

const TENANT: &str = "99999999-9999-4999-8999-999999999999";
const CLIENT_ID: &str = "11111111-2222-4333-8444-555555555555";
const REDIRECT: &str = "http://localhost:5173/signin/callback";
const KID: &str = "entra-test-key-1";
const REFERENCE_POLICY_FILE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/config/abac-policy.reference.json"
);
const SIGNING_KEY: &str = include_str!("fixtures/oidc/test-signing-key.pem");
const OTHER_KEY: &str = include_str!("fixtures/oidc/other-signing-key.pem");
/// The public modulus of `test-signing-key.pem` (base64url); the exponent is 65537.
const SIGNING_KEY_N: &str = "7PinPxL11AlNoGZwU6he0b9HoQ5aZAM3ENP2PkuCk0wgYhPispUH9dyGxn67mI43PG151SmS2nm_XtRBhhcaeVwMI9iznYaP3ciUm6qU5AtQ1dNH5AQQUYRbNMWj1Js2nAA9RSRQAE5g9EKNYy7P6rUbX_ajkY5Qtt2wmHvDOJoJKT7Lul3pqZHvrMQYufT9tT-qeUSjc0Dg_bAsoa3AYKAJwx2m7eTSHKPVCuO8G-ltx7HLSBzhCd8i5PGxwGJ_vKhrgUHWb24pPWesUSv2HvaGfep0bfEVv9WIKB4vMo3BRJz4B-Uu04nxoX5v_DR7Z7L1jFXcqJh7rMi0X3GWxQ";

/// The signing key from its PEM text: the PKCS#1 DER inside the armor. (The PEM
/// helpers of the JWT crate are not enabled in this build.)
fn rsa_encoding_key(pem: &str) -> EncodingKey {
    let body: String = pem.lines().filter(|l| !l.starts_with("-----")).collect();
    let der = base64::engine::general_purpose::STANDARD
        .decode(body)
        .expect("PEM body is base64");
    EncodingKey::from_rsa_der(&der)
}

/// A user the mock tenant can sign in.
struct User {
    login: &'static str,
    oid: &'static str,
    name: &'static str,
    roles: &'static [&'static str],
}

const USERS: [User; 2] = [
    User {
        login: "hr@example.test",
        oid: "aaaaaaaa-bbbb-4ccc-8ddd-000000000001",
        name: "Hannah Resources",
        roles: &["wpm-hr"],
    },
    User {
        login: "plain@example.test",
        oid: "aaaaaaaa-bbbb-4ccc-8ddd-000000000002",
        name: "Pat Plain",
        roles: &[],
    },
];

/// What the provider remembers about an issued code: the user's login, the PKCE
/// challenge, and the redirect uri it was issued for.
type IssuedCode = (String, String, String);

#[derive(Clone)]
struct Provider {
    base: String,
    /// code -> (user login, PKCE challenge, redirect uri)
    codes: Arc<Mutex<HashMap<String, IssuedCode>>>,
}

impl Provider {
    fn issuer(&self) -> String {
        format!("{}/{TENANT}/v2.0", self.base)
    }

    /// A v2 access token, signed with `key`, shaped like Entra's.
    fn token(&self, user: &User, key: &str, bend: impl FnOnce(&mut Value)) -> String {
        let now = chrono::Utc::now().timestamp();
        let mut claims = json!({
            "iss": self.issuer(), "aud": CLIENT_ID, "tid": TENANT, "ver": "2.0",
            "sub": format!("pairwise-{}", user.oid), "oid": user.oid,
            "name": user.name, "preferred_username": user.login,
            "roles": user.roles, "azp": CLIENT_ID,
            "iat": now, "nbf": now - 5, "exp": now + 3600,
        });
        bend(&mut claims);
        let mut header = Header::new(Algorithm::RS256);
        header.kid = Some(KID.to_string());
        jsonwebtoken::encode(&header, &claims, &rsa_encoding_key(key)).unwrap()
    }
}

async fn discovery(State(p): State<Provider>, Path(tenant): Path<String>) -> Json<Value> {
    assert_eq!(tenant, TENANT);
    Json(json!({
        "issuer": p.issuer(),
        "authorization_endpoint": format!("{}/{TENANT}/oauth2/v2.0/authorize", p.base),
        "token_endpoint": format!("{}/{TENANT}/oauth2/v2.0/token", p.base),
        "jwks_uri": format!("{}/{TENANT}/discovery/v2.0/keys", p.base),
        "response_types_supported": ["code", "id_token"],
        "id_token_signing_alg_values_supported": ["RS256"],
        "code_challenge_methods_supported": ["S256"],
    }))
}

async fn keys() -> Json<Value> {
    Json(json!({ "keys": [{
        "kty": "RSA", "use": "sig", "alg": "RS256", "kid": KID,
        "n": SIGNING_KEY_N, "e": "AQAB",
    }]}))
}

/// The user has "signed in" as `login_hint`: redirect back with a one-time code.
async fn authorize(
    State(p): State<Provider>,
    Query(q): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    assert_eq!(q["client_id"], CLIENT_ID);
    assert_eq!(q["response_type"], "code");
    assert_eq!(q["code_challenge_method"], "S256");
    let code = format!("code-{}", uuid::Uuid::new_v4());
    p.codes.lock().unwrap().insert(
        code.clone(),
        (
            q["login_hint"].clone(),
            q["code_challenge"].clone(),
            q["redirect_uri"].clone(),
        ),
    );
    Redirect::to(&format!(
        "{}?code={code}&state={}",
        q["redirect_uri"], q["state"]
    ))
}

/// Redeem a code: checks the client, the redirect uri and PKCE, then issues the token.
async fn token_endpoint(
    State(p): State<Provider>,
    Form(f): Form<HashMap<String, String>>,
) -> impl IntoResponse {
    let bad = |why: &str| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "invalid_grant", "error_description": why })),
        )
            .into_response()
    };
    if f.get("grant_type").map(String::as_str) != Some("authorization_code")
        || f.get("client_id").map(String::as_str) != Some(CLIENT_ID)
    {
        return bad("grant_type or client_id");
    }
    let Some((login, challenge, redirect)) = p
        .codes
        .lock()
        .unwrap()
        .remove(f.get("code").map_or("", String::as_str))
    else {
        return bad("unknown or used code");
    };
    if f.get("redirect_uri") != Some(&redirect) {
        return bad("redirect_uri mismatch");
    }
    let verifier = f.get("code_verifier").map_or("", String::as_str);
    if URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes())) != challenge {
        return bad("PKCE verifier does not match");
    }
    let user = USERS.iter().find(|u| u.login == login).unwrap();
    let access_token = p.token(user, SIGNING_KEY, |_| {});
    (
        [(header::CACHE_CONTROL, "no-store")],
        Json(json!({ "token_type": "Bearer", "expires_in": 3599, "access_token": access_token })),
    )
        .into_response()
}

async fn start_provider() -> Provider {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let provider = Provider {
        base: format!("http://{}", listener.local_addr().unwrap()),
        codes: Arc::default(),
    };
    let app = Router::new()
        .route(
            "/{tenant}/v2.0/.well-known/openid-configuration",
            get(discovery),
        )
        .route("/{tenant}/discovery/v2.0/keys", get(keys))
        .route("/{tenant}/oauth2/v2.0/authorize", get(authorize))
        .route("/{tenant}/oauth2/v2.0/token", post(token_endpoint))
        .with_state(provider.clone());
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    provider
}

/// Sign in as `login` the way a client does: authorization request with PKCE, then
/// redeem the code. Returns the access token.
async fn sign_in(provider: &Provider, discovery: &Value, login: &str) -> String {
    let http = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap();
    let verifier = format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    );
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    let state = uuid::Uuid::new_v4().to_string();
    let response = http
        .get(discovery["authorization_endpoint"].as_str().unwrap())
        .query(&[
            ("client_id", CLIENT_ID),
            ("response_type", "code"),
            ("redirect_uri", REDIRECT),
            ("scope", "api://wpm/.default openid"),
            ("state", &state),
            ("code_challenge", &challenge),
            ("code_challenge_method", "S256"),
            ("login_hint", login),
        ])
        .send()
        .await
        .unwrap();
    assert!(response.status().is_redirection(), "{}", response.status());
    let location = response.headers()[header::LOCATION]
        .to_str()
        .unwrap()
        .to_string();
    assert!(location.starts_with(REDIRECT), "{location}");
    let query: HashMap<String, String> = url_pairs(location.split_once('?').unwrap().1);
    assert_eq!(query["state"], state, "the state round-trips");
    let token: Value = http
        .post(discovery["token_endpoint"].as_str().unwrap())
        .form(&[
            ("grant_type", "authorization_code"),
            ("client_id", CLIENT_ID),
            ("code", &query["code"]),
            ("redirect_uri", REDIRECT),
            ("code_verifier", &verifier),
        ])
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    // A second redemption of the same code is refused.
    let again = http
        .post(discovery["token_endpoint"].as_str().unwrap())
        .form(&[
            ("grant_type", "authorization_code"),
            ("client_id", CLIENT_ID),
            ("code", &query["code"]),
            ("redirect_uri", REDIRECT),
            ("code_verifier", &verifier),
        ])
        .send()
        .await
        .unwrap();
    assert_eq!(again.status(), 400, "a code is single-use");
    let _ = provider;
    token["access_token"].as_str().unwrap().to_string()
}

fn url_pairs(query: &str) -> HashMap<String, String> {
    query
        .split('&')
        .filter_map(|pair| pair.split_once('='))
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test --no-default-features --features keycloak --test oidc_entra -- --ignored`"]
async fn entra_sign_in_gates_and_authorises() {
    let provider = start_provider().await;

    // ── Discovery, as an operator would configure the service from it.
    let discovery: Value = reqwest::get(format!(
        "{}/{TENANT}/v2.0/.well-known/openid-configuration",
        provider.base
    ))
    .await
    .unwrap()
    .json()
    .await
    .unwrap();
    assert_eq!(discovery["issuer"], provider.issuer());

    // ── Sign in as two users with the authorization-code exchange.
    let hr = sign_in(&provider, &discovery, "hr@example.test").await;
    let plain = sign_in(&provider, &discovery, "plain@example.test").await;
    // The token has the shape the documentation promises.
    let payload: Value = serde_json::from_slice(
        &URL_SAFE_NO_PAD
            .decode(hr.split('.').nth(1).unwrap())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(payload["ver"], "2.0");
    assert_eq!(payload["roles"], json!(["wpm-hr"]));
    assert_eq!(payload["oid"], USERS[0].oid);

    // ── Configure the service exactly as the runbook says; sign-in is NOT switched off.
    // `set_var` is `unsafe` in edition 2024; single-threaded setup before boot.
    unsafe {
        std::env::remove_var("WPM_REQUIRE_AUTH");
        std::env::set_var("WPM_KEYCLOAK_ISSUER", discovery["issuer"].as_str().unwrap());
        std::env::set_var("WPM_KEYCLOAK_AUDIENCE", CLIENT_ID);
        std::env::set_var(
            "WPM_KEYCLOAK_JWKS_URL",
            discovery["jwks_uri"].as_str().unwrap(),
        );
        std::env::set_var("WPM_OIDC_SUBJECT_CLAIM", "oid");
        std::env::set_var("WPM_ABAC_POLICY_FILE", REFERENCE_POLICY_FILE);
        std::env::set_var("WPM_RATE_LIMIT_PER_MINUTE", "0");
        std::env::set_var("WPM_RATE_LIMIT_SENSITIVE_PER_MINUTE", "0");
    }

    let wrong_audience =
        provider.token(&USERS[0], SIGNING_KEY, |c| c["aud"] = json!("someone-else"));
    let wrong_tenant = provider.token(&USERS[0], SIGNING_KEY, |c| {
        c["iss"] = json!(format!("{}/another-tenant/v2.0", provider.base));
    });
    let expired = provider.token(&USERS[0], SIGNING_KEY, |c| {
        let past = chrono::Utc::now().timestamp() - 7200;
        c["iat"] = json!(past - 60);
        c["nbf"] = json!(past - 60);
        c["exp"] = json!(past);
    });
    let forged = provider.token(&USERS[0], OTHER_KEY, |_| {});
    // Algorithm confusion: an HS256 token whose "secret" is public material.
    let hs256 = {
        let mut header = Header::new(Algorithm::HS256);
        header.kid = Some(KID.to_string());
        jsonwebtoken::encode(
            &header,
            &json!({ "iss": provider.issuer(), "aud": CLIENT_ID, "oid": USERS[0].oid,
                     "roles": ["wpm-hr"], "iat": 1, "exp": 4_000_000_000_i64 }),
            &EncodingKey::from_secret(SIGNING_KEY_N.as_bytes()),
        )
        .unwrap()
    };

    request::<App, _, _>(|request, ctx| async move {
        let bearer = |token: &str| format!("Bearer {token}");
        let status = |token: Option<&str>, path: &'static str| {
            let request = &request;
            let token = token.map(bearer);
            async move {
                let mut call = request.get(path);
                if let Some(token) = token {
                    call = call.add_header("authorization", token);
                }
                call.await.status_code()
            }
        };

        // Sign-in is enforced by default: no token is a 401, and the posture says so.
        assert_eq!(
            request.get("/_posture").await.json::<Value>()["auth_enforced"],
            true
        );
        assert_eq!(status(None, "/api/workers").await, 401);

        // A user with no app role: an ordinary authenticated reader, never a writer.
        assert_eq!(status(Some(&plain), "/api/workers").await, 200);
        assert_eq!(
            request
                .post("/api/workers")
                .add_header("authorization", bearer(&plain))
                .json(&json!({ "person_ref": "person:x" }))
                .await
                .status_code(),
            403,
            "no app role, no write"
        );

        // The `wpm-hr` app role maps to the HR persona: the write passes the policy gate
        // (the incomplete body is then judged on its own merits, never 401 or 403).
        let code = request
            .post("/api/workers")
            .add_header("authorization", bearer(&hr))
            .json(&json!({ "person_ref": "person:x" }))
            .await
            .status_code();
        assert_ne!(code, 401, "a valid Entra token authenticates");
        assert_ne!(code, 403, "the wpm-hr app role authorises the write");

        // Every kind of bad token is a 401.
        for (name, token) in [
            ("wrong audience", &wrong_audience),
            ("another tenant's issuer", &wrong_tenant),
            ("expired", &expired),
            ("signed by another key", &forged),
            ("HS256 algorithm confusion", &hs256),
        ] {
            assert_eq!(status(Some(token), "/api/workers").await, 401, "{name}");
        }
        assert_eq!(status(Some("not.a.jwt"), "/api/workers").await, 401);

        // The audit actor is the tenant-wide object id (`oid`), not the per-application `sub`.
        let actor = ctx
            .db
            .query_one_raw(sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                "SELECT actor FROM audit_logs WHERE actor IS NOT NULL ORDER BY id DESC LIMIT 1"
                    .to_string(),
            ))
            .await
            .unwrap()
            .and_then(|row| row.try_get::<String>("", "actor").ok());
        if let Some(actor) = actor {
            assert!(
                !actor.starts_with("pairwise-"),
                "the audit names the oid, got {actor}"
            );
        }
    })
    .await;
}
