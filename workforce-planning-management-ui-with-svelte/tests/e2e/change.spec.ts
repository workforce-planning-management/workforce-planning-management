// AI-driven change (`/change`): an initiative, the roles it touches, the skills whose demand it moves and
// how ready the affected workforce is (aggregate; no one is named). The API is stubbed per test; an
// unstubbed /api/proxy call 404s loudly.

import { expect, test, type Page } from "@playwright/test";
import { stubSignedIn } from "./support";

const INITIATIVE = {
  pid: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
  name: "Support copilot",
  kind: "ai_assistance",
  status: "draft",
  starts_on: null,
  roles_affected: 1,
  skills_shifting: 1,
};
const ROLE = {
  pid: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
  job_title: "Support Agent",
  description: null,
  source_ref: null,
  requirement_count: 0,
  framework: null,
  profession: null,
  role_name: null,
};
const SKILL = {
  pid: "cccccccc-cccc-4ccc-8ccc-cccccccccccc",
  name: "Prompting",
  category: "technical",
  external_refs: [],
};

const READINESS = {
  derivation:
    "Counts of employed workers in the affected roles; no one is named.",
  initiative: { pid: INITIATIVE.pid, name: INITIATIVE.name, status: "draft" },
  bar: 3,
  affected_workers: 12,
  with_active_reskill_plan: 3,
  reskill_plan_coverage: { numerator: 3, denominator: 12, value: 0.25 },
  roles: [
    {
      job_title: "Support Agent",
      impact: "reshaped",
      timeframe: "within_1y",
      employed_workers: 12,
      with_active_reskill_plan: 3,
      reskill_plan_coverage: { numerator: 3, denominator: 12, value: 0.25 },
    },
  ],
  rising_skills: [
    {
      skill: "Prompting",
      meeting: 2,
      below: 7,
      undeclared: 3,
      meeting_share: { numerator: 2, denominator: 12, value: 0.1667 },
    },
  ],
  declining_skills: ["Manual triage"],
};

async function stubChange(page: Page, writes: string[] = [], status = "draft") {
  await page.route("**/api/proxy/role-profiles", (route) =>
    route.fulfill({ json: [ROLE] }),
  );
  await page.route("**/api/proxy/skills", (route) =>
    route.fulfill({ json: [SKILL] }),
  );
  await page.route("**/api/proxy/change-initiatives", (route) => {
    if (route.request().method() === "POST") {
      writes.push(`POST create ${route.request().postData()}`);
      return route.fulfill({ json: { pid: INITIATIVE.pid } });
    }
    return route.fulfill({ json: [{ ...INITIATIVE, status }] });
  });
  await page.route(
    `**/api/proxy/change-initiatives/${INITIATIVE.pid}`,
    (route) =>
      route.fulfill({
        json: {
          pid: INITIATIVE.pid,
          name: INITIATIVE.name,
          description: null,
          kind: INITIATIVE.kind,
          status,
          role_impacts: [],
          skill_shifts: [],
        },
      }),
  );
  await page.route(
    `**/api/proxy/change-initiatives/${INITIATIVE.pid}/readiness`,
    (route) =>
      route.fulfill({
        json: { ...READINESS, initiative: { ...READINESS.initiative, status } },
      }),
  );
  for (const part of ["status", "role-impacts", "skill-shifts"]) {
    await page.route(
      `**/api/proxy/change-initiatives/${INITIATIVE.pid}/${part}`,
      (route) => {
        writes.push(
          `${route.request().method()} ${part} ${route.request().postData()}`,
        );
        return route.fulfill({
          json: part === "status" ? { pid: INITIATIVE.pid } : {},
        });
      },
    );
  }
}

test.describe("AI-driven change", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
  });

  test("shows an initiative's readiness in aggregate, with the working shown and nobody named", async ({
    page,
  }) => {
    await stubChange(page);
    await page.goto("/change");
    await expect(page).toHaveTitle("Change — WPM");
    await expect(page.getByTestId("change-select")).toContainText(
      "Support copilot",
    );
    await expect(page.getByTestId("change-summary")).toContainText("12");
    await expect(page.getByTestId("change-summary")).toContainText("3");
    await expect(page.getByTestId("change-roles")).toContainText(
      "Support Agent",
    );
    await expect(page.getByTestId("change-roles")).toContainText("12");
    await expect(page.getByTestId("change-skills")).toContainText("Prompting");
    await expect(page.getByText("Manual triage")).toBeVisible();
    await expect(page.getByText("Test Worker")).toHaveCount(0);
  });

  test("opens a new initiative", async ({ page }) => {
    const writes: string[] = [];
    await stubChange(page, writes);
    await page.goto("/change");
    await page.getByTestId("change-new-name").fill("Invoice automation");
    await page.getByTestId("change-create").click();
    await expect.poll(() => writes.length).toBe(1);
    expect(JSON.parse(writes[0]!.replace("POST create ", ""))).toEqual({
      name: "Invoice automation",
      kind: "ai_assistance",
    });
  });

  test("records a role impact and a skill shift while the initiative is open", async ({
    page,
  }) => {
    const writes: string[] = [];
    await stubChange(page, writes);
    await page.goto("/change");
    const roleForm = page
      .locator("form")
      .filter({ hasText: "Role" })
      .filter({ hasText: "Impact" });
    await roleForm.getByLabel("Role").selectOption(ROLE.pid);
    await roleForm.getByRole("button", { name: "Add" }).click();
    await expect.poll(() => writes.length).toBe(1);
    expect(writes[0]).toContain("PUT role-impacts");
    expect(writes[0]).toContain(`"role_profile_pid":"${ROLE.pid}"`);

    const skillForm = page.locator("form").filter({ hasText: "Direction" });
    await skillForm.getByLabel("Skill").selectOption(SKILL.pid);
    await skillForm.getByRole("button", { name: "Add" }).click();
    await expect.poll(() => writes.length).toBe(2);
    expect(writes[1]).toContain("PUT skill-shifts");
    expect(writes[1]).toContain('"direction":"rising"');
  });

  test("offers only the legal next statuses of a draft", async ({ page }) => {
    await stubChange(page, [], "draft");
    await page.goto("/change");
    await expect(page.getByRole("button", { name: /Mark/ })).toHaveCount(2); // active, cancelled
  });

  test("a completed initiative has no forms", async ({ page }) => {
    await stubChange(page, [], "completed");
    await page.goto("/change");
    await expect(page.getByTestId("change-summary")).toBeVisible();
    await expect(page.getByRole("button", { name: /Mark/ })).toHaveCount(0);
    await expect(page.getByLabel("Direction")).toHaveCount(0);
  });

  test("an illegal move is refused by the service and said in words", async ({
    page,
  }) => {
    await stubChange(page);
    await page.route(
      `**/api/proxy/change-initiatives/${INITIATIVE.pid}/status`,
      (route) =>
        route.fulfill({
          status: 422,
          json: { description: "draft cannot become completed" },
        }),
    );
    await page.goto("/change");
    await page.getByRole("button", { name: /Mark/ }).first().click();
    // The catalogue's words, not the service's.
    await expect(page.getByTestId("error")).toContainText(
      "That was not accepted",
    );
    await expect(page.getByTestId("error")).not.toContainText(
      "draft cannot become completed",
    );
  });
});
