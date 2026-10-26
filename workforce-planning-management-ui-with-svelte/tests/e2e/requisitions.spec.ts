// Requisition board. The API is stubbed per test; an unstubbed /api/proxy call 404s loudly.

import { expect, test } from "@playwright/test";
import { WORKER, stubSignedIn } from "./support";

test.describe("Requisition board", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
  });

  test("requisition board renders its columns and cards", async ({ page }) => {
    await page.route(/\/api\/proxy\/requisitions\?limit=500$/, (route) =>
      route.fulfill({
        json: [
          {
            pid: "99999999-9999-4999-8999-999999999999",
            organization_ref: WORKER.organization_ref,
            department: "engineering",
            job_title: "Platform Engineer",
            headcount: 2,
            salary_min_minor: 3000000,
            salary_max_minor: 5000000,
            salary_currency: "GBP",
            status: "interviewing",
            opened_on: "2026-06-01",
          },
        ],
      }),
    );
    await page.goto("/requisitions");
    const board = page.getByTestId("requisition-board");
    await expect(board).toContainText("interviewing");
    await expect(board).toContainText("Platform Engineer");
    await expect(board).toContainText("£30,000.00");
  });

  test("a long list says how many there are and loads the next page on request", async ({ page }) => {
    const row = (n: number) => ({
      pid: `00000000-0000-4000-8000-${String(n).padStart(12, "0")}`,
      organization_ref: WORKER.organization_ref,
      department: "engineering",
      job_title: `Role ${n}`,
      headcount: 1,
      salary_min_minor: null,
      salary_max_minor: null,
      salary_currency: null,
      status: "open",
      opened_on: "2026-06-01",
    });
    const seen: string[] = [];
    await page.route(/\/api\/proxy\/requisitions\?/, (route) => {
      const url = new URL(route.request().url());
      seen.push(url.search);
      const offset = Number(url.searchParams.get("offset") ?? 0);
      const rows = offset === 0 ? Array.from({ length: 500 }, (_, i) => row(i + 1)) : [row(501)];
      return route.fulfill({
        json: rows,
        headers: { "x-total-count": "501", "x-limit": "500", "x-offset": String(offset) },
      });
    });
    await page.goto("/requisitions");
    await expect(page.getByTestId("load-more-count")).toHaveText("Showing 500 of 501");
    await page.getByTestId("load-more-button").click();
    await expect(page.getByTestId("requisition-board")).toContainText("Role 501");
    await expect(page.getByTestId("load-more")).toHaveCount(0);
    expect(seen).toEqual(["?limit=500", "?limit=500&offset=500"]);
  });

  test("a short list shows no paging control", async ({ page }) => {
    await page.route(/\/api\/proxy\/requisitions\?/, (route) =>
      route.fulfill({ json: [], headers: { "x-total-count": "0", "x-limit": "500", "x-offset": "0" } }),
    );
    await page.goto("/requisitions");
    await expect(page.getByTestId("requisition-board")).toBeVisible();
    await expect(page.getByTestId("load-more")).toHaveCount(0);
  });
});
