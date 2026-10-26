//! A read-only **Model Context Protocol** server's pure core (WPM-R149, WPM-D101), DB-free and clock-free. Spec:
//! `spec/integrations.md`.
//!
//! The server is an adapter. Every tool is one `GET` route of this API, called under the caller's own token so the API's
//! policy decides; there is no tool without a route, no write tool, and no service account. This module holds the tool
//! table and turns a JSON-RPC message into a [`Plan`]: a reply to send, an acknowledgement, or one API call to make.
//!
//! Pinned to the MCP revision [`PROTOCOL_VERSION`] (tools and Streamable HTTP pages read 2026-10-24). A later revision is a
//! change to this constant and these tests, not a surprise.

use serde_json::{Map, Value, json};

/// The one MCP revision this server speaks.
pub const PROTOCOL_VERSION: &str = "2025-11-25";

/// JSON-RPC error codes used.
const INVALID_REQUEST: i64 = -32600;
const METHOD_NOT_FOUND: i64 = -32601;
const INVALID_PARAMS: i64 = -32602;

/// One argument of a tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Arg {
    /// The argument name; also the query parameter, or the path parameter.
    pub name: &'static str,
    /// What it is.
    pub description: &'static str,
    /// How it is checked.
    pub kind: Kind,
    /// Whether the call needs it.
    pub required: bool,
}

/// What an argument may hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A `YYYY-MM-DD` date.
    Date,
    /// A UUID, which goes into the path.
    Pid,
    /// A URN such as `organization:<uuid>`.
    Urn,
    /// A whole number of 1 to 9999.
    Number,
}

/// A tool: one `GET` route.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tool {
    /// The tool name (letters, digits, underscore).
    pub name: &'static str,
    /// What it returns, for the model.
    pub description: &'static str,
    /// The route, with `{pid}` where a [`Kind::Pid`] argument goes.
    pub route: &'static str,
    /// The arguments.
    pub args: &'static [Arg],
}

const AS_OF: Arg = Arg {
    name: "as_of",
    description: "The date to compute for, YYYY-MM-DD; default today.",
    kind: Kind::Date,
    required: false,
};
const FROM: Arg = Arg {
    name: "from",
    description: "First day of the period, YYYY-MM-DD.",
    kind: Kind::Date,
    required: false,
};
const TO: Arg = Arg {
    name: "to",
    description: "Last day of the period, YYYY-MM-DD.",
    kind: Kind::Date,
    required: false,
};
const ORGANIZATION: Arg = Arg {
    name: "organization",
    description: "An organization URN, organization:<uuid>.",
    kind: Kind::Urn,
    required: true,
};
const PLAN: Arg = Arg {
    name: "pid",
    description: "The workforce plan's id (a UUID).",
    kind: Kind::Pid,
    required: true,
};

/// The tools. Each is a documented `GET` route; a test fails if one is not, or is not listed in the need-to-know table.
pub const TOOLS: &[Tool] = &[
    Tool {
        name: "my_contact_details",
        description: "The caller's own contact details, as the API shows them to the person themselves.",
        route: "/api/me/contact-details",
        args: &[],
    },
    Tool {
        name: "my_time_off",
        description: "The caller's own leave allowances, days taken, booked and remaining, in calendar days.",
        route: "/api/me/time-off",
        args: &[Arg {
            name: "year",
            description: "The leave year, e.g. 2026.",
            kind: Kind::Number,
            required: false,
        }],
    },
    Tool {
        name: "list_workforce_plans",
        description: "Workforce plans (draft worlds of aggregate headcount) in the caller's organizations.",
        route: "/api/workforce-plans",
        args: &[Arg {
            name: "organization",
            description: "Restrict to one organization URN.",
            kind: Kind::Urn,
            required: false,
        }],
    },
    Tool {
        name: "get_workforce_plan",
        description: "One workforce plan: its horizon, assumptions and demand lines.",
        route: "/api/workforce-plans/{pid}",
        args: &[PLAN],
    },
    Tool {
        name: "workforce_plan_forecast",
        description: "A plan's supply projection, headcount and FTE gap, and demand, with the assumptions it rests on.",
        route: "/api/workforce-plans/{pid}/forecast",
        args: &[PLAN],
    },
    Tool {
        name: "workforce_overview",
        description: "The shape of the workforce: headcount by department, status and employment type, as aggregates.",
        route: "/api/workforce-intelligence/overview",
        args: &[AS_OF],
    },
    Tool {
        name: "workforce_metrics",
        description: "Headcount, starters, leavers, turnover, span of control and time to fill for a period, each with its definition.",
        route: "/api/workforce-intelligence/metrics",
        args: &[FROM, TO],
    },
    Tool {
        name: "headcount_history",
        description: "Headcount snapshots over time, as aggregates.",
        route: "/api/workforce-intelligence/headcount-history",
        args: &[
            Arg {
                name: "organization",
                description: "Restrict to one organization URN.",
                kind: Kind::Urn,
                required: false,
            },
            FROM,
            TO,
        ],
    },
    Tool {
        name: "human_capital_report",
        description: "The dated human capital report for an organization: each figure with its definition, source, period and exclusions; small groups withheld; unknown where it cannot be computed.",
        route: "/api/reports/human-capital",
        args: &[ORGANIZATION, AS_OF],
    },
    Tool {
        name: "ai_register",
        description: "How people are scored: each scoring feature and what the deployer recorded about oversight, notice and logs; unknown where nothing is recorded.",
        route: "/api/ai-register",
        args: &[],
    },
];

/// The tool with this name.
#[must_use]
pub fn tool(name: &str) -> Option<&'static Tool> {
    TOOLS.iter().find(|t| t.name == name)
}

/// The route as the need-to-know table writes it: no `/api/` prefix and `{}` for a path parameter.
#[must_use]
pub fn table_route(t: &Tool) -> String {
    let mut out = String::new();
    let mut rest = t.route.trim_start_matches("/api/");
    while let Some(i) = rest.find('{') {
        out.push_str(&rest[..i]);
        out.push_str("{}");
        rest = rest[i..].split_once('}').map_or("", |(_, after)| after);
    }
    out.push_str(rest);
    out
}

fn schema_type(k: Kind) -> Value {
    match k {
        Kind::Date => json!({ "type": "string", "format": "date" }),
        Kind::Pid => json!({ "type": "string", "format": "uuid" }),
        Kind::Urn => json!({ "type": "string" }),
        Kind::Number => json!({ "type": "integer", "minimum": 1, "maximum": 9999 }),
    }
}

/// The tool as `tools/list` shows it.
#[must_use]
pub fn describe(t: &Tool) -> Value {
    let mut properties = Map::new();
    for a in t.args {
        let mut schema = schema_type(a.kind);
        schema["description"] = json!(a.description);
        properties.insert(a.name.to_string(), schema);
    }
    let required: Vec<&str> = t
        .args
        .iter()
        .filter(|a| a.required)
        .map(|a| a.name)
        .collect();
    json!({
        "name": t.name,
        "description": t.description,
        "inputSchema": { "type": "object", "properties": properties, "required": required, "additionalProperties": false },
        "annotations": { "readOnlyHint": true, "destructiveHint": false, "idempotentHint": true, "openWorldHint": false },
    })
}

fn valid(kind: Kind, v: &Value) -> Option<String> {
    match kind {
        Kind::Date => v
            .as_str()
            .filter(|s| chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d").is_ok())
            .map(str::to_string),
        Kind::Pid => v
            .as_str()
            .filter(|s| uuid::Uuid::parse_str(s).is_ok())
            .map(str::to_string),
        Kind::Urn => v
            .as_str()
            .filter(|s| {
                let Some((kind, id)) = s.split_once(':') else {
                    return false;
                };
                !kind.is_empty()
                    && kind.chars().all(|c| c.is_ascii_lowercase() || c == '_')
                    && uuid::Uuid::parse_str(id).is_ok()
            })
            .map(str::to_string),
        Kind::Number => v
            .as_i64()
            .filter(|n| (1..=9999).contains(n))
            .map(|n| n.to_string()),
    }
}

/// The path and query for a call, or why the arguments are wrong. Nothing outside the declared arguments gets through, and
/// every value has been checked to be a date, a UUID, a URN or a number, so none can carry a path or a query of its own.
///
/// # Errors
///
/// A message naming the argument at fault.
pub fn target(t: &Tool, arguments: &Value) -> Result<String, String> {
    let empty = Map::new();
    let given = match arguments {
        Value::Null => &empty,
        Value::Object(map) => map,
        _ => return Err("arguments must be an object".to_string()),
    };
    if let Some(unknown) = given
        .keys()
        .find(|k| !t.args.iter().any(|a| a.name == k.as_str()))
    {
        return Err(format!("{unknown} is not an argument of {}", t.name));
    }
    let mut path = t.route.to_string();
    let mut query = Vec::new();
    for a in t.args {
        match given.get(a.name) {
            None | Some(Value::Null) if a.required => {
                return Err(format!("{} is required", a.name));
            }
            None | Some(Value::Null) => {}
            Some(v) => {
                let value = valid(a.kind, v)
                    .ok_or_else(|| format!("{} is not a valid {}", a.name, kind_word(a.kind)))?;
                if a.kind == Kind::Pid {
                    path = path.replace(&format!("{{{}}}", a.name), &value);
                } else {
                    query.push(format!("{}={}", a.name, encode(&value)));
                }
            }
        }
    }
    Ok(if query.is_empty() {
        path
    } else {
        format!("{path}?{}", query.join("&"))
    })
}

fn kind_word(k: Kind) -> &'static str {
    match k {
        Kind::Date => "date (YYYY-MM-DD)",
        Kind::Pid => "id (a UUID)",
        Kind::Urn => "URN (kind:<uuid>)",
        Kind::Number => "whole number (1 to 9999)",
    }
}

/// Percent-encode everything that is not unreserved; a `:` in a URN becomes `%3A`.
fn encode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

/// What to do with one incoming JSON-RPC message.
#[derive(Debug, Clone, PartialEq)]
pub enum Plan {
    /// Send this body with HTTP 200.
    Reply(Value),
    /// A notification or a response: HTTP 202 with no body.
    Accepted,
    /// Call this route under the caller's token and wrap the answer for this request id.
    Call {
        /// The JSON-RPC id to answer.
        id: Value,
        /// The tool.
        tool: &'static Tool,
        /// The path and query.
        target: String,
    },
}

fn error(id: &Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

fn result(id: &Value, body: &Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": body })
}

/// Decide what to do with a parsed message.
#[must_use]
pub fn plan(message: &Value) -> Plan {
    let Some(object) = message.as_object() else {
        return Plan::Reply(error(
            &Value::Null,
            INVALID_REQUEST,
            "a message is one JSON object; batches are not supported",
        ));
    };
    if object.get("jsonrpc") != Some(&json!("2.0")) {
        return Plan::Reply(error(
            &Value::Null,
            INVALID_REQUEST,
            "jsonrpc must be \"2.0\"",
        ));
    }
    let method = object.get("method").and_then(Value::as_str);
    let id = object.get("id");
    let usable = id.filter(|i| i.is_string() || i.is_number());
    match (method, id, usable) {
        // A notification has a method and no id: nothing to answer.
        (Some(_), None, _) => Plan::Accepted,
        (Some(method), Some(_), Some(id)) => request(method, id, object.get("params")),
        // A response from the client answers something the server asked; this server asks nothing, so it only acknowledges.
        (None, Some(_), Some(_))
            if object.contains_key("result") || object.contains_key("error") =>
        {
            Plan::Accepted
        }
        _ => Plan::Reply(error(
            &Value::Null,
            INVALID_REQUEST,
            "a request has a method and an id that is a string or a number",
        )),
    }
}

fn request(method: &str, id: &Value, params: Option<&Value>) -> Plan {
    match method {
        "initialize" => Plan::Reply(result(
            id,
            &json!({
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": { "tools": { "listChanged": false } },
                "serverInfo": { "name": "workforce-planning-management", "version": env!("CARGO_PKG_VERSION") },
                "instructions": "Read-only. Every tool is one read of the WPM API under your own access: you see what you would see signed in, and a tool the API refuses you is refused. Aggregates under the small-group floor are withheld, and an unknown is never a zero.",
            }),
        )),
        "ping" => Plan::Reply(result(id, &json!({}))),
        "tools/list" => Plan::Reply(result(
            id,
            &json!({ "tools": TOOLS.iter().map(describe).collect::<Vec<_>>() }),
        )),
        "tools/call" => {
            let Some(name) = params.and_then(|p| p.get("name")).and_then(Value::as_str) else {
                return Plan::Reply(error(id, INVALID_PARAMS, "params.name is required"));
            };
            let Some(t) = tool(name) else {
                return Plan::Reply(error(id, INVALID_PARAMS, &format!("Unknown tool: {name}")));
            };
            let arguments = params
                .and_then(|p| p.get("arguments"))
                .unwrap_or(&Value::Null);
            match target(t, arguments) {
                Ok(target) => Plan::Call {
                    id: id.clone(),
                    tool: t,
                    target,
                },
                // An input a model can correct is a tool error, not a protocol one.
                Err(why) => Plan::Reply(result(id, &tool_error(&why))),
            }
        }
        _ => Plan::Reply(error(
            id,
            METHOD_NOT_FOUND,
            &format!("method not found: {method}"),
        )),
    }
}

/// A tool execution error: `isError` with a message the model can act on.
#[must_use]
pub fn tool_error(message: &str) -> Value {
    json!({ "content": [{ "type": "text", "text": message }], "isError": true })
}

/// Wrap what the API answered. A success carries the body as text, and as `structuredContent` when it is a JSON object; any
/// other status is a tool error naming the status, which is the API's answer to this caller and nothing more.
#[must_use]
pub fn answer(id: &Value, status: u16, body: &str) -> Value {
    if (200..300).contains(&status) {
        let mut out = json!({ "content": [{ "type": "text", "text": body }], "isError": false });
        if let Ok(parsed @ Value::Object(_)) = serde_json::from_str::<Value>(body) {
            out["structuredContent"] = parsed;
        }
        return result(id, &out);
    }
    result(
        id,
        &tool_error(&format!("The API answered {status}: {body}")),
    )
}

/// Whether an `MCP-Protocol-Version` header value is one this server speaks.
#[must_use]
pub fn supported_version(header: &str) -> bool {
    header == PROTOCOL_VERSION
}

/// Whether an `Origin` header may reach the server: absent is fine (an agent is not a browser); present, it must be listed.
#[must_use]
pub fn origin_allowed(origin: Option<&str>, allowed: &[String]) -> bool {
    origin.is_none_or(|o| allowed.iter().any(|a| a == o))
}

/// The path-aware well-known path of RFC 9728 for a protected resource: `https://host/mcp` is described at
/// `https://host/.well-known/oauth-protected-resource/mcp`, and a resource with no path at `.../oauth-protected-resource`.
///
/// # Errors
///
/// A message when `resource` is not an acceptable URL (https, or http to this machine, no query, fragment or credentials).
pub fn metadata_path(resource: &str) -> Result<String, String> {
    let resource =
        crate::rules::webhook::check_url(resource).map_err(|e| format!("the resource: {e}"))?;
    let (_, rest) = resource
        .split_once("://")
        .ok_or("the resource needs a scheme")?;
    let path = rest
        .find('/')
        .map_or("", |i| &rest[i..])
        .trim_end_matches('/');
    Ok(format!("/.well-known/oauth-protected-resource{path}"))
}

/// The URL of the metadata document, which a `401` points a client at (`resource_metadata`).
///
/// # Errors
///
/// As [`metadata_path`].
pub fn metadata_url(resource: &str) -> Result<String, String> {
    let path = metadata_path(resource)?;
    let resource = resource.trim();
    let (scheme, rest) = resource
        .split_once("://")
        .ok_or("the resource needs a scheme")?;
    let authority = rest.split('/').next().unwrap_or("");
    Ok(format!("{scheme}://{authority}{path}"))
}

/// The OAuth protected-resource metadata of RFC 9728 for the MCP endpoint: which resource this is and which authorization servers
/// issue tokens for it. Bearer tokens are accepted in the `Authorization` header only. It states what the deployer configured; WPM is
/// not an authorization server; `/mcp` itself refuses a token whose audience is not this resource ([`audience_ok`]).
///
/// # Errors
///
/// A message when the resource or a server URL is not acceptable, or no authorization server is named.
pub fn protected_resource(resource: &str, servers: &[String]) -> Result<Value, String> {
    metadata_path(resource)?;
    if servers.is_empty() {
        return Err("name at least one authorization server".to_string());
    }
    let mut issuers = Vec::new();
    for server in servers {
        issuers.push(
            crate::rules::webhook::check_url(server)
                .map_err(|e| format!("authorization server {server}: {e}"))?,
        );
    }
    Ok(json!({
        "resource": resource.trim(),
        "authorization_servers": issuers,
        "bearer_methods_supported": ["header"],
        "resource_name": "WPM MCP server",
    }))
}

/// Whether a token was issued for this resource (RFC 8707, and the MCP authorization section: a server accepts only tokens meant for
/// it, so a token another service holds cannot be replayed here). The two URLs are compared as written, after trimming spaces and one
/// trailing `/`; nothing is lower-cased or resolved, so a deployer must write the same URL in the token audience and in the resource.
#[must_use]
pub fn audience_ok(resource: &str, aud: &str) -> bool {
    let plain = |u: &str| u.trim().trim_end_matches('/').to_string();
    !plain(resource).is_empty() && plain(resource) == plain(aud)
}

/// The `WWW-Authenticate` challenge of a `401`: a bearer token is needed, and where the metadata is when it is configured.
#[must_use]
pub fn challenge(metadata_url: Option<&str>) -> String {
    metadata_url.map_or_else(
        || "Bearer".to_string(),
        |url| format!("Bearer resource_metadata=\"{url}\""),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(name: &str, arguments: &Value) -> Plan {
        plan(
            &json!({ "jsonrpc": "2.0", "id": 7, "method": "tools/call", "params": { "name": name, "arguments": arguments } }),
        )
    }

    fn reply(p: Plan) -> Value {
        match p {
            Plan::Reply(v) => v,
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn initialize_names_the_pinned_revision_and_only_tools() {
        let r = reply(plan(
            &json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {} }),
        ));
        assert_eq!(r["result"]["protocolVersion"], "2025-11-25");
        assert_eq!(
            r["result"]["capabilities"],
            json!({ "tools": { "listChanged": false } }),
            "no resources, prompts or sampling"
        );
        assert_eq!(r["id"], 1);
    }

    #[test]
    fn the_list_has_every_tool_marked_read_only_with_a_closed_schema() {
        let r = reply(plan(
            &json!({ "jsonrpc": "2.0", "id": "a", "method": "tools/list" }),
        ));
        let tools = r["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), TOOLS.len());
        for t in tools {
            assert_eq!(t["annotations"]["readOnlyHint"], true);
            assert_eq!(t["annotations"]["destructiveHint"], false);
            assert_eq!(t["inputSchema"]["additionalProperties"], false);
            assert!(
                t["name"]
                    .as_str()
                    .unwrap()
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_')
            );
        }
    }

    #[test]
    fn every_tool_is_one_get_route_with_a_unique_name_and_no_write_verb() {
        let names: std::collections::BTreeSet<&str> = TOOLS.iter().map(|t| t.name).collect();
        assert_eq!(names.len(), TOOLS.len());
        for t in TOOLS {
            assert!(t.route.starts_with("/api/"), "{}", t.name);
            for verb in [
                "create", "update", "delete", "set_", "put", "post", "record", "approve", "send",
            ] {
                assert!(!t.name.contains(verb), "{} reads like a write", t.name);
            }
        }
    }

    #[test]
    fn a_call_becomes_a_path_and_a_query_from_checked_values_only() {
        let pid = "11111111-2222-4333-8444-555555555555";
        assert_eq!(
            call("get_workforce_plan", &json!({ "pid": pid })),
            Plan::Call {
                id: json!(7),
                tool: tool("get_workforce_plan").unwrap(),
                target: format!("/api/workforce-plans/{pid}")
            }
        );
        let Plan::Call { target, .. } = call(
            "human_capital_report",
            &json!({ "organization": "organization:33333333-3333-4333-8333-333333333333", "as_of": "2026-10-24" }),
        ) else {
            panic!()
        };
        assert_eq!(
            target,
            "/api/reports/human-capital?organization=organization%3A33333333-3333-4333-8333-333333333333&as_of=2026-10-24"
        );
        let Plan::Call { target, .. } = call("my_time_off", &json!({})) else {
            panic!()
        };
        assert_eq!(target, "/api/me/time-off");
    }

    #[test]
    fn nothing_can_steer_a_call_to_another_route() {
        // A path traversal, a query smuggled in a value, an extra argument and a wrong type are all refused as tool errors.
        for (name, args) in [
            ("get_workforce_plan", json!({ "pid": "../../workers" })),
            (
                "get_workforce_plan",
                json!({ "pid": "11111111-2222-4333-8444-555555555555/../x" }),
            ),
            (
                "workforce_overview",
                json!({ "as_of": "2026-10-24&organization=x" }),
            ),
            ("workforce_overview", json!({ "as_of": 20_261_024 })),
            ("workforce_overview", json!({ "route": "/api/workers" })),
            (
                "human_capital_report",
                json!({ "organization": "organization:x\n/../workers" }),
            ),
            ("human_capital_report", json!({ "as_of": "2026-10-24" })),
            ("my_time_off", json!({ "year": 0 })),
            ("my_time_off", json!([])),
        ] {
            let r = reply(call(name, &args));
            assert_eq!(r["result"]["isError"], true, "{name} {args}");
        }
    }

    #[test]
    fn an_unknown_tool_or_method_is_a_protocol_error() {
        assert_eq!(
            reply(call("delete_worker", &json!({})))["error"]["code"],
            -32602
        );
        assert_eq!(
            reply(plan(
                &json!({ "jsonrpc": "2.0", "id": 1, "method": "resources/list" })
            ))["error"]["code"],
            -32601
        );
        assert_eq!(
            reply(plan(
                &json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {} })
            ))["error"]["code"],
            -32602
        );
    }

    #[test]
    fn notifications_and_responses_are_accepted_without_a_body_and_batches_are_not() {
        assert_eq!(
            plan(&json!({ "jsonrpc": "2.0", "method": "notifications/initialized" })),
            Plan::Accepted
        );
        assert_eq!(
            plan(&json!({ "jsonrpc": "2.0", "id": 3, "result": {} })),
            Plan::Accepted
        );
        assert_eq!(
            reply(plan(
                &json!([{ "jsonrpc": "2.0", "id": 1, "method": "ping" }])
            ))["error"]["code"],
            -32600
        );
        assert_eq!(
            reply(plan(&json!({ "id": 1, "method": "ping" })))["error"]["code"],
            -32600
        );
        assert_eq!(
            reply(plan(
                &json!({ "jsonrpc": "2.0", "id": null, "method": "ping" })
            ))["error"]["code"],
            -32600
        );
        assert_eq!(
            reply(plan(
                &json!({ "jsonrpc": "2.0", "id": 1, "method": "ping" })
            ))["result"],
            json!({})
        );
    }

    #[test]
    fn the_api_answer_is_wrapped_as_it_is_and_a_refusal_stays_a_refusal() {
        let ok = answer(&json!(9), 200, r#"{"headcount":12}"#);
        assert_eq!(ok["result"]["isError"], false);
        assert_eq!(
            ok["result"]["structuredContent"],
            json!({ "headcount": 12 })
        );
        assert_eq!(ok["result"]["content"][0]["text"], r#"{"headcount":12}"#);
        let list = answer(&json!(9), 200, "[1,2]");
        assert!(
            list["result"].get("structuredContent").is_none(),
            "structured content is an object"
        );
        for status in [401, 403, 404, 422] {
            let refused = answer(&json!(9), status, "no");
            assert_eq!(refused["result"]["isError"], true);
            assert!(
                refused["result"]["content"][0]["text"]
                    .as_str()
                    .unwrap()
                    .contains(&status.to_string())
            );
        }
    }

    #[test]
    fn the_table_route_matches_the_need_to_know_spelling() {
        assert_eq!(
            table_route(tool("workforce_plan_forecast").unwrap()),
            "workforce-plans/{}/forecast"
        );
        assert_eq!(table_route(tool("my_time_off").unwrap()), "me/time-off");
    }

    /// WPM-D101: no tool without an endpoint. Every tool's route is a documented `GET` of the `OpenAPI` document and is
    /// classified in the need-to-know table, so the API decides who may read it; its arguments are parameters of that
    /// operation, not something the adapter invented.
    #[test]
    fn every_tool_maps_to_a_documented_get_route_that_is_classified() {
        let document = crate::openapi::spec();
        for t in TOOLS {
            let operation = &document["paths"][t.route]["get"];
            assert!(
                operation.is_object(),
                "{} ({}) is not a documented GET",
                t.name,
                t.route
            );
            let route = table_route(t);
            assert!(
                crate::rules::access::READS.iter().any(|(r, _)| *r == route),
                "{} ({route}) is not in the need-to-know table",
                t.name
            );
        }
    }

    /// Nothing the table says only privileged callers may read is hidden from them or opened to others by the adapter: the
    /// classification is the API's, so a tool over a privileged route is still refused to a caller who is not privileged. The
    /// request tests prove the refusal; this proves the tools do not add a route that has none.
    #[test]
    fn no_tool_route_is_a_write_or_an_export() {
        for t in TOOLS {
            assert!(
                !t.route.contains("csv")
                    && !t.route.contains("export")
                    && !t.route.contains("download"),
                "{}",
                t.name
            );
        }
    }

    #[test]
    fn a_token_is_for_this_resource_only_when_its_audience_is_the_resource() {
        let r = "https://wpm.example/mcp";
        assert!(audience_ok(r, "https://wpm.example/mcp"));
        assert!(
            audience_ok(r, " https://wpm.example/mcp/ "),
            "spaces and one trailing slash"
        );
        assert!(audience_ok("https://wpm.example/mcp/", r));
        for other in [
            "main-x-service",
            "https://wpm.example",
            "https://wpm.example/api",
            "https://other.example/mcp",
            "HTTPS://WPM.EXAMPLE/mcp",
            "",
        ] {
            assert!(!audience_ok(r, other), "{other:?}");
        }
        assert!(
            !audience_ok("", ""),
            "no resource and no audience is not a match"
        );
    }

    #[test]
    fn origin_and_version_checks() {
        assert!(origin_allowed(None, &[]), "an agent sends no Origin");
        assert!(!origin_allowed(Some("https://evil.example"), &[]));
        assert!(origin_allowed(
            Some("https://app.example"),
            &["https://app.example".to_string()]
        ));
        assert!(
            supported_version("2025-11-25")
                && !supported_version("2025-06-18")
                && !supported_version("")
        );
    }

    #[test]
    fn the_metadata_path_follows_the_resource_path_as_rfc_9728_says() {
        assert_eq!(
            metadata_path("https://wpm.example/mcp").unwrap(),
            "/.well-known/oauth-protected-resource/mcp"
        );
        assert_eq!(
            metadata_path("https://wpm.example/").unwrap(),
            "/.well-known/oauth-protected-resource"
        );
        assert_eq!(
            metadata_path("https://wpm.example").unwrap(),
            "/.well-known/oauth-protected-resource"
        );
        assert_eq!(
            metadata_path("https://wpm.example/a/mcp/").unwrap(),
            "/.well-known/oauth-protected-resource/a/mcp"
        );
        assert_eq!(
            metadata_url("https://wpm.example/mcp").unwrap(),
            "https://wpm.example/.well-known/oauth-protected-resource/mcp"
        );
        assert_eq!(
            metadata_url("http://localhost:5150/mcp").unwrap(),
            "http://localhost:5150/.well-known/oauth-protected-resource/mcp"
        );
        for bad in [
            "http://wpm.example/mcp",
            "https://wpm.example/mcp?x=1",
            "https://wpm.example/mcp#f",
            "wpm.example/mcp",
            "https://user@wpm.example/mcp",
        ] {
            assert!(metadata_path(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn the_metadata_names_the_resource_and_its_authorization_servers_and_nothing_it_does_not_know()
    {
        let doc = protected_resource(
            "https://wpm.example/mcp",
            &["https://login.example/realms/wpm".to_string()],
        )
        .unwrap();
        assert_eq!(doc["resource"], "https://wpm.example/mcp");
        assert_eq!(
            doc["authorization_servers"],
            json!(["https://login.example/realms/wpm"])
        );
        assert_eq!(doc["bearer_methods_supported"], json!(["header"]));
        let keys: Vec<&str> = doc
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(
            keys,
            [
                "authorization_servers",
                "bearer_methods_supported",
                "resource",
                "resource_name"
            ],
            "no scopes or keys WPM does not define"
        );
        assert!(
            protected_resource("https://wpm.example/mcp", &[]).is_err(),
            "a resource with no authorization server is not described"
        );
        assert!(
            protected_resource(
                "https://wpm.example/mcp",
                &["http://login.example".to_string()]
            )
            .is_err()
        );
        assert!(
            protected_resource(
                "http://wpm.example/mcp",
                &["https://login.example".to_string()]
            )
            .is_err()
        );
    }

    #[test]
    fn a_challenge_points_at_the_metadata_when_there_is_some() {
        assert_eq!(challenge(None), "Bearer");
        assert_eq!(
            challenge(Some(
                "https://wpm.example/.well-known/oauth-protected-resource/mcp"
            )),
            "Bearer resource_metadata=\"https://wpm.example/.well-known/oauth-protected-resource/mcp\""
        );
    }
}
