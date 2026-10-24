//! The read-only MCP server (WPM-R149, WPM-D101): it speaks the pinned revision, calls the API as the caller, adds no tool
//! without a route, and refuses a call whose audit cannot be written (WPM-D107).

use sea_orm::ConnectionTrait;
use serde_json::{Value, json};
use serial_test::serial;

async fn rpc(request: &loco_rs::TestServer, body: Value) -> (u16, Value) {
    let response = request.post("/mcp").json(&body).await;
    let status = response.status_code().as_u16();
    let text = response.text();
    (status, serde_json::from_str(&text).unwrap_or(Value::Null))
}

#[tokio::test]
#[serial]
#[ignore = "requires PostgreSQL (config/test.yaml); run with `cargo test -- --ignored`"]
#[allow(clippy::too_many_lines)] // one scenario read top to bottom
async fn tools_are_the_api_reads_and_a_call_that_cannot_be_audited_is_not_made() {
    crate::requests::request_open(|request, ctx| async move {
        let (status, init) = rpc(&request, json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {} })).await;
        assert_eq!(status, 200);
        assert_eq!(init["result"]["protocolVersion"], "2025-11-25");

        let (_, list) = rpc(&request, json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" })).await;
        let names: Vec<&str> = list["result"]["tools"].as_array().unwrap().iter().map(|t| t["name"].as_str().unwrap()).collect();
        assert!(names.contains(&"human_capital_report") && names.contains(&"my_time_off"), "{names:?}");

        // A tool is a read of the API: the answer is the route's own.
        let (_, ai) = rpc(&request, json!({ "jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": { "name": "ai_register", "arguments": {} } })).await;
        assert_eq!(ai["id"], 3);
        assert_eq!(ai["result"]["isError"], false, "{ai}");
        let direct: Value = request.get("/api/ai-register").await.json();
        assert_eq!(ai["result"]["structuredContent"], direct);

        // The API's refusals are the tool's: a bad id is the route's 4xx, not something the adapter hides.
        let (_, missing) = rpc(&request, json!({ "jsonrpc": "2.0", "id": 4, "method": "tools/call", "params": { "name": "get_workforce_plan", "arguments": { "pid": "11111111-2222-4333-8444-555555555555" } } })).await;
        assert_eq!(missing["result"]["isError"], true);
        assert!(missing["result"]["content"][0]["text"].as_str().unwrap().contains("404"), "{missing}");

        // Transport: a notification is 202, a GET is 405, an unlisted Origin is 403, another revision is 400, junk is 400.
        assert_eq!(rpc(&request, json!({ "jsonrpc": "2.0", "method": "notifications/initialized" })).await.0, 202);
        assert_eq!(request.get("/mcp").await.status_code().as_u16(), 405);
        let evil = request.post("/mcp").add_header("origin", "https://evil.example").json(&json!({ "jsonrpc": "2.0", "id": 1, "method": "ping" })).await;
        assert_eq!(evil.status_code().as_u16(), 403);
        let old = request.post("/mcp").add_header("mcp-protocol-version", "2025-06-18").json(&json!({ "jsonrpc": "2.0", "id": 1, "method": "ping" })).await;
        assert_eq!(old.status_code().as_u16(), 400);
        let junk = request.post("/mcp").text("not json").await;
        assert_eq!(junk.status_code().as_u16(), 400);

        // Each call left a row with the tool name and nothing else.
        let rows = ctx.db.query_all_raw(sea_orm::Statement::from_string(sea_orm::DatabaseBackend::Postgres,
            "SELECT snapshot::text AS snapshot FROM audit_logs WHERE action = 'mcp_tool_called'".to_string())).await.unwrap();
        assert_eq!(rows.len(), 2, "one per tool call, none for initialize or list");
        for row in rows {
            let snapshot: Option<String> = row.try_get("", "snapshot").unwrap();
            let snapshot = snapshot.unwrap_or_default();
            assert!(snapshot.contains("tool") && !snapshot.contains("11111111"), "{snapshot}");
        }

        // No audit, no call (WPM-D107).
        ctx.db.execute_unprepared(
            "CREATE FUNCTION refuse_audit() RETURNS trigger AS $$ BEGIN RAISE EXCEPTION 'audit refused'; END $$ LANGUAGE plpgsql;
             CREATE TRIGGER refuse_audit BEFORE INSERT ON audit_logs FOR EACH ROW EXECUTE FUNCTION refuse_audit()").await.unwrap();
        let (status, refused) = rpc(&request, json!({ "jsonrpc": "2.0", "id": 5, "method": "tools/call", "params": { "name": "ai_register", "arguments": {} } })).await;
        assert_eq!(status, 500, "{refused}");
        assert_eq!(refused["error"]["code"], -32603);
        assert!(refused.get("result").is_none(), "nothing was read");
        ctx.db.execute_unprepared("DROP TRIGGER refuse_audit ON audit_logs; DROP FUNCTION refuse_audit()").await.unwrap();
    })
    .await;
}
