// A person's declared skills (/me): a level stated on SFIA is kept as stated, counted on WPM's 1–5 by the stated rule, and shown
// with the framework's name and wording. The API is stubbed per test; an unstubbed /api/proxy call 404s loudly.

import { expect, test } from "@playwright/test";
import { WORKER, stubSignedIn } from "./support";

const SKILL = "cccccccc-cccc-4ccc-8ccc-cccccccccccc";
const sfia = { slug: "sfia", name: "SFIA — Skills Framework", source_url: null, licence: null, attribution: null, scale_max: 7, scale_labels: null, imported_on: "2026-10-27", note: null, profiles: 0, roles: 0 };

test.describe("Declared skills", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
    await page.route("**/api/proxy/skills", (route) => route.fulfill({ json: [{ pid: SKILL, name: "Synthetic widget tuning", category: "technical", external_refs: [] }] }));
  });

  test("declares a skill on SFIA's scale and shows the level as stated, the count, the framework's name and its wording", async ({ page }) => {
    let rows: unknown[] = [];
    const sent: Array<Record<string, unknown>> = [];
    await page.route("**/api/proxy/capability-frameworks", (route) => route.fulfill({ json: [sfia] }));
    await page.route(`**/api/proxy/workers/${WORKER.pid}/skills`, async (route) => {
      if (route.request().method() === "PUT") {
        sent.push(route.request().postDataJSON());
        rows = [{ skill_pid: SKILL, proficiency: 4, target: null, assessed_on: "2026-10-27", source_framework: "sfia", source_framework_name: sfia.name, source_level: 5, source_scale_max: 7, mapping: "linear", native_description: "Invented words for level five" }];
        return route.fulfill({ json: rows[0] });
      }
      return route.fulfill({ json: rows });
    });
    await page.goto("/me");
    const panel = page.getByTestId("declared-skills");
    await expect(panel.getByTestId("declared-none")).toBeVisible();
    await panel.getByTestId("declared-skill").selectOption(SKILL);
    await panel.getByTestId("declared-scale").selectOption("sfia");
    await panel.getByTestId("declared-level").selectOption("5");
    await panel.getByTestId("declared-save").click();
    expect(sent.at(-1)).toEqual({ skill_pid: SKILL, framework: "sfia", framework_level: 5 });
    const line = panel.getByTestId(`declared-${SKILL}`);
    await expect(line).toContainText("Synthetic widget tuning");
    await expect(line).toContainText("4 / 5");
    await expect(line).toContainText("5 of 7 on SFIA — Skills Framework, counted as 4 of 5 (linear): Invented words for level five");
    
  });

  test("without SFIA the scale choice is not offered and a plain 1–5 is sent", async ({ page }) => {
    const sent: Array<Record<string, unknown>> = [];
    await page.route("**/api/proxy/capability-frameworks", (route) => route.fulfill({ json: [] }));
    await page.route(`**/api/proxy/workers/${WORKER.pid}/skills`, async (route) => {
      if (route.request().method() === "PUT") {
        sent.push(route.request().postDataJSON());
        return route.fulfill({ json: {} });
      }
      return route.fulfill({ json: [] });
    });
    await page.goto("/me");
    const panel = page.getByTestId("declared-skills");
    await expect(panel.getByTestId("declared-scale")).toHaveCount(0);
    await panel.getByTestId("declared-skill").selectOption(SKILL);
    await panel.getByTestId("declared-level").selectOption("2");
    await panel.getByTestId("declared-save").click();
    await expect.poll(() => sent.length).toBe(1);
    expect(sent[0]).toEqual({ skill_pid: SKILL, proficiency: 2 });
  });
});
