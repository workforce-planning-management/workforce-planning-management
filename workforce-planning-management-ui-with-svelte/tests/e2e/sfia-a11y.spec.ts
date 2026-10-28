// The SFIA surfaces pass axe's WCAG 2 A and AA rules with data on screen: the start-from-SFIA panel with a pick, a requirement
// stated on SFIA, and the declared-skills panel with a SFIA level. The API is stubbed. Automated rules only.

import AxeBuilder from "@axe-core/playwright";
import { expect, test, type Page } from "@playwright/test";
import { base, json, stubRoles } from "./fixtures";
import { WORKER, stubSignedIn } from "./support";

const R1 = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";
const SKILL = "cccccccc-cccc-4ccc-8ccc-cccccccccccc";
const sfia = { slug: "sfia", name: "SFIA — Skills Framework", source_url: null, licence: null, attribution: null, scale_max: 7, scale_labels: null, imported_on: "2026-10-27", note: null, profiles: 0, roles: 0 };

async function violations(page: Page) {
  const result = await new AxeBuilder({ page }).withTags(["wcag2a", "wcag2aa"]).analyze();
  return result.violations.map((v) => `${v.id}: ${v.nodes.map((n) => n.html).join(" | ")}`);
}

test("/roles with the SFIA panel and a requirement stated on SFIA", async ({ page }) => {
  await base(page);
  await stubRoles(page);
  await json(page, "**/api/proxy/capability-frameworks", [sfia]);
  await json(page, `**/api/proxy/role-profiles/${R1}`, {
    pid: R1, job_title: "Support Agent", description: null, source_ref: null, framework: null, profession: null, role_name: null, level_name: null, level_order: null, management_track: false,
    requirements: [{ skill_pid: SKILL, skill: "Prompting", category: "technical", min_proficiency: 3, importance: "important", note: null, source_framework: "sfia", source_framework_name: sfia.name, source_level: 4, source_scale_max: 7, mapping: "linear", native_description: "Invented words" }],
  });
  await json(page, "**/api/proxy/sfia/skills?**", [{ code: "ZZZ1", name: "Synthetic widget tuning", category: "Invented", subcategory: "Sub", retired_in: null, levels: [2, 3, 5] }]);
  await page.goto("/roles");
  await page.getByTestId("sfia-query").fill("widget");
  await page.getByTestId("sfia-search").click();
  await page.getByTestId("sfia-add-ZZZ1").click();
  await expect(page.getByTestId("sfia-picked")).toBeVisible();
  await expect(page.getByTestId("role-requirements")).toContainText("counted as 3 of 5");
  expect(await violations(page)).toEqual([]);
});

test("/me declared skills with a SFIA level", async ({ page }) => {
  await stubSignedIn(page);
  await json(page, "**/api/proxy/skills", [{ pid: SKILL, name: "Synthetic widget tuning", category: "technical", external_refs: [] }]);
  await json(page, "**/api/proxy/capability-frameworks", [sfia]);
  await json(page, `**/api/proxy/workers/${WORKER.pid}/skills`, [
    { skill_pid: SKILL, proficiency: 4, target: null, assessed_on: "2026-10-27", source_framework: "sfia", source_framework_name: sfia.name, source_level: 5, source_scale_max: 7, mapping: "linear", native_description: "Invented words for level five" },
  ]);
  await page.goto("/me");
  await expect(page.getByTestId(`declared-${SKILL}`)).toBeVisible();
  expect(await violations(page)).toEqual([]);
});
