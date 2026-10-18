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
import { isIdempotencyKey } from "#lib/idempotency.js";

const proxy: RequestHandler = async ({
  request,
  params,
  url,
  locals,
  fetch,
}) => {
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

  // Copy request headers, but drop hop-by-hop / origin-specific ones and
  // never forward the browser's cookie to the entity service.
  const headers = new Headers(request.headers);
  headers.delete("cookie");
  headers.delete("host");
  headers.delete("connection");
  headers.delete("content-length");
  headers.set("accepts-version", "1.0");

  // Pass an idempotency key through only if it is a plain token (WPM-R132);
  // anything else is dropped rather than forwarded.
  const key = headers.get("idempotency-key");
  if (key !== null && !isIdempotencyKey(key)) headers.delete("idempotency-key");

  // Session → short-lived PASETO, server-side only (WPM-T18).
  if (locals.sessionId && !demo) {
    const token = await exchangeToken(fetch, locals.sessionId);
    if (token) headers.set("authorization", `Bearer ${token}`);
  }

  const init: RequestInit = { method: request.method, headers };
  if (request.method !== "GET" && request.method !== "HEAD") {
    init.body = await request.arrayBuffer();
  }

  const upstream = await fetch(target, init);

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
