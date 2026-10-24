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
use axum::response::{IntoResponse, Response};
use axum::routing::post;
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

/// Add `POST /mcp` to the router. `api` is the same router with the sign-in and need-to-know layers on it, which the tool
/// calls go through. The endpoint itself needs a valid token when sign-in is enforced; the per-call decision is the API's.
pub fn mount(router: Router, api: Router, ctx: AppContext) -> Router {
    router.merge(
        Router::new()
            .route("/mcp", post(handle))
            .with_state(Mcp { api, ctx }),
    )
}
