// Minimal typed fetch wrapper over the BFF proxy.

import { API_BASE_URL } from "#lib/config.js";

/** Error carrying the upstream status + parsed body, for page handling. */
export class ApiError extends Error {
  status: number;
  body: unknown;
  constructor(status: number, body: unknown) {
    // RFC 9457: `detail` says what went wrong this time (WPM-R238).
    const detail =
      typeof body === "object" && body !== null && "detail" in body
        ? String((body as { detail: unknown }).detail)
        : `API error ${status}`;
    super(detail);
    this.status = status;
    this.body = body;
  }
}

/** Methods that do not change anything, so need no idempotency key. */
const SAFE_METHODS = new Set(["GET", "HEAD", "OPTIONS"]);

/**
 * A fresh key for one user action (WPM-R132): a random UUID, or, where
 * `crypto.randomUUID` is missing (an insecure context), 128 random bits in
 * hexadecimal. The key names the *action*, not the request, so a retry of
 * the same action must pass the same key back in.
 */
export function newIdempotencyKey(): string {
  const c = globalThis.crypto;
  if (typeof c?.randomUUID === "function") return c.randomUUID();
  const bytes = new Uint8Array(16);
  c.getRandomValues(bytes);
  return Array.from(bytes, (b) => b.toString(16).padStart(2, "0")).join("");
}

/** The one request: the parsed body and the response, for a caller that wants its headers. */
async function send<T>(
  path: string,
  init?: {
    method?: string;
    body?: unknown;
    fetch?: typeof fetch;
    idempotencyKey?: string;
  },
): Promise<{ parsed: T; response: Response }> {
  const doFetch = init?.fetch ?? fetch;
  const url = `${API_BASE_URL}${path}`;
  const method = (init?.method ?? "GET").toUpperCase();
  const headers: Record<string, string> = {};
  if (init?.body !== undefined) headers["content-type"] = "application/json";
  if (!SAFE_METHODS.has(method)) {
    headers["idempotency-key"] = init?.idempotencyKey ?? newIdempotencyKey();
  }
  const response = await doFetch(url, {
    method,
    headers,
    body: init?.body !== undefined ? JSON.stringify(init.body) : undefined,
  });
  const text = await response.text();
  const parsed: unknown = text ? JSON.parse(text) : null;
  if (!response.ok) throw new ApiError(response.status, parsed);
  return { parsed: parsed as T, response };
}

/**
 * Perform one JSON request against the proxy; throws [`ApiError`] on non-2xx.
 *
 * A request that changes something (anything but GET, HEAD and OPTIONS) is
 * sent with an `Idempotency-Key` header so that a repeat of the same action
 * (a double click, a retry after a dropped connection) can be recognised.
 * Pass `idempotencyKey` to repeat an action under its original key.
 */
export async function api<T>(
  path: string,
  init?: Parameters<typeof send>[1],
): Promise<T> {
  return (await send<T>(path, init)).parsed;
}

/**
 * One page of a collection (WPM-T478): its rows, and the cursor of the next page from the service's `Link: <...>;
 * rel="next"`, or `null` on the last page. There is no total: a total would count rows the caller may not see.
 */
export type Page<T> = {
  items: T[];
  /** The signed `cursor` to send for the next page, opaque to the UI; `null` when no more rows follow. */
  next: string | null;
};

/** The `cursor` of a `Link` header's `rel="next"` target, or `null` when there is none. */
export function nextCursor(link: string | null): string | null {
  if (!link) return null;
  // Each link is `<target>` then its parameters up to the next `<`; a target may hold commas (`?fields=a,b`).
  for (const [, target = "", params = ""] of link.matchAll(
    /<([^>]*)>([^<]*)/g,
  )) {
    if (!/;\s*rel\s*=\s*"?next"?(?=[\s;,]|$)/.test(params)) continue;
    const query = target.split("?")[1] ?? "";
    const cursor = new URLSearchParams(query).get("cursor");
    if (cursor) return cursor;
  }
  return null;
}

/** A GET of a paged list: the rows and the next page's cursor. */
export async function apiPage<T>(
  path: string,
  init?: { fetch?: typeof fetch },
): Promise<Page<T>> {
  const { parsed, response } = await send<T[]>(path, init);
  return { items: parsed, next: nextCursor(response.headers.get("link")) };
}
