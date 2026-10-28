// The proxy exchanges a session once for a page's worth of calls, forgets the token when the API refuses
// it, and sign-out forgets it too (WPM-T264).

import { beforeEach, describe, expect, it } from "vitest";

import { GET } from "../../src/routes/api/proxy/[...path]/+server";
import { POST as signOut } from "../../src/routes/signout/+server";
import { tokens } from "../../src/lib/server/token-cache";

function token(exp: Date): string {
  const message = Buffer.from(JSON.stringify({ exp: exp.toISOString() }));
  return `v4.public.${Buffer.concat([message, Buffer.alloc(64, 1)]).toString("base64url")}`;
}

/** A fetch standing in for the authentication service and the API; it records what it was asked. */
function upstream(apiStatus = 200) {
  const seen = { exchanges: 0, bearers: [] as (string | null)[], signouts: 0 };
  const fetchStub = (async (url: string | URL, init?: RequestInit) => {
    const u = String(url);
    if (u.endsWith("/api/auth/token")) {
      seen.exchanges += 1;
      return Response.json({ token: token(new Date(Date.now() + 600_000)) });
    }
    if (u.endsWith("/api/auth/signout")) {
      seen.signouts += 1;
      return new Response(null, { status: 204 });
    }
    seen.bearers.push(new Headers(init?.headers).get("authorization"));
    return Response.json([], { status: apiStatus });
  }) as typeof fetch;
  return { seen, fetchStub };
}

function call(fetchStub: typeof fetch, sid: string, path = "workers") {
  return GET({
    request: new Request(`http://app/api/proxy/${path}`),
    params: { path },
    url: new URL(`http://app/api/proxy/${path}`),
    locals: { sessionId: sid },
    fetch: fetchStub,
    getClientAddress: () => "203.0.113.9",
  } as never);
}

beforeEach(() => {
  tokens.forget("sid-1");
  tokens.forget("sid-2");
});

describe("the proxy and the token cache", () => {
  it("one page load of twenty calls exchanges once and sends the same bearer on each", async () => {
    const { seen, fetchStub } = upstream();
    await Promise.all(
      Array.from({ length: 20 }, (_, i) =>
        call(fetchStub, "sid-1", `workers?n=${i}`),
      ),
    );
    expect(seen.exchanges).toBe(1);
    expect(seen.bearers).toHaveLength(20);
    expect(new Set(seen.bearers).size).toBe(1);
    expect(seen.bearers[0]).toMatch(/^Bearer v4\.public\./);
  });

  it("a token the API refuses is exchanged again on the next call", async () => {
    const refused = upstream(401);
    await call(refused.fetchStub, "sid-2");
    await call(refused.fetchStub, "sid-2");
    expect(refused.seen.exchanges).toBe(2);
  });

  it("sign-out forgets the token", async () => {
    const { seen, fetchStub } = upstream();
    await call(fetchStub, "sid-1");
    expect(tokens.size).toBeGreaterThan(0);
    const cookies = { delete() {}, set() {}, get: () => undefined };
    await signOut({
      locals: { sessionId: "sid-1" },
      cookies,
      fetch: fetchStub,
      url: new URL("http://app/signout"),
    } as never).catch(() => undefined); // it redirects by throwing
    const before = seen.exchanges;
    await call(fetchStub, "sid-1");
    expect(seen.exchanges).toBeGreaterThan(before);
  });
});
