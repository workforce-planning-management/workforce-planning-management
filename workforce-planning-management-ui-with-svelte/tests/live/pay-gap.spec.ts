// Pay gap against the real service (see README.md). Needs `WPM_EQUALITY_MONITORING_BASIS` and `WPM_EQUALITY_CATEGORIES` on the service, an
// organization (`PAY_GAP_ORG`) with at least ten of each sex on one job title with pay, fewer than ten men on a second title, and two
// categories (one per title). With enforcement off the service reports no organization scope, so that one call is stubbed to name the
// organization; the report and categories are real. Asserts rules and shapes, never exact figures. `PAGE_SHOTS=<dir>` writes screenshots.

import { expect, test } from "@playwright/test";

test.skip(
  !process.env.WPM_LIVE_API,
  "needs a running service: set WPM_LIVE_API=1",
);

const org = process.env.PAY_GAP_ORG ?? "";
const shots = process.env.PAGE_SHOTS;

test.beforeEach(async ({ page }) => {
  await page.context().addCookies([
    {
      name: "__Host-mxi_session",
      value: "live-test-session",
      domain: "localhost",
      path: "/",
      httpOnly: true,
      secure: true,
      sameSite: "Lax",
    },
  ]);
  await page.route("**/api/proxy/me/organizations/scope", (route) =>
    route.fulfill({ json: [org] }),
  );
});

test("the real report: a figure where both sexes reach the floor, a withheld one where they do not, no one named, a CSV link", async ({
  page,
}) => {
  await page.goto("/pay-gap");
  await expect(page.getByTestId("category-row")).toHaveCount(2);
  const rows = page.getByTestId("category-gap-row");
  await expect(rows).toHaveCount(2);
  await expect(page.getByTestId("gap-withheld")).not.toHaveCount(0); // the title with fewer than ten men
  await expect(page.getByTestId("gap-value")).not.toHaveCount(0); // the title with both sexes over the floor
  // WPM holds no variable pay, so the total-pay indicators say that, not "nobody in the group".
  await expect(page.getByTestId("indicator-row").first()).toContainText(
    "variable pay is recorded",
  );
  await expect(page.getByTestId("pay-gap-audited")).toBeVisible();
  const text = await page.locator("main").innerText();
  expect(text).not.toMatch(/Gap Person|GP-\d/);
  const csv = await page.request.get(
    (await page.getByTestId("pay-gap-csv").getAttribute("href")) as string,
  );
  expect(csv.status()).toBe(200);
  expect(await csv.text()).not.toMatch(/Gap Person|GP-\d/);
  if (shots) {
    await page.screenshot({
      path: `${shots}/pay-gap-live-light.png`,
      fullPage: true,
    });
    await page.setViewportSize({ width: 390, height: 800 });
    await page.screenshot({
      path: `${shots}/pay-gap-live-phone.png`,
      fullPage: true,
    });
  }
});

test("the real report in the dark theme", async ({ page }) => {
  test.skip(!shots, "set PAGE_SHOTS=<dir> to write the screenshot");
  await page.addInitScript(() => localStorage.setItem("mxi.wpm.theme", "dark"));
  await page.goto("/pay-gap");
  await expect(page.getByTestId("pay-gap-indicators")).toBeVisible();
  await page.screenshot({
    path: `${shots}/pay-gap-live-dark.png`,
    fullPage: true,
  });
});
