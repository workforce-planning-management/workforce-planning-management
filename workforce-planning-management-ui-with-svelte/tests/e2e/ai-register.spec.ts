// The AI and scoring register (`/ai-register`). The API is stubbed per test; an unstubbed /api/proxy call 404s loudly.

import { expect, test } from "@playwright/test";
import { stubSignedIn } from "./support";

const unknown = { status: "unknown" };
const recorded = (value: unknown) => ({ status: "recorded", value });
const feature = (id: string, name: string) => ({
  id,
  name,
  computed_by: "WPM",
  informs: "a hiring decision",
  made_of: "declared skills",
});
const row = (
  id: string,
  name: string,
  extra: Record<string, unknown> = {},
) => ({
  feature: feature(id, name),
  advisory: "Advisory. A person decides.",
  annex_category: unknown,
  overseer_role: unknown,
  representatives_informed_on: unknown,
  workers_informed_on: unknown,
  log_retention_months: unknown,
  retention_note: unknown,
  retention_below_minimum: false,
  ...extra,
});

test.describe("AI register", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
  });

  test("shows what is recorded and says unknown where nothing is, flags a short retention, and records a feature", async ({
    page,
  }) => {
    let put: Record<string, unknown> | null = null;
    let features = [
      row("S1", "Skill match score"),
      row("S2", "Readiness", {
        annex_category: recorded("recruitment_selection"),
        overseer_role: recorded("Head of HR"),
        log_retention_months: recorded(3),
        retention_note: recorded("Data protection law"),
        retention_below_minimum: true,
      }),
    ];
    await page.route("**/api/proxy/ai-register", (route) =>
      route.fulfill({
        json: {
          applies_from: "2027-12-02",
          applies_now: false,
          features,
          minimum_log_months: 6,
          note: "This register records what the deployer says.",
        },
      }),
    );
    await page.route("**/api/proxy/ai-register/S1", (route) => {
      put = route.request().postDataJSON();
      features = [
        row("S1", "Skill match score", {
          annex_category: recorded(put?.annex_category),
          overseer_role: recorded(put?.overseer_role),
        }),
        features[1]!,
      ];
      return route.fulfill({
        json: {
          pid: "p",
          feature: "S1",
          decision_gate: {
            change: "new_or_changed_score",
            standing: { kind: "reference_needed", bodies: 2 },
            consult: [],
            equality_impact_template: "section 11",
            note: "advice",
          },
        },
      });
    });
    await page.goto("/ai-register");
    await expect(page.getByTestId("ai-applies")).toContainText(
      "The Annex III duties apply from 2 Dec 2027",
    );
    const cards = page.getByTestId("ai-feature");
    await expect(cards).toHaveCount(2);
    await expect(cards.nth(0).getByTestId("ai-unknown")).toHaveCount(6); // nothing recorded is unknown, never "none"
    await expect(cards.nth(1)).toContainText("Head of HR");
    await expect(cards.nth(1)).toContainText("3 months");
    await expect(cards.nth(1).getByTestId("ai-below-minimum")).toContainText(
      "Under the 6 months",
    );
    await cards.nth(0).getByTestId("ai-edit").click();
    await page.getByTestId("ai-category").selectOption("task_allocation");
    await page.getByTestId("ai-overseer").fill("Head of Mobility");
    await page.getByTestId("ai-save").click();
    await expect(cards.nth(0)).toContainText("Head of Mobility");
    expect(put).toEqual({
      annex_category: "task_allocation",
      overseer_role: "Head of Mobility",
    });
    await expect(page.getByTestId("ai-gate")).toContainText(
      "2 body or bodies to consult",
    );
    await expect(page.getByTestId("ai-gate")).toContainText(
      "Advice, not a block.",
    );
  });

  test("a refusal to record is shown in the service's words", async ({
    page,
  }) => {
    await page.route("**/api/proxy/ai-register", (route) =>
      route.fulfill({
        json: {
          applies_from: "2027-12-02",
          applies_now: false,
          features: [row("S1", "Skill match score")],
          minimum_log_months: 6,
          note: "n",
        },
      }),
    );
    await page.route("**/api/proxy/ai-register/S1", (route) =>
      route.fulfill({ status: 403, json: { error: "forbidden" } }),
    );
    await page.goto("/ai-register");
    await page.getByTestId("ai-edit").click();
    await page.getByTestId("ai-overseer").fill("Head of HR");
    await page.getByTestId("ai-save").click();
    await expect(page.getByTestId("error")).toContainText(
      "You are not allowed to do this.",
    );
  });
});
