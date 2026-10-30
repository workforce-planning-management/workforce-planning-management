// The workstation checklist's words (WPM-T386). A default item arrives with a stable `item_key` and its English text; the page shows
// the key's catalogue string (`erg.items.<key>`) in the reader's language. A custom item has no key and is shown as typed, and so is a
// key this client does not know yet (a newer server), rather than a raw key.

import { tp } from "./i18n.svelte.js";
import type { Locale } from "./locales.js";

/** An item's words in the page's language (or `locale`): its key's translation, else the stored text. */
export function ergonomicItemLabel(
  key: string | null | undefined,
  text: string,
  locale?: Locale,
): string {
  if (!key) return text;
  return tp(`erg.items.${key}`, {}, locale) ?? text;
}
