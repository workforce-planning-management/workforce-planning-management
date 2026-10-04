// Locale codes and URL mapping, shared by the router hooks, the i18n
// store and the layout. Pure and framework-free (no `$app` imports) so
// the hooks and unit tests can load it.
//
// Content locales follow the family convention (see
// public-value-metrics): `<language>-001` is the language's "World"
// variant, `<language>-<region>` a regional one. Each is served under
// its own URL prefix — `/en-001/workers`, `/cy-001/workers` — and a bare
// two-letter alias (`/en/…`, `/cy/…`) redirects to its `-001` locale, so
// every locale has one canonical address.

/** Content locales, sorted by code (the Lily LocalePicker contract). */
export const LOCALES = [
  "ar-001",
  "bn-001",
  "cy-001",
  "de-001",
  "de-de",
  "en-001",
  "en-gb",
  "en-us",
  "es-001",
  "es-es",
  "fr-001",
  "hi-001",
  "id-001",
  "pt-001",
  "ru-001",
  "ur-001",
  "zh-001",
] as const;

/** A supported content locale (one of {@link LOCALES}). */
export type Locale = (typeof LOCALES)[number];

/** Fallback locale for an unknown key, locale, or missing translation. */
export const DEFAULT_LOCALE: Locale = "en-001";

/** Bare language code -> its `-001` locale (`en` -> `en-001`). */
export const LOCALE_ALIASES: Readonly<Record<string, Locale>> =
  Object.fromEntries(
    LOCALES.filter((l) => l.endsWith("-001")).map((l) => [l.slice(0, -4), l]),
  );

/** Human-readable name for the locale switcher, written in that locale. */
export const LOCALE_LABELS: Record<Locale, string> = {
  "ar-001": "العربية",
  "bn-001": "বাংলা",
  "cy-001": "Cymraeg",
  "de-001": "Deutsch",
  "de-de": "Deutsch - Deutschland",
  "en-001": "English",
  "en-gb": "English - Great Britain",
  "en-us": "English - United States",
  "es-001": "Español",
  "es-es": "Español - España",
  "fr-001": "Français",
  "hi-001": "हिन्दी",
  "id-001": "Bahasa Indonesia",
  "pt-001": "Português",
  "ru-001": "Русский",
  "ur-001": "اردو",
  "zh-001": "中文",
};

/** Right-to-left languages (primary subtag); the layout mirrors `<html dir>`. */
const RTL_LANGUAGES = ["ar", "ur"];

/** Whether `locale` (any spelling) is written right-to-left. */
export function isRtl(locale: string): boolean {
  const primary = locale.split(/[-_]/)[0]?.toLowerCase() ?? "";
  return RTL_LANGUAGES.includes(primary);
}

/**
 * Normalise raw input to a content locale, or null if unsupported.
 * Accepts the content code (`en-001`), a bare alias (`en`), the legacy
 * underscore spellings (`en_US`, `de_DE`) and any region subtag of a
 * supported language (`es-MX` -> `es-001`); case-insensitive.
 */
export function normaliseLocale(raw: string | null | undefined): Locale | null {
  if (!raw) return null;
  const normalized = raw.trim().replace(/_/g, "-").toLowerCase();
  const exact = LOCALES.find((l) => l === normalized);
  if (exact) return exact;
  const primary = normalized.split("-")[0] ?? "";
  return LOCALE_ALIASES[primary] ?? null;
}

/** A URL path split into its locale prefix and the rest. */
export interface SplitPath {
  /** Canonical locale named by the prefix, or null when there is none. */
  locale: Locale | null;
  /** True when the prefix was a bare alias (`/en/…`) to redirect. */
  alias: boolean;
  /** The path with the prefix removed; always starts with `/`. */
  rest: string;
}

/** Split `pathname` into its locale prefix (content code or alias) and the rest. */
export function splitLocale(pathname: string): SplitPath {
  const match = /^\/([^/]+)(\/.*)?$/.exec(pathname);
  const head = match?.[1]?.toLowerCase();
  const rest = match?.[2] ?? "/";
  if (head) {
    const exact = LOCALES.find((l) => l === head);
    if (exact) return { locale: exact, alias: false, rest };
    const aliased = LOCALE_ALIASES[head];
    if (aliased) return { locale: aliased, alias: true, rest };
  }
  return { locale: null, alias: false, rest: pathname };
}

/** Prefix an app path with `locale`: `("cy-001", "/workers")` -> `/cy-001/workers`. */
export function localePath(locale: Locale, path: string): string {
  if (/^[a-z][a-z0-9+.-]*:|^\/\/|^#/i.test(path)) return path;
  const [target, ...hash] = path.split("#");
  const suffix = hash.length ? `#${hash.join("#")}` : "";
  const clean = target === "/" || target === "" ? "" : target;
  return `/${locale}${clean}${suffix}`;
}

/** Pick the best locale from an `Accept-Language` header, or the default. */
export function negotiateLocale(header: string | null | undefined): Locale {
  for (const part of (header ?? "").split(",")) {
    const found = normaliseLocale(part.split(";")[0]);
    if (found) return found;
  }
  return DEFAULT_LOCALE;
}

/** Interface languages Sveltia CMS ships (its own list, not ours). */
export const CMS_UI_LANGUAGES = [
  "ar",
  "bg",
  "ca",
  "cs",
  "da",
  "de",
  "el",
  "en-CA",
  "en-GB",
  "en-US",
  "es-CO",
  "fa",
  "fi",
  "fr",
  "hr",
  "it",
  "ja",
  "ko",
  "nl",
  "pl",
  "pt-BR",
  "pt-PT",
  "ru",
  "sv",
  "tr",
  "uk",
  "vi",
  "zh-CN",
  "zh-TW",
] as const;

/**
 * The Sveltia CMS interface language for each content locale: the closest
 * one the CMS ships. Bengali, Welsh, Hindi, Indonesian and Urdu have no
 * CMS translation, so they get English (Welsh: British English).
 */
export const CMS_UI_LOCALE: Record<Locale, (typeof CMS_UI_LANGUAGES)[number]> =
  {
    "ar-001": "ar",
    "bn-001": "en-US",
    "cy-001": "en-GB",
    "de-001": "de",
    "de-de": "de",
    "en-001": "en-US",
    "en-gb": "en-GB",
    "en-us": "en-US",
    "es-001": "es-CO",
    "es-es": "es-CO",
    "fr-001": "fr",
    "hi-001": "en-US",
    "id-001": "en-US",
    "pt-001": "pt-BR",
    "ru-001": "ru",
    "ur-001": "en-US",
    "zh-001": "zh-CN",
  };
