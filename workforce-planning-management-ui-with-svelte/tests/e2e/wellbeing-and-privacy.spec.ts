// Wellbeing and privacy. The API is stubbed per test; an unstubbed /api/proxy call 404s loudly.

import { expect, test } from "@playwright/test";
import { stubSignedIn } from "./support";

test.describe("Wellbeing and privacy", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
  });

  test("wellbeing area renders rules and aggregate-only uptake", async ({
    page,
  }) => {
    await page.route("**/api/proxy/wellbeing-entitlements", (route) =>
      route.fulfill({
        json: [
          {
            pid: "w1",
            name: "Seasonal flu vaccination",
            kind: "health",
            benefit_plan_pid: null,
            description: "Free NHS flu jab for frontline staff.",
            info_url: null,
            min_age: null,
            max_age: null,
            departments: ["engineering"],
            job_titles: [],
            doses: 2,
            active_from: null,
            active_until: null,
          },
          {
            pid: "w2",
            name: "Cycle-to-work scheme",
            kind: "benefit",
            benefit_plan_pid: "bp1",
            description: "Save on a bike through salary sacrifice.",
            info_url: null,
            min_age: null,
            max_age: null,
            departments: [],
            job_titles: [],
            doses: 1,
            active_from: null,
            active_until: null,
          },
        ],
      }),
    );
    await page.route("**/api/proxy/wellbeing/uptake", (route) =>
      route.fulfill({
        json: {
          as_of: "2026-07-24T00:00:00Z",
          derivation:
            "uptake = (booked + done) / all acknowledgements; counts only",
          entitlements: [
            {
              entitlement_pid: "w1",
              name: "Seasonal flu vaccination",
              kind: "health",
              by_response: { booked: 3, done: 1, declined: 1, dismissed: 0 },
              uptake_rate: { numerator: 4, denominator: 5, value: 0.8 },
              enrolment_conversion: null,
            },
            {
              entitlement_pid: "w2",
              name: "Cycle-to-work scheme",
              kind: "benefit",
              by_response: { booked: 0, done: 2, declined: 0, dismissed: 2 },
              uptake_rate: { numerator: 2, denominator: 4, value: 0.5 },
              enrolment_conversion: {
                numerator: 2,
                denominator: 4,
                value: 0.5,
              },
            },
          ],
        },
      }),
    );
    await page.route("**/api/proxy/pulse-surveys", (route) =>
      route.fulfill({
        json: [
          {
            pid: "s1",
            name: "July pulse",
            question: "How are you doing this week?",
            active_from: null,
            active_until: null,
            open: true,
          },
        ],
      }),
    );
    await page.route("**/api/proxy/pulse-surveys/s1/results", (route) =>
      route.fulfill({
        json: {
          as_of: "2026-07-25T00:00:00Z",
          survey: {
            pid: "s1",
            name: "July pulse",
            question: "How are you doing this week?",
          },
          overall: {
            suppressed: false,
            count: 6,
            distribution: [1, 1, 1, 1, 2],
            mean: 3.5,
          },
          departments: [
            {
              department: "engineering",
              suppressed: false,
              count: 5,
              distribution: [1, 1, 1, 1, 1],
              mean: 3.0,
            },
            { department: "finance", suppressed: true },
          ],
          derivation:
            "anonymous by construction; cells under 5 responses are suppressed",
        },
      }),
    );
    await page.goto("/wellbeing");
    await expect(
      page.getByTestId("wellbeing-rules").getByText("Seasonal flu vaccination"),
    ).toBeVisible();
    await expect(
      page.getByTestId("wellbeing-rules").getByText("engineering"),
    ).toBeVisible();
    await expect(
      page.getByTestId("wellbeing-rules").getByText("Cycle-to-work scheme"),
    ).toBeVisible();
    await expect(
      page.getByTestId("wellbeing-rules").getByText("Benefit"),
    ).toBeVisible();
    await expect(
      page.getByTestId("wellbeing-uptake").getByText("80% (4/5)"),
    ).toBeVisible();
    await expect(
      page.getByTestId("wellbeing-uptake").getByText(/Enrolled after prompt/),
    ).toBeVisible();
    await expect(
      page
        .getByTestId("wellbeing-uptake")
        .getByText(/50% \(2\/4\)/)
        .first(),
    ).toBeVisible();
    await expect(page.getByTestId("pulse-overall")).toContainText("3.5");
    await expect(
      page.getByTestId("pulse-results").getByText("Hidden below 5 responses"),
    ).toBeVisible();
  });

  test("privacy area renders the retention report and runs the sweep", async ({
    page,
  }) => {
    await page.route("**/api/proxy/retention", (route) =>
      route.fulfill({
        json: {
          as_of: "2026-07-25T00:00:00Z",
          horizon_days: 365,
          soft_deleted_past_horizon: { workers: 2, time_entries: 14 },
          expired_consent_candidates: 3,
          derivation:
            "soft-deleted rows older than the horizon are hard-deleted by the sweep",
        },
      }),
    );
    await page.route("**/api/proxy/retention/sweep", (route) =>
      route.fulfill({
        json: {
          horizon_days: 365,
          deleted: { workers: 2, time_entries: 14 },
          rows_deleted: 16,
          candidates_scrubbed: 3,
        },
      }),
    );
    await page.goto("/privacy");
    const report = page.getByTestId("retention-report");
    await expect(report.getByText("365")).toBeVisible();
    await expect(report.getByText("time_entries")).toBeVisible();
    await page.getByTestId("run-sweep").click();
    await expect(page.getByTestId("sweep-result")).toContainText(
      "16 rows deleted",
    );
  });
});
