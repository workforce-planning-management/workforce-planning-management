// BFF server-side session helpers (never imported by browser code).
//
// The durable session lives in the authentication service; the browser
// holds only an opaque session id in the httpOnly `__Host-mxi_session`
// cookie (see `agents/share/authentication-sessions.md` §6). The SvelteKit
// server reads/sets that cookie and exchanges it for short-lived PASETO
// tokens server-side — the browser never sees a token.

/** The httpOnly session cookie name (host-locked via the `__Host-` prefix). */
export const SESSION_COOKIE = "__Host-mxi_session";

/** Cookie attributes for `cookies.set` — httpOnly, Secure, host-locked. */
export const SESSION_COOKIE_OPTIONS = {
  path: "/",
  httpOnly: true,
  secure: true,
  sameSite: "lax",
} as const;

/** Extract `__Host-mxi_session` from a single `Set-Cookie` header line. */
export function parseSessionId(setCookie: string): string | null {
  const prefix = `${SESSION_COOKIE}=`;
  const segment = setCookie
    .split(";")
    .map((s) => s.trim())
    .find((s) => s.startsWith(prefix));
  if (!segment) return null;
  const value = segment.slice(prefix.length);
  return value.length > 0 ? value : null;
}

/** Find the session id across all `Set-Cookie` lines of an upstream
 *  response (uses `getSetCookie()` where available). */
export function sessionIdFromResponse(response: Response): string | null {
  const headers = response.headers as Headers & {
    getSetCookie?: () => string[];
  };
  const lines = headers.getSetCookie?.() ?? [
    response.headers.get("set-cookie") ?? "",
  ];
  for (const line of lines) {
    const sid = parseSessionId(line);
    if (sid) return sid;
  }
  return null;
}

// ─── Plain http on this machine ─────────────────────────────────────────────
// `__Host-` needs `Secure`, and a `Secure` cookie set over `http://localhost` is kept by Chromium and
// Firefox but refused by Safari (WebKit), so on a local trial in Safari a sign-in appeared to do nothing:
// the server set the cookie, the browser dropped it, and the next page sent the visitor back to sign in.
// Only for a plain-http address on this machine, the session therefore also travels in a cookie without the
// prefix and without `Secure`. Anywhere else the host-locked cookie is the only one that counts, and a
// cookie of the other name is ignored.

/** The session cookie used on plain http on this machine. */
export const SESSION_COOKIE_LOCAL = "mxi_session";

/** Whether `url` is plain http on this machine (never a deployed address). */
export function isLocalHttp(url: URL): boolean {
  return (
    url.protocol === "http:" &&
    ["localhost", "127.0.0.1", "[::1]"].includes(url.hostname)
  );
}

type CookieJar = {
  get(name: string): string | undefined;
  set(name: string, value: string, options: Record<string, unknown>): void;
  delete(name: string, options: Record<string, unknown>): void;
};

/** The session id the request carries, from the cookie that counts for its address. */
export function readSessionId(cookies: CookieJar, url: URL): string | null {
  const name = isLocalHttp(url) ? SESSION_COOKIE_LOCAL : SESSION_COOKIE;
  return cookies.get(name) ?? cookies.get(SESSION_COOKIE) ?? null;
}

/** Store a session id, in the cookie that suits the address. */
export function setSessionCookie(
  cookies: CookieJar,
  url: URL,
  id: string,
): void {
  if (isLocalHttp(url)) {
    cookies.set(SESSION_COOKIE_LOCAL, id, {
      path: "/",
      httpOnly: true,
      secure: false,
      sameSite: "lax",
    });
  } else {
    cookies.set(SESSION_COOKIE, id, SESSION_COOKIE_OPTIONS);
  }
}

/** Forget the session, whichever cookie holds it. */
export function clearSessionCookie(cookies: CookieJar, url: URL): void {
  cookies.delete(SESSION_COOKIE, SESSION_COOKIE_OPTIONS);
  if (isLocalHttp(url)) {
    cookies.delete(SESSION_COOKIE_LOCAL, { path: "/" });
  }
}
