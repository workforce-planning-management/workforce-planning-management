// Lightweight i18n for the WPM SPA (family pattern, copy-adapted from
// the project-portfolio-management front-end): per-locale strings plus a
// reactive `$state` current-locale (Svelte 5 runes), exposed via a
// `t(key)` accessor.
//
// The strings are *content*, not code: one flat `key -> string` JSON
// file per locale at `content/locales/<locale>/ui.json`, edited through
// Sveltia CMS (`static/admin/config.yml`). `en-001` is the source of
// truth; every locale must cover the same key set (the parity test pins
// this). The locale is carried by the URL — `/cy-001/workers` (see
// `locales.ts` and `hooks.ts`) — and
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

/**
 * The CLDR plural categories, in CLDR's order (WPM-T374). A string that fills a count is a *plural group*: a
 * catalogue object whose keys are categories, `{"one": "…", "other": "…"}` in English and all six in Arabic.
 */
export const PLURAL_CATEGORIES = [
  "zero",
  "one",
  "two",
  "few",
  "many",
  "other",
] as const;
export type PluralCategory = (typeof PLURAL_CATEGORIES)[number];

/** The base key of a plural group: `pages.cpd.requirement_s_defined` for `….one` and `….other`. */
type PluralBase<K> = K extends `${infer B}.other`
  ? `${B}.one` extends StringKey
    ? B
    : never
  : never;
export type PluralKey = PluralBase<StringKey>;

function isPluralGroup(node: Nested): boolean {
  const keys = Object.keys(node);
  return (
    keys.includes("other") &&
    keys.every(
      (k) =>
        (PLURAL_CATEGORIES as readonly string[]).includes(k) &&
        typeof node[k] === "string",
    )
  );
}

function pluralBases(node: Nested, prefix = ""): string[] {
  const out: string[] = [];
  for (const [key, value] of Object.entries(node)) {
    if (typeof value === "string") continue;
    if (isPluralGroup(value)) out.push(`${prefix}${key}`);
    else out.push(...pluralBases(value, `${prefix}${key}.`));
  }
  return out;
}

/** Every plural group's base key, from the English catalogue. */
export const PLURAL_KEYS = pluralBases(
  files["../../content/locales/en-001/ui.json"] ?? {},
) as PluralKey[];

/** The plural categories `locale` uses for whole and decimal counts (`Intl.PluralRules`): the forms its file must hold. */
export function pluralCategories(locale: string): PluralCategory[] {
  try {
    const have = new Intl.PluralRules(locale).resolvedOptions()
      .pluralCategories as string[];
    return PLURAL_CATEGORIES.filter((c) => have.includes(c));
  } catch {
    return ["one", "other"];
  }
}

/** The CLDR plural category of `count` in `locale`: "one" for 1 in English, "few" for 3 in Arabic, "other" for anything in Chinese. */
export function pluralCategory(count: number, locale: string): PluralCategory {
  try {
    return new Intl.PluralRules(locale).select(count) as PluralCategory;
  } catch {
    return count === 1 ? "one" : "other";
  }
}

/**
 * The form of plural group `key` for `count` in `locale`: the locale's own category, then its `-001` base, then the
 * group's `other`, and only then English (by English's rules). Pure — unit-testable without a component.
 */
export function translatePlural(
  key: PluralKey,
  count: number,
  locale: Locale = current,
): string {
  const base = `${locale.split("-")[0]}-001` as Locale;
  const category = pluralCategory(count, locale);
  for (const k of [`${key}.${category}`, `${key}.other`]) {
    const found = STRINGS[locale]?.[k] ?? STRINGS[base]?.[k];
    if (found !== undefined) return found;
  }
  const en = STRINGS[DEFAULT_LOCALE];
  return (
    en[`${key}.${pluralCategory(count, DEFAULT_LOCALE)}`] ??
    en[`${key}.other`] ??
    key
  );
}

/**
 * Reactive plural translation: `tn("pages.cpd.requirement_s_defined", n, { requirements: n })` picks the CLDR form
 * for `n` in the page's locale and fills the placeholders (numbers in the page's format). The count is passed in
 * `params` under the name the string uses; `count` only chooses the form.
 */
export function tn(
  key: PluralKey,
  count: number,
  params: Record<string, unknown> = {},
): string {
  return fill(translatePlural(key, count, current), params, current);
}

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

/**
 * Reactive translation with values: `tf("groups.since", { joined_at })` fills the
 * `{joined_at}` placeholder in the catalogue string. A placeholder with no value is
 * left as written, so a missing value is visible rather than blank. A number is written in the
 * page's own format; a date is passed through `formatDate` first (`locale-format.ts`).
 */
export function tf(
  key: StringKey,
  params: Record<string, unknown> = {},
): string {
  return fill(translate(key, current), params, current);
}

/**
 * A placeholder's value as text in `locale`: a number through `Intl.NumberFormat` (grouping, the locale's decimal
 * mark and digits: "1,234", "1.234", "১,২৩৪"), anything else as written (WPM-T375). A figure that must not be grouped,
 * such as a year, is passed already formatted (`formatNumber(year, { useGrouping: false })`).
 */
export function placeholderText(
  value: unknown,
  locale: Locale = current,
): string {
  if (typeof value === "number" && Number.isFinite(value)) {
    try {
      return new Intl.NumberFormat(locale).format(value);
    } catch {
      return String(value);
    }
  }
  return String(value);
}

function fill(
  text: string,
  params: Record<string, unknown>,
  locale: Locale,
): string {
  return text.replace(/\{(\w+)\}/g, (whole, name: string) =>
    name in params ? placeholderText(params[name], locale) : whole,
  );
}

/**
 * A closed-vocabulary value the server sends as a token (a status, a kind, a category:
 * `on_leave`, `ai_assistance`) in the current language. The catalogue holds them under
 * `values.<token>`. A token the catalogue does not know yet is shown humanized in the source
 * language (underscores to spaces) rather than blank, and a test pins that every token the
 * server's vocabularies define has an entry.
 */
export function tv(value: string | null | undefined): string {
  if (value === null || value === undefined) return "";
  return tp(`values.${value}`) ?? value.replaceAll("_", " ");
}

/** Reactive locale-prefixed link for an app path: `l("/workers")`. */
export function l(path: string): string {
  return localePath(current, path);
}

/**
 * Translate `key` and fill `{name}` placeholders from `params`; used for
 * server-derived findings whose figures arrive as data. Returns null when
 * the catalog has no such key (a newer server code than this client), so
 * the caller can fall back to the server's own text.
 */
export function tp(
  key: string,
  params: Record<string, unknown> = {},
  locale: Locale = current,
): string | null {
  if (!(key in STRINGS[DEFAULT_LOCALE])) return null;
  return fill(translate(key as StringKey, locale), params, locale);
}
