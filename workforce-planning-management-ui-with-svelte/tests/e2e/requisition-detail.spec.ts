// A requisition's page (`/requisitions/[pid]`): its applications and the stage each can move to; hiring
// needs a worker number. The API is stubbed per test; an unstubbed /api/proxy call 404s loudly.

import { expect, test } from "@playwright/test";
import { WORKER, stubSignedIn } from "./support";

const REQUISITION = {
  pid: "55555555-5555-4555-8555-555555555555",
  organization_ref: WORKER.organization_ref,
  department: "engineering",
  job_title: "Platform Engineer",
  headcount: 1,
  salary_min_minor: null,
  salary_max_minor: null,
  salary_currency: null,
  status: "open",
  opened_on: "2026-06-01",
};
const application = (n: number, stage: string) => ({
  pid: `${n}${n}${n}${n}${n}${n}${n}${n}-${n}${n}${n}${n}-4${n}${n}${n}-8${n}${n}${n}-${String(n).repeat(12)}`,
  requisition_pid: REQUISITION.pid,
  candidate_pid: `${String(n).repeat(8)}-cccc-4ccc-8ccc-cccccccccccc`,
  stage,
  notes: null,
});

test.describe("Requisition detail", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
    await page.route(`**/api/proxy/requisitions/${REQUISITION.pid}`, (route) =>
      route.fulfill({ json: REQUISITION }),
    );
  });

  test("lists the applications, moves one forward a stage, and hires from an offer with a worker number", async ({
    page,
  }) => {
    const received = application(1, "received");
    const offer = application(2, "offer");
    const moves: Array<{ url: string; body: unknown }> = [];
    await page.route(
      `**/api/proxy/requisitions/${REQUISITION.pid}/applications`,
      (route) => route.fulfill({ json: [received, offer] }),
    );
    for (const a of [received, offer]) {
      await page.route(`**/api/proxy/applications/${a.pid}/stage`, (route) => {
        moves.push({
          url: route.request().url(),
          body: route.request().postDataJSON(),
        });
        return route.fulfill({ json: { ...a } });
      });
    }
    await page.goto(`/requisitions/${REQUISITION.pid}`);
    await expect(page).toHaveTitle("Vacancies — WPM");
    await expect(page.getByRole("heading", { level: 1 })).toContainText(
      "Platform Engineer",
    );
    const table = page.getByTestId("applications");
    await expect(table).toContainText("11111111");
    await expect(table).toContainText("22222222");

    await table.getByRole("button", { name: "→ screened" }).click();
    await expect.poll(() => moves.length).toBe(1);
    expect(moves[0]?.body).toEqual({ to: "screened" });

    // An offer is hired with a worker number, and only the offer row offers that.
    await expect(table.getByRole("textbox")).toHaveCount(1);
    await table
      .getByRole("textbox", { name: "Worker number for the hire" })
      .fill("E-0042");
    await table.getByRole("button", { name: "→ hired" }).click();
    await expect.poll(() => moves.length).toBe(2);
    expect(moves[1]?.body).toEqual({ to: "hired", worker_number: "E-0042" });
  });

  test("a refused move is shown beside the table", async ({ page }) => {
    const offer = application(2, "offer");
    await page.route(
      `**/api/proxy/requisitions/${REQUISITION.pid}/applications`,
      (route) => route.fulfill({ json: [offer] }),
    );
    await page.route(`**/api/proxy/applications/${offer.pid}/stage`, (route) =>
      route.fulfill({
        status: 422,
        json: { description: "worker number E-0042 is already taken" },
      }),
    );
    await page.goto(`/requisitions/${REQUISITION.pid}`);
    await page
      .getByRole("textbox", { name: "Worker number for the hire" })
      .fill("E-0042");
    await page.getByRole("button", { name: "→ hired" }).click();
    // The catalogue's words, not the service's.
    await expect(page.getByTestId("action-error")).toContainText(
      "That was not accepted",
    );
    await expect(page.getByTestId("action-error")).not.toContainText(
      "already taken",
    );
  });

  test("a requisition that cannot be read shows the error and no table", async ({
    page,
  }) => {
    await page.route(`**/api/proxy/requisitions/${REQUISITION.pid}`, (route) =>
      route.fulfill({ status: 404, json: { description: "not found" } }),
    );
    await page.route(
      `**/api/proxy/requisitions/${REQUISITION.pid}/applications`,
      (route) => route.fulfill({ json: [] }),
    );
    await page.goto(`/requisitions/${REQUISITION.pid}`);
    await expect(page.getByTestId("error")).toContainText("not found");
    await expect(page.getByTestId("applications")).toHaveCount(0);
  });
});
