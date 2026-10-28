// What a worker's page lets someone do in the panels that were split out of it (WPM-T267): answer a health
// prompt, send a pulse score, ask for an adjustment, record an ergonomic assessment, run a 360. Each write is
// recorded and its body checked. The API is stubbed per test; an unstubbed /api/proxy call 404s loudly.

import { expect, test, type Page } from "@playwright/test";
import { WORKER } from "./support";
import { base, json, stubWorkerPage } from "./fixtures";

const W = `**/api/proxy/workers/${WORKER.pid}`;

/** Record every POST under a pattern as "path body" and answer it. */
async function record(
  page: Page,
  pattern: string,
  writes: string[],
  answer: unknown = { pid: "new" },
) {
  await page.route(pattern, (route) => {
    if (route.request().method() === "GET") return route.fallback();
    writes.push(
      `${new URL(route.request().url()).pathname.split("/proxy")[1]} ${route.request().postData() ?? ""}`.trim(),
    );
    return route.fulfill({ json: answer });
  });
}

test.describe("Worker page panels", () => {
  test.beforeEach(async ({ page }) => {
    await base(page);
    await stubWorkerPage(page);
  });

  test("answers a health prompt and sends an anonymous pulse score", async ({
    page,
  }) => {
    const writes: string[] = [];
    await record(page, `${W}/wellbeing-acknowledgements`, writes);
    await record(page, "**/api/proxy/pulse-surveys/s1/responses", writes, {});
    await page.goto(`/workers/${WORKER.pid}`);
    await page
      .getByTestId("wellbeing-prompts")
      .getByRole("button", { name: "I've booked it" })
      .click();
    await expect.poll(() => writes.length).toBe(1);
    expect(writes[0]).toBe(
      `/workers/${WORKER.pid}/wellbeing-acknowledgements {"entitlement_pid":"p1","response":"booked"}`,
    );
    await page
      .getByTestId("pulse-surveys")
      .getByRole("button", { name: "4", exact: true })
      .click();
    await expect(page.getByTestId("pulse-surveys")).toContainText("Thanks");
    expect(writes[1]).toBe(
      `/pulse-surveys/s1/responses {"worker_pid":"${WORKER.pid}","score":4}`,
    );
  });

  test("a refused answer is shown beside the worker's facts", async ({
    page,
  }) => {
    await page.route(`${W}/wellbeing-acknowledgements`, (route) =>
      route.fulfill({ status: 422, json: { description: "already answered" } }),
    );
    await page.goto(`/workers/${WORKER.pid}`);
    await page
      .getByTestId("wellbeing-prompts")
      .getByRole("button", { name: "Done" })
      .click();
    await expect(page.getByTestId("action-error")).toContainText(
      "already answered",
    );
  });

  test("asks for an adjustment only once all three answers are in, and decides one", async ({
    page,
  }) => {
    const writes: string[] = [];
    await record(page, `${W}/adjustment-requests`, writes);
    await record(
      page,
      "**/api/proxy/adjustment-requests/j1/status",
      writes,
      {},
    );
    page.on(
      "dialog",
      (dialog) => void dialog.accept("Agreed with the manager"),
    );
    await page.goto(`/workers/${WORKER.pid}`);
    const panel = page.getByTestId("adjustments");
    const ask = panel.getByRole("button", { name: "Ask for a change" });
    await expect(ask).toBeDisabled();
    await panel.getByLabel("What barrier do you face?").fill("Noise");
    await panel
      .getByLabel("How does it affect your work?")
      .fill("Cannot focus");
    await expect(ask).toBeDisabled();
    await panel.getByLabel("What change would reduce it?").fill("A quiet desk");
    await ask.click();
    await expect.poll(() => writes.length).toBe(1);
    expect(JSON.parse(writes[0]!.split(" ").slice(1).join(" "))).toEqual({
      category: "quieter_workspace",
      barrier: "Noise",
      impact: "Cannot focus",
      adjustment: "A quiet desk",
    });
    await panel.getByRole("button", { name: "Agree" }).click();
    await expect.poll(() => writes.length).toBe(2);
    expect(writes[1]).toContain('"to":"agreed"');
    expect(writes[1]).toContain("Agreed with the manager");
  });

  test("records an ergonomic assessment and completes an open one", async ({
    page,
  }) => {
    const writes: string[] = [];
    await record(page, `${W}/ergonomic-assessments`, writes);
    await record(
      page,
      "**/api/proxy/ergonomic-assessments/g1/complete",
      writes,
      {},
    );
    await page.goto(`/workers/${WORKER.pid}`);
    const panel = page.getByTestId("ergonomics");
    const create = panel.getByRole("button", { name: "New assessment" });
    await expect(create).toBeDisabled();
    await panel.getByLabel("Workstation").fill("Desk 9");
    await create.click();
    await expect.poll(() => writes.length).toBe(1);
    expect(writes[0]).toContain('"workstation":"Desk 9"');
    await expect(panel).toContainText("Desk 4");
    await panel.getByRole("button", { name: "Complete" }).click();
    await expect.poll(() => writes.length).toBe(2);
  });

  test("starts a 360, opens it to nominate a rater, and moves it to collecting", async ({
    page,
  }) => {
    const writes: string[] = [];
    // A colleague to nominate: the person themself is not offered.
    await json(page, "**/api/proxy/workers", [
      WORKER,
      {
        ...WORKER,
        pid: "44444444-4444-4444-8444-444444444444",
        display_name: "Colleague One",
      },
    ]);
    await json(page, `${W}/appraisals`, [
      {
        pid: "a1",
        status: "draft",
        competencies: ["communication", "delivery"],
        shared_on: null,
        nominated: 0,
        responded: 0,
      },
    ]);
    await record(page, `${W}/appraisals`, writes);
    await record(page, "**/api/proxy/appraisals/a1/nominations", writes);
    await record(page, "**/api/proxy/appraisals/a1/status", writes, {});
    await json(page, "**/api/proxy/appraisals/a1", {
      pid: "a1",
      worker_pid: WORKER.pid,
      status: "draft",
      competencies: ["communication", "delivery"],
      shared_on: null,
      nominations: [],
    });
    await page.goto(`/workers/${WORKER.pid}`);
    const panel = page.getByTestId("appraisals");
    await panel.getByRole("button", { name: "New 360" }).click();
    await expect.poll(() => writes.length).toBe(1);
    expect(writes[0]).toContain(
      '"competencies":["communication","collaboration","delivery"]',
    );

    await panel
      .getByRole("button", { name: /communication · delivery/ })
      .click();
    const nominate = panel.getByRole("button", { name: "Nominate" });
    await expect(nominate).toBeDisabled();
    await panel
      .getByLabel("Colleague to nominate")
      .selectOption({ label: "Colleague One" });
    await panel
      .getByLabel("Relationship to the person")
      .selectOption("manager");
    await nominate.click();
    await expect.poll(() => writes.length).toBe(2);
    expect(writes[1]).toContain('"group":"manager"');

    await panel.getByRole("button", { name: "Start collecting" }).click();
    await expect.poll(() => writes.length).toBe(3);
    expect(writes[2]).toContain('"to":"collecting"');
  });

  test("answers a request to rate someone else", async ({ page }) => {
    const writes: string[] = [];
    await record(page, "**/api/proxy/appraisals/a9/responses", writes, {});
    await page.goto(`/workers/${WORKER.pid}`);
    const requests = page.getByTestId("appraisal-requests");
    await requests.getByRole("button", { name: "Respond" }).click();
    await requests.getByLabel("Comment (optional)").fill("Clear and kind");
    await requests.getByRole("button", { name: "Respond" }).click();
    await expect.poll(() => writes.length).toBe(1);
    expect(writes[0]).toContain('"comment":"Clear and kind"');
    expect(writes[0]).toContain('"communication":3');
  });
});
