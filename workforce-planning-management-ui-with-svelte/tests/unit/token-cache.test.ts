// Unit tests: one exchange serves a page's worth of calls, and a token is dropped before it can run
// out, on sign-out and when the API refuses it (WPM-T264).

import { describe, expect, it } from "vitest";

import { TokenCache, tokenExpiry } from "../../src/lib/server/token-cache";

/** A PASETO-shaped public token whose message is this JSON, with a 64-byte stand-in signature. */
function token(claims: object, version = "v4"): string {
  const message = Buffer.from(JSON.stringify(claims));
  const signature = Buffer.alloc(version === "v3" ? 96 : 64, 7);
  return `${version}.public.${Buffer.concat([message, signature]).toString("base64url")}`;
}

const T0 = Date.parse("2026-10-24T10:00:00Z");
const at = (secondsFromT0: number) => new Date(T0 + secondsFromT0 * 1000).toISOString();

/** An exchange that counts its calls and hands out tokens that expire `life` seconds after T0. */
function exchanger(life = 600) {
  let calls = 0;
  return {
    get calls() {
      return calls;
    },
    run: async () => {
      calls += 1;
      return token({ exp: at(life), n: calls });
    },
  };
}

describe("tokenExpiry", () => {
  it("reads exp from v2, v3 and v4 public tokens", () => {
    for (const v of ["v2", "v3", "v4"]) {
      expect(tokenExpiry(token({ exp: at(60) }, v))).toBe(T0 + 60_000);
    }
  });

  it("says nothing it cannot read", () => {
    expect(tokenExpiry("not a token")).toBeNull();
    expect(tokenExpiry("v4.local.abcd")).toBeNull();
    expect(tokenExpiry(token({ sub: "x" }))).toBeNull();
    expect(tokenExpiry(token({ exp: "tomorrow" }))).toBeNull();
    expect(tokenExpiry(token({ exp: 12345 }))).toBeNull();
    expect(tokenExpiry("v4.public.AAAA")).toBeNull();
  });
});

describe("TokenCache", () => {
  it("a page load of many calls exchanges once", async () => {
    let now = T0;
    const cache = new TokenCache(() => now);
    const ex = exchanger();
    // Twenty calls in flight at once, then twenty more after the first answers.
    const first = await Promise.all(Array.from({ length: 20 }, () => cache.get("sid", ex.run)));
    const later = await Promise.all(Array.from({ length: 20 }, () => cache.get("sid", ex.run)));
    expect(ex.calls).toBe(1);
    expect(new Set([...first, ...later]).size).toBe(1);
  });

  it("keeps a token per session", async () => {
    const cache = new TokenCache(() => T0);
    const ex = exchanger();
    const a = await cache.get("a", ex.run);
    const b = await cache.get("b", ex.run);
    expect(a).not.toBe(b);
    expect(ex.calls).toBe(2);
    expect(await cache.get("a", ex.run)).toBe(a);
  });

  it("exchanges again shortly before the token expires, not after", async () => {
    let now = T0;
    const cache = new TokenCache(() => now);
    const ex = exchanger(600);
    await cache.get("sid", ex.run);
    now = T0 + 569_000; // 31 s left: still kept
    await cache.get("sid", ex.run);
    expect(ex.calls).toBe(1);
    now = T0 + 571_000; // 29 s left: inside the margin
    const renewed = exchanger(1200);
    await cache.get("sid", renewed.run);
    expect(renewed.calls).toBe(1);
    expect(cache.size).toBe(1);
  });

  it("does not keep a token whose expiry it cannot read, or that is already near its end", async () => {
    const cache = new TokenCache(() => T0);
    let calls = 0;
    const unreadable = async () => {
      calls += 1;
      return "v4.public.AAAA";
    };
    await cache.get("sid", unreadable);
    await cache.get("sid", unreadable);
    expect(calls).toBe(2);
    const nearEnd = exchanger(10);
    await cache.get("other", nearEnd.run);
    await cache.get("other", nearEnd.run);
    expect(nearEnd.calls).toBe(2);
    expect(cache.size).toBe(0);
  });

  it("does not keep a failed exchange", async () => {
    const cache = new TokenCache(() => T0);
    let calls = 0;
    expect(await cache.get("sid", async () => (calls++, null))).toBeNull();
    expect(await cache.get("sid", async () => (calls++, null))).toBeNull();
    expect(calls).toBe(2);
    await expect(cache.get("sid", () => Promise.reject(new Error("down")))).rejects.toThrow("down");
    // And a rejection does not leave later callers waiting on it.
    const ex = exchanger();
    expect(await cache.get("sid", ex.run)).not.toBeNull();
  });

  it("forget drops the token at once, and an exchange that was running is not kept", async () => {
    const cache = new TokenCache(() => T0);
    const ex = exchanger();
    await cache.get("sid", ex.run);
    cache.forget("sid");
    expect(cache.size).toBe(0);
    await cache.get("sid", ex.run);
    expect(ex.calls).toBe(2);

    let release!: (t: string) => void;
    const slow = cache.get("late", () => new Promise<string>((res) => (release = res)));
    cache.forget("late");
    release(token({ exp: at(600) }));
    await slow;
    expect(cache.size).toBe(1); // only "sid"; "late" was signed out meanwhile
  });

  it("remembers a bounded number of sessions", async () => {
    const cache = new TokenCache(() => T0);
    const ex = exchanger();
    for (let i = 0; i < 1_100; i++) await cache.get(`sid-${i}`, ex.run);
    expect(cache.size).toBeLessThanOrEqual(1_000);
  });
});
