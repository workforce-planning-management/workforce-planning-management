// The bargaining, consultation and decision-gate pages pass axe's WCAG 2 A and AA rules, with data on screen. The API is stubbed.

import AxeBuilder from "@axe-core/playwright";
import { expect, test, type Page } from "@playwright/test";
import { stubSignedIn } from "./support";

const body = {
  pid: "b1",
  organization_ref: "o",
  name: "Care Union",
  kind: "union",
  basis: "other",
  recognised_on: "2020-03-01",
  contact_role: "HR lead",
  units: 1,
};
const cell = { kind: "shown", count: 12 };
const consultation = {
  pid: "c1",
  organization_ref: "o",
  body_pid: "b1",
  plan_pid: null,
  establishment: "Leeds",
  profile: "great_britain",
  establishment_size: 200,
  dismissals: 25,
  earliest_termination_on: "2027-03-01",
  topics: ["redundancy"],
  document_ref: null,
  started_on: null,
  ended_on: null,
  outcome: "open",
  agreement_pid: null,
};

async function violations(page: Page) {
  const result = await new AxeBuilder({ page })
    .withTags(["wcag2a", "wcag2aa"])
    .analyze();
  return result.violations.map(
    (v) => `${v.id}: ${v.nodes.map((n) => n.html).join(" | ")}`,
  );
}

test.describe("bargaining pages and axe", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
    await page.route("**/api/proxy/recognised-bodies?*", (route) =>
      route.fulfill({ json: { bodies: [body] } }),
    );
  });

  test("/bargaining with a unit, its pack and facility time", async ({
    page,
  }) => {
    await page.route("**/api/proxy/recognised-bodies/b1/units", (route) =>
      route.fulfill({
        json: {
          body_pid: "b1",
          floor: 5,
          derivation: "r",
          units: [
            {
              pid: "u1",
              name: "Care",
              department: "Care",
              location: null,
              employment_type: null,
              job_title: null,
              coverage: { kind: "shown", covered: 12 },
            },
          ],
        },
      }),
    );
    await page.route(
      "**/api/proxy/recognised-bodies/b1/facility-time",
      (route) =>
        route.fulfill({
          json: {
            body_pid: "b1",
            periods: [
              {
                pid: "f",
                period_start: "2026-01-01",
                period_end: "2026-03-31",
                minutes: 700,
                hours: "11.7",
                budget_minutes: 600,
                over_budget: true,
              },
            ],
            total_minutes: 700,
          },
        }),
    );
    await page.route(
      "**/api/proxy/bargaining-units/u1/information-pack",
      (route) =>
        route.fulfill({
          json: {
            unit_pid: "u1",
            floor: 5,
            derivation: "d",
            pack: {
              as_of: "2026-10-27",
              headcount: 12,
              by_department: [
                { group: "Care", headcount: cell, fte_hundredths: 1 },
              ],
              by_employment_type: [],
              by_pay_band: [],
              starters_last_year: { kind: "withheld" },
              leavers_last_year: cell,
              contingent: [],
            },
            vacancies: { kind: "shown", open_posts: 1 },
          },
        }),
    );
    await page.goto("/bargaining");
    await page.getByTestId("bg-show-units").click();
    await page.getByTestId("bg-pack").click();
    await expect(page.getByTestId("bg-pack-result")).toBeVisible();
    expect(await violations(page)).toEqual([]);
  });

  test("/consultations with a record and its timeline", async ({ page }) => {
    await page.route("**/api/proxy/consultations?*", (route) =>
      route.fulfill({ json: { consultations: [consultation] } }),
    );
    await page.route("**/api/proxy/consultations/c1/timeline", (route) =>
      route.fulfill({
        json: {
          consultation,
          assessment: {
            profile: "great_britain",
            source: "s",
            may_apply: true,
            evidence: { counted: 25, needed: 20, window_days: 90 },
            consultation_days: 30,
            latest_consultation_start: "2027-01-30",
            latest_authority_notice: "2027-01-30",
            days_remaining: 100,
            already_late: false,
            checklist: ["notify_the_authority"],
            agreed_period_days: null,
            deployer_to_confirm: "x",
          },
          timeliness: { kind: "not_begun", days_left: 100 },
          derivation: "d",
        },
      }),
    );
    await page.goto("/consultations");
    await page.getByTestId("cs-timeline").click();
    await expect(page.getByTestId("cs-assessment")).toBeVisible();
    expect(await violations(page)).toEqual([]);
  });

  test("/decision-gates with a gate and a check", async ({ page }) => {
    await page.route("**/api/proxy/decision-gates?*", (route) =>
      route.fulfill({
        json: {
          changes: ["other"],
          gates: [
            { pid: "g", change_kind: "other", body_pid: "b1", note: "n" },
          ],
        },
      }),
    );
    await page.route("**/api/proxy/decision-gates/check?*", (route) =>
      route.fulfill({
        json: {
          change: "other",
          standing: { kind: "reference_given" },
          consult: [],
          equality_impact_template: null,
          note: "n",
        },
      }),
    );
    await page.goto("/decision-gates");
    await page.getByTestId("dg-check").click();
    await expect(page.getByTestId("dg-result")).toBeVisible();
    expect(await violations(page)).toEqual([]);
  });
});
