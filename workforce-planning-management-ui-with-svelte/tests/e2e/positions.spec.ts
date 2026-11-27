// Positions (`/positions`): the establishment per cost centre and the seats. Over-establishment is not netted against a vacancy,
// an unknown FTE says unknown, a withheld figure says withheld, a reader who is not HR sees whether a seat is held and not by whom,
// and a refused move shows the service's words. The API is stubbed per test.

import { expect, test } from "@playwright/test";
import { WORKER, stubSignedIn } from "./support";

const SEAT = {
  pid: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
  organization_ref: WORKER.organization_ref,
  cost_centre: "CC-100",
  title: "Support Engineer",
  budget_fte_percent: 100,
  funding_kind: "core",
  funding_ends_on: null,
  programme_ref: null,
  state: "approved",
  overtime_status: "unknown",
  requisition_pid: null,
  demand_line_pid: null,
};

const FIGURE = (percent: number) => ({ status: "known", percent });
const CENTRE = {
  cost_centre: "CC-100",
  positions: 2,
  budgeted: FIGURE(150),
  held: FIGURE(160),
  vacant: FIGURE(40),
  over: FIGURE(50),
  frozen_vacant: FIGURE(0),
  by_state: { approved: 1, filled: 1 },
};

test.describe("Positions", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
  });

  test("shows the establishment without netting, and plans and moves a seat", async ({
    page,
  }) => {
    const seats = [{ ...SEAT, holder_worker_pid: null }];
    const planned: unknown[] = [];
    const moved: unknown[] = [];
    await page.route(
      "**/api/proxy/positions/establishment?organization=*",
      (route) =>
        route.fulfill({
          json: {
            small_group_floor: null,
            cost_centres: [CENTRE],
            derivation: "",
          },
        }),
    );
    await page.route("**/api/proxy/positions?organization=*", (route) =>
      route.fulfill({ json: { states: [], positions: seats } }),
    );
    await page.route("**/api/proxy/positions", (route) => {
      planned.push(route.request().postDataJSON());
      seats.push({
        ...SEAT,
        pid: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
        title: "Analyst",
        state: "planned",
        holder_worker_pid: null,
      });
      return route.fulfill({
        json: { pid: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb" },
      });
    });
    await page.route(`**/api/proxy/positions/${SEAT.pid}/status`, (route) => {
      moved.push(route.request().postDataJSON());
      return route.fulfill({ json: { pid: SEAT.pid, state: "frozen" } });
    });

    await page.goto("/positions");
    const establishment = page.getByTestId("establishment");
    await expect(establishment).toContainText("CC-100");
    // Over 0.50 and vacant 0.40 both appear: one is not taken off the other.
    await expect(establishment).toContainText("0.50");
    await expect(establishment).toContainText("0.40");
    await expect(establishment).toContainText("1.50");
    await expect(page.getByTestId("position-list")).toContainText(
      "Support Engineer",
    );
    await expect(page.getByTestId("position-list")).toContainText("Not held");

    await page
      .getByRole("combobox", { name: "Move to" })
      .first()
      .selectOption("frozen");
    await page
      .getByRole("button", { name: "Move", exact: true })
      .first()
      .click();
    await expect.poll(() => moved).toEqual([{ to: "frozen" }]);

    await page.getByRole("textbox", { name: /Cost centre/ }).fill("CC-200");
    await page.getByRole("textbox", { name: /Title/ }).fill("Analyst");
    await page.getByRole("button", { name: "Plan", exact: true }).click();
    await expect(page.getByTestId("position-list")).toContainText("Analyst");
    expect(planned).toEqual([
      {
        organization_ref: WORKER.organization_ref,
        cost_centre: "CC-200",
        title: "Analyst",
        budget_fte_percent: 100,
        funding_kind: "core",
      },
    ]);
  });

  test("a person records a seat as exempt, and a seat nobody recorded says so", async ({
    page,
  }) => {
    const seats = [{ ...SEAT, holder_worker_pid: null }];
    const planned: Record<string, unknown>[] = [];
    await page.route(
      "**/api/proxy/positions/establishment?organization=*",
      (route) =>
        route.fulfill({
          json: { small_group_floor: null, cost_centres: [], derivation: "" },
        }),
    );
    await page.route("**/api/proxy/positions?organization=*", (route) =>
      route.fulfill({ json: { states: [], positions: seats } }),
    );
    await page.route("**/api/proxy/positions", (route) => {
      const body = route.request().postDataJSON() as Record<string, unknown>;
      planned.push(body);
      seats.push({
        ...SEAT,
        pid: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
        title: "Director",
        overtime_status: body.overtime_status as string,
        holder_worker_pid: null,
      });
      return route.fulfill({
        json: { pid: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb" },
      });
    });
    await page.goto("/positions");
    const list = page.getByTestId("position-list");
    await expect(list).toContainText("Not recorded");
    await page.getByRole("textbox", { name: /Cost centre/ }).fill("CC-300");
    await page.getByRole("textbox", { name: /Title/ }).fill("Director");
    await page.getByTestId("position-overtime").selectOption("exempt");
    await page.getByRole("button", { name: "Plan", exact: true }).click();
    await expect.poll(() => planned[0]?.overtime_status).toBe("exempt");
    await expect(list.getByRole("row", { name: /Director/ })).toContainText(
      "Exempt",
    );
  });

  test("says unknown and withheld in words, and that a seat is held without saying by whom", async ({
    page,
  }) => {
    await page.route(
      "**/api/proxy/positions/establishment?organization=*",
      (route) =>
        route.fulfill({
          json: {
            small_group_floor: 10,
            cost_centres: [
              {
                ...CENTRE,
                held: { status: "unknown" },
                vacant: { status: "withheld" },
                over: { status: "withheld" },
              },
            ],
            derivation: "",
          },
        }),
    );
    await page.route("**/api/proxy/positions?organization=*", (route) =>
      route.fulfill({
        json: {
          states: [],
          positions: [{ ...SEAT, state: "filled", held: true }],
        },
      }),
    );
    await page.goto("/positions");
    await expect(page.getByTestId("establishment")).toContainText("unknown");
    await expect(page.getByTestId("establishment")).toContainText("withheld");
    await expect(page.getByTestId("position-list")).toContainText("Held");
  });

  test("says so in the catalogue's words when a move is refused", async ({
    page,
  }) => {
    await page.route(
      "**/api/proxy/positions/establishment?organization=*",
      (route) =>
        route.fulfill({
          json: { small_group_floor: null, cost_centres: [], derivation: "" },
        }),
    );
    await page.route("**/api/proxy/positions?organization=*", (route) =>
      route.fulfill({ json: { states: [], positions: [SEAT] } }),
    );
    await page.route(`**/api/proxy/positions/${SEAT.pid}/status`, (route) =>
      route.fulfill({
        status: 422,
        json: {
          description:
            'position is "approved"; it can be open only once a requisition is linked to it',
        },
      }),
    );
    await page.goto("/positions");
    await page
      .getByRole("combobox", { name: "Move to" })
      .first()
      .selectOption("open");
    await page
      .getByRole("button", { name: "Move", exact: true })
      .first()
      .click();
    // The catalogue's words, not the service's.
    await expect(page.getByTestId("error")).toContainText(
      "That was not accepted",
    );
    await expect(page.getByTestId("error")).not.toContainText(
      "requisition is linked",
    );
  });

  test("with no positions it says so", async ({ page }) => {
    await page.route(
      "**/api/proxy/positions/establishment?organization=*",
      (route) =>
        route.fulfill({
          json: { small_group_floor: null, cost_centres: [], derivation: "" },
        }),
    );
    await page.route("**/api/proxy/positions?organization=*", (route) =>
      route.fulfill({ json: { states: [], positions: [] } }),
    );
    await page.goto("/positions");
    await expect(page.getByTestId("position-list")).toContainText(
      "No positions yet",
    );
  });

  test("groups the establishment by band and pay point, ages the vacancies, withholds small critical groups and grades a seat", async ({
    page,
  }) => {
    const seats = [
      {
        ...SEAT,
        holder_worker_pid: null,
        critical: true,
        grade_scale: "national-2026-27",
        grade_band: "5",
        grade_pay_point: 2,
      },
      {
        ...SEAT,
        pid: "cccccccc-cccc-4ccc-8ccc-cccccccccccc",
        title: "Analyst",
        holder_worker_pid: null,
        critical: false,
        grade_scale: null,
        grade_band: null,
        grade_pay_point: null,
      },
    ];
    const graded: unknown[] = [];
    let cleared = 0;
    const group = (key: string, positions: number) => ({
      ...CENTRE,
      cost_centre: undefined,
      group: key,
      positions,
    });
    await page.route(
      "**/api/proxy/positions/establishment?organization=*",
      (route) =>
        route.fulfill({
          json: {
            small_group_floor: null,
            cost_centres: [CENTRE],
            bands: [group("national-2026-27:5", 1), group("unknown", 1)],
            pay_points: [group("national-2026-27:5:2", 1), group("unknown", 1)],
            derivation: "",
          },
        }),
    );
    await page.route("**/api/proxy/positions?organization=*", (route) =>
      route.fulfill({ json: { states: [], positions: seats } }),
    );
    const ages = (key: string, n: number | "withheld") => ({
      group: key,
      vacancies: n,
      by_age: { "0-30": n, "31-90": 0, "91-180": 0, "181+": 0 },
      start_unknown: 0,
      frozen: 0,
      frozen_without_review_date: 0,
      review_overdue: 0,
      abolished: 3,
      withheld: n === "withheld",
    });
    await page.route(
      "**/api/proxy/positions/vacancy-age?organization=*",
      (route) =>
        route.fulfill({
          json: {
            small_group_floor: 10,
            age_bands_days: ["0-30", "31-90", "91-180", "181+"],
            cost_centres: [ages("CC-100", 2)],
            bands: [ages("national-2026-27:5", 1), ages("unknown", 1)],
            critical: ages("critical", "withheld"),
            derivation: "",
          },
        }),
    );
    const hidden = (key: string) => ({
      group: key,
      withheld: true,
      posts: null,
      tenures: null,
      days_held: null,
      days_empty: null,
      held_now: null,
      counted_partly: null,
    });
    await page.route(
      "**/api/proxy/positions/critical-continuity?organization=*",
      (route) =>
        route.fulfill({
          json: {
            small_group_floor: 10,
            window_months: 36,
            cost_centres: { groups: [hidden("CC-100")], total: hidden("all") },
            bands: {
              groups: [hidden("national-2026-27:5")],
              total: hidden("all"),
            },
            derivation: "",
          },
        }),
    );
    await page.route(`**/api/proxy/positions/${SEAT.pid}/holders`, (route) =>
      route.fulfill({
        json: {
          pid: SEAT.pid,
          holders: [
            {
              worker_pid: null,
              erased: true,
              started_on: "2025-01-01",
              start_known: false,
              ended_on: "2025-06-01",
            },
            {
              worker_pid: "dddddddd-dddd-4ddd-8ddd-dddddddddddd",
              erased: false,
              started_on: "2025-06-01",
              start_known: true,
              ended_on: null,
            },
          ],
          continuity: {
            window_from: "2023-11-27",
            counted_from: "2025-01-01",
            complete: false,
            tenures: 2,
            days_held: 695,
            days_empty: 0,
          },
        },
      }),
    );
    await page.route(`**/api/proxy/positions/${SEAT.pid}/grade`, (route) => {
      if (route.request().method() === "DELETE") {
        cleared += 1;
      } else {
        graded.push(route.request().postDataJSON());
      }
      return route.fulfill({ json: { pid: SEAT.pid } });
    });

    await page.goto("/positions");
    const list = page.getByTestId("position-list");
    await expect(list).toContainText("✓ Critical");
    await expect(list).toContainText("national-2026-27:5:2");

    // The establishment by band, then by band and pay point: an ungraded seat is "unknown", never put in a band.
    const establishment = page.getByTestId("establishment");
    await page.getByTestId("establishment-group-by").selectOption("band");
    await expect(establishment).toContainText("national-2026-27:5");
    await expect(establishment).toContainText("unknown");
    await page.getByTestId("establishment-group-by").selectOption("pay_point");
    await expect(establishment).toContainText("national-2026-27:5:2");

    // Vacancy age by cost centre, and for the critical posts: a withheld group says so in words.
    const ageTable = page.getByTestId("vacancy-age-table");
    await expect(ageTable).toContainText("CC-100");
    await page.getByTestId("vacancy-group-by").selectOption("critical");
    await expect(ageTable).toContainText("Critical posts");
    await expect(ageTable).toContainText("withheld");

    // Critical-post continuity: every group and the total withheld, none shown as a number.
    const continuity = page.getByTestId("critical-continuity-table");
    await expect(continuity).toContainText("All critical posts");
    await expect(continuity).toContainText("withheld");

    // One seat's holders: the erased one is named nowhere, an unknown start says so, and a short history says so.
    await page.getByTestId(`holders-${SEAT.pid}`).click();
    const holders = page.getByTestId("position-holders");
    await expect(holders).toContainText("Erased");
    await expect(holders).toContainText("Start unknown");
    await expect(holders).toContainText("Holds it now");
    await expect(page.getByTestId("position-continuity-partial")).toBeVisible();

    // Grade a seat and clear it.
    await page.getByTestId("grade-seat").selectOption(SEAT.pid);
    await page.getByTestId("grade-band").fill("6");
    await page.getByTestId("grade-pay-point").fill("1");
    await page.getByTestId("grade-save").click();
    await expect
      .poll(() => graded)
      .toEqual([{ scale: "national-2026-27", band: "6", pay_point: 1 }]);
    await page.getByTestId("grade-clear").click();
    await expect.poll(() => cleared).toBe(1);
  });

  test("hides one seat's holders from a reader who may not see them", async ({
    page,
  }) => {
    await page.route(
      "**/api/proxy/positions/establishment?organization=*",
      (route) =>
        route.fulfill({
          json: { small_group_floor: 10, cost_centres: [], derivation: "" },
        }),
    );
    await page.route("**/api/proxy/positions?organization=*", (route) =>
      route.fulfill({
        json: { states: [], positions: [{ ...SEAT, held: false }] },
      }),
    );
    await page.route(`**/api/proxy/positions/${SEAT.pid}/holders`, (route) =>
      route.fulfill({ status: 403, json: { code: "forbidden" } }),
    );
    await page.goto("/positions");
    await page.getByTestId(`holders-${SEAT.pid}`).click();
    await expect(page.getByTestId("position-list")).toContainText(
      "Support Engineer",
    );
    await expect(page.getByTestId("position-holders")).toHaveCount(0);
  });
});
