//! Bearer-token authentication for the workforce-planning-management API.
//!
//! Exactly one token-verification **backend** is compiled in, chosen at
//! build time by Cargo feature (see `Cargo.toml`'s `[features]` table);
//! mixing both, or neither, is a compile error below. Both backends
//! verify a bearer token into the *same* [`Claims`] type (from the
//! `authentication-verifier` crate) and hand it to the *same*
//! downstream code in this module — the ABAC policy engine
//! ([`policy`], [`authorize_record`]), masking ([`mask_worker`],
//! [`mask_payslip`]), and the [`AuthUser`]/[`MaybeAuthUser`] extractors
//! — so no controller, and nothing below this point in the file, knows
//! or cares which backend is active.
//!
//! - **`paseto`** (default; [`paseto`] submodule) — offline PASETO
//!   `v4.public` (Ed25519) verification against the sibling
//!   [authentication-service](../../../authentication/authentication-service-with-loco)'s
//!   published key set. See that submodule's docs for
//!   `WPM_PASETO_KEYS*` / `WPM_TOKEN_*`.
//! - **`keycloak`** ([`keycloak`] submodule) — this service verifies a
//!   Keycloak-issued JWT directly against the realm's JWKS; no PASETO,
//!   no sibling authentication service in this request path. See that
//!   submodule's docs for `WPM_KEYCLOAK_*` and the realm-claim → ABAC
//!   `attrs` mapping (also documented in `spec/auth.md`'s "Keycloak as
//!   the identity provider" runbook, which additionally covers the
//!   *other*, independent way Keycloak enters this family: as the
//!   sibling authentication service's own upstream IdP via its
//!   `AUTH_OIDC_*` config — a different integration point from this
//!   one, and usable without ever enabling this crate's `keycloak`
//!   feature).
//!
//! [`AuthUser`] is an Axum extractor that requires a valid bearer
//! token and yields the verified [`Claims`]; a handler that wants the
//! caller identity *when present* (e.g. to stamp an audit `actor`)
//! takes [`MaybeAuthUser`] instead. Both backends verify statelessly
//! and offline — no database hit, no introspection call per request.
//!
//! ## Blanket enforcement
//!
//! When `WPM_REQUIRE_AUTH` is truthy (`1`/`true`/`yes`/`on`,
//! case-insensitive), the active backend's `enforce` decision — wired
//! as an Axum middleware layer in `src/app.rs` — requires a valid
//! bearer token on every route except the public health/ping,
//! OpenAPI/Swagger, and Prometheus metrics paths (see
//! [`is_public_path`]). It is **off by default**: unset/blank/junk ⇒
//! today's behaviour, where the extractor is opt-in per handler and
//! the extractor path proves end-to-end verification. Activation is an
//! operations decision once the SSO token flow is live; see
//! `agents/share/authentication-sessions.md` and
//! `agents/share/jwt-enforcement.md` for the family-wide contract.
//!
//! ## Authorization (ABAC)
//!
//! Inside the same guard — so it applies only when `WPM_REQUIRE_AUTH`
//! is on — a verified token is further checked against an
//! **attribute-based access control** policy per
//! `agents/share/authorization-attributes.md`: the request's action is
//! derived from the HTTP method plus this crate's destructive named
//! POSTs ([`DESTRUCTIVE_POST_SUFFIXES`]), and the shared engine in the
//! `authentication-verifier` crate evaluates the policy over the
//! token's `attrs` claim — populated identically regardless of which
//! backend produced it. The policy is read once per process
//! ([`policy`], built by [`policy_from_env`]) from `WPM_ABAC_POLICY`
//! (inline JSON) or `WPM_ABAC_POLICY_FILE` (path); unset or unparsable
//! ⇒ the built-in default policy (`svc=true` ⇒ everything;
//! `access=admin` ⇒ destructive+write; `access=write` ⇒ write;
//! otherwise read-only) — the service always boots. **401** =
//! missing/bad credential; **403** = valid credential, policy denied
//! (body carries the deciding rule). Employment data is personal data:
//! deployments can express e.g. department or purpose-of-use scoping
//! as configured policy rules over the same `attrs` claim, no code
//! change required.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use crate::models::_entities::{payslips, workers};
use authentication_verifier::{Action, Claims, Policy, ReloadablePolicy};
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::{Method, StatusCode};

#[cfg(all(feature = "paseto", feature = "keycloak"))]
compile_error!(
    "enable exactly one of the \"paseto\" or \"keycloak\" Cargo features, not both (see Cargo.toml's [features] table)"
);
#[cfg(not(any(feature = "paseto", feature = "keycloak")))]
compile_error!(
    "enable exactly one of the \"paseto\" or \"keycloak\" Cargo features (see Cargo.toml's [features] table)"
);

#[cfg(feature = "paseto")]
mod paseto;
#[cfg(feature = "paseto")]
pub use paseto::*;

#[cfg(feature = "keycloak")]
mod keycloak;
#[cfg(feature = "keycloak")]
pub use keycloak::*;

/// The resource entity this crate guards, as seen by ABAC policies
/// (the `entity` pseudo-attribute in rule `when` clauses).
pub const ENTITY: &str = "wpm";

/// Path suffixes of this crate's **destructive named POSTs** (per
/// `authorization-attributes.md` §2): the family-wide trio — record
/// merge, batch deduplicate, bulk import (listed ahead of the
/// corresponding features so the guard is already correct when they
/// land) — plus WPM's subject-rights operations (WPM-R30): worker
/// **erasure** and the retention **sweep**, both of which destroy or
/// irreversibly anonymise data and so must require `access=admin`.
/// A POST whose path ends with one of these derives
/// [`Action::Destructive`] instead of [`Action::Write`].
pub const DESTRUCTIVE_POST_SUFFIXES: [&str; 5] =
    ["/merge", "/deduplicate", "/import", "/erase", "/sweep"];

/// Whether blanket `/api/*` enforcement is on, read once from
/// `WPM_REQUIRE_AUTH` and cached. Off by default — see the
/// module docs and `agents/share/jwt-enforcement.md`. Mirrors
/// [`verifier`]: a process-wide `OnceLock` built from the environment.
#[must_use]
pub fn require_auth() -> bool {
    static REQUIRE_AUTH: OnceLock<bool> = OnceLock::new();
    *REQUIRE_AUTH
        .get_or_init(|| parse_bool(&crate::compat::env_var("WPM_REQUIRE_AUTH").unwrap_or_default()))
}

/// Lenient boolean parse: `1`/`true`/`yes`/`on` (case-insensitive,
/// surrounding whitespace ignored) ⇒ `true`; everything else
/// (incl. empty) ⇒ `false`.
#[must_use]
pub fn parse_bool(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

/// Paths that stay public even when enforcement is on: health/ping, the
/// `OpenAPI` doc + Swagger UI, and the Prometheus metrics endpoint (so a
/// scraper needs no bearer token). Everything else requires a valid bearer
/// token.
pub(crate) fn is_public_path(path: &str) -> bool {
    path == "/_health"
        || path == "/_ping"
        || path == "/api-docs/openapi.json"
        || path.starts_with("/swagger-ui")
        || path == "/metrics.prom"
}

/// Derive the request's ABAC action from its HTTP method and path (per
/// `authorization-attributes.md` §2): `GET`/`HEAD`/`OPTIONS` ⇒ `Read`;
/// `DELETE` ⇒ `Delete`; a `POST` whose path ends with a
/// [`DESTRUCTIVE_POST_SUFFIXES`] entry ⇒ `Destructive`; every other
/// `POST`/`PUT`/`PATCH` (and any unrecognised method) ⇒ `Write`.
#[must_use]
pub fn derive_action(method: &Method, path: &str) -> Action {
    // SEC-G6: normalise a trailing slash before the destructive-suffix
    // check, so `POST /api/wards/merge/` (and `//`) is still classified as
    // `Destructive` rather than silently downgraded to `Write` — which would
    // let an `access=write` (non-admin) caller reach a destructive op.
    let path = path.trim_end_matches('/');
    match *method {
        Method::GET | Method::HEAD | Method::OPTIONS => Action::Read,
        Method::DELETE => Action::Delete,
        Method::POST
            if DESTRUCTIVE_POST_SUFFIXES
                .iter()
                .any(|suffix| path.ends_with(suffix)) =>
        {
            Action::Destructive
        }
        _ => Action::Write,
    }
}

/// Load the ABAC policy: `WPM_ABAC_POLICY` (inline JSON) wins, then
/// `WPM_ABAC_POLICY_FILE` (path to a JSON file), else the built-in
/// default policy. A present-but-unparsable policy (bad JSON, unknown
/// effect/action names, unreadable file) `tracing::warn!`s and falls
/// back to the default — the service always boots, matching the
/// key-fetch posture. Read once per process via [`policy`]; restart to
/// change.
#[must_use]
pub fn policy_from_env() -> Policy {
    let source = crate::compat::env_var("WPM_ABAC_POLICY")
        .filter(|v| !v.trim().is_empty())
        .or_else(|| {
            let path = crate::compat::env_var("WPM_ABAC_POLICY_FILE")
                .filter(|v| !v.trim().is_empty())?;
            match std::fs::read_to_string(path.trim()) {
                Ok(contents) => Some(contents),
                Err(error) => {
                    tracing::warn!(%error, %path, "ABAC policy file unreadable; using the built-in default policy");
                    None
                }
            }
        });
    match source {
        Some(json) => parse_policy(&json).unwrap_or_else(Policy::default_policy),
        None => Policy::default_policy(),
    }
}

/// Parse a mounted ABAC policy, first migrating any pre-rename
/// `entity: "hcm"` condition to the current entity name
/// ([`crate::compat::migrate_policy_entity`]).
///
/// The migration matters because a stale entity condition fails
/// **silently**: the rule simply stops matching and the decision falls
/// through to the default, so a policy that used to deny could start
/// allowing with nothing in the logs. Rewriting it (with a deprecation
/// warning) keeps a mounted policy meaning what its author wrote.
///
/// Returns `None` — warn-logged — when the JSON is not a valid policy,
/// so the caller falls back to the built-in default and the service
/// still boots.
fn parse_policy(json: &str) -> Option<Policy> {
    // Migrate on the parsed JSON when it is well-formed; a policy that
    // does not even parse as JSON is handed to `Policy::from_json`
    // unchanged so its own error message is the one reported.
    let migrated = match serde_json::from_str::<serde_json::Value>(json) {
        Ok(mut value) => {
            crate::compat::migrate_policy_entity(&mut value);
            serde_json::to_string(&value).unwrap_or_else(|_| json.to_string())
        }
        Err(_) => json.to_string(),
    };
    match Policy::from_json(&migrated) {
        Ok(policy) => Some(policy),
        Err(error) => {
            tracing::warn!(%error, "ABAC policy JSON invalid; using the built-in default policy");
            None
        }
    }
}

/// The process-wide **hot-reloadable** ABAC policy, initialised from
/// `WPM_ABAC_POLICY` / `WPM_ABAC_POLICY_FILE` (else the built-in
/// default). Read the active snapshot with `policy().current()` per
/// request; swap it at runtime with [`reload_policy`] (e.g. from the
/// policy-file watcher in `app.rs`) — **no restart needed**.
#[must_use]
pub fn policy() -> &'static ReloadablePolicy {
    static POLICY: OnceLock<ReloadablePolicy> = OnceLock::new();
    POLICY.get_or_init(|| ReloadablePolicy::new(policy_from_env()))
}

/// Re-read the ABAC policy from the environment (`WPM_ABAC_POLICY` /
/// `WPM_ABAC_POLICY_FILE`, same rules and default-fallback as
/// [`policy_from_env`]) and swap it into the live [`policy`] holder.
/// Called by the policy-file watcher (`app.rs`) when the file changes;
/// a malformed policy falls back to the built-in default (never leaves
/// the service unprotected) and warn-logs. New requests see the new
/// policy immediately; in-flight ones finish against their snapshot.
pub fn reload_policy() {
    policy().store(policy_from_env());
    tracing::info!("ABAC policy reloaded");
}

/// How often the policy-file watcher polls for a change (seconds).
const POLICY_WATCH_SECS: u64 = 15;

/// Spawn a background task that hot-reloads the ABAC policy when the
/// `WPM_ABAC_POLICY_FILE` changes on disk — so operators can edit the
/// policy file and have it take effect without a restart. It polls the
/// file's mtime every [`POLICY_WATCH_SECS`] and calls [`reload_policy`]
/// on a change (mtime-poll, not an OS notify, to stay dependency-light).
///
/// A no-op when `WPM_ABAC_POLICY_FILE` is unset — an inline
/// `WPM_ABAC_POLICY` (or the built-in default) has nothing to watch,
/// and env vars do not change at runtime. Call once at boot
/// (`app.rs::after_routes`).
pub fn spawn_policy_watcher() {
    let Some(path) = crate::compat::env_var("WPM_ABAC_POLICY_FILE")
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
    else {
        return;
    };
    tokio::spawn(async move {
        let mut last = file_mtime(&path);
        let mut ticker = tokio::time::interval(std::time::Duration::from_secs(POLICY_WATCH_SECS));
        // The first tick fires immediately; skip it so we react only to
        // subsequent changes, not the initial state we already loaded.
        ticker.tick().await;
        loop {
            ticker.tick().await;
            let now = file_mtime(&path);
            if now != last {
                last = now;
                reload_policy();
            }
        }
    });
    tracing::info!(
        secs = POLICY_WATCH_SECS,
        "watching WPM_ABAC_POLICY_FILE for changes"
    );
}

/// The last-modified time of `path`, or `None` if it cannot be read
/// (missing file, permission error, or a platform without mtime).
fn file_mtime(path: &str) -> Option<std::time::SystemTime> {
    std::fs::metadata(path)
        .and_then(|meta| meta.modified())
        .ok()
}

/// Derive the **record-level resource attributes** of a stored
/// worker for the ABAC decision (`authorization-attributes.md` §9).
/// Keys a policy matches with `resource.<key>`:
///
/// | Resource key | From | Example tokens |
/// |---|---|---|
/// | `resource.person` | `Worker::person_ref` | the bare person uuid (enables `$sub` self-rules) |
/// | `resource.person_ref` | `Worker::person_ref` | the full `person:` URN |
/// | `resource.department` | `Worker::department` | `engineering`, `cardiology`, … |
/// | `resource.status` | `Worker::status` | `onboarding`, `active`, `terminated`, … |
/// | `resource.manager` | `Worker::manager_pid` | the manager's worker pid |
/// | `resource.organization_ref` | `Worker::organization_ref` | the full `organization:` URN |
///
/// A deployment can then write e.g. "allow read only when
/// `resource.department` is one of the caller's `dept` attributes",
/// "allow when `resource.person = $sub`" (self-service, WPM-R8), or
/// "allow a **masked** read otherwise" (an `allow` rule with the
/// `mask` obligation) — entirely as policy, no code change.
#[must_use]
pub fn worker_resource_attrs(worker: &workers::Model) -> BTreeMap<String, Vec<String>> {
    let mut attrs = BTreeMap::new();
    let person = worker
        .person_ref
        .split_once(':')
        .map_or(worker.person_ref.as_str(), |(_, id)| id);
    attrs.insert("person".to_string(), vec![person.to_string()]);
    attrs.insert("person_ref".to_string(), vec![worker.person_ref.clone()]);
    attrs.insert("department".to_string(), vec![worker.department.clone()]);
    attrs.insert("status".to_string(), vec![worker.status.clone()]);
    attrs.insert(
        "organization_ref".to_string(),
        vec![worker.organization_ref.clone()],
    );
    if let Some(manager_pid) = worker.manager_pid {
        attrs.insert("manager".to_string(), vec![manager_pid.to_string()]);
    }
    attrs
}

/// The placeholder shown for a redacted text field under the `mask`
/// obligation (spec `auth.md`).
pub const MASKED: &str = "\u{2022}\u{2022}\u{2022}";

/// Apply the `mask` obligation to a worker: redact the salary
/// (amount **and** currency). Employment facts — title, department,
/// dates, status — remain visible (WPM-R15: structure stays, money
/// goes).
#[must_use]
pub fn mask_worker(mut worker: workers::Model) -> workers::Model {
    worker.salary_minor = None;
    worker.salary_currency = None;
    worker
}

/// Apply the `mask` obligation to a payslip: zero the amounts and
/// drop the deduction lines, leaving the run/worker linkage (a
/// masked caller can see a payslip *exists*, not what it pays).
#[must_use]
pub fn mask_payslip(mut payslip: payslips::Model) -> payslips::Model {
    payslip.gross_minor = 0;
    payslip.net_minor = 0;
    payslip.deductions = serde_json::json!([]);
    payslip
}

/// Environment attributes for the current request, for the `env.*`
/// policy namespace (`authorization-attributes.md` §10). Derived here —
/// not in the pure engine — so the clock stays at the service edge and
/// the engine stays deterministic:
///
/// | Env key | Meaning |
/// |---|---|
/// | `env.hour` | Current UTC hour, `0`–`23`. |
/// | `env.after_hours` | `true` outside 08:00–17:59 UTC, else `false`. |
///
/// A deployment can then write e.g. "deny write when
/// `env.after_hours=true` unless `access=admin`". With no `env.*` rule
/// these attributes are inert.
#[must_use]
pub fn request_env_attrs() -> BTreeMap<String, Vec<String>> {
    use chrono::Timelike;
    env_attrs_at(chrono::Utc::now().hour())
}

/// Pure derivation of the environment attributes for a given UTC `hour`
/// (`0`–`23`), split out of [`request_env_attrs`] so it is unit-testable
/// without a clock.
#[must_use]
fn env_attrs_at(hour: u32) -> BTreeMap<String, Vec<String>> {
    let after_hours = !(8..18).contains(&hour);
    let mut env = BTreeMap::new();
    env.insert("hour".to_string(), vec![hour.to_string()]);
    env.insert("after_hours".to_string(), vec![after_hours.to_string()]);
    env
}

/// **Record-level** authorization for a handler that has loaded the
/// target record: evaluate the policy with the record's resource
/// attributes ([`stay_resource_attrs`] / [`ward_resource_attrs`]) and
/// the request's environment attributes
/// ([`request_env_attrs`]). A finer, second pass on top of the coarse
/// blanket guard — the guard already required a valid token and a
/// coarse (entity-level) allow before the handler ran; this refines the
/// decision with attributes of the *specific* record and the request
/// context.
///
/// Gated on the same `WPM_REQUIRE_AUTH` flag as the blanket guard:
/// when enforcement is **off** this is a no-op (behaviour-neutral, no
/// authn/authz); when **on**, the blanket guard guarantees a token, so
/// absent claims here are a `401` fail-safe.
///
/// # Errors
///
/// `401` if enforcement is on but no verified claims are present (should
/// not happen behind the blanket guard — fail safe). `403` if the policy
/// denies the action given the record's / request's attributes (the
/// message names the deciding rule).
/// On an **allow**, returns the decision's **obligations** — advisory
/// instructions the handler must honour, e.g. `["mask"]` (return the
/// masked view). When enforcement is off the vec is empty (no authz).
pub fn authorize_record(
    caller: &MaybeAuthUser,
    action: Action,
    resource: &BTreeMap<String, Vec<String>>,
) -> Result<Vec<String>, (StatusCode, String)> {
    if !require_auth() {
        return Ok(Vec::new());
    }
    let claims = caller
        .claims()
        .ok_or((StatusCode::UNAUTHORIZED, "missing bearer token".to_string()))?;
    let decision = policy().current().evaluate_with_context(
        claims,
        action,
        ENTITY,
        resource,
        &request_env_attrs(),
    );
    if decision.allowed {
        Ok(decision.obligations)
    } else {
        Err((StatusCode::FORBIDDEN, decision.reason))
    }
}

/// Read env var `name`, treating unset/blank as absent and falling back
/// to `default`. Used by both backends for issuer/audience-style
/// config so a blank value doesn't override the sensible default. Goes
/// through [`crate::compat::env_var`], so a deployment still setting
/// the pre-rename `HCM_*` spelling keeps working (with a deprecation
/// warning) instead of silently reverting to the default.
pub(crate) fn env_or(name: &str, default: &str) -> String {
    crate::compat::env_var(name).unwrap_or_else(|| default.to_string())
}

/// A request whose bearer token passed the active backend's signature /
/// issuer / audience / expiry verification (PASETO or Keycloak JWT, per
/// the `paseto`/`keycloak` feature). The wrapped [`Claims`] identify the
/// caller (`sub` is the user id). Taking this argument makes a handler
/// require authentication.
pub struct AuthUser(pub Claims);

/// Extracting an [`AuthUser`] verifies the request's bearer token against
/// the active backend's process-wide [`verifier`]; a missing/invalid
/// token rejects with `401` before the handler runs, so the type is the
/// "require auth" gate.
impl<S: Send + Sync> FromRequestParts<S> for AuthUser {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Snapshot the current (hot-reloadable) verifier for this request.
        let verifier = verifier().current();
        bearer_claims(&parts.headers, &verifier).map(AuthUser)
    }
}

/// Like [`AuthUser`], but never rejects: yields `Some(claims)` when a
/// valid bearer token is present and `None` otherwise. Handlers use it to
/// stamp the caller identity (e.g. the audit `actor`) without requiring
/// authentication on the route.
pub struct MaybeAuthUser(pub Option<Claims>);

impl MaybeAuthUser {
    /// The caller's `sub` (user `pid`) if a valid token was presented.
    #[must_use]
    pub fn actor(&self) -> Option<&str> {
        self.0.as_ref().map(|c| c.sub.as_str())
    }

    /// The verified [`Claims`] if a valid token was presented, for the
    /// record-level authorization pass ([`authorize_record`]).
    #[must_use]
    pub fn claims(&self) -> Option<&Claims> {
        self.0.as_ref()
    }
}

/// Extracting a [`MaybeAuthUser`] never rejects: a valid token yields
/// `Some(claims)`, anything else `None`. Handlers use it to opportunistically
/// stamp the caller identity without requiring authentication.
impl<S: Send + Sync> FromRequestParts<S> for MaybeAuthUser {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let verifier = verifier().current();
        Ok(Self(bearer_claims(&parts.headers, &verifier).ok()))
    }
}

/// Backend-agnostic pins: ABAC-policy evaluation, resource-attribute
/// derivation, masking, and the `env.*`/`derive_action`/`parse_bool`
/// helpers — none of which touch token verification, so these run
/// identically regardless of which of `paseto`/`keycloak` is compiled
/// in. Each backend's own token-verification round trip (signing,
/// `bearer_claims`, `enforce`) is pinned in that backend's own test
/// module instead (`src/auth/paseto.rs`, `src/auth/keycloak.rs`).
#[cfg(test)]
mod tests {
    use super::*;

    /// `parse_bool` accepts the documented truthy set and rejects the
    /// rest (including empty, `0`, and junk).
    #[test]
    fn parse_bool_truthy_and_falsy() {
        for t in ["1", "true", "TRUE", "Yes", "on", " on ", "ON"] {
            assert!(parse_bool(t), "{t:?} should parse true");
        }
        for f in ["", " ", "0", "false", "no", "off", "junk", "2"] {
            assert!(!parse_bool(f), "{f:?} should parse false");
        }
    }

    /// Action derivation (`authorization-attributes.md` §2): safe
    /// methods read; DELETE deletes; the crate's destructive named
    /// POSTs (merge / deduplicate / import) are destructive, not
    /// write; every other POST/PUT/PATCH writes.
    #[test]
    fn derive_action_matrix() {
        for method in [Method::GET, Method::HEAD, Method::OPTIONS] {
            assert_eq!(derive_action(&method, "/api/wards"), Action::Read);
        }
        assert_eq!(
            derive_action(&Method::DELETE, "/api/wards/1"),
            Action::Delete
        );
        for path in [
            "/api/wards/merge",
            "/api/wards/deduplicate",
            "/api/wards/import",
            "/api/workers/0c4f1e2a-0000-4000-8000-000000000001/erase",
            "/api/retention/sweep",
        ] {
            assert_eq!(derive_action(&Method::POST, path), Action::Destructive);
        }
        assert_eq!(derive_action(&Method::POST, "/api/wards"), Action::Write);
        assert_eq!(
            derive_action(&Method::POST, "/api/wards/check-duplicates"),
            Action::Write
        );
        assert_eq!(derive_action(&Method::PUT, "/api/wards/1"), Action::Write);
        assert_eq!(derive_action(&Method::PATCH, "/api/wards/1"), Action::Write);
        // GET on a destructive-suffixed path is still a read — only
        // POST consults the suffix list.
        assert_eq!(
            derive_action(&Method::GET, "/api/wards/merge"),
            Action::Read
        );
        // SEC-G6: a trailing slash must NOT downgrade a destructive POST to
        // Write (a non-admin `access=write` caller must not reach merge).
        for path in [
            "/api/wards/merge/",
            "/api/wards/merge//",
            "/api/wards/deduplicate/",
            "/api/wards/import/",
        ] {
            assert_eq!(
                derive_action(&Method::POST, path),
                Action::Destructive,
                "{path} must stay Destructive"
            );
        }
    }

    /// `policy_from_env` never breaks boot: bad policy JSON falls back
    /// to the built-in default policy (the pure fallback path
    /// `policy_from_env` takes on parse failure).
    #[test]
    fn policy_bad_json_falls_back_to_default() {
        assert!(Policy::from_json("{ not json").is_err());
        assert_eq!(
            Policy::from_json("{ not json").unwrap_or_else(|_| Policy::default_policy()),
            Policy::default_policy()
        );
    }

    /// Build `Claims` with the given subject attributes for the
    /// record-level decision tests (no signing needed).
    fn claims_with_attrs(attrs: &[(&str, &[&str])]) -> Claims {
        Claims {
            sub: "11111111-1111-1111-1111-111111111111".into(),
            email: "alice@example.com".into(),
            name: "Alice".into(),
            iss: "authentication-service".into(),
            aud: "main-x-service".into(),
            exp: 2_000_000_000,
            iat: 1_900_000_000,
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
        }
    }

    /// The environment-attribute derivation flags working vs after
    /// hours from a UTC hour (`authorization-attributes.md` §10).
    #[test]
    fn env_attrs_at_flags_after_hours() {
        for hour in 8..18 {
            let env = env_attrs_at(hour);
            assert_eq!(
                env["after_hours"],
                vec!["false".to_string()],
                "{hour}:00 is working hours"
            );
            assert_eq!(env["hour"], vec![hour.to_string()]);
        }
        for hour in [0, 7, 18, 22, 23] {
            assert_eq!(
                env_attrs_at(hour)["after_hours"],
                vec!["true".to_string()],
                "{hour}:00 is after hours"
            );
        }
    }

    /// A worker model for the resource-attribute tests.
    fn an_worker(
        person: uuid::Uuid,
        department: &str,
        status: &str,
    ) -> crate::models::_entities::workers::Model {
        crate::models::_entities::workers::Model {
            location: None,
            created_at: chrono::Utc::now().into(),
            updated_at: chrono::Utc::now().into(),
            id: 1,
            pid: uuid::Uuid::new_v4(),
            person_ref: format!("person:{person}"),
            upstream_worker_ref: None,
            organization_ref: format!("organization:{}", uuid::Uuid::new_v4()),
            worker_number: "E-1001".to_string(),
            display_name: "Ada Lovelace".to_string(),
            status: status.to_string(),
            employment_type: "permanent".to_string(),
            fte_percent: 100,
            department: department.to_string(),
            job_title: "Engineer".to_string(),
            manager_pid: None,
            salary_minor: Some(3_600_000),
            salary_currency: Some("GBP".to_string()),
            hired_on: chrono::NaiveDate::from_ymd_opt(2026, 1, 5).unwrap(),
            terminated_on: None,
            deleted_at: None,
        }
    }

    /// The record-level resource-attribute derivation maps an
    /// worker's person / department / status to the `resource.*`
    /// tokens a policy matches (`authorization-attributes.md` §9).
    #[test]
    fn worker_resource_attrs_maps_person_department_status() {
        let person = uuid::Uuid::new_v4();
        let attrs = worker_resource_attrs(&an_worker(person, "engineering", "active"));
        assert_eq!(attrs["person"], vec![person.to_string()]);
        assert_eq!(attrs["person_ref"], vec![format!("person:{person}")]);
        assert_eq!(attrs["department"], vec!["engineering".to_string()]);
        assert_eq!(attrs["status"], vec!["active".to_string()]);
        assert!(!attrs.contains_key("manager"), "no manager ⇒ no key");
    }

    /// `resource.organization_ref` (multi-organization, WPM-Rxx) is
    /// the worker's own organization URN verbatim, enabling an
    /// org-scoped policy rule the same way `resource.department`
    /// already does for department scoping.
    #[test]
    fn worker_resource_attrs_maps_organization_ref() {
        let worker = an_worker(uuid::Uuid::new_v4(), "engineering", "active");
        let org = worker.organization_ref.clone();
        let attrs = worker_resource_attrs(&worker);
        assert_eq!(attrs["organization_ref"], vec![org]);
    }

    /// The attributes drive the persona decisions through the shared
    /// engine: a `$sub` self-rule lets a worker read their own
    /// record, a department-scoped HR rule reads the department, and
    /// the mask-obligation fallback yields a masked read for others.
    #[test]
    fn worker_resource_attrs_drive_self_dept_and_masking() {
        let me = uuid::Uuid::new_v4();
        let policy = Policy::from_json(
            r#"{ "rules": [
                { "effect": "allow", "actions": ["read"], "when": { "resource.person": ["$sub"] } },
                { "effect": "allow", "actions": ["read"], "when": { "hr": ["true"], "resource.department": ["engineering"] } },
                { "effect": "allow", "actions": ["read"], "when": {}, "obligations": ["mask"] }
            ] }"#,
        )
        .expect("policy parses");
        // Self-read: the caller's sub equals the worker's person id.
        let mut own = claims_with_attrs(&[]);
        own.sub = me.to_string();
        let self_read = policy.evaluate_with_resource(
            &own,
            Action::Read,
            ENTITY,
            &worker_resource_attrs(&an_worker(me, "engineering", "active")),
        );
        assert!(self_read.allowed);
        assert!(self_read.obligations.is_empty());
        // Department-scoped HR read.
        let hr = claims_with_attrs(&[("hr", &["true"])]);
        let dept_read = policy.evaluate_with_resource(
            &hr,
            Action::Read,
            ENTITY,
            &worker_resource_attrs(&an_worker(uuid::Uuid::new_v4(), "engineering", "active")),
        );
        assert!(dept_read.allowed);
        assert!(dept_read.obligations.is_empty());
        // Anyone else falls through to the masked-read rule.
        let other = claims_with_attrs(&[]);
        let masked_read = policy.evaluate_with_resource(
            &other,
            Action::Read,
            ENTITY,
            &worker_resource_attrs(&an_worker(me, "finance", "active")),
        );
        assert!(masked_read.allowed, "falls through to the masked-read rule");
        assert!(masked_read.requires("mask"));
    }

    /// `mask_worker` redacts the salary and keeps employment facts;
    /// `mask_payslip` zeroes the money and drops the lines.
    #[test]
    fn mask_redacts_money_and_keeps_structure() {
        let masked = mask_worker(an_worker(uuid::Uuid::new_v4(), "engineering", "active"));
        assert!(masked.salary_minor.is_none());
        assert!(masked.salary_currency.is_none());
        assert_eq!(masked.department, "engineering");
        assert_eq!(masked.job_title, "Engineer");
        let slip = crate::models::_entities::payslips::Model {
            created_at: chrono::Utc::now().into(),
            updated_at: chrono::Utc::now().into(),
            id: 1,
            pid: uuid::Uuid::new_v4(),
            run_pid: uuid::Uuid::new_v4(),
            worker_pid: uuid::Uuid::new_v4(),
            currency: "GBP".to_string(),
            gross_minor: 300_000,
            deductions: serde_json::json!([{"label": "tax", "amount_minor": 39_050}]),
            net_minor: 260_950,
            deleted_at: None,
        };
        let slip = mask_payslip(slip);
        assert_eq!(slip.gross_minor, 0);
        assert_eq!(slip.net_minor, 0);
        assert_eq!(slip.deductions, serde_json::json!([]));
        assert_eq!(slip.currency, "GBP");
    }
}
