// What to say when a write fails: a refusal for want of the right (403) is said in words, anything else
// is the service's own message (a validation message names the field), never a silent failure.

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
 * What to show for a failed call. The words come from the catalogue, so every locale reads them.
 * A `403`, `409` or `422` adds the service's own message after them, because it names the rule, the
 * field or the state that was refused; a `401`, `404` or `5xx` adds nothing (a `5xx` gives no detail on
 * purpose, WPM-T259).
 */
export function errorText(cause: unknown): string {
  const lead = t(errorKey(cause));
  if (
    cause instanceof ApiError &&
    (cause.status === 403 || cause.status === 409 || cause.status === 422)
  ) {
    const detail = cause.message.trim();
    if (detail && detail !== `API error ${cause.status}`) {
      return `${lead}${/[.!?。]$/.test(lead) ? " " : ": "}${detail}`;
    }
  }
  return lead;
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
