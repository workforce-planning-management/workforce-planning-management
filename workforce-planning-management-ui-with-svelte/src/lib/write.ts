// What to say when a write fails: always the catalogue's words for what the failure means, in the person's language, never a
// silent failure and never the service's own text. The service's messages are English and written for developers, so no page
// shows them (`tests/unit/no-api-error-text.test.ts` keeps it so).

import { ApiError } from "./api/client";
import { t, type StringKey } from "./i18n.svelte";

/** The catalogue key that says, in the person's language, what a failed call means (WPM-T261). */
export function errorKey(cause: unknown): StringKey {
  if (cause instanceof ApiError) {
    if (cause.status === 401) return "errors.unauthorized";
    if (cause.status === 403) return "errors.forbidden";
    if (cause.status === 404) return "errors.notFound";
    if (cause.status === 409) return "errors.conflict";
    if (cause.status === 422) return "errors.invalid";
    if (cause.status >= 500) return "errors.server";
    return "errors.generic";
  }
  // `fetch` rejects with a TypeError when the service cannot be reached at all.
  if (cause instanceof TypeError) return "errors.network";
  return "errors.generic";
}

/**
 * What to show for a failed call: the catalogue's words for its status, and nothing the service said. Every locale reads them. A
 * `5xx` gives no detail on purpose (WPM-T259), and neither does any other status: a refusal that needs to say which field or rule
 * is the page's own work, in catalogue words, not the service's message.
 */
export function errorText(cause: unknown): string {
  return t(errorKey(cause));
}

export interface Failure {
  /** The caller may read this but may not change it. */
  notAllowed: boolean;
  /** The service's message, or null when the failure was the refusal. */
  message: string | null;
}

export function failure(cause: unknown): Failure {
  if (cause instanceof ApiError && cause.status === 403) {
    return { notAllowed: true, message: null };
  }
  return { notAllowed: false, message: errorText(cause) };
}

/**
 * Run a write and say how it went: `null` when it went through, otherwise why not (WPM-T265). The one
 * place a page turns a failed write into something to show; what the page does after a success
 * (read again, close a form) stays its own.
 */
export async function attempt(
  action: () => Promise<unknown>,
): Promise<Failure | null> {
  try {
    await action();
    return null;
  } catch (cause) {
    return failure(cause);
  }
}
