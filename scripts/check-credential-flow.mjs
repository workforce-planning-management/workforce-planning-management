#!/usr/bin/env node
// Check the verifiable credential flow end to end against a RUNNING demo service, with an independent verifier (WPM-T303).
//
// The verifier below is written from the standards (RFC 7515 and RFC 8037 for the signature, the W3C Verifiable Credentials Data Model 2.0,
// Open Badges 3.0 and the W3C Bitstring Status List v1.0 for the shapes), using only Node's own crypto, zlib and fetch: it shares no code with
// the service. It checks that a credential issued for a verified CPD entry is a compact JWS (`alg` EdDSA, `typ` vc+jwt) that verifies against
// the issuer's PUBLISHED key, that the payload has the right contexts and types and the skill's ESCO and SFIA alignments, that it holds nothing about
// the person, that its status entry points at a revocation list that verifies too, and that the bit for this credential is 0 until the worker
// revokes it and 1 after. It also checks that an altered credential and a credential under another key do not verify.
//
//     node scripts/check-credential-flow.mjs --print-env     # the settings to start the service with (a throwaway key; never a secret)
//     node scripts/check-credential-flow.mjs [http://localhost:5150]
//
// This needs sign-in ON (the worker acts as themself through /api/me), so the service is started with the printed settings against a database
// holding nothing real. The script mints its own test tokens with the throwaway key.
import { createHash, createPrivateKey, createPublicKey, sign, verify, randomUUID } from "node:crypto";
import { gunzipSync } from "node:zlib";
import path from "node:path";
import { fileURLToPath } from "node:url";

const b64u = (buf) => Buffer.from(buf).toString("base64url");
const API = (process.argv.find((a) => a.startsWith("http")) ?? "http://localhost:5150").replace(/\/$/, "");
const HERE = path.dirname(fileURLToPath(import.meta.url));

// Throwaway keys: one mints test tokens (PASETO v4.public), one signs credentials. They protect nothing.
const TOKEN_SEED = Buffer.alloc(32, 9);
const CREDENTIAL_SEED = Buffer.alloc(32, 7);
const PKCS8 = Buffer.from("302e020100300506032b657004220420", "hex");
const privateOf = (seed) => createPrivateKey({ key: Buffer.concat([PKCS8, seed]), format: "der", type: "pkcs8" });
const rawPublic = (priv) => createPublicKey(priv).export({ format: "jwk" }).x; // base64url
const tokenKid = b64u(createHash("sha256").update(Buffer.from(rawPublic(privateOf(TOKEN_SEED)), "base64url")).digest());

if (process.argv.includes("--print-env")) {
  const jwks = { keys: [{ kty: "OKP", crv: "Ed25519", use: "sig", kid: tokenKid, x: rawPublic(privateOf(TOKEN_SEED)) }] };
  console.log(`export WPM_REQUIRE_AUTH=1`);
  console.log(`export WPM_PASETO_KEYS='${JSON.stringify(jwks)}'`);
  console.log(`export WPM_ABAC_POLICY_FILE='${path.join(HERE, "../workforce-planning-management-service-with-rust/config/abac-policy.reference.json")}'`);
  console.log(`export WPM_CREDENTIAL_ISSUER='${API}'`);
  console.log(`export WPM_CREDENTIAL_SIGNING_KEY='${b64u(CREDENTIAL_SEED)}'`);
  console.log(`export WPM_CREDENTIAL_ISSUER_NAME='Credential flow check'`);
  process.exit(0);
}

// ── PASETO v4.public, to act as a person ────────────────────────────────────────────────────────────────────────────────────────────────
const le64 = (n) => { const b = Buffer.alloc(8); b.writeBigUInt64LE(BigInt(n)); return b; };
const pae = (...pieces) => Buffer.concat([le64(pieces.length), ...pieces.flatMap((p) => [le64(p.length), p])]);
function mint(sub, attrs = {}) {
  const iat = Math.floor(Date.now() / 1000);
  const m = Buffer.from(JSON.stringify({ sub, email: "person@example.com", name: "Test Person", iss: "authentication-service", aud: "main-x-service", exp: iat + 3600, iat, sid: "check", attrs }));
  const f = Buffer.from(JSON.stringify({ kid: tokenKid }));
  const h = Buffer.from("v4.public.");
  const sig = sign(null, pae(h, m, f, Buffer.alloc(0)), privateOf(TOKEN_SEED));
  return `v4.public.${b64u(Buffer.concat([m, sig]))}.${b64u(f)}`;
}

let failures = 0;
function check(ok, what) { console.log(`${ok ? "  ok  " : "  FAIL"} ${what}`); if (!ok) failures += 1; }

async function call(method, route, token, body) {
  const res = await fetch(API + route, { method, headers: { "content-type": "application/json", "idempotency-key": randomUUID(), ...(token ? { authorization: `Bearer ${token}` } : {}) }, body: body === undefined ? undefined : JSON.stringify(body) });
  const text = await res.text();
  let json = null; try { json = JSON.parse(text); } catch { /* not JSON */ }
  return { status: res.status, json, text };
}
async function must(method, route, token, body) {
  const r = await call(method, route, token, body);
  if (r.status !== 200) throw new Error(`${method} ${route} -> ${r.status} ${r.text.slice(0, 200)}`);
  return r.json;
}

// ── The independent verifier ────────────────────────────────────────────────────────────────────────────────────────────────────────────
function parseJws(jws) {
  const parts = jws.split(".");
  if (parts.length !== 3) return null;
  const [h, p, s] = parts;
  try { return { header: JSON.parse(Buffer.from(h, "base64url")), payload: JSON.parse(Buffer.from(p, "base64url")), signingInput: `${h}.${p}`, signature: Buffer.from(s, "base64url") }; } catch { return null; }
}
function verifiesWith(jwk, parsed) {
  if (!parsed || parsed.header.alg !== "EdDSA") return false;
  const key = createPublicKey({ key: { kty: jwk.kty, crv: jwk.crv, x: jwk.x }, format: "jwk" });
  return verify(null, Buffer.from(parsed.signingInput), key, parsed.signature);
}
const bitAt = (list, index) => (list[index >> 3] >> (7 - (index & 7))) & 1; // left-most bit first (Bitstring Status List v1.0)

async function main() {
  const sub = randomUUID();
  const uniq = randomUUID(); // makes this run's skill references unique; deliberately not the token subject
  const svc = mint("svc-user", { svc: ["true"] });
  const hr = mint("hr-user", { hr: ["true"] });
  const me = mint(sub);
  const org = `organization:${randomUUID()}`;
  const number = `CF-${sub.slice(0, 6)}`;
  const displayName = `Credential Check ${sub.slice(0, 6)}`;

  console.log("Set-up (as service, HR and the worker):");
  const worker = await must("POST", "/api/workers", svc, { person_ref: `person:${sub}`, organization_ref: org, worker_number: number, display_name: displayName, employment_type: "permanent", department: "care", job_title: "Carer", hired_on: "2020-01-05", fte_percent: 100 });
  await must("POST", `/api/workers/${worker.pid}/status`, svc, { to: "active" });
  const skill = await must("POST", "/api/skills", svc, { name: `Cred skill ${uniq.slice(0, 6)}`, category: "technical" });
  await must("POST", `/api/skills/${skill.pid}/refs`, svc, { framework_slug: "esco", ref: `http://data.europa.eu/esco/skill/${uniq}`, label: "ESCO test skill" });
  await must("POST", `/api/skills/${skill.pid}/refs`, svc, { framework_slug: "sfia", ref: `PROG-${uniq.slice(0, 6)}`, label: "Programming" });
  const entry = await must("POST", `/api/workers/${worker.pid}/cpd-entries`, svc, { entry_date: "2026-09-01", activity: "Secret course title", category: "course", unit: "hours", amount: 3, skill_pid: skill.pid });
  const early = await call("POST", "/api/me/credentials", me, { cpd_entry_pid: entry.pid });
  check(early.status === 422, `an entry nobody has verified cannot be turned into a credential (${early.status})`);
  await must("POST", `/api/cpd-entries/${entry.pid}/verify`, hr);

  console.log("The issuer's published key, with no sign-in:");
  const meta = await call("GET", "/api/credentials/issuer", null);
  check(meta.status === 200 && meta.json.jwk?.crv === "Ed25519" && meta.json.jwk?.kty === "OKP", "the issuer metadata and its Ed25519 key are public");
  check(createHash("sha256").update(Buffer.from(meta.json.jwk.x, "base64url")).digest().length === 32 && meta.json.jwk.x === rawPublic(privateOf(CREDENTIAL_SEED)), "the published key is the public half of the key the service was started with");
  check((await call("POST", "/api/me/credentials", null, { cpd_entry_pid: entry.pid })).status === 401, "asking for a credential without a token is refused (401)");

  console.log("The credential:");
  const issued = await must("POST", "/api/me/credentials", me, { cpd_entry_pid: entry.pid });
  const jws = issued.credential;
  const parsed = parseJws(jws);
  check(parsed && parsed.header.alg === "EdDSA" && parsed.header.typ === "vc+jwt" && parsed.header.kid === meta.json.jwk.kid, "a compact JWS, alg EdDSA, typ vc+jwt, with the issuer's key id");
  check(verifiesWith(meta.json.jwk, parsed), "the signature verifies against the published key");
  const vc = parsed.payload;
  check(vc["@context"][0] === "https://www.w3.org/ns/credentials/v2" && vc["@context"].includes("https://purl.imsglobal.org/spec/ob/v3p0/context-3.0.3.json"), "the contexts are Verifiable Credentials 2.0 first, then Open Badges 3.0.3");
  check(vc.type.includes("VerifiableCredential") && vc.type.includes("OpenBadgeCredential"), "the types are VerifiableCredential and OpenBadgeCredential");
  check(vc.issuer?.id === API && typeof vc.validFrom === "string", "the issuer is the configured URL and validFrom is set");
  const subject = vc.credentialSubject;
  check(subject?.type?.includes("AchievementSubject") && subject.id === undefined, "an AchievementSubject with no subject id");
  const targets = (subject?.achievement?.alignment ?? []).map((a) => `${a.targetFramework}:${a.targetCode}`);
  check(targets.some((t) => t.includes(`esco/skill/${uniq}`)) && targets.some((t) => t.includes(`PROG-${uniq.slice(0, 6)}`)), `the alignments carry the skill's ESCO and SFIA references (${JSON.stringify(targets)})`);
  const all = JSON.stringify(vc);
  if (process.env.SHOW) console.log(JSON.stringify(vc, null, 1));
  check(![displayName, number, sub, "person@example.com"].some((v) => all.includes(v)), "nothing about the person is in the credential: no name, employee number, token subject or email");
  // The badge needs a name, and the service uses the activity title the worker typed. That is theirs to share, but it is free text: a title
  // that held personal detail would carry it into the credential.
  check(subject?.achievement?.name === "Secret course title" && vc.name === "Secret course title", "the achievement is named by the activity title, and nothing else of the entry is in it");
  const status = vc.credentialStatus;
  check(status?.type === "BitstringStatusListEntry" && status.statusPurpose === "revocation" && /^\d+$/.test(String(status.statusListIndex)), "a BitstringStatusListEntry for revocation");

  console.log("The revocation list:");
  const fetchList = async () => {
    const r = await call("GET", new URL(status.statusListCredential).pathname, null);
    const p = parseJws(r.json.jwt);
    const cs = p.payload.credentialSubject;
    const bits = gunzipSync(Buffer.from(cs.encodedList.slice(1), "base64url")); // multibase 'u' = base64url, no padding
    return { ok: r.status === 200 && verifiesWith(meta.json.jwk, p), p, cs, bits, prefix: cs.encodedList[0] };
  };
  const before = await fetchList();
  check(before.ok, "the list is public and its own signature verifies against the same key");
  check(before.p.payload.type.includes("BitstringStatusListCredential") && before.cs.type === "BitstringStatusList" && before.cs.statusPurpose === "revocation" && before.prefix === "u", "a BitstringStatusListCredential, purpose revocation, multibase u");
  check(before.bits.length * 8 >= 131072, `at least 131 072 bits (${before.bits.length * 8})`);
  check(bitAt(before.bits, Number(status.statusListIndex)) === 0, "this credential's bit is 0: not revoked");

  console.log("Revocation:");
  const stranger = await call("POST", `/api/me/credentials/${issued.pid}/revoke`, mint(randomUUID()));
  check(stranger.status !== 200, `another person cannot revoke it (${stranger.status})`);
  await must("POST", `/api/me/credentials/${issued.pid}/revoke`, me);
  const after = await fetchList();
  check(after.ok && bitAt(after.bits, Number(status.statusListIndex)) === 1, "after the worker revokes it the bit is 1, on a list that still verifies");
  check(verifiesWith(meta.json.jwk, parsed), "the revoked credential's own signature is unchanged: revocation is on the list, not in the credential");
  const again = await call("POST", "/api/me/credentials", me, { cpd_entry_pid: entry.pid });
  check(again.status === 200 && again.json.pid !== issued.pid, "a revoked credential can be issued again, as a new one");

  console.log("Tampering:");
  const [h, p, s] = jws.split(".");
  const altered = parseJws(`${h}.${b64u(Buffer.from(JSON.stringify({ ...vc, validFrom: "2000-01-01T00:00:00Z" })))}.${s}`);
  check(!verifiesWith(meta.json.jwk, altered), "a credential with its payload changed does not verify");
  const other = { ...meta.json.jwk, x: rawPublic(privateOf(Buffer.alloc(32, 5))) };
  check(!verifiesWith(other, parsed), "a credential does not verify under another key");
  check(!verifiesWith(meta.json.jwk, parseJws(`${b64u(JSON.stringify({ alg: "none", typ: "vc+jwt" }))}.${p}.${s}`)), "a header that says alg none is not accepted");

  console.log(failures ? `FAILED: ${failures} check(s)` : "all checks passed");
  process.exit(failures ? 1 : 0);
}
main().catch((e) => { console.error(e.message); process.exit(2); });
