// Learning economics (WPM-R227 to WPM-R230): a plan's fees beside the cost of its hours, a stated benefit, unknown that names
// what is missing, train or hire, and spend against a budget. The API is stubbed per test.

import { expect, test } from "@playwright/test";
import { stubPositions } from "./fixtures";
import { stubSignedIn, WORKER as PERSON } from "./support";

const PLAN = "22222222-2222-4222-8222-222222222222";
const WORKER = "33333333-3333-4333-8333-333333333333";

const COST = {
  plan_pid: PLAN,
  cost: {
    fees: { GBP: 50000 },
    courses_with_fee: 1,
    courses_without_fee: 1,
    hours: 30,
    estimated_hours: 20,
  },
  hours_cost: {
    status: "resolved",
    currency: "GBP",
    hours: 30,
    hourly_minor: 2500,
    minor: 75000,
    on_cost_bp: 2500,
    full_time_annual_hours: 1800,
  },
  derivation: "",
};

const figures = (verdict: string, npv: number) => ({
  rate_bp: 500,
  npv_minor: npv,
  investment_minor: 50000,
  benefit_minor: 200000,
  payback: { periods: 0, fraction_bp: 2500 },
  discounted_payback: null,
  verdict,
  absent: null,
});

test.describe("Learning economics", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
    await page.route("**/api/proxy/me/organizations", (route) =>
      route.fulfill({
        json: [
          {
            organization_ref: PERSON.organization_ref,
            role: "employee",
            worker_pid: WORKER,
          },
        ],
      }),
    );
    await page.route("**/api/proxy/workers/*/development-plans", (route) =>
      route.fulfill({
        json: {
          worker_pid: WORKER,
          plans: [
            {
              pid: PLAN,
              kind: "upskill",
              status: "active",
              target_job_title: "Staff engineer",
              items: [],
            },
          ],
        },
      }),
    );
    await page.route(`**/api/proxy/development-plans/${PLAN}/cost`, (route) =>
      route.fulfill({ json: COST }),
    );
  });

  test("keeps the fees and the hours apart and says what is unknown", async ({
    page,
  }) => {
    await page.route(
      `**/api/proxy/development-plans/${PLAN}/viability`,
      (route) =>
        route.fulfill({
          json: {
            plan_pid: PLAN,
            viability: {
              currency: null,
              rate_bp: 500,
              on_fees: null,
              with_hours: null,
              missing: ["benefit", "fee"],
            },
            benefits: [],
            derivation: "",
          },
        }),
    );
    await page.goto("/me/development");
    await expect(page.getByTestId("plan-fees")).toContainText("£500.00");
    await expect(page.getByTestId("plan-no-fee")).toContainText(": 1");
    await expect(page.getByTestId("plan-hours-cost")).toContainText("£750.00");
    await expect(page.getByTestId("plan-missing")).toContainText(
      "a stated benefit, a course fee",
    );
    await expect(page.getByTestId("plan-unmeasured").first()).toHaveText(
      "Unmeasured",
    );
    await expect(page.getByTestId("plan-verdict")).toHaveCount(0);
  });

  test("states a benefit and shows the verdicts", async ({ page }) => {
    let stated: unknown = null;
    await page.route(
      `**/api/proxy/development-plans/${PLAN}/viability`,
      (route) =>
        route.fulfill({
          json: {
            plan_pid: PLAN,
            viability: {
              currency: "GBP",
              rate_bp: 500,
              on_fees: figures("viable", 140476),
              with_hours: figures("not_viable", -5000),
              missing: [],
            },
            benefits: stated
              ? [
                  {
                    pid: "b1",
                    year: 1,
                    amount_minor: 200000,
                    currency: "GBP",
                    source: "Avoids a contract",
                    stated_by: null,
                    stated_on: "2026-11-05",
                  },
                ]
              : [],
            derivation: "",
          },
        }),
    );
    await page.route(
      `**/api/proxy/me/development-plans/${PLAN}/benefits`,
      (route) => {
        stated = route.request().postDataJSON();
        return route.fulfill({ json: { pid: "b1" } });
      },
    );
    await page.goto("/me/development");
    await page.getByTestId("benefit-amount").fill("200000");
    await page.getByTestId("benefit-currency").fill("gbp");
    await page.getByTestId("benefit-source").fill("Avoids a contract");
    await page.getByTestId("benefit-add").click();
    await expect(page.getByTestId("plan-benefits")).toContainText(
      "Avoids a contract",
    );
    expect(stated).toMatchObject({ amount_minor: 200000, currency: "GBP" });
    await expect(page.getByTestId("plan-verdict").first()).toHaveText("Viable");
    await expect(page.getByTestId("plan-verdict").nth(1)).toHaveText(
      "Not viable",
    );
  });

  test("train or hire names what is missing and shows both totals", async ({
    page,
  }) => {
    await stubPositions(page);
    await page.route(
      "**/api/proxy/positions/*/train-or-hire?candidate=*",
      (route) =>
        route.fulfill({
          json: {
            position_pid: "p",
            candidate_pid: PERSON.pid,
            currency: "GBP",
            salary_visible: false,
            answer: "unknown",
            missing: ["candidate_pay", "pay_withheld"],
            train: {
              total_minor: null,
              fees_minor: 100000,
              hours_cost_minor: null,
              ramp_up_minor: null,
            },
            hire: {
              total_minor: 2000000,
              recruitment_minor: 500000,
              vacancy_minor: 600000,
              ramp_up_minor: 900000,
              pay_difference_minor: null,
            },
          },
        }),
    );
    await page.goto("/positions");
    await page.getByTestId("toh-seat").selectOption({ index: 1 });
    await page.getByTestId("toh-candidate").selectOption(PERSON.pid);
    await page.getByTestId("toh-compare").click();
    await expect(page.getByTestId("toh-answer")).toHaveText("Unknown");
    await expect(page.getByTestId("toh-missing")).toContainText(
      "the candidate's pay",
    );
    await expect(page.getByTestId("toh-hire")).toContainText("£20,000.00");
    await expect(page.getByTestId("toh-train")).toContainText("—");
  });

  test("a course shows its fee, or that none is recorded", async ({ page }) => {
    const skill = "aaaaaaaa-0000-4000-8000-000000000001";
    await page.route("**/api/proxy/skills", (route) =>
      route.fulfill({
        json: [{ pid: skill, name: "Rust", category: "technical" }],
      }),
    );
    await page.route(`**/api/proxy/skills/${skill}/courses`, (route) =>
      route.fulfill({
        json: {
          skill_pid: skill,
          hours_per_level: null,
          default_hours_per_level: 30,
          courses: [
            {
              pid: "c1",
              course_ref: "A",
              title: "Priced",
              hours: 5,
              levels: 1,
              fee_minor: 30000,
              fee_currency: "GBP",
            },
            {
              pid: "c2",
              course_ref: "B",
              title: "Unpriced",
              hours: 5,
              levels: 1,
              fee_minor: null,
              fee_currency: null,
            },
          ],
        },
      }),
    );
    await page.goto("/skill-gaps");
    await page.getByTestId("catalogue-skill").selectOption(skill);
    await expect(page.getByTestId("course-fee").first()).toContainText(
      "£300.00",
    );
    await expect(page.getByTestId("course-fee").nth(1)).toContainText(
      "no fee recorded",
    );
  });

  test("spend stands against a budget per currency", async ({ page }) => {
    await page.route("**/api/proxy/learning-budgets?*", (route) =>
      route.fulfill({
        json: {
          organization: PERSON.organization_ref,
          year: 2026,
          derivation: "",
          departments: [
            {
              department: "engineering",
              completions_without_fee: 2,
              lines: [
                {
                  currency: "EUR",
                  budget_minor: null,
                  spent_minor: 5000,
                  remaining_minor: null,
                },
                {
                  currency: "GBP",
                  budget_minor: 120000,
                  spent_minor: 30000,
                  remaining_minor: 90000,
                },
              ],
            },
          ],
        },
      }),
    );
    await page.goto("/learning");
    await expect(page.getByTestId("budget-line").first()).toContainText("—");
    await expect(page.getByTestId("budget-line").nth(1)).toContainText(
      "£900.00",
    );
    await expect(page.getByTestId("budget-no-fee")).toContainText(": 2");
  });
});
