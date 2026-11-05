// The names of the inputs a learning cost is missing (WPM-D172): the service sends tokens, the reader gets words.

import { t, type StringKey } from "#lib/i18n.svelte.js";

/** The tokens are listed in `tests/unit/economics.test.ts`, which fails when one has no words. */
export function inputNames(tokens: readonly string[]): string {
  return tokens
    .map((token) => t(`economics.input.${token}` as StringKey))
    .join(", ");
}
