// SFIA in role profiles and declared skills against the real service (see README.md). The catalogue is loaded first with the
// real task, from the invented fixtures (nothing here is SFIA text):
//   cargo run -- task import_sfia file:tests/fixtures/sfia-synthetic/skills.csv levels_file:tests/fixtures/sfia-synthetic/levels.csv \
//     version:9 licence_checked_on:2026-10-27
// Enforcement is off. Each run uses its own worker, skill and job title. `PAGE_SHOTS=<dir>` also writes screenshots.

import { expect, test } from "@playwright/test";

test.skip(!process.env.WPM_LIVE_API, "needs a running service: set WPM_LIVE_API=1");

const api = "http://localhost:5150/api";
const org = `organization:${crypto.randomUUID()}`;
const tag = crypto.randomUUID().slice(0, 6);
const shots = process.env.PAGE_SHOTS;

test("start a role from SFIA skills, then declare a skill on SFIA's scale: levels kept, mapped, named and worded", async ({ page, request }) => {
  await page.context().addCookies([
    { name: "__Host-mxi_session", value: "live-test-session", domain: "localhost", path: "/", httpOnly: true, secure: true, sameSite: "Lax" },
  ]);
  await page.route("**/api/proxy/me/organizations/scope", (route) => route.fulfill({ json: [org] }));

  // The role: ZZZ1 at SFIA level 3 (counted as 2 of 5), in the category the planner chooses.
  await page.goto("/roles");
  await page.getByTestId("sfia-query").fill("widget");
  await page.getByTestId("sfia-search").click();
  await page.getByTestId("sfia-level-ZZZ1").selectOption("3");
  await page.getByTestId("sfia-add-ZZZ1").click();
  await page.getByTestId("sfia-title").fill(`Widget tuner ${tag}`);
  await page.getByTestId("sfia-category").selectOption("domain");
  await page.getByTestId("sfia-create").click();
  await expect(page.getByTestId("role-select")).toHaveValue(/.+/); // the new profile is the one selected
  await expect(page.getByTestId("role-select").locator("option:checked")).toContainText(`Widget tuner ${tag}`);
  const requirements = page.getByTestId("role-requirements");
  await expect(requirements).toContainText("3 of 7 on SFIA — Skills Framework for the Information Age, counted as 2 of 5 (linear)");
  await expect(page.getByTestId("role-requirement-wording")).toContainText("Invented text: tunes widgets unaided.");
  if (shots) await page.screenshot({ path: `${shots}/sfia-roles.png`, fullPage: true });

  // The skill the role created is in the planner's category, and linked by its SFIA code.
  const skills = (await (await request.get(`${api}/skills`)).json()) as Array<{ pid: string; name: string; category: string; external_refs: Array<{ framework: string; ref: string }> }>;
  const skill = skills.find((s) => s.external_refs.some((r) => r.framework === "sfia" && r.ref === "ZZZ1"));
  expect(skill?.category).toBe("domain");

  // A worker declares it on SFIA's scale from their page.
  const worker = await (
    await request.post(`${api}/workers`, {
      data: {
        person_ref: `person:${crypto.randomUUID()}`, organization_ref: org, worker_number: `S${tag}`, display_name: `Sfia ${tag}`,
        employment_type: "permanent", fte_percent: 100, department: "Care", job_title: "Carer", hired_on: "2024-01-01",
      },
    })
  ).json();
  await page.goto(`/workers/${worker.pid}`);
  const panel = page.getByTestId("declared-skills");
  await panel.getByTestId("declared-skill").selectOption(skill!.pid);
  await panel.getByTestId("declared-scale").selectOption("sfia");
  await panel.getByTestId("declared-level").selectOption("5");
  await panel.getByTestId("declared-save").click();
  const line = panel.getByTestId(`declared-${skill!.pid}`);
  await expect(line).toContainText("4 / 5");
  await expect(line).toContainText("5 of 7 on SFIA — Skills Framework for the Information Age, counted as 4 of 5 (linear): Invented text: sets the tuning policy.");
  await expect(line).not.toContainText("))");
  if (shots) await panel.screenshot({ path: `${shots}/sfia-declared.png` });

  // A level SFIA does not define for the skill is refused, in words.
  await panel.getByTestId("declared-level").selectOption("4");
  await panel.getByTestId("declared-save").click();
  await expect(panel.getByTestId("error")).toContainText("not defined at level 4");
});
