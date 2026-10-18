// The session cookie on plain http on this machine (Safari refuses a Secure cookie set over http://localhost,
// so a sign-in there looked like it did nothing) and everywhere else.

import { describe, expect, it } from "vitest";
import {
  SESSION_COOKIE,
  SESSION_COOKIE_LOCAL,
  clearSessionCookie,
  isLocalHttp,
  readSessionId,
  setSessionCookie,
} from "../../src/lib/server/session";

function jar(initial: Record<string, string> = {}) {
  const values = { ...initial };
  const set: Array<[string, string, Record<string, unknown>]> = [];
  const deleted: string[] = [];
  return {
    values,
    set,
    deleted,
    get: (name: string) => values[name],
    setCookie: (
      name: string,
      value: string,
      options: Record<string, unknown>,
    ) => {
      set.push([name, value, options]);
      values[name] = value;
    },
    delete: (name: string) => {
      deleted.push(name);
      delete values[name];
    },
  };
}
const cookies = (j: ReturnType<typeof jar>) => ({
  get: j.get,
  set: j.setCookie,
  delete: j.delete,
});

describe("which addresses are local plain http", () => {
  it("only http on this machine", () => {
    expect(isLocalHttp(new URL("http://localhost:4520/x"))).toBe(true);
    expect(isLocalHttp(new URL("http://127.0.0.1:4520/x"))).toBe(true);
    expect(isLocalHttp(new URL("https://localhost/x"))).toBe(false);
    expect(isLocalHttp(new URL("http://wpm.example.org/x"))).toBe(false);
    expect(isLocalHttp(new URL("http://localhost.example.org/x"))).toBe(false);
  });
});

describe("storing and reading the session", () => {
  it("on plain local http: a cookie without the prefix and without Secure", () => {
    const j = jar();
    setSessionCookie(
      cookies(j),
      new URL("http://localhost:4520/signin"),
      "abc",
    );
    expect(j.set).toEqual([
      [
        SESSION_COOKIE_LOCAL,
        "abc",
        { path: "/", httpOnly: true, secure: false, sameSite: "lax" },
      ],
    ]);
    expect(readSessionId(cookies(j), new URL("http://localhost:4520/x"))).toBe(
      "abc",
    );
  });

  it("anywhere else: the host-locked Secure cookie, and the plain one is ignored", () => {
    const j = jar();
    setSessionCookie(
      cookies(j),
      new URL("https://wpm.example.org/signin"),
      "abc",
    );
    expect(j.set[0]![0]).toBe(SESSION_COOKIE);
    expect(j.set[0]![2]).toMatchObject({ secure: true, httpOnly: true });
    const planted = jar({ [SESSION_COOKIE_LOCAL]: "planted" });
    expect(
      readSessionId(cookies(planted), new URL("https://wpm.example.org/x")),
    ).toBeNull();
    expect(
      readSessionId(cookies(j), new URL("https://wpm.example.org/x")),
    ).toBe("abc");
  });

  it("on local http a host-locked cookie still counts (an earlier sign-in, or a browser that kept it)", () => {
    const j = jar({ [SESSION_COOKIE]: "old" });
    expect(readSessionId(cookies(j), new URL("http://localhost:4520/x"))).toBe(
      "old",
    );
  });

  it("signing out clears both on local http, and only the host-locked one elsewhere", () => {
    const local = jar();
    clearSessionCookie(cookies(local), new URL("http://localhost:4520/x"));
    expect(local.deleted).toEqual([SESSION_COOKIE, SESSION_COOKIE_LOCAL]);
    const remote = jar();
    clearSessionCookie(cookies(remote), new URL("https://wpm.example.org/x"));
    expect(remote.deleted).toEqual([SESSION_COOKIE]);
  });
});
