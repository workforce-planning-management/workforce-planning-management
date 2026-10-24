// The organization now and as planned (`/org-chart/shape`): layers, span of control and each role added, removed, moved or resized.
// It names no one; a count under the floor is withheld. The API is stubbed per test.

import { expect, test } from "@playwright/test";
import { WORKER, stubSignedIn } from "./support";

const PLAN = {
  pid: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
  name: "Flatten",
  organization_ref: WORKER.organization_ref,
  horizon_start: "2026-01-01",
  horizon_end: "2027-12-31",
  status: "draft",
  attrition_bp: null,
  demand_lines: 0,
};

const role = (title: string, extra: Record<string, unknown>) => ({
  title,
  current: { status: "shown", value: 0 },
  target: 0,
  change: "same",
  delta: 0,
  moved: false,
  current_departments: [],
  target_departments: [],
  ...extra,
});

const DIFF = {
  plan: PLAN.pid,
  target_set: true,
  small_group_floor: 10,
  diff: {
    current: {
      layers: 3,
      headcount: 13,
      span: { managers: 3, mean_hundredths: 400, max: 5 },
    },
    target: {
      layers: 3,
      headcount: 15,
      span: { managers: 2, mean_hundredths: 700, max: 12 },
    },
    roles: [
      role("Engineer", {
        current: { status: "shown", value: 10 },
        target: 12,
        change: "larger",
        delta: 2,
      }),
      role("Head", {
        current: { status: "withheld" },
        target: 1,
        change: "same",
        delta: null,
        moved: true,
        current_departments: ["exec"],
        target_departments: ["leadership"],
      }),
      role("Product Lead", { target: 1, change: "added", delta: 1 }),
    ],
  },
  derivation: "",
};

test.describe("The organization now and as planned", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
    await page.route("**/api/proxy/workforce-plans", (route) =>
      route.fulfill({ json: [PLAN] }),
    );
  });

  test("compares layers, span and roles, and withholds a small count", async ({
    page,
  }) => {
    await page.route(
      `**/api/proxy/workforce-plans/${PLAN.pid}/organization-diff`,
      (route) => route.fulfill({ json: DIFF }),
    );
    await page.goto("/org-chart/shape");
    const sides = page.getByTestId("shape-sides");
    await expect(sides).toContainText("Layers");
    await expect(sides).toContainText("4.00");
    await expect(sides).toContainText("7.00");
    const roles = page.getByTestId("shape-roles");
    await expect(roles).toContainText("Engineer");
    await expect(roles).toContainText("larger");
    await expect(roles).toContainText("added");
    await expect(roles).toContainText("withheld");
    await expect(roles).toContainText("moved from exec to leadership");
    await expect(
      page.getByText("A count of fewer than 10 people is withheld"),
    ).toBeVisible();
  });

  test("says when the plan has no target yet", async ({ page }) => {
    await page.route(
      `**/api/proxy/workforce-plans/${PLAN.pid}/organization-diff`,
      (route) =>
        route.fulfill({
          json: { plan: PLAN.pid, target_set: false, derivation: "" },
        }),
    );
    await page.goto("/org-chart/shape");
    await expect(page.getByTestId("shape-no-target")).toContainText(
      "no target organization",
    );
  });

  test("with no plans it says so", async ({
    page,
  }) => {
    await page.route("**/api/proxy/workforce-plans", (route) =>
      route.fulfill({ json: [] }),
    );
    await page.goto("/org-chart/shape");
    await expect(page.getByTestId("shape-none")).toContainText("no plans");
  });

  test("the org chart links to it", async ({ page }) => {
    await page.route("**/api/proxy/**", (route) => route.fulfill({ json: [] }));
    await page.goto("/org-chart");
    await page.getByRole("link", { name: "Compare with a plan" }).click();
    await expect(page).toHaveURL(/\/org-chart\/shape$/);
    await expect(page.getByRole("heading", { level: 1 })).toContainText("The organization now and as planned");
  });

  test("a failed comparison shows the error in words", async ({ page }) => {
    await page.route(
      `**/api/proxy/workforce-plans/${PLAN.pid}/organization-diff`,
      (route) =>
        route.fulfill({
          status: 500,
          json: { description: "relation does not exist" },
        }),
    );
    await page.goto("/org-chart/shape");
    await expect(page.getByTestId("error")).toContainText(
      "Something went wrong on our side",
    );
    await expect(page.getByTestId("error")).not.toContainText("relation");
  });
});
