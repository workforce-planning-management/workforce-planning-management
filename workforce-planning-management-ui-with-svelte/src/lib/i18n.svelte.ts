// Lightweight i18n for the WPM SPA (family pattern, copy-adapted from
// the project-portfolio-management front-end): per-locale strings plus a
// reactive `$state` current-locale (Svelte 5 runes), exposed via a
// `t(key)` accessor.
//
// The strings are *content*, not code: one flat `key -> string` JSON
// file per locale at `content/locales/<locale>/ui.json`, edited through
// Sveltia CMS (`static/admin/config.yml`). `en-001` is the source of
// truth; every locale must cover the same key set (the parity test pins
// this). The locale is carried by the URL — `/cy-001/workers`, with bare
// aliases like `/cy/` redirecting (see `locales.ts` and `hooks.ts`) — and
// the root layout feeds it in with `i18n.set`. The last choice persists
// to localStorage only to pick the locale for an unprefixed visit.

import { browser } from "$app/env";
import {
  DEFAULT_LOCALE,
  LOCALES,
  localePath,
  normaliseLocale,
  type Locale,
} from "./locales";

export {
  DEFAULT_LOCALE,
  LOCALES,
  LOCALE_ALIASES,
  LOCALE_LABELS,
  isRtl,
  localePath,
  negotiateLocale,
  normaliseLocale,
  splitLocale,
  type Locale,
} from "./locales";

/** localStorage key under which the chosen UI locale persists. */
export const LOCALE_KEY = "mxi.wpm.locale";

/**
 * The pre-rename locale key (2026-07-23, `HCM` -> `WPM`).
 *
 * A returning user has their choice stored under the old key. Reading
 * it once — see `readStoredLocale` — is the difference between "the app
 * remembers me" and "the app silently reset to English".
 */
// NOTE to a future renamer: this literal is deliberately the OLD name.
// A blanket search-and-replace over the codebase must not "fix" it.
export const LEGACY_LOCALE_KEY = "mxi.hcm.locale";

// Every translatable UI string, keyed by a stable dotted key, loaded
// from the per-locale content files at build time. The files nest on the
// dots (`{"nav": {"workers": "…"}}`) because that is the shape Sveltia
// CMS edits; `flatten` turns them back into `nav.workers`.
interface Nested {
  [key: string]: string | Nested;
}

function flatten(node: Nested, prefix = ""): Record<string, string> {
  const out: Record<string, string> = {};
  for (const [key, value] of Object.entries(node)) {
    if (typeof value === "string") out[`${prefix}${key}`] = value;
    else Object.assign(out, flatten(value, `${prefix}${key}.`));
  }
  return out;
}

/** Dotted keys of a nested catalog: `{nav: {workers: ""}}` -> `"nav.workers"`. */
type DottedKeys<T, P extends string = ""> = {
  [K in keyof T & string]: T[K] extends string
    ? `${P}${K}`
    : DottedKeys<T[K], `${P}${K}.`>;
}[keyof T & string];

const files = import.meta.glob<Nested>("../../content/locales/*/ui.json", {
  eager: true,
  import: "default",
});

const STRINGS = Object.fromEntries(
  LOCALES.map((locale) => [
    locale,
    flatten(files[`../../content/locales/${locale}/ui.json`] ?? {}),
  ]),
) as Record<Locale, Record<string, string>>;

/** The set of valid translation keys (derived from the English catalog). */
export type StringKey = DottedKeys<
  typeof import("../../content/locales/en-001/ui.json")
>;

/** Every translatable key, for the per-locale coverage test. */
export const STRING_KEYS = Object.keys(STRINGS[DEFAULT_LOCALE]) as StringKey[];

/** Raw per-locale strings table, exposed for coverage testing. */
export const STRINGS_BY_LOCALE: Record<
  Locale,
  Record<string, string>
> = STRINGS;

// Seed the reactive locale from localStorage (default off the browser).
//
// Falls back to the pre-rename key once, adopting and re-persisting the
// value under the current key so the migration happens silently on the
// user's next visit rather than resetting their language.
function readStoredLocale(): Locale {
  if (!browser || typeof localStorage === "undefined") return DEFAULT_LOCALE;

  const stored = normaliseLocale(localStorage.getItem(LOCALE_KEY));
  if (stored !== null) return stored;

  const legacy = normaliseLocale(localStorage.getItem(LEGACY_LOCALE_KEY));
  if (legacy === null) return DEFAULT_LOCALE;
  try {
    localStorage.setItem(LOCALE_KEY, legacy);
    localStorage.removeItem(LEGACY_LOCALE_KEY);
  } catch {
    // A full or blocked store must not stop the app rendering; the
    // locale is still adopted for this session.
  }
  return legacy;
}

// Reactive current-locale state; mutating it re-renders every `t(...)`.
let current = $state<Locale>(readStoredLocale());

/** Reactive current-locale store with persistence. */
export const i18n = {
  /** The currently selected locale. */
  get locale(): Locale {
    return current;
  },
  /** Switch the active locale and persist the choice. */
  set(next: string): void {
    const locale = normaliseLocale(next) ?? DEFAULT_LOCALE;
    current = locale;
    if (browser && typeof localStorage !== "undefined") {
      try {
        localStorage.setItem(LOCALE_KEY, locale);
      } catch {
        // A blocked store must not stop the switch.
      }
    }
  },
  /** The list of supported locales (for the switcher). */
  get locales(): readonly Locale[] {
    return LOCALES;
  },
};

/**
 * Translate `key` in `locale` with graceful fallbacks (locale → its
 * `-001` language base → en-001 → the key itself). Pure — unit-testable
 * without a component.
 */
export function translate(key: StringKey, locale: Locale = current): string {
  const base = `${locale.split("-")[0]}-001` as Locale;
  return (
    STRINGS[locale]?.[key] ??
    STRINGS[base]?.[key] ??
    STRINGS[DEFAULT_LOCALE][key] ??
    key
  );
}

/** Reactive translation accessor for components: `t("dash.title")`. */
export function t(key: StringKey): string {
  return translate(key, current);
}

/** Reactive locale-prefixed link for an app path: `l("/workers")`. */
export function l(path: string): string {
  return localePath(current, path);
}
