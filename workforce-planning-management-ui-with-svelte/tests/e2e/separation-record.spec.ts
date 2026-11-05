// The separation record (WPM-R222): built from the leaver record, what is missing is named, a refused reader sees nothing, and the
// export link carries the period. The API is stubbed per test.

import { expect, test, type Page } from "@playwright/test";
import { stubSignedIn, WORKER } from "./support";

const MV = "bbbbbbbb-0000-4000-8000-00000000b001";

const movement = {
  pid: MV,
  kind: "leaver",
  worker_pid: WORKER.pid,
  worker_name: WORKER.display_name,
  department: "engineering",
  job_title: "Engineer",
  organization_ref: WORKER.organization_ref,
  effective_on: "2027-03-31",
  reason: "redundancy",
  status: "completed",
  notes: null,
  completed_at: null,
  progress: { closed: 0, total: 0, overdue: 0 },
};

const record = {
  movement_pid: MV,
  worker_pid: WORKER.pid,
  last_day: "2027-03-31",
  reason_code: "redundancy",
  final_pay_on: null,
  pay_summary_ref: null,
  missing: ["final_pay_on", "pay_summary_ref"],
};

async function stub(page: Page) {
  await stubSignedIn(page);
  await page.route("**/api/proxy/workers?*", (route) =>
    route.fulfill({ json: [WORKER] }),
  );
  await page.route(`**/api/proxy/movements/${MV}`, (route) =>
    route.fulfill({ json: { ...movement, items: [] } }),
  );
  await page.route(`**/api/proxy/movements/${MV}/handover`, (route) =>
    route.fulfill({
      json: { last_day: "2027-03-31", remaining: 0, counts: {}, items: [] },
    }),
  );
  await page.route(`**/api/proxy/movements/${MV}/handover/actions`, (route) =>
    route.fulfill({ json: [] }),
  );
}

test.describe("Separation record", () => {
  test("names what is missing, and saves", async ({ page }) => {
    let sent: unknown = null;
    await stub(page);
    await page.route(
      `**/api/proxy/movements/${MV}/separation-record`,
      (route) => {
        if (route.request().method() === "GET") {
          return route.fulfill({ json: record });
        }
        sent = route.request().postDataJSON();
        return route.fulfill({
          json: {
            ...record,
            final_pay_on: "2027-04-25",
            pay_summary_ref: "payroll:run-42",
            missing: [],
          },
        });
      },
    );
    await page.goto(`/movements/${MV}`);
    await expect(page.getByTestId("separation-reason")).toContainText(
      "Redundancy",
    );
    await expect(page.getByTestId("separation-missing")).toContainText(
      "Final pay date",
    );
    await page.getByTestId("separation-final-pay").fill("2027-04-25");
    await page.getByTestId("separation-ref").fill("payroll:run-42");
    await page.getByTestId("separation-save").click();
    await expect(page.getByTestId("separation-complete")).toBeVisible();
    expect(sent).toEqual({
      final_pay_on: "2027-04-25",
      pay_summary_ref: "payroll:run-42",
    });
  });

  test("someone who is refused sees no panel and no error", async ({
    page,
  }) => {
    await stub(page);
    await page.route(
      `**/api/proxy/movements/${MV}/separation-record`,
      (route) => route.fulfill({ status: 403, json: {} }),
    );
    await page.goto(`/movements/${MV}`);
    await expect(page.getByTestId("held-none")).toBeVisible();
    await expect(page.getByTestId("separation-panel")).toHaveCount(0);
    await expect(page.getByTestId("separation-error")).toHaveCount(0);
  });

  test("the export link appears once the period runs forwards", async ({
    page,
  }) => {
    await stubSignedIn(page);
    await page.route("**/api/proxy/movements?*", (route) =>
      route.fulfill({ json: [] }),
    );
    await page.goto("/movements");
    const panel = page.getByTestId("separation-export");
    await panel.locator("summary").click();
    await panel.getByTestId("separation-from").fill("2027-04-01");
    await panel.getByTestId("separation-to").fill("2027-03-01");
    await expect(panel.getByTestId("separation-download")).toHaveCount(0);
    await panel.getByTestId("separation-from").fill("2027-03-01");
    await panel.getByTestId("separation-to").fill("2027-03-31");
    await expect(panel.getByTestId("separation-download")).toHaveAttribute(
      "href",
      /separation-records\/export\?organization=.*from=2027-03-01&to=2027-03-31/,
    );
  });
});
