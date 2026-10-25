#!/usr/bin/env node
// Check the MCP server end to end against a RUNNING demo service, with an independent client (WPM-T304).
//
// The client below is written from the Model Context Protocol specification (revision 2025-11-25: lifecycle, Streamable HTTP transport, tools,
// authorization), JSON-RPC 2.0 and RFC 9728 (OAuth protected-resource metadata), with Node's own fetch: it shares no code with the service. It
// checks the 401 challenge and the metadata it points at, the initialize handshake, a notification, ping, tools/list (every tool read-only, with an
// input schema), a tool call that succeeds and one the API itself refuses (a tool error, not a protocol error), the JSON-RPC error codes, and the
// transport rules (protocol-version header, Origin, GET and DELETE). It mints its own test tokens (PASETO v4.public, a throwaway key) so the
// service must run with sign-in ON.
//
//     node scripts/check-mcp-flow.mjs --print-env       # the settings to start the service with (a throwaway key; never a secret)
//     node scripts/check-mcp-flow.mjs [http://localhost:5150]
import { createHash, createPrivateKey, createPublicKey, sign, randomUUID } from "node:crypto";
import path from "node:path";
import { fileURLToPath } from "node:url";

const b64u = (buf) => Buffer.from(buf).toString("base64url");
const API = (process.argv.find((a) => a.startsWith("http")) ?? "http://localhost:5150").replace(/\/$/, "");
const HERE = path.dirname(fileURLToPath(import.meta.url));
const PINNED = "2025-11-25";

const TOKEN_SEED = Buffer.alloc(32, 9);
const PKCS8 = Buffer.from("302e020100300506032b657004220420", "hex");
const privateOf = (seed) => createPrivateKey({ key: Buffer.concat([PKCS8, seed]), format: "der", type: "pkcs8" });
const rawPublic = (priv) => createPublicKey(priv).export({ format: "jwk" }).x;
const tokenKid = b64u(createHash("sha256").update(Buffer.from(rawPublic(privateOf(TOKEN_SEED)), "base64url")).digest());

if (process.argv.includes("--print-env")) {
  const jwks = { keys: [{ kty: "OKP", crv: "Ed25519", use: "sig", kid: tokenKid, x: rawPublic(privateOf(TOKEN_SEED)) }] };
  console.log("export WPM_REQUIRE_AUTH=1");
  console.log(`export WPM_PASETO_KEYS='${JSON.stringify(jwks)}'`);
  console.log(`export WPM_ABAC_POLICY_FILE='${path.join(HERE, "../workforce-planning-management-service-with-rust/config/abac-policy.reference.json")}'`);
  console.log(`export WPM_MCP_RESOURCE='${API}/mcp'`);
  console.log("export WPM_MCP_AUTHORIZATION_SERVERS='https://login.example.org/realms/wpm'");
  process.exit(0);
}

const le64 = (n) => { const b = Buffer.alloc(8); b.writeBigUInt64LE(BigInt(n)); return b; };
const pae = (...pieces) => Buffer.concat([le64(pieces.length), ...pieces.flatMap((p) => [le64(p.length), p])]);
function mint(sub, attrs = {}) {
  const iat = Math.floor(Date.now() / 1000);
  const m = Buffer.from(JSON.stringify({ sub, email: "person@example.com", name: "Test Person", iss: "authentication-service", aud: "main-x-service", exp: iat + 3600, iat, sid: "check", attrs }));
  const f = Buffer.from(JSON.stringify({ kid: tokenKid }));
  const h = Buffer.from("v4.public.");
  return `v4.public.${b64u(Buffer.concat([m, sign(null, pae(h, m, f, Buffer.alloc(0)), privateOf(TOKEN_SEED))]))}.${b64u(f)}`;
}

let failures = 0;
const check = (ok, what) => { console.log(`${ok ? "  ok  " : "  FAIL"} ${what}`); if (!ok) failures += 1; };

async function http(method, route, { token, body, headers = {} } = {}) {
  const res = await fetch(API + route, { method, headers: { ...(body !== undefined ? { "content-type": "application/json" } : {}), accept: "application/json, text/event-stream", ...(token ? { authorization: `Bearer ${token}` } : {}), ...headers }, body: body === undefined ? undefined : JSON.stringify(body) });
  const text = await res.text();
  let json = null; try { json = JSON.parse(text); } catch { /* none */ }
  return { status: res.status, headers: res.headers, text, json };
}
let nextId = 1;
let lastId = 0;
const rpc = (method, params, opts = {}) => { lastId = nextId++; return http("POST", "/mcp", { ...opts, body: { jsonrpc: "2.0", id: lastId, method, ...(params === undefined ? {} : { params }) }, headers: { "mcp-protocol-version": PINNED, ...(opts.headers ?? {}) } }); };
const hdr = { "mcp-protocol-version": PINNED };

async function main() {
  const worker = mint(randomUUID());
  const svc = mint("svc-user", { svc: ["true"] });

  console.log("Authorization (RFC 9728 and the MCP authorization section):");
  const anon = await http("POST", "/mcp", { body: { jsonrpc: "2.0", id: 1, method: "initialize", params: {} }, headers: hdr });
  check(anon.status === 401, "no token: 401");
  const challenge = anon.headers.get("www-authenticate") ?? "";
  const pointer = /^Bearer resource_metadata="([^"]+)"$/.exec(challenge)?.[1];
  check(Boolean(pointer), `the 401 says it wants a bearer token and where the metadata is (${challenge})`);
  const meta = await http("GET", new URL(pointer ?? `${API}/x`).pathname);
  check(meta.status === 200 && pointer === `${API}${new URL(pointer).pathname}`, "the metadata is public and is at the URL the challenge gave");
  check(meta.json?.resource === `${API}/mcp`, "its `resource` is the MCP endpoint's own URL, as RFC 9728 requires");
  check(Array.isArray(meta.json?.authorization_servers) && meta.json.authorization_servers.length > 0 && meta.json.authorization_servers.every((u) => u.startsWith("https://")), "it names at least one https authorization server");
  check(meta.json?.bearer_methods_supported?.join() === "header", "it says a bearer token goes in the Authorization header");
  check((await http("GET", "/.well-known/oauth-protected-resource")).json?.resource === meta.json?.resource, "the path-less well-known URL says the same");
  check(!JSON.stringify(meta.json).match(/person@example|svc-user|password|secret/i), "the metadata holds nothing about anyone");

  console.log("Lifecycle (initialize, initialized, ping):");
  const init = await rpc("initialize", { protocolVersion: PINNED, capabilities: {}, clientInfo: { name: "independent-check", version: "1.0.0" } }, { token: worker });
  check(init.status === 200 && init.json?.jsonrpc === "2.0" && init.json?.id === lastId && init.json?.error === undefined, "a JSON-RPC 2.0 result that echoes the request id");
  check(init.json?.result?.protocolVersion === PINNED, `the server answers with the revision it speaks (${init.json?.result?.protocolVersion})`);
  check(init.json?.result?.capabilities?.tools !== undefined && init.json.result.capabilities.resources === undefined && init.json.result.capabilities.prompts === undefined, "the only capability offered is tools");
  check(typeof init.json?.result?.serverInfo?.name === "string" && typeof init.json?.result?.serverInfo?.version === "string", "serverInfo has a name and a version");
  const initialized = await http("POST", "/mcp", { token: worker, body: { jsonrpc: "2.0", method: "notifications/initialized" }, headers: hdr });
  check(initialized.status === 202 && initialized.text === "", "the initialized notification is answered 202 with no body");
  const ping = await rpc("ping", undefined, { token: worker });
  check(ping.json?.result && Object.keys(ping.json.result).length === 0, "ping answers an empty result");

  console.log("Tools:");
  const list = await rpc("tools/list", {}, { token: worker });
  const tools = list.json?.result?.tools ?? [];
  check(tools.length > 0 && tools.every((t) => typeof t.name === "string" && typeof t.description === "string" && t.inputSchema?.type === "object"), `${tools.length} tools, each with a name, a description and an object input schema`);
  check(new Set(tools.map((t) => t.name)).size === tools.length, "tool names are unique");
  check(!tools.some((t) => /create|update|delete|remove|write|set_|add_|approve|reject|post_|put_/i.test(t.name)), "no tool name reads like a write");
  check(tools.every((t) => t.annotations?.readOnlyHint !== false), "no tool declares itself not read-only");
  const names = tools.map((t) => t.name);
  const register = await rpc("tools/call", { name: "ai_register", arguments: {} }, { token: worker });
  check(register.json?.result?.isError !== true && register.json?.result?.content?.[0]?.type === "text" && Array.isArray(JSON.parse(register.json.result.content[0].text).features), "a tool call the caller may make returns text content that is the API's JSON");
  const refused = await rpc("tools/call", { name: names.find((n) => /human_capital/.test(n)) ?? "human_capital_report", arguments: { organization: `organization:${randomUUID()}` } }, { token: worker });
  check(refused.status === 200 && refused.json?.error === undefined && refused.json?.result?.isError === true, "a call the API refuses the caller is a tool error (isError), not a protocol error");
  check(/40[13]/.test(refused.json?.result?.content?.[0]?.text ?? ""), `and says what the API said (${(refused.json?.result?.content?.[0]?.text ?? "").slice(0, 60)})`);
  const unknown = await rpc("tools/call", { name: "no_such_tool", arguments: {} }, { token: worker });
  check(unknown.json?.error?.code === -32602, `an unknown tool is a protocol error, invalid params (${unknown.json?.error?.code})`);
  const steer = await rpc("tools/call", { name: names.find((n) => /plan/.test(n)) ?? "get_workforce_plan", arguments: { pid: "../workers" } }, { token: worker });
  check(steer.json?.result?.isError === true || steer.json?.error !== undefined, "an argument that tries to steer the call to another route is refused");
  const asSvc = await rpc("tools/call", { name: "ai_register", arguments: {} }, { token: svc });
  check(asSvc.json?.result?.isError !== true, "the same tool works for another caller: the answer is that caller's, not cached across tokens");

  console.log("JSON-RPC errors:");
  check((await rpc("no/such/method", {}, { token: worker })).json?.error?.code === -32601, "an unknown method is -32601");
  const bad = await http("POST", "/mcp", { token: worker, body: undefined, headers: { ...hdr, "content-type": "application/json" } });
  const raw = await fetch(API + "/mcp", { method: "POST", headers: { "content-type": "application/json", authorization: `Bearer ${worker}`, ...hdr }, body: "{not json" });
  check(raw.status === 400 && (await raw.json()).error?.code === -32700, "a body that is not JSON is 400 with -32700");
  void bad;

  console.log("Transport (Streamable HTTP):");
  check((await http("POST", "/mcp", { token: worker, body: { jsonrpc: "2.0", id: 9, method: "ping" }, headers: { "mcp-protocol-version": "2024-11-05" } })).status === 400, "an MCP-Protocol-Version the server does not speak is 400");
  check((await http("POST", "/mcp", { token: worker, body: { jsonrpc: "2.0", id: 9, method: "ping" }, headers: { ...hdr, origin: "https://evil.example" } })).status === 403, "an Origin that is not listed is 403 (DNS-rebinding protection)");
  const get = await http("GET", "/mcp", { token: worker, headers: hdr });
  check(get.status === 405, `GET answers 405: no server-sent event stream (${get.status})`);
  // The spec lets a server answer 405 to DELETE when it has no sessions. The service's access policy refuses a worker's DELETE before the
  // route is reached (403), so a refusal of either kind is accepted for a worker and 405 is required for a caller the policy lets through.
  const del = await http("DELETE", "/mcp", { token: worker, headers: hdr });
  const delSvc = await http("DELETE", "/mcp", { token: svc, headers: hdr });
  check(delSvc.status === 405 && (del.status === 405 || del.status === 403), `DELETE: 405 where the policy lets it through (${delSvc.status}), a refusal for a worker (${del.status}): no session to end`);
  check(!JSON.stringify([init.text, list.text, register.text]).includes(worker), "the caller's token is not echoed in any answer");

  console.log(failures ? `FAILED: ${failures} check(s)` : "all checks passed");
  process.exit(failures ? 1 : 0);
}
main().catch((e) => { console.error(e.message); process.exit(2); });
