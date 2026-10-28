// The AI and scoring register against the real service (see README.md). Needs no organization or data. It records one feature, so it
// changes the data: a rerun starts from what the last run recorded. `PAGE_SHOTS=<dir>` writes light, dark and phone screenshots.

import { expect, test } from "@playwright/test";

test.skip(
  !process.env.WPM_LIVE_API,
  "needs a running service: set WPM_LIVE_API=1",
);

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
});

test("the real register: eight features, unknown where nothing is recorded, a short retention needs its reason, and a record shows", async ({
  page,
}) => {
  await page.goto("/ai-register");
  const cards = page.getByTestId("ai-feature");
  await expect(cards).toHaveCount(8);
  await expect(cards.first().getByTestId("ai-unknown")).toHaveCount(6);
  if (shots)
    await page.screenshot({
      path: `${shots}/ai-register-live-light.png`,
      fullPage: true,
    });

  // Three months with no reason: the service refuses, in words, and nothing is recorded.
  await cards.first().getByTestId("ai-edit").click();
  await page.getByTestId("ai-months").fill("3");
  await page.getByTestId("ai-save").click();
  await expect(page.getByTestId("error")).toContainText(/retention/i);
  await expect(cards.first().getByTestId("ai-unknown")).toHaveCount(6);

  // With a reason it is recorded and flagged as under the six months.
  await page.getByTestId("ai-overseer").fill("Head of Mobility");
  await page.getByTestId("ai-category").selectOption("task_allocation");
  await page
    .getByTestId("ai-note")
    .fill("Data protection law requires deletion at three months");
  await page.getByTestId("ai-save").click();
  await expect(cards.first()).toContainText("Head of Mobility");
  await expect(cards.first()).toContainText("3 months");
  await expect(cards.first().getByTestId("ai-below-minimum")).toBeVisible();
  if (shots) {
    await page.screenshot({
      path: `${shots}/ai-register-live-recorded.png`,
      fullPage: true,
    });
    await page.setViewportSize({ width: 390, height: 800 });
    await page.screenshot({
      path: `${shots}/ai-register-live-phone.png`,
      fullPage: true,
    });
  }
});

test("the real register in the dark theme", async ({ page }) => {
  test.skip(!shots, "set PAGE_SHOTS=<dir> to write the screenshot");
  await page.addInitScript(() => localStorage.setItem("mxi.wpm.theme", "dark"));
  await page.goto("/ai-register");
  await expect(page.getByTestId("ai-feature").first()).toBeVisible();
  await page.screenshot({
    path: `${shots}/ai-register-live-dark.png`,
    fullPage: true,
  });
});
