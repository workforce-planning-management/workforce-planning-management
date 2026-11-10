//! Security headers and rate limiting (WPM-R73, WPM-R74), plus the posture
//! probe (WPM-R72).
//!
//! Everything decisive is pure and clock-free so it can be tested: the header
//! set ([`security_headers`]), the route classes ([`is_sensitive`]) and the
//! caller ([`caller`]). The counting is `axa::ratelimit`'s, a fixed window per
//! caller, with its own tests (WPM-D188).
//!
//! **The limiter keeps no history** (WPM-D54). A caller is held only as a keyed
//! hash of its network address, for at most two windows, and is never logged,
//! so the limiter stores no address, no subject and no token.
//!
//! **The key is the client's address as `axa` resolves it, never a header the
//! caller controls** (WPM-R239, WPM-D187). A bearer token, or the left end of
//! `X-Forwarded-For`, would let a guesser vary it on every request and never
//! meet a limit. [`axa::client::resolve`] believes that header only from a peer
//! in `TRUSTED_PROXIES`, and walks it from the right, so behind a proxy that
//! list must be set, or every caller shares the proxy's address and one bucket.
//!
//! [`rate_limit_mw`] resolves the address itself, with the same list. It runs
//! outside `axa::middleware::apply`, because a limit has to run before sign-in
//! refuses, so the address `apply` records is not on the request yet.

use axa::{
    client::ProxyNet,
    prelude::*,
    ratelimit::{EXEMPT, Limiter, client_key},
};
use axum::{
    extract::{ConnectInfo, Request, State},
    http::{Extensions, HeaderMap, HeaderName, HeaderValue, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::{
    net::{IpAddr, SocketAddr},
    num::NonZeroU32,
    sync::Arc,
};

/// The window the limits count over, in seconds.
pub const WINDOW_SECS: NonZeroU32 = NonZeroU32::new(60).unwrap();
/// Default requests per window for an ordinary route.
pub const DEFAULT_LIMIT: u32 = 600;
/// Default requests per window for a sensitive route ([`is_sensitive`]).
pub const DEFAULT_SENSITIVE_LIMIT: u32 = 20;

/// The window a lockout counts `401`s over, and the longest it lasts: 15 minutes (WPM-R242).
pub const LOCKOUT_WINDOW_SECS: NonZeroU32 = NonZeroU32::new(900).unwrap();

/// The failed sign-in lockout `LOCKOUT_CLIENT_ADDRESS_AFTER_COUNT` sets (WPM-R242, WPM-D190): a client address
/// that earns that many `401`s in [`LOCKOUT_WINDOW_SECS`] is refused with `429` until the window ends. Unset,
/// empty or `0` is no lockout. Anything else that is not a whole number is an error, so a typo does not leave
/// the lockout silently off; the message does not quote the value.
///
/// # Errors
/// When the value is not a whole number.
pub fn lockout_from(value: Option<&str>) -> std::result::Result<Option<Limiter>, String> {
    let Some(value) = value.map(str::trim).filter(|v| !v.is_empty()) else {
        return Ok(None);
    };
    let count: u32 = value.parse().map_err(|_| {
        "LOCKOUT_CLIENT_ADDRESS_AFTER_COUNT is not a whole number; set a count of 401s, or 0 for no lockout"
            .to_string()
    })?;
    Ok(NonZeroU32::new(count).map(|count| Limiter::new(count, LOCKOUT_WINDOW_SECS)))
}

/// The content security policy for JSON responses: nothing may load, and
/// nothing may frame the response.
const API_CSP: &str = "default-src 'none'; frame-ancestors 'none'";
/// The policy for the Swagger UI, which needs its own scripts and styles.
const DOCS_CSP: &str = "default-src 'self'; script-src 'self' 'unsafe-inline'; \
                        style-src 'self' 'unsafe-inline'; img-src 'self' data:; \
                        frame-ancestors 'none'";

/// Whether a path is the interactive docs, which need a looser policy.
fn is_docs(path: &str) -> bool {
    path.starts_with("/swagger-ui")
}

/// The security headers for a response to `path`. `https` adds
/// `Strict-Transport-Security`. Personal data is never cached: every `/api`
/// response is `no-store`.
#[must_use]
pub fn security_headers(path: &str, https: bool) -> Vec<(HeaderName, HeaderValue)> {
    let mut out = vec![
        (
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static(if is_docs(path) { DOCS_CSP } else { API_CSP }),
        ),
        (
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ),
        (
            header::REFERRER_POLICY,
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ),
        (
            HeaderName::from_static("permissions-policy"),
            HeaderValue::from_static(
                "camera=(), microphone=(), geolocation=(), payment=(), usb=()",
            ),
        ),
        (
            HeaderName::from_static("cross-origin-opener-policy"),
            HeaderValue::from_static("same-origin"),
        ),
        (
            HeaderName::from_static("cross-origin-resource-policy"),
            HeaderValue::from_static("same-origin"),
        ),
    ];
    if https {
        out.push((
            header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=63072000; includeSubDomains"),
        ));
    }
    if path == "/api" || path.starts_with("/api/") {
        out.push((header::CACHE_CONTROL, HeaderValue::from_static("no-store")));
    }
    out
}

/// Whether the request arrived over TLS, as a proxy reports it, or HSTS was
/// forced on with `WPM_HSTS`.
fn is_https(headers: &HeaderMap) -> bool {
    headers
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.eq_ignore_ascii_case("https"))
        || crate::compat::env_var("WPM_HSTS").is_some_and(|v| crate::auth::parse_bool(&v))
}

/// Middleware: set the security headers on every response, including errors.
pub async fn security_headers_mw(req: Request, next: Next) -> Response {
    let path = req.uri().path().to_string();
    let https = is_https(req.headers());
    let mut response = next.run(req).await;
    let headers = response.headers_mut();
    for (name, value) in security_headers(&path, https) {
        // A handler's own `Cache-Control` wins; every other header is ours.
        if name == header::CACHE_CONTROL && headers.contains_key(&name) {
            continue;
        }
        headers.insert(name, value);
    }
    response
}

/// Routes that destroy, export, bulk-change or mint tokens get the stricter
/// limit.
#[must_use]
pub fn is_sensitive(method: &axum::http::Method, path: &str) -> bool {
    if *method == axum::http::Method::GET {
        return path.ends_with("/export") || path.contains("/subject-access");
    }
    [
        "/erase",
        "/sweep",
        "/import",
        "/merge",
        "/deduplicate",
        "/token",
    ]
    .iter()
    .any(|suffix| path.ends_with(suffix))
}

/// The two limits, one per class of route ([`is_sensitive`]), counted apart,
/// and the proxies whose `X-Forwarded-For` is believed. `axa::ratelimit`
/// counts; a class whose setting is `0` has no limiter and is not limited.
#[derive(Debug, Clone)]
pub struct RateLimits {
    ordinary: Option<Limiter>,
    sensitive: Option<Limiter>,
    trusted: Vec<ProxyNet>,
}

impl RateLimits {
    /// `limit` and `sensitive_limit` requests per caller in each window of
    /// [`WINDOW_SECS`]. A limit of `0` disables that class.
    #[must_use]
    pub fn new(limit: u32, sensitive_limit: u32, trusted: Vec<ProxyNet>) -> Self {
        let limiter = |limit| NonZeroU32::new(limit).map(|limit| Limiter::new(limit, WINDOW_SECS));
        Self {
            ordinary: limiter(limit),
            sensitive: limiter(sensitive_limit),
            trusted,
        }
    }

    /// The limits `WPM_RATE_LIMIT_PER_MINUTE` and
    /// `WPM_RATE_LIMIT_SENSITIVE_PER_MINUTE` set, or the defaults.
    #[must_use]
    pub fn from_env(trusted: Vec<ProxyNet>) -> Self {
        let read = |name: &str, default: u32| {
            crate::compat::env_var(name)
                .and_then(|v| v.trim().parse::<u32>().ok())
                .unwrap_or(default)
        };
        Self::new(
            read("WPM_RATE_LIMIT_PER_MINUTE", DEFAULT_LIMIT),
            read(
                "WPM_RATE_LIMIT_SENSITIVE_PER_MINUTE",
                DEFAULT_SENSITIVE_LIMIT,
            ),
            trusted,
        )
    }

    /// The limiter for a request in this class, if the class is limited.
    fn class(&self, sensitive: bool) -> Option<&Limiter> {
        if sensitive {
            self.sensitive.as_ref()
        } else {
            self.ordinary.as_ref()
        }
    }
}

/// The caller as it is counted: the `address` resolved from the connection
/// and `TRUSTED_PROXIES`, reduced by [`axa::ratelimit::client_key`]. An
/// IPv4-mapped address counts as IPv4, and IPv6 counts by its `/64`, the block
/// one customer is routinely given. With no address, every caller shares one
/// count. The limiter keeps only a keyed hash of it.
#[must_use]
pub fn caller(address: Option<IpAddr>) -> Option<IpAddr> {
    let mut extensions = Extensions::new();
    if let Some(address) = address {
        extensions.insert(address);
    }
    client_key(&extensions)
}

/// What to log when the retired `WPM_TRUST_FORWARDED` is still set. It does
/// nothing now, and a deployment that relies on it needs `TRUSTED_PROXIES`.
#[must_use]
pub fn trust_forwarded_warning(setting: Option<&str>) -> Option<&'static str> {
    setting.map(|_| {
        "WPM_TRUST_FORWARDED does nothing and TRUSTED_PROXIES replaces it; \
         behind a proxy, list it there or every caller shares its rate limit"
    })
}

/// Middleware: refuse with `429`, `Retry-After` and the `RateLimit-*`
/// headers when a caller is over its limit, and stamp those headers on what
/// it lets through. Health, ping, readiness and the posture probe are never
/// limited.
pub async fn rate_limit_mw(
    State(limits): State<Arc<RateLimits>>,
    req: Request,
    next: Next,
) -> Response {
    let address = req
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|info| axa::client::resolve(info.0.ip(), req.headers(), &limits.trusted));
    // The lockout inside counts by the address `axa` keeps on the request, which `axa::middleware::apply` has not
    // put there yet (WPM-D190).
    let mut req = req;
    if let Some(address) = address {
        req.extensions_mut().insert(address);
    }
    let path = req.uri().path();
    if path == "/_posture" || EXEMPT.contains(&path) {
        return next.run(req).await;
    }
    let Some(limiter) = limits.class(is_sensitive(req.method(), path)) else {
        return next.run(req).await;
    };
    let verdict = limiter.check(caller(address));
    if !verdict.allowed {
        return verdict.refusal();
    }
    let mut response = next.run(req).await;
    verdict.stamp(&mut response);
    response
}

/// `GET /_posture` — whether sign-in is enforced. Public, so a client can
/// warn when it is not. It states nothing else.
#[debug_handler]
async fn posture() -> Response {
    format::json(serde_json::json!({ "auth_enforced": crate::auth::require_auth() }))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

/// The posture route, mounted at the root.
pub fn routes() -> Routes {
    Routes::new().add("/_posture", get(posture))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Method;

    #[test]
    fn the_lockout_is_off_unless_a_count_is_set_and_a_bad_count_stops_the_start() {
        for off in [None, Some(""), Some("  "), Some("0")] {
            assert!(lockout_from(off).unwrap().is_none(), "{off:?}");
        }
        assert!(lockout_from(Some("5")).unwrap().is_some());
        assert!(lockout_from(Some(" 10 ")).unwrap().is_some());
        for bad in ["five", "-1", "2.5", "99999999999"] {
            let message = lockout_from(Some(bad)).unwrap_err();
            assert!(!message.contains(bad), "{message} quotes the value");
        }
    }

    fn names(path: &str, https: bool) -> Vec<String> {
        security_headers(path, https)
            .into_iter()
            .map(|(n, _)| n.as_str().to_string())
            .collect()
    }

    #[test]
    fn every_response_carries_the_baseline_headers() {
        let n = names("/api/workers", false);
        for h in [
            "content-security-policy",
            "x-content-type-options",
            "referrer-policy",
            "permissions-policy",
            "cross-origin-opener-policy",
            "cross-origin-resource-policy",
            "cache-control",
        ] {
            assert!(n.contains(&h.to_string()), "{h} missing from {n:?}");
        }
        assert!(!n.contains(&"strict-transport-security".to_string()));
    }

    #[test]
    fn hsts_only_over_tls_and_no_store_only_on_the_api() {
        assert!(names("/api/x", true).contains(&"strict-transport-security".to_string()));
        assert!(!names("/_health/live", false).contains(&"cache-control".to_string()));
    }

    #[test]
    fn the_docs_get_a_looser_policy_but_still_no_framing() {
        let csp = |p: &str| {
            security_headers(p, false)
                .into_iter()
                .find(|(n, _)| n == header::CONTENT_SECURITY_POLICY)
                .unwrap()
                .1
                .to_str()
                .unwrap()
                .to_string()
        };
        assert!(csp("/api/x").starts_with("default-src 'none'"));
        assert!(csp("/swagger-ui/").contains("script-src 'self'"));
        assert!(csp("/swagger-ui/").contains("frame-ancestors 'none'"));
        assert!(csp("/api/x").contains("frame-ancestors 'none'"));
    }

    #[test]
    fn sensitive_routes_are_the_destructive_and_export_ones() {
        assert!(is_sensitive(&Method::POST, "/api/workers/1/erase"));
        assert!(is_sensitive(&Method::POST, "/api/privacy/sweep"));
        assert!(is_sensitive(&Method::POST, "/api/x/import"));
        assert!(is_sensitive(&Method::GET, "/api/workers/1/subject-access"));
        assert!(!is_sensitive(&Method::GET, "/api/workers"));
        assert!(!is_sensitive(&Method::POST, "/api/workers"));
    }

    fn limits(limit: u32, sensitive: u32) -> RateLimits {
        RateLimits::new(limit, sensitive, Vec::new())
    }

    #[test]
    fn the_classes_are_counted_apart() {
        let l = limits(1, 1);
        let key = caller(None);
        assert!(l.class(false).unwrap().check(key).allowed);
        assert!(
            l.class(true).unwrap().check(key).allowed,
            "the sensitive class"
        );
        assert!(!l.class(false).unwrap().check(key).allowed);
        assert!(!l.class(true).unwrap().check(key).allowed);
    }

    #[test]
    fn the_limits_are_the_settings_per_window() {
        let l = limits(3, 1);
        let ordinary = l.class(false).unwrap().check(caller(None));
        assert_eq!(ordinary.limit, 3);
        assert_eq!(ordinary.reset_seconds, u64::from(WINDOW_SECS.get()));
        assert_eq!(l.class(true).unwrap().check(caller(None)).limit, 1);
    }

    #[test]
    fn a_zero_limit_disables_the_class() {
        let l = limits(0, 5);
        assert!(l.class(false).is_none());
        assert!(l.class(true).is_some());
        assert!(limits(5, 0).class(true).is_none());
    }

    fn address(text: &str) -> IpAddr {
        text.parse().unwrap()
    }

    #[test]
    fn the_caller_is_the_address() {
        let one = caller(Some(address("203.0.113.1")));
        assert_eq!(one, Some(address("203.0.113.1")));
        assert_ne!(one, caller(Some(address("203.0.113.2"))));
    }

    #[test]
    fn with_no_address_every_caller_shares_one_bucket() {
        assert_eq!(caller(None), None);
        assert_ne!(caller(None), caller(Some(address("203.0.113.1"))));
    }

    #[test]
    fn an_ipv4_mapped_address_is_the_ipv4_address() {
        assert_eq!(
            caller(Some(address("::ffff:203.0.113.9"))),
            caller(Some(address("203.0.113.9")))
        );
    }

    #[test]
    fn ipv6_counts_by_its_block() {
        let block = caller(Some(address("2001:db8:1:2::1")));
        assert_eq!(block, caller(Some(address("2001:db8:1:2:ffff::9"))));
        assert_ne!(block, caller(Some(address("2001:db8:1:3::1"))));
    }

    #[test]
    fn the_key_comes_from_axa_and_a_forged_entry_changes_nothing() {
        // The caller wrote the first entry, and the proxy appended the second.
        let forwarded = |value: &'static str| {
            let mut headers = HeaderMap::new();
            headers.insert("x-forwarded-for", HeaderValue::from_static(value));
            headers
        };
        let proxy = address("10.0.0.1");
        let trusted = ["10.0.0.0/8".parse().unwrap()];
        let key = |headers: &HeaderMap, trusted: &[ProxyNet]| {
            caller(Some(axa::client::resolve(proxy, headers, trusted)))
        };
        let real = forwarded("203.0.113.9");
        let forged = forwarded("198.51.100.1, 203.0.113.9");
        assert_eq!(key(&real, &trusted), key(&forged, &trusted));
        assert_eq!(key(&real, &trusted), caller(Some(address("203.0.113.9"))));
        assert_ne!(
            key(&real, &trusted),
            key(&forwarded("203.0.113.10"), &trusted)
        );
        // A peer nobody listed is the caller, whatever it sends.
        assert_eq!(key(&real, &[]), caller(Some(proxy)));
    }

    #[test]
    fn the_retired_setting_warns_and_names_its_replacement() {
        assert_eq!(trust_forwarded_warning(None), None);
        for setting in ["1", "true", "0"] {
            let warning = trust_forwarded_warning(Some(setting)).unwrap();
            assert!(warning.contains("does nothing"), "{warning}");
            assert!(warning.contains("TRUSTED_PROXIES"), "{warning}");
        }
    }
}
