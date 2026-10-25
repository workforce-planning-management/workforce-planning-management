// Pay position (`/pay-range`). The API is stubbed per test; an unstubbed /api/proxy call 404s loudly.

import { expect, test } from "@playwright/test";
import { stubSignedIn } from "./support";

const shown = (workers: number, compa: number, pen: number | null) => ({
  kind: "shown",
  value: {
    workers,
    compa_mean_bp: compa,
    penetration: pen === null ? { kind: "unknown", reason: "no_width" } : { kind: "shown", value: { workers, mean_bp: pen } },
  },
});

const REPORT = {
  organization: "o", as_of: "2026-10-25", currency: "GBP",
  basis: "Annual full-time-equivalent basic salary over the band midpoint.",
  derivation: "An outlier is a group, never a worker.",
  report: {
    floor: 10, cutoff_bp: 1000,
    overall: shown(40, 10123, 4980),
    groupings: [
      {
        by: "band",
        groups: [
          { key: "B3", figure: shown(25, 10456, 5100), outlier: { kind: "no" } },
          { key: "B4", figure: { kind: "withheld" }, outlier: { kind: "unknown" } },
          { key: "B5", figure: shown(15, 8800, null), outlier: { kind: "below", delta_bp: 1323 } },
        ],
      },
      { by: "level", groups: [] }, { by: "department", groups: [] }, { by: "location", groups: [] },
    ],
    excluded: { no_pay: { kind: "under_floor" }, no_band: { kind: "exactly", workers: 12 } },
  },
};

test.describe("Pay position", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
  });

  test("shows compa-ratio and penetration, withholds a small group, names an outlier group, and asks again with the cut-off", async ({ page }) => {
    const urls: string[] = [];
    await page.route("**/api/proxy/reports/pay-range?*", (route) => {
      urls.push(route.request().url());
      return route.fulfill({ json: REPORT });
    });
    await page.goto("/pay-range");
    const band = page.getByTestId("pay-range-band");
    await expect(band.getByTestId("pay-range-row")).toHaveCount(3);
    await expect(band.getByTestId("pay-range-row").nth(0)).toContainText("104.56%");
    await expect(band.getByTestId("pay-range-row").nth(0)).toContainText("51.00%");
    await expect(band.getByTestId("pay-range-withheld")).toContainText("Fewer than 10 workers");
    await expect(band.getByTestId("pay-range-row").nth(2)).toContainText("The band has one rate, so no range");
    await expect(band.getByTestId("pay-range-outlier").nth(2)).toContainText("Below everyone by 13.23%");
    await expect(page.getByTestId("pay-range-overall")).toContainText("40 workers, mean compa-ratio 101.23%");
    await expect(page.getByTestId("pay-range-basis")).toContainText("No pay recorded: fewer than 10");
    await expect(page.getByTestId("pay-range-basis")).toContainText("No band: 12");
    await page.getByTestId("pay-range-cutoff").fill("5");
    await expect.poll(() => urls.some((u) => u.includes("cutoff=5"))).toBe(true);
  });

  test("a refusal is shown in words and no figure is drawn", async ({ page }) => {
    await page.route("**/api/proxy/reports/pay-range?*", (route) =>
      route.fulfill({ status: 403, json: { error: "a pay position analysis is made of salaries and needs payroll read access" } }),
    );
    await page.goto("/pay-range");
    await expect(page.getByTestId("error")).toContainText("You are not allowed to do this.");
    await expect(page.getByTestId("pay-range-band")).toHaveCount(0);
  });
});
