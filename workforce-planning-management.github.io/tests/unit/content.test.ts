import { describe, expect, it, vi } from "vitest";
import docs from "../../src/content/docs.json" with { type: "json" };
import {
  alternates,
  allBriefs,
  briefsOf,
  counts,
  findBrief,
  hasBrief,
  homeAlternates,
  joinRepoPath,
  localiseDates,
  privacyAlternates,
  productionAlternates,
  render,
  renderSegments,
  rewriteHref,
  searchDocs,
  sitemapRoutes,
  sitemapXml,
  slug,
  source,
  splitHero,
  startAlternates,
  titleOf,
} from "../../src/lib/server/content.js";
import { BRIEF_LOCALES, HOME_LOCALES, REPO_URL } from "../../src/lib/site.js";

// The module reads the files under `docs.files`; synthetic documents are added beside the real ones.
const FILES = docs.files as Record<string, string>;
const route = (r: string) => `/base${r}`;
const same = (r: string) => r;

describe("source, titleOf", () => {
  it("returns a document and names a missing one", () => {
    expect(source("README.md").length).toBeGreaterThan(0);
    expect(() => source("nope.md")).toThrow(/not in src\/content\/docs.json: nope.md/);
  });
  it("uses the first level-one heading, without its marks, else the path", () => {
    FILES["t/title.md"] = "intro\n\n# A `code` and **bold** title\n\n# Second";
    FILES["t/none.md"] = "no heading here";
    expect(titleOf("t/title.md")).toBe("A code and bold title");
    expect(titleOf("t/none.md")).toBe("t/none.md");
  });
});

describe("briefs", () => {
  it("finds the briefs of a locale and each by slug", () => {
    const first = docs.briefs[0]!;
    const locale = first.locale as (typeof BRIEF_LOCALES)[number];
    expect(briefsOf(locale).length).toBeGreaterThan(0);
    expect(hasBrief(locale, first.slug)).toBe(true);
    expect(hasBrief(locale, "no-such-brief")).toBe(false);
    expect(findBrief(locale, first.slug)?.path).toBe(first.path);
    expect(findBrief(locale, "no-such-brief")).toBeUndefined();
    expect(allBriefs().length).toBe(docs.briefs.filter((b) => (BRIEF_LOCALES as readonly string[]).includes(b.locale)).length);
  });
  it("links a brief to its peers by peer id, and falls back to a list", () => {
    const first = docs.briefs.find((b) => b.peer)!;
    const locale = first.locale as (typeof BRIEF_LOCALES)[number];
    const alt = alternates(locale, first.slug);
    for (const l of BRIEF_LOCALES) expect(alt[l]).toMatch(new RegExp(`^/${l}/for/`));
    expect(alt[locale]).toBe(`/${locale}/for/${first.slug}/`);
    const missing = alternates(locale, "no-such-brief");
    for (const l of BRIEF_LOCALES) expect(missing[l]).toBe(`/${l}/for/`);
    const lists = alternates(locale, null);
    expect(Object.keys(lists).length).toBe(docs.lists.length);
  });
  it("gives every locale a home, start, production and privacy address", () => {
    for (const l of BRIEF_LOCALES) {
      expect(homeAlternates()[l]).toBeTruthy();
      expect(startAlternates()[l]).toBeTruthy();
      expect(productionAlternates()[l]).toBe(`/${l}/path-to-production/`);
      expect(privacyAlternates()[l]).toBe(`/${l}/privacy/`);
    }
    const homeless = BRIEF_LOCALES.find((l) => !(HOME_LOCALES as readonly string[]).includes(l));
    if (homeless) expect(startAlternates()[homeless]).toBe(`/${homeless}/for/`);
  });
});

describe("slug", () => {
  it("makes a GitHub heading anchor", () => {
    expect(slug("  Hello, World! ")).toBe("hello-world");
    expect(slug("A `code` and **bold**")).toBe("a-code-and-bold");
    expect(slug("[link text](http://x) here")).toBe("link-text-here");
    expect(slug("Ünï côdé 日本語")).toBe("ünï-côdé-日本語");
    expect(slug("snake_case_ word")).toBe("snake_case-word");
  });
});

describe("joinRepoPath, rewriteHref", () => {
  it("joins onto the directory of the document, and refuses to climb out", () => {
    expect(joinRepoPath("a/b/c.md", "../d.md")).toBe("a/d.md");
    expect(joinRepoPath("a/b/c.md", "./e//f.md")).toBe("a/b/e/f.md");
    expect(joinRepoPath("a/b/c.md", "/x/y.md")).toBe("x/y.md");
    expect(joinRepoPath("c.md", "../x")).toBeNull();
  });
  it("keeps anchors and external links", () => {
    for (const h of ["#top", "https://example.com/x", "mailto:a@b.c", "//cdn.example/x"]) expect(rewriteHref("README.md", h, route)).toBe(h);
  });
  it("sends a document the site shows to its page, with the fragment", () => {
    expect(rewriteHref("README.md", "INSTALL.md#step", route)).toBe("/base/install/#step");
  });
  it("sends another repository path to GitHub, as a file or a folder", () => {
    expect(rewriteHref("README.md", "src/main.rs", route)).toBe(`${REPO_URL}/blob/main/src/main.rs`);
    expect(rewriteHref("README.md", "src/dir", route)).toBe(`${REPO_URL}/tree/main/src/dir`);
    expect(rewriteHref("README.md", "a%20b/c d.md#x", route)).toBe(`${REPO_URL}/blob/main/a%20b/c%20d.md#x`);
  });
  it("keeps a malformed escape as written, and a link out of the repository goes to its root", () => {
    expect(rewriteHref("README.md", "bad%zz.md", route)).toBe(`${REPO_URL}/blob/main/bad%25zz.md`);
    expect(rewriteHref("README.md", "../../out.md#f", route)).toBe(`${REPO_URL}#f`);
  });
  it("an empty path is the document itself", () => {
    expect(rewriteHref("INSTALL.md", "", route)).toBe("/base/install/");
  });
});

describe("localiseDates", () => {
  it("shows a date, a range and a backwards range in the locale, inside <time>", () => {
    expect(localiseDates("on 2026-10-11 ok", "en-GB")).toBe('on <time datetime="2026-10-11">11 Oct 2026</time> ok');
    expect(localiseDates("2026-10-01 to 2026-10-30", "en-GB")).toMatch(/^<time datetime="2026-10-01\/2026-10-30">.*Oct 2026<\/time>$/);
    expect(localiseDates("2026-10-30 to 2026-10-01", "en-GB")).toBe(
      '<time datetime="2026-10-30">30 Oct 2026</time> to <time datetime="2026-10-01">1 Oct 2026</time>',
    );
  });
  it("leaves timestamps, ids, impossible dates and other text alone, and escapes", () => {
    expect(localiseDates("2026-10-11T10:00", "en")).toBe("2026-10-11T10:00");
    expect(localiseDates("x2026-10-11", "en")).toBe("x2026-10-11");
    expect(localiseDates("2026-02-30", "en")).toBe("2026-02-30");
    expect(localiseDates("2026-10-11 to 2026-02-30", "en")).toContain("2026-02-30");
    expect(localiseDates('a < b & "c"', "en")).toBe("a &lt; b &amp; &quot;c&quot;");
  });
});

describe("splitHero", () => {
  it("divides before the first h2", () => {
    expect(splitHero("<h1>x</h1><p>y</p><h2>s</h2><p>z</p>")).toEqual({ hero: "<h1>x</h1><p>y</p>", rest: "<h2>s</h2><p>z</p>" });
    expect(splitHero("<p>only</p>")).toEqual({ hero: "<p>only</p>", rest: "" });
  });
});

describe("render", () => {
  it("renders headings with unique ids, the title, and escapes raw HTML", () => {
    FILES["t/r.md"] = "---\nfront: matter\n---\n# Title `x`\n\n## Same\n\n## Same\n\n<script>alert(1)</script>\n\ntext <b>raw</b> & 2026-10-11";
    const { title, html } = render("t/r.md", same, "en-GB");
    expect(title).toBe("Title x");
    expect(html).toContain('<h2 id="same">');
    expect(html).toContain('<h2 id="same-1">');
    expect(html).not.toContain("<script>");
    expect(html).toContain("&lt;script&gt;");
    expect(html).not.toContain("front: matter");
    expect(html).toContain('<time datetime="2026-10-11">11 Oct 2026</time>');
  });
  it("uses the path when there is no heading", () => {
    FILES["t/nohead.md"] = "just text";
    expect(render("t/nohead.md", same, "en").title).toBe("t/nohead.md");
  });
  it("marks a row of links as actions, and rewrites links through route", () => {
    FILES["zz-a.md"] = "[one](INSTALL.md) [two](README.md)\n\n[solo](INSTALL.md) text";
    const { html } = render("zz-a.md", route, "en");
    expect(html).toContain('<p class="actions">');
    expect(html).toContain('href="/base/install/"');
    expect(html.match(/class="actions"/g)?.length).toBe(1);
  });
  it("renders task boxes, code blocks and tables the way the site needs", () => {
    FILES["t/b.md"] =
      "- [x] done\n- [ ] todo\n\n```rust extra\nlet a = 1 < 2;\n```\n\n```\nplain\n```\n\n| h1 | h2 |\n| --- | --- |\n| a | **b** |\n";
    const { html } = render("t/b.md", same, "en");
    expect(html).toContain('disabled checked aria-label="Done"');
    expect(html).toContain('disabled aria-label="Not done"');
    expect(html).toContain('<pre tabindex="0"><code class="language-rust">let a = 1 &lt; 2;');
    expect(html).toContain('<pre tabindex="0"><code>plain');
    expect(html).toContain('<div class="table-scroll" tabindex="0"><table><thead><tr><th>h1</th><th>h2</th></tr></thead>');
    expect(html).toContain("<td><strong>b</strong></td>");
  });
  it("leaves a list alone unless it is a readiness list on a path-to-production page", () => {
    FILES["t/l.md"] = "- **75%** **Name one.** more\n- plain item\n";
    expect(render("t/l.md", same, "en").html).toContain("<ul>");
    const lists: never[][] = [];
    expect(render("t/l.md", same, "en", lists).html).toContain("<ul>");
    expect(lists.length).toBe(0);
  });
});

describe("renderSegments", () => {
  it("turns a readiness list into sliders' data, between the prose", () => {
    FILES["t/p.md"] = "# P\n\nBefore.\n\n- **75%** **First item.** its detail\n- **0%**  **Second.**\n\nAfter.\n";
    const { title, segments } = renderSegments("t/p.md", same, "en");
    expect(title).toBe("P");
    expect(segments.map((s) => s.kind)).toEqual(["html", "readiness", "html"]);
    const list = segments[1];
    expect(list?.kind === "readiness" && list.items.map((i) => [i.estimate, i.title])).toEqual([
      [75, "First item"],
      [0, "Second"],
    ]);
  });
  it("refuses an estimate above 100%, a missing name, and a page with no list", () => {
    FILES["t/q1.md"] = "- **101%** **Too high.**\n";
    FILES["t/q2.md"] = "- **50%** nameless\n";
    FILES["t/q3.md"] = "# nothing\n\ntext\n";
    expect(() => renderSegments("t/q1.md", same, "en")).toThrow(/above 100%/);
    expect(() => renderSegments("t/q2.md", same, "en")).toThrow(/no name in bold/);
    expect(() => renderSegments("t/q3.md", same, "en")).toThrow(/no readiness list/);
  });
  it("renders the real path-to-production pages", () => {
    const first = docs.productions[0]!;
    expect(renderSegments(first.path, same, first.locale).segments.some((s) => s.kind === "readiness")).toBe(true);
  });
});

describe("the site's index and sitemaps", () => {
  it("indexes every page with plain text", () => {
    const all = searchDocs();
    expect(all.length).toBeGreaterThan(docs.briefs.length);
    expect(all[0]?.route).toBe("/docs/");
    for (const d of all) expect(d.text).not.toMatch(/<\/?(p|h[1-6]|div|pre|table|ul|li|a|code)[ >]/);
  });
  it("lists the whole site, and one locale's pages", () => {
    const everything = sitemapRoutes();
    expect(everything[0]).toBe("/");
    expect(everything).toContain("/search/");
    const one = sitemapRoutes("it-001");
    expect(one).toContain("/it-001/");
    expect(one).toContain("/it-001/search/");
    expect(one.every((r) => r.startsWith("/it-001/"))).toBe(true);
    const homeless = BRIEF_LOCALES.find((l) => !(HOME_LOCALES as readonly string[]).includes(l));
    if (homeless) expect(sitemapRoutes(homeless)).not.toContain(`/${homeless}/start/`);
  });
  it("writes a sitemap with encoded, escaped addresses", () => {
    const xml = sitemapXml(["/a/", "/ü&x/"], "https://s.example");
    expect(xml).toContain("<loc>https://s.example/a/</loc>");
    expect(xml).toContain("<loc>https://s.example/%C3%BC&amp;x/</loc>");
    expect(xml.startsWith('<?xml version="1.0" encoding="UTF-8"?>')).toBe(true);
  });
  it("counts the documents, briefs and locales", () => {
    const c = counts();
    expect(c.briefs).toBe(docs.briefs.length);
    expect(c.locales).toBe(docs.lists.length);
    expect(c.documents).toBe(Object.keys(FILES).length);
  });
});

describe("a copy that disagrees with the site's locales fails at load", () => {
  for (const [field, message] of [
    ["lists", /has locales/],
    ["productions", /path to production/],
    ["privacies", /privacy notice/],
    ["homes", /has homes/],
  ] as const) {
    it(`${field}`, async () => {
      vi.resetModules();
      vi.doMock("../../src/content/docs.json", () => ({
        default: { ...docs, [field]: (docs[field] as unknown[]).slice(1) },
      }));
      await expect(import("../../src/lib/server/content.js")).rejects.toThrow(message);
      vi.doUnmock("../../src/content/docs.json");
      vi.resetModules();
    });
  }
});
