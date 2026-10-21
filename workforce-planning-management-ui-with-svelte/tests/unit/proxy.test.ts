// What the API proxy passes on (WPM-T248): an allow-list of headers, and no path that leaves `/api/`.

import { afterEach, describe, expect, it, vi } from "vitest";
import { forwardHeaders, isSafeApiPath } from "../../src/lib/server/proxy";

describe("the headers the proxy forwards", () => {
  const browser = new Headers({
    accept: "application/json",
    "content-type": "application/json",
    "if-none-match": '"abc"',
    "idempotency-key": "0123456789abcdef0123",
    authorization: "Bearer from-the-browser",
    cookie: "session=secret",
    "x-forwarded-for": "198.51.100.9",
    "x-anything": "else",
  });

  it("keeps the allow-list and the version, and drops everything else", () => {
    const sent = forwardHeaders(browser);
    expect([...sent.keys()].sort()).toEqual([
      "accept",
      "accepts-version",
      "content-type",
      "idempotency-key",
      "if-none-match",
    ]);
    expect(sent.get("accepts-version")).toBe("1.0");
  });

  it("never passes on the browser's authorization; only the exchanged token", () => {
    expect(forwardHeaders(browser).get("authorization")).toBeNull();
    expect(
      forwardHeaders(browser, { token: null }).get("authorization"),
    ).toBeNull();
    expect(
      forwardHeaders(browser, { token: "real" }).get("authorization"),
    ).toBe("Bearer real");
  });

  it("names the caller by the address this server sees, not the one the browser wrote", () => {
    expect(forwardHeaders(browser).get("x-forwarded-for")).toBeNull();
    expect(
      forwardHeaders(browser, { clientAddress: "203.0.113.4" }).get(
        "x-forwarded-for",
      ),
    ).toBe("203.0.113.4");
  });

  it("drops an idempotency key that is not a plain token", () => {
    const odd = new Headers({ "idempotency-key": "short" });
    expect(forwardHeaders(odd).get("idempotency-key")).toBeNull();
  });
});

describe("the paths the proxy accepts", () => {
  it("accepts ordinary API paths", () => {
    expect(isSafeApiPath("workers")).toBe(true);
    expect(isSafeApiPath("workers/0c0e/upline")).toBe(true);
    expect(isSafeApiPath("me/organizations/scope")).toBe(true);
  });

  it("refuses a path that could leave /api/", () => {
    for (const path of [
      "..",
      "../_posture",
      "workers/../../x",
      ".",
      "workers/./x",
      "",
      "workers//x",
      "a\\..\\b",
    ]) {
      expect(isSafeApiPath(path), path).toBe(false);
    }
  });
});

describe("the proxy route", () => {
  afterEach(() => {
    vi.resetModules();
    vi.doUnmock("../../src/lib/server/config");
  });

  async function route() {
    vi.resetModules();
    vi.doMock("../../src/lib/server/config", () => ({
      WPM_API_URL: "http://real.test",
      WPM_DEMO_API_URL: "",
      AUTH_API_URL: "http://auth.test",
      WPM_PUBLIC_URL: "",
    }));
    return (await import("../../src/routes/api/proxy/[...path]/+server")).GET;
  }

  it("with no session, sends no authorization even when the browser offers one", async () => {
    const GET = await route();
    const seen: Array<string | null> = [];
    const fetchStub = (async (_url: string | URL, init?: RequestInit) => {
      seen.push(new Headers(init?.headers).get("authorization"));
      return new Response("[]", { status: 200 });
    }) as typeof fetch;
    const response = await GET({
      request: new Request("http://ui.test/api/proxy/workers", {
        headers: { authorization: "Bearer forged" },
      }),
      params: { path: "workers" },
      url: new URL("http://ui.test/api/proxy/workers"),
      locals: { sessionId: null },
      fetch: fetchStub,
      getClientAddress: () => "203.0.113.4",
    } as unknown as Parameters<typeof GET>[0]);
    expect(response.status).toBe(200);
    expect(seen).toEqual([null]);
  });

  it("answers 400 for a dot segment and calls nothing", async () => {
    const GET = await route();
    const fetchStub = vi.fn();
    const response = await GET({
      request: new Request("http://ui.test/api/proxy/x"),
      params: { path: "../_posture" },
      url: new URL("http://ui.test/api/proxy/x"),
      locals: { sessionId: null },
      fetch: fetchStub as unknown as typeof fetch,
    } as unknown as Parameters<typeof GET>[0]);
    expect(response.status).toBe(400);
    expect(fetchStub).not.toHaveBeenCalled();
  });
});
