// One loader (WPM-T260): on a worker's page a failed secondary panel reports its own error in its own place
// and the rest of the page still shows. The API is stubbed per test; an unstubbed /api/proxy call 404s loudly.

import { expect, test, type Page } from "@playwright/test";

async function signIn(page: Page) {
  await page.context().addCookies([
    {
      name: "__Host-mxi_session",
      value: "loader-spec-session",
      domain: "localhost",
      path: "/",
      httpOnly: true,
      secure: true,
      sameSite: "Lax",
    },
  ]);
}

const WORKER = {
  pid: "11111111-1111-4111-8111-111111111111",
  person_ref: "person:22222222-2222-4222-8222-222222222222",
  upstream_worker_ref: null,
  organization_ref: "organization:33333333-3333-4333-8333-333333333333",
  worker_number: "E-0001",
  display_name: "Test Worker 001",
  status: "active",
  employment_type: "employee",
  fte_percent: 100,
  department: "engineering",
  job_title: "Engineer",
  location: null,
  manager_pid: null,
  salary_minor: null,
  salary_currency: null,
  hired_on: "2026-01-05",
  terminated_on: null,
  engagement_ends_on: null,
};

test.describe("a worker's page loads panel by panel (WPM-T260)", () => {
  test.beforeEach(async ({ page }) => {
    await signIn(page);
    await page.route("**/api/proxy/**", (route) =>
      route.fulfill({
        status: 404,
        body: "unstubbed: " + route.request().url(),
      }),
    );
    for (const list of [
      "onboarding",
      "leave-entitlements",
      "leave-requests",
      "reviews",
      "training-enrollments",
      "appraisals",
      "appraisal-requests",
      "notifications",
      "ergonomic-assessments",
      "adjustment-requests",
    ]) {
      await page.route(`**/api/proxy/workers/${WORKER.pid}/${list}`, (route) =>
        route.fulfill({ json: [] }),
      );
    }
    await page.route(
      `**/api/proxy/workers/${WORKER.pid}/wellbeing-prompts`,
      (route) => route.fulfill({ json: { prompts: [] } }),
    );
    await page.route("**/api/proxy/pulse-surveys", (route) =>
      route.fulfill({ json: [] }),
    );
    await page.route(`**/api/proxy/workers/${WORKER.pid}`, (route) =>
      route.fulfill({ json: WORKER }),
    );
  });

  test("a failed payslips panel says so and hides nothing else", async ({
    page,
  }) => {
    await page.route(`**/api/proxy/workers/${WORKER.pid}/payslips`, (route) =>
      route.fulfill({
        status: 500,
        json: { error: "internal", description: "payslips are down" },
      }),
    );
    await page.goto(`/workers/${WORKER.pid}`);
    await expect(page.getByTestId("facts")).toContainText("Engineer");
    await expect(page.getByTestId("panel-error-payslips")).toBeVisible();
    // The other panels loaded, so none of them reports an error.
    await expect(page.getByTestId("panel-error-leave")).toHaveCount(0);
    await expect(page.getByTestId("panel-error-balances")).toHaveCount(0);
    await expect(page.getByTestId("appraisals")).toBeVisible();
  });

  test("when the worker itself fails to load, the page says so", async ({
    page,
  }) => {
    await page.route(`**/api/proxy/workers/${WORKER.pid}`, (route) =>
      route.fulfill({
        status: 500,
        json: { error: "internal", description: "down" },
      }),
    );
    await page.goto(`/workers/${WORKER.pid}`);
    await expect(page.getByTestId("error")).toBeVisible();
    await expect(page.getByTestId("facts")).toHaveCount(0);
  });
});
