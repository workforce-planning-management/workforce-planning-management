// Payroll (`/payroll`): the runs, each linking to its page. The run page itself is covered in
// `workers-and-payroll.spec.ts`. The API is stubbed per test; an unstubbed /api/proxy call 404s loudly.

import { expect, test } from "@playwright/test";
import { WORKER, stubSignedIn } from "./support";

const RUNS = [
  {
    pid: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
    organization_ref: WORKER.organization_ref,
    period_start: "2026-09-01",
    period_end: "2026-09-30",
    status: "paid",
  },
  {
    pid: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
    organization_ref: WORKER.organization_ref,
    period_start: "2026-10-01",
    period_end: "2026-10-31",
    status: "draft",
  },
];

test.describe("Payroll runs", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
  });

  test("lists the runs with their period and status, each linking to its page", async ({
    page,
  }) => {
    await page.route("**/api/proxy/payroll-runs?limit=500", (route) =>
      route.fulfill({ json: RUNS }),
    );
    await page.goto("/payroll");
    await expect(page).toHaveTitle("Payroll — WPM");
    const table = page.getByTestId("runs");
    // A period is one range in the page's locale (Intl's formatRange), not two dates with an arrow (WPM-T377).
    await expect(table).toContainText("1–30 Sept 2026");
    await expect(table).toContainText("1–31 Oct 2026");
    await expect(table).not.toContainText("→");
    const link = table.getByRole("link", { name: "1–31 Oct 2026" });
    await expect(link).toHaveAttribute(
      "href",
      `/en-001/payroll/${RUNS[1]?.pid}`,
    );
  });

  test("a caller who may not read payroll sees the error, not an empty list", async ({
    page,
  }) => {
    await page.route("**/api/proxy/payroll-runs?limit=500", (route) =>
      route.fulfill({
        status: 403,
        json: { description: "payroll is for HR" },
      }),
    );
    await page.goto("/payroll");
    // The catalogue's words, not the service's: "payroll is for HR" is never shown.
    await expect(page.getByTestId("error")).toContainText(
      "You are not allowed to do this.",
    );
    await expect(page.getByTestId("error")).not.toContainText(
      "payroll is for HR",
    );
    await expect(page.getByTestId("runs")).toHaveCount(0);
  });

  test("no runs yet is an empty table, not an error", async ({ page }) => {
    await page.route("**/api/proxy/payroll-runs?limit=500", (route) =>
      route.fulfill({ json: [] }),
    );
    await page.goto("/payroll");
    await expect(page.getByTestId("runs")).toBeVisible();
    await expect(page.getByTestId("error")).toHaveCount(0);
  });

  test("more runs than a page show a button, which follows the cursor, a failure of it keeping the first", async ({
    page,
  }) => {
    const run = (n: number) => ({
      ...RUNS[0],
      pid: `00000000-0000-4000-8000-${String(n).padStart(12, "0")}`,
      period_start: `2020-01-${String((n % 28) + 1).padStart(2, "0")}`,
    });
    let failNext = true;
    const seen: string[] = [];
    await page.route(/\/api\/proxy\/payroll-runs\?/, (route) => {
      const url = new URL(route.request().url());
      seen.push(url.search);
      if (!url.searchParams.has("cursor"))
        return route.fulfill({
          json: Array.from({ length: 500 }, (_, i) => run(i)),
          headers: {
            link: '</api/payroll-runs?limit=500&cursor=SIGNED>; rel="next"',
          },
        });
      if (failNext) {
        failNext = false;
        return route.fulfill({
          status: 500,
          json: { description: "try again" },
        });
      }
      return route.fulfill({ json: [run(500)] });
    });
    await page.goto("/payroll");
    await expect(page.getByTestId("load-more-button")).toBeVisible();
    // No total is made up: the service sends none.
    await expect(page.getByTestId("load-more")).not.toContainText("of");
    await page.getByTestId("load-more-button").click();
    await expect(page.getByTestId("more-error")).toBeVisible();
    await expect(page.getByTestId("runs")).toBeVisible();
    await expect(page.getByTestId("load-more-button")).toBeVisible();
    await page.getByTestId("load-more-button").click();
    await expect(page.getByTestId("load-more")).toHaveCount(0);
    expect(seen).toEqual([
      "?limit=500",
      "?limit=500&cursor=SIGNED",
      "?limit=500&cursor=SIGNED",
    ]);
  });
});

test.describe("dates in the page's locale (WPM-T375)", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
    await page.route("**/api/proxy/payroll-runs?limit=500", (route) =>
      route.fulfill({ json: RUNS }),
    );
  });

  test("a German page writes the period the German way", async ({ page }) => {
    await page.goto("/de-001/payroll");
    await expect(page.getByTestId("runs")).toContainText("01.–30.09.2026");
  });

  test("a Bengali page writes it in Bengali, in its own digits", async ({
    page,
  }) => {
    await page.goto("/bn-001/payroll");
    const table = page.getByTestId("runs");
    await expect(table).toContainText("২০২৬");
    await expect(table).not.toContainText("2026-09-01");
  });
});
