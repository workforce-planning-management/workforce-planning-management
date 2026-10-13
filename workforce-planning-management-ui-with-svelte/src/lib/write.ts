// What to say when a write fails: a refusal for want of the right (403) is said in words, anything else
// is the service's own message (a validation message names the field), never a silent failure.

import { ApiError } from "./api/client";

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
  return {
    notAllowed: false,
    message: cause instanceof Error ? cause.message : String(cause),
  };
}
