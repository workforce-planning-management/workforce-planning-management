//! A read-only Model Context Protocol server (WPM-R149, WPM-D101) at `POST /mcp`: Streamable HTTP, JSON replies, no session
//! and no server-sent events. Spec: `spec/integrations.md`.
//!
//! A tool call is made **in process through the API's own router**, with the caller's `Authorization` header and the same
//! sign-in and need-to-know layers as any request, so the API decides who may read what. There is no service account, no
//! cache across callers and no write tool. Each call is audited with the tool name before it is made; if the audit cannot be
//! written the call is refused (WPM-D107).

use axum::Router;
use axum::body::{Body, Bytes};
use axum::extract::State;
use axum::http::{HeaderMap, Request, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use loco_rs::prelude::AppContext;
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;

use crate::auth::MaybeAuthUser;
use crate::models::audit_logs::Model as Audit;
use crate::rules::mcp::{self, Plan};

/// The largest API answer passed on to a model.
const MAX_ANSWER_BYTES: usize = 2 * 1024 * 1024;

/// What the endpoint needs: the guarded API, and the context to audit with.
#[derive(Clone)]
pub struct Mcp {
    api: Router,
    ctx: AppContext,
}

fn allowed_origins() -> Vec<String> {
    crate::compat::env_var("WPM_MCP_ALLOWED_ORIGINS")
        .map(|v| {
            v.split(',')
                .map(|o| o.trim().to_string())
                .filter(|o| !o.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

fn json_status(status: StatusCode, body: &Value) -> Response {
    (status, axum::Json(body.clone())).into_response()
}

fn failure(status: StatusCode, code: i64, message: &str) -> Response {
    json_status(
        status,
        &json!({ "jsonrpc": "2.0", "id": null, "error": { "code": code, "message": message } }),
    )
}

/// Make the call through the API, as the caller.
async fn through_the_api(api: Router, target: &str, headers: &HeaderMap) -> (u16, String) {
    let mut request = Request::builder().method("GET").uri(target);
    if let Some(token) = headers.get(header::AUTHORIZATION) {
        request = request.header(header::AUTHORIZATION, token);
    }
    let Ok(request) = request.body(Body::empty()) else {
        return (500, "could not form the request".to_string());
    };
    let Ok(response) = api.oneshot(request).await;
    let status = response.status().as_u16();
    match axum::body::to_bytes(response.into_body(), MAX_ANSWER_BYTES).await {
        Ok(bytes) => (status, String::from_utf8_lossy(&bytes).into_owned()),
        Err(_) => (502, "the answer was too large to pass on".to_string()),
    }
}

/// `POST /mcp` — one JSON-RPC message.
async fn handle(
    State(mcp): State<Mcp>,
    caller: MaybeAuthUser,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let origin = headers.get(header::ORIGIN).and_then(|v| v.to_str().ok());
    if !mcp::origin_allowed(origin, &allowed_origins()) {
        return failure(
            StatusCode::FORBIDDEN,
            -32600,
            "this Origin may not use the MCP endpoint",
        );
    }
    if let Some(version) = headers
        .get("mcp-protocol-version")
        .and_then(|v| v.to_str().ok())
        && !mcp::supported_version(version)
    {
        return failure(
            StatusCode::BAD_REQUEST,
            -32600,
            &format!(
                "unsupported MCP-Protocol-Version; this server speaks {}",
                mcp::PROTOCOL_VERSION
            ),
        );
    }
    let Ok(message) = serde_json::from_slice::<Value>(&body) else {
        return failure(StatusCode::BAD_REQUEST, -32700, "the body is not JSON");
    };
    match mcp::plan(&message) {
        Plan::Reply(reply) => json_status(StatusCode::OK, &reply),
        Plan::Accepted => StatusCode::ACCEPTED.into_response(),
        Plan::Call { id, tool, target } => {
            // The audit comes first: a call that cannot be recorded is not made.
            if let Err(error) = Audit::record(
                &mcp.ctx.db,
                "mcp",
                Uuid::nil(),
                "mcp_tool_called",
                caller.actor(),
                Some(json!({ "tool": tool.name })),
            )
            .await
            {
                tracing::error!(%error, "mcp: the audit could not be written, so the call was refused");
                return json_status(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    &json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32603, "message": "internal error" } }),
                );
            }
            let (status, answer) = through_the_api(mcp.api.clone(), &target, &headers).await;
            json_status(StatusCode::OK, &mcp::answer(&id, status, &answer))
        }
    }
}

/// What the deployer configured to describe this resource: `WPM_MCP_RESOURCE` (the URL of the MCP endpoint as clients reach it) and
/// `WPM_MCP_AUTHORIZATION_SERVERS` (comma-separated issuer URLs). `None` until both are set.
fn resource_settings() -> Option<(String, Vec<String>)> {
    let resource = crate::compat::env_var("WPM_MCP_RESOURCE").filter(|r| !r.trim().is_empty())?;
    let servers: Vec<String> = crate::compat::env_var("WPM_MCP_AUTHORIZATION_SERVERS")?
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    (!servers.is_empty()).then_some((resource, servers))
}

/// `GET /.well-known/oauth-protected-resource` and `.../mcp` — the OAuth protected-resource metadata of RFC 9728. Public, like a
/// verifier's key: it says where tokens come from, and holds nothing about a person. 404 until the deployer configures it.
async fn protected_resource_metadata() -> Response {
    let Some((resource, servers)) = resource_settings() else {
        return json_status(
            StatusCode::NOT_FOUND,
            &json!({ "error": "not_configured", "description": "this deployment has not set WPM_MCP_RESOURCE and WPM_MCP_AUTHORIZATION_SERVERS" }),
        );
    };
    match mcp::protected_resource(&resource, &servers) {
        Ok(document) => json_status(StatusCode::OK, &document),
        Err(why) => {
            tracing::error!(%why, "mcp: the protected-resource settings are not usable");
            json_status(
                StatusCode::INTERNAL_SERVER_ERROR,
                &json!({ "error": "misconfigured", "description": "the protected-resource settings are not usable" }),
            )
        }
    }
}

/// A `401` carries `WWW-Authenticate: Bearer` (RFC 6750), and on the MCP endpoint, where the metadata is (`resource_metadata`, as the
/// MCP authorization section asks). An answer that already has the header is left alone.
pub async fn challenge_mw(request: Request<Body>, next: Next) -> Response {
    let on_mcp = request.uri().path() == "/mcp";
    let mut response = next.run(request).await;
    if response.status() == StatusCode::UNAUTHORIZED
        && !response.headers().contains_key(header::WWW_AUTHENTICATE)
    {
        let url = on_mcp
            .then(|| {
                resource_settings().and_then(|(resource, _)| mcp::metadata_url(&resource).ok())
            })
            .flatten();
        if let Ok(value) = header::HeaderValue::from_str(&mcp::challenge(url.as_deref())) {
            response
                .headers_mut()
                .insert(header::WWW_AUTHENTICATE, value);
        }
    }
    response
}

/// Add `POST /mcp` to the router. `api` is the same router with the sign-in and need-to-know layers on it, which the tool
/// calls go through. The endpoint itself needs a valid token when sign-in is enforced; the per-call decision is the API's.
pub fn mount(router: Router, api: Router, ctx: AppContext) -> Router {
    router.merge(
        Router::new()
            .route("/mcp", post(handle))
            .route(
                "/.well-known/oauth-protected-resource",
                get(protected_resource_metadata),
            )
            .route(
                "/.well-known/oauth-protected-resource/mcp",
                get(protected_resource_metadata),
            )
            .with_state(Mcp { api, ctx }),
    )
}
