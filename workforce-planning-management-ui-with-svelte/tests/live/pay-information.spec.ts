// Pay information against the real service (see README.md). Needs `WPM_EQUALITY_MONITORING_BASIS` and `WPM_EQUALITY_CATEGORIES`, the
// organization `PAY_INFO_ORG` with workers of both sexes on pay and a category, and (set in the database, since a request is made by the
// worker through /api/me, which needs a token) one overdue request from a woman in a category with at least ten of each sex, and one
// open request from a man in a category with fewer than ten men. With enforcement off the service reports no organization scope, so that
// one call is stubbed; the queue, the answer and the recording are real. Asserts rules and shapes. `PAGE_SHOTS=<dir>` writes screenshots.

import { expect, test } from "@playwright/test";

test.skip(
  !process.env.WPM_LIVE_API,
  "needs a running service: set WPM_LIVE_API=1",
);

const org = process.env.PAY_INFO_ORG ?? "";
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

test("the real queue: an overdue request, the answer with a withheld average, and recording it keeps only the day", async ({
  page,
}) => {
  await page.goto("/pay-information");
  const rows = page.getByTestId("pay-info-queue-row");
  await expect(rows).toHaveCount(2);
  await expect(rows.first()).toContainText("Overdue");
  if (shots)
    await page.screenshot({
      path: `${shots}/pay-information-live-queue.png`,
      fullPage: true,
    });

  // The woman in a category with both sexes over the floor: both averages shown.
  await rows.first().getByTestId("pay-info-work-out").click();
  await expect(page.getByTestId("pay-info-own")).toContainText("£");
  await expect(page.getByTestId("pay-info-female")).toContainText("£");
  await expect(page.getByTestId("pay-info-male")).toContainText("£");
  if (shots)
    await page.screenshot({
      path: `${shots}/pay-information-live-answer.png`,
      fullPage: true,
    });

  // The man in a category with fewer than ten men: his sex's average is withheld, and no other worker is named.
  await page.reload();
  await rows.nth(1).getByTestId("pay-info-work-out").click();
  await expect(page.getByTestId("pay-info-male")).toContainText("withheld");
  const text = await page.locator("main").innerText();
  expect((text.match(/Gap Person/g) ?? []).length).toBe(2); // the two askers, nobody else

  await page.getByTestId("pay-info-mark").click();
  await expect(page.getByTestId("pay-info-answer")).toHaveCount(0);
  await expect(rows.nth(1)).toContainText("Answered");
});

test("the real queue in the dark theme", async ({ page }) => {
  test.skip(!shots, "set PAGE_SHOTS=<dir> to write the screenshot");
  await page.addInitScript(() => localStorage.setItem("mxi.wpm.theme", "dark"));
  await page.goto("/pay-information");
  await expect(page.getByTestId("pay-info-queue")).toBeVisible();
  await page.getByTestId("pay-info-work-out").first().click();
  await expect(page.getByTestId("pay-info-answer")).toBeVisible();
  await page.screenshot({
    path: `${shots}/pay-information-live-dark.png`,
    fullPage: true,
  });
});
