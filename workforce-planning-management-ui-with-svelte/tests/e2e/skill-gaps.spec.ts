// Skill gaps. The API is stubbed per test; an unstubbed /api/proxy call 404s loudly.

import { expect, test } from "@playwright/test";
import { WORKER, stubSignedIn } from "./support";

test.describe("Skill gaps", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
  });

  test("skill gaps: a person's ranked gaps and the workforce roll-up (counts only)", async ({
    page,
  }) => {
    await page.route(`**/api/proxy/workers/${WORKER.pid}/skill-gaps`, (route) =>
      route.fulfill({
        json: {
          worker_pid: WORKER.pid,
          includes_aspirations: true,
          counts: { met: 1, below: 1, undeclared: 1 },
          gaps: [
            {
              skill_pid: "s1",
              skill: "Data modelling",
              category: "technical",
              required: 4,
              importance: "critical",
              sources: ["role", "target"],
              declared: 2,
              status: "below",
              shortfall: 2,
              priority: 6,
            },
            {
              skill_pid: "s2",
              skill: "Stakeholder management",
              category: "leadership",
              required: 3,
              importance: "important",
              sources: ["role", "aspiration"],
              declared: null,
              status: "undeclared",
              shortfall: null,
              priority: 0,
            },
          ],
        },
      }),
    );
    const paces: string[] = [];
    await page.route(
      `**/api/proxy/workers/${WORKER.pid}/training-plan**`,
      (route) => {
        const weekly =
          new URL(route.request().url()).searchParams.get("weekly_hours") ?? "";
        paces.push(weekly);
        return route.fulfill({
          json: {
            worker_pid: WORKER.pid,
            weekly_hours: weekly ? Number(weekly) : 4,
            start: "2026-10-06",
            total_hours: 110,
            total_weeks: 22,
            finish_on: "2027-03-08",
            plan: [
              {
                skill_pid: "s1",
                skill: "Data modelling",
                importance: "critical",
                required: 4,
                declared: 2,
                shortfall: 2,
                priority: 6,
                recommendation: {
                  courses: [
                    {
                      course_ref: "course:a",
                      title: "Deep dive",
                      hours: 60,
                      levels: 2,
                    },
                  ],
                  course_levels: 2,
                  estimated_levels: 0,
                  hours: 60,
                  basis: "courses",
                },
                starts_on: "2026-10-06",
                ends_on: "2027-01-11",
                weeks: 15,
                cumulative_hours: 60,
              },
              {
                skill_pid: "s3",
                skill: "Facilitation",
                importance: "useful",
                required: 3,
                declared: 1,
                shortfall: 2,
                priority: 2,
                recommendation: {
                  courses: [],
                  course_levels: 0,
                  estimated_levels: 2,
                  hours: 50,
                  basis: "estimate",
                },
                starts_on: "2027-01-12",
                ends_on: "2027-03-08",
                weeks: 8,
                cumulative_hours: 110,
              },
            ],
            assess_first: [
              {
                skill_pid: "s2",
                skill: "Stakeholder management",
                required: 3,
                importance: "important",
              },
            ],
          },
        });
      },
    );
    await page.goto("/me");
    // The training plan: hours by what they rest on, scheduled, and honest about unknowns.
    const planTable = page.getByTestId("training-plan");
    await expect(planTable).toContainText("Catalogue courses");
    await expect(planTable).toContainText("Deep dive (60)");
    await expect(
      planTable.getByText("Estimate", { exact: true }),
    ).toBeVisible();
    await expect(page.getByTestId("plan-total")).toContainText("110");
    await expect(page.getByTestId("plan-total")).toContainText("2027-03-08");
    await expect(page.getByTestId("plan-assess")).toContainText(
      "Stakeholder management",
    );
    await page.getByTestId("plan-weekly").fill("6");
    await expect.poll(() => paces.includes("6")).toBe(true);
    const mine = page.getByTestId("skill-gaps-table");
    await expect(mine).toContainText("Data modelling");
    await expect(mine).toContainText("Role, Your target");
    // Unknown is shown as "Not declared" — never a number of levels short.
    const unknownRow = mine.locator('tr[data-status="undeclared"]');
    await expect(unknownRow).toContainText("Not declared");
    await expect(unknownRow).toContainText("Role, Aspiration");

    await page.route(
      "**/api/proxy/workforce-intelligence/skill-gaps**",
      (route) =>
        route.fulfill({
          json: {
            as_of: "2026-10-06",
            workers_considered: 40,
            workers_with_needs: 31,
            skills_total: 1,
            skills: [
              {
                skill_pid: "s1",
                skill: "Data modelling",
                category: "technical",
                needed_by: 12,
                met: 5,
                below: 6,
                undeclared: 1,
                total_shortfall: 11,
                critical_below: 4,
                score: 31,
                departments: [
                  { department: "engineering", below: 4 },
                  { department: "finance", below: 2 },
                ],
              },
            ],
          },
        }),
    );
    await page.route(
      "**/api/proxy/workforce-intelligence/training-demand**",
      (route) =>
        route.fulfill({
          json: {
            as_of: "2026-10-06",
            workers_considered: 40,
            people_with_gaps: 12,
            total_hours: 880,
            average_hours_per_person: 73.3,
            skills: [
              {
                skill_pid: "s1",
                skill: "Data modelling",
                people: 6,
                hours: 420,
                people_on_estimate: 2,
              },
            ],
            departments: [{ department: "engineering", people: 8, hours: 600 }],
          },
        }),
    );
    const catalogue: Array<Record<string, unknown>> = [];
    await page.route("**/api/proxy/skills", (route) =>
      route.fulfill({
        json: [
          {
            pid: "s1",
            name: "Data modelling",
            category: "technical",
            external_refs: [],
          },
        ],
      }),
    );
    await page.route("**/api/proxy/skills/s1/courses", (route) => {
      if (route.request().method() === "POST") {
        catalogue.push({
          pid: `c${catalogue.length + 1}`,
          levels: 1,
          ...route.request().postDataJSON(),
        });
        return route.fulfill({ json: catalogue.at(-1) });
      }
      return route.fulfill({
        json: {
          skill_pid: "s1",
          hours_per_level: null,
          default_hours_per_level: 30,
          courses: catalogue,
        },
      });
    });
    await page.goto("/skill-gaps");
    await expect(page.getByTestId("demand-total")).toContainText("880");
    await expect(page.getByTestId("demand-total")).toContainText("73.3");
    await expect(page.getByTestId("demand-skills")).toContainText(
      "Data modelling",
    );
    await expect(page.getByTestId("demand-departments")).toContainText(
      "engineering: 600 (8)",
    );
    // The catalogue editor adds a course to a skill.
    await page.getByTestId("catalogue-skill").selectOption("s1");
    await expect(page.getByTestId("catalogue-courses")).toContainText(
      "No courses listed",
    );
    await page
      .getByTestId("course-ref")
      .fill("course:11111111-1111-4111-8111-1111111111a1");
    await page.getByTestId("course-title").fill("Fast intro");
    await page.getByTestId("course-hours").fill("10");
    await page.getByTestId("course-add").click();
    await expect(page.getByTestId("catalogue-courses")).toContainText(
      "Fast intro",
    );
    const table = page.getByTestId("workforce-gaps");
    await expect(table).toContainText("Data modelling");
    await expect(table).toContainText("engineering (4), finance (2)");
    await expect(page.locator("body")).toContainText("nobody is named");
  });
});
