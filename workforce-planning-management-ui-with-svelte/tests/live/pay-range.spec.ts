// Pay position against the real service (see README.md). Needs at least ten workers on one band with pay and fewer than ten on another
// (`PAY_RANGE_ORG` names the organization). With enforcement off the service reports no organization scope, so this one call is stubbed
// to name it; the report itself is real. Asserts rules and shapes, never exact figures. Set PAGE_SHOTS=<dir> for light, dark and phone
// screenshots.

import { expect, test } from "@playwright/test";

test.skip(!process.env.WPM_LIVE_API, "needs a running service: set WPM_LIVE_API=1");

const org = process.env.PAY_RANGE_ORG ?? "";
const shots = process.env.PAGE_SHOTS;

test("the real report: a small group is withheld, no one is named, the audit is said, a currency can be chosen", async ({ page }) => {
  await page.context().addCookies([
    { name: "__Host-mxi_session", value: "live-test-session", domain: "localhost", path: "/", httpOnly: true, secure: true, sameSite: "Lax" },
  ]);
  await page.route("**/api/proxy/me/organizations/scope", (route) => route.fulfill({ json: [org] }));
  await page.goto("/pay-range");

  const band = page.getByTestId("pay-range-band");
  await expect(band).toBeVisible();
  await expect(band.getByTestId("pay-range-withheld")).toHaveCount(1); // the band with fewer than ten
  await expect(band.getByTestId("pay-range-row").filter({ hasText: "%" })).not.toHaveCount(0);
  await expect(page.getByTestId("pay-range-audited")).toBeVisible();
  // Everyone seeded has pay and a band, so nobody is left out, and that is said as none, not as "fewer than ten".
  await expect(page.getByTestId("pay-range-basis")).toContainText("No pay recorded: 0");
  await expect(page.getByTestId("pay-range-basis")).toContainText("No band: 0");
  // With a band withheld, a total that would let it be worked out by subtraction is withheld too (WPM-D108).
  await expect(page.getByTestId("pay-range-overall")).toContainText("Showing it would reveal a withheld group");
  const text = await page.locator("main").innerText();
  expect(text).not.toMatch(/Live Person|LV-\d/); // no one is named

  if (shots) {
    await page.screenshot({ path: `${shots}/pay-range-live-light.png`, fullPage: true });
    await page.setViewportSize({ width: 390, height: 800 });
    await page.screenshot({ path: `${shots}/pay-range-live-phone.png`, fullPage: true });
    await page.setViewportSize({ width: 1280, height: 900 });
  }

  // A currency nobody is paid in leaves everyone out, and the page says why rather than drawing zeros.
  const again = page.waitForResponse((r) => r.url().includes("currency=USD"));
  await page.getByTestId("pay-range-currency").fill("usd");
  expect((await again).status()).toBe(200);
  await expect(page.getByTestId("pay-range-overall")).not.toContainText("compa-ratio");
  await expect(page.getByTestId("pay-range-no-groups-level")).toBeVisible();
});

test("the real report in the dark theme", async ({ page }) => {
  test.skip(!shots, "set PAGE_SHOTS=<dir> to write the screenshot");
  await page.context().addCookies([
    { name: "__Host-mxi_session", value: "live-test-session", domain: "localhost", path: "/", httpOnly: true, secure: true, sameSite: "Lax" },
  ]);
  await page.addInitScript(() => localStorage.setItem("mxi.wpm.theme", "dark"));
  await page.route("**/api/proxy/me/organizations/scope", (route) => route.fulfill({ json: [org] }));
  await page.goto("/pay-range");
  await expect(page.getByTestId("pay-range-band")).toBeVisible();
  await page.screenshot({ path: `${shots}/pay-range-live-dark.png`, fullPage: true });
});
