//! **Errors are RFC 9457 problem details** (WPM-R238, WPM-D186): one conversion, so that no handler or layer has to know the
//! format and nothing that answers with an error can be missed.
//!
//! [`axa::Error`] still answers `{"error", "description"}`, and WPM's own layers answer in plain text. [`problem_mw`] rewrites
//! either as `application/problem+json`: `error` becomes the extension `code`, `description` becomes `detail`, and every other
//! member and every header stays. It runs twice (see [`crate::app::App::router`]), because `axa` compresses inside WPM's layers.
//! A body this does not recognise as an error is left exactly as it is: a JSON-RPC answer on `/mcp` is one.

use axa::problem::{PROBLEM_JSON, Problem};
use axum::{
    body::{Body, to_bytes},
    extract::Request,
    http::{HeaderMap, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::Value;

/// The most of an error body to read. Past it the words are dropped, not the status.
const MAX_ERROR_BODY: usize = 1024 * 1024;

/// The problem a response's `body` becomes, or `None` to leave it alone.
///
/// Plain text from a `5xx` is dropped, because nothing says it was written for a client (WPM-T259).
#[must_use]
pub fn from_body(status: StatusCode, content_type: Option<&str>, body: &[u8]) -> Option<Problem> {
    let media_type = content_type
        .and_then(|value| value.split(';').next())
        .map(|value| value.trim().to_ascii_lowercase());
    match media_type.as_deref() {
        _ if body.is_empty() => Some(Problem::new(status)),
        Some("application/json") => from_json(status, body),
        Some("text/plain") | None => {
            let text = std::str::from_utf8(body).ok().map(str::trim);
            let problem = Problem::new(status);
            Some(match text {
                Some(text) if !text.is_empty() && !status.is_server_error() => {
                    problem.with_detail(text)
                }
                _ => problem,
            })
        }
        Some(_) => None,
    }
}

/// `{"error", "description"}` and anything else beside them, or `None` for JSON that is not that shape.
fn from_json(status: StatusCode, body: &[u8]) -> Option<Problem> {
    let Value::Object(members) = serde_json::from_slice(body).ok()? else {
        return None;
    };
    if members.get("error").is_some_and(|error| !error.is_string())
        || !["error", "description", "errors"]
            .iter()
            .any(|name| members.contains_key(*name))
    {
        return None;
    }
    let mut problem = Problem::new(status);
    for (name, value) in members {
        problem = match (name.as_str(), value) {
            (_, Value::Null) => problem,
            ("error", code) => problem.with_extension("code", code),
            ("description", Value::String(detail)) => problem.with_detail(detail),
            (name, value) => problem.with_extension(name, value),
        };
    }
    Some(problem)
}

/// Whether a response could need converting. A compressed body cannot be read here, and the layer inside compression has had it.
fn may_convert(status: StatusCode, headers: &HeaderMap) -> bool {
    (status.is_client_error() || status.is_server_error())
        && !headers.contains_key(header::CONTENT_ENCODING)
}

/// `response`, as a problem if it is an error that is not one yet. Its status and headers stay.
async fn convert(response: Response) -> Response {
    let status = response.status();
    if !may_convert(status, response.headers()) {
        return response;
    }
    let (mut parts, body) = response.into_parts();
    let content_type = parts
        .headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    if content_type
        .as_deref()
        .is_some_and(|value| value.starts_with(PROBLEM_JSON))
    {
        return Response::from_parts(parts, body);
    }
    let (problem, original) = match to_bytes(body, MAX_ERROR_BODY).await {
        Ok(bytes) => (
            from_body(status, content_type.as_deref(), &bytes),
            Body::from(bytes),
        ),
        Err(_) => (Some(Problem::new(status)), Body::empty()),
    };
    let Some(problem) = problem else {
        return Response::from_parts(parts, original);
    };
    let (fresh, body) = problem.into_response().into_parts();
    parts.headers.remove(header::CONTENT_LENGTH);
    if let Some(media_type) = fresh.headers.get(header::CONTENT_TYPE) {
        parts
            .headers
            .insert(header::CONTENT_TYPE, media_type.clone());
    }
    Response::from_parts(parts, body)
}

/// The layer: every error leaves as a problem.
pub async fn problem_mw(request: Request, next: Next) -> Response {
    convert(next.run(request).await).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;
    use serde_json::json;

    const JSON: Option<&str> = Some("application/json");

    fn value(problem: Option<Problem>) -> Value {
        serde_json::to_value(problem.expect("a problem")).unwrap()
    }

    #[test]
    fn the_axa_body_becomes_a_problem() {
        let body = br#"{"error":"validation","description":"name is required"}"#;
        assert_eq!(
            value(from_body(StatusCode::UNPROCESSABLE_ENTITY, JSON, body)),
            json!({
                "type": "about:blank", "title": "Unprocessable Entity", "status": 422,
                "detail": "name is required", "code": "validation",
            })
        );
    }

    #[test]
    fn other_members_stay_beside_the_standard_ones() {
        let body = br#"{"error":"unsupported_api_version","requested":"9.0","supported":["1.0"]}"#;
        let problem = value(from_body(StatusCode::NOT_ACCEPTABLE, JSON, body));
        assert_eq!(problem["code"], "unsupported_api_version");
        assert_eq!(problem["requested"], "9.0");
        assert_eq!(problem["supported"], json!(["1.0"]));
        assert!(problem.get("detail").is_none());
        assert!(problem.get("error").is_none());
    }

    #[test]
    fn no_member_can_overwrite_a_standard_one() {
        let body = br#"{"error":"x","status":200,"title":"fine","type":"y","errors":null}"#;
        let problem = value(from_body(StatusCode::BAD_REQUEST, JSON, body));
        assert_eq!(problem["status"], 400);
        assert_eq!(problem["title"], "Bad Request");
        assert_eq!(problem["type"], "about:blank");
        assert!(problem.get("errors").is_none(), "a null says nothing");
    }

    #[test]
    fn field_errors_stay_as_they_were() {
        let body = br#"{"errors":{"name":["too short"]}}"#;
        let problem = value(from_body(StatusCode::BAD_REQUEST, JSON, body));
        assert_eq!(problem["errors"], json!({ "name": ["too short"] }));
    }

    #[test]
    fn plain_text_is_the_detail_of_a_client_error() {
        let problem = value(from_body(
            StatusCode::TOO_MANY_REQUESTS,
            Some("text/plain; charset=utf-8"),
            b"rate limit exceeded; retry later\n",
        ));
        assert_eq!(problem["status"], 429);
        assert_eq!(problem["detail"], "rate limit exceeded; retry later");
    }

    #[test]
    fn plain_text_from_a_server_error_does_not_reach_the_client() {
        let problem = value(from_body(
            StatusCode::INTERNAL_SERVER_ERROR,
            Some("text/plain"),
            b"relation \"workers\" does not exist",
        ));
        assert!(problem.get("detail").is_none());
        assert_eq!(problem["title"], "Internal Server Error");
    }

    #[test]
    fn an_empty_body_is_named_by_its_status() {
        for content_type in [None, JSON, Some("text/plain"), Some("text/html")] {
            let problem = value(from_body(StatusCode::REQUEST_TIMEOUT, content_type, b""));
            assert_eq!(problem["status"], 408);
            assert_eq!(problem["title"], "Request Timeout");
        }
    }

    #[test]
    fn a_body_that_is_not_an_error_is_left_alone() {
        let status = StatusCode::BAD_REQUEST;
        let rpc = br#"{"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":"no"}}"#;
        assert!(from_body(status, JSON, rpc).is_none(), "JSON-RPC");
        assert!(from_body(status, JSON, br#"{"pid":"1"}"#).is_none());
        assert!(from_body(status, JSON, b"[1]").is_none());
        assert!(from_body(status, JSON, b"not json").is_none());
        assert!(from_body(status, Some("text/html"), b"<p>no</p>").is_none());
    }

    #[test]
    fn invalid_text_gives_only_the_status() {
        let problem = value(from_body(StatusCode::BAD_REQUEST, None, &[0xff, 0xfe]));
        assert!(problem.get("detail").is_none());
    }

    #[test]
    fn success_and_compressed_bodies_are_not_touched() {
        let plain = HeaderMap::new();
        let mut compressed = HeaderMap::new();
        compressed.insert(header::CONTENT_ENCODING, HeaderValue::from_static("gzip"));
        assert!(may_convert(StatusCode::NOT_FOUND, &plain));
        assert!(may_convert(StatusCode::BAD_GATEWAY, &plain));
        assert!(!may_convert(StatusCode::OK, &plain));
        assert!(!may_convert(StatusCode::NOT_MODIFIED, &plain));
        assert!(!may_convert(StatusCode::NOT_FOUND, &compressed));
    }

    #[tokio::test]
    async fn the_status_and_headers_stay_and_the_length_goes() {
        let mut response = (StatusCode::CONFLICT, "still running").into_response();
        let headers = response.headers_mut();
        headers.insert(header::RETRY_AFTER, HeaderValue::from_static("1"));
        headers.insert(header::CONTENT_LENGTH, HeaderValue::from_static("13"));
        let response = convert(response).await;
        assert_eq!(response.status(), StatusCode::CONFLICT);
        assert_eq!(response.headers()[header::RETRY_AFTER], "1");
        assert_eq!(response.headers()[header::CONTENT_TYPE], PROBLEM_JSON);
        assert!(!response.headers().contains_key(header::CONTENT_LENGTH));
    }

    #[tokio::test]
    async fn a_second_pass_changes_nothing() {
        let once = convert((StatusCode::FORBIDDEN, "no").into_response()).await;
        let twice = convert(once).await;
        let bytes = to_bytes(twice.into_body(), MAX_ERROR_BODY).await.unwrap();
        let problem: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(problem["detail"], "no");
        assert_eq!(problem["status"], 403);
    }

    #[tokio::test]
    async fn a_success_passes_through_with_its_body() {
        let response = convert((StatusCode::OK, "fine").into_response()).await;
        let bytes = to_bytes(response.into_body(), MAX_ERROR_BODY)
            .await
            .unwrap();
        assert_eq!(&bytes[..], b"fine");
    }
}
