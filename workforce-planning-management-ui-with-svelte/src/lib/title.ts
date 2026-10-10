// Page titles for the `page.data.title` convention (see `routes/+layout.svelte`),
// translated for the locale in the URL. A route's `load` runs on the server too, where
// there is no single "current" locale, so the locale is read from the address rather than
// from the reactive state the components use.

import {
  DEFAULT_LOCALE,
  splitLocale,
  translate,
  type StringKey,
} from "./i18n.svelte";

/** The catalogue string `key` in the locale the address carries. */
export function pageTitle(key: StringKey, url: URL): string {
  const locale = splitLocale(url.pathname).locale ?? DEFAULT_LOCALE;
  return translate(key, locale);
}
