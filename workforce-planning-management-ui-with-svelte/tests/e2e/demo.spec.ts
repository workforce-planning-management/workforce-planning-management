// The guided tour of the demo (`/demo`) and its banner. A demo session is the cookie value `demo:…`; the
// sign-in that sets it, and the proxy's routing of it, are unit-tested (tests/unit/demo.test.ts). The API
// is stubbed here, so these specs check what the reader sees. Screenshots go to test-results/.

import { expect, test, type Page } from "@playwright/test";
import { DEMO_STEPS } from "../../src/lib/demoTour";

async function demoSession(page: Page) {
  await page.context().addCookies([
    {
      name: "__Host-mxi_session",
      value: "demo:e2e-session",
      domain: "localhost",
      path: "/",
      httpOnly: true,
      secure: true,
      sameSite: "Lax",
    },
  ]);
  await page.route("**/api/proxy/**", (route) => route.fulfill({ json: [] }));
}

test("the tour page lists every page by section and starts at step 1", async ({
  page,
}) => {
  await demoSession(page);
  await page.goto("/demo");
  await expect(page.getByTestId("demo-banner")).toContainText(
    "every person, figure and document here is fictitious",
  );
  await expect(page.getByTestId("demo-section")).toHaveCount(9);
  const links = page.getByTestId("demo-section").locator("a");
  await expect(links).toHaveCount(DEMO_STEPS.length);
  // The banner has no step here (this is the tour's own page), only a way back to it.
  await expect(page.getByTestId("demo-step")).toHaveCount(0);
  await page.getByTestId("demo-start").click();
  await expect(page).toHaveURL(new RegExp(`/en-001${DEMO_STEPS[0]!.path}$`));
  await expect(page.getByTestId("demo-step")).toHaveText(
    `Step 1 of ${DEMO_STEPS.length}`,
  );
  await expect(page.getByTestId("demo-previous")).toHaveCount(0);
});

test("Next and Previous walk the pages in order, and the last step offers to finish", async ({
  page,
}) => {
  await demoSession(page);
  await page.goto(`/en-001${DEMO_STEPS[0]!.path}`);
  await page.getByTestId("demo-next").click();
  await expect(page).toHaveURL(new RegExp(`${DEMO_STEPS[1]!.path}$`));
  await expect(page.getByTestId("demo-step")).toHaveText(
    `Step 2 of ${DEMO_STEPS.length}`,
  );
  await page.getByTestId("demo-previous").click();
  await expect(page).toHaveURL(new RegExp(`${DEMO_STEPS[0]!.path}$`));
  await page.goto(`/en-001${DEMO_STEPS.at(-1)!.path}`);
  await expect(page.getByTestId("demo-next")).toHaveCount(0);
  await page.getByTestId("demo-finish").click();
  await expect(page).toHaveURL(/\/en-001\/demo$/);
});

test("the banner keeps the locale, and a page that is not a step shows no step", async ({
  page,
}) => {
  await demoSession(page);
  await page.goto("/cy-001/workers");
  await expect(page.getByTestId("demo-banner")).toContainText("ffuglennol");
  await expect(page.getByTestId("demo-next")).toHaveAttribute(
    "href",
    "/cy-001/directory",
  );
  await page.goto("/cy-001/signin");
  await expect(page.getByTestId("demo-step")).toHaveCount(0);
});

test("a visitor with an ordinary session sees no banner", async ({ page }) => {
  await page.context().addCookies([
    {
      name: "__Host-mxi_session",
      value: "ordinary-session",
      domain: "localhost",
      path: "/",
      httpOnly: true,
      secure: true,
      sameSite: "Lax",
    },
  ]);
  await page.route("**/api/proxy/**", (route) => route.fulfill({ json: [] }));
  await page.goto("/workers");
  await expect(page.locator("h1")).toBeVisible();
  await expect(page.getByTestId("demo-banner")).toHaveCount(0);
});

for (const [scheme, size, label] of [
  ["light", { width: 1280, height: 900 }, "desktop"],
  ["dark", { width: 1280, height: 900 }, "desktop"],
  ["light", { width: 390, height: 844 }, "phone"],
  ["dark", { width: 390, height: 844 }, "phone"],
] as const) {
  test.describe(`${scheme} ${label}`, () => {
    test.use({ colorScheme: scheme, viewport: size });
    test("the tour page and a step fit without the page scrolling sideways", async ({
      page,
    }) => {
      await page.addInitScript((theme) => {
        try {
          localStorage.setItem("mxi.wpm.theme", theme);
        } catch {
          /* a blocked store only leaves the default theme */
        }
      }, scheme);
      await demoSession(page);
      for (const [name, path] of [
        ["demo", "/demo"],
        ["demo-step", "/oversight"],
      ] as const) {
        await page.goto(path);
        await expect(page.getByTestId("demo-banner")).toBeVisible();
        const overflow = await page.evaluate(
          () =>
            document.documentElement.scrollWidth -
            document.documentElement.clientWidth,
        );
        expect(overflow, name).toBeLessThanOrEqual(0);
        await page.screenshot({
          path: `test-results/${name}-${scheme}-${label}.png`,
          fullPage: true,
        });
      }
    });
  });
}
