// The exchanged access token, kept per session until shortly before it expires (WPM-T264).
//
// Every call the browser makes goes through the proxy, and each used to exchange the session for a
// fresh token: a page that fires twenty requests made twenty exchanges. Here one exchange serves them
// all. The cache lives in this server's memory only; the token never reaches the browser, and a
// restart simply exchanges again.
//
// What this does not change: the API still checks the token's signature, expiry and lifetime on every
// request. The expiry read here only decides how long to keep a token, so a wrong guess costs one
// refused request, not access. A session ended at the authentication service keeps working until its
// token expires, which is the same bound the API already documents (spec/auth.md, "Token age and
// revocation"); sign-out in this app forgets the token at once.

/** Keep a token this long short of its expiry, so one is never sent as it runs out. */
const SKEW_MS = 30_000;
/** At most this many sessions are remembered; beyond it the soonest to expire go first. */
const MAX_SESSIONS = 1_000;

/**
 * When a PASETO public token expires, in milliseconds since the epoch, or `null` when it cannot be told.
 * A `public` token is `vN.public.<base64url(message + signature)>` with an optional footer; the message
 * is JSON and carries `exp` as an ISO 8601 time. The signature is not checked here (the API does that).
 */
export function tokenExpiry(token: string): number | null {
  const [version, purpose, payload] = token.split(".");
  if (!version || purpose !== "public" || !payload) return null;
  const signature = version === "v3" ? 96 : 64;
  try {
    const bytes = Buffer.from(payload, "base64url");
    if (bytes.length <= signature) return null;
    const claims: unknown = JSON.parse(
      bytes.subarray(0, bytes.length - signature).toString("utf8"),
    );
    if (typeof claims !== "object" || claims === null) return null;
    const exp = (claims as { exp?: unknown }).exp;
    if (typeof exp !== "string") return null;
    const at = Date.parse(exp);
    return Number.isNaN(at) ? null : at;
  } catch {
    return null;
  }
}

interface Entry {
  token: string;
  expiresAt: number;
}

export class TokenCache {
  private readonly kept = new Map<string, Entry>();
  private readonly inFlight = new Map<string, Promise<string | null>>();

  constructor(private readonly now: () => number = Date.now) {}

  /** The token for this session: the kept one if it has time left, else the result of one `exchange`. */
  get(
    sid: string,
    exchange: () => Promise<string | null>,
  ): Promise<string | null> {
    const have = this.kept.get(sid);
    if (have && have.expiresAt - SKEW_MS > this.now())
      return Promise.resolve(have.token);
    this.kept.delete(sid);
    // Requests that arrive while one is exchanging wait for it rather than starting their own.
    const pending = this.inFlight.get(sid);
    if (pending) return pending;
    const started: Promise<string | null> = exchange()
      .then((token) => {
        // Forgotten while exchanging (sign-out): the answer is used by those waiting, not kept.
        if (token && this.inFlight.get(sid) === started) this.keep(sid, token);
        return token;
      })
      .finally(() => {
        if (this.inFlight.get(sid) === started) this.inFlight.delete(sid);
      });
    this.inFlight.set(sid, started);
    return started;
  }

  /** Forget a session's token: on sign-out, and when the API refuses the one it was given. */
  forget(sid: string): void {
    this.kept.delete(sid);
    this.inFlight.delete(sid);
  }

  /** How many sessions have a token kept (for tests). */
  get size(): number {
    return this.kept.size;
  }

  private keep(sid: string, token: string): void {
    const expiresAt = tokenExpiry(token);
    // A token whose expiry cannot be read is used once and not kept.
    if (expiresAt === null || expiresAt - SKEW_MS <= this.now()) return;
    this.kept.set(sid, { token, expiresAt });
    if (this.kept.size > MAX_SESSIONS) this.trim();
  }

  private trim(): void {
    const now = this.now();
    for (const [sid, entry] of this.kept)
      if (entry.expiresAt <= now) this.kept.delete(sid);
    const soonest = [...this.kept].sort(
      (a, b) => a[1].expiresAt - b[1].expiresAt,
    );
    for (const [sid] of soonest.slice(
      0,
      Math.max(0, this.kept.size - MAX_SESSIONS),
    )) {
      this.kept.delete(sid);
    }
  }
}

/** The one cache the proxy and sign-out share. */
export const tokens = new TokenCache();
