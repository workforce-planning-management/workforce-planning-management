// The built site: every page is there; every picker in the Lily picker bar works; the search finds pages; axe finds no serious or critical WCAG 2.2 AA violation under the light and the
// dark theme (another theme's result is reported, not failed: WPM-R247); the locale switcher opens the same brief in another locale; an Arabic or Urdu page is laid out from the
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
const productionRoutes = BRIEF_LOCALES.map((l) => `/${l}/path-to-production/`);
const privacyRoutes = BRIEF_LOCALES.map((l) => `/${l}/privacy/`);
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
  ...productionRoutes,
  ...privacyRoutes,
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
  const all = [...docRoutes, ...homeRoutes, ...productionRoutes, ...privacyRoutes, ...listRoutes, ...briefRoutes];
  expect(all.length).toBe(2 + COMPARE_SLUGS.length + DOC_PAGES.length + 2 * HOME_LOCALES.length + 3 * BRIEF_LOCALES.length + BRIEFS.length);
  // Every locale has the 13 briefs, except the three English regional variants, which have the 11 shared ones.
  for (const l of BRIEF_LOCALES)
    expect(BRIEFS.filter((b) => b.locale === l).length, l).toBe(["en-gb", "en-gb-oxendict", "en-us"].includes(l) ? 11 : 13);
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
  await page.getByRole("button", { name: "Sprache" }).click();
  await page.locator(".locale-picker-option").filter({ hasText: /^\s*Français\s*$/ }).click();
  await expect(page).toHaveURL(/\/fr-001\/for\/$/);
  await open(page, "/readme/", "light");
  await page.getByRole("button", { name: "Language" }).click();
  await page.locator(".locale-picker-option").filter({ hasText: /^\s*Deutsch\s*$/ }).click();
  await expect(page).toHaveURL(/\/de-001\/$/);
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
    await page.goto(encodeURI(briefRoute(brief(l, "chief-risk-officer"))));
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

/** Each picker's English label, as its key in a catalogue. */
const KEYS: Record<string, keyof ReturnType<typeof ui>["pickers"]> = {
  Pages: "pages",
  "Search this site": "search",
  Theme: "theme",
  Language: "language",
  "Text size": "textSize",
  Share: "share",
};

for (const [name, width] of [["desktop", 1280], ["phone", 390], ["small phone", 320]] as const)
  for (const [locale, route] of [["en-001", "/readme/"], ["ar-001", encodeURI(briefRoute(brief("ar-001", "chief-executive-officer")))]] as const)
    test(`every picker opens in full, inside the screen, without moving the page: ${route} (${name})`, async ({ page }) => {
      await page.addInitScript(noShareSheet);
      await page.setViewportSize({ width, height: 800 });
      await open(page, route, "light");
      const tops = [];
      const inLanguage = PICKERS.map((p) => ({ ...p, button: ui(locale).pickers[KEYS[p.button]!] }));
      for (const { button } of inLanguage) {
        const box = await page.getByRole("button", { name: button, exact: true }).boundingBox();
        tops.push(box);
        expect(box?.width, button).toBeGreaterThanOrEqual(24);
      }
      if (width > 390) expect(new Set(tops.map((b) => b?.y)).size).toBe(1);
      for (const { button, popup } of inLanguage) {
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

test("the bar offers every Lily theme by its short name, in two alphabetical groups, and every text size", async ({ page }) => {
  await open(page, "/readme/", "light");
  await page.getByRole("button", { name: "Theme", exact: true }).click();
  const themes = await page.locator(".theme-picker-option").allTextContents();
  expect(themes).toHaveLength(THEMES.length);
  // Lily 0.8.0 names the NHS themes shortly ("NHS England (patients)"): the 36 generic themes alphabetically, then the
  // 27 that follow a company's or a public body's design system, alphabetically by that name.
  const names = themes.map((t) => t.trim());
  expect(names).toContain("NHS England (patients)");
  expect(names.some((t) => /^United Kingdom National Health Service/.test(t))).toBe(false);
  const byName = (a: string, b: string) => a.localeCompare(b, "en", { sensitivity: "base" });
  const generic = names.slice(0, 36);
  const branded = names.slice(36);
  expect(branded).toHaveLength(27);
  expect(generic).toEqual([...generic].sort(byName));
  expect(branded).toEqual([...branded].sort(byName));
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
    expect(JSON.stringify(catalogue).replace(/\\./g, ""), language).not.toContain('""');
  }
});

for (const [locale, headline] of [
  ["ar-001", "إدارة تخطيط القوى العاملة"],
  ["en-001", "Workforce Planning Management"],
  ["en-us", "Workforce Planning Management"],
  ["en-gb", "Workforce Planning Management"],
  ["en-gb-oxendict", "Workforce Planning Management"],
  ["cy-gb", "Rheoli Cynllunio'r Gweithlu"],
  ["cy-001", "Rheoli Cynllunio'r Gweithlu"],
  ["zh-cn", "劳动力规划管理"],
  ["zh-001", "劳动力规划管理"],
  ["hi-in", "वर्कफ़ोर्स प्लानिंग मैनेजमेंट"],
  ["hi-001", "वर्कफ़ोर्स प्लानिंग मैनेजमेंट"],
  ["es-001", "Gestión de la planificación de la plantilla"],
  ["es-es", "Gestión de la planificación de la plantilla"],
  ["es-mx", "Gestión de la planificación de la plantilla"],
  ["fr-001", "Gestion de la planification des effectifs"],
  ["fr-ca", "Gestion de la planification des effectifs"],
  ["fr-fr", "Gestion de la planification des effectifs"],
  ["bn-001", "ওয়ার্কফোর্স প্ল্যানিং ম্যানেজমেন্ট"],
  ["bn-bd", "ওয়ার্কফোর্স প্ল্যানিং ম্যানেজমেন্ট"],
  ["bn-in", "ওয়ার্কফোর্স প্ল্যানিং ম্যানেজমেন্ট"],
  ["id-001", "Manajemen Perencanaan Tenaga Kerja"],
  ["id-id", "Manajemen Perencanaan Tenaga Kerja"],
  ["id-my", "Manajemen Perencanaan Tenaga Kerja"],
  ["it-001", "Pianificazione della forza lavoro"],
  ["it-it", "Pianificazione della forza lavoro"],
  ["ja-001", "ワークフォース・プランニング・マネジメント"],
  ["ja-jp", "ワークフォース・プランニング・マネジメント"],
  ["ko-001", "인력 계획 관리"],
  ["ko-kr", "인력 계획 관리"],
  ["nl-001", "Personeelsplanning"],
  ["nl-nl", "Personeelsplanning"],
  ["da-001", "Planlægning af arbejdsstyrken"],
  ["de-001", "Personalplanung"],
  ["de-at", "Personalplanung"],
  ["de-de", "Personalplanung"],
  ["pt-001", "Planeamento de Pessoal"],
  ["pt-pt", "Planeamento de Pessoal"],
  ["pt-br", "Planejamento de Pessoal"],
  ["ur-001", "ورک فورس پلاننگ مینجمنٹ"],
  ["vi-001", "Hoạch định nhân lực"],
  ["vi-vn", "Hoạch định nhân lực"],
  ["ru-001", "Управление планированием персонала"],
  ["ru-by", "Управление планированием персонала"],
  ["ru-ru", "Управление планированием персонала"],
  ["sv-001", "Arbetskraftsplanering"],
  ["sv-se", "Arbetskraftsplanering"],
  ["sw-001", "Usimamizi wa Mipango ya Wafanyakazi"],
  ["sw-tz", "Usimamizi wa Mipango ya Wafanyakazi"],
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
    // One per `## ` section of the locale's own page, which differ: en-001 has a Getting help section the others do not yet.
    const sections = ((docs.files as Record<string, string>)[`locales/${locale}/index.md`] ?? "")
      .split("\n")
      .filter((line) => line.startsWith("## ")).length;
    expect(sections).toBeGreaterThanOrEqual(5);
    await expect(page.locator("main h2")).toHaveCount(sections);
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
    for (const name of ["العربية", "English", "Cymraeg", "हिन्दी", "中文"]) await expect(line).toContainText(name);
    await expect(page.locator("footer")).toContainText(s.footer.notice);
  });

test("an Arabic page is right to left in its chrome and left to right in its code", async ({ page }) => {
  await open(page, "/ar-001/start/", "light");
  await expect(page.locator("html")).toHaveAttribute("dir", "rtl");
  await expect(page.locator("footer")).toHaveAttribute("dir", "rtl");
  await expect(page.locator("main pre").first()).toHaveCSS("direction", "ltr");
  await expect(page.locator("main p code").first()).toHaveCSS("direction", "ltr");
  await expect(page.locator("main p code").first()).toHaveCSS("unicode-bidi", "isolate");
  await expect(page.locator(".source")).toHaveAttribute("dir", "rtl");
  await expect(page.locator("footer")).toContainText("\u2066workforce-planning-management.github.io/\u2069");
  await page.goto("/404.html");
  await expect(page.locator('section[lang="ar-001"]')).toHaveAttribute("dir", "rtl");
});

// The estimates as the Markdown writes them: `- **75%** **Name.** …`, one slider each.
function estimatesOf(locale: string): number[] {
  const text = (docs.files as Record<string, string>)[`locales/${locale}/path-to-production/index.md`] ?? "";
  return [...text.matchAll(/^- \*\*(\d{1,3})%\*\* \*\*/gm)].map((m) => Number(m[1]));
}

for (const locale of BRIEF_LOCALES)
  test(`/${locale}/path-to-production/ has a Lily slider at the author's estimate for every item`, async ({ page }) => {
    const s = ui(locale);
    const estimates = estimatesOf(locale);
    expect(estimates.length).toBeGreaterThanOrEqual(10);
    expect(estimates.every((n) => n >= 0 && n < 100)).toBe(true);
    await open(page, `/${locale}/path-to-production/`, "light");
    await expect(page.locator("html")).toHaveAttribute("lang", locale);
    const sliders = page.locator('main input[type="range"]');
    await expect(sliders).toHaveCount(estimates.length);
    expect(await sliders.evaluateAll((els) => els.map((e) => Number((e as HTMLInputElement).value)))).toEqual(estimates);
    const prefix = s.readiness.label.split("{name}")[0] ?? "";
    for (const label of await sliders.evaluateAll((els) => els.map((e) => e.getAttribute("aria-label") ?? "")))
      expect(label.startsWith(prefix) && label.length > prefix.length, label).toBe(true);
    await expect(page.locator("main .readiness-note")).toHaveText(s.readiness.note);
    // No item claims to be done, and the page says what production means before it lists anything.
    await expect(page.locator("main h2").first()).toBeVisible();
  });

test("moving a readiness slider shows the reader's value beside the author's estimate, and saves nothing", async ({ page }) => {
  await open(page, "/en-001/path-to-production/", "light");
  const first = page.locator("main .readiness li").first();
  const slider = first.locator('input[type="range"]');
  await expect(slider).toHaveValue("75");
  await expect(first.locator("output")).toHaveText("75%");
  const stored = () => page.evaluate(() => JSON.stringify({ ...localStorage }) + JSON.stringify({ ...sessionStorage }));
  const before = await stored();
  await slider.focus();
  await page.keyboard.press("ArrowRight");
  await expect(slider).toHaveValue("80");
  await expect(slider).toHaveAttribute("aria-valuetext", "80%");
  await expect(first.locator("output")).toContainText("80%");
  await expect(first.locator("output")).toContainText("(author's estimate: 75%)");
  await page.keyboard.press("Home");
  await expect(first.locator("output strong")).toHaveText("0%");
  expect(await stored()).toBe(before);
  await page.reload({ waitUntil: "networkidle" });
  await expect(page.locator('main .readiness input[type="range"]').first()).toHaveValue("75");
});

test("each home's demo notice links to its path to production, and the pages picker lists it", async ({ page }) => {
  for (const locale of HOME_LOCALES) {
    await page.goto(`/${locale}/`);
    const link = page.locator("main .landing-hero blockquote a").first();
    await expect(link, locale).toHaveAttribute("href", new RegExp(`/${locale}/path-to-production/$`));
  }
  await page.goto("/cy-001/path-to-production/");
  await expect(page.locator("main h1")).toHaveText("Y llwybr i ddefnydd cynhyrchu");
  await page.getByRole("button", { name: ui("cy-001").pickers.pages, exact: true }).click();
  await expect(page.locator(".link-picker-link").filter({ hasText: ui("cy-001").nav.production })).toBeVisible();
});

test("the footer ends its sentence in the language's own punctuation", async ({ page }) => {
  await page.goto("/hi-in/");
  await expect(page.locator("footer")).toContainText(`${ui("hi-in").footer.countsEnd} ${ui("hi-in").footer.notice}`);
  expect(ui("hi-in").footer.countsEnd).toBe("।");
});

for (const route of ["/ar-001/", "/ar-001/start/", "/ar-001/path-to-production/", "/ur-001/path-to-production/", "/hi-in/", "/hi-001/start/"])
  test(`${route} fits a phone and passes axe`, async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 844 });
    await open(page, route, "light");
    expect(await axe(page)).toEqual([]);
    const overflow = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
    expect(overflow).toBeLessThanOrEqual(0);
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
  // The header has no Home link; the brand is the way home.
  await page.locator(".site-header .brand").click();
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
  await expect(page.locator("main table")).toContainText("Typical fit");
  await expect(page.getByTestId("translations")).toContainText("Translations: العربية, বাংলা, Cymraeg, Dansk, Deutsch, English, Español, Français, हिन्दी, Bahasa Indonesia, Italiano, 日本語, 한국어, Nederlands, Português, Русский, Svenska, Kiswahili, اردو, Tiếng Việt, 中文");
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
  expect(urls.length).toBe(docRoutes.length + homeRoutes.length + productionRoutes.length + privacyRoutes.length + listRoutes.length + briefRoutes.length + 1);
  expect(urls.every((u) => u.startsWith("https://workforce-planning-management.github.io/"))).toBe(true);
  const cy = await (await request.get("/cy-001/sitemap.xml")).text();
  expect(cy).toContain("/cy-001/</loc>");
  expect(cy).toContain("/cy-001/for/prif-swyddog-risg/</loc>");
  expect(cy).not.toContain("/zh-001/");
});

// WPM-T525: RFC 9116. The file is served from /.well-known/, names a contact and the policy, and has not expired.
test("security.txt names the contact and the policy, and has not expired", async ({ request }) => {
  const response = await request.get("/.well-known/security.txt");
  expect(response.status()).toBe(200);
  const text = await response.text();
  const field = (name: string) => text.match(new RegExp(`^${name}: (.+)$`, "m"))?.[1] ?? "";
  const contacts = [...text.matchAll(/^Contact: (.+)$/gm)].map((m) => m[1]);
  expect(contacts).toEqual([
    "mailto:joel@joelparkerhenderson.com",
    // GitHub's private vulnerability reporting (WPM-T537).
    "https://github.com/workforce-planning-management/workforce-planning-management/security/advisories/new",
  ]);
  expect(field("Policy")).toMatch(/\/SECURITY\.md$/);
  expect(field("Canonical")).toBe("https://workforce-planning-management.github.io/.well-known/security.txt");
  const expires = Date.parse(field("Expires"));
  expect(expires).toBeGreaterThan(Date.now());
  // RFC 9116 asks for an expiry less than a year away; renewing it is part of the yearly review in SECURITY.md §11.
  expect(expires - Date.now()).toBeLessThanOrEqual(366 * 24 * 60 * 60 * 1000);
});

// WPM-R247: light and dark must pass axe; any other theme is the reader's choice, so its axe result is reported
// (as an annotation on the test) and does not fail the run. Every theme must load with no CSP error.
const VOUCHED = new Set(["light", "dark"]);
for (const theme of THEMES)
  for (const route of ["/", "/docs/"])
  test(`the ${theme} theme on ${route} loads with no CSP error${VOUCHED.has(theme) ? " and passes axe" : ""}`, async ({ page }, info) => {
    const csp: string[] = [];
    page.on("console", (m) => {
      if (/Content Security Policy/i.test(m.text())) csp.push(m.text());
    });
    await page.addInitScript((t) => localStorage.setItem("mxi.wpm.theme", t), theme);
    await page.goto(route, { waitUntil: "networkidle" });
    await page.waitForFunction((t) => document.documentElement.dataset.theme === t, theme);
    const found = await axe(page);
    if (VOUCHED.has(theme)) expect(found).toEqual([]);
    else for (const v of found) info.annotations.push({ type: "axe (reader's theme, not a failure)", description: v });
    expect(csp).toEqual([]);
  });

// WPM-R253: the theme is kept in localStorage and in a first-party cookie, only once the reader chooses; either one
// alone brings it back; and the privacy notice names everything the site keeps.
// SvelteKit keeps scroll positions in sessionStorage under `sveltekit:` keys as the reader moves between pages; they
// are not a choice, so they are left out here and checked against the notice on their own.
const SVELTEKIT = /^sveltekit:/;
const kept = (page: Page) =>
  page.evaluate(
    (prefix) => ({
      local: { ...localStorage },
      session: Object.fromEntries(Object.entries({ ...sessionStorage }).filter(([k]) => !new RegExp(prefix).test(k))),
      cookie: document.cookie,
    }),
    SVELTEKIT.source,
  );

test("a first visit keeps nothing; choosing a theme keeps it in localStorage and in a cookie", async ({ page }) => {
  await open(page, "/readme/", "light");
  expect(await kept(page)).toEqual({ local: {}, session: {}, cookie: "" });
  await page.getByRole("button", { name: "Theme", exact: true }).click();
  await page.locator(".theme-picker-option", { hasText: /^\s*Dark\s*$/ }).click();
  await page.waitForFunction(() => document.documentElement.dataset.theme === "dark");
  const after = await kept(page);
  expect(after.local).toEqual({ "mxi.wpm.theme": "dark" });
  expect(after.session).toEqual({});
  expect(after.cookie).toBe("wpm-theme=dark");
  const [cookie] = await page.context().cookies();
  expect(cookie).toMatchObject({ name: "wpm-theme", value: "dark", path: "/", sameSite: "Lax", httpOnly: false });
  // About 400 days: a browser may round or cap the lifetime it was given.
  expect(cookie!.expires * 1000 - Date.now()).toBeGreaterThan(399 * 24 * 3600 * 1000);
});

test("the theme comes back from the cookie alone, and from localStorage alone", async ({ page, context, baseURL }) => {
  await context.addCookies([{ name: "wpm-theme", value: "dark", url: baseURL! }]);
  await page.goto("/readme/", { waitUntil: "networkidle" });
  await page.waitForFunction(() => document.documentElement.dataset.theme === "dark");
  await context.clearCookies();
  await page.evaluate(() => localStorage.setItem("mxi.wpm.theme", "dark"));
  await page.reload({ waitUntil: "networkidle" });
  await page.waitForFunction(() => document.documentElement.dataset.theme === "dark");
});

test("a cookie naming no theme is ignored", async ({ page, context, baseURL }) => {
  await context.addCookies([{ name: "wpm-theme", value: "..%2Fevil", url: baseURL! }]);
  await open(page, "/readme/", "light");
  expect(await page.evaluate(() => [...document.styleSheets].map((s) => s.href ?? "").filter((h) => /evil/.test(h)))).toEqual([]);
});

test("the privacy notice names everything the site keeps, and every page's footer links to it", async ({ page }) => {
  await open(page, "/readme/", "light");
  await page.getByRole("button", { name: "Theme", exact: true }).click();
  await page.locator(".theme-picker-option", { hasText: /^\s*Dark\s*$/ }).click();
  await page.getByRole("button", { name: "Text size", exact: true }).click();
  await page.locator(".text-size-picker-option", { hasText: /^\s*Large\s*$/ }).click();
  const state = await kept(page);
  const names = [
    ...Object.keys(state.local),
    ...Object.keys(state.session),
    ...state.cookie.split(";").map((c) => c.split("=")[0]!.trim()).filter(Boolean),
  ];
  expect(names.sort()).toEqual(["mxi.wpm.text-size", "mxi.wpm.theme", "wpm-theme"]);
  await page.getByTestId("privacy-link").click();
  await expect(page).toHaveURL(/\/en-001\/privacy\/$/);
  // SvelteKit writes its keys when a page is left, so load the notice again from the address.
  await page.goto("/en-001/privacy/", { waitUntil: "networkidle" });
  const framework = await page.evaluate(() => Object.keys({ ...sessionStorage }));
  expect(framework.length).toBeGreaterThan(0);
  for (const key of framework) expect(key).toMatch(SVELTEKIT);
  const notice = await page.locator("main").innerText();
  for (const name of [...names, ...framework]) expect(notice, name).toContain(name);
  const footers = [["/", "en-001"], ["/readme/", "en-001"], ["/cy-001/", "cy-001"], ["/ar-001/for/", "ar-001"], ["/ur-001/path-to-production/", "ur-001"]] as const;
  for (const [route, locale] of footers) {
    await page.goto(route);
    await expect(page.getByTestId("privacy-link"), route).toHaveAttribute("href", new RegExp(`/${locale}/privacy/$`));
  }
});

test("a text size is kept only when chosen, and comes back on the next page", async ({ page }) => {
  await open(page, "/readme/", "light");
  await expect(page.locator("html")).toHaveAttribute("data-text-size", "normal");
  await page.goto("/docs/", { waitUntil: "networkidle" });
  expect(await kept(page)).toEqual({ local: {}, session: {}, cookie: "" });
  await page.getByRole("button", { name: "Text size", exact: true }).click();
  await page.locator(".text-size-picker-option", { hasText: /^\s*Larger\s*$/ }).click();
  await expect(page.locator("html")).toHaveAttribute("data-text-size", "larger");
  expect((await kept(page)).local).toEqual({ "mxi.wpm.text-size": "larger" });
  await page.goto("/readme/", { waitUntil: "networkidle" });
  await expect(page.locator("html")).toHaveAttribute("data-text-size", "larger");
  await page.evaluate(() => localStorage.setItem("mxi.wpm.text-size", "enormous"));
  await page.reload({ waitUntil: "networkidle" });
  await expect(page.locator("html")).toHaveAttribute("data-text-size", "normal");
});

// Every locale's notice is a translation of the English one: the same headings, list items and table rows, and every
// name in backticks (each storage key, cookie attribute and example value) unchanged, so a key named in English is
// named in every language. The language picker opens the same notice in another locale.
const FILES = docs.files as Record<string, string>;
const shape = (md: string) => ({
  headings: md.split("\n").filter((l) => /^#{1,6} /.test(l)).map((l) => l.match(/^#+/)![0]),
  items: md.split("\n").filter((l) => /^- /.test(l)).length,
  rows: md.split("\n").filter((l) => /^\|/.test(l)).map((l) => l.split("|").length),
  code: [...md.matchAll(/`([^`]+)`/g)].map((m) => m[1]).filter((c) => !c!.startsWith("locales/")).sort(),
  links: [...md.matchAll(/\]\(([^)]+)\)|<(https:[^>]+)>/g)].map((m) => m[1] ?? m[2]).sort(),
});
test("every locale's privacy notice has the English notice's shape, names and links", () => {
  const english = shape(FILES["locales/en-001/privacy/index.md"]!);
  for (const l of BRIEF_LOCALES) {
    const md = FILES[`locales/${l}/privacy/index.md`];
    expect(md, l).toBeDefined();
    expect(shape(md!), l).toEqual(english);
  }
});

test("the language picker opens the same privacy notice in another locale", async ({ page }) => {
  await open(page, "/en-001/privacy/", "light");
  await page.getByRole("button", { name: "Language", exact: true }).click();
  // Exactly: a regional sibling such as "Cymraeg - Ariannin" shares the language's name.
  await page.getByRole("option", { name: LOCALE_LABELS["cy-001"], exact: true }).click();
  await expect(page).toHaveURL(/\/cy-001\/privacy\/$/);
  await expect(page.locator("main")).toContainText("mxi.wpm.theme");
});
