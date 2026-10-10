//! Security headers and rate limiting (WPM-R73, WPM-R74), plus the posture
//! probe (WPM-R72).
//!
//! Everything decisive is pure and clock-free so it can be tested: the header
//! set ([`security_headers`]), the limiter ([`RateLimiter::check`], which takes
//! the time as an argument) and the route classes ([`is_sensitive`]).
//!
//! **The limiter keeps no history** (WPM-D54). A caller is reduced to a 64-bit
//! hash of its network address; the hash and a counter live in memory for one
//! window and are never logged, so the limiter stores no address, no subject
//! and no token.
//!
//! **The key is the peer address, never a header the caller controls.** A
//! bearer token or `X-Forwarded-For` taken at face value would let a guesser
//! vary it on every request and never meet a limit. `X-Forwarded-For` is read
//! only when `WPM_TRUST_FORWARDED` is set, which a deployment behind a reverse
//! proxy that *overwrites* that header must do (otherwise every caller shares
//! the proxy's address, and one bucket).

use axum::{
    extract::{ConnectInfo, Request},
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use loco_rs::prelude::*;
use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    net::{IpAddr, SocketAddr},
    sync::{Mutex, OnceLock},
    time::{Instant, SystemTime},
};

/// The window the limits count over, in seconds.
pub const WINDOW_SECS: u64 = 60;
/// Default requests per window for an ordinary route.
pub const DEFAULT_LIMIT: u32 = 600;
/// Default requests per window for a sensitive route ([`is_sensitive`]).
pub const DEFAULT_SENSITIVE_LIMIT: u32 = 20;
/// Entries held before expired ones are pruned.
const PRUNE_AT: usize = 10_000;

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

/// A fixed-window counter per caller key. Not a token bucket: simple, and
/// enough to stop a loop or a guesser.
#[derive(Debug)]
pub struct RateLimiter {
    limit: u32,
    sensitive_limit: u32,
    /// key → (window start in seconds, count in that window)
    seen: HashMap<u64, (u64, u32)>,
}

/// The answer to one request.
#[derive(Debug, PartialEq, Eq)]
pub enum Verdict {
    /// Within the limit.
    Allow,
    /// Over the limit; try again after this many seconds.
    Deny {
        /// Seconds until the window ends.
        retry_after: u64,
    },
}

impl RateLimiter {
    /// A limiter. A limit of `0` disables that class.
    #[must_use]
    pub fn new(limit: u32, sensitive_limit: u32) -> Self {
        Self {
            limit,
            sensitive_limit,
            seen: HashMap::new(),
        }
    }

    /// Count one request from `key` at `now` (seconds), in the ordinary or
    /// the sensitive class.
    pub fn check(&mut self, key: u64, sensitive: bool, now: u64) -> Verdict {
        let limit = if sensitive {
            self.sensitive_limit
        } else {
            self.limit
        };
        if limit == 0 {
            return Verdict::Allow;
        }
        if self.seen.len() >= PRUNE_AT {
            self.seen
                .retain(|_, (start, _)| now.saturating_sub(*start) < WINDOW_SECS);
        }
        // A sensitive request is counted apart from ordinary ones.
        let slot = key ^ u64::from(sensitive);
        let entry = self.seen.entry(slot).or_insert((now, 0));
        if now.saturating_sub(entry.0) >= WINDOW_SECS {
            *entry = (now, 0);
        }
        if entry.1 >= limit {
            return Verdict::Deny {
                retry_after: WINDOW_SECS
                    .saturating_sub(now.saturating_sub(entry.0))
                    .max(1),
            };
        }
        entry.1 += 1;
        Verdict::Allow
    }
}

/// The caller reduced to a hash of its address: the first forwarded address
/// when `trust_forwarded` and the header is present, otherwise the peer
/// address, otherwise one shared bucket. Only the hash is kept.
#[must_use]
pub fn caller_key(peer: Option<IpAddr>, headers: &HeaderMap, trust_forwarded: bool) -> u64 {
    let forwarded = if trust_forwarded {
        headers
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.split(',').next().unwrap_or(v).trim().to_string())
            .filter(|v| !v.is_empty())
    } else {
        None
    };
    let basis = forwarded
        .or_else(|| peer.map(|ip| ip.to_string()))
        .unwrap_or_default();
    let mut hasher = DefaultHasher::new();
    basis.hash(&mut hasher);
    hasher.finish()
}

/// Whether `X-Forwarded-For` is trusted (`WPM_TRUST_FORWARDED`), read once.
fn trust_forwarded() -> bool {
    static TRUST: OnceLock<bool> = OnceLock::new();
    *TRUST.get_or_init(|| {
        crate::compat::env_var("WPM_TRUST_FORWARDED").is_some_and(|v| crate::auth::parse_bool(&v))
    })
}

fn limiter() -> &'static Mutex<RateLimiter> {
    static LIMITER: OnceLock<Mutex<RateLimiter>> = OnceLock::new();
    LIMITER.get_or_init(|| {
        let read = |name: &str, default: u32| {
            crate::compat::env_var(name)
                .and_then(|v| v.trim().parse::<u32>().ok())
                .unwrap_or(default)
        };
        Mutex::new(RateLimiter::new(
            read("WPM_RATE_LIMIT_PER_MINUTE", DEFAULT_LIMIT),
            read(
                "WPM_RATE_LIMIT_SENSITIVE_PER_MINUTE",
                DEFAULT_SENSITIVE_LIMIT,
            ),
        ))
    })
}

fn now_secs() -> u64 {
    static START: OnceLock<(Instant, u64)> = OnceLock::new();
    let (start, base) = START.get_or_init(|| {
        let base = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        (Instant::now(), base)
    });
    base + start.elapsed().as_secs()
}

/// Middleware: refuse with `429` and `Retry-After` when a caller is over its
/// limit. Health, ping and the posture probe are never limited.
pub async fn rate_limit_mw(req: Request, next: Next) -> Response {
    let path = req.uri().path();
    if matches!(path, "/_health" | "/_ping" | "/_posture") {
        return next.run(req).await;
    }
    let sensitive = is_sensitive(req.method(), path);
    let peer = req
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|info| info.0.ip());
    let key = caller_key(peer, req.headers(), trust_forwarded());
    let verdict = limiter()
        .lock()
        .map_or(Verdict::Allow, |mut l| l.check(key, sensitive, now_secs()));
    match verdict {
        Verdict::Allow => next.run(req).await,
        Verdict::Deny { retry_after } => (
            StatusCode::TOO_MANY_REQUESTS,
            [(header::RETRY_AFTER, retry_after.to_string())],
            "rate limit exceeded; retry later",
        )
            .into_response(),
    }
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
        assert!(!names("/_health", false).contains(&"cache-control".to_string()));
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

    #[test]
    fn the_limit_trips_then_the_window_resets() {
        let mut l = RateLimiter::new(3, 1);
        for _ in 0..3 {
            assert_eq!(l.check(7, false, 100), Verdict::Allow);
        }
        assert_eq!(
            l.check(7, false, 130),
            Verdict::Deny { retry_after: 30 },
            "fourth in the window is refused, with the time left"
        );
        assert_eq!(l.check(7, false, 160), Verdict::Allow, "a new window");
    }

    #[test]
    fn callers_and_classes_are_counted_apart() {
        let mut l = RateLimiter::new(1, 1);
        assert_eq!(l.check(1, false, 0), Verdict::Allow);
        assert_eq!(l.check(2, false, 0), Verdict::Allow, "another caller");
        assert_eq!(l.check(1, true, 0), Verdict::Allow, "the sensitive class");
        assert_ne!(l.check(1, false, 1), Verdict::Allow);
        assert_ne!(l.check(1, true, 1), Verdict::Allow);
    }

    #[test]
    fn a_zero_limit_disables_the_class() {
        let mut l = RateLimiter::new(0, 0);
        for _ in 0..1000 {
            assert_eq!(l.check(1, false, 0), Verdict::Allow);
            assert_eq!(l.check(1, true, 0), Verdict::Allow);
        }
    }

    #[test]
    fn the_caller_key_is_the_address_and_not_a_header_the_caller_controls() {
        let one: IpAddr = "203.0.113.1".parse().unwrap();
        let two: IpAddr = "203.0.113.2".parse().unwrap();
        let none = HeaderMap::new();
        // Different peers, different buckets; the same peer, the same bucket.
        assert_ne!(
            caller_key(Some(one), &none, false),
            caller_key(Some(two), &none, false)
        );
        assert_eq!(
            caller_key(Some(one), &none, false),
            caller_key(Some(one), &none, false)
        );
        // A token, or a forwarded address when not trusted, changes nothing:
        // varying either must not buy a fresh bucket.
        let mut spoof = HeaderMap::new();
        spoof.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("Bearer one"),
        );
        spoof.insert("x-forwarded-for", HeaderValue::from_static("198.51.100.9"));
        assert_eq!(
            caller_key(Some(one), &spoof, false),
            caller_key(Some(one), &none, false)
        );
    }

    #[test]
    fn a_trusted_proxy_header_names_the_caller_by_its_first_hop() {
        let proxy: IpAddr = "10.0.0.1".parse().unwrap();
        let mut a = HeaderMap::new();
        a.insert(
            "x-forwarded-for",
            HeaderValue::from_static("203.0.113.9, 10.0.0.1"),
        );
        let mut b = HeaderMap::new();
        b.insert("x-forwarded-for", HeaderValue::from_static("203.0.113.9"));
        let mut c = HeaderMap::new();
        c.insert("x-forwarded-for", HeaderValue::from_static("203.0.113.10"));
        assert_eq!(
            caller_key(Some(proxy), &a, true),
            caller_key(Some(proxy), &b, true)
        );
        assert_ne!(
            caller_key(Some(proxy), &a, true),
            caller_key(Some(proxy), &c, true)
        );
        // Empty header: fall back to the peer.
        let mut empty = HeaderMap::new();
        empty.insert("x-forwarded-for", HeaderValue::from_static(""));
        assert_eq!(
            caller_key(Some(proxy), &empty, true),
            caller_key(Some(proxy), &HeaderMap::new(), true)
        );
    }
}
