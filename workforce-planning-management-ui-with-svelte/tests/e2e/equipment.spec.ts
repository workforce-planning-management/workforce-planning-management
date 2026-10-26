// Equipment on a worker's page. The API is stubbed per test; an unstubbed /api/proxy call 404s loudly.

import { expect, test } from "@playwright/test";
import { WORKER, stubSignedIn } from "./support";

test.describe("Equipment", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
  });

  test("HR gives a laptop, then writes it off with a reason", async ({
    page,
  }) => {
    let issued = false;
    let closed: Record<string, unknown> | null = null;
    const issues = () =>
      issued
        ? [
            {
              issue_pid: "i1",
              asset_pid: "a1",
              kind: "laptop",
              tag: "LT-1",
              issued_on: "2026-10-25",
              returned_on: closed ? "2026-10-25" : null,
              outcome: closed ? "written_off" : null,
              note: closed ? "lost" : null,
            },
          ]
        : [];
    await page.route(`**/api/proxy/workers/${WORKER.pid}/equipment`, (route) =>
      route.fulfill({
        json: {
          worker_pid: WORKER.pid,
          out: issued && !closed ? 1 : 0,
          issues: issues(),
        },
      }),
    );
    await page.route("**/api/proxy/assets?*", (route) =>
      route.fulfill({
        json: {
          assets: [
            {
              pid: "a1",
              organization_ref: "org-1",
              kind: "laptop",
              tag: "LT-1",
              owner_role: "IT",
              out: issued && !closed,
            },
          ],
        },
      }),
    );
    await page.route("**/api/proxy/assets/a1/issue", (route) => {
      issued = true;
      return route.fulfill({ json: { pid: "i1" } });
    });
    await page.route("**/api/proxy/asset-issues/i1/return", (route) => {
      closed = route.request().postDataJSON();
      return route.fulfill({ json: { pid: "i1", outcome: "written_off" } });
    });

    await page.goto("/me");
    const panel = page.getByTestId("equipment");
    await expect(panel.getByTestId("equipment-none")).toBeVisible();
    await panel.getByTestId("equipment-pick").selectOption("a1");
    await panel.getByTestId("equipment-issue").click();
    await expect(panel.getByTestId("equipment-out")).toContainText(
      "Still held: 1",
    );
    await panel.getByTestId("equipment-close").click();
    await expect(panel.getByTestId("equipment-write-off")).toBeDisabled();
    await panel.getByTestId("equipment-note").fill("lost");
    await panel.getByTestId("equipment-write-off").click();
    await expect(panel.getByTestId("equipment-out")).toContainText(
      "Still held: 0",
    );
    expect(closed).toEqual({ outcome: "written_off", note: "lost" });
  });

  test("a worker without HR access sees the list and no controls", async ({
    page,
  }) => {
    await page.route(`**/api/proxy/workers/${WORKER.pid}/equipment`, (route) =>
      route.fulfill({ json: { worker_pid: WORKER.pid, out: 0, issues: [] } }),
    );
    await page.route("**/api/proxy/assets?*", (route) =>
      route.fulfill({ status: 403, json: { error: "forbidden" } }),
    );
    await page.goto("/me");
    const panel = page.getByTestId("equipment");
    await expect(panel.getByTestId("equipment-none")).toBeVisible();
    await expect(panel.getByTestId("equipment-issue")).toHaveCount(0);
  });

  test("a manager sees how many of their team hold equipment, and nothing under the floor", async ({
    page,
  }) => {
    let answer: Record<string, unknown> = {
      kind: "shown",
      holding: 3,
      none: 7,
    };
    await page.route(
      `**/api/proxy/workers/${WORKER.pid}/team-equipment-count`,
      (route) =>
        route.fulfill({
          json: { manager_pid: WORKER.pid, floor: 10, count: answer },
        }),
    );
    await page.goto("/me");
    await expect(page.getByTestId("team-equipment-count")).toHaveText(
      "3 hold equipment, 7 hold none",
    );
    await expect(page.getByTestId("team-equipment")).not.toContainText(
      "laptop",
    );

    answer = { kind: "withheld" };
    await page.goto("/me");
    await expect(page.getByTestId("team-equipment-count")).toHaveText(
      "Fewer than 10 people: no count is shown",
    );

    answer = { kind: "no_one" };
    await page.goto("/me");
    await expect(page.getByTestId("team-equipment")).toHaveCount(0); // no team, no panel
  });
});
