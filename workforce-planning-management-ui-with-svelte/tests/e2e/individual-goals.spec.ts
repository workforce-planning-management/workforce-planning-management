// Individual goals (WPM-R224 to WPM-R226): a person's goals with measures that show no score, proposing, agreeing, a check-in,
// a manager's view by name, and the roll-up's floor. The API is stubbed per test.

import { expect, test, type Page } from "@playwright/test";
import { stubSignedIn, WORKER as PERSON } from "./support";

const ME = "33333333-3333-4333-8333-333333333333";

const goal = (
  status: string,
  proposed_by = "worker",
  title = "Publish it",
) => ({
  goal: {
    pid: "g1",
    worker_pid: ME,
    title,
    description: null,
    period_start: "2026-01-01",
    period_end: "2026-12-31",
    status,
    proposed_by,
    agreed_on: null,
    aligns_to: null,
  },
  measures: [
    {
      pid: "m1",
      name: "Pages",
      kind: "number",
      unit: "pages",
      start_value: 0,
      target_value: 40,
      current_value: null,
      done: null,
    },
  ],
  check_ins: [],
});

async function stub(page: Page, goals: unknown[], team: unknown[] = []) {
  await stubSignedIn(page);
  await page.route("**/api/proxy/me/organizations", (route) =>
    route.fulfill({
      json: [
        {
          organization_ref: PERSON.organization_ref,
          role: "employee",
          worker_pid: ME,
        },
      ],
    }),
  );
  await page.route(`**/api/proxy/workers/${ME}/goals`, (route) =>
    route.fulfill({ json: { worker_pid: ME, goals } }),
  );
  await page.route("**/api/proxy/me/team-goals", (route) =>
    route.fulfill({ json: { team } }),
  );
}

test.describe("Individual goals", () => {
  test("shows a measure's own values and no score", async ({ page }) => {
    await stub(page, [goal("in_progress")]);
    await page.goto("/me/goals");
    const measure = page.getByTestId("goal-measure");
    await expect(measure).toContainText("Current —");
    await expect(measure).toContainText("Target 40");
    await expect(measure).not.toContainText("%");
  });

  test("the proposer waits and the other party agrees", async ({ page }) => {
    let agreed = false;
    await stub(page, [goal("proposed", "manager")]);
    await page.route("**/api/proxy/me/goals/g1/agree", (route) => {
      agreed = true;
      return route.fulfill({ json: { pid: "g1", status: "agreed" } });
    });
    await page.goto("/me/goals");
    await page.getByTestId("goal-agree").click();
    await expect.poll(() => agreed).toBe(true);

    await stub(page, [goal("proposed", "worker")]);
    await page.reload();
    await expect(page.getByTestId("goal-waiting")).toBeVisible();
    await expect(page.getByTestId("goal-agree")).toHaveCount(0);
  });

  test("proposes a goal with a measure", async ({ page }) => {
    let sent: unknown = null;
    await stub(page, []);
    await page.route("**/api/proxy/me/goals", (route) => {
      sent = route.request().postDataJSON();
      return route.fulfill({ json: { pid: "g2" } });
    });
    await page.goto("/me/goals");
    await expect(page.getByTestId("goals-none")).toBeVisible();
    await page.getByTestId("goal-title").fill("Publish it");
    await page.getByTestId("goal-start").fill("2026-01-01");
    await page.getByTestId("goal-end").fill("2026-12-31");
    await page.getByTestId("measure-name").fill("Pages");
    await page.getByTestId("measure-target").fill("40");
    await page.getByTestId("goal-propose").click();
    await expect
      .poll(() => sent)
      .toMatchObject({
        title: "Publish it",
        measures: [{ name: "Pages", kind: "number", target_value: 40 }],
      });
  });

  test("a check-in sends its measure", async ({ page }) => {
    let sent: unknown = null;
    await stub(page, [goal("agreed")]);
    await page.route("**/api/proxy/me/goals/g1/check-ins", (route) => {
      sent = route.request().postDataJSON();
      return route.fulfill({ json: { pid: "c1" } });
    });
    await page.goto("/me/goals");
    await page.getByTestId("checkin-status").selectOption("at_risk");
    await page.getByTestId("checkin-note").fill("Waiting on review");
    await page.getByTestId("checkin-measure").selectOption("m1");
    await page.getByTestId("checkin-value").fill("12");
    await page.getByTestId("checkin-add").click();
    await expect
      .poll(() => sent)
      .toMatchObject({
        status: "at_risk",
        measure_pid: "m1",
        current_value: 12,
      });
  });

  test("a manager sees their team by name", async ({ page }) => {
    await stub(
      page,
      [],
      [
        {
          worker_pid: "w2",
          display_name: "Test Worker 002",
          goals: [goal("proposed", "worker", "Their goal")],
        },
      ],
    );
    await page.goto("/me/goals");
    const person = page.getByTestId("team-person");
    await expect(person).toContainText("Test Worker 002");
    await expect(person).toContainText("Their goal");
    await expect(person.getByTestId("goal-agree")).toBeVisible();
  });

  test("the roll-up withholds a small department", async ({ page }) => {
    await stubSignedIn(page);
    await page.route("**/api/proxy/goals/rollup?*", (route) =>
      route.fulfill({
        json: {
          organization: PERSON.organization_ref,
          floor: 5,
          derivation: "",
          departments: [
            { department: "engineering", by_status: { agreed: 7 } },
            { department: "legal", by_status: null },
          ],
        },
      }),
    );
    await page.goto("/development");
    await expect(page.getByTestId("rollup-line").first()).toContainText(
      "agreed 7",
    );
    await expect(page.getByTestId("rollup-line").nth(1)).toContainText(
      "too few people",
    );
  });
});
