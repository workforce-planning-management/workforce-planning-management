// Minimal typed fetch wrapper over the BFF proxy.

import { API_BASE_URL } from "#lib/config.js";

/** Error carrying the upstream status + parsed body, for page handling. */
export class ApiError extends Error {
  status: number;
  body: unknown;
  constructor(status: number, body: unknown) {
    const description =
      typeof body === "object" && body !== null && "description" in body
        ? String((body as { description: unknown }).description)
        : `API error ${status}`;
    super(description);
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
  init?: {
    method?: string;
    body?: unknown;
    fetch?: typeof fetch;
    idempotencyKey?: string;
  },
): Promise<T> {
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
  return parsed as T;
}
