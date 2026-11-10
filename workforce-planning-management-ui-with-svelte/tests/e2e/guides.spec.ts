// The walkthroughs (WPM-R244): one page per role under the tour, reachable from the tour and from help, each task linking to the
// page it starts from. Public pages; screenshots at phone and desktop width.

import { expect, test } from "@playwright/test";

test.describe("Walkthroughs", () => {
  test("the tour and help both lead to the three walkthroughs", async ({
    page,
  }) => {
    await page.goto("/tour");
    await expect(page.getByTestId("tour-guide-employee")).toHaveText(
      "For an employee",
    );
    await page.getByTestId("tour-guide-manager").click();
    await expect(page).toHaveURL(/\/en-001\/tour\/manager$/);
    await expect(page.getByTestId("guide-title")).toHaveText("For a manager");

    await page.goto("/help");
    await page.getByTestId("help-guide-hr").click();
    await expect(page).toHaveURL(/\/en-001\/tour\/hr$/);
    await expect(page.getByTestId("guide-title")).toHaveText("For HR");
  });

  test("an employee's walkthrough says where to click and links each task to its page", async ({
    page,
  }) => {
    await page.goto("/tour/employee");
    const tasks = page.getByTestId("guide-task");
    await expect(tasks.first()).toContainText(
      "Find your mandatory training and certifications",
    );
    await expect(tasks.first().getByTestId("guide-open")).toHaveAttribute(
      "href",
      "/en-001/me/required",
    );
    await expect(tasks.nth(1)).toContainText("Waiting for you panel");
    const goal = tasks.filter({ hasText: "Propose a goal" });
    await expect(goal.getByTestId("guide-open")).toHaveAttribute(
      "href",
      "/en-001/me/goals",
    );
    await expect(page.getByTestId("guide-help")).toHaveAttribute(
      "href",
      "/en-001/help",
    );
    await page.screenshot({
      path: "test-results/guide-employee-desktop.png",
      fullPage: true,
    });
  });

  test("HR's walkthrough points at the HR pages", async ({ page }) => {
    await page.goto("/tour/hr");
    await expect(page.getByTestId("guide-open")).toHaveCount(5);
    const hrefs = await page
      .getByTestId("guide-open")
      .evaluateAll((links) => links.map((a) => a.getAttribute("href")));
    expect(hrefs).toEqual([
      "/en-001/mandatory-learning",
      "/en-001/mandatory-learning",
      "/en-001/qualifications",
      "/en-001/requisitions",
      "/en-001/movements",
    ]);
  });

  test("an unknown walkthrough is not found, with help offered", async ({
    page,
  }) => {
    await page.goto("/tour/astronaut");
    await expect(page.getByTestId("error-page-title")).toHaveText(
      "Page not found",
    );
    await expect(page.getByTestId("error-page-help")).toBeVisible();
  });

  test("a walkthrough reads right to left in Arabic and fits a phone", async ({
    page,
  }) => {
    await page.setViewportSize({ width: 375, height: 812 });
    await page.goto("/ar-001/tour/manager");
    await expect(page.locator("html")).toHaveAttribute("dir", "rtl");
    await expect(page.getByTestId("guide-task")).toHaveCount(4);
    const overflow = await page.evaluate(
      () => document.documentElement.scrollWidth - window.innerWidth,
    );
    expect(overflow).toBeLessThanOrEqual(0);
    await page.screenshot({
      path: "test-results/guide-manager-ar-phone.png",
      fullPage: true,
    });
  });
});
