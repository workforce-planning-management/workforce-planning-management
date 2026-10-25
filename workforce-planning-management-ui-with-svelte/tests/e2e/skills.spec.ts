// Skills catalogue (`/skills`): search, rename, recategorise, accept keyword-rule category suggestions,
// merge and delete. The API is stubbed per test; an unstubbed /api/proxy call 404s loudly.

import { expect, test, type Page } from "@playwright/test";
import { stubSignedIn } from "./support";

const SKILLS = [
  { pid: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa", name: "Rust", category: "technical", external_refs: [] },
  { pid: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb", name: "Rustlang", category: "other", external_refs: [] },
  {
    pid: "cccccccc-cccc-4ccc-8ccc-cccccccccccc",
    name: "Coaching",
    category: "leadership",
    external_refs: [{ framework: "esco", ref: "http://data.europa.eu/esco/skill/x", label: "coach people", version: null }],
  },
];

/** The skills endpoints; every write is recorded as "METHOD path body". */
async function stubSkills(page: Page, writes: string[]) {
  // Registered first: a later route wins, so the specific ones below override this catch-all.
  await page.route("**/api/proxy/skills/*", (route) => {
    const request = route.request();
    if (request.method() === "GET") return route.fulfill({ status: 404, body: "unstubbed" });
    writes.push(`${request.method()} ${new URL(request.url()).pathname.split("/proxy")[1]} ${request.postData() ?? ""}`.trim());
    if (request.url().endsWith("/merge")) {
      return route.fulfill({ json: { records_moved: 3, records_merged: 1 } });
    }
    return route.fulfill({ json: {} });
  });  await page.route("**/api/proxy/skills", (route) => route.fulfill({ json: SKILLS }));
  await page.route("**/api/proxy/skill-mappings", (route) => route.fulfill({ json: { note: "", mappings: [] } }));
  await page.route("**/api/proxy/skills/*/merge", (route) => {
    writes.push(`POST ${new URL(route.request().url()).pathname.split("/proxy")[1]} ${route.request().postData()}`);
    return route.fulfill({ json: { merged_into: SKILLS[0]?.pid, records_moved: 3, records_merged: 1 } });
  });
  await page.route("**/api/proxy/skills/category-suggestions", (route) =>
    route.fulfill({
      json: {
        derivation: "Keyword rules over the skill name.",
        suggestions: [{ pid: SKILLS[1]?.pid, name: "Rustlang", suggested: "technical", keyword: "rust" }],
        unsuggested: 1,
      },
    }),
  );
  await page.route("**/api/proxy/skills/category-suggestions/apply", (route) => {
    writes.push(`POST apply ${route.request().postData()}`);
    return route.fulfill({ json: { applied: 1, skipped: 0 } });
  });

}

test.describe("Skills catalogue", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
  });

  test("counts the catalogue by category, filters by search and category", async ({ page }) => {
    await stubSkills(page, []);
    await page.goto("/skills");
    await expect(page).toHaveTitle("Skills — WPM");
    await expect(page.getByTestId("skill-counts")).toContainText("3");
    const table = page.getByTestId("skill-table");
    await expect(table.getByRole("row")).toHaveCount(4); // header + three skills
    await page.getByTestId("skill-search").fill("coach");
    await expect(table).toContainText("Coaching");
    await expect(table).not.toContainText("Rustlang");
    await expect(table).toContainText("esco: coach people");
    await page.getByTestId("skill-search").fill("zzz");
    await expect(table).toContainText("No skills match");
  });

  test("renames a skill and changes its category", async ({ page }) => {
    const writes: string[] = [];
    await stubSkills(page, writes);
    await page.goto("/skills");
    const row = page.getByRole("row", { name: /Rustlang/ });
    await row.getByRole("button", { name: "Rename" }).click();
    await page.getByLabel("Rename Rustlang").fill("Rust language");
    await page.getByRole("button", { name: "Save" }).click();
    await expect.poll(() => writes.length).toBe(1);
    expect(writes[0]).toBe(`PUT /api/proxy/skills/${SKILLS[1]?.pid} {"name":"Rust language"}`.replace("/api/proxy", ""));
    await page.getByLabel("Category of Rustlang").selectOption("technical");
    await expect.poll(() => writes.length).toBe(2);
    expect(writes[1]).toContain('{"category":"technical"}');
  });

  test("accepts one category suggestion, or all of them", async ({ page }) => {
    const writes: string[] = [];
    await stubSkills(page, writes);
    await page.goto("/skills");
    await expect(page.getByTestId("suggestions")).toContainText("Rustlang");
    await expect(page.getByTestId("suggestions")).toContainText("rust");
    await page.getByTestId("apply-all").click();
    await expect.poll(() => writes.length).toBe(1);
    expect(writes[0]).toBe(`POST apply {"skill_pids":["${SKILLS[1]?.pid}"]}`);
  });

  test("merges one skill into another after confirming, and says what moved", async ({ page }) => {
    const writes: string[] = [];
    await stubSkills(page, writes);
    page.on("dialog", (dialog) => void dialog.accept());
    await page.goto("/skills");
    const row = page.getByRole("row", { name: /Rustlang/ });
    await row.getByRole("button", { name: "Merge…" }).click();
    await page.getByLabel("Merge Rustlang into").selectOption({ label: "Rust" });
    await page.getByRole("button", { name: "Merge", exact: true }).click();
    await expect(page.getByTestId("notice")).toContainText("3 moved, 1 combined");
    expect(writes[0]).toContain("/merge");
  });

  test("declining the confirmation deletes nothing", async ({ page }) => {
    const writes: string[] = [];
    await stubSkills(page, writes);
    page.on("dialog", (dialog) => void dialog.dismiss());
    await page.goto("/skills");
    await page.getByRole("row", { name: /Coaching/ }).getByRole("button", { name: "Delete" }).click();
    await page.waitForTimeout(300);
    expect(writes).toEqual([]);
  });

  test("a skill that is in use cannot be deleted, and the service's reason is shown", async ({ page }) => {
    await stubSkills(page, []);
    await page.route(`**/api/proxy/skills/${SKILLS[0]?.pid}`, (route) =>
      route.request().method() === "DELETE"
        ? route.fulfill({ status: 409, json: { description: "2 people declare this skill" } })
        : route.fulfill({ status: 404, body: "unstubbed" }),
    );
    page.on("dialog", (dialog) => void dialog.accept());
    await page.goto("/skills");
    await page.getByRole("row", { name: /^Rust Rename/ }).getByRole("button", { name: "Delete" }).click();
    await expect(page.getByTestId("error")).toContainText("2 people declare this skill");
  });
});
