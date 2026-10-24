// Requisition board. The API is stubbed per test; an unstubbed /api/proxy call 404s loudly.

import { expect, test } from "@playwright/test";
import { WORKER, stubSignedIn } from "./support";

test.describe("Requisition board", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
  });

  test("requisition board renders its columns and cards", async ({ page }) => {
    await page.route("**/api/proxy/requisitions", (route) =>
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
});
