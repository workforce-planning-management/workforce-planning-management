// Org chart and benchmarks. The API is stubbed per test; an unstubbed /api/proxy call 404s loudly.

import { expect, test } from "@playwright/test";
import { WORKER, stubSignedIn } from "./support";

test.describe("Org chart and benchmarks", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
  });

  test("org chart renders one section per organization membership, no switcher", async ({
    page,
  }) => {
    const secondOrg = "organization:44444444-4444-4444-8444-444444444444";
    await page.route(
      (url) =>
        url.pathname === "/api/proxy/org-chart" &&
        url.searchParams.get("organization") === WORKER.organization_ref,
      (route) =>
        route.fulfill({
          json: [
            {
              pid: WORKER.pid,
              display_name: WORKER.display_name,
              job_title: WORKER.job_title,
              department: WORKER.department,
              reports: [],
            },
          ],
        }),
    );
    await page.route(
      (url) =>
        url.pathname === "/api/proxy/org-chart" &&
        url.searchParams.get("organization") === secondOrg,
      (route) => route.fulfill({ json: [] }),
    );
    await page.goto("/org-chart");
    const sections = page.getByTestId("org-chart");
    await expect(sections).toHaveCount(2);
    await expect(sections.nth(0)).toContainText(WORKER.organization_ref);
    await expect(sections.nth(0)).toContainText(WORKER.display_name);
    await expect(sections.nth(1)).toContainText(secondOrg);
  });

  test("org confederation expands org chart to a descendant with no direct membership", async ({
    page,
  }) => {
    // A membership in a parent confederation reads as membership in
    // every descendant too — the backend's `scope` already includes
    // this third org even though it never appears in `/me/organizations`
    // (the dashboard's literal-grants list, unaffected by confederation).
    const confederatedChild =
      "organization:55555555-5555-4555-8555-555555555555";
    await page.route("**/api/proxy/me/organizations/scope", (route) =>
      route.fulfill({
        json: [
          WORKER.organization_ref,
          "organization:44444444-4444-4444-8444-444444444444",
          confederatedChild,
        ],
      }),
    );
    await page.route(
      (url) => url.pathname === "/api/proxy/org-chart",
      (route) => route.fulfill({ json: [] }),
    );
    await page.goto("/org-chart");
    await expect(page.getByTestId("org-chart")).toHaveCount(3);
    await expect(page.locator("body")).toContainText(confederatedChild);

    // The dashboard's membership list is unaffected: still exactly the
    // two literal grants, no confederation-only descendant.
    await page.goto("/");
    const membershipSection = page.getByTestId("my-organizations");
    await expect(membershipSection).not.toContainText(confederatedChild);
  });

  test("benchmark comparison renders one section per organization membership", async ({
    page,
  }) => {
    const secondOrg = "organization:44444444-4444-4444-8444-444444444444";
    await page.route("**/api/proxy/benchmarks", (route) =>
      route.fulfill({ json: [] }),
    );
    await page.route(
      (url) =>
        url.pathname === "/api/proxy/benchmarks/comparison" &&
        url.searchParams.get("organization") === WORKER.organization_ref,
      (route) =>
        route.fulfill({
          json: {
            organization: WORKER.organization_ref,
            rows: [
              {
                worker_pid: WORKER.pid,
                job_title: WORKER.job_title,
                department: WORKER.department,
                benchmark_pid: null,
                flag: null,
              },
            ],
          },
        }),
    );
    await page.route(
      (url) =>
        url.pathname === "/api/proxy/benchmarks/comparison" &&
        url.searchParams.get("organization") === secondOrg,
      (route) => route.fulfill({ json: { organization: secondOrg, rows: [] } }),
    );
    await page.goto("/benchmarks");
    const tables = page.getByTestId("comparison");
    await expect(tables).toHaveCount(2);
    await expect(page.locator("body")).toContainText(WORKER.organization_ref);
    await expect(page.locator("body")).toContainText(secondOrg);
    await expect(tables.nth(0)).toContainText(WORKER.pid.slice(0, 8));
  });
});
