// Pay information: a worker's own requests on `/me` and HR's queue and answer on `/pay-information`.
// The API is stubbed per test; an unstubbed /api/proxy call 404s loudly.

import { expect, test } from "@playwright/test";
import { stubSignedIn } from "./support";

test.describe("Pay information", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
  });

  test("a worker asks once, sees the due day and no figure, and cannot ask while one waits", async ({
    page,
  }) => {
    let asked = false;
    await page.route("**/api/proxy/me/pay-information-requests", (route) => {
      if (route.request().method() === "POST") {
        asked = true;
        return route.fulfill({ json: { pid: "r1", due_on: "2026-12-25" } });
      }
      return route.fulfill({
        json: asked
          ? [
              {
                pid: "r1",
                asked_on: "2026-10-25",
                due_on: "2026-12-25",
                answered_on: null,
                standing: "open",
              },
            ]
          : [],
      });
    });
    await page.goto("/me");
    const panel = page.getByTestId("my-pay-information");
    await expect(panel.getByTestId("pay-info-none")).toBeVisible();
    await expect(panel.getByTestId("pay-info-ask")).toBeEnabled();
    await panel.getByTestId("pay-info-ask").click();
    await expect(panel.getByTestId("pay-info-standing")).toContainText(
      "Answer due by 2026-12-25",
    );
    await expect(panel.getByTestId("pay-info-ask")).toBeDisabled();
    await expect(panel).not.toContainText("£");
  });

  test("HR works out an answer with a withheld average, sees no one else, and records it as answered", async ({
    page,
  }) => {
    let answered = false;
    const row = (standing: string) => ({
      pid: "r1",
      asked_on: "2026-08-01",
      due_on: "2026-10-01",
      answered_on: answered ? "2026-10-25" : null,
      standing,
      worker_pid: "w1",
      worker_number: "E-001",
      display_name: "Asker One",
      organization_ref: "o",
    });
    await page.route("**/api/proxy/pay-information-requests?*", (route) =>
      route.fulfill({ json: [row(answered ? "answered" : "overdue")] }),
    );
    await page.route(
      "**/api/proxy/pay-information-requests/r1/answer",
      (route) =>
        route.fulfill({
          json: {
            request_pid: "r1",
            asked_on: "2026-08-01",
            due_on: "2026-10-01",
            standing: "overdue",
            currency: "GBP",
            category: { name: "Care", basis: "Same work" },
            answer: {
              own_pay_minor: 3_800_000,
              female_average_minor: { status: "shown", value: 3_900_000 },
              male_average_minor: { status: "withheld" },
            },
            small_group_floor: 10,
            derivation: "Annual full-time-equivalent basic salary.",
          },
        }),
    );
    await page.route(
      "**/api/proxy/pay-information-requests/r1/answered",
      (route) => {
        answered = true;
        return route.fulfill({ json: row("answered") });
      },
    );
    await page.goto("/pay-information");
    const queue = page.getByTestId("pay-info-queue-row");
    await expect(queue).toContainText("Asker One");
    await expect(queue).toContainText("Overdue: was due 2026-10-01");
    await queue.getByTestId("pay-info-work-out").click();
    await expect(page.getByTestId("pay-info-own")).toContainText("£38,000.00");
    await expect(page.getByTestId("pay-info-category")).toContainText("Care");
    await expect(page.getByTestId("pay-info-female")).toContainText(
      "£39,000.00",
    );
    await expect(page.getByTestId("pay-info-male")).toContainText(
      "withheld (fewer than 10 of that sex)",
    );
    await page.getByTestId("pay-info-mark").click();
    await expect(page.getByTestId("pay-info-answer")).toHaveCount(0);
    await expect(page.getByTestId("pay-info-queue-row")).toContainText(
      "Answered 2026-10-25",
    );
    await expect(page.getByTestId("pay-info-work-out")).toHaveCount(0);
  });

  test("HR without payroll access is refused the figures in words and nothing is drawn", async ({
    page,
  }) => {
    await page.route("**/api/proxy/pay-information-requests?*", (route) =>
      route.fulfill({
        json: [
          {
            pid: "r1",
            asked_on: "2026-08-01",
            due_on: "2026-12-01",
            answered_on: null,
            standing: "open",
            worker_pid: "w1",
            worker_number: "E-001",
            display_name: "Asker One",
            organization_ref: "o",
          },
        ],
      }),
    );
    await page.route(
      "**/api/proxy/pay-information-requests/r1/answer",
      (route) => route.fulfill({ status: 403, json: { error: "forbidden" } }),
    );
    await page.goto("/pay-information");
    await page.getByTestId("pay-info-work-out").click();
    await expect(page.getByTestId("error")).toContainText(
      "You are not allowed to do this.",
    );
    await expect(page.getByTestId("pay-info-answer")).toHaveCount(0);
  });
});
