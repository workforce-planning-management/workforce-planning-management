//! Exact monetary integers at the JSON boundary (WPM-T552, WPM-D206).
//!
//! PostgreSQL and Rust hold signed 64-bit integers. Browser JSON consumers
//! hold exact integers only through 2^53 − 1. This middleware refuses a body
//! or query value named `*_minor` outside that shared range before a handler
//! can store or calculate with it.

use axum::{
    body::{Body, to_bytes},
    extract::Request,
    http::{Method, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::Value;

// The application's existing HTTP body limit (`app.rs`), repeated because
// this layer has to buffer and restore JSON before the extractor sees it.
const MAX_BODY: usize = 2 * 1024 * 1024;

fn outside(number: &serde_json::Number) -> bool {
    number.to_string().parse::<i128>().map_or(true, |value| {
        value.unsigned_abs() > u128::from(crate::rules::currency::MAX_EXACT_MINOR.unsigned_abs())
    })
}

fn invalid_minor(value: &Value) -> Option<&str> {
    match value {
        Value::Array(items) => items.iter().find_map(invalid_minor),
        Value::Object(fields) => fields.iter().find_map(|(name, value)| {
            if name.ends_with("_minor") {
                match value {
                    Value::Null => None,
                    Value::Number(number) if !outside(number) => None,
                    _ => Some(name.as_str()),
                }
            } else {
                invalid_minor(value)
            }
        }),
        _ => None,
    }
}

fn payload_outside(value: &Value) -> bool {
    match value {
        Value::Number(number) => outside(number),
        Value::Array(items) => items.iter().any(payload_outside),
        Value::Object(fields) => fields.values().any(payload_outside),
        _ => false,
    }
}

fn invalid_response_minor(value: &Value) -> Option<&str> {
    match value {
        Value::Array(items) => items.iter().find_map(invalid_response_minor),
        Value::Object(fields) => fields.iter().find_map(|(name, value)| {
            if name.ends_with("_minor") {
                match value {
                    Value::Null => None,
                    Value::Number(number) if !outside(number) => None,
                    // Aggregate figures are typed wrappers such as
                    // `{ "status": "shown", "value": 123 }`. Check every number in the
                    // wrapper; its value no longer carries the `_minor` suffix itself.
                    Value::Array(_) | Value::Object(_) if !payload_outside(value) => None,
                    _ => Some(name.as_str()),
                }
            } else {
                invalid_response_minor(value)
            }
        }),
        _ => None,
    }
}

fn invalid_query(query: &str) -> Option<&str> {
    query.split('&').find_map(|pair| {
        let (name, value) = pair.split_once('=')?;
        if name.ends_with("_minor")
            && value.parse::<i128>().is_ok_and(|number| {
                number.unsigned_abs()
                    > u128::from(crate::rules::currency::MAX_EXACT_MINOR.unsigned_abs())
            })
        {
            Some(name)
        } else {
            None
        }
    })
}

fn refused(field: &str) -> Response {
    (
        StatusCode::UNPROCESSABLE_ENTITY,
        format!(
            "{field} must be between -{} and {} so JSON keeps it exact",
            crate::rules::currency::MAX_EXACT_MINOR,
            crate::rules::currency::MAX_EXACT_MINOR
        ),
    )
        .into_response()
}

/// Reject an out-of-range `*_minor` JSON or query integer.
pub async fn exact_minor_mw(req: Request, next: Next) -> Response {
    if !req.uri().path().starts_with("/api/") {
        return next.run(req).await;
    }
    if let Some(field) = req.uri().query().and_then(invalid_query) {
        return refused(field);
    }
    let is_json = req
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("application/json"));
    if !is_json
        || matches!(
            req.method(),
            &Method::GET | &Method::HEAD | &Method::OPTIONS
        )
    {
        return exact_response(next.run(req).await).await;
    }
    let (parts, body) = req.into_parts();
    let Ok(bytes) = to_bytes(body, MAX_BODY).await else {
        return (StatusCode::PAYLOAD_TOO_LARGE, "request body too large").into_response();
    };
    if let Ok(value) = serde_json::from_slice::<Value>(&bytes)
        && let Some(field) = invalid_minor(&value)
    {
        return refused(field);
    }
    exact_response(
        next.run(Request::from_parts(parts, Body::from(bytes)))
            .await,
    )
    .await
}

async fn exact_response(response: Response) -> Response {
    if !response.status().is_success()
        || !response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value.starts_with("application/json"))
    {
        return response;
    }
    let (parts, body) = response.into_parts();
    let Ok(bytes) = to_bytes(body, usize::MAX).await else {
        tracing::error!("could not read a JSON response at the exact-money boundary");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    if let Ok(value) = serde_json::from_slice::<Value>(&bytes)
        && let Some(field) = invalid_response_minor(&value)
    {
        tracing::error!(field, "refused an inexact minor-unit JSON response");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    Response::from_parts(parts, Body::from(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, routing::post};
    use serde_json::json;
    use tower::ServiceExt;

    #[test]
    fn nested_minor_units_stop_at_the_shared_json_limit() {
        assert_eq!(
            invalid_minor(&json!({ "amount_minor": 9_007_199_254_740_991_i64 })),
            None
        );
        assert_eq!(
            invalid_minor(&json!({ "lines": [{ "amount_minor": 9_007_199_254_740_992_i64 }] })),
            Some("amount_minor")
        );
        assert_eq!(
            invalid_minor(&json!({ "amount_minor": -9_007_199_254_740_992_i64 })),
            Some("amount_minor")
        );
        assert_eq!(
            invalid_minor(&json!({ "ordinary_integer": i64::MAX })),
            None
        );
    }

    #[test]
    fn response_figures_check_their_nested_value() {
        assert_eq!(
            invalid_response_minor(&json!({
                "female_average_minor": { "status": "shown", "value": 9_000_000 }
            })),
            None
        );
        assert_eq!(
            invalid_response_minor(&json!({
                "female_average_minor": { "status": "withheld" }
            })),
            None
        );
        assert_eq!(
            invalid_response_minor(&json!({
                "female_average_minor": {
                    "status": "shown", "value": 9_007_199_254_740_992_i64
                }
            })),
            Some("female_average_minor")
        );
        assert_eq!(
            invalid_minor(&json!({
                "female_average_minor": { "status": "shown", "value": 9_000_000 }
            })),
            Some("female_average_minor"),
            "writes keep the scalar field contract"
        );
    }

    #[test]
    fn query_minor_units_use_the_same_limit() {
        assert_eq!(invalid_query("band=7&salary_minor=9007199254740991"), None);
        assert_eq!(
            invalid_query("band=7&salary_minor=9007199254740992"),
            Some("salary_minor")
        );
    }

    #[tokio::test]
    async fn middleware_refuses_an_unsafe_write_before_the_handler() {
        let app = Router::new()
            .route("/api/exact", post(|| async { StatusCode::NO_CONTENT }))
            .layer(axum::middleware::from_fn(exact_minor_mw));
        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/exact")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(r#"{"amount_minor":9007199254740992}"#))
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn middleware_refuses_an_unsafe_read_before_json_changes_it() {
        let app = Router::new()
            .route(
                "/api/exact",
                axum::routing::get(|| async {
                    (
                        [(header::CONTENT_TYPE, "application/json")],
                        r#"{"amount_minor":9007199254740992}"#,
                    )
                }),
            )
            .layer(axum::middleware::from_fn(exact_minor_mw));
        let request = Request::builder()
            .uri("/api/exact")
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
