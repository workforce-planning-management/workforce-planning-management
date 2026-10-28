// Role profiles (`/roles`): what a profile requires, adding and removing a required skill, and starting a
// profile from an ESCO occupation. The API is stubbed per test; an unstubbed /api/proxy call 404s loudly.

import { expect, test, type Page } from "@playwright/test";
import { base, json, stubRoles } from "./fixtures";

const R1 = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";
const PROMPTING = "cccccccc-cccc-4ccc-8ccc-cccccccccccc";

/** Record every non-GET under a pattern as "METHOD path body" and answer it. */
async function record(
  page: Page,
  pattern: string,
  writes: string[],
  answer: unknown = {},
) {
  await page.route(pattern, (route) => {
    if (route.request().method() === "GET") return route.fallback();
    writes.push(
      `${route.request().method()} ${new URL(route.request().url()).pathname.split("/proxy")[1]} ${route.request().postData() ?? ""}`.trim(),
    );
    return route.fulfill({ json: answer });
  });
}

test.describe("Role profiles", () => {
  test.beforeEach(async ({ page }) => {
    await base(page);
    await stubRoles(page);
  });

  test("shows a profile with its requirements, its coverage and the top of its ladder", async ({
    page,
  }) => {
    await page.goto("/roles");
    await expect(page).toHaveTitle("Role profiles — WPM");
    await expect(
      page.getByRole("heading", { level: 2, name: "Support Agent" }),
    ).toBeVisible();
    await expect(page.getByTestId("role-requirements")).toContainText(
      "Prompting",
    );
    await expect(page.getByTestId("role-requirements")).toContainText("3 / 5");
    await expect(page.getByTestId("role-coverage")).toContainText("Prompting");
    await expect(
      page.getByText("This is the top level of the role."),
    ).toBeVisible();
  });

  test("requires a skill and removes one", async ({ page }) => {
    const writes: string[] = [];
    await record(page, `**/api/proxy/role-profiles/${R1}/requirements`, writes);
    await record(
      page,
      `**/api/proxy/role-profiles/${R1}/requirements/${PROMPTING}`,
      writes,
    );
    await page.goto("/roles");
    await page.getByTestId("role-req-skill").selectOption(PROMPTING);
    await page.getByTestId("role-req-level").selectOption("4");
    await page.getByTestId("role-req-importance").selectOption("critical");
    await page.getByTestId("role-req-add").click();
    await expect.poll(() => writes.length).toBe(1);
    expect(writes[0]).toBe(
      `PUT /role-profiles/${R1}/requirements {"skill_pid":"${PROMPTING}","min_proficiency":4,"importance":"critical"}`,
    );
    await page
      .getByTestId("role-requirements")
      .getByRole("button", { name: "Remove" })
      .click();
    await expect.poll(() => writes.length).toBe(2);
    expect(writes[1]).toBe(
      `DELETE /role-profiles/${R1}/requirements/${PROMPTING}`,
    );
  });

  test("a refused requirement is shown", async ({ page }) => {
    await page.route(
      `**/api/proxy/role-profiles/${R1}/requirements`,
      (route) =>
        route.request().method() === "GET"
          ? route.fallback()
          : route.fulfill({
              status: 422,
              json: { description: "minimum must be 1 to 5" },
            }),
    );
    await page.goto("/roles");
    await page.getByTestId("role-req-skill").selectOption(PROMPTING);
    await page.getByTestId("role-req-add").click();
    await expect(page.getByTestId("error")).toContainText(
      "minimum must be 1 to 5",
    );
  });

  test("starts a draft profile from an ESCO occupation, then shows it", async ({
    page,
  }) => {
    const writes: string[] = [];
    const NEW = "dddddddd-dddd-4ddd-8ddd-dddddddddddd";
    await json(page, "**/api/proxy/capability-frameworks", [
      {
        slug: "esco",
        name: "ESCO",
        source_url: null,
        licence: null,
        attribution: null,
        scale_max: 5,
        scale_labels: null,
        imported_on: "2026-10-01",
        note: null,
        profiles: 0,
        roles: 0,
      },
    ]);
    await json(page, "**/api/proxy/esco/occupations**", [
      {
        uri: "http://data.europa.eu/esco/occupation/1",
        label: "customer support agent",
        isco_code: "4222",
        essential_skills: 12,
        optional_skills: 8,
      },
    ]);
    await json(page, "**/api/proxy/esco/occupation?**", {
      uri: "http://data.europa.eu/esco/occupation/1",
      label: "customer support agent",
      isco_code: "4222",
      description: "Answers customers.",
      skills: [
        {
          uri: "s1",
          label: "handle complaints",
          relation: "essential",
          catalogue_skill_pid: null,
        },
      ],
    });
    await record(page, "**/api/proxy/role-profiles/from-esco", writes, {
      pid: NEW,
      job_title: "customer support agent",
      skills_created: 1,
      skills_linked: 0,
    });
    await page.goto("/roles");
    await page.getByTestId("esco-query").fill("support");
    await page.getByTestId("esco-search").click();
    await page
      .getByTestId("esco-hits")
      .getByRole("button", { name: "customer support agent" })
      .click();
    await expect(page.getByText("handle complaints")).toBeVisible();
    await page.getByTestId("esco-level").selectOption("2");
    await page.getByTestId("esco-create").click();
    await expect.poll(() => writes.length).toBe(1);
    expect(writes[0]).toContain(
      '"occupation_uri":"http://data.europa.eu/esco/occupation/1"',
    );
    expect(writes[0]).toContain('"default_min_proficiency":2');
    // The form closes and the page reads the profiles again.
    await expect(page.getByTestId("esco-hits")).toHaveCount(0);
  });

  test("starts a draft profile from SFIA skills at levels SFIA defines, kept on SFIA's scale", async ({
    page,
  }) => {
    const writes: string[] = [];
    const NEW = "dddddddd-dddd-4ddd-8ddd-dddddddddddd";
    await json(page, "**/api/proxy/capability-frameworks", [
      {
        slug: "sfia",
        name: "SFIA",
        source_url: null,
        licence: null,
        attribution: null,
        scale_max: 7,
        scale_labels: null,
        imported_on: "2026-10-27",
        note: null,
        profiles: 0,
        roles: 0,
      },
    ]);
    await json(page, "**/api/proxy/sfia/skills?**", [
      {
        code: "ZZZ1",
        name: "Synthetic widget tuning",
        category: "Invented",
        subcategory: "Sub",
        retired_in: null,
        levels: [2, 3, 5],
      },
    ]);
    await record(page, "**/api/proxy/role-profiles/from-sfia", writes, {
      pid: NEW,
      requirements_created: 1,
      skills_created: 1,
      skills_linked: 0,
    });
    await page.goto("/roles");
    await page.getByTestId("sfia-query").fill("widget");
    await page.getByTestId("sfia-search").click();
    await page.getByTestId("sfia-level-ZZZ1").selectOption("5");
    await page.getByTestId("sfia-add-ZZZ1").click();
    await expect(page.getByTestId("sfia-picked")).toContainText(
      "ZZZ1 Synthetic widget tuning",
    );
    await page.getByTestId("sfia-title").fill("Widget tuner");
    await page.getByTestId("sfia-category").selectOption("domain");
    await page.getByTestId("sfia-create").click();
    await expect.poll(() => writes.length).toBe(1);
    expect(writes[0]).toContain('"job_title":"Widget tuner"');
    expect(writes[0]).toContain('"skills":[{"code":"ZZZ1","level":5}]');
    expect(writes[0]).toContain('"category":"domain"');
    await expect(page.getByTestId("sfia-picked")).toHaveCount(0);
  });

  test("a requirement stated on SFIA shows its own level, the mapping and the wording", async ({
    page,
  }) => {
    await json(page, `**/api/proxy/role-profiles/${R1}`, {
      pid: R1,
      job_title: "Support Agent",
      description: null,
      source_ref: null,
      framework: null,
      profession: null,
      role_name: null,
      level_name: null,
      level_order: null,
      management_track: false,
      requirements: [
        {
          skill_pid: PROMPTING,
          skill: "Prompting",
          category: "technical",
          min_proficiency: 3,
          importance: "important",
          note: null,
          source_framework: "sfia",
          source_framework_name: "SFIA — Skills Framework",
          source_level: 4,
          source_scale_max: 7,
          mapping: "linear",
          native_description: "Invented words for level four",
        },
      ],
    });
    await page.goto("/roles");
    const table = page.getByTestId("role-requirements");
    await expect(table).toContainText(
      "4 of 7 on SFIA — Skills Framework, counted as 3 of 5 (linear)",
    );
    await expect(page.getByTestId("role-requirement-wording")).toContainText(
      "Invented words for level four",
    );
  });
});
