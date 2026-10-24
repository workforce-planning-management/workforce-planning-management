// Payroll (`/payroll`): the runs, each linking to its page. The run page itself is covered in
// `workers-and-payroll.spec.ts`. The API is stubbed per test; an unstubbed /api/proxy call 404s loudly.

import { expect, test } from "@playwright/test";
import { WORKER, stubSignedIn } from "./support";

const RUNS = [
  { pid: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa", organization_ref: WORKER.organization_ref, period_start: "2026-09-01", period_end: "2026-09-30", status: "paid" },
  { pid: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb", organization_ref: WORKER.organization_ref, period_start: "2026-10-01", period_end: "2026-10-31", status: "draft" },
];

test.describe("Payroll runs", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
  });

  test("lists the runs with their period and status, each linking to its page", async ({ page }) => {
    await page.route("**/api/proxy/payroll-runs", (route) => route.fulfill({ json: RUNS }));
    await page.goto("/payroll");
    await expect(page).toHaveTitle("Payroll — WPM");
    const table = page.getByTestId("runs");
    await expect(table).toContainText("2026-09-01 → 2026-09-30");
    await expect(table).toContainText("2026-10-01 → 2026-10-31");
    const link = table.getByRole("link", { name: "2026-10-01 → 2026-10-31" });
    await expect(link).toHaveAttribute("href", `/en-001/payroll/${RUNS[1]?.pid}`);
  });

  test("a caller who may not read payroll sees the error, not an empty list", async ({ page }) => {
    await page.route("**/api/proxy/payroll-runs", (route) =>
      route.fulfill({ status: 403, json: { description: "payroll is for HR" } }),
    );
    await page.goto("/payroll");
    await expect(page.getByTestId("error")).toContainText("payroll is for HR");
    await expect(page.getByTestId("runs")).toHaveCount(0);
  });

  test("no runs yet is an empty table, not an error", async ({ page }) => {
    await page.route("**/api/proxy/payroll-runs", (route) => route.fulfill({ json: [] }));
    await page.goto("/payroll");
    await expect(page.getByTestId("runs")).toBeVisible();
    await expect(page.getByTestId("error")).toHaveCount(0);
  });
});
