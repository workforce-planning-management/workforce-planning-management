// Pay gap (`/pay-gap`): categories and the seven indicators. The API is stubbed per test; an unstubbed /api/proxy call 404s loudly.

import { expect, test } from "@playwright/test";
import { stubSignedIn } from "./support";

const gap = (female: number, male: number, bp: number) => ({ status: "shown", value: { female_minor: female, male_minor: male, gap_bp: bp } });

const REPORT = {
  organization: "o", as_of: "2026-10-25", currency: "GBP",
  basis: { pay: "Annual full-time-equivalent basic salary.", variable: "WPM holds no variable pay.", variable_gap_taken_over: "all_known" },
  reporting: { every_years: 1, first_report_on: "2027-06-07", next_report_on: "2027-06-07" },
  derivation: "No figure is a head count.",
  report: {
    floor: 10,
    mean_gap: gap(4_000_000, 4_400_000, 909),
    mean_variable_gap: { status: "unknown", value: "variable_not_recorded" },
    median_gap: { status: "withheld" },
    median_variable_gap: { status: "unknown", value: "no_workers" },
    receiving_variable: { status: "unknown", value: "no_workers" },
    quartiles: [
      { status: "shown", value: { female_bp: 6000, male_bp: 4000 } },
      { status: "withheld" }, { status: "withheld" }, { status: "withheld" },
    ],
    categories: [
      { category: "Care", basic: gap(3_000_000, 3_000_000, 0), variable: { status: "unknown", value: "no_workers" } },
      { category: "Legal", basic: { status: "unknown", value: "one_sided" }, variable: { status: "unknown", value: "no_workers" } },
    ],
    exclusions: {
      no_sex_declared: { status: "under_floor" }, sex_not_female_or_male: { status: "exactly", value: 0 },
      no_pay_recorded: { status: "exactly", value: 12 }, variable_unknown: { status: "exactly", value: 40 }, uncategorised: { status: "under_floor" },
    },
  },
};

test.describe("Pay gap", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
  });

  test("shows the indicators with their reasons, withholds small cells, names no one, and offers the CSV", async ({ page }) => {
    const urls: string[] = [];
    await page.route("**/api/proxy/pay-categories?*", (route) => route.fulfill({ json: [] }));
    await page.route("**/api/proxy/pay-transparency/gap-report?*", (route) => {
      urls.push(route.request().url());
      return route.fulfill({ json: REPORT });
    });
    await page.goto("/pay-gap");
    const rows = page.getByTestId("indicator-row");
    await expect(rows.nth(0)).toContainText("9.09%");
    await expect(rows.nth(0)).toContainText("£40,000.00");
    await expect(rows.nth(1).getByTestId("gap-unknown")).toContainText("Nobody's variable pay is recorded");
    await expect(rows.nth(2).getByTestId("gap-withheld")).toContainText("Withheld: fewer than 10 of one sex");
    await expect(page.getByTestId("quartile-row")).toHaveCount(4);
    await expect(page.getByTestId("quartile-row").nth(0)).toContainText("60.00%");
    await expect(page.getByTestId("category-gap-row").nth(1)).toContainText("Only one sex is present");
    await expect(page.getByTestId("pay-gap-exclusions")).toContainText("No basic pay recorded: 12");
    await expect(page.getByTestId("pay-gap-exclusions")).toContainText("In no category: fewer than 10");
    await expect(page.getByTestId("pay-gap-reporting")).toContainText("first on 2027-06-07");
    await expect(page.getByTestId("pay-gap-audited")).toContainText("audit log");
    await expect(page.getByTestId("pay-gap-csv")).toHaveAttribute("href", /\/api\/proxy\/pay-transparency\/gap-report\/csv\?organization=/);
    await page.getByTestId("pay-gap-variable").selectOption("recipients_only");
    await expect.poll(() => urls.some((u) => u.includes("variable=recipients_only"))).toBe(true);
  });

  test("HR sets up a category from job titles, one per line, and removes it", async ({ page }) => {
    const rows: Array<Record<string, unknown>> = [];
    let posted: Record<string, unknown> | null = null;
    await page.route("**/api/proxy/pay-transparency/gap-report?*", (route) => route.fulfill({ json: REPORT }));
    await page.route("**/api/proxy/pay-categories?*", (route) => route.fulfill({ json: rows }));
    await page.route("**/api/proxy/pay-categories", (route) => {
      posted = route.request().postDataJSON();
      rows.push({ pid: "c1", organization_ref: "o", ...posted });
      return route.fulfill({ json: { pid: "c1" } });
    });
    await page.route("**/api/proxy/pay-categories/c1", (route) => {
      rows.length = 0;
      return route.fulfill({ json: {} });
    });
    await page.goto("/pay-gap");
    await expect(page.getByTestId("categories-none")).toBeVisible();
    await page.getByTestId("category-new").click();
    await page.getByTestId("category-name").fill("Care");
    await page.getByTestId("category-basis").fill("Same skills and responsibility");
    await page.getByTestId("category-titles").fill("Carer\n  Senior carer \n\n");
    await page.getByTestId("category-save").click();
    await expect(page.getByTestId("category-row")).toContainText("Carer, Senior carer");
    expect(posted).toMatchObject({ name: "Care", basis: "Same skills and responsibility", job_titles: ["Carer", "Senior carer"] });
    await page.getByTestId("category-delete").click();
    await expect(page.getByTestId("categories-none")).toBeVisible();
  });

  test("a refusal of the report is shown in words and no figure is drawn", async ({ page }) => {
    await page.route("**/api/proxy/pay-categories?*", (route) => route.fulfill({ json: [] }));
    await page.route("**/api/proxy/pay-transparency/gap-report?*", (route) => route.fulfill({ status: 403, json: { error: "forbidden" } }));
    await page.goto("/pay-gap");
    await expect(page.getByTestId("error")).toContainText("You are not allowed to do this.");
    await expect(page.getByTestId("pay-gap-indicators")).toHaveCount(0);
  });
});
