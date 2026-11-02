// HR's queue of declared qualifications awaiting verification (WPM-R171, WPM-T435). The API is stubbed per test.

import { expect, test } from "@playwright/test";
import { WORKER, stubSignedIn } from "./support";

const item = {
  pid: "q1",
  worker_pid: "99999999-9999-4999-8999-999999999999",
  display_name: "Report One",
  kind: "licence",
  title: "Forklift",
  awarding_body: null,
  framework: null,
  level: null,
  awarded_on: null,
  expires_on: null,
  status: "declared",
  expired: false,
  verified_on: null,
  evidence_ref: null,
  declared_on: "2026-10-30",
};

test.describe("Qualification queue", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
  });

  test("lists declared qualifications by name and verifying one reads the queue again", async ({
    page,
  }) => {
    let queue = [item];
    let sent: unknown = null;
    await page.route("**/api/proxy/qualifications/unverified?*", (route) =>
      route.fulfill({
        json: {
          organization: WORKER.organization_ref,
          as_of: "2026-11-02",
          queue,
        },
      }),
    );
    await page.route("**/api/proxy/qualifications/q1/verify", (route) => {
      sent = route.request().postDataJSON();
      queue = [];
      return route.fulfill({ json: { pid: "q1", status: "verified" } });
    });
    await page.goto("/en-001/qualifications");
    await page
      .getByTestId("queue-organization")
      .selectOption(WORKER.organization_ref);
    const row = page.getByTestId("queue-item");
    await expect(row).toContainText("Report One");
    await expect(row).toContainText("Forklift");
    await row.getByRole("textbox").fill("HR file 12");
    await row.getByTestId("queue-verify").click();
    await expect(page.getByTestId("queue-none")).toBeVisible();
    expect(sent).toEqual({ evidence_ref: "HR file 12" });
  });

  test("an empty queue says nothing is waiting", async ({ page }) => {
    await page.route("**/api/proxy/qualifications/unverified?*", (route) =>
      route.fulfill({
        json: {
          organization: WORKER.organization_ref,
          as_of: "2026-11-02",
          queue: [],
        },
      }),
    );
    await page.goto("/en-001/qualifications");
    await page
      .getByTestId("queue-organization")
      .selectOption(WORKER.organization_ref);
    await expect(page.getByTestId("queue-none")).toContainText(
      "Nothing is waiting",
    );
  });
});
