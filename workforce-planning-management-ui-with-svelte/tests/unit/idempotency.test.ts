// Unit tests: the UI sends an Idempotency-Key on every request that changes
// something, one key per action, never on a read (WPM-R132); and the proxy
// forwards only plain keys.

import { describe, expect, it } from "vitest";

import { api, newIdempotencyKey } from "../../src/lib/api/client";
import { isIdempotencyKey } from "../../src/lib/idempotency";

/** A fetch that records each request's method and headers. */
function recorder() {
  const seen: { method: string; headers: Record<string, string> }[] = [];
  const fetchStub = (async (_url: string | URL, init?: RequestInit) => {
    seen.push({
      method: String(init?.method),
      headers: { ...(init?.headers as Record<string, string>) },
    });
    return new Response("{}", { status: 200 });
  }) as typeof fetch;
  return { seen, fetchStub };
}

describe("Idempotency-Key", () => {
  it("is sent on every method that changes something", async () => {
    const { seen, fetchStub } = recorder();
    for (const method of ["POST", "PUT", "PATCH", "DELETE", "post"]) {
      await api("/x", { method, body: { a: 1 }, fetch: fetchStub });
    }
    expect(seen).toHaveLength(5);
    for (const r of seen) {
      expect(
        isIdempotencyKey(r.headers["idempotency-key"] ?? ""),
        r.method,
      ).toBe(true);
    }
  });

  it("is sent even on a change with no body", async () => {
    const { seen, fetchStub } = recorder();
    await api("/x", { method: "DELETE", fetch: fetchStub });
    expect(seen[0].headers["idempotency-key"]).toBeTruthy();
    expect(seen[0].headers["content-type"]).toBeUndefined();
  });

  it("is never sent on a read", async () => {
    const { seen, fetchStub } = recorder();
    await api("/x", { fetch: fetchStub });
    await api("/x", { method: "GET", fetch: fetchStub });
    await api("/x", { method: "HEAD", fetch: fetchStub });
    await api("/x", { method: "OPTIONS", fetch: fetchStub });
    for (const r of seen) expect(r.headers["idempotency-key"]).toBeUndefined();
  });

  it("is new for each action", async () => {
    const { seen, fetchStub } = recorder();
    await api("/x", { method: "POST", body: {}, fetch: fetchStub });
    await api("/x", { method: "POST", body: {}, fetch: fetchStub });
    expect(seen[0].headers["idempotency-key"]).not.toBe(
      seen[1].headers["idempotency-key"],
    );
  });

  it("is the same when a retry passes the original key back", async () => {
    const { seen, fetchStub } = recorder();
    const key = newIdempotencyKey();
    await api("/x", { method: "POST", idempotencyKey: key, fetch: fetchStub });
    await api("/x", { method: "POST", idempotencyKey: key, fetch: fetchStub });
    expect(seen[0].headers["idempotency-key"]).toBe(key);
    expect(seen[1].headers["idempotency-key"]).toBe(key);
  });

  it("is a key the proxy accepts, with or without crypto.randomUUID", () => {
    expect(isIdempotencyKey(newIdempotencyKey())).toBe(true);
    const original = globalThis.crypto.randomUUID;
    try {
      Object.defineProperty(globalThis.crypto, "randomUUID", {
        value: undefined,
        configurable: true,
      });
      const key = newIdempotencyKey();
      expect(key).toMatch(/^[0-9a-f]{32}$/);
      expect(isIdempotencyKey(key)).toBe(true);
    } finally {
      Object.defineProperty(globalThis.crypto, "randomUUID", {
        value: original,
        configurable: true,
      });
    }
  });
});

describe("the proxy's key check", () => {
  it("accepts plain tokens of 16 to 64 characters", () => {
    expect(isIdempotencyKey("a".repeat(16))).toBe(true);
    expect(isIdempotencyKey("a".repeat(64))).toBe(true);
    expect(isIdempotencyKey("3f2b8c1e-aaaa-4bbb-8ccc-0123456789ab")).toBe(true);
  });

  it("refuses anything else", () => {
    for (const bad of [
      "",
      "short",
      "a".repeat(15),
      "a".repeat(65),
      "has space in it 123456",
      "semi;colon-1234567890",
      "new\nline-1234567890ab",
      "ключ-1234567890abcdef",
    ]) {
      expect(isIdempotencyKey(bad), JSON.stringify(bad)).toBe(false);
    }
  });
});

describe("the proxy handler", () => {
  async function forwarded(key: string | null) {
    const { POST } =
      await import("../../src/routes/api/proxy/[...path]/+server");
    let seen: Headers | undefined;
    const fetchStub = (async (_url: string | URL, init?: RequestInit) => {
      seen = new Headers(init?.headers);
      return new Response("{}", { status: 200 });
    }) as typeof fetch;
    const headers = new Headers({ "content-type": "application/json" });
    if (key !== null) headers.set("idempotency-key", key);
    await POST({
      request: new Request("http://ui.test/api/proxy/workers", {
        method: "POST",
        headers,
        body: "{}",
      }),
      params: { path: "workers" },
      url: new URL("http://ui.test/api/proxy/workers"),
      locals: {},
      fetch: fetchStub,
    } as unknown as Parameters<typeof POST>[0]);
    return seen!;
  }

  it("forwards a plain key to the API unchanged", async () => {
    const key = "3f2b8c1e-aaaa-4bbb-8ccc-0123456789ab";
    expect((await forwarded(key)).get("idempotency-key")).toBe(key);
  });

  it("drops a key that is not a plain token", async () => {
    expect(
      (await forwarded("bad key; drop table")).get("idempotency-key"),
    ).toBe(null);
  });

  it("adds none when the browser sent none", async () => {
    expect((await forwarded(null)).get("idempotency-key")).toBe(null);
  });
});
