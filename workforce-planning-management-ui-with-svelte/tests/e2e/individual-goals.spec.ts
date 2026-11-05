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
  await page.route("**/api/proxy/me/team-course-requests", (route) =>
    route.fulfill({ json: { waiting: [] } }),
  );
  await page.route("**/api/proxy/me/course-requests", (route) =>
    route.fulfill({ json: { has_manager: true, requests: [] } }),
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

test.describe("from a gap to a goal, and evidence beside the claim (WPM-R233, WPM-R234)", () => {
  const evidence = (
    state: string,
    declared: number | null,
    status = "agreed",
  ) => ({
    ...goal(status),
    skill_name: "Facilitation",
    skill_evidence: { target: 3, declared, state },
  });

  test("a goal shows the declared level beside its target, and unknown is not short", async ({
    page,
  }) => {
    await stub(page, [
      evidence("short", 2),
      {
        ...evidence("undeclared", null),
        goal: { ...goal("agreed").goal, pid: "g2" },
      },
    ]);
    await page.goto("/me/goals");
    const lines = page.getByTestId("goal-evidence");
    await expect(lines.first()).toHaveAttribute("data-state", "short");
    await expect(lines.first()).toContainText("Declared level 2 of 3");
    await expect(lines.nth(1)).toContainText("not known");
    await expect(page.getByTestId("goal-evidence-differs")).toHaveCount(0);
  });

  test("achieved with the level short says both, and neither overrides the other", async ({
    page,
  }) => {
    await stub(page, [evidence("short", 2, "achieved")]);
    await page.goto("/me/goals");
    await expect(page.getByTestId("goal-evidence-differs")).toBeVisible();
    await expect(page.getByTestId("goal").first()).toContainText(
      "Declared level 2 of 3",
    );
  });

  const gap = (
    skill_pid: string,
    skill: string,
    status: string,
    declared: number | null,
  ) => ({
    skill_pid,
    skill,
    category: "technical",
    required: 4,
    importance: "critical",
    sources: ["role"],
    declared,
    status,
    shortfall: declared === null ? null : 4 - declared,
    priority: 6,
  });

  async function onMe(page: Page, goals: unknown[]) {
    await stub(page, goals);
    await page.route(`**/api/proxy/workers/${ME}/skill-gaps`, (route) =>
      route.fulfill({
        json: {
          worker_pid: ME,
          includes_aspirations: true,
          counts: { met: 0, below: 2, undeclared: 1 },
          gaps: [
            gap("s1", "Data modelling", "below", 2),
            gap("s2", "Facilitation", "below", 3),
            gap("s3", "Negotiation", "undeclared", null),
          ],
        },
      }),
    );
    await page.route(`**/api/proxy/workers/${ME}/training-plan**`, (route) =>
      route.fulfill({ status: 404, json: { error: "not stubbed" } }),
    );
  }

  test("a gap becomes a proposed goal, an open goal is not offered twice, and an unknown gets no offer", async ({
    page,
  }) => {
    await onMe(page, [
      { ...goal("agreed"), goal: { ...goal("agreed").goal, skill_pid: "s2" } },
    ]);
    const sent: Record<string, unknown>[] = [];
    await page.route("**/api/proxy/me/goals", async (route) => {
      sent.push(route.request().postDataJSON());
      await route.fulfill({ json: { pid: "g9" } });
    });
    await page.goto("/me");
    const table = page.getByTestId("skill-gaps-table");
    await expect(table.getByTestId("gap-goal-open")).toHaveCount(1);
    await expect(table.getByTestId("gap-make-goal")).toHaveCount(1);
    await table.getByTestId("gap-make-goal").click();
    await expect.poll(() => sent.length).toBe(1);
    expect(sent[0]).toMatchObject({
      title: "Reach level 4 in Data modelling",
      skill_pid: "s1",
      skill_target_level: 4,
      measures: [],
    });
    expect(String(sent[0]!.period_end) > String(sent[0]!.period_start)).toBe(
      true,
    );
  });
});

test.describe("A goal's target and courses", () => {
  const skillGoal = (status: string, target: number | null = 3) => ({
    ...goal(status),
    goal: { ...goal(status).goal, skill_pid: "s1", skill_target_level: target },
    skill_name: "Facilitation",
  });

  async function withCourses(page: Page, goals: unknown[], courses: unknown[]) {
    await stub(page, goals);
    await page.route("**/api/proxy/skills/s1/courses", (route) =>
      route.fulfill({
        json: {
          skill_pid: "s1",
          hours_per_level: null,
          default_hours_per_level: 30,
          courses,
        },
      }),
    );
    await page.route(
      `**/api/proxy/workers/${ME}/training-enrollments`,
      (route) => {
        return route.fulfill({
          json: [
            {
              pid: "e1",
              worker_pid: ME,
              course_ref: "course:done",
              status: "completed",
              completed_on: "2026-02-01",
              certificate_expires_on: null,
            },
          ],
        });
      },
    );
  }

  const course = (pid: string, title: string, fee_minor: number | null) => ({
    pid,
    course_ref: `course:${pid}`,
    title,
    hours: 12,
    levels: 1,
    fee_minor,
    fee_currency: fee_minor === null ? null : "GBP",
  });

  test("the form sends a target only with a skill, and changing the skill clears it", async ({
    page,
  }) => {
    await stub(page, []);
    const sent: Record<string, unknown>[] = [];
    await page.route("**/api/proxy/me/goals", async (route) => {
      sent.push(route.request().postDataJSON());
      await route.fulfill({ json: { pid: "g9" } });
    });
    await page.goto("/me/goals");
    const form = page.getByTestId("goal-form");
    await form.locator("summary").click();
    await expect(form.getByTestId("goal-target-level")).toHaveCount(0);
    await expect(
      form.getByTestId("goal-skill-pick").locator("option"),
    ).toHaveCount(2);
    await form.getByTestId("goal-skill-pick").selectOption("s1");
    await form.getByTestId("goal-target-level").selectOption("4");
    await form.getByTestId("goal-title").fill("Reach it");
    await form.getByTestId("goal-start").fill("2026-01-01");
    await form.getByTestId("goal-end").fill("2026-12-31");
    await form.getByTestId("goal-propose").click();
    await expect.poll(() => sent.length).toBe(1);
    expect(sent[0]).toMatchObject({ skill_pid: "s1", skill_target_level: 4 });

    // The form empties after a proposal, which closes its links.
    await form.locator("summary").click();
    await form.getByTestId("goal-skill-pick").selectOption("s1");
    await form.getByTestId("goal-target-level").selectOption("2");
    await form.getByTestId("goal-skill-pick").selectOption("");
    await form.locator("summary").click();
    await form.getByTestId("goal-skill-pick").selectOption("s1");
    await expect(form.getByTestId("goal-target-level")).toHaveValue("");
  });

  test("a stored target can change but is not offered as blank", async ({
    page,
  }) => {
    await stub(page, [skillGoal("proposed", 3)]);
    await page.goto("/me/goals");
    await page.getByTestId("goal-change").locator("summary").click();
    const level = page.getByTestId("goal").getByTestId("goal-target-level");
    await expect(level.locator("option")).toHaveText(["1", "2", "3", "4", "5"]);
    await expect(level).toHaveValue("3");
  });

  test("a goal lists its skill's courses with the fee, and unknown is not free", async ({
    page,
  }) => {
    await withCourses(
      page,
      [skillGoal("agreed")],
      [
        course("paid", "Priced course", 12500),
        course("free", "Free course", 0),
        course("none", "Unpriced course", null),
        course("done", "Finished course", 5000),
      ],
    );
    await page.goto("/me/goals");
    await page.getByTestId("goal-courses").locator("summary").click();
    const rows = page.getByTestId("goal-course");
    await expect(rows).toHaveCount(4);
    await expect(rows.nth(0).getByTestId("goal-course-fee")).toContainText(
      "125",
    );
    await expect(rows.nth(2).getByTestId("goal-course-fee")).toHaveText(
      "no fee recorded",
    );
    await expect(rows.nth(2).getByTestId("goal-course-fee")).not.toHaveText(
      /0/,
    );
    await expect(rows.nth(0).getByTestId("goal-course-standing")).toHaveCount(
      0,
    );
    await expect(rows.nth(3).getByTestId("goal-course-standing")).toHaveText(
      "completed",
    );
  });

  test("a reader who may not see the person's training still sees the catalogue, with no status claimed", async ({
    page,
  }) => {
    await withCourses(
      page,
      [skillGoal("agreed")],
      [course("done", "Finished course", 5000)],
    );
    await page.route(
      `**/api/proxy/workers/${ME}/training-enrollments`,
      (route) => route.fulfill({ status: 403, json: { error: "no" } }),
    );
    await page.goto("/me/goals");
    await page.getByTestId("goal-courses").locator("summary").click();
    await expect(page.getByTestId("goal-course")).toHaveCount(1);
    await expect(page.getByTestId("goal-course-standing")).toHaveCount(0);
  });

  test("a free course enrols at once, any other asks, and a waiting one can be taken back", async ({
    page,
  }) => {
    await withCourses(
      page,
      [skillGoal("agreed")],
      [
        course("free", "Free course", 0),
        course("paid", "Priced course", 12500),
        course("none", "Unpriced course", null),
      ],
    );
    const requests: Array<{
      pid: string;
      course_ref: string;
      status: string;
      decided_on: null;
    }> = [];
    const started: string[] = [];
    await page.route("**/api/proxy/me/course-requests", (route) => {
      if (route.request().method() === "GET") {
        return route.fulfill({ json: { has_manager: true, requests } });
      }
      const ref = route.request().postDataJSON().course_ref as string;
      started.push(ref);
      requests.unshift({
        pid: `r${started.length}`,
        course_ref: ref,
        status: "requested",
        decided_on: null,
      });
      return route.fulfill({
        json: {
          has_manager: true,
          started: { outcome: "requested", pid: "r1" },
        },
      });
    });
    const withdrawn: string[] = [];
    await page.route("**/api/proxy/me/course-requests/*/withdraw", (route) => {
      withdrawn.push(route.request().url());
      requests[0]!.status = "withdrawn";
      return route.fulfill({ json: {} });
    });
    await page.goto("/me/goals");
    await page.getByTestId("goal-courses").locator("summary").click();
    const rows = page.getByTestId("goal-course");
    await expect(rows.nth(0).getByTestId("goal-course-enrol")).toHaveText(
      "Enrol",
    );
    await expect(rows.nth(1).getByTestId("goal-course-ask")).toHaveText(
      "Ask to enrol",
    );
    await expect(rows.nth(2).getByTestId("goal-course-ask")).toBeVisible();
    await rows.nth(1).getByTestId("goal-course-ask").click();
    await expect(rows.nth(1).getByTestId("goal-course-request")).toHaveText(
      "requested",
    );
    await expect(rows.nth(1).getByTestId("goal-course-ask")).toHaveCount(0);
    expect(started).toEqual(["course:paid"]);
    await rows.nth(1).getByTestId("goal-course-withdraw").click();
    await expect(rows.nth(1).getByTestId("goal-course-ask")).toBeVisible();
    expect(withdrawn).toHaveLength(1);
  });

  test("someone with no manager is told nobody can approve yet", async ({
    page,
  }) => {
    await withCourses(
      page,
      [skillGoal("agreed")],
      [course("paid", "Priced course", 12500)],
    );
    await page.route("**/api/proxy/me/course-requests", (route) =>
      route.fulfill({ json: { has_manager: false, requests: [] } }),
    );
    await page.goto("/me/goals");
    await page.getByTestId("goal-courses").locator("summary").click();
    await expect(page.getByTestId("goal-courses-no-manager")).toBeVisible();
  });

  test("a manager reads a report's goal courses with nothing to press, and decides a request with its fee", async ({
    page,
  }) => {
    await stub(
      page,
      [],
      [
        {
          worker_pid: "w2",
          display_name: "Test Worker 002",
          goals: [skillGoal("agreed")],
        },
      ],
    );
    await page.route("**/api/proxy/skills/s1/courses", (route) =>
      route.fulfill({
        json: {
          skill_pid: "s1",
          hours_per_level: null,
          default_hours_per_level: 30,
          courses: [course("free", "Free course", 0)],
        },
      }),
    );
    await page.route(
      `**/api/proxy/workers/${ME}/training-enrollments`,
      (route) => route.fulfill({ json: [] }),
    );
    const waiting = [
      {
        pid: "r1",
        worker_pid: "w2",
        display_name: "Test Worker 002",
        course_ref: "course:paid",
        title: "Priced course",
        hours: 8,
        fee_minor: 12500,
        fee_currency: "GBP",
      },
      {
        pid: "r2",
        worker_pid: "w2",
        display_name: "Test Worker 002",
        course_ref: "course:none",
        title: null,
        hours: null,
        fee_minor: null,
        fee_currency: null,
      },
    ];
    await page.route("**/api/proxy/me/team-course-requests", (route) =>
      route.fulfill({ json: { waiting } }),
    );
    const decided: string[] = [];
    await page.route("**/api/proxy/me/course-requests/*/*", (route) => {
      decided.push(
        new URL(route.request().url()).pathname.split("/").slice(-2).join("/"),
      );
      waiting.shift();
      return route.fulfill({ json: {} });
    });
    await page.goto("/me/goals");
    const person = page.getByTestId("team-person");
    await person.getByTestId("goal-courses").locator("summary").click();
    await expect(person.getByTestId("goal-course")).toHaveCount(1);
    await expect(
      person.getByTestId("goal-course").getByRole("button"),
    ).toHaveCount(0);

    const rows = page.getByTestId("course-request");
    await expect(rows).toHaveCount(2);
    await expect(rows.nth(0).getByTestId("course-request-fee")).toContainText(
      "125",
    );
    await expect(rows.nth(1).getByTestId("course-request-fee")).toHaveText(
      "no fee recorded",
    );
    await rows.nth(0).getByTestId("course-approve").click();
    await expect(rows).toHaveCount(1);
    await rows.nth(0).getByTestId("course-decline").click();
    await expect(page.getByTestId("course-requests")).toHaveCount(0);
    expect(decided).toEqual(["r1/approve", "r2/decline"]);
  });

  test("a skill with no course says so, and a closed goal lists none", async ({
    page,
  }) => {
    await withCourses(page, [skillGoal("in_progress")], []);
    await page.goto("/me/goals");
    await page.getByTestId("goal-courses").locator("summary").click();
    await expect(page.getByTestId("goal-courses-none")).toBeVisible();

    await withCourses(page, [skillGoal("achieved")], []);
    await page.goto("/me/goals");
    await expect(page.getByTestId("goal")).toBeVisible();
    await expect(page.getByTestId("goal-courses")).toHaveCount(0);
  });

  test("a manager makes a report's gap a proposal that person must agree", async ({
    page,
  }) => {
    await stub(
      page,
      [],
      [{ worker_pid: "w2", display_name: "Test Worker 002", goals: [] }],
    );
    let gapsRead = 0;
    await page.route("**/api/proxy/workers/w2/skill-gaps", (route) => {
      gapsRead += 1;
      return route.fulfill({
        json: {
          worker_pid: "w2",
          includes_aspirations: false,
          counts: { met: 0, below: 1, undeclared: 0 },
          gaps: [
            {
              skill_pid: "s1",
              skill: "Facilitation",
              required: 4,
              declared: 2,
              status: "below",
              shortfall: 2,
              importance: "important",
              priority: 6,
              sources: ["role"],
            },
          ],
        },
      });
    });
    await page.route("**/api/proxy/workers/w2/goals", (route) =>
      route.fulfill({
        json: { worker_pid: "w2", has_manager: true, goals: [] },
      }),
    );
    await page.route("**/api/proxy/workers/w2/training-plan**", (route) =>
      route.fulfill({ status: 404, json: { error: "not stubbed" } }),
    );
    const sent: Record<string, unknown>[] = [];
    await page.route("**/api/proxy/me/reports/w2/goals", async (route) => {
      sent.push(route.request().postDataJSON());
      await route.fulfill({ json: { pid: "g9" } });
    });
    await page.goto("/me/goals");
    const gaps = page.getByTestId("report-gaps");
    expect(gapsRead).toBe(0);
    await gaps.locator("summary").click();
    await expect(
      gaps.getByRole("heading", { name: "Skill gaps", exact: true }),
    ).toBeVisible();
    await gaps.getByTestId("gap-make-goal").click();
    await expect.poll(() => sent.length).toBe(1);
    expect(sent[0]).toMatchObject({
      skill_pid: "s1",
      skill_target_level: 4,
      measures: [],
    });
  });
});
