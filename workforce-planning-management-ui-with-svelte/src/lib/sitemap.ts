// The sitemap and robots.txt, generated from the locale list and the public
// pages (`publicPages.ts`). Pure: the origin is a parameter, so it is unit
// tested and the routes only supply `url.origin` (or `WPM_PUBLIC_URL` behind a
// proxy).
//
// Each public page appears once per locale, with `xhtml:link rel="alternate"`
// entries pointing at its siblings in every other locale (and `x-default` at
// `en-001`), so search engines serve the right language. A bare language
// alias (`/en/…`) is a redirect and is deliberately not listed.

import { INDEXABLE_PAGES } from "./publicPages";
import { LOCALES, localePath, type Locale } from "./locales";

/** The default locale search engines should fall back to. */
const X_DEFAULT: Locale = "en-001";

/**
 * The `hreflang` for a content locale. Search engines accept a language with an
 * optional ISO 3166-1 *alpha-2* region, not the UN numeric `001`, so a
 * `<language>-001` locale is just its language (`cy-001` → `cy`) and a regional
 * one is `language-REGION` (`en-gb` → `en-GB`).
 */
export function hreflang(locale: Locale): string {
  const [language, region] = locale.split("-");
  if (!language) return locale;
  if (!region || region === "001") return language;
  return `${language}-${region.toUpperCase()}`;
}

/** One URL in the sitemap with its alternates. */
export interface SitemapUrl {
  loc: string;
  alternates: Array<{ hreflang: string; href: string }>;
}

/** `origin` without a trailing slash. */
const base = (origin: string): string => origin.replace(/\/+$/, "");

/** Every locale's URL for every indexable page. */
export function sitemapUrls(
  origin: string,
  locales: readonly Locale[] = LOCALES,
  pages: readonly string[] = INDEXABLE_PAGES,
): SitemapUrl[] {
  const root = base(origin);
  const out: SitemapUrl[] = [];
  for (const page of pages) {
    const alternates = [
      ...locales.map((l) => ({
        hreflang: hreflang(l),
        href: `${root}${localePath(l, page)}`,
      })),
      { hreflang: "x-default", href: `${root}${localePath(X_DEFAULT, page)}` },
    ];
    // Two locales can share an hreflang only if the locale list is malformed;
    // keep the first of any duplicate so the XML stays valid.
    const seen = new Set<string>();
    const unique = alternates.filter(
      (a) => !seen.has(a.hreflang) && seen.add(a.hreflang),
    );
    for (const l of locales) {
      out.push({ loc: `${root}${localePath(l, page)}`, alternates: unique });
    }
  }
  return out;
}

/** XML-escape text for an element or attribute. */
export function xmlEscape(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&apos;");
}

/** The `sitemap.xml` document. */
export function renderSitemap(origin: string): string {
  const urls = sitemapUrls(origin)
    .map((u) => {
      const links = u.alternates
        .map(
          (a) =>
            `    <xhtml:link rel="alternate" hreflang="${xmlEscape(a.hreflang)}" href="${xmlEscape(a.href)}"/>`,
        )
        .join("\n");
      return `  <url>\n    <loc>${xmlEscape(u.loc)}</loc>\n${links}\n  </url>`;
    })
    .join("\n");
  return `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:xhtml="http://www.w3.org/1999/xhtml">
${urls}
</urlset>
`;
}

/** The `robots.txt` document: the app is for signed-in users, so only the public
 *  pages are offered, and the API, the CMS and sign-out are kept out. */
export function renderRobots(origin: string): string {
  return `User-agent: *
Allow: /
Disallow: /api/
Disallow: /admin
Disallow: /*/admin
Disallow: /signout

Sitemap: ${base(origin)}/sitemap.xml
`;
}
