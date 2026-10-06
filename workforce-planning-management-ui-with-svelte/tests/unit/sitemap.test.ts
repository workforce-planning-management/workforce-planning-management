import { describe, expect, it } from "vitest";
import { LOCALES } from "../../src/lib/locales";
import { INDEXABLE_PAGES, isPublicPath } from "../../src/lib/publicPages";
import {
  hreflang,
  renderRobots,
  renderSitemap,
  sitemapUrls,
  xmlEscape,
} from "../../src/lib/sitemap";

describe("sitemap", () => {
  it("lists every indexable page in every locale", () => {
    const urls = sitemapUrls("https://wpm.example.org");
    expect(urls).toHaveLength(LOCALES.length * INDEXABLE_PAGES.length);
    const locs = urls.map((u) => u.loc);
    expect(locs).toContain("https://wpm.example.org/en-001");
    expect(locs).toContain("https://wpm.example.org/cy-001/tour");
    expect(locs).toContain("https://wpm.example.org/es-es/signin");
    expect(new Set(locs).size).toBe(locs.length);
  });

  it("only ever offers pages the sign-in gate leaves public", () => {
    for (const page of INDEXABLE_PAGES)
      expect(isPublicPath(page), page).toBe(true);
    // The gate: signed-in pages are not public, and "/" is exact-matched.
    expect(isPublicPath("/workers")).toBe(false);
    expect(isPublicPath("/ceo")).toBe(false);
    expect(isPublicPath("/")).toBe(true);
    // A one-time-token landing page is public but not worth indexing.
    expect(isPublicPath("/verify")).toBe(true);
    expect(INDEXABLE_PAGES).not.toContain("/verify");
  });

  it("maps locale codes to hreflang search engines accept", () => {
    expect(hreflang("en-001")).toBe("en");
    expect(hreflang("cy-001")).toBe("cy");
    expect(hreflang("en-gb")).toBe("en-GB");
    expect(hreflang("de-de")).toBe("de-DE");
    expect(hreflang("es-es")).toBe("es-ES");
    for (const l of LOCALES)
      expect(hreflang(l), l).toMatch(/^[a-z]{2}(-[A-Z]{2})?$/);
  });

  it("gives every URL an alternate per locale plus x-default, with no duplicates", () => {
    const [first] = sitemapUrls("https://wpm.example.org/");
    const tags = first!.alternates.map((a) => a.hreflang);
    expect(new Set(tags).size).toBe(tags.length);
    expect(tags).toContain("x-default");
    expect(tags).toContain("en-GB");
    expect(
      first!.alternates.find((a) => a.hreflang === "x-default")!.href,
    ).toBe("https://wpm.example.org/en-001");
    // A trailing slash on the origin does not double up.
    expect(first!.loc).toBe("https://wpm.example.org/ar-001");
  });

  it("renders valid, escaped XML", () => {
    const xml = renderSitemap("https://wpm.example.org");
    expect(xml.startsWith('<?xml version="1.0" encoding="UTF-8"?>')).toBe(true);
    expect(xml).toContain('xmlns:xhtml="http://www.w3.org/1999/xhtml"');
    expect(xml.match(/<url>/g)).toHaveLength(
      LOCALES.length * INDEXABLE_PAGES.length,
    );
    expect(xml).toContain(
      '<xhtml:link rel="alternate" hreflang="cy" href="https://wpm.example.org/cy-001/tour"/>',
    );
    // An origin with a query-ish character is escaped, never raw.
    const odd = renderSitemap("https://x.example/?a=1&b=2");
    expect(odd).not.toMatch(/&b=2(?!;)/);
    expect(xmlEscape(`<a href="x">&'`)).toBe(
      "&lt;a href=&quot;x&quot;&gt;&amp;&apos;",
    );
  });

  it("serves a robots.txt that names the sitemap and keeps crawlers off the API and CMS", () => {
    const robots = renderRobots("https://wpm.example.org/");
    expect(robots).toContain("Sitemap: https://wpm.example.org/sitemap.xml");
    expect(robots).toContain("Disallow: /api/");
    expect(robots).toContain("Disallow: /*/admin");
    expect(robots).toContain("Disallow: /signout");
  });
});
