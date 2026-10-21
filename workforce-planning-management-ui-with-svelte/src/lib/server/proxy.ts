// What the API proxy passes on (WPM-T248): an allow-list of request headers, and only paths under `/api/`.

import { isIdempotencyKey } from "#lib/idempotency.js";

/** The browser's request headers the API may see. Everything else stays here. */
const FORWARDED = ["accept", "content-type", "if-none-match"] as const;

/**
 * The headers to send to the API for a browser request: the allow-list, a plain idempotency key, the API version,
 * the caller's address as this server sees it, and the bearer only if `token` is given. A browser's own
 * `authorization`, cookie or `x-forwarded-for` is never passed on.
 */
export function forwardHeaders(
  incoming: Headers,
  options: { token?: string | null; clientAddress?: string | null } = {},
): Headers {
  const headers = new Headers();
  for (const name of FORWARDED) {
    const value = incoming.get(name);
    if (value !== null) headers.set(name, value);
  }
  const key = incoming.get("idempotency-key");
  if (key !== null && isIdempotencyKey(key))
    headers.set("idempotency-key", key);
  headers.set("accepts-version", "1.0");
  if (options.clientAddress)
    headers.set("x-forwarded-for", options.clientAddress);
  if (options.token) headers.set("authorization", `Bearer ${options.token}`);
  return headers;
}

/** Whether a proxied path stays under `/api/`: no empty, `.` or `..` segment, and no backslash. */
export function isSafeApiPath(path: string): boolean {
  if (path.includes("\\")) return false;
  return path
    .split("/")
    .every((segment) => segment !== "" && segment !== "." && segment !== "..");
}
