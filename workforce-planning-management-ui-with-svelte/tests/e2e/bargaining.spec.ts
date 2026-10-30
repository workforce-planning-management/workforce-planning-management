// Recognised bodies and bargaining units (`/bargaining`). The API is stubbed per test; an unstubbed /api/proxy call 404s loudly.

import { expect, test } from "@playwright/test";
import { stubSignedIn } from "./support";

const body = (pid: string, name: string, units: number) => ({
  pid,
  organization_ref: "o",
  name,
  kind: "union",
  basis: "voluntary_agreement",
  recognised_on: "2020-03-01",
  contact_role: "HR relations lead",
  units,
});
const unit = (pid: string, name: string, coverage: unknown) => ({
  pid,
  name,
  department: "Care",
  location: null,
  employment_type: null,
  job_title: null,
  coverage,
});

test.describe("Bargaining", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
  });

  test("lists bodies with a contact role, shows coverage as a count, withheld or no one, and adds a unit", async ({
    page,
  }) => {
    let posted: Record<string, unknown> | null = null;
    let units = [
      unit("u1", "Care staff", { kind: "shown", covered: 12 }),
      unit("u2", "Night", { kind: "withheld" }),
      unit("u3", "Empty", { kind: "no_one" }),
    ];
    await page.route("**/api/proxy/recognised-bodies?*", (route) =>
      route.fulfill({
        json: { bodies: [body("b1", "Care Union", units.length)] },
      }),
    );
    await page.route(
      "**/api/proxy/recognised-bodies/b1/facility-time",
      (route) =>
        route.fulfill({
          json: { body_pid: "b1", periods: [], total_minutes: 0 },
        }),
    );
    await page.route("**/api/proxy/recognised-bodies/b1/units", (route) => {
      if (route.request().method() === "POST") {
        posted = route.request().postDataJSON();
        units = [...units, unit("u4", "Kitchen", { kind: "withheld" })];
        return route.fulfill({ json: { pid: "u4" } });
      }
      return route.fulfill({
        json: { body_pid: "b1", floor: 5, derivation: "rules", units },
      });
    });
    await page.goto("/bargaining");
    await expect(page.getByTestId("bg-body")).toContainText("Care Union");
    await expect(page.getByTestId("bg-body")).toContainText(
      "contact role: HR relations lead",
    );
    await page.getByTestId("bg-show-units").click();
    const coverage = page.getByTestId("bg-coverage");
    await expect(coverage).toHaveCount(3);
    await expect(coverage.nth(0)).toHaveText("12 covered");
    await expect(coverage.nth(1)).toContainText("fewer than 5 covered");
    await expect(coverage.nth(2)).toHaveText("no one covered");
    await page.getByTestId("bg-unit-name").fill("Kitchen");
    await page.getByTestId("bg-unit-department").fill("Catering");
    await page.getByTestId("bg-unit-add").click();
    await expect(page.getByTestId("bg-unit")).toHaveCount(4);
    expect(posted).toEqual({ name: "Kitchen", department: "Catering" });
  });

  test("reads an information pack with withheld cells, and records facility time per body with an over-budget flag", async ({
    page,
  }) => {
    let posted: Record<string, unknown> | null = null;
    let periods: unknown[] = [];
    await page.route("**/api/proxy/recognised-bodies?*", (route) =>
      route.fulfill({ json: { bodies: [body("b1", "Care Union", 1)] } }),
    );
    await page.route("**/api/proxy/recognised-bodies/b1/units", (route) =>
      route.fulfill({
        json: {
          body_pid: "b1",
          floor: 5,
          derivation: "r",
          units: [unit("u1", "Care staff", { kind: "shown", covered: 12 })],
        },
      }),
    );
    await page.route(
      "**/api/proxy/recognised-bodies/b1/facility-time",
      (route) => {
        if (route.request().method() === "POST") {
          posted = route.request().postDataJSON();
          periods = [
            {
              pid: "f1",
              period_start: "2026-01-01",
              period_end: "2026-03-31",
              minutes: 700,
              hours: "11.7",
              budget_minutes: 600,
              over_budget: true,
            },
          ];
          return route.fulfill({ json: { pid: "f1" } });
        }
        return route.fulfill({
          json: {
            body_pid: "b1",
            periods,
            total_minutes: periods.length ? 700 : 0,
          },
        });
      },
    );
    const cell = (n: number | null) =>
      n === null ? { kind: "withheld" } : { kind: "shown", count: n };
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
                { group: "Care", headcount: cell(12), fte_hundredths: 1000 },
              ],
              by_employment_type: [
                {
                  group: "permanent",
                  headcount: cell(null),
                  fte_hundredths: null,
                },
              ],
              by_pay_band: [],
              starters_last_year: cell(null),
              leavers_last_year: cell(3),
              contingent: [],
            },
            vacancies: { kind: "unknown", reason: "x" },
          },
        }),
    );
    await page.goto("/bargaining");
    await page.getByTestId("bg-show-units").click();
    await expect(page.getByTestId("bg-no-facility")).toBeVisible();
    await page.getByTestId("bg-pack").click();
    const result = page.getByTestId("bg-pack-result");
    await expect(result).toContainText("it is logged");
    await expect(result).toContainText("12 people covered, as of 27 Oct 2026");
    await expect(result).toContainText("withheld (under 5)");
    await expect(page.getByTestId("bg-vacancies")).toContainText("Not shown");
    await page.getByTestId("bg-fstart").fill("2026-01-01");
    await page.getByTestId("bg-fend").fill("2026-03-31");
    await page.getByTestId("bg-fminutes").fill("700");
    await page.getByTestId("bg-fbudget").fill("600");
    await page.getByTestId("bg-fadd").click();
    await expect(page.getByTestId("bg-period")).toContainText(
      "700 min (11.7 h)",
    );
    await expect(page.getByTestId("bg-over")).toBeVisible();
    expect(posted).toEqual({
      period_start: "2026-01-01",
      period_end: "2026-03-31",
      minutes: 700,
      budget_minutes: 600,
    });
  });

  test("a refusal is shown in the service's words", async ({ page }) => {
    await page.route("**/api/proxy/recognised-bodies?*", (route) =>
      route.fulfill({ status: 403, json: { error: "forbidden" } }),
    );
    await page.goto("/bargaining");
    await expect(page.getByTestId("error")).toContainText(
      "You are not allowed to do this.",
    );
  });
});
