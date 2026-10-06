//! The `keycloak` backend: this service verifies a Keycloak-issued JWT
//! directly against the realm's JWKS — no PASETO, no sibling
//! authentication service in this request path. Everything downstream
//! of a verified [`Claims`] — the ABAC engine, masking,
//! `AuthUser`/`MaybeAuthUser` — lives in the parent [`super`] module
//! and is shared with the `paseto` backend; this module owns only "how
//! do we turn a bearer header into `Claims`", plus the realm-claim →
//! ABAC `attrs` mapping below.
//!
//! This is a *different* Keycloak integration point from the one in
//! `spec/auth.md`'s "Keycloak as the identity provider" runbook: that
//! runbook covers pointing the *sibling authentication service* at
//! Keycloak as its own upstream `IdP` (via that service's own
//! `AUTH_OIDC_*` config), after which it keeps minting PASETO for this
//! crate's `paseto` backend to verify, unchanged. This module instead
//! lets this service skip that service entirely and verify a
//! Keycloak-issued JWT itself.
//!
//! ## Key source
//!
//! The process-wide [`verifier`] is seeded once at boot ([`init`] is
//! called from `App::after_routes`) and built from the environment:
//!
//! - `WPM_KEYCLOAK_JWKS_URL` — the realm's JWKS endpoint (Keycloak:
//!   `<issuer>/protocol/openid-connect/certs`). Set (non-blank) ⇒
//!   fetched over HTTP **once at boot**; on failure, or with the URL
//!   unset/blank, the verifier keeps an empty JWKS — every token is
//!   rejected, but the service still boots, matching the `paseto`
//!   backend's fail-safe posture for an unset `WPM_PASETO_KEYS`. The
//!   JWKS is then **re-fetched periodically** ([`spawn_key_refresh`])
//!   so a key rotation is picked up without a restart (interval
//!   `WPM_KEYCLOAK_JWKS_REFRESH_SECS`, default 1 h; `0` disables).
//! - `WPM_KEYCLOAK_ISSUER` — expected `iss` (the realm issuer URL,
//!   e.g. `https://keycloak.example/realms/wpm`).
//! - `WPM_KEYCLOAK_AUDIENCE` — expected `aud` (the confidential client
//!   id registered for this deployment). Keycloak may emit `aud` as a
//!   bare string or an array; both are accepted.
//!
//! `iss`/`aud` are checked explicitly in [`bearer_claims`] rather than
//! through `jsonwebtoken`'s own issuer/audience validation, so the
//! check is visible here and independent of that crate's exact
//! validation semantics. The algorithm allow-list ([`Validation`]) is
//! fixed to `RS256`/`ES256` regardless of the token's own `alg` header,
//! guarding against an algorithm-confusion downgrade.
//!
//! ## Claim mapping
//!
//! Realm-mapper configuration a deployment sets up in Keycloak, and
//! the WPM ABAC `attrs` key each produces (mirrors, and should stay in
//! sync with, the table in `spec/auth.md`):
//!
//! | WPM `attrs` key | Keycloak source |
//! |---|---|
//! | `hr` | realm role `wpm-hr` present ⇒ `["true"]` |
//! | `payroll` | realm role `wpm-payroll` present ⇒ `["true"]` |
//! | `svc` | realm role `wpm-svc` present ⇒ `["true"]` (service-account callers only) |
//! | `access` | realm role `wpm-admin` present ⇒ `["admin"]`, else `["write"]` for any authenticated caller |
//! | `department` | `groups` claim (group-membership mapper): each path's leaf segment |
//! | `organization_ref` | `organization_ref` claim (custom user/group attribute mapper): verbatim, multi-valued |
//!
//! `resource.person = $sub` self-rules need no mapper: `$sub` is
//! Keycloak's own `sub` claim, relayed as-is.

use super::{
    BTreeMap, Claims, ENTITY, Method, OnceLock, Policy, StatusCode, derive_action, env_or,
    is_public_path,
};
use axum::http::HeaderMap;
use axum::http::header::AUTHORIZATION;
use jsonwebtoken::jwk::JwkSet;
use jsonwebtoken::{Algorithm, DecodingKey, Header as JwtHeader, Validation};

/// The raw claims of a Keycloak-issued JWT — only the fields the claim
/// mapping above reads. Deserialized once per request, then folded
/// into this crate's backend-agnostic [`Claims`] by [`bearer_claims`].
#[derive(serde::Deserialize)]
struct KeycloakClaims {
    sub: String,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    preferred_username: Option<String>,
    iss: String,
    /// Keycloak emits either a bare string or an array here depending
    /// on realm/client configuration; checked against the configured
    /// audience in [`bearer_claims`] without assuming a shape.
    aud: serde_json::Value,
    exp: i64,
    iat: i64,
    #[serde(default)]
    nbf: Option<i64>,
    #[serde(default)]
    sid: Option<String>,
    #[serde(default)]
    realm_access: Option<RealmAccess>,
    #[serde(default)]
    groups: Option<Vec<String>>,
    #[serde(default)]
    organization_ref: Option<Vec<String>>,
}

/// The `realm_access` claim's shape (a Keycloak default protocol
/// mapper): the caller's realm roles.
#[derive(serde::Deserialize)]
struct RealmAccess {
    #[serde(default)]
    roles: Vec<String>,
}

/// Map a verified Keycloak JWT's claims to the ABAC `attrs` this
/// crate's policy engine already reads. See the module docs' claim
/// mapping table for the realm-mapper configuration this expects.
fn attrs_from_keycloak_claims(claims: &KeycloakClaims) -> BTreeMap<String, Vec<String>> {
    let mut attrs = BTreeMap::new();
    let roles = claims.realm_access.as_ref().map_or(&[][..], |r| &r.roles);
    let has_role = |role: &str| roles.iter().any(|r| r == role);
    if has_role("wpm-hr") {
        attrs.insert("hr".to_string(), vec!["true".to_string()]);
    }
    if has_role("wpm-payroll") {
        attrs.insert("payroll".to_string(), vec!["true".to_string()]);
    }
    if has_role("wpm-svc") {
        attrs.insert("svc".to_string(), vec!["true".to_string()]);
    }
    attrs.insert(
        "access".to_string(),
        vec![
            if has_role("wpm-admin") {
                "admin"
            } else {
                "write"
            }
            .to_string(),
        ],
    );
    if let Some(groups) = &claims.groups {
        let departments: Vec<String> = groups
            .iter()
            .filter_map(|g| g.rsplit('/').next())
            .filter(|leaf| !leaf.is_empty())
            .map(ToString::to_string)
            .collect();
        if !departments.is_empty() {
            attrs.insert("department".to_string(), departments);
        }
    }
    if let Some(org_refs) = &claims.organization_ref
        && !org_refs.is_empty()
    {
        attrs.insert("organization_ref".to_string(), org_refs.clone());
    }
    attrs
}

/// This backend's hot-reloadable key material: the realm's JWKS plus
/// the issuer/audience every token is checked against.
/// `authentication-verifier` has no JWKS concept, so this is a small
/// local equivalent of its `Verifier`.
#[derive(Clone)]
pub struct KeyMaterial {
    jwks: std::sync::Arc<JwkSet>,
    issuer: String,
    audience: String,
}

/// A minimal hot-swappable holder, mirroring
/// `authentication_verifier::ReloadableVerifier`'s `.current()`/
/// `.store()` shape so `src/app.rs` needs no `#[cfg]` to call whichever
/// backend is active.
pub struct Reloadable<T>(std::sync::Mutex<std::sync::Arc<T>>);

impl<T: Clone> Reloadable<T> {
    /// Wrap an initial value.
    fn new(value: T) -> Self {
        Self(std::sync::Mutex::new(std::sync::Arc::new(value)))
    }

    /// A cheap clone of the current value.
    ///
    /// # Panics
    ///
    /// If the lock was poisoned by a panic while storing.
    #[must_use]
    pub fn current(&self) -> T {
        (**self.0.lock().expect("reloadable lock poisoned")).clone()
    }

    /// Swap in a new value; a caller already holding a snapshot from
    /// [`Reloadable::current`] keeps using theirs.
    ///
    /// # Panics
    ///
    /// If the lock was poisoned by a panic while reading.
    pub fn store(&self, value: T) {
        *self.0.lock().expect("reloadable lock poisoned") = std::sync::Arc::new(value);
    }
}

/// An empty JWKS: rejects every token until a real key set is fetched.
/// Built by deserializing rather than a struct literal, since
/// `jsonwebtoken::jwk::Jwk`/`JwkSet` are meant to be built from wire
/// JSON, not constructed directly.
fn empty_jwks() -> JwkSet {
    serde_json::from_str(r#"{"keys":[]}"#).expect("empty JWKS always parses")
}

/// Build the process-wide [`KeyMaterial`] from the environment: issuer,
/// audience, and an empty JWKS (populated by [`init`]'s boot fetch).
fn build_from_env() -> KeyMaterial {
    KeyMaterial {
        jwks: std::sync::Arc::new(empty_jwks()),
        issuer: env_or("WPM_KEYCLOAK_ISSUER", ""),
        audience: env_or("WPM_KEYCLOAK_AUDIENCE", ""),
    }
}

/// The process-wide **hot-reloadable** key material. Lazily built from
/// the environment on first use ([`build_from_env`]); [`init`] swaps in
/// the boot-time fetched JWKS, and [`spawn_key_refresh`] swaps in a
/// re-fetched JWKS periodically (**key rotation without a restart**).
static KEY_MATERIAL: OnceLock<Reloadable<KeyMaterial>> = OnceLock::new();

/// The process-wide reloadable key material. Read the active snapshot
/// with `verifier().current()` per request; it is swapped by [`init`]
/// (boot fetch) and [`spawn_key_refresh`] (periodic re-fetch).
#[must_use]
pub fn verifier() -> &'static Reloadable<KeyMaterial> {
    KEY_MATERIAL.get_or_init(|| Reloadable::new(build_from_env()))
}

/// Fetch the JWKS at `url`; `None` (warn-logged) on any network or
/// parse failure, so the caller can fall back to whatever key material
/// it already has. Short timeout, no redirects — the same SSRF
/// posture as this crate's other upstream fetch
/// (`clients::reqwest_get_json`, security invariant 7).
async fn fetch_jwks(url: &str) -> Option<JwkSet> {
    let Ok(client) = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .redirect(reqwest::redirect::Policy::none())
        .build()
    else {
        return None;
    };
    let response = match client.get(url).send().await {
        Ok(response) => response,
        Err(error) => {
            tracing::warn!(url, %error, "Keycloak JWKS fetch failed");
            return None;
        }
    };
    match response.json::<JwkSet>().await {
        Ok(jwks) => {
            tracing::info!(
                url,
                keys = jwks.keys.len(),
                "Keycloak JWKS fetched over HTTP"
            );
            Some(jwks)
        }
        Err(error) => {
            tracing::warn!(url, %error, "Keycloak JWKS response unparsable");
            None
        }
    }
}

/// Seed the process-wide [`verifier`] before the app serves traffic
/// (called from `App::after_routes`). A no-op when
/// `WPM_KEYCLOAK_JWKS_URL` is unset/blank; on a failed fetch the
/// env-built (empty) JWKS stands, so the service always boots.
pub async fn init() {
    let Some(url) =
        crate::compat::env_var("WPM_KEYCLOAK_JWKS_URL").filter(|s| !s.trim().is_empty())
    else {
        return;
    };
    if let Some(jwks) = fetch_jwks(url.trim()).await {
        let current = verifier().current();
        verifier().store(KeyMaterial {
            jwks: std::sync::Arc::new(jwks),
            ..current
        });
    }
}

/// Default JWKS refresh interval (seconds) when
/// `WPM_KEYCLOAK_JWKS_REFRESH_SECS` is unset — mirrors the `paseto`
/// backend's own default.
const KEY_REFRESH_DEFAULT_SECS: u64 = 3600;

/// Spawn a background task that periodically re-fetches the realm's
/// JWKS from `WPM_KEYCLOAK_JWKS_URL` and swaps it into the live
/// [`verifier`], so a **key rotation** at Keycloak is picked up
/// **without restarting** this service. On a failed fetch it keeps the
/// current JWKS. Interval from `WPM_KEYCLOAK_JWKS_REFRESH_SECS`
/// (default [`KEY_REFRESH_DEFAULT_SECS`]); **`0` disables** the loop.
///
/// A no-op when `WPM_KEYCLOAK_JWKS_URL` is unset. Call once at boot
/// (`app.rs::after_routes`).
pub fn spawn_key_refresh() {
    let Some(url) = crate::compat::env_var("WPM_KEYCLOAK_JWKS_URL")
        .map(|u| u.trim().to_string())
        .filter(|u| !u.is_empty())
    else {
        return;
    };
    let secs = crate::compat::env_var("WPM_KEYCLOAK_JWKS_REFRESH_SECS")
        .and_then(|v| v.trim().parse::<u64>().ok())
        .unwrap_or(KEY_REFRESH_DEFAULT_SECS);
    if secs == 0 {
        return; // explicitly disabled
    }
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(std::time::Duration::from_secs(secs));
        ticker.tick().await; // the first tick is immediate — skip it
        loop {
            ticker.tick().await;
            if let Some(jwks) = fetch_jwks(&url).await {
                let current = verifier().current();
                verifier().store(KeyMaterial {
                    jwks: std::sync::Arc::new(jwks),
                    ..current
                });
                tracing::info!("refreshed Keycloak JWKS");
            } else {
                tracing::warn!("Keycloak JWKS refresh failed; keeping current keys");
            }
        }
    });
    tracing::info!(secs, "polling WPM_KEYCLOAK_JWKS_URL for key rotation");
}

/// Extract and verify the bearer token from request headers, mapping
/// it to this crate's backend-agnostic [`Claims`]. Pure (the key
/// material is passed in), so it is unit-testable without the global.
///
/// # Errors
///
/// `401` when the `Authorization` header is missing, is not a bearer
/// token, the token is malformed, its `kid` is not in `key_material`'s
/// JWKS, its signature/expiry/not-before fails, or its `iss`/`aud`
/// does not match `key_material`.
pub fn bearer_claims(
    headers: &HeaderMap,
    key_material: &KeyMaterial,
) -> Result<Claims, (StatusCode, String)> {
    let unauthorized = |msg: &str| (StatusCode::UNAUTHORIZED, msg.to_string());

    let header = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| unauthorized("missing authorization header"))?;
    let token = header
        .strip_prefix("Bearer ")
        .or_else(|| header.strip_prefix("bearer "))
        .ok_or_else(|| unauthorized("expected bearer token"))?
        .trim();

    let jwt_header: JwtHeader =
        jsonwebtoken::decode_header(token).map_err(|_| unauthorized("malformed token header"))?;
    let kid = jwt_header
        .kid
        .as_deref()
        .ok_or_else(|| unauthorized("token header missing kid"))?;
    let jwk = key_material
        .jwks
        .find(kid)
        .ok_or_else(|| unauthorized("unknown signing key"))?;
    let decoding_key =
        DecodingKey::from_jwk(jwk).map_err(|_| unauthorized("unusable signing key"))?;

    // The algorithm allow-list is fixed here, independent of the
    // token's own (attacker-controlled) `alg` header, guarding against
    // an algorithm-confusion downgrade.
    // `jsonwebtoken` refuses a validation list that mixes key families
    // (RS256 + ES256) with `InvalidAlgorithm`, so the single algorithm
    // checked is the header's — but only if it is on the allow-list.
    let algorithm = [Algorithm::RS256, Algorithm::ES256]
        .into_iter()
        .find(|allowed| *allowed == jwt_header.alg)
        .ok_or_else(|| unauthorized("unsupported signing algorithm"))?;
    let mut validation = Validation::new(algorithm);
    // `iss`/`aud` are checked explicitly below instead.
    validation.validate_aud = false;
    validation.validate_nbf = true;

    let data = jsonwebtoken::decode::<KeycloakClaims>(token, &decoding_key, &validation)
        .map_err(|error| unauthorized(&error.to_string()))?;
    let claims = data.claims;

    if claims.iss != key_material.issuer {
        return Err(unauthorized("unexpected issuer"));
    }
    let audience_matches = match &claims.aud {
        serde_json::Value::String(aud) => *aud == key_material.audience,
        serde_json::Value::Array(values) => values
            .iter()
            .any(|v| v.as_str() == Some(key_material.audience.as_str())),
        _ => false,
    };
    if !audience_matches {
        return Err(unauthorized("unexpected audience"));
    }

    // Derived before the field moves below: it borrows the whole struct.
    let attrs = attrs_from_keycloak_claims(&claims);
    Ok(Claims {
        sub: claims.sub,
        email: claims.email.unwrap_or_default(),
        name: claims
            .name
            .or(claims.preferred_username)
            .unwrap_or_default(),
        iss: key_material.issuer.clone(),
        aud: key_material.audience.clone(),
        exp: claims.exp,
        iat: claims.iat,
        nbf: claims.nbf,
        sid: claims.sid.unwrap_or_default(),
        scope: Vec::new(),
        roles: claims
            .realm_access
            .as_ref()
            .map(|r| r.roles.clone())
            .unwrap_or_default(),
        attrs,
    })
}

/// The blanket-enforcement decision: authentication, then ABAC
/// authorization. `Ok(())` ⇒ let the request through; `Err((401|403,
/// msg))` ⇒ reject. Pure: the caller passes the flag, method, path,
/// headers, key material and policy, so it is fully unit-testable
/// without booting the app or a database. Mirrors the `paseto`
/// backend's `enforce` exactly, aside from the key-material type.
///
/// # Errors
///
/// `401` when enforcement is on, the path is not public, and the
/// request carries no valid bearer token. `403` when the token is
/// valid but the ABAC policy denies the derived action (the message
/// names the deciding rule, per `authorization-attributes.md` §5).
pub fn enforce(
    require_auth: bool,
    method: &Method,
    path: &str,
    headers: &HeaderMap,
    key_material: &KeyMaterial,
    policy: &Policy,
) -> Result<(), (StatusCode, String)> {
    if !require_auth || is_public_path(path) {
        return Ok(());
    }
    let claims = bearer_claims(headers, key_material)?;
    let decision = policy.evaluate(&claims, derive_action(method, path), ENTITY);
    if decision.allowed {
        Ok(())
    } else {
        Err((StatusCode::FORBIDDEN, decision.reason))
    }
}

/// Pins for the claim-to-`attrs` mapping (pure, no crypto) and the
/// token-verification round trip (an in-process ES256 keypair signs
/// test tokens and a matching JWKS verifies them — the JWKS/JWT
/// equivalent of `paseto`'s in-process Ed25519 pins). This module is
/// new and, unlike `paseto`'s, has never been exercised against a real
/// Keycloak realm or compiled in this environment — treat it as a
/// starting point for review, not a finished, field-proven path.
#[cfg(test)]
mod tests {
    use super::*;
    use aws_lc_rs::signature::KeyPair;
    use base64::Engine;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;

    const ISSUER: &str = "https://keycloak.example/realms/wpm";
    const AUDIENCE: &str = "wpm-service";

    /// Claim-mapping pins, no crypto: build a [`KeycloakClaims`]
    /// directly and check the derived `attrs`.
    fn claims(roles: &[&str], groups: &[&str], organization_ref: &[&str]) -> KeycloakClaims {
        KeycloakClaims {
            sub: "11111111-1111-1111-1111-111111111111".to_string(),
            email: Some("alice@example.com".to_string()),
            name: Some("Alice".to_string()),
            preferred_username: None,
            iss: ISSUER.to_string(),
            aud: serde_json::json!(AUDIENCE),
            exp: 2_000_000_000,
            iat: 1_900_000_000,
            nbf: None,
            sid: Some("test-sid".to_string()),
            realm_access: Some(RealmAccess {
                roles: roles.iter().map(ToString::to_string).collect(),
            }),
            groups: (!groups.is_empty()).then(|| groups.iter().map(ToString::to_string).collect()),
            organization_ref: (!organization_ref.is_empty())
                .then(|| organization_ref.iter().map(ToString::to_string).collect()),
        }
    }

    #[test]
    fn plain_authenticated_human_gets_access_write() {
        let attrs = attrs_from_keycloak_claims(&claims(&[], &[], &[]));
        assert_eq!(attrs["access"], vec!["write".to_string()]);
        assert!(!attrs.contains_key("hr"));
        assert!(!attrs.contains_key("payroll"));
        assert!(!attrs.contains_key("svc"));
    }

    #[test]
    fn realm_roles_map_to_hr_payroll_svc_and_admin() {
        let attrs = attrs_from_keycloak_claims(&claims(&["wpm-hr"], &[], &[]));
        assert_eq!(attrs["hr"], vec!["true".to_string()]);
        assert_eq!(attrs["access"], vec!["write".to_string()]);

        let attrs = attrs_from_keycloak_claims(&claims(&["wpm-payroll"], &[], &[]));
        assert_eq!(attrs["payroll"], vec!["true".to_string()]);

        let attrs = attrs_from_keycloak_claims(&claims(&["wpm-svc"], &[], &[]));
        assert_eq!(attrs["svc"], vec!["true".to_string()]);

        let attrs = attrs_from_keycloak_claims(&claims(&["wpm-admin"], &[], &[]));
        assert_eq!(attrs["access"], vec!["admin".to_string()]);
    }

    #[test]
    fn group_paths_map_to_department_leaf_segments() {
        let attrs =
            attrs_from_keycloak_claims(&claims(&[], &["/org/engineering", "/org/finance"], &[]));
        let mut department = attrs["department"].clone();
        department.sort();
        assert_eq!(
            department,
            vec!["engineering".to_string(), "finance".to_string()]
        );
    }

    #[test]
    fn organization_ref_is_relayed_verbatim_and_multi_valued() {
        let attrs = attrs_from_keycloak_claims(&claims(
            &[],
            &[],
            &[
                "organization:11111111-1111-1111-1111-111111111111",
                "organization:2",
            ],
        ));
        assert_eq!(
            attrs["organization_ref"],
            vec![
                "organization:11111111-1111-1111-1111-111111111111".to_string(),
                "organization:2".to_string(),
            ]
        );
    }

    /// Generate a throwaway ES256 (P-256) keypair for signing test
    /// tokens, returning the PKCS8 DER (for `EncodingKey::from_ec_der`)
    /// and the base64url `x`/`y` coordinates (for the matching JWKS
    /// entry `DecodingKey::from_jwk` verifies against).
    fn generate_es256_test_key() -> (Vec<u8>, String, String) {
        let rng = aws_lc_rs::rand::SystemRandom::new();
        let pkcs8 = aws_lc_rs::signature::EcdsaKeyPair::generate_pkcs8(
            &aws_lc_rs::signature::ECDSA_P256_SHA256_FIXED_SIGNING,
            &rng,
        )
        .expect("generate EC keypair");
        let key_pair = aws_lc_rs::signature::EcdsaKeyPair::from_pkcs8(
            &aws_lc_rs::signature::ECDSA_P256_SHA256_FIXED_SIGNING,
            pkcs8.as_ref(),
        )
        .expect("parse generated keypair");
        let public = key_pair.public_key().as_ref();
        assert_eq!(
            public.len(),
            65,
            "uncompressed P-256 point (0x04 || X || Y)"
        );
        let x = URL_SAFE_NO_PAD.encode(&public[1..33]);
        let y = URL_SAFE_NO_PAD.encode(&public[33..65]);
        (pkcs8.as_ref().to_vec(), x, y)
    }

    /// The [`KeyMaterial`] a test verifies against: a JWKS holding one
    /// EC key at `kid`, plus the configured issuer/audience.
    fn key_material(kid: &str, x: &str, y: &str, issuer: &str, audience: &str) -> KeyMaterial {
        let jwks_json = format!(
            r#"{{"keys":[{{"kty":"EC","crv":"P-256","kid":"{kid}","x":"{x}","y":"{y}","use":"sig","alg":"ES256"}}]}}"#
        );
        KeyMaterial {
            jwks: std::sync::Arc::new(serde_json::from_str(&jwks_json).expect("jwks parses")),
            issuer: issuer.to_string(),
            audience: audience.to_string(),
        }
    }

    /// Sign `claims` (already the wire-shape JSON, so a test can bend
    /// any field — issuer, audience, expiry — independent of
    /// `KeycloakClaims`'s own `Deserialize` impl) as an ES256 JWT with
    /// `kid` in the header.
    fn sign(pkcs8_der: &[u8], kid: &str, claims: &serde_json::Value) -> String {
        let mut header = JwtHeader::new(Algorithm::ES256);
        header.kid = Some(kid.to_string());
        let key = jsonwebtoken::EncodingKey::from_ec_der(pkcs8_der);
        jsonwebtoken::encode(&header, claims, &key).expect("sign JWT")
    }

    /// A default well-formed claims payload, overridable per test.
    fn default_claims_json() -> serde_json::Value {
        serde_json::json!({
            "sub": "11111111-1111-1111-1111-111111111111",
            "email": "alice@example.com",
            "iss": ISSUER,
            "aud": AUDIENCE,
            "exp": 2_000_000_000,
            "iat": 1_900_000_000,
        })
    }

    fn bearer(token: &str) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert(AUTHORIZATION, format!("Bearer {token}").parse().unwrap());
        h
    }

    #[test]
    fn valid_token_yields_claims() {
        let (pkcs8, x, y) = generate_es256_test_key();
        let material = key_material("test-key", &x, &y, ISSUER, AUDIENCE);
        let token = sign(&pkcs8, "test-key", &default_claims_json());
        let claims = bearer_claims(&bearer(&token), &material).expect("valid token verifies");
        assert_eq!(claims.sub, "11111111-1111-1111-1111-111111111111");
        assert_eq!(claims.email, "alice@example.com");
        assert_eq!(claims.iss, ISSUER);
        assert_eq!(claims.aud, AUDIENCE);
    }

    #[test]
    fn missing_header_is_401() {
        let material = key_material("test-key", "", "", ISSUER, AUDIENCE);
        assert_eq!(
            bearer_claims(&HeaderMap::new(), &material).unwrap_err().0,
            StatusCode::UNAUTHORIZED
        );
    }

    #[test]
    fn non_bearer_header_is_401() {
        let material = key_material("test-key", "", "", ISSUER, AUDIENCE);
        let mut h = HeaderMap::new();
        h.insert(AUTHORIZATION, "Basic abc123".parse().unwrap());
        assert_eq!(
            bearer_claims(&h, &material).unwrap_err().0,
            StatusCode::UNAUTHORIZED
        );
    }

    #[test]
    fn unknown_kid_is_401() {
        let (pkcs8, x, y) = generate_es256_test_key();
        // The JWKS is keyed under a different `kid` than the token uses.
        let material = key_material("other-key", &x, &y, ISSUER, AUDIENCE);
        let token = sign(&pkcs8, "test-key", &default_claims_json());
        assert_eq!(
            bearer_claims(&bearer(&token), &material).unwrap_err().0,
            StatusCode::UNAUTHORIZED
        );
    }

    #[test]
    fn wrong_issuer_is_401() {
        let (pkcs8, x, y) = generate_es256_test_key();
        let material = key_material("test-key", &x, &y, ISSUER, AUDIENCE);
        let mut claims = default_claims_json();
        claims["iss"] = serde_json::json!("https://not-the-configured-realm.example");
        let token = sign(&pkcs8, "test-key", &claims);
        assert_eq!(
            bearer_claims(&bearer(&token), &material).unwrap_err().0,
            StatusCode::UNAUTHORIZED
        );
    }

    #[test]
    fn wrong_audience_is_401() {
        let (pkcs8, x, y) = generate_es256_test_key();
        let material = key_material("test-key", &x, &y, ISSUER, AUDIENCE);
        let mut claims = default_claims_json();
        claims["aud"] = serde_json::json!("some-other-client");
        let token = sign(&pkcs8, "test-key", &claims);
        assert_eq!(
            bearer_claims(&bearer(&token), &material).unwrap_err().0,
            StatusCode::UNAUTHORIZED
        );
    }

    #[test]
    fn audience_as_array_matches_when_configured_audience_is_a_member() {
        let (pkcs8, x, y) = generate_es256_test_key();
        let material = key_material("test-key", &x, &y, ISSUER, AUDIENCE);
        let mut claims = default_claims_json();
        claims["aud"] = serde_json::json!(["some-other-client", AUDIENCE]);
        let token = sign(&pkcs8, "test-key", &claims);
        assert!(bearer_claims(&bearer(&token), &material).is_ok());
    }

    #[test]
    fn expired_token_is_401() {
        let (pkcs8, x, y) = generate_es256_test_key();
        let material = key_material("test-key", &x, &y, ISSUER, AUDIENCE);
        let mut claims = default_claims_json();
        claims["exp"] = serde_json::json!(1_000);
        claims["iat"] = serde_json::json!(500);
        let token = sign(&pkcs8, "test-key", &claims);
        assert_eq!(
            bearer_claims(&bearer(&token), &material).unwrap_err().0,
            StatusCode::UNAUTHORIZED
        );
    }

    #[test]
    fn tampered_signature_is_401() {
        let (pkcs8, x, y) = generate_es256_test_key();
        let material = key_material("test-key", &x, &y, ISSUER, AUDIENCE);
        let mut token = sign(&pkcs8, "test-key", &default_claims_json());
        let last = token.pop().unwrap();
        token.push(if last == 'a' { 'b' } else { 'a' });
        assert_eq!(
            bearer_claims(&bearer(&token), &material).unwrap_err().0,
            StatusCode::UNAUTHORIZED
        );
    }

    #[test]
    fn realm_roles_carry_through_to_the_abac_decision() {
        let (pkcs8, x, y) = generate_es256_test_key();
        let material = key_material("test-key", &x, &y, ISSUER, AUDIENCE);
        let mut claims = default_claims_json();
        claims["realm_access"] = serde_json::json!({"roles": ["wpm-hr"]});
        let token = sign(&pkcs8, "test-key", &claims);
        let verified = bearer_claims(&bearer(&token), &material).expect("valid token verifies");
        assert_eq!(verified.attrs["hr"], vec!["true".to_string()]);
    }

    #[test]
    fn enforce_off_allows_without_token() {
        let material = key_material("test-key", "", "", ISSUER, AUDIENCE);
        let policy = Policy::default_policy();
        assert!(
            enforce(
                false,
                &Method::GET,
                "/api/workers",
                &HeaderMap::new(),
                &material,
                &policy
            )
            .is_ok()
        );
    }

    #[test]
    fn enforce_on_protected_without_token_is_401() {
        let material = key_material("test-key", "", "", ISSUER, AUDIENCE);
        let policy = Policy::default_policy();
        let err = enforce(
            true,
            &Method::GET,
            "/api/workers",
            &HeaderMap::new(),
            &material,
            &policy,
        )
        .unwrap_err();
        assert_eq!(err.0, StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn enforce_on_protected_with_valid_token_is_ok() {
        let (pkcs8, x, y) = generate_es256_test_key();
        let material = key_material("test-key", &x, &y, ISSUER, AUDIENCE);
        let policy = Policy::default_policy();
        let token = sign(&pkcs8, "test-key", &default_claims_json());
        assert!(
            enforce(
                true,
                &Method::GET,
                "/api/workers",
                &bearer(&token),
                &material,
                &policy
            )
            .is_ok()
        );
    }

    #[test]
    fn enforce_on_allows_public_paths_without_token() {
        let material = key_material("test-key", "", "", ISSUER, AUDIENCE);
        let policy = Policy::default_policy();
        assert!(
            enforce(
                true,
                &Method::GET,
                "/_health",
                &HeaderMap::new(),
                &material,
                &policy
            )
            .is_ok()
        );
    }
}
