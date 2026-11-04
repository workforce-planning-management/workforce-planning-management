// The built site: every page is there; every picker in the Lily picker bar works; the search finds pages; axe finds no serious or critical WCAG 2.2 AA violation under the light and the
// dark theme; the locale switcher opens the same brief in another locale; an Arabic or Urdu page is laid out from the
// right without overflowing; dates read as the WPM UI's locale-format.ts shows them; and the content security policy
// blocks nothing the site itself does.
import { test, expect, type Page } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";
import docs from "../../src/content/docs.json" with { type: "json" };
import { readdirSync } from "node:fs";
import { BRIEF_LOCALES, COMPARE_SLUGS, DOC_PAGES, HOME_LOCALES, LOCALE_LABELS, pickerLocales } from "../../src/lib/site.ts";
import { catalogues, ui } from "../../src/lib/i18n.ts";
import { search } from "../../src/lib/search.ts";

type Brief = { locale: string; slug: string; peer: string | null };
const BRIEFS: Brief[] = docs.briefs;

// Every Lily theme, as sync-themes copied them from the installed package.
const THEMES = readdirSync("static/assets/themes").map((f) => f.replace(/\.css$/, ""));

const compareRoutes = ["/compare/", ...COMPARE_SLUGS.map((c) => `/compare/${c}/`)];
const docRoutes = ["/", ...DOC_PAGES.map((d) => `/${d.slug}/`), ...compareRoutes];
const homeRoutes = HOME_LOCALES.flatMap((l) => [`/${l}/`, `/${l}/start/`]);
const listRoutes = BRIEF_LOCALES.map((l) => `/${l}/for/`);
const briefRoute = (b: Brief) => `/${b.locale}/for/${b.slug}/`;
const briefRoutes = BRIEFS.map(briefRoute);

/** The brief of `locale` that is the same page as `peerOf`'s (by `.locale-peer-id`). */
function brief(locale: string, peerOf: string): Brief {
  const peer = BRIEFS.find((b) => b.slug === peerOf)?.peer;
  const found = BRIEFS.find((b) => b.locale === locale && b.peer === peer);
  if (!found) throw new Error(`no ${locale} brief for ${peerOf}`);
  return found;
}

// Audited: every document, every locale's list, and one brief per locale (a different role each time).
const audited = [
  ...docRoutes,
  ...homeRoutes,
  ...listRoutes,
  ...BRIEF_LOCALES.map((l, i) => {
    const own = BRIEFS.filter((b) => b.locale === l);
    return briefRoute(own[i % own.length] as Brief);
  }),
];

/** Load a page under a colour scheme and wait until ThemePicker has applied the matching Lily theme. */
async function open(page: Page, route: string, scheme: "light" | "dark") {
  await page.emulateMedia({ colorScheme: scheme });
  await page.goto(route, { waitUntil: "networkidle" });
  await page.waitForFunction(
    (t) =>
      document.documentElement.dataset.theme === t &&
      [...document.styleSheets].some((s) => (s.href ?? "").endsWith(`/assets/themes/${t}.css`)),
    scheme,
  );
}

async function axe(page: Page): Promise<string[]> {
  const results = await new AxeBuilder({ page })
    .withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa"])
    .analyze();
  return results.violations
    .filter((v) => v.impact === "serious" || v.impact === "critical")
    .map((v) => `${v.id}: ${v.nodes.slice(0, 3).map((n) => n.target.join(" ")).join("; ")}`);
}

async function overflow(page: Page): Promise<number> {
  return page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
}

test("every page is built and answers 200", async ({ request }) => {
  const all = [...docRoutes, ...homeRoutes, ...listRoutes, ...briefRoutes];
  expect(all.length).toBe(2 + COMPARE_SLUGS.length + DOC_PAGES.length + 2 * HOME_LOCALES.length + BRIEF_LOCALES.length + BRIEFS.length);
  expect(BRIEFS.length).toBe(BRIEF_LOCALES.length * 11);
  for (const route of all) {
    const res = await request.get(encodeURI(route));
    expect(res.status(), route).toBe(200);
  }
});

for (const route of audited)
  for (const scheme of ["light", "dark"] as const)
    test(`${route} passes axe (${scheme})`, async ({ page }) => {
      const csp: string[] = [];
      page.on("console", (m) => {
        if (/Content Security Policy/i.test(m.text())) csp.push(m.text());
      });
      await open(page, encodeURI(route), scheme);
      await expect(page.locator("h1").first()).toBeVisible();
      expect(await axe(page)).toEqual([]);
      expect(csp).toEqual([]);
    });

test("a brief's html carries its locale and direction; a document is English", async ({ page }) => {
  await page.goto(encodeURI(briefRoute(brief("cy-001", "chief-risk-officer"))));
  await expect(page.locator("html")).toHaveAttribute("lang", "cy-001");
  await expect(page.locator("html")).toHaveAttribute("dir", "ltr");
  for (const l of ["ar-001", "ur-001"]) {
    await page.goto(encodeURI(briefRoute(brief(l, "chief-risk-officer"))));
    await expect(page.locator("html")).toHaveAttribute("lang", l);
    await expect(page.locator("html")).toHaveAttribute("dir", "rtl");
  }
  await page.goto("/install/");
  await expect(page.locator("html")).toHaveAttribute("lang", "en");
  await expect(page.locator("html")).toHaveAttribute("dir", "ltr");
});

test("the locale switcher opens the same brief in the chosen locale, by its peer id", async ({ page }) => {
  await open(page, "/en-001/for/chief-risk-officer/", "light");
  await page.getByRole("button", { name: "Language" }).click();
  await page.locator(".locale-picker-option").filter({ hasText: /^\s*Cymraeg\s*$/ }).click();
  await expect(page).toHaveURL(new RegExp(`/cy-001/for/${brief("cy-001", "chief-risk-officer").slug}/$`));
  await expect(page.locator("html")).toHaveAttribute("lang", "cy-001");

  await page.getByRole("button", { name: ui("cy-001").pickers.language }).click();
  await page.locator(".locale-picker-option").filter({ hasText: /^\s*العربية\s*$/ }).click();
  const ar = brief("ar-001", "chief-risk-officer");
  await expect(page).toHaveURL(encodeURI(briefRoute(ar)));
  await expect(page.locator("html")).toHaveAttribute("dir", "rtl");
  await expect(page.locator("h1").first()).toContainText("المخاطر");
});

test("the locale switcher on a list opens the other locale's list; on an English document, its home or list", async ({
  page,
}) => {
  await open(page, "/de-001/for/", "light");
  await page.getByRole("button", { name: "Language" }).click();
  await page.locator(".locale-picker-option").filter({ hasText: /^\s*Français\s*$/ }).click();
  await expect(page).toHaveURL(/\/fr-001\/for\/$/);
  await open(page, "/readme/", "light");
  await page.getByRole("button", { name: "Language" }).click();
  await page.locator(".locale-picker-option").filter({ hasText: /^\s*Deutsch\s*$/ }).click();
  await expect(page).toHaveURL(/\/de-001\/for\/$/);
  await open(page, "/readme/", "light");
  await page.getByRole("button", { name: "Language" }).click();
  await page.locator(".locale-picker-option").filter({ hasText: /^\s*Cymraeg\s*$/ }).click();
  await expect(page).toHaveURL(/\/cy-001\/$/);
});

test("dates read as the WPM UI shows them: the page's locale, a range with the locale's own separator", async ({
  page,
}) => {
  const fmt = (loc: string, iso: string) =>
    new Intl.DateTimeFormat(loc, { dateStyle: "medium", timeZone: "UTC" }).format(new Date(`${iso}T00:00:00Z`));
  for (const l of ["de-001", "ar-001", "zh-001", "en-001"]) {
    await page.goto(encodeURI(briefRoute(brief(l, "chief-executive-officer"))));
    const time = page.locator('main time[datetime="2026-10-11"]').first();
    await expect(time, l).toHaveText(fmt(l, "2026-10-11"));
  }
  await page.goto("/readme/");
  const range = page.locator('main time[datetime="2026-07-18/2026-10-07"]');
  await expect(range).toHaveText(
    new Intl.DateTimeFormat("en-001", { dateStyle: "medium", timeZone: "UTC" }).formatRange(
      new Date("2026-07-18T00:00:00Z"),
      new Date("2026-10-07T00:00:00Z"),
    ),
  );
  // A date inside code is code: left as written.
  await page.goto("/family-bridges/");
  await expect(page.locator("main pre").filter({ hasText: "2026-10-30T10:00:00Z" }).first()).toBeVisible();
  await expect(page.locator("main pre time")).toHaveCount(0);
});

test("the footer's figures are formatted numbers from the copy of the documents", async ({ page }) => {
  await page.goto("/");
  const n = (x: number) => new Intl.NumberFormat("en-001", { maximumFractionDigits: 0 }).format(x);
  await expect(page.getByTestId("site-counts")).toHaveText(
    `${n(Object.keys(docs.files).length)} documents and ${n(BRIEFS.length)} role briefs in ${n(BRIEF_LOCALES.length)} locales`,
  );
});

test("links between the site's own documents stay on the site; others go to the repository", async ({ page }) => {
  await page.goto("/docs/");
  await expect(page.locator("main a", { hasText: "README.md" }).first()).toHaveAttribute("href", /readme\/$/);
  await expect(page.locator("main a", { hasText: "spec/readiness-assessment.md" }).first()).toHaveAttribute(
    "href",
    "https://github.com/workforce-planning-management/workforce-planning-management/blob/main/spec/readiness-assessment.md",
  );
  // A list's links to its briefs, with translated directory names, stay on the site.
  await page.goto("/ar-001/for/");
  const first = page.locator("main table a").first();
  await first.click();
  await expect(page).toHaveURL(/\/ar-001\/for\/[^/]+\/$/);
  await expect(page.getByTestId("up")).toHaveText("وثائق للأدوار");
});

for (const route of ["/ar-001/for/", briefRoute(brief("ar-001", "chief-executive-officer")), briefRoute(brief("ur-001", "chief-people-officer"))])
  for (const scheme of ["light", "dark"] as const)
    test(`${route} is laid out from the right without overflow (${scheme})`, async ({ page }) => {
      await open(page, encodeURI(route), scheme);
      expect(await overflow(page)).toBeLessThanOrEqual(0);
      const width = page.viewportSize()?.width ?? 0;
      // The brand sits at the inline start (the right), the pickers at the inline end (the left).
      const brand = await page.locator(".brand").boundingBox();
      const pickers = await page.locator(".pickers").boundingBox();
      expect(brand && brand.x + brand.width).toBeGreaterThan(width / 2);
      expect(pickers && pickers.x).toBeLessThan(width / 2);
      // The heading's text starts at the right edge of the column.
      const h1 = await page.locator("main h1").first().evaluate((e) => getComputedStyle(e).direction);
      expect(h1).toBe("rtl");
      expect(await axe(page)).toEqual([]);
    });

test("at phone width no page scrolls sideways, in either direction", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  for (const route of [
    ...docRoutes,
    "/en-001/for/",
    "/ar-001/for/",
    briefRoute(brief("ar-001", "chief-technology-officer")),
    briefRoute(brief("hi-in", "chief-technology-officer")),
  ]) {
    await page.goto(encodeURI(route));
    expect(await overflow(page), route).toBeLessThanOrEqual(0);
  }
});

test("no element carries a style attribute (the CSP allows only SvelteKit's announcer)", async ({ page }) => {
  for (const route of ["/", "/spec/", briefRoute(brief("ar-001", "chief-legal-officer"))]) {
    await open(page, encodeURI(route), "dark");
    const styled = await page.evaluate(() =>
      [...document.querySelectorAll("[style]")]
        .filter((e) => e.id !== "svelte-announcer")
        .map((e) => e.outerHTML.slice(0, 120)),
    );
    expect(styled, route).toEqual([]);
  }
});

const PICKERS = [
  { button: "Pages", popup: ".link-picker-list" },
  { button: "Search this site", popup: ".search-picker-panel" },
  { button: "Theme", popup: ".theme-picker-list" },
  { button: "Language", popup: ".locale-picker-list" },
  { button: "Text size", popup: ".text-size-picker-list" },
  { button: "Share", popup: ".share-picker-list" },
];

/** The list on a device with no share sheet, so the destinations show. */
const noShareSheet = () => {
  Object.defineProperty(navigator, "share", { value: undefined, configurable: true });
};

for (const [name, width] of [["desktop", 1280], ["phone", 390], ["small phone", 320]] as const)
  for (const route of ["/readme/", encodeURI(briefRoute(brief("ar-001", "chief-executive-officer")))])
    test(`every picker opens in full, inside the screen, without moving the page: ${route} (${name})`, async ({ page }) => {
      await page.addInitScript(noShareSheet);
      await page.setViewportSize({ width, height: 800 });
      await open(page, route, "light");
      const tops = [];
      for (const { button } of PICKERS) {
        const box = await page.getByRole("button", { name: button, exact: true }).boundingBox();
        tops.push(box);
        expect(box?.width, button).toBeGreaterThanOrEqual(24);
      }
      if (width > 390) expect(new Set(tops.map((b) => b?.y)).size).toBe(1);
      for (const { button, popup } of PICKERS) {
        const scroll = await page.evaluate(() => [scrollX, scrollY]);
        await page.getByRole("button", { name: button, exact: true }).click();
        const list = page.locator(popup);
        await expect(list, button).toBeVisible();
        const box = await list.boundingBox();
        const under = await page.getByRole("button", { name: button, exact: true }).boundingBox();
        expect(box && under && box.y >= under.y + under.height - 1, `${button} under its button`).toBe(true);
        expect(box && box.x, `${button} start`).toBeGreaterThanOrEqual(0);
        expect(box && box.x + box.width, `${button} end`).toBeLessThanOrEqual(width);
        // Wide enough: no line of the popup is cut off.
        expect(await list.evaluate((e) => e.scrollWidth - e.clientWidth), `${button} text`).toBeLessThanOrEqual(1);
        expect(await page.evaluate(() => [scrollX, scrollY]), `${button} jump`).toEqual(scroll);
        expect(await overflow(page), `${button} overflow`).toBeLessThanOrEqual(0);
        await page.keyboard.press("Escape");
        await expect(list, button).toBeHidden();
      }
    });

test("hovering each picker button changes its colour", async ({ page }) => {
  await open(page, "/readme/", "light");
  for (const { button } of PICKERS) {
    const control = page.getByRole("button", { name: button, exact: true });
    const colours = () => control.evaluate((e) => getComputedStyle(e).backgroundColor + getComputedStyle(e).color);
    await page.mouse.move(0, 700);
    const before = await colours();
    await control.hover();
    await expect.poll(colours, button).not.toBe(before);
  }
});

test("the bar offers every Lily theme, the UK and US themes last, and every text size", async ({ page }) => {
  await open(page, "/readme/", "light");
  await page.getByRole("button", { name: "Theme", exact: true }).click();
  const themes = await page.locator(".theme-picker-option").allTextContents();
  expect(themes).toHaveLength(THEMES.length);
  const national = themes.map((t) => /^\s*United (Kingdom|States)/.test(t));
  expect(national.indexOf(true)).toBe(national.lastIndexOf(false) + 1);
  await page.keyboard.press("Escape");
  await page.getByRole("button", { name: "Text size", exact: true }).click();
  expect((await page.locator(".text-size-picker-option").allTextContents()).map((t) => t.trim())).toEqual([
    "Largest",
    "Larger",
    "Large",
    "Normal",
    "Small",
    "Smaller",
    "Smallest",
  ]);
});

test("the language list is sorted by code, has no parentheses, and offers cy-001 and not cy-gb", async ({ page }) => {
  await open(page, "/readme/", "light");
  await page.getByRole("button", { name: "Language", exact: true }).click();
  const labels = (await page.locator(".locale-picker-option").allTextContents()).map((t) => t.trim());
  expect(labels).toEqual(pickerLocales(null).map((l) => LOCALE_LABELS[l]));
  expect(labels).toHaveLength(BRIEF_LOCALES.length - 1);
  expect(labels.filter((l) => /[()]/.test(l))).toEqual([]);
  expect(labels).not.toContain(LOCALE_LABELS["cy-gb"]);
  await open(page, "/cy-gb/for/", "light");
  await page.getByRole("button", { name: ui("cy-gb").pickers.language, exact: true }).click();
  await expect(page.locator(".locale-picker-option[aria-selected=true]")).toContainText(LOCALE_LABELS["cy-gb"]);
});

test("the pages picker goes to a page without reloading", async ({ page }) => {
  await open(page, "/readme/", "light");
  await page.getByRole("button", { name: "Pages", exact: true }).click();
  await page.locator(".link-picker-link").filter({ hasText: "Install" }).click();
  await expect(page).toHaveURL(/\/install\/$/);
});

test("the share picker copies the link and builds each destination", async ({ page, context }) => {
  await page.addInitScript(noShareSheet);
  await context.grantPermissions(["clipboard-read", "clipboard-write"]);
  await open(page, "/install/", "light");
  await page.getByRole("button", { name: "Share", exact: true }).click();
  const list = page.locator(".share-picker-list");
  for (const name of ["Copy link", "Email link", "Share on LinkedIn", "Share on Reddit", "Share on Bluesky", "Share on Mastodon"])
    await expect(list.getByText(name, { exact: true })).toBeVisible();
  await list.getByText("Copy link", { exact: true }).click();
  expect(await page.evaluate(() => navigator.clipboard.readText())).toMatch(/\/install\/$/);
});

test("a search from the bar lists the pages holding every word, in their own language", async ({ page }) => {
  await open(page, "/readme/", "light");
  await page.getByRole("button", { name: "Search this site", exact: true }).click();
  await page.getByRole("searchbox", { name: "Words to find" }).first().fill("subtree export");
  await page.keyboard.press("Enter");
  await expect(page).toHaveURL(/\/search\/\?q=subtree%20export$/);
  await expect(page.getByTestId("search-count")).toContainText("Pages with every word of “subtree export”");
  await expect(page.locator("main")).not.toHaveText(/could not be loaded/);
  await page.locator(".hits a").first().click();
  await expect(page).not.toHaveURL(/search/);

  await page.goto("/search/?q=المخاطر");
  const hit = page.locator(".hits a").first();
  await expect(hit).toHaveAttribute("lang", "ar-001");
  await expect(hit).toHaveAttribute("dir", "rtl");
  await page.goto("/search/?q=zzqqzz");
  await expect(page.getByTestId("search-count")).toHaveText(/: 0$/);
});

test("the search matches every word, title matches first", () => {
  const docs = [
    { route: "/a/", title: "Other", locale: "en", text: "risk risk risk plan" },
    { route: "/b/", title: "Risk", locale: "en", text: "a plan" },
    { route: "/c/", title: "Plan", locale: "en", text: "nothing else" },
  ];
  expect(search(docs, "RISK plan").map((h) => h.route)).toEqual(["/b/", "/a/"]);
  expect(search(docs, "   ")).toEqual([]);
  expect(search(docs, "zebra")).toEqual([]);
});

for (const scheme of ["light", "dark"] as const)
  test(`/search/ passes axe with results (${scheme})`, async ({ page }) => {
    await open(page, "/search/?q=workforce", scheme);
    await expect(page.locator(".hits a").first()).toBeVisible();
    expect(await axe(page)).toEqual([]);
  });

const keysOf = (o: object, at = ""): string[] =>
  Object.entries(o).flatMap(([k, v]) => (typeof v === "object" ? keysOf(v, `${at}${k}.`) : [`${at}${k}`]));

test("every catalogue has every key of the English source, and no empty string", () => {
  const source = keysOf(catalogues.en!).sort();
  for (const [language, catalogue] of Object.entries(catalogues)) {
    expect(keysOf(catalogue).sort(), language).toEqual(source);
    expect(JSON.stringify(catalogue), language).not.toContain('""');
  }
});

for (const [locale, headline] of [
  ["en-001", "Workforce Planning Management"],
  ["en-us", "Workforce Planning Management"],
  ["en-gb", "Workforce Planning Management"],
  ["en-gb-oxendict", "Workforce Planning Management"],
  ["cy-gb", "Rheoli Cynllunio'r Gweithlu"],
  ["cy-001", "Rheoli Cynllunio'r Gweithlu"],
  ["zh-cn", "劳动力规划管理"],
  ["zh-001", "劳动力规划管理"],
] as const)
  test(`/${locale}/ is the locale's home: hero, wordmark, chrome and Translations line in its language`, async ({ page }) => {
    const s = ui(locale);
    await open(page, `/${locale}/`, "light");
    await expect(page.locator("html")).toHaveAttribute("lang", locale);
    await expect(page.locator("main h1")).toHaveText(headline);
    await expect(page.locator(".brand")).toHaveText(s.brand);
    await expect(page.locator("main .landing-hero h1 + p")).not.toBeEmpty();
    await expect(page.locator("main .landing-hero blockquote")).toBeVisible();
    await expect(page.locator("main .landing-hero p.actions a")).toHaveCount(3);
    await expect(page.locator("main h2")).toHaveCount(5);
    await expect(page.locator("main table")).toBeVisible();
    await expect(page.locator("main .source")).toHaveCount(0);
    await expect(page.locator(".site-header nav").getByRole("link", { name: s.nav.briefs, exact: true })).toBeVisible();
    await page.locator(".site-header nav").getByRole("link", { name: s.nav.start, exact: true }).click();
    await expect(page).toHaveURL(new RegExp(`/${locale}/start/$`));
    await expect(page.locator("main pre").first()).toContainText("git clone");
    await page.goto(`/${locale}/`);
    await expect(page.getByRole("button", { name: s.pickers.textSize, exact: true })).toBeVisible();
    const line = page.getByTestId("translations");
    await expect(line).toContainText(s.translations);
    for (const name of ["English", "Cymraeg", "中文"]) await expect(line).toContainText(name);
    await expect(page.locator("footer")).toContainText(s.footer.notice);
  });

for (const slug of COMPARE_SLUGS)
  test(`/compare/${slug}/ is fair: it says when to choose the other product, that WPM is a demo, and its basis`, async ({ page }) => {
    await open(page, `/compare/${slug}/`, "light");
    await expect(page.locator("main h1")).toContainText("WPM compared with");
    await expect(page.locator("main")).not.toContainText("!");
    await expect(page.locator("main table")).toContainText("Suitable for production use");
    await expect(page.locator("main blockquote")).toContainText("demo software");
    await expect(page.getByRole("heading", { name: /^When .+ is the better fit$/ })).toHaveCount(2);
    await expect(page.getByRole("heading", { name: "Basis for this comparison" })).toBeVisible();
    await expect(page.locator("main p.actions a")).toHaveCount(2);
    await expect(page.locator("main")).toContainText("not affiliated");
  });

test("/compare/ links to every comparison, and the landing page links to /compare/", async ({ page }) => {
  await open(page, "/compare/", "light");
  for (const slug of COMPARE_SLUGS) await expect(page.locator(`main a[href$="/${slug}/"]`)).toBeVisible();
  await page.locator(".site-header nav").getByRole("link", { name: "Home", exact: true }).click();
  await expect(page.locator('main a[href$="compare/"]')).toBeVisible();
});

test("the not-found page speaks each catalogue's language and links home", async ({ page }) => {
  await page.goto("/404.html", { waitUntil: "networkidle" });
  await expect(page.getByTestId("not-found")).toHaveCount(Object.keys(catalogues).length);
  await expect(page.locator("main h1")).toHaveText("Page not found");
  expect(await axe(page)).toEqual([]);
  await page.getByRole("link", { name: ui("cy-001").notFound.home }).click();
  await expect(page).toHaveURL(/\/cy-001\/$/);
});

test("en-us and en-gb keep their own spelling", async ({ request }) => {
  const us = await (await request.get("/en-us/")).text();
  const gb = await (await request.get("/en-gb/")).text();
  expect(us).toContain("each behavior is traced");
  expect(gb).toContain("each behaviour is traced");
  expect(gb).toContain("localisations");
});

test("the root home has the hero and Translations line, and the language picker goes to a locale's home", async ({ page }) => {
  await open(page, "/", "light");
  await expect(page.locator("main h1")).toHaveText("Workforce Planning Management");
  await expect(page.getByRole("heading", { name: "Comparison", exact: true })).toBeVisible();
  await expect(page.locator("main table")).toContainText("Suitable for production use");
  await expect(page.getByTestId("translations")).toContainText("Translations: Cymraeg, English, 中文");
  await page.getByTestId("translations").getByRole("link", { name: "中文" }).click();
  await expect(page).toHaveURL(/\/zh-001\/$/);
  await expect(page.locator(".brand")).toHaveText("劳动力规划管理");
});

test("a Welsh brief has Welsh chrome, and a search from it stays Welsh and keeps the query across languages", async ({ page }) => {
  const s = ui("cy-001");
  await open(page, encodeURI(briefRoute(brief("cy-001", "chief-risk-officer"))), "light");
  await page.getByRole("button", { name: s.pickers.search, exact: true }).click();
  await page.getByRole("searchbox", { name: s.pickers.searchInput }).first().fill("risg");
  await page.keyboard.press("Enter");
  await expect(page).toHaveURL(/\/cy-001\/search\/\?q=risg$/);
  await expect(page.locator("main h1")).toHaveText(s.search.title);
  await page.getByRole("button", { name: s.pickers.language, exact: true }).click();
  await page.locator(".locale-picker-option").filter({ hasText: /^\s*中文\s*$/ }).click();
  await expect(page).toHaveURL(/\/zh-001\/search\/\?q=risg$/);
  await expect(page.locator("main h1")).toHaveText(ui("zh-001").search.title);
});

test("the sitemaps list the whole site and each locale's pages, in plain XML", async ({ request }) => {
  const whole = await (await request.get("/sitemap.xml")).text();
  expect(whole.startsWith("<?xml")).toBe(true);
  const urls = [...whole.matchAll(/<loc>([^<]+)<\/loc>/g)].map((m) => m[1] ?? "");
  expect(new Set(urls).size).toBe(urls.length);
  expect(urls.length).toBe(docRoutes.length + homeRoutes.length + listRoutes.length + briefRoutes.length + 1);
  expect(urls.every((u) => u.startsWith("https://workforce-planning-management.github.io/"))).toBe(true);
  const cy = await (await request.get("/cy-001/sitemap.xml")).text();
  expect(cy).toContain("/cy-001/</loc>");
  expect(cy).toContain("/cy-001/for/prif-swyddog-risg/</loc>");
  expect(cy).not.toContain("/zh-001/");
});

for (const theme of THEMES)
  for (const route of ["/", "/docs/"])
  test(`the ${theme} theme on ${route} loads with no CSP error and passes axe`, async ({ page }) => {
    const csp: string[] = [];
    page.on("console", (m) => {
      if (/Content Security Policy/i.test(m.text())) csp.push(m.text());
    });
    await page.addInitScript((t) => localStorage.setItem("mxi.wpm.theme", t), theme);
    await page.goto(route, { waitUntil: "networkidle" });
    await page.waitForFunction((t) => document.documentElement.dataset.theme === t, theme);
    expect(await axe(page)).toEqual([]);
    expect(csp).toEqual([]);
  });
