// Workforce (`/workforce`): leave requests waiting for a decision, the shift rota, ergonomic issues and working-time
// flags. The API is stubbed per test; an unstubbed /api/proxy call 404s loudly.

import { expect, test } from "@playwright/test";
import { MASKED_WORKER, WORKER, stubSignedIn } from "./support";

const REQUEST = {
  pid: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
  worker_pid: WORKER.pid,
  kind: "annual",
  start_on: "2026-11-02",
  end_on: "2026-11-06",
  days: 5,
  status: "requested",
  negative_balance: false,
};
const SHIFT = {
  shift: {
    pid: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
    department: "support",
    starts_at: "2026-11-02T08:00:00Z",
    ends_at: "2026-11-02T16:00:00Z",
    required_headcount: 3,
  },
  assignments: [
    { pid: "cccccccc-cccc-4ccc-8ccc-cccccccccccc", worker_pid: WORKER.pid },
  ],
};

async function stubWorkforce(
  page: import("@playwright/test").Page,
  options: {
    requests?: unknown[];
    flagged?: unknown[];
    issues?: unknown[];
  } = {},
) {
  await page.route(
    `**/api/proxy/workers/${WORKER.pid}/leave-requests`,
    (route) => route.fulfill({ json: options.requests ?? [REQUEST] }),
  );
  await page.route(
    `**/api/proxy/workers/${MASKED_WORKER.pid}/leave-requests`,
    (route) => route.fulfill({ json: [] }),
  );
  await page.route("**/api/proxy/shifts**", (route) =>
    route.fulfill({ json: [SHIFT] }),
  );
  await page.route("**/api/proxy/workforce/working-time**", (route) =>
    route.fulfill({
      json: {
        as_of: "2026-10-24",
        reference_weeks: 17,
        rest_window_days: 7,
        workers_checked: 2,
        derivation: "Average weekly minutes over the reference weeks.",
        flagged: options.flagged ?? [],
      },
    }),
  );
  await page.route("**/api/proxy/ergonomics/issues", (route) =>
    route.fulfill({
      json: {
        as_of: "2026-10-24",
        by_department: {},
        issues: options.issues ?? [],
        derivation: "x",
      },
    }),
  );
}

test.describe("Workforce", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
  });

  test("a manager sees pending leave and the rota, and approves a request", async ({
    page,
  }) => {
    const decided: string[] = [];
    await stubWorkforce(page);
    await page.route(
      `**/api/proxy/leave-requests/${REQUEST.pid}/approve`,
      (route) => {
        decided.push("approve");
        return route.fulfill({ json: { ...REQUEST, status: "approved" } });
      },
    );
    await page.goto("/workforce");
    await expect(page).toHaveTitle("Workforce — WPM");
    const pending = page.getByTestId("pending-leave");
    await expect(pending).toContainText("Test Worker 001");
    await expect(pending).toContainText("5 calendar days");
    await expect(page.getByTestId("rota")).toContainText("support");
    await expect(page.getByTestId("rota")).toContainText("1 / 3");
    await expect(page.getByTestId("working-time-clear")).toBeVisible();
    await pending.getByRole("button", { name: "Approve" }).click();
    expect(decided).toEqual(["approve"]);
  });

  test("a refused decision is shown beside the table, in words", async ({
    page,
  }) => {
    await stubWorkforce(page);
    await page.route(
      `**/api/proxy/leave-requests/${REQUEST.pid}/reject`,
      (route) =>
        route.fulfill({
          status: 422,
          json: { description: "already decided" },
        }),
    );
    await page.goto("/workforce");
    await page
      .getByTestId("pending-leave")
      .getByRole("button", { name: "Reject" })
      .click();
    // The catalogue's words, not the service's.
    await expect(page.getByTestId("action-error")).toContainText(
      "That was not accepted",
    );
    await expect(page.getByTestId("action-error")).not.toContainText(
      "already decided",
    );
  });

  test("working-time flags and ergonomic issues appear only when there are some", async ({
    page,
  }) => {
    await stubWorkforce(page, {
      flagged: [
        {
          worker_pid: WORKER.pid,
          display_name: "Test Worker 001",
          department: "engineering",
          average_weekly: {
            numerator_minutes: 3000,
            denominator_weeks: 1,
            value_minutes_per_week: 3000,
          },
          over_48h: true,
          rest_breaches: [],
        },
      ],
      issues: [
        {
          department: "engineering",
          worker_pid: WORKER.pid,
          display_name: "Test Worker 001",
          workstation: "Desk 4",
          item: "monitor_height",
          note: null,
          assessment_status: "completed",
        },
      ],
    });
    await page.goto("/workforce");
    await expect(page.getByTestId("working-time")).toContainText(
      "Test Worker 001",
    );
    await expect(page.getByTestId("working-time-clear")).toHaveCount(0);
    await expect(page.getByTestId("ergonomic-issues")).toContainText("Desk 4");
  });

  test("one failed read is the page's error and nothing is drawn as empty", async ({
    page,
  }) => {
    await stubWorkforce(page);
    await page.route("**/api/proxy/shifts**", (route) =>
      route.fulfill({ status: 500, json: { description: "boom" } }),
    );
    await page.goto("/workforce");
    await expect(page.getByTestId("error")).toContainText(
      "Something went wrong on our side",
    );
    await expect(page.getByTestId("pending-leave")).toHaveCount(0);
  });
});
