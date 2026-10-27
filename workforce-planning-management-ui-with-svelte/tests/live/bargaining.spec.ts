// Bargaining, consultations and decision gates against the real service (see README.md). Needs no seed data: each run uses its own
// organization reference (enforcement is off, so the scope call is stubbed to name it; everything else is real). A unit with no workers
// is under the floor, so its pack gives nothing. Set PAGE_SHOTS=<dir> for screenshots.

import { expect, test } from "@playwright/test";

test.skip(!process.env.WPM_LIVE_API, "needs a running service: set WPM_LIVE_API=1");

const shots = process.env.PAGE_SHOTS;
const org = `organization:${crypto.randomUUID()}`;

test("a body, a unit under the floor, facility time over budget, a consultation, and a gate", async ({ page }) => {
  await page.context().addCookies([
    { name: "__Host-mxi_session", value: "live-test-session", domain: "localhost", path: "/", httpOnly: true, secure: true, sameSite: "Lax" },
  ]);
  await page.route("**/api/proxy/me/organizations/scope", (route) => route.fulfill({ json: [org] }));

  await page.goto("/bargaining");
  await expect(page.getByTestId("bg-none")).toBeVisible();
  await page.getByTestId("bg-name").fill("Care Union");
  await page.getByTestId("bg-date").fill("2020-03-01");
  await page.getByTestId("bg-role").fill("HR relations lead");
  await page.getByTestId("bg-add").click();
  await expect(page.getByTestId("bg-body")).toContainText("contact role: HR relations lead");

  await page.getByTestId("bg-show-units").click();
  await page.getByTestId("bg-unit-name").fill("Care staff");
  await page.getByTestId("bg-unit-department").fill("Care");
  await page.getByTestId("bg-unit-add").click();
  await expect(page.getByTestId("bg-unit")).toHaveCount(1);
  await expect(page.getByTestId("bg-coverage")).toContainText(/no one|fewer than/);
  await page.getByTestId("bg-pack").click();
  await expect(page.getByTestId("bg-pack-result")).toContainText("it is logged");

  await page.getByTestId("bg-fstart").fill("2026-01-01");
  await page.getByTestId("bg-fend").fill("2026-03-31");
  await page.getByTestId("bg-fminutes").fill("700");
  await page.getByTestId("bg-fbudget").fill("600");
  await page.getByTestId("bg-fadd").click();
  await expect(page.getByTestId("bg-period")).toContainText("700");
  await expect(page.getByTestId("bg-over")).toBeVisible();
  if (shots) await page.screenshot({ path: `${shots}/bargaining-live.png`, fullPage: true });

  await page.goto("/consultations");
  await expect(page.getByTestId("cs-none")).toBeVisible();
  await page.getByTestId("cs-establishment").fill("Leeds site");
  await page.getByTestId("cs-size").fill("200");
  await page.getByTestId("cs-dismissals").fill("25");
  await page.getByTestId("cs-termination").fill("2027-12-01");
  await page.getByTestId("cs-topic-redundancy").check();
  await page.getByTestId("cs-open").click();
  await expect(page.getByTestId("cs-row")).toContainText("Leeds site");
  await page.getByTestId("cs-timeline").click();
  await expect(page.getByTestId("cs-assessment")).toContainText("A collective duty may apply");
  await expect(page.getByTestId("cs-assessment")).toContainText("not a verdict");
  await page.getByTestId("cs-began").fill("2026-10-01");
  await page.getByTestId("cs-begin").click();
  await expect(page.getByTestId("cs-timeliness")).toContainText("began in time");
  if (shots) await page.screenshot({ path: `${shots}/consultations-live.png`, fullPage: true });

  await page.goto("/decision-gates");
  await page.getByTestId("dg-change").selectOption("new_or_changed_score");
  await page.getByTestId("dg-add").click();
  await expect(page.getByTestId("dg-gate")).toContainText("Care Union");
  await page.getByTestId("dg-check").click();
  await expect(page.getByTestId("dg-standing")).toContainText("1 body or bodies to consult");
  await expect(page.getByTestId("dg-result")).toContainText("section 11");
  if (shots) await page.screenshot({ path: `${shots}/decision-gates-live.png`, fullPage: true });
});
