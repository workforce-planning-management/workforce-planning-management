// Signed-out behaviour (WPM-T38): the gate, security headers, the sitemap, the splash and locale choice at "/".
// No session cookie here, the opposite of every other spec.

import { expect, test } from "@playwright/test";

test.describe("sign-in gate (WPM-T38)", () => {
  // These tests run with NO session cookie — the opposite of every
  // other test in this file — so they get their own describe block
  // rather than relying on the file-level beforeEach.
  test("a signed-out visitor is redirected from a protected page to /signin", async ({
    page,
  }) => {
    await page.goto("/workers");
    await expect(page).toHaveURL(/\/signin$/);
  });

  test("pages send security headers and the policy blocks nothing the app needs", async ({
    page,
  }) => {
    const violations: string[] = [];
    page.on("console", (message) => {
      if (/content security policy|refused to/i.test(message.text())) {
        violations.push(message.text());
      }
    });
    const response = await page.goto("/signin");
    expect(response).not.toBeNull();
    const headers = response!.headers();
    expect(headers["x-content-type-options"]).toBe("nosniff");
    expect(headers["x-frame-options"]).toBe("DENY");
    expect(headers["referrer-policy"]).toBe("strict-origin-when-cross-origin");
    expect(headers["cross-origin-opener-policy"]).toBe("same-origin");
    const csp = headers["content-security-policy"] ?? "";
    expect(csp).toContain("frame-ancestors 'none'");
    expect(csp).toContain("default-src 'self'");
    expect(csp).not.toContain("script-src 'self' 'unsafe-inline'");
    // The app still hydrates under the policy: the page is interactive.
    await expect(page.locator("body")).not.toBeEmpty();
    // A second public page, to exercise client navigation.
    await page.goto("/en-001/tour");
    expect(violations).toEqual([]);
  });

  test("the sitemap lists the public pages in every locale, and robots.txt points at it", async ({
    request,
  }) => {
    const sitemap = await request.get("/sitemap.xml");
    expect(sitemap.status()).toBe(200);
    expect(sitemap.headers()["content-type"]).toContain("application/xml");
    const origin = new URL(sitemap.url()).origin;
    const xml = await sitemap.text();
    // 17 locales × 3 public pages, each with hreflang alternates and x-default.
    expect(xml.match(/<url>/g)).toHaveLength(51);
    expect(xml).toContain(`<loc>${origin}/cy-001/tour</loc>`);
    expect(xml).toContain(`hreflang="x-default" href="${origin}/en-001"`);
    // Signed-in pages are never listed.
    expect(xml).not.toContain("/workers");
    expect(xml).not.toContain("/ceo");

    const robots = await request.get("/robots.txt");
    expect(robots.status()).toBe(200);
    const text = await robots.text();
    expect(text).toContain(`Sitemap: ${origin}/sitemap.xml`);
    expect(text).toContain("Disallow: /api/");
  });

  test("a signed-out visitor sees the marketing splash at / instead of a redirect", async ({
    page,
  }) => {
    const response = await page.goto("/");
    expect(response?.status()).toBe(200);
    await expect(page).toHaveURL(/\/en-001$/);
    await expect(page.getByTestId("splash")).toBeVisible();
    await expect(page.getByTestId("splash")).toContainText(
      "People operations, all in one place",
    );
    await page.getByTestId("hero-cta").click();
    await expect(page).toHaveURL(/\/signin$/);
  });

  // The browser's own language picks the locale for a bare `/` — not the server's
  // `Accept-Language` guess — unless the visitor has already chosen one.
  for (const [tag, expected] of [
    ["cy-GB", "cy-001"],
    ["en-GB", "en-gb"],
    ["de-DE", "de-de"],
    ["pt-BR", "pt-001"],
  ] as const) {
    test(`bare / follows navigator.language ${tag} to /${expected}/`, async ({
      browser,
    }) => {
      const context = await browser.newContext({ locale: tag });
      const page = await context.newPage();
      await page.goto("/");
      await expect(page).toHaveURL(new RegExp(`/${expected}$`));
      expect(await page.evaluate(() => document.documentElement.lang)).toBe(
        expected,
      );
      await context.close();
    });
  }

  test("the browser's language wins over a different Accept-Language header", async ({
    browser,
  }) => {
    // The server would pick French from the header; the browser says Welsh.
    const context = await browser.newContext({
      locale: "cy-GB",
      extraHTTPHeaders: { "accept-language": "fr" },
    });
    const page = await context.newPage();
    await page.goto("/");
    await expect(page).toHaveURL(/\/cy-001$/);
    // …and an unserved browser language falls back to that header's pick.
    const other = await browser.newContext({
      locale: "ja-JP",
      extraHTTPHeaders: { "accept-language": "fr" },
    });
    const p2 = await other.newPage();
    await p2.goto("/");
    await expect(p2).toHaveURL(/\/fr-001$/);
    await context.close();
    await other.close();
  });

  test("bare / with a browser language the app does not serve falls back to the server's pick", async ({
    browser,
  }) => {
    const context = await browser.newContext({ locale: "ja-JP" });
    const page = await context.newPage();
    await page.goto("/");
    await expect(page).toHaveURL(/\/en-001$/);
    await context.close();
  });

  test("a remembered locale beats the browser language, and /signin keeps its own rule", async ({
    browser,
  }) => {
    const context = await browser.newContext({ locale: "cy-GB" });
    await context.addCookies([
      { name: "wpm-locale", value: "de-001", domain: "localhost", path: "/" },
    ]);
    const page = await context.newPage();
    await page.goto("/");
    await expect(page).toHaveURL(/\/de-001$/);
    await context.close();
    // Other unprefixed paths are still redirected by the server (Accept-Language).
    const other = await browser.newContext({ locale: "cy-GB" });
    const p2 = await other.newPage();
    await p2.goto("/signin");
    await expect(p2).toHaveURL(/\/cy-001\/signin$/);
    await other.close();
  });

  test("without JavaScript a bare / carries a noscript refresh to the server's pick", async ({
    request,
  }) => {
    const html = await (
      await request.get("/", { headers: { "accept-language": "cy-GB" } })
    ).text();
    expect(html).toContain(
      '<noscript><meta http-equiv="refresh" content="0;url=/cy-001"></noscript>',
    );
    // A locale page carries no such refresh.
    const localised = await (await request.get("/cy-001")).text();
    expect(localised).not.toContain("http-equiv");
  });

  test("/tour stays reachable with no session and covers the app in depth", async ({
    page,
  }) => {
    const response = await page.goto("/tour");
    expect(response?.status()).toBe(200);
    await expect(page).toHaveURL(/\/tour$/);
    await expect(page.locator("h1")).toHaveText("See how it all fits together");
    await expect(page.locator("body")).toContainText(
      "Hiring & onboarding, start to finish",
    );
    await expect(page.locator("body")).toContainText(
      "Belong to more than one organization",
    );
    await page.locator('nav.top a[href="/en-001/signin"]').click();
    await expect(page).toHaveURL(/\/signin$/);
  });

  test("/signin itself stays reachable with no session", async ({ page }) => {
    const response = await page.goto("/signin");
    expect(response?.status()).toBe(200);
    await expect(page).toHaveURL(/\/signin$/);
  });
});
