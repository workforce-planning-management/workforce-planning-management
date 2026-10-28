// Decision gates (`/decision-gates`). The API is stubbed per test; an unstubbed /api/proxy call 404s loudly.

import { expect, test } from "@playwright/test";
import { stubSignedIn } from "./support";

const body = {
  pid: "b1",
  organization_ref: "o",
  name: "Care Union",
  kind: "union",
  basis: "other",
  recognised_on: "2020-03-01",
  contact_role: "HR lead",
  units: 0,
};

test.describe("Decision gates", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
  });

  test("sets a gate and checks a change: it advises and says what it needs", async ({
    page,
  }) => {
    let gates: unknown[] = [];
    let posted: Record<string, unknown> | null = null;
    await page.route("**/api/proxy/recognised-bodies?*", (route) =>
      route.fulfill({ json: { bodies: [body] } }),
    );
    await page.route("**/api/proxy/decision-gates?*", (route) =>
      route.fulfill({
        json: {
          changes: ["redundancy_proposal", "new_or_changed_score"],
          gates,
        },
      }),
    );
    await page.route("**/api/proxy/decision-gates", (route) => {
      posted = route.request().postDataJSON();
      gates = [
        {
          pid: "g1",
          change_kind: "new_or_changed_score",
          body_pid: "b1",
          note: null,
        },
      ];
      return route.fulfill({ json: { pid: "g1" } });
    });
    await page.route("**/api/proxy/decision-gates/check?*", (route) =>
      route.fulfill({
        json: {
          change: "new_or_changed_score",
          standing: { kind: "reference_needed", bodies: 1 },
          consult: [],
          equality_impact_template:
            "spec/equality-impact/template.md, section 11",
          note: "advice",
        },
      }),
    );
    await page.goto("/decision-gates");
    await expect(page.getByTestId("dg-none")).toBeVisible();
    await page.getByTestId("dg-change").selectOption("new_or_changed_score");
    await page.getByTestId("dg-add").click();
    await expect(page.getByTestId("dg-gate")).toContainText("Care Union");
    expect(posted).toEqual({
      organization_ref: expect.any(String),
      change_kind: "new_or_changed_score",
      body_pid: "b1",
    });
    await page.getByTestId("dg-check").click();
    await expect(page.getByTestId("dg-standing")).toContainText(
      "1 body or bodies to consult",
    );
    await expect(page.getByTestId("dg-result")).toContainText(
      "Advice, not a block.",
    );
    await expect(page.getByTestId("dg-result")).toContainText("section 11");
  });

  test("a refusal is shown in the service's words", async ({ page }) => {
    await page.route("**/api/proxy/recognised-bodies?*", (route) =>
      route.fulfill({ json: { bodies: [body] } }),
    );
    await page.route("**/api/proxy/decision-gates?*", (route) =>
      route.fulfill({ status: 403, json: { error: "forbidden" } }),
    );
    await page.goto("/decision-gates");
    await expect(page.getByTestId("error")).toContainText(
      "You are not allowed to do this.",
    );
  });
});
