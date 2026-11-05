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

async function stub(
  page: Page,
  goals: unknown[],
  team: unknown[] = [],
  hasManager = true,
) {
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
    route.fulfill({ json: { worker_pid: ME, has_manager: hasManager, goals } }),
  );
  await page.route("**/api/proxy/me/team-goals", (route) =>
    route.fulfill({ json: { team } }),
  );
  await page.route("**/api/proxy/skills", (route) =>
    route.fulfill({
      json: [
        {
          pid: "s1",
          name: "Facilitation",
          category: "technical",
          external_refs: [],
        },
      ],
    }),
  );
  await page.route("**/api/proxy/workers/*/development-plans", (route) =>
    route.fulfill({
      json: {
        worker_pid: ME,
        plans: [
          {
            pid: "p1",
            items: [
              {
                pid: "i1",
                plan_pid: "p1",
                skill_pid: "s1",
                current_level: 1,
                target_level: 3,
                method: "course",
                course_ref: null,
                due_on: null,
                status: "planned",
              },
            ],
          },
        ],
      },
    }),
  );
}

test.describe("Individual goals", () => {
  test("a manager proposes a first goal for someone with none", async ({
    page,
  }) => {
    let sent: Record<string, unknown> | null = null;
    await stub(
      page,
      [],
      [{ worker_pid: PERSON.pid, display_name: "Ann Report", goals: [] }],
    );
    await page.route(`**/api/proxy/me/reports/${PERSON.pid}/goals`, (route) => {
      sent = route.request().postDataJSON();
      return route.fulfill({ json: { pid: "g2" } });
    });
    await page.goto("/me/goals");
    const person = page.getByTestId("team-person");
    await expect(person).toContainText("Ann Report");
    await person.getByTestId("report-propose").locator("summary").click();
    await person.getByTestId("goal-title").fill("Lead a retrospective");
    await person.getByTestId("goal-start").fill("2027-01-01");
    await person.getByTestId("goal-end").fill("2027-12-31");
    await person.getByText("Links (optional)").click();
    await person.getByTestId("goal-skill-pick").selectOption("s1");
    await person.getByTestId("goal-plan-pick").selectOption("i1");
    await person.getByTestId("goal-aligns-input").fill("objective:reliability");
    await person.getByTestId("goal-propose").click();
    await expect.poll(() => sent).not.toBeNull();
    expect(sent).toMatchObject({
      title: "Lead a retrospective",
      skill_pid: "s1",
      plan_item_pid: "i1",
      aligns_to: "objective:reliability",
    });
  });

  test("a goal shows what it links to", async ({ page }) => {
    const view = goal("agreed");
    await stub(page, [
      {
        ...view,
        skill_name: "Facilitation",
        plan_step: "Coaching",
        goal: { ...view.goal, aligns_to: "objective:reliability" },
      },
    ]);
    await page.goto("/me/goals");
    await expect(page.getByTestId("goal-skill")).toContainText("Facilitation");
    await expect(page.getByTestId("goal-plan-step")).toContainText("Coaching");
    await expect(page.getByTestId("goal-aligns")).toContainText(
      "objective:reliability",
    );
    await expect(page.getByTestId("goal-aligns").getByRole("link")).toHaveCount(
      0,
    );
  });

  test("a change to a proposal keeps a measure the form does not show", async ({
    page,
  }) => {
    let sent: { measures: unknown[] } | null = null;
    const view = goal("proposed", "manager");
    const measures = [
      ...view.measures,
      { ...view.measures[0], pid: "m2", name: "Reviews", target_value: 3 },
    ];
    await stub(page, [{ ...view, measures }]);
    await page.route("**/api/proxy/me/goals/g1", (route) => {
      sent = route.request().postDataJSON();
      return route.fulfill({ json: { pid: "g1" } });
    });
    await page.goto("/me/goals");
    const change = page.getByTestId("goal-change");
    await change.locator("summary").click();
    await expect(change).toContainText("will need to agree your version");
    await change.getByTestId("goal-title").fill("Publish it sooner");
    await change.getByTestId("goal-propose").click();
    await expect.poll(() => sent).not.toBeNull();
    expect(sent!.measures).toHaveLength(2);
  });

  test("counts what waits for you, and says when nobody can agree", async ({
    page,
  }) => {
    await stub(page, [goal("proposed", "manager")]);
    await page.goto("/me/goals");
    await expect(page.getByTestId("goals-waiting-count")).toContainText("1");
    await expect(page.getByTestId("goals-no-manager")).toHaveCount(0);
  });

  test("a person with no manager is told their proposal cannot be agreed yet", async ({
    page,
  }) => {
    await stub(page, [goal("proposed", "worker")], [], false);
    await page.goto("/me/goals");
    await expect(page.getByTestId("goals-no-manager")).toBeVisible();
    await expect(page.getByTestId("goals-waiting-count")).toHaveCount(0);
  });

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
