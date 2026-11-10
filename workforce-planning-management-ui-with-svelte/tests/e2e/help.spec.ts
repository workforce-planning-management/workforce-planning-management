// Getting help (WPM-R243): a Help link in every header, a public help page with the project's real channels, and every error
// saying what to try next with a link to help. The API is stubbed per test; an unstubbed /api/proxy call 404s loudly.

import { expect, test } from "@playwright/test";
import { stubSignedIn } from "./support";

const ISSUES =
  "https://github.com/workforce-planning-management/workforce-planning-management/issues";

test.describe("Getting help", () => {
  test("the header links to help, which needs no sign-in and names the issue tracker", async ({
    page,
  }) => {
    await page.goto("/tour");
    const link = page.getByTestId("help-link");
    await expect(link).toHaveText("Help");
    await link.click();
    await expect(page).toHaveURL(/\/en-001\/help$/);
    await expect(page.locator("h1")).toHaveText("Get help");
    await expect(page.getByTestId("help-issues")).toHaveAttribute(
      "href",
      ISSUES,
    );
    await expect(page.getByTestId("help-security")).toHaveAttribute(
      "href",
      /SECURITY\.md$/,
    );
    await expect(page.getByTestId("help-tour")).toHaveAttribute(
      "href",
      "/en-001/tour",
    );
    await expect(page.getByTestId("help-errors")).toContainText(
      "Try again first",
    );
    await expect(page.getByTestId("help-deployer")).toContainText(
      "its own service desk",
    );
    await page.screenshot({
      path: "test-results/help-desktop.png",
      fullPage: true,
    });
  });

  test("the help page reads in the URL's locale", async ({ page }) => {
    await page.goto("/cy-001/help");
    await expect(page.locator("h1")).toHaveText("Cael help");
    await expect(page.getByTestId("help-link")).toHaveText("Cymorth");
  });

  test("the tour points to help", async ({ page }) => {
    await page.goto("/tour");
    await page.getByTestId("tour-help-link").click();
    await expect(page).toHaveURL(/\/en-001\/help$/);
  });

  test("a failed load says what to try and links to help", async ({ page }) => {
    await stubSignedIn(page);
    await page.route("**/api/proxy/workers/*/development-plans", (route) =>
      route.fulfill({ status: 500, body: "{}" }),
    );
    await page.goto("/me/development");
    await expect(page.getByTestId("error")).toContainText(
      "Something went wrong on our side",
    );
    const next = page.getByTestId("error-next");
    await expect(next).toContainText("Try again, or reload the page.");
    await page.screenshot({ path: "test-results/help-error-note.png" });
    await next.getByRole("link", { name: "Get help" }).click();
    await expect(page).toHaveURL(/\/en-001\/help$/);
  });

  test("a refusal points to a manager or HR", async ({ page }) => {
    await stubSignedIn(page);
    await page.route("**/api/proxy/workers/*/development-plans", (route) =>
      route.fulfill({ status: 403, body: "{}" }),
    );
    await page.goto("/me/development");
    await expect(page.getByTestId("error-next")).toContainText(
      "ask your manager or HR",
    );
  });

  test("an address that does not exist says so and offers help", async ({
    page,
  }) => {
    await stubSignedIn(page);
    await page.goto("/en-001/no-such-page");
    await expect(page.getByTestId("error-page-title")).toHaveText(
      "Page not found",
    );
    await expect(page.getByTestId("error-page-help")).toHaveAttribute(
      "href",
      "/en-001/help",
    );
    await expect(page.getByTestId("error-page-home")).toHaveAttribute(
      "href",
      "/en-001",
    );
  });

  test("help fits a phone without sideways scrolling", async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 812 });
    await page.goto("/help");
    await expect(page.locator("h1")).toHaveText("Get help");
    const overflow = await page.evaluate(
      () => document.documentElement.scrollWidth - window.innerWidth,
    );
    expect(overflow).toBeLessThanOrEqual(0);
    await page.screenshot({
      path: "test-results/help-phone.png",
      fullPage: true,
    });
  });
});
