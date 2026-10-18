// What an export holds (WPM-R133): the same figures the table shows, with units and currency, and an unknown as a dash.

import { expect, test, type Page } from "@playwright/test";

async function stub(page: Page) {
  await page.context().addCookies([
    {
      name: "__Host-mxi_session",
      value: "export-spec-session",
      domain: "localhost",
      path: "/",
      httpOnly: true,
      secure: true,
      sameSite: "Lax",
    },
  ]);
  await page.route("**/api/proxy/**", (route) =>
    route.fulfill({ status: 404, body: "unstubbed: " + route.request().url() }),
  );
  await page.route("**/api/proxy/me/organizations**", (route) =>
    route.fulfill({ json: [] }),
  );
}

async function downloadText(
  page: Page,
  button: string,
): Promise<{ name: string; text: string }> {
  const download = page.waitForEvent("download");
  await page.getByRole("button", { name: button }).first().click();
  const file = await download;
  const { readFileSync } = await import("node:fs");
  return {
    name: file.suggestedFilename(),
    text: readFileSync((await file.path())!, "utf8"),
  };
}

test("the metrics export carries the value column, and an unknown rate is a dash", async ({
  page,
}) => {
  await stub(page);
  await page.route("**/api/proxy/workforce-intelligence/metrics**", (route) =>
    route.fulfill({
      json: {
        period: { from: "2026-01-01", to: "2026-06-30" },
        definitions: {
          headcount: "h",
          starters: "s",
          leavers: "l",
          turnover_rate: "t",
          span_of_control: "p",
          time_to_fill: "f",
        },
        headcount: { opening: 40, closing: 44, opening_date: "2026-01-01" },
        starters: 7,
        leavers: 3,
        turnover_rate: null,
        span_of_control: null,
        time_to_fill: null,
      },
    }),
  );
  await page.route("**/api/proxy/workforce-intelligence/insights**", (route) =>
    route.fulfill({ json: { insights: [] } }),
  );
  await page.goto("/metrics");
  await expect(page.getByTestId("metrics-turnover")).toBeVisible();
  const { name, text } = await downloadText(page, "TSV");
  expect(name).toBe("metrics.tsv");
  expect(text).toContain("40 → 44");
  expect(text).toMatch(/\t7\t/);
  // Turnover and span are unknown: a dash, never a 0 and never an empty cell.
  expect(text).toMatch(/\t—\t/);
  expect(text).not.toMatch(/\t0\t/);
});

test("the pay-band export carries each amount with its currency", async ({
  page,
}) => {
  await stub(page);
  await page.route("**/api/proxy/benchmarks", (route) =>
    route.fulfill({
      json: [
        {
          pid: "aaaaaaaa-0000-4000-8000-0000000000b1",
          job_title: "Analyst",
          currency: "GBP",
          min_minor: 3000000,
          median_minor: 3600000,
          max_minor: 4200000,
          source: "s",
          as_of: "2026-01-01",
        },
      ],
    }),
  );
  await page.route("**/api/proxy/benchmarks/comparison**", (route) =>
    route.fulfill({ json: { organization: "x", rows: [] } }),
  );
  await page.goto("/benchmarks");
  await expect(page.getByTestId("bands")).toContainText("Analyst");
  const { name, text } = await downloadText(page, "CSV");
  expect(name).toBe("pay-bands.csv");
  expect(text).toContain("£30,000.00");
  expect(text).toContain("£42,000.00");
  expect(text).not.toContain("3000000");
});
