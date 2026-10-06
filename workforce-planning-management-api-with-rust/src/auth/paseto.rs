//! The `paseto` backend (default feature): offline PASETO `v4.public`
//! (Ed25519) verification against the sibling authentication
//! service's published key set. Everything downstream of a verified
//! [`Claims`] — the ABAC engine, masking, `AuthUser`/`MaybeAuthUser` —
//! lives in the parent [`super`] module and is shared with the
//! `keycloak` backend; this module owns only "how do we turn a bearer
//! header into `Claims`".
//!
//! ## Key source
//!
//! The process-wide [`verifier`] is seeded once at boot ([`init`] is
//! called from `App::after_routes`, before the app serves traffic) and
//! built from the environment:
//!
//! - `WPM_PASETO_KEYS_URL` — optional URL of the auth service's
//!   published key set (`/.well-known/paseto-keys`). Set (non-blank) ⇒
//!   the key set is fetched over HTTP **once at boot** via
//!   [`Verifier::from_paseto_keys_url`]; on success the fetched key set
//!   wins over `WPM_PASETO_KEYS` (`tracing::info!`), on failure the
//!   service logs a `tracing::warn!` and falls back to the env path
//!   below — the service always boots. The key set is then **re-fetched
//!   periodically** ([`spawn_key_refresh`]) so a key rotation is picked
//!   up without a restart (interval `WPM_PASETO_KEYS_REFRESH_SECS`,
//!   default 1 h; `0` disables; keeps the current keys on a failed
//!   fetch).
//! - `WPM_PASETO_KEYS` — the Ed25519 key set (JSON, OKP/Ed25519
//!   JWK form) the auth service publishes at `/.well-known/paseto-keys`.
//!   Absent ⇒ an empty key set, so every token is rejected (the service
//!   still boots).
//! - `WPM_TOKEN_ISSUER` — expected `iss` (default
//!   `authentication-service`).
//! - `WPM_TOKEN_AUDIENCE` — expected `aud` (default
//!   `main-x-service`).

use super::{
    Claims, ENTITY, Method, OnceLock, Policy, StatusCode, derive_action, env_or, is_public_path,
};
use authentication_verifier::{ReloadableVerifier, Verifier};
use axum::http::HeaderMap;
use axum::http::header::AUTHORIZATION;

/// Default issuer expected in tokens (`iss`).
const DEFAULT_ISSUER: &str = "authentication-service";
/// Default audience expected in tokens (`aud`).
const DEFAULT_AUDIENCE: &str = "main-x-service";

/// The process-wide **hot-reloadable** token verifier. Lazily built from
/// the environment on first use ([`build_from_env`]); [`init`] swaps in
/// the boot-time fetched key set, and [`spawn_key_refresh`] swaps in a
/// re-fetched key set periodically (**key rotation without a restart**).
static VERIFIER: OnceLock<ReloadableVerifier> = OnceLock::new();

/// The process-wide reloadable token verifier. Read the active snapshot
/// with `verifier().current()` per request; it is swapped by [`init`]
/// (boot fetch) and [`spawn_key_refresh`] (periodic re-fetch).
#[must_use]
pub fn verifier() -> &'static ReloadableVerifier {
    VERIFIER.get_or_init(|| ReloadableVerifier::new(build_from_env()))
}

/// Seed the process-wide [`verifier`] before the app serves traffic
/// (called from `App::after_routes`). When `WPM_PASETO_KEYS_URL` is set
/// (non-blank) the published key set is fetched over HTTP **once at boot**
/// and, on success, swapped in over the env-built one; on fetch failure,
/// or with the URL unset/blank, the env-built verifier stands, so the
/// service always boots. Idempotent enough to call once at boot.
pub async fn init() {
    if let Some(url) =
        crate::compat::env_var("WPM_PASETO_KEYS_URL").filter(|s| !s.trim().is_empty())
    {
        let issuer = env_or("WPM_TOKEN_ISSUER", DEFAULT_ISSUER);
        let audience = env_or("WPM_TOKEN_AUDIENCE", DEFAULT_AUDIENCE);
        // `fetch_or` keeps the env-built verifier as the fallback on a
        // failed fetch, so the service always has a usable verifier.
        let fetched = fetch_or(url.trim(), &issuer, &audience, build_from_env()).await;
        verifier().store(fetched);
    }
}

/// Default key-set refresh interval (seconds) when
/// `WPM_PASETO_KEYS_REFRESH_SECS` is unset. One hour — key rotation is
/// infrequent, so a slow poll suffices.
const KEY_REFRESH_DEFAULT_SECS: u64 = 3600;

/// Spawn a background task that periodically re-fetches the published
/// key set from `WPM_PASETO_KEYS_URL` and swaps it into the live
/// [`verifier`], so a **key rotation** at the auth-service is picked up
/// **without restarting** this service. On a failed fetch it keeps the
/// current key set (a transient auth-service outage never locks callers
/// out). Interval from `WPM_PASETO_KEYS_REFRESH_SECS` (default
/// [`KEY_REFRESH_DEFAULT_SECS`]); **`0` disables** the loop.
///
/// A no-op when `WPM_PASETO_KEYS_URL` is unset (env-supplied keys have
/// nothing to re-fetch). Call once at boot (`app.rs::after_routes`).
pub fn spawn_key_refresh() {
    let Some(url) = crate::compat::env_var("WPM_PASETO_KEYS_URL")
        .map(|u| u.trim().to_string())
        .filter(|u| !u.is_empty())
    else {
        return;
    };
    let secs = crate::compat::env_var("WPM_PASETO_KEYS_REFRESH_SECS")
        .and_then(|v| v.trim().parse::<u64>().ok())
        .unwrap_or(KEY_REFRESH_DEFAULT_SECS);
    if secs == 0 {
        return; // explicitly disabled
    }
    let issuer = env_or("WPM_TOKEN_ISSUER", DEFAULT_ISSUER);
    let audience = env_or("WPM_TOKEN_AUDIENCE", DEFAULT_AUDIENCE);
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(std::time::Duration::from_secs(secs));
        ticker.tick().await; // the first tick is immediate — skip it
        loop {
            ticker.tick().await;
            match Verifier::from_paseto_keys_url(&url, &issuer, &audience).await {
                Ok(fetched) => {
                    tracing::info!(keys = fetched.key_count(), "refreshed PASETO key set");
                    verifier().store(fetched);
                }
                Err(error) => {
                    tracing::warn!(%error, "PASETO key-set refresh failed; keeping current keys");
                }
            }
        }
    });
    tracing::info!(secs, "polling WPM_PASETO_KEYS_URL for key rotation");
}

/// Build a verifier by fetching the published key set from `url`
/// ([`Verifier::from_paseto_keys_url`]); on success the fetched key set
/// wins (`tracing::info!`), on any fetch/parse failure the given
/// `fallback` verifier is returned after a `tracing::warn!` — never a
/// panic, so the caller always boots. Pure dependency injection (URL,
/// issuer, audience and fallback are all passed in), so it is testable
/// against a local HTTP listener without touching the process global.
pub async fn fetch_or(url: &str, issuer: &str, audience: &str, fallback: Verifier) -> Verifier {
    match Verifier::from_paseto_keys_url(url, issuer, audience).await {
        Ok(fetched) => {
            tracing::info!(
                url,
                keys = fetched.key_count(),
                "PASETO key set fetched over HTTP; fetched key set wins over the env key set"
            );
            fetched
        }
        Err(error) => {
            tracing::warn!(
                url,
                %error,
                "PASETO key set fetch failed; falling back to the env-configured key set"
            );
            fallback
        }
    }
}

/// Build the process-wide [`Verifier`] from the environment: issuer,
/// audience, and the published key set. A missing/blank/unparseable key
/// set yields an empty key set (every token rejected) so the service still
/// boots without credentials configured.
fn build_from_env() -> Verifier {
    let issuer = env_or("WPM_TOKEN_ISSUER", DEFAULT_ISSUER);
    let audience = env_or("WPM_TOKEN_AUDIENCE", DEFAULT_AUDIENCE);
    let keys = crate::compat::env_var("WPM_PASETO_KEYS")
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
        .unwrap_or_else(|| serde_json::json!({ "keys": [] }));
    Verifier::from_paseto_keys_value(&keys, &issuer, &audience)
        .unwrap_or_else(|_| empty_verifier(&issuer, &audience))
}

/// A verifier with no keys: rejects every token until a real key set is
/// configured. Infallible — an empty `keys` array always parses.
fn empty_verifier(issuer: &str, audience: &str) -> Verifier {
    let empty = serde_json::json!({ "keys": [] });
    Verifier::from_paseto_keys_value(&empty, issuer, audience).expect("empty key set always builds")
}

/// Extract and verify the bearer token from request headers. Pure (the
/// verifier is passed in), so it is unit-testable without the global.
///
/// # Errors
///
/// `401` when the `Authorization` header is missing, is not a bearer
/// token, or the token fails PASETO signature / issuer / audience / expiry
/// verification.
pub fn bearer_claims(
    headers: &HeaderMap,
    verifier: &Verifier,
) -> Result<Claims, (StatusCode, String)> {
    let header = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or((
            StatusCode::UNAUTHORIZED,
            "missing authorization header".to_string(),
        ))?;
    let token = header
        .strip_prefix("Bearer ")
        .or_else(|| header.strip_prefix("bearer "))
        .ok_or((
            StatusCode::UNAUTHORIZED,
            "expected bearer token".to_string(),
        ))?;
    verifier
        .verify(token.trim())
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))
}

/// The blanket-enforcement decision: authentication, then ABAC
/// authorization. `Ok(())` ⇒ let the request through; `Err((401|403,
/// msg))` ⇒ reject. Pure: the caller passes the flag, method, path,
/// headers, verifier and policy, so it is fully unit-testable without
/// booting the app or a database.
///
/// # Errors
///
/// `401` when enforcement is on, the path is not public, and the request
/// carries no valid bearer token (missing/malformed/expired/tampered).
/// `403` when the token is valid but the ABAC policy denies the derived
/// action (the message names the deciding rule, per
/// `authorization-attributes.md` §5).
pub fn enforce(
    require_auth: bool,
    method: &Method,
    path: &str,
    headers: &HeaderMap,
    verifier: &Verifier,
    policy: &Policy,
) -> Result<(), (StatusCode, String)> {
    if !require_auth || is_public_path(path) {
        return Ok(());
    }
    let claims = bearer_claims(headers, verifier)?;
    let decision = policy.evaluate(&claims, derive_action(method, path), ENTITY);
    if decision.allowed {
        Ok(())
    } else {
        Err((StatusCode::FORBIDDEN, decision.reason))
    }
}

/// DB-free, fully in-process pins for token verification and the blanket
/// `enforce` decision. A throwaway Ed25519 key mints PASETO tokens and a
/// matching key set, so the whole verification path (valid / missing /
/// non-bearer / expired / tampered / empty-keys) and the on/off/public-path
/// enforcement matrix are exercised without the auth service or a database.
#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use ed25519_dalek::SigningKey;
    use rusty_paseto::core::{
        Footer, Key, Paseto, PasetoAsymmetricPrivateKey, Payload, Public, V4,
    };
    use sha2::{Digest, Sha256};

    /// Issuer the test tokens and verifier agree on.
    const ISSUER: &str = "authentication-service";
    /// Audience the test tokens and verifier agree on.
    const AUDIENCE: &str = "main-x-service";
    /// A throwaway Ed25519 seed, used only to mint test tokens and a
    /// matching key set in-process. Not a secret — never used in production.
    const SEED: [u8; 32] = [
        3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3,
        3, 3,
    ];

    /// Build a key set + matching `kid` from the test public key, the same
    /// way the auth service publishes it.
    fn test_keys_and_kid() -> (serde_json::Value, String) {
        let public = SigningKey::from_bytes(&SEED).verifying_key().to_bytes();
        let kid = URL_SAFE_NO_PAD.encode(Sha256::digest(public));
        let keys = serde_json::json!({
            "keys": [{
                "kty": "OKP", "crv": "Ed25519", "use": "sig",
                "kid": kid, "x": URL_SAFE_NO_PAD.encode(public),
            }]
        });
        (keys, kid)
    }

    /// Mint a signed PASETO `v4.public` token for the test identity with
    /// `kid` in the footer and `exp` set `exp_offset_secs` from a fixed
    /// `iat` (negative offsets produce an already-expired token).
    fn sign(kid: &str, exp_offset_secs: i64) -> String {
        sign_with_attrs(kid, exp_offset_secs, &[])
    }

    /// Like [`sign`], with the given ABAC subject attributes minted into
    /// the token's `attrs` claim (e.g. `&[("access", &["write"])]`).
    fn sign_with_attrs(kid: &str, exp_offset_secs: i64, attrs: &[(&str, &[&str])]) -> String {
        let iat: i64 = 1_700_000_000;
        let claims = Claims {
            sub: "11111111-1111-1111-1111-111111111111".into(),
            email: "alice@example.com".into(),
            name: "Alice".into(),
            iss: ISSUER.into(),
            aud: AUDIENCE.into(),
            exp: iat + exp_offset_secs,
            iat,
            nbf: None,
            sid: "test-sid".into(),
            scope: Vec::new(),
            roles: Vec::new(),
            attrs: attrs
                .iter()
                .map(|(key, values)| {
                    (
                        (*key).to_string(),
                        values.iter().map(ToString::to_string).collect(),
                    )
                })
                .collect(),
        };
        let keypair = SigningKey::from_bytes(&SEED).to_keypair_bytes();
        let key = Key::<64>::from(keypair);
        let private = PasetoAsymmetricPrivateKey::<V4, Public>::from(&key);
        let payload = serde_json::to_string(&claims).expect("serialize claims");
        let footer = format!(r#"{{"kid":"{kid}"}}"#);
        let mut builder = Paseto::<V4, Public>::builder();
        builder.set_payload(Payload::from(payload.as_str()));
        builder.set_footer(Footer::from(footer.as_str()));
        builder.try_sign(&private).expect("sign")
    }

    /// Wrap a token in a `HeaderMap` with `Authorization: Bearer <token>`.
    fn bearer(token: &str) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert(AUTHORIZATION, format!("Bearer {token}").parse().unwrap());
        h
    }

    /// A well-formed, in-date, correctly-signed token verifies and yields
    /// the expected claims.
    #[test]
    fn valid_token_yields_claims() {
        let (keys, kid) = test_keys_and_kid();
        let verifier = Verifier::from_paseto_keys_value(&keys, ISSUER, AUDIENCE).unwrap();
        let token = sign(&kid, 10_000_000_000);
        let claims = bearer_claims(&bearer(&token), &verifier).expect("valid token verifies");
        assert_eq!(claims.sub, "11111111-1111-1111-1111-111111111111");
        assert_eq!(claims.email, "alice@example.com");
    }

    /// No `Authorization` header ⇒ `401`.
    #[test]
    fn missing_header_is_401() {
        let (keys, _) = test_keys_and_kid();
        let verifier = Verifier::from_paseto_keys_value(&keys, ISSUER, AUDIENCE).unwrap();
        let err = bearer_claims(&HeaderMap::new(), &verifier).unwrap_err();
        assert_eq!(err.0, StatusCode::UNAUTHORIZED);
    }

    /// A non-bearer scheme (e.g. `Basic`) ⇒ `401`.
    #[test]
    fn non_bearer_header_is_401() {
        let (keys, _) = test_keys_and_kid();
        let verifier = Verifier::from_paseto_keys_value(&keys, ISSUER, AUDIENCE).unwrap();
        let mut h = HeaderMap::new();
        h.insert(AUTHORIZATION, "Basic abc123".parse().unwrap());
        assert_eq!(
            bearer_claims(&h, &verifier).unwrap_err().0,
            StatusCode::UNAUTHORIZED
        );
    }

    /// A token whose `exp` is in the past ⇒ `401`.
    #[test]
    fn expired_token_is_401() {
        let (keys, kid) = test_keys_and_kid();
        let verifier = Verifier::from_paseto_keys_value(&keys, ISSUER, AUDIENCE).unwrap();
        let token = sign(&kid, -60);
        assert_eq!(
            bearer_claims(&bearer(&token), &verifier).unwrap_err().0,
            StatusCode::UNAUTHORIZED
        );
    }

    /// Flipping a token character breaks PASETO verification ⇒ `401`.
    #[test]
    fn tampered_token_is_401() {
        let (keys, kid) = test_keys_and_kid();
        let verifier = Verifier::from_paseto_keys_value(&keys, ISSUER, AUDIENCE).unwrap();
        let mut token = sign(&kid, 10_000_000_000);
        let last = token.pop().unwrap();
        token.push(if last == 'a' { 'b' } else { 'a' });
        assert_eq!(
            bearer_claims(&bearer(&token), &verifier).unwrap_err().0,
            StatusCode::UNAUTHORIZED
        );
    }

    /// A no-key verifier (the boot fallback) rejects even a valid token.
    #[test]
    fn empty_verifier_rejects_everything() {
        let verifier = empty_verifier(ISSUER, AUDIENCE);
        let (_, kid) = test_keys_and_kid();
        let token = sign(&kid, 10_000_000_000);
        assert_eq!(
            bearer_claims(&bearer(&token), &verifier).unwrap_err().0,
            StatusCode::UNAUTHORIZED
        );
    }

    /// The default policy the enforcement tests share (what an
    /// unconfigured service uses).
    fn policy() -> Policy {
        Policy::default_policy()
    }

    /// Enforcement off ⇒ a protected path passes with no token — for
    /// reads and mutations alike (no authn and no authz when the flag
    /// is off; today's default behaviour stays intact).
    #[test]
    fn enforce_off_allows_without_token() {
        let (keys, _) = test_keys_and_kid();
        let verifier = Verifier::from_paseto_keys_value(&keys, ISSUER, AUDIENCE).unwrap();
        let policy = policy();
        for method in [Method::GET, Method::POST, Method::DELETE] {
            assert!(
                enforce(
                    false,
                    &method,
                    "/api/wards",
                    &HeaderMap::new(),
                    &verifier,
                    &policy
                )
                .is_ok(),
                "{method} should pass with enforcement off"
            );
        }
    }

    /// SEC-G8 — the default-off **exposure pin**. With `WPM_REQUIRE_AUTH`
    /// off (the shipped default), the most sensitive reads — the audit
    /// trail, patient locate, and a single stay's PII — are **open without
    /// a token**. This is by design
    /// (see `agents/share/security.md` §4), but it means **activation is a
    /// tracked release gate**: a deployment exposed to untrusted callers
    /// MUST set the flag before it is reachable. This test documents that
    /// exposure explicitly so flipping the default cannot happen silently.
    #[test]
    fn default_off_exposes_sensitive_reads_activation_is_a_release_gate() {
        let (keys, _) = test_keys_and_kid();
        let verifier = Verifier::from_paseto_keys_value(&keys, ISSUER, AUDIENCE).unwrap();
        let policy = policy();
        let no_token = HeaderMap::new();
        for path in [
            "/api/stays/0c4f1e2a-0000-4000-8000-000000000001", // a stay's PII
            "/api/locate/person:0c4f1e2a-0000-4000-8000-000000000001", // patient location
            "/api/audits/recent",                              // system-wide audit
            "/api/whiteboard/0c4f1e2a-0000-4000-8000-000000000002", // ward board
        ] {
            assert!(
                enforce(false, &Method::GET, path, &no_token, &verifier, &policy).is_ok(),
                "SEC-G8: with the flag OFF, {path} is open without a token (activation is the gate)"
            );
        }
    }

    /// Enforcement on ⇒ the public paths (health/ping, `OpenAPI`,
    /// Swagger UI, Prometheus metrics) still pass without a token.
    #[test]
    fn enforce_on_allows_public_paths() {
        let (keys, _) = test_keys_and_kid();
        let verifier = Verifier::from_paseto_keys_value(&keys, ISSUER, AUDIENCE).unwrap();
        let policy = policy();
        for path in [
            "/_health",
            "/_ping",
            "/api-docs/openapi.json",
            "/swagger-ui",
            "/swagger-ui/index.html",
            "/metrics.prom",
        ] {
            assert!(
                enforce(
                    true,
                    &Method::GET,
                    path,
                    &HeaderMap::new(),
                    &verifier,
                    &policy
                )
                .is_ok(),
                "{path} should be public"
            );
        }
    }

    /// Enforcement on, protected path, no token ⇒ `401`.
    #[test]
    fn enforce_on_protected_without_token_is_401() {
        let (keys, _) = test_keys_and_kid();
        let verifier = Verifier::from_paseto_keys_value(&keys, ISSUER, AUDIENCE).unwrap();
        let err = enforce(
            true,
            &Method::GET,
            "/api/wards",
            &HeaderMap::new(),
            &verifier,
            &policy(),
        )
        .unwrap_err();
        assert_eq!(err.0, StatusCode::UNAUTHORIZED);
    }

    /// Enforcement on, protected path, valid token ⇒ a read passes.
    #[test]
    fn enforce_on_protected_with_valid_token_is_ok() {
        let (keys, kid) = test_keys_and_kid();
        let verifier = Verifier::from_paseto_keys_value(&keys, ISSUER, AUDIENCE).unwrap();
        let token = sign(&kid, 10_000_000_000);
        assert!(
            enforce(
                true,
                &Method::GET,
                "/api/wards",
                &bearer(&token),
                &verifier,
                &policy()
            )
            .is_ok()
        );
    }

    /// Enforcement on, protected path, expired token ⇒ `401`.
    #[test]
    fn enforce_on_protected_with_expired_token_is_401() {
        let (keys, kid) = test_keys_and_kid();
        let verifier = Verifier::from_paseto_keys_value(&keys, ISSUER, AUDIENCE).unwrap();
        let token = sign(&kid, -60);
        let err = enforce(
            true,
            &Method::GET,
            "/api/wards",
            &bearer(&token),
            &verifier,
            &policy(),
        )
        .unwrap_err();
        assert_eq!(err.0, StatusCode::UNAUTHORIZED);
    }

    /// Enforcement on, protected path, tampered token ⇒ `401`.
    #[test]
    fn enforce_on_protected_with_tampered_token_is_401() {
        let (keys, kid) = test_keys_and_kid();
        let verifier = Verifier::from_paseto_keys_value(&keys, ISSUER, AUDIENCE).unwrap();
        let mut token = sign(&kid, 10_000_000_000);
        let last = token.pop().unwrap();
        token.push(if last == 'a' { 'b' } else { 'a' });
        let err = enforce(
            true,
            &Method::GET,
            "/api/wards",
            &bearer(&token),
            &verifier,
            &policy(),
        )
        .unwrap_err();
        assert_eq!(err.0, StatusCode::UNAUTHORIZED);
    }

    /// ABAC default policy, empty `attrs` ⇒ GET allowed, POST `403`
    /// (default allow-read / deny-mutation).
    #[test]
    fn abac_empty_attrs_reads_but_cannot_write() {
        let (keys, kid) = test_keys_and_kid();
        let verifier = Verifier::from_paseto_keys_value(&keys, ISSUER, AUDIENCE).unwrap();
        let policy = policy();
        let token = sign_with_attrs(&kid, 10_000_000_000, &[]);
        assert!(
            enforce(
                true,
                &Method::GET,
                "/api/wards",
                &bearer(&token),
                &verifier,
                &policy
            )
            .is_ok()
        );
        let err = enforce(
            true,
            &Method::POST,
            "/api/wards",
            &bearer(&token),
            &verifier,
            &policy,
        )
        .unwrap_err();
        assert_eq!(err.0, StatusCode::FORBIDDEN);
    }

    /// ABAC `access=write` ⇒ POST/PUT allowed; DELETE and the merge
    /// POST still `403` (write is not destructive).
    #[test]
    fn abac_access_write_writes_but_not_destructive() {
        let (keys, kid) = test_keys_and_kid();
        let verifier = Verifier::from_paseto_keys_value(&keys, ISSUER, AUDIENCE).unwrap();
        let policy = policy();
        let token = sign_with_attrs(&kid, 10_000_000_000, &[("access", &["write"])]);
        for method in [Method::POST, Method::PUT] {
            assert!(
                enforce(
                    true,
                    &method,
                    "/api/wards",
                    &bearer(&token),
                    &verifier,
                    &policy
                )
                .is_ok(),
                "{method} should be allowed for access=write"
            );
        }
        let delete = enforce(
            true,
            &Method::DELETE,
            "/api/wards/1",
            &bearer(&token),
            &verifier,
            &policy,
        )
        .unwrap_err();
        assert_eq!(delete.0, StatusCode::FORBIDDEN);
        let merge = enforce(
            true,
            &Method::POST,
            "/api/wards/merge",
            &bearer(&token),
            &verifier,
            &policy,
        )
        .unwrap_err();
        assert_eq!(merge.0, StatusCode::FORBIDDEN);
    }

    /// ABAC `access=admin` ⇒ DELETE and the destructive named POSTs
    /// are allowed (destructive covers delete).
    #[test]
    fn abac_access_admin_allows_destructive() {
        let (keys, kid) = test_keys_and_kid();
        let verifier = Verifier::from_paseto_keys_value(&keys, ISSUER, AUDIENCE).unwrap();
        let policy = policy();
        let token = sign_with_attrs(&kid, 10_000_000_000, &[("access", &["admin"])]);
        assert!(
            enforce(
                true,
                &Method::DELETE,
                "/api/wards/1",
                &bearer(&token),
                &verifier,
                &policy
            )
            .is_ok()
        );
        for path in ["/api/wards/merge", "/api/wards/deduplicate"] {
            assert!(
                enforce(
                    true,
                    &Method::POST,
                    path,
                    &bearer(&token),
                    &verifier,
                    &policy
                )
                .is_ok(),
                "{path} should be allowed for access=admin"
            );
        }
    }

    /// ABAC `svc=true` (machine peer) ⇒ everything is allowed.
    #[test]
    fn abac_svc_true_allows_everything() {
        let (keys, kid) = test_keys_and_kid();
        let verifier = Verifier::from_paseto_keys_value(&keys, ISSUER, AUDIENCE).unwrap();
        let policy = policy();
        let token = sign_with_attrs(&kid, 10_000_000_000, &[("svc", &["true"])]);
        for (method, path) in [
            (Method::GET, "/api/wards"),
            (Method::POST, "/api/wards"),
            (Method::PUT, "/api/wards/1"),
            (Method::DELETE, "/api/wards/1"),
            (Method::POST, "/api/wards/merge"),
            (Method::POST, "/api/wards/deduplicate"),
        ] {
            assert!(
                enforce(true, &method, path, &bearer(&token), &verifier, &policy).is_ok(),
                "{method} {path} should be allowed for svc=true"
            );
        }
    }

    /// A configured deny rule ahead of an allow rule wins
    /// (first-match-wins pin, through the guard).
    #[test]
    fn abac_configured_deny_beats_later_allow() {
        let (keys, kid) = test_keys_and_kid();
        let verifier = Verifier::from_paseto_keys_value(&keys, ISSUER, AUDIENCE).unwrap();
        let policy = Policy::from_json(
            r#"{ "rules": [
                { "effect": "deny",  "actions": ["write"], "when": { "purpose": ["research"] } },
                { "effect": "allow", "actions": ["write"], "when": { "access": ["write"] } }
            ] }"#,
        )
        .expect("policy parses");
        let denied = sign_with_attrs(
            &kid,
            10_000_000_000,
            &[("access", &["write"]), ("purpose", &["research"])],
        );
        let err = enforce(
            true,
            &Method::POST,
            "/api/wards",
            &bearer(&denied),
            &verifier,
            &policy,
        )
        .unwrap_err();
        assert_eq!(err.0, StatusCode::FORBIDDEN);
        let allowed = sign_with_attrs(&kid, 10_000_000_000, &[("access", &["write"])]);
        assert!(
            enforce(
                true,
                &Method::POST,
                "/api/wards",
                &bearer(&allowed),
                &verifier,
                &policy
            )
            .is_ok()
        );
    }

    /// 401 vs 403: missing/bad credential is `401`; a valid credential
    /// the policy denies is `403` with the deciding-rule reason.
    #[test]
    fn abac_401_versus_403_distinction() {
        let (keys, kid) = test_keys_and_kid();
        let verifier = Verifier::from_paseto_keys_value(&keys, ISSUER, AUDIENCE).unwrap();
        let policy = policy();
        let no_token = enforce(
            true,
            &Method::POST,
            "/api/wards",
            &HeaderMap::new(),
            &verifier,
            &policy,
        )
        .unwrap_err();
        assert_eq!(no_token.0, StatusCode::UNAUTHORIZED);
        let token = sign_with_attrs(&kid, 10_000_000_000, &[]);
        let denied = enforce(
            true,
            &Method::POST,
            "/api/wards",
            &bearer(&token),
            &verifier,
            &policy,
        )
        .unwrap_err();
        assert_eq!(denied.0, StatusCode::FORBIDDEN);
        assert_eq!(denied.1, "default deny");
    }

    /// Serve `keys` as the key-set JSON from a local ephemeral-port HTTP
    /// listener (the auth service's `/.well-known/paseto-keys` stand-in)
    /// and return the URL to fetch it from.
    async fn serve_keys(keys: serde_json::Value) -> String {
        let app = axum::Router::new().route(
            "/.well-known/paseto-keys",
            axum::routing::get(move || {
                let keys = keys.clone();
                async move { axum::Json(keys) }
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind ephemeral port");
        let addr = listener.local_addr().expect("local addr");
        tokio::spawn(async move {
            axum::serve(listener, app).await.expect("serve key set");
        });
        format!("http://{addr}/.well-known/paseto-keys")
    }

    /// Boot-time fetch happy path: `fetch_or` against a local listener
    /// serving a valid key set builds the verifier from the **fetched**
    /// keys — a token signed by the served key verifies even though the
    /// fallback verifier has no keys (the fetched key set wins).
    #[tokio::test]
    async fn fetch_or_fetched_key_set_wins() {
        let (keys, kid) = test_keys_and_kid();
        let url = serve_keys(keys).await;
        let verifier = fetch_or(&url, ISSUER, AUDIENCE, empty_verifier(ISSUER, AUDIENCE)).await;
        assert_eq!(verifier.key_count(), 1);
        let token = sign(&kid, 10_000_000_000);
        let claims =
            bearer_claims(&bearer(&token), &verifier).expect("token signed by fetched key");
        assert_eq!(claims.sub, "11111111-1111-1111-1111-111111111111");
    }

    /// Boot-time fetch fallback: a fast-failing URL (nothing listens on
    /// port 1) makes `fetch_or` return the env-style fallback verifier —
    /// no panic, and tokens signed by the fallback key set still verify,
    /// so the service always boots.
    #[tokio::test]
    async fn fetch_or_unreachable_url_falls_back() {
        let (keys, kid) = test_keys_and_kid();
        let fallback = Verifier::from_paseto_keys_value(&keys, ISSUER, AUDIENCE).unwrap();
        let verifier = fetch_or("http://127.0.0.1:1/", ISSUER, AUDIENCE, fallback).await;
        assert_eq!(verifier.key_count(), 1);
        let token = sign(&kid, 10_000_000_000);
        assert!(bearer_claims(&bearer(&token), &verifier).is_ok());
    }
}
