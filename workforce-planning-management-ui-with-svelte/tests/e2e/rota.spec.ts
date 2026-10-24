// On-call rota. The API is stubbed per test; an unstubbed /api/proxy call 404s loudly.

import { expect, test } from "@playwright/test";
import { MASKED_WORKER, WORKER, stubSignedIn } from "./support";

test.describe("On-call rota", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
  });

  test("on-call rota shows who is on call, why, and takes a swap", async ({
    page,
  }) => {
    const rotaPid = "aaaaaaaa-0000-4000-8000-00000000a001";
    const swaps: Array<Record<string, unknown>> = [];
    const view = () => ({
      pid: rotaPid,
      organization_ref: WORKER.organization_ref,
      name: "Platform on-call",
      description: null,
      period_days: 7,
      starts_on: "2026-10-05",
      members: [
        {
          position: 1,
          worker_pid: WORKER.pid,
          name: WORKER.display_name,
          job_title: "Engineer",
        },
        {
          position: 2,
          worker_pid: MASKED_WORKER.pid,
          name: MASKED_WORKER.display_name,
          job_title: "Engineer",
        },
      ],
      overrides: swaps,
      window: { from: "2026-10-05", to: "2026-11-01" },
      runs: [
        {
          from: "2026-10-05",
          to: "2026-10-11",
          worker_pid: WORKER.pid,
          worker_name: WORKER.display_name,
          source: "rotation",
        },
        {
          from: "2026-10-12",
          to: "2026-10-18",
          worker_pid: WORKER.pid,
          worker_name: WORKER.display_name,
          source: "skipped",
        },
        {
          from: "2026-10-19",
          to: "2026-10-25",
          worker_pid: null,
          worker_name: null,
          source: null,
        },
      ],
      load: [{ worker_pid: WORKER.pid, name: WORKER.display_name, days: 14 }],
      on_call_today: { worker_pid: WORKER.pid, name: WORKER.display_name },
    });
    await page.route("**/api/proxy/rotas", (route) =>
      route.fulfill({
        json: [
          {
            pid: rotaPid,
            organization_ref: WORKER.organization_ref,
            name: "Platform on-call",
            description: null,
            period_days: 7,
            starts_on: "2026-10-05",
            members: 2,
            on_call_today: {
              worker_pid: WORKER.pid,
              name: WORKER.display_name,
            },
          },
        ],
      }),
    );
    await page.route(`**/api/proxy/rotas/${rotaPid}`, (route) =>
      route.fulfill({ json: view() }),
    );
    await page.route(`**/api/proxy/rotas/${rotaPid}/overrides`, (route) => {
      swaps.push({
        pid: "s1",
        worker_name: MASKED_WORKER.display_name,
        note: null,
        ...route.request().postDataJSON(),
      });
      return route.fulfill({ json: { pid: "s1" } });
    });

    const asked: Array<Record<string, unknown>> = [];
    await page.route(`**/api/proxy/rotas/${rotaPid}/swap-requests`, (route) => {
      if (route.request().method() === "POST") {
        asked.push({
          pid: "q1",
          rota_pid: rotaPid,
          rota_name: "Platform on-call",
          requester_name: WORKER.display_name,
          taker_name: MASKED_WORKER.display_name,
          note: null,
          status: "requested",
          ...route.request().postDataJSON(),
        });
        return route.fulfill({ json: { pid: "q1" } });
      }
      return route.fulfill({ json: asked });
    });

    await page.goto("/rota");
    await expect(page.getByTestId("on-call-now")).toContainText(
      WORKER.display_name,
    );
    const runs = page.getByTestId("rota-runs");
    await expect(runs).toContainText("Rotation");
    await expect(runs).toContainText("Covering for someone away");
    // A stretch nobody can take is shown plainly, not guessed.
    await expect(runs).toContainText("Nobody is available");

    await page.getByTestId("swap-worker").selectOption(MASKED_WORKER.pid);
    const swapForm = page.getByTestId("swap-form");
    await swapForm.getByLabel("From").fill("2026-10-05");
    await swapForm.getByLabel("To").fill("2026-10-06");
    await page.getByTestId("swap-add").click();
    await expect(page.getByTestId("rota-swaps")).toContainText(
      MASKED_WORKER.display_name,
    );

    // Ask a colleague to take your on-call days.
    await expect(page.getByTestId("rota-requests")).toContainText(
      "No open swap requests.",
    );
    await page.getByTestId("ask-requester").selectOption(WORKER.pid);
    await page.getByTestId("ask-taker").selectOption(MASKED_WORKER.pid);
    const form = page.getByTestId("swap-request-form");
    await form.getByLabel("From").fill("2026-10-05");
    await form.getByLabel("To").fill("2026-10-11");
    await page.getByTestId("swap-request-add").click();
    await expect(page.getByTestId("rota-requests")).toContainText("requested");
    await expect(page.getByTestId("rota-requests")).toContainText(
      MASKED_WORKER.display_name,
    );
  });
});
