// Locale, CMS shell and navigation chrome. The API is stubbed per test; an unstubbed /api/proxy call 404s loudly.

import { expect, test } from "@playwright/test";
import { chooseLocale, stubSignedIn } from "./support";

test.describe("Locale, CMS shell and navigation chrome", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
  });

  test("locale switcher retranslates the chrome (and ar flips direction)", async ({
    page,
  }) => {
    await page.goto("/workers");
    // An unprefixed visit lands on the default locale's content route.
    await expect(page).toHaveURL(/\/en-001\/workers$/);
    await expect(page.locator("h1")).toHaveText("Workers");
    await chooseLocale(page, "Deutsch");
    await expect(page).toHaveURL(/\/de-001\/workers$/);
    await expect(page.locator("h1")).toHaveText("Arbeitskräfte");
    await chooseLocale(page, "العربية");
    await expect(page).toHaveURL(/\/ar-001\/workers$/);
    await expect(page.locator("html")).toHaveAttribute("dir", "rtl");
  });

  test("the CMS shell is served per locale, in that locale's CMS language", async ({
    page,
  }) => {
    await page.route("https://unpkg.com/**", (route) =>
      route.fulfill({ contentType: "text/javascript", body: "" }),
    );
    await page.goto("/de-001/admin/");
    await expect(page).toHaveURL(/\/de-001\/admin\/?$/);
    await expect(page.locator("html")).toHaveAttribute("lang", "de-001");
    expect(
      await page.evaluate(
        () =>
          JSON.parse(localStorage.getItem("sveltia-cms.prefs") ?? "{}").locale,
      ),
    ).toBe("de");
    const config = await page.request.get("/admin/config.yml");
    expect(config.status()).toBe(200);
  });

  test("a bare language is not a route: /cy/workers is not forwarded to /cy-001/workers", async ({
    page,
  }) => {
    await page.goto("/cy/workers");
    // Treated like any other unprefixed path: sent under the visitor's own locale
    // (Playwright's `en`), with `/cy/workers` as the (unknown) page beneath it.
    await expect(page).toHaveURL(/\/en-001\/cy\/workers$/);
    await expect(page).not.toHaveURL(/\/cy-001\//);
    // The full content code is the only address for a locale.
    await page.goto("/cy-001/workers");
    await expect(page).toHaveURL(/\/cy-001\/workers$/);
    await expect(page.locator("html")).toHaveAttribute("lang", "cy-001");
  });

  test("hamburger menu opens the left nav drawer with every section link", async ({
    page,
  }) => {
    await page.goto("/");
    const drawer = page.locator(".drawer");
    await expect(drawer).toBeHidden();
    await page.getByRole("button", { name: "Menu" }).click();
    await expect(drawer).toBeVisible();
    await expect(drawer).toContainText("Workers");
    await expect(drawer).toContainText("Org chart");
    await expect(drawer).toContainText("Benchmarks");
    await drawer.getByRole("link", { name: "Workers" }).click();
    await expect(page).toHaveURL(/\/workers$/);
    await expect(drawer).toBeHidden();
  });

  test("a converted page renders in Welsh, with no English left on it (WPM-R94)", async ({
    page,
  }) => {
    await page.route("**/api/proxy/learning/mentorship-overview**", (route) =>
      route.fulfill({
        json: {
          as_of: "2026-07-20",
          active_pairings: 1,
          mentor_load: [
            { mentor_pid: "e1", mentor: "Ada Mentor", active_mentees: 1 },
          ],
          unmatched_total: 0,
          unmatched_workers: [],
          stale_days: 30,
          stale_mentorships: [
            {
              pid: "m1",
              mentor: "Ada Mentor",
              mentee: "Sam Mentee",
              last_session: "2026-05-01",
            },
          ],
        },
      }),
    );
    await page.goto("/cy-001/mentorship");
    const body = page.locator("body");
    // The headings and labels of the page are in Welsh …
    await expect(body).toContainText("Llwyth mentor");
    await expect(body).toContainText("Mentora hen");
    await expect(body).toContainText("Mentoreion gweithredol");
    await expect(body).toContainText("Mae pawb gweithredol mewn mentora.");
    // … and none of its English survives.
    for (const english of [
      "Mentor load",
      "Stale mentorships",
      "Active mentees",
      "Last session",
      "Unmatched workers",
    ]) {
      await expect(body).not.toContainText(english);
    }
    // The page title is translated too.
    await expect(page).toHaveTitle(/WPM$/);
  });
});
