// Learning and mentorship. The API is stubbed per test; an unstubbed /api/proxy call 404s loudly.

import { expect, test } from "@playwright/test";
import { stubSignedIn } from "./support";

test.describe("Learning and mentorship", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
  });

  test("learning area renders skills matrix, analytics, and path progress", async ({
    page,
  }) => {
    await page.route("**/api/proxy/learning/skills-matrix", (route) =>
      route.fulfill({
        json: {
          as_of: "2026-07-20T00:00:00Z",
          note: "coverage over declared proficiencies only",
          matrix: [
            {
              department: "engineering",
              skill: "Rust",
              workers: 2,
              average_proficiency: 3.5,
              below_target: 1,
            },
          ],
          gaps: [
            {
              worker_pid: "e2",
              department: "engineering",
              skill: "Rust",
              proficiency: 2,
              target: 4,
            },
          ],
        },
      }),
    );
    await page.route("**/api/proxy/learning/training-analytics", (route) =>
      route.fulfill({
        json: {
          as_of: "2026-07-20",
          horizon: "2026-10-18",
          note: "completion rate = completed / non-failed",
          departments: [
            {
              department: "engineering",
              by_status: { completed: 1 },
              completion_rate: { numerator: 1, denominator: 1, value: 1 },
              certs_expiring: 0,
            },
          ],
        },
      }),
    );
    await page.route("**/api/proxy/learning-paths", (route) =>
      route.fulfill({
        json: [
          { pid: "path1", name: "Backend basics", summary: null, steps: 2 },
        ],
      }),
    );
    await page.route("**/api/proxy/learning-paths/path1/progress", (route) =>
      route.fulfill({
        json: {
          as_of: "2026-07-20T00:00:00Z",
          path: { pid: "path1", name: "Backend basics" },
          steps: [{ course_ref: "course:a", title: "Intro", position: 0 }],
          derivation:
            "a step is complete iff a completed training enrolment matches",
          members: [
            {
              worker_pid: "e2",
              display_name: "Sam Mentee",
              completed_steps: 1,
              total_steps: 2,
            },
          ],
        },
      }),
    );
    await page.goto("/learning");
    await expect(
      page.getByTestId("skills-matrix").getByText("Rust"),
    ).toBeVisible();
    await expect(
      page.getByTestId("skills-gaps").getByText(/Rust in engineering/),
    ).toBeVisible();
    await expect(
      page.getByTestId("training-analytics").getByText("100% (1/1)"),
    ).toBeVisible();
    await expect(
      page.getByTestId("path-progress").getByText("Sam Mentee"),
    ).toBeVisible();
  });

  test("mentorship area renders load, unmatched, and stale", async ({
    page,
  }) => {
    await page.route("**/api/proxy/learning/mentorship-overview**", (route) =>
      route.fulfill({
        json: {
          as_of: "2026-07-20",
          active_pairings: 1,
          mentor_load: [
            { mentor_pid: "e1", mentor: "Ada Mentor", active_mentees: 1 },
          ],
          unmatched_total: 1,
          unmatched_workers: [
            { pid: "e3", display_name: "Solo Dev", department: "engineering" },
          ],
          stale_days: 30,
          stale_mentorships: [
            {
              pid: "m1",
              mentor: "Ada Mentor",
              mentee: "Sam Mentee",
              last_session: "2026-05-01",
            },
          ],
        },
      }),
    );
    await page.goto("/mentorship");
    await expect(
      page.getByTestId("mentor-load").getByText("Ada Mentor"),
    ).toBeVisible();
    await expect(
      page.getByTestId("unmatched").getByText("Solo Dev"),
    ).toBeVisible();
    await expect(
      page.getByTestId("stale-mentorships").getByText("1 May 2026"),
    ).toBeVisible();
  });
});
