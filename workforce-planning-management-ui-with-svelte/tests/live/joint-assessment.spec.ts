// The joint pay assessment against the real service (see README.md; the same set-up as `pay-gap.spec.ts`, `PAY_GAP_ORG`): a category
// whose gap in basic pay is at the threshold (5.00%) and a category whose gap is withheld. With enforcement off the service reports no
// organization scope, so that one call is stubbed; the signals and the records are real. It records a report date eight months ago
// (past the six-month period) and then a justification, so it changes the data: seed again to rerun. `PAGE_SHOTS=<dir>` writes screenshots.

import { expect, test } from "@playwright/test";

test.skip(
  !process.env.WPM_LIVE_API,
  "needs a running service: set WPM_LIVE_API=1",
);

const org = process.env.PAY_GAP_ORG ?? "";
const shots = process.env.PAGE_SHOTS;

const daysAgo = (n: number) =>
  new Date(Date.now() - n * 86_400_000).toISOString().slice(0, 10);

test("the real signal: pending until reported, due when the period has run out, not due once justified; a withheld gap cannot be said", async ({
  page,
}) => {
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
  await page.goto("/pay-gap");
  const rows = page.getByTestId("assessment-row");
  await expect(rows).toHaveCount(2);
  const care = rows.filter({ hasText: "Care" });
  const legal = rows.filter({ hasText: "Legal" });
  await expect(care.getByTestId("assessment-signal")).toContainText("Pending");
  await expect(legal.getByTestId("assessment-signal")).toContainText(
    "withheld",
  );

  // Reported eight months ago, nothing justified or remedied: the period has run out and an assessment is due.
  await care.getByTestId("assessment-edit").click();
  await page.getByTestId("assessment-reported").fill(daysAgo(240));
  await page.getByTestId("assessment-save").click();
  await expect(care.getByTestId("assessment-signal")).toContainText(
    "A joint assessment is due",
  );
  if (shots)
    await page.screenshot({
      path: `${shots}/joint-assessment-live-due.png`,
      fullPage: true,
    });

  // A justification on objective criteria ends it. One that is too short is stopped by the form and nothing is sent.
  await care.getByTestId("assessment-edit").click();
  await page.getByTestId("assessment-justified").check();
  await page.getByTestId("assessment-justification").fill("Too short");
  await page.getByTestId("assessment-save").click();
  await expect(page.getByTestId("assessment-form")).toBeVisible();
  await expect(care.getByTestId("assessment-signal")).toContainText(
    "A joint assessment is due",
  );
  await page
    .getByTestId("assessment-justification")
    .fill("Different qualifications set by a published pay scale");
  await page.getByTestId("assessment-save").click();
  await expect(care.getByTestId("assessment-signal")).toContainText(
    "Not due: the gap is justified",
  );
  await expect(care.getByTestId("assessment-recorded")).toContainText(
    "Justified.",
  );
  if (shots)
    await page.screenshot({
      path: `${shots}/joint-assessment-live-justified.png`,
      fullPage: true,
    });
});
