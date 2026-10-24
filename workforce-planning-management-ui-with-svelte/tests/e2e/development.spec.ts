// Development (`/development`): succession gaps, succession plans with ranked candidates, and certificates
// about to expire. The API is stubbed per test; an unstubbed /api/proxy call 404s loudly.

import { expect, test } from "@playwright/test";
import { WORKER, stubSignedIn } from "./support";

const PLAN = {
  pid: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
  role_title: "Head of Platform",
  department: "engineering",
  criticality: 5,
};

test.describe("Development", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
  });

  test("shows the succession gaps, the plans with their candidates and what certifies out soon", async ({
    page,
  }) => {
    await page.route("**/api/proxy/succession-plans/gaps", (route) =>
      route.fulfill({ json: { gaps: [PLAN] } }),
    );
    await page.route("**/api/proxy/succession-plans", (route) =>
      route.fulfill({
        json: [
          {
            plan: PLAN,
            candidates: [{ pid: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb", worker_pid: WORKER.pid, readiness: "ready_now", rank: 1 }],
          },
        ],
      }),
    );
    await page.route("**/api/proxy/training/expiring?within_days=90", (route) =>
      route.fulfill({
        json: {
          as_of: "2026-10-24",
          horizon: "2027-01-22",
          expiring: [
            {
              pid: "cccccccc-cccc-4ccc-8ccc-cccccccccccc",
              worker_pid: WORKER.pid,
              course_ref: "course:fire-safety",
              status: "completed",
              certificate_expires_on: "2026-12-01",
            },
          ],
        },
      }),
    );
    await page.goto("/development");
    await expect(page).toHaveTitle("Development — WPM");
    await expect(page.getByTestId("gaps")).toContainText("Head of Platform");
    await expect(page.getByTestId("succession")).toContainText("Head of Platform");
    await expect(page.getByTestId("succession")).toContainText("ready_now");
    await expect(page.getByTestId("expiring")).toContainText("course:fire-safety");
    await expect(page.getByTestId("expiring")).toContainText("1 Dec 2026");
  });

  test("with nothing to report the gap list shows a dash, not a zero", async ({ page }) => {
    await page.route("**/api/proxy/succession-plans/gaps", (route) => route.fulfill({ json: { gaps: [] } }));
    await page.route("**/api/proxy/succession-plans", (route) => route.fulfill({ json: [] }));
    await page.route("**/api/proxy/training/expiring?within_days=90", (route) =>
      route.fulfill({ json: { as_of: "2026-10-24", horizon: "2027-01-22", expiring: [] } }),
    );
    await page.goto("/development");
    await expect(page.getByTestId("gaps")).toContainText("—");
  });

  test("a failed read is said in words, and the page does not pretend to be empty", async ({ page }) => {
    await page.route("**/api/proxy/succession-plans/gaps", (route) =>
      route.fulfill({ status: 403, json: { description: "only HR reads succession" } }),
    );
    await page.route("**/api/proxy/succession-plans", (route) => route.fulfill({ json: [] }));
    await page.route("**/api/proxy/training/expiring?within_days=90", (route) =>
      route.fulfill({ json: { as_of: "x", horizon: "x", expiring: [] } }),
    );
    await page.goto("/development");
    await expect(page.getByTestId("error")).toContainText("You are not allowed to do this.");
    await expect(page.getByTestId("error")).toContainText("only HR reads succession");
    await expect(page.getByTestId("gaps")).toHaveCount(0);
  });
});
