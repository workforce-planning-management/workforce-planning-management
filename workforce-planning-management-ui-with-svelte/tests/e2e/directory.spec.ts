// Directory. The API is stubbed per test; an unstubbed /api/proxy call 404s loudly.

import { expect, test } from "@playwright/test";
import { WORKER, stubSignedIn } from "./support";

test.describe("Directory", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
  });

  test("directory searches employed workers and shows no pay", async ({
    page,
  }) => {
    const rows = [
      {
        pid: WORKER.pid,
        display_name: WORKER.display_name,
        job_title: WORKER.job_title,
        department: WORKER.department,
        location: "Leeds",
        organization_ref: WORKER.organization_ref,
        manager_name: null,
        away_today: true,
        covered_by: "Kim Lee",
        on_call: ["Platform on-call"],
      },
    ];
    const seen: string[] = [];
    await page.route(
      (url) => url.pathname === "/api/proxy/directory",
      (route) => {
        seen.push(new URL(route.request().url()).searchParams.get("q") ?? "");
        return route.fulfill({
          json: seen.at(-1) === "nobody" ? [] : rows,
        });
      },
    );
    await page.goto("/directory");
    const table = page.getByTestId("directory-table");
    await expect(table).toContainText(WORKER.display_name);
    await expect(table).toContainText("Leeds");
    await expect(table).not.toContainText("£");
    await expect(page.getByTestId("on-call")).toHaveText(
      "On call: Platform on-call",
    );
    await expect(page.getByTestId("away")).toHaveText("Away today");
    await expect(page.getByTestId("covered-by")).toContainText(
      "Covered by Kim Lee",
    );
    await page.getByTestId("directory-search").fill("nobody");
    await expect(page.getByTestId("directory-none")).toBeVisible();
    expect(seen).toContain("nobody");
  });

  test("directory columns sort on click, unknown values last and shown as a dash", async ({
    page,
  }) => {
    const row = (pid: string, name: string, manager: string | null) => ({
      pid,
      display_name: name,
      job_title: "Engineer",
      department: "Platform",
      location: null,
      organization_ref: WORKER.organization_ref,
      manager_name: manager,
      away_today: false,
      covered_by: null,
      on_call: [],
    });
    await page.route(
      (url) => url.pathname === "/api/proxy/directory",
      (route) =>
        route.fulfill({
          json: [
            row("p1", "Zed", null),
            row("p2", "Amy", "Bo"),
            row("p3", "Cy", "Al"),
          ],
        }),
    );
    await page.goto("/directory");
    const names = page.locator(
      '[data-testid="directory-table"] tbody tr td:first-child',
    );
    await expect(names).toHaveText(["Zed", "Amy", "Cy"]);
    await page.getByRole("button", { name: "Name" }).click();
    await expect(names).toHaveText(["Amy", "Cy", "Zed"]);
    await page.getByRole("button", { name: "Manager" }).click();
    await expect(names).toHaveText(["Cy", "Amy", "Zed"]);
    await expect(page.getByTestId("directory-table")).toContainText("—");
  });
});
