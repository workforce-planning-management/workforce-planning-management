// The site's pages and how a repository path maps to one. Safe for the browser: the document text itself is read
// only on the server side, during prerendering (`#lib/server/content.ts`).

export const REPO_URL = "https://github.com/workforce-planning-management/workforce-planning-management";

/** A page made from one repository document. `slug` is the route under the site root ("" is the home page). */
export type DocPage = { slug: string; path: string; nav: string };

export const DOC_PAGES: readonly DocPage[] = [
  { slug: "", path: "index.md", nav: "Home" },
  { slug: "readme", path: "README.md", nav: "README" },
  { slug: "install", path: "INSTALL.md", nav: "Install" },
  { slug: "family-bridges", path: "docs/tutorials/family-bridges.md", nav: "Family bridges" },
  { slug: "spec", path: "spec/index.md", nav: "Specification" },
  { slug: "worldwide", path: "spec/worldwide.md", nav: "Worldwide" },
];

/**
 * The locales that have role briefs (`locales/<locale>/for/`), in the repository's spelling, sorted by code (the
 * Lily LocalePicker contract). `#lib/server/content.ts` fails the build if `src/content/docs.json` holds another set.
 */
export const BRIEF_LOCALES = [
  "ar-001",
  "bn-001",
  "cy-001",
  "cy-gb",
  "de-001",
  "de-de",
  "en-001",
  "en-gb",
  "en-gb-oxendict",
  "en-us",
  "es-001",
  "es-es",
  "fr-001",
  "hi-001",
  "hi-in",
  "id-001",
  "pt-001",
  "ru-001",
  "ur-001",
  "zh-001",
  "zh-cn",
] as const;
export type BriefLocale = (typeof BRIEF_LOCALES)[number];

/** The locale the navigation's "Role briefs" link opens: the English source the translations defer to. */
export const DEFAULT_BRIEF_LOCALE: BriefLocale = "en-gb-oxendict";

/** The locale of the site's own English pages (the documents and the navigation), for dates and numbers. */
export const SITE_LOCALE = "en-001";

/**
 * Each locale's name for the locale switcher, written in that locale: the WPM UI's `LOCALE_LABELS`
 * (`workforce-planning-management-ui-with-svelte/src/lib/locales.ts`) for its 17, and the same pattern for the four
 * regional brief locales the UI does not have.
 */
export const LOCALE_LABELS: Record<BriefLocale, string> = {
  "ar-001": "العربية",
  "bn-001": "বাংলা",
  "cy-001": "Cymraeg",
  "cy-gb": "Cymraeg - Y Deyrnas Unedig",
  "de-001": "Deutsch",
  "de-de": "Deutsch - Deutschland",
  "en-001": "English",
  "en-gb": "English - Great Britain",
  "en-gb-oxendict": "English - Great Britain (Oxford spelling)",
  "en-us": "English - United States",
  "es-001": "Español",
  "es-es": "Español - España",
  "fr-001": "Français",
  "hi-001": "हिन्दी",
  "hi-in": "हिन्दी - भारत",
  "id-001": "Bahasa Indonesia",
  "pt-001": "Português",
  "ru-001": "Русский",
  "ur-001": "اردو",
  "zh-001": "中文",
  "zh-cn": "中文 - 中国",
};

export function isBriefLocale(s: string): s is BriefLocale {
  return (BRIEF_LOCALES as readonly string[]).includes(s);
}

/** Right-to-left languages (primary subtag): the WPM UI's list. A page in one of them is laid out from the right. */
const RTL_LANGUAGES = ["ar", "ur"];

/** Whether `locale` is written right to left (`ar-001`, `ur-001`). */
export function isRtl(locale: string): boolean {
  return RTL_LANGUAGES.includes(locale.split(/[-_]/)[0]?.toLowerCase() ?? "");
}

/** The locale named by a site pathname's first segment, if it is one. */
export function localeOfPath(pathname: string, base = ""): BriefLocale | null {
  const rest = pathname.startsWith(base) ? pathname.slice(base.length) : pathname;
  const first = rest.split("/").filter(Boolean)[0] ?? "";
  return isBriefLocale(first) ? first : null;
}

/** The route of a role brief, or of a locale's list of briefs when `slug` is null. */
export function briefRoute(locale: BriefLocale, slug: string | null): string {
  return slug ? `/${locale}/for/${slug}/` : `/${locale}/for/`;
}

/**
 * The site route ("/install/", "/cy-001/for/prif-swyddog-risg/") of a repository path, or null if it has none.
 * `hasBrief` says whether a locale has a brief under that directory name.
 */
export function routeOfRepoPath(path: string, hasBrief: (locale: BriefLocale, slug: string) => boolean): string | null {
  const clean = path.replace(/\/+$/, "");
  for (const d of DOC_PAGES) if (clean === d.path) return d.slug ? `/${d.slug}/` : "/";
  if (clean === "spec") return "/spec/";
  const m = /^locales\/([^/]+)\/for(?:\/([^/]+))?(?:\/(?:index|README)\.md)?$/.exec(clean);
  if (m && m[1] && isBriefLocale(m[1])) {
    if (!m[2] || m[2] === "index.md" || m[2] === "README.md") return briefRoute(m[1], null);
    if (hasBrief(m[1], m[2])) return briefRoute(m[1], m[2]);
  }
  return null;
}
