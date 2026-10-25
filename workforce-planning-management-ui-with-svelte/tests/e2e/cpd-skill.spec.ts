// A CPD entry may name the skill it developed, so a credential issued from it carries that skill's ESCO and SFIA references.
// The API is stubbed per test; an unstubbed /api/proxy call 404s loudly.

import { expect, test } from "@playwright/test";

const WORKER = "11111111-1111-4111-8111-111111111111";
const SKILL = "22222222-2222-4222-8222-222222222222";

test("a CPD entry names its skill, the body carries it, and the row shows its name", async ({ page }) => {
  await page.context().addCookies([
    { name: "__Host-mxi_session", value: "cpd-skill-spec", domain: "localhost", path: "/", httpOnly: true, secure: true, sameSite: "Lax" },
  ]);
  const entries: Array<Record<string, unknown>> = [];
  let posted: Record<string, unknown> | null = null;
  await page.route("**/api/proxy/**", (route) => route.fulfill({ status: 404, body: "unstubbed: " + route.request().url() }));
  await page.route("**/api/proxy/workers", (route) => route.fulfill({ json: [{ pid: WORKER, display_name: "Test Worker 001" }] }));
  await page.route("**/api/proxy/skills", (route) =>
    route.fulfill({ json: [{ pid: SKILL, name: "Data analysis", category: "technical", external_refs: [] }] }),
  );
  await page.route("**/api/proxy/cpd/overview", (route) =>
    route.fulfill({ json: { derivation: "", headcount: 1, requirements: [], registrations: { expiring: 0, expired: 0 } } }),
  );
  await page.route("**/api/proxy/cpd-requirements", (route) => route.fulfill({ json: [] }));
  await page.route(`**/api/proxy/workers/${WORKER}/cpd-progress`, (route) =>
    route.fulfill({ json: { derivation: "", worker_pid: WORKER, registrations: [], requirements: [] } }),
  );
  await page.route(`**/api/proxy/workers/${WORKER}/cpd-entries`, (route) => {
    if (route.request().method() === "POST") {
      posted = route.request().postDataJSON();
      entries.push({
        pid: "e1", entry_date: "2026-09-01", category: "course", unit: "hours", amount: 2, amount_hundredths: 200,
        evidence_note: null, evidence_url: null, source: "manual", lms_url: null, verified_on: null, verified_by: null,
        skill_pid: null, ...posted,
      });
      return route.fulfill({ json: { pid: "e1" } });
    }
    return route.fulfill({ json: entries });
  });
  await page.route("**/api/proxy/calendar**", (route) => route.fulfill({ json: { events: [] } }));

  await page.goto("/en-001/cpd");
  await page.getByTestId("cpd-worker").selectOption(WORKER);
  await page.getByTestId("cpd-entry-activity").fill("SQL course");
  await page.getByTestId("cpd-entry-skill-pick").selectOption(SKILL);
  await page.getByTestId("cpd-entry-add").click();
  await expect(page.getByTestId("cpd-entry-skill")).toContainText("Data analysis");
  expect(posted).toMatchObject({ activity: "SQL course", skill_pid: SKILL });

  // with no skill chosen the body carries none
  await page.getByTestId("cpd-entry-activity").fill("Reading");
  await page.getByTestId("cpd-entry-add").click();
  await expect.poll(() => (posted as Record<string, unknown>).activity).toBe("Reading");
  expect(posted).not.toHaveProperty("skill_pid");
});
