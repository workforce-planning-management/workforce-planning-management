// The information pack over a unit that meets the floor, and the decision-gate pointer on the AI register, against the real service
// (see README.md). Workers, a body, a unit and a gate are made through the API (enforcement is off); the pages are driven in the UI.
// Needs no seed data: each run uses its own organization. It records feature S1 on the AI register, so it changes that data.

import { expect, test } from "@playwright/test";

test.skip(!process.env.WPM_LIVE_API, "needs a running service: set WPM_LIVE_API=1");

const api = "http://localhost:5150/api";
const org = `organization:${crypto.randomUUID()}`;

test("a pack over twelve people shows counts, never a name; recording a score names the gate to consult", async ({ page, request }) => {
  for (let n = 0; n < 12; n++) {
    const made = await request.post(`${api}/workers`, {
      data: {
        person_ref: `person:${crypto.randomUUID()}`, organization_ref: org, worker_number: `W${n}`, display_name: `Worker ${n}`,
        employment_type: "permanent", fte_percent: 100, department: "Care", job_title: "Carer", hired_on: "2024-01-01",
      },
    });
    expect(made.ok(), await made.text()).toBe(true);
  }
  const body = await (
    await request.post(`${api}/recognised-bodies`, {
      data: { organization_ref: org, name: "Care Union", kind: "union", basis: "other", recognised_on: "2020-03-01", contact_role: "HR lead" },
    })
  ).json();
  const unit = await request.post(`${api}/recognised-bodies/${body.pid}/units`, { data: { name: "Care staff", department: "Care" } });
  expect(unit.ok(), await unit.text()).toBe(true);
  const gate = await request.post(`${api}/decision-gates`, {
    data: { organization_ref: org, change_kind: "new_or_changed_score", body_pid: body.pid },
  });
  expect(gate.ok(), await gate.text()).toBe(true);

  await page.context().addCookies([
    { name: "__Host-mxi_session", value: "live-test-session", domain: "localhost", path: "/", httpOnly: true, secure: true, sameSite: "Lax" },
  ]);
  await page.route("**/api/proxy/me/organizations/scope", (route) => route.fulfill({ json: [org] }));

  await page.goto("/bargaining");
  await page.getByTestId("bg-show-units").click();
  await expect(page.getByTestId("bg-coverage")).toHaveText("12 covered");
  await page.getByTestId("bg-pack").click();
  const pack = page.getByTestId("bg-pack-result");
  await expect(pack).toContainText("12 people covered");
  await expect(pack).toContainText("Care: 12");
  await expect(pack).not.toContainText("Worker 0"); // no name, ever

  await page.goto("/ai-register");
  await page.getByTestId("ai-edit").first().click();
  await page.getByTestId("ai-overseer").fill("Head of Mobility");
  await page.getByTestId("ai-save").click();
  await expect(page.getByTestId("ai-gate")).toContainText("to consult");
  await expect(page.getByTestId("ai-gate")).toContainText("section 11");
});
