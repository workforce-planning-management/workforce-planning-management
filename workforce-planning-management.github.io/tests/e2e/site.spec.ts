// The built site: every page is there; axe finds no serious or critical WCAG 2.2 AA violation under the light and the
// dark theme; the locale switcher opens the same brief in another locale; an Arabic or Urdu page is laid out from the
// right without overflowing; dates read as the WPM UI's locale-format.ts shows them; and the content security policy
// blocks nothing the site itself does.
import { test, expect, type Page } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";
import docs from "../../src/content/docs.json" with { type: "json" };
import { BRIEF_LOCALES, DOC_PAGES } from "../../src/lib/site.ts";

type Brief = { locale: string; slug: string; peer: string | null };
const BRIEFS: Brief[] = docs.briefs;

const docRoutes = DOC_PAGES.map((d) => (d.slug ? `/${d.slug}/` : "/"));
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
  const all = [...docRoutes, ...listRoutes, ...briefRoutes];
  expect(all.length).toBe(DOC_PAGES.length + BRIEF_LOCALES.length + BRIEFS.length);
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
  await page.getByRole("button", { name: "Language of this page" }).click();
  await page.locator(".locale-picker-option").filter({ hasText: /^\s*Cymraeg\s*$/ }).click();
  await expect(page).toHaveURL(new RegExp(`/cy-001/for/${brief("cy-001", "chief-risk-officer").slug}/$`));
  await expect(page.locator("html")).toHaveAttribute("lang", "cy-001");

  await page.getByRole("button", { name: "Language of this page" }).click();
  await page.locator(".locale-picker-option").filter({ hasText: /^\s*العربية\s*$/ }).click();
  const ar = brief("ar-001", "chief-risk-officer");
  await expect(page).toHaveURL(encodeURI(briefRoute(ar)));
  await expect(page.locator("html")).toHaveAttribute("dir", "rtl");
  await expect(page.locator("h1").first()).toContainText("المخاطر");
});

test("the locale switcher on a list opens the other locale's list, and is not on the English documents", async ({
  page,
}) => {
  await open(page, "/de-001/for/", "light");
  await page.getByRole("button", { name: "Language of this page" }).click();
  await page.locator(".locale-picker-option").filter({ hasText: /^\s*Français\s*$/ }).click();
  await expect(page).toHaveURL(/\/fr-001\/for\/$/);
  await open(page, "/readme/", "light");
  await expect(page.getByRole("button", { name: "Language of this page" })).toHaveCount(0);
  await expect(page.getByRole("button", { name: "Theme" })).toBeVisible();
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
  await page.goto("/");
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
