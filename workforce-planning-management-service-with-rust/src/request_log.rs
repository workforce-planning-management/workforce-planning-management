//! One log line per request, with a request id and nothing personal (WPM-R122, WPM-T197).
//!
//! Loco's own request logger records the whole URI, query string included, so a directory search for
//! a person's name or a filter on a department would be written to every log it reaches. It is switched
//! off in every profile and this replaces it. What a line holds:
//!
//! * `request_id`: the caller's `X-Request-Id` when it is plain (letters, digits, `-`, `_`, `@`, at most
//!   128 of them), else a new UUID; also set on the response, so a report can be matched to a line;
//! * `method`;
//! * `route`: the path with every segment that is not a plain lowercase word replaced by `{}`
//!   (`/api/workers/{}`), so no identifier, reference or name in a path reaches the log;
//! * `status` and `latency_ms`, once the response is made.
//!
//! The query string, the headers (so no token, address or user agent), the body and the caller are not read.

use std::time::Instant;

use axum::{
    extract::Request,
    http::{HeaderName, HeaderValue},
    middleware::Next,
    response::Response,
};
use tracing::Instrument;

const X_REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");
/// The longest caller-supplied id kept.
const MAX_ID_LEN: usize = 128;

/// The path with each identifier-like segment replaced by `{}`.
#[must_use]
pub fn route_label(path: &str) -> String {
    let plain = |segment: &str| {
        let mut chars = segment.chars();
        chars.next().is_some_and(|c| c.is_ascii_lowercase())
            && segment.len() <= 40
            && chars.all(|c| c.is_ascii_lowercase() || c == '-')
    };
    let mut out = String::new();
    for segment in path.split('/').filter(|s| !s.is_empty()) {
        out.push('/');
        out.push_str(if plain(segment) { segment } else { "{}" });
    }
    if out.is_empty() {
        out.push('/');
    }
    out
}

/// The caller's id when it is plain, else a new one.
#[must_use]
pub fn request_id_from(incoming: Option<&str>) -> String {
    match incoming {
        Some(id)
            if !id.is_empty()
                && id.len() <= MAX_ID_LEN
                && id
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '@')) =>
        {
            id.to_string()
        }
        _ => uuid::Uuid::new_v4().to_string(),
    }
}

/// The outermost layer: sets the id, opens the span, logs the finish.
pub async fn request_log_mw(mut request: Request, next: Next) -> Response {
    let id = request_id_from(
        request
            .headers()
            .get(&X_REQUEST_ID)
            .and_then(|v| v.to_str().ok()),
    );
    // a plain id is always a valid header value; the fallback keeps this free of a panic
    let value = HeaderValue::from_str(&id).unwrap_or_else(|_| HeaderValue::from_static("unknown"));
    request.headers_mut().insert(X_REQUEST_ID, value.clone());
    let span = tracing::info_span!(
        "http-request",
        request_id = %id,
        method = %request.method(),
        route = %route_label(request.uri().path()),
    );
    let started = Instant::now();
    let mut response = next.run(request).instrument(span.clone()).await;
    response.headers_mut().insert(X_REQUEST_ID, value);
    span.in_scope(|| {
        tracing::info!(
            status = response.status().as_u16(),
            latency_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
            "request finished"
        );
    });
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifiers_in_a_path_become_braces() {
        assert_eq!(route_label("/api/workers"), "/api/workers");
        assert_eq!(
            route_label("/api/workers/7b1c2f0e-5d3a-4c1b-9e8f-0a1b2c3d4e5f/status"),
            "/api/workers/{}/status"
        );
        assert_eq!(route_label("/api/audits/12345"), "/api/audits/{}");
        assert_eq!(route_label("/api/people/person:abc"), "/api/people/{}");
        assert_eq!(route_label("/api/skills/Ada%20Lovelace"), "/api/skills/{}");
        assert_eq!(route_label("/api/Directory"), "/api/{}");
        assert_eq!(route_label("/"), "/");
        assert_eq!(route_label(""), "/");
    }

    #[test]
    fn a_plain_request_id_is_kept_and_anything_else_is_replaced() {
        assert_eq!(request_id_from(Some("abc-123_X@y")), "abc-123_X@y");
        for bad in [
            "",
            "has space",
            "new\nline",
            "emoji-\u{1f600}",
            &"x".repeat(129),
        ] {
            let made = request_id_from(Some(bad));
            assert_ne!(made, bad);
            assert!(uuid::Uuid::parse_str(&made).is_ok(), "{made}");
        }
        assert!(uuid::Uuid::parse_str(&request_id_from(None)).is_ok());
    }
}
