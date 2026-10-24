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

  test("shows the service's words when a move is refused", async ({ page }) => {
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
    await expect(page.getByTestId("error")).toContainText(
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
});
