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
            statutory_overtime_minor: 4500,
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
    await expect(payslips).toContainText("Statutory overtime premium (in gross)");
    await expect(payslips).toContainText("£45.00");
    await expect(payslips).toContainText("tax");
  });

  test("the minimum-wage check runs only when asked and shows no age", async ({
    page,
  }) => {
    const run = {
      pid: "88888888-8888-4888-8888-888888888888",
      organization_ref: WORKER.organization_ref,
      period_start: "2026-07-01",
      period_end: "2026-07-31",
      status: "calculated",
    };
    let asked = 0;
    await page.route(`**/api/proxy/payroll-runs/${run.pid}`, (route) =>
      route.fulfill({ json: run }),
    );
    await page.route(`**/api/proxy/payroll-runs/${run.pid}/payslips`, (route) =>
      route.fulfill({ json: [] }),
    );
    await page.route(
      `**/api/proxy/payroll-runs/${run.pid}/minimum-wage-check`,
      (route) => {
        asked += 1;
        return route.fulfill({
          json: {
            run_pid: run.pid,
            period_start: run.period_start,
            period_end: run.period_end,
            indicative: true,
            note: "Indicative only. An ok is not a statement that an employer complies.",
            payslips: 2,
            counts: { below: 1, unknown: 1 },
            unknown_reasons: { no_date_of_birth: 1 },
            workers: [
              {
                worker_pid: WORKER.pid,
                outcome: "below",
                reason: null,
                band: "adult",
              },
              {
                worker_pid: "99999999-9999-4999-8999-999999999999",
                outcome: "unknown",
                reason: "no_date_of_birth",
                band: null,
              },
            ],
          },
        });
      },
    );
    await page.goto(`/payroll/${run.pid}`);
    await expect(page.getByTestId("min-wage-run")).toBeVisible();
    expect(asked, "an audited read is never made on page load").toBe(0);
    await page.getByTestId("min-wage-run").click();
    await expect(page.getByTestId("min-wage-note")).toContainText(
      "Indicative only",
    );
    await expect(page.getByTestId("min-wage-count-below")).toContainText(
      "Below: 1",
    );
    const rows = page.getByTestId("min-wage-rows");
    await expect(rows).toContainText("no date of birth");
    await expect(rows).not.toContainText(/\b\d{2} years\b/);
    expect(asked).toBe(1);
  });

  test("the overtime check runs only when asked, says not enough where it cannot tell, and changes no pay", async ({
    page,
  }) => {
    const run = {
      pid: "88888888-8888-4888-8888-888888888888",
      organization_ref: WORKER.organization_ref,
      period_start: "2026-07-01",
      period_end: "2026-07-31",
      status: "calculated",
    };
    let asked = 0;
    await page.route(`**/api/proxy/payroll-runs/${run.pid}`, (route) =>
      route.fulfill({ json: run }),
    );
    await page.route(`**/api/proxy/payroll-runs/${run.pid}/payslips`, (route) =>
      route.fulfill({ json: [] }),
    );
    await page.route(
      `**/api/proxy/payroll-runs/${run.pid}/overtime-check`,
      (route) => {
        asked += 1;
        return route.fulfill({
          json: {
            run_pid: run.pid,
            period_start: run.period_start,
            period_end: run.period_end,
            indicative: true,
            note: "Indicative only. Nothing here changes a payslip.",
            payslips: 3,
            counts: { derived: 1, exempt: 1, unknown: 1 },
            reasons: { status_not_recorded: 1 },
            workers: [
              {
                worker_pid: WORKER.pid,
                outcome: "derived",
                reason: null,
                overtime_minutes: 360,
                minutes_at: { "15000": 360 },
                premium_minor: 4500,
                currency: "GBP",
              },
              {
                worker_pid: "99999999-9999-4999-8999-999999999999",
                outcome: "exempt",
                reason: null,
                below_exempt_floor: null,
              },
              {
                worker_pid: "77777777-7777-4777-8777-777777777777",
                outcome: "unknown",
                reason: "status_not_recorded",
              },
            ],
          },
        });
      },
    );
    await page.goto(`/payroll/${run.pid}`);
    await expect(page.getByTestId("overtime-run")).toBeVisible();
    expect(asked, "an audited read is never made on page load").toBe(0);
    await page.getByTestId("overtime-run").click();
    await expect(page.getByTestId("overtime-note")).toContainText(
      "changes a payslip",
    );
    await expect(page.getByTestId("overtime-count-derived")).toContainText(
      "Derived: 1",
    );
    const rows = page.getByTestId("overtime-rows");
    await expect(rows).toContainText("Overtime status not recorded");
    await expect(rows).toContainText("No floor configured");
    await expect(rows).toContainText("360");
    expect(asked).toBe(1);
  });
});
