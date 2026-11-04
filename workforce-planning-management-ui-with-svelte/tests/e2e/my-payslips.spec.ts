// My payslips (WPM-R178): the person's approved and paid payslips, the year to date, a query, and payroll's list. The API is
// stubbed per test; an unstubbed /api/proxy call 404s loudly.

import { expect, test } from "@playwright/test";
import { stubSignedIn } from "./support";

const SLIP = {
  pid: "11111111-1111-4111-8111-111111111111",
  period_start: "2026-07-01",
  period_end: "2026-07-31",
  run_status: "paid",
  currency: "GBP",
  gross_minor: 300000,
  statutory_overtime_minor: 4500,
  deductions: [
    { label: "tax", amount_minor: 50000 },
    { label: "pension", amount_minor: 15000 },
  ],
  net_minor: 235000,
  query: null,
};

const YEAR = {
  starts_on: "2026-04-06",
  ends_on: "2027-04-05",
  currency: "GBP",
  payslips: 1,
  gross_minor: 300000,
  deductions: SLIP.deductions,
  net_minor: 235000,
};

test.describe("My payslips", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
  });

  test("shows the lines and the year to date", async ({ page }) => {
    await page.route("**/api/proxy/me/payslips", (route) =>
      route.fulfill({ json: { payslips: [SLIP], year_to_date: [YEAR] } }),
    );
    await page.goto("/me/payslips");
    const slip = page.getByTestId("payslip");
    await expect(slip).toContainText("£3,000.00");
    await expect(slip).toContainText("Statutory overtime premium (in gross)");
    await expect(page.getByTestId("payslip-net")).toHaveText("£2,350.00");
    await expect(page.getByTestId("payslips-ytd-year")).toContainText(
      "£2,350.00",
    );
  });

  test("with no tax year start says so and shows no total", async ({
    page,
  }) => {
    await page.route("**/api/proxy/me/payslips", (route) =>
      route.fulfill({ json: { payslips: [SLIP], year_to_date: null } }),
    );
    await page.goto("/me/payslips");
    await expect(page.getByTestId("payslips-ytd-none")).toBeVisible();
    await expect(page.getByTestId("payslips-ytd-year")).toHaveCount(0);
  });

  test("with no payslips says so", async ({ page }) => {
    await page.route("**/api/proxy/me/payslips", (route) =>
      route.fulfill({ json: { payslips: [], year_to_date: [] } }),
    );
    await page.goto("/me/payslips");
    await expect(page.getByTestId("payslips-none")).toBeVisible();
  });

  test("a query is sent with its comment and then shown as open", async ({
    page,
  }) => {
    let raised = false;
    let body: unknown = null;
    await page.route("**/api/proxy/me/payslips", (route) =>
      route.fulfill({
        json: {
          payslips: [
            {
              ...SLIP,
              query: raised
                ? {
                    pid: "q",
                    comment: "Short",
                    raised_on: "2026-08-02",
                    closed_on: null,
                    open: true,
                  }
                : null,
            },
          ],
          year_to_date: null,
        },
      }),
    );
    await page.route(
      `**/api/proxy/me/payslips/${SLIP.pid}/query`,
      async (route) => {
        body = route.request().postDataJSON();
        raised = true;
        await route.fulfill({ json: { pid: "q" } });
      },
    );
    await page.goto("/me/payslips");
    await page.getByTestId("payslip-query-open").click();
    await page.getByTestId("payslip-query-comment").fill("Short");
    await page.getByTestId("payslip-query-send").click();
    await expect(page.getByTestId("payslip-query")).toBeVisible();
    expect(body).toEqual({ comment: "Short" });
    await expect(page.getByTestId("payslip-query-open")).toHaveCount(0);
  });

  test("printing hides the page around the payslip", async ({ page }) => {
    await page.route("**/api/proxy/me/payslips", (route) =>
      route.fulfill({ json: { payslips: [SLIP], year_to_date: [YEAR] } }),
    );
    await page.goto("/me/payslips");
    await expect(page.getByTestId("payslip")).toBeVisible();
    await page.emulateMedia({ media: "print" });
    await expect(page.getByTestId("payslip")).toBeVisible();
    await expect(page.locator("nav.top")).toBeHidden();
    await expect(page.getByTestId("payslip-print")).toBeHidden();
    await expect(page.getByTestId("payslips-ytd")).toBeHidden();
  });

  test("payroll sees a query and closes it", async ({ page }) => {
    let open = true;
    const row = {
      pid: "22222222-2222-4222-8222-222222222222",
      worker_pid: "w",
      display_name: "Alex Example",
      payslip_pid: SLIP.pid,
      run_pid: "r",
      period_start: "2026-07-01",
      period_end: "2026-07-31",
      comment: "The overtime looks short",
      raised_on: "2026-08-02",
      closed_on: null,
      open: true,
    };
    await page.route("**/api/proxy/payroll-runs*", (route) =>
      route.fulfill({ json: [] }),
    );
    await page.route("**/api/proxy/payslip-queries", (route) =>
      route.fulfill({ json: open ? [row] : [] }),
    );
    await page.route(
      `**/api/proxy/payslip-queries/${row.pid}/close`,
      async (route) => {
        open = false;
        await route.fulfill({ json: { ...row, open: false } });
      },
    );
    await page.goto("/payroll");
    await expect(page.getByTestId("payslip-query-row")).toContainText(
      "The overtime looks short",
    );
    await page.getByTestId("payslip-query-close").click();
    await expect(page.getByTestId("payslip-queries-none")).toBeVisible();
  });

  test("a caller refused the queries sees no panel", async ({ page }) => {
    await page.route("**/api/proxy/payroll-runs*", (route) =>
      route.fulfill({ json: [] }),
    );
    await page.route("**/api/proxy/payslip-queries", (route) =>
      route.fulfill({ status: 403, json: {} }),
    );
    await page.goto("/payroll");
    await expect(page.getByTestId("payslip-queries")).toHaveCount(0);
  });
});
