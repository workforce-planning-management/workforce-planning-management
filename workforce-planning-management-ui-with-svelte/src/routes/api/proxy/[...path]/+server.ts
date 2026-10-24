// BFF reverse proxy for workforce-planning-management API calls.
//
// The browser calls this same-origin proxy; the server forwards to the
// workforce-planning-management service. No token ever lives in browser JS: when a
// session cookie is present (WPM-T18 magic-link flow), the server
// exchanges it for a short-lived PASETO and injects the bearer here.
// The service enforces authentication by default (WPM-D52); only a service
// started with `WPM_REQUIRE_AUTH=0` (a demo) accepts the unauthenticated
// forward, and then the exchange is simply not needed.
//
// `WPM_API_URL` is the service ROOT (`http://localhost:5150`); the service
// serves its API under `/api/`, so `/api/proxy/workers` goes to `<root>/api/workers`.

import type { RequestHandler } from "./$types";
import { WPM_API_URL, WPM_DEMO_API_URL } from "#lib/server/config.js";
import { demoMemberships, isDemoSession } from "#lib/server/demo.js";
import { exchangeToken } from "#lib/server/auth.js";
import { tokens } from "#lib/server/token-cache.js";
import { forwardHeaders, isSafeApiPath } from "#lib/server/proxy.js";

const proxy: RequestHandler = async ({
  request,
  params,
  url,
  locals,
  fetch,
  getClientAddress,
}) => {
  // A dot segment would leave `/api/` once the URL is resolved (WPM-T248).
  if (!isSafeApiPath(params.path)) {
    return new Response(JSON.stringify({ error: "bad_path" }), {
      status: 400,
      headers: { "content-type": "application/json" },
    });
  }
  // A demo session goes to the demo service (fictitious data, no authentication) and nowhere else; with
  // no demo configured it opens nothing.
  const demo = isDemoSession(locals.sessionId);
  if (demo && !WPM_DEMO_API_URL) {
    return new Response(JSON.stringify({ error: "no_demo" }), {
      status: 401,
      headers: { "content-type": "application/json" },
    });
  }
  // Nothing at the authentication service says who the demo user is, so the memberships are made up here.
  if (
    demo &&
    request.method === "GET" &&
    url.pathname.endsWith("/me/organizations")
  ) {
    return Response.json(await demoMemberships(fetch, WPM_DEMO_API_URL));
  }
  if (
    demo &&
    request.method === "GET" &&
    url.pathname.endsWith("/me/organizations/scope")
  ) {
    const memberships = await demoMemberships(fetch, WPM_DEMO_API_URL);
    return Response.json([
      ...new Set(memberships.map((m) => m.organization_ref)),
    ]);
  }
  const target = `${demo ? WPM_DEMO_API_URL : WPM_API_URL}/api/${params.path}${url.search}`;

  // Session → short-lived PASETO, server-side only (WPM-T18). Only the allow-listed headers go with it: never the
  // browser's cookie, its own authorization or its x-forwarded-for (WPM-T248); a plain idempotency key does (WPM-R132).
  // One exchange serves every call of a page until shortly before the token expires (WPM-T264).
  const sid = locals.sessionId && !demo ? locals.sessionId : null;
  const token = sid
    ? await tokens.get(sid, () => exchangeToken(fetch, sid))
    : null;
  let clientAddress: string | null = null;
  try {
    clientAddress = getClientAddress();
  } catch {
    // Some adapters and tests cannot say; the API then sees this server as the caller.
  }
  const headers = forwardHeaders(request.headers, { token, clientAddress });

  const init: RequestInit = { method: request.method, headers };
  if (request.method !== "GET" && request.method !== "HEAD") {
    init.body = await request.arrayBuffer();
  }

  const upstream = await fetch(target, init);
  // A token the API refused is not kept: the next call exchanges again.
  if (upstream.status === 401 && sid) tokens.forget(sid);

  const responseHeaders = new Headers();
  for (const name of ["content-type", "etag"]) {
    const value = upstream.headers.get(name);
    if (value) responseHeaders.set(name, value);
  }
  return new Response(upstream.body, {
    status: upstream.status,
    headers: responseHeaders,
  });
};

export const GET = proxy;
export const POST = proxy;
export const PUT = proxy;
export const PATCH = proxy;
export const DELETE = proxy;
