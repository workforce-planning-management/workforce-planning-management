// Workers and payroll. The API is stubbed per test; an unstubbed /api/proxy call 404s loudly.

import { expect, test } from "@playwright/test";
import { WORKER, stubSignedIn } from "./support";

test.describe("Workers and payroll", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
  });

  test("worker list shows money for visible salaries and Hidden for masked", async ({
    page,
  }) => {
    await page.goto("/workers");
    const table = page.getByTestId("worker-table");
    await expect(table).toContainText("E-0001");
    await expect(table).toContainText("£36,000.00");
    await expect(table).toContainText("Hidden");
  });

  test("payroll run detail drives the lifecycle actions", async ({ page }) => {
    const run = {
      pid: "66666666-6666-4666-8666-666666666666",
      organization_ref: WORKER.organization_ref,
      period_start: "2026-07-01",
      period_end: "2026-07-31",
      status: "calculated",
    };
    await page.route(`**/api/proxy/payroll-runs/${run.pid}`, (route) =>
      route.fulfill({ json: run }),
    );
    await page.route(`**/api/proxy/payroll-runs/${run.pid}/payslips`, (route) =>
      route.fulfill({
        json: [
          {
            pid: "77777777-7777-4777-8777-777777777777",
            run_pid: run.pid,
            worker_pid: WORKER.pid,
            currency: "GBP",
            gross_minor: 300000,
            deductions: [{ label: "tax", amount_minor: 39050 }],
            net_minor: 260950,
          },
        ],
      }),
    );
    await page.goto(`/payroll/${run.pid}`);
    await expect(page.getByTestId("run-status")).toHaveText("calculated");
    await expect(page.getByTestId("action-approve")).toBeVisible();
    const payslips = page.getByTestId("payslips");
    await expect(payslips).toContainText("£3,000.00");
    await expect(payslips).toContainText("£2,609.50");
    await expect(payslips).toContainText("tax");
  });
});
