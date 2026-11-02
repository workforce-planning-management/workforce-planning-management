// Statutory and mandatory learning (WPM-R170, WPM-T315): HR's roll-up page and the person's own panel on /me. The API is stubbed per
// test; an unstubbed /api/proxy call 404s loudly.

import { expect, test } from "@playwright/test";
import { WORKER, stubSignedIn } from "./support";

test.describe("Statutory and mandatory learning", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
  });

  test("the roll-up shows counts and a rate, hides a small group, and says unknown apart", async ({
    page,
  }) => {
    await page.route(
      "**/api/proxy/learning-requirements/rollup?organization=*",
      (route) =>
        route.fulfill({
          json: {
            organization: WORKER.organization_ref,
            as_of: "2026-11-02",
            floor: 10,
            requirements: [
              {
                requirement_pid: "r1",
                subject: "Fire safety",
                basis: "statutory",
                hidden: false,
                compliant: 8,
                due_soon: 2,
                expired: 1,
                never_completed: 1,
                not_applicable: 0,
                unknown: 3,
                rate: 10 / 12,
              },
              {
                requirement_pid: "r2",
                subject: "Data protection",
                basis: "mandatory",
                hidden: true,
              },
            ],
          },
        }),
    );
    await page.goto("/en-001/mandatory-learning");
    await page
      .getByTestId("learning-organization")
      .selectOption(WORKER.organization_ref);
    const table = page.getByTestId("learning-rollup");
    const fire = table.getByRole("row", { name: /Fire safety/ });
    await expect(fire).toContainText("Statutory");
    await expect(fire).toContainText("83");
    await expect(fire).toContainText("3");
    const small = table.getByRole("row", { name: /Data protection/ });
    await expect(small).toContainText("Too few people to show");
    await expect(small).not.toContainText("0%");
  });

  test("a person sees their own status, and unknown is said in words", async ({
    page,
  }) => {
    await page.route(
      `**/api/proxy/workers/${WORKER.pid}/mandatory-learning`,
      (route) =>
        route.fulfill({
          json: {
            worker_pid: WORKER.pid,
            as_of: "2026-11-02",
            requirements: [
              {
                requirement_pid: "r1",
                subject: "Fire safety",
                basis: "statutory",
                source: "Fire Safety Order",
                status: "due_soon",
                last_completed_on: "2025-12-01",
                expires_on: "2026-12-01",
              },
              {
                requirement_pid: "r2",
                subject: "Manual handling",
                basis: "mandatory",
                source: null,
                status: "unknown",
                last_completed_on: null,
                expires_on: null,
              },
              {
                requirement_pid: "r3",
                subject: "Theatre safety",
                basis: "mandatory",
                source: null,
                status: "not_applicable",
                last_completed_on: null,
                expires_on: null,
              },
            ],
          },
        }),
    );
    await page.route(
      `**/api/proxy/workers/${WORKER.pid}/team-mandatory-learning`,
      (route) =>
        route.fulfill({
          json: {
            manager_pid: WORKER.pid,
            as_of: "2026-11-02",
            people: [
              {
                worker_pid: "99999999-9999-4999-8999-999999999999",
                display_name: "Report One",
                report_kind: "direct",
                requirements: [
                  {
                    requirement_pid: "r1",
                    subject: "Fire safety",
                    basis: "statutory",
                    status: "expired",
                    expires_on: "2026-10-01",
                  },
                ],
              },
              {
                worker_pid: "88888888-8888-4888-8888-888888888888",
                display_name: "Report Two",
                report_kind: "direct",
                requirements: [
                  {
                    requirement_pid: "r1",
                    subject: "Fire safety",
                    basis: "statutory",
                    status: "compliant",
                    expires_on: "2027-06-01",
                  },
                ],
              },
            ],
          },
        }),
    );
    await page.goto("/me");
    const team = page.getByTestId("team-learning");
    await expect(team).toContainText("Report One");
    await expect(team).toContainText("Expired");
    await expect(team).not.toContainText("Report Two");
    const panel = page.getByTestId("my-learning");
    await expect(panel).toContainText("Fire safety");
    await expect(panel.getByTestId("learning-due_soon")).toContainText(
      "Due soon",
    );
    await expect(panel.getByTestId("learning-unknown")).toContainText(
      "does not say which group you are in",
    );
    await expect(panel).not.toContainText("Theatre safety");
  });
});
