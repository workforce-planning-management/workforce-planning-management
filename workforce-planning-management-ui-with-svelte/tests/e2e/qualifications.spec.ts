// Qualifications and the qualification gap (WPM-R171, WPM-T316): the person's panel on /me. A declared qualification is never shown
// as met; with no role the gap says so; adding and verifying read the list again. The API is stubbed per test.

import { expect, test } from "@playwright/test";
import { WORKER, stubSignedIn } from "./support";

const URL = `**/api/proxy/workers/${WORKER.pid}/qualifications`;

const list = (over: object) => ({
  worker_pid: WORKER.pid,
  as_of: "2026-11-02",
  qualifications: [],
  role: null,
  gap: [],
  ...over,
});

test.describe("Qualifications", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
  });

  test("a declared qualification is not met, and no role says so in words", async ({
    page,
  }) => {
    await page.route(URL, (route) =>
      route.fulfill({
        json: list({
          qualifications: [
            {
              pid: "q1",
              kind: "academic",
              title: "Degree",
              awarding_body: null,
              framework: "RQF",
              level: 6,
              awarded_on: "2015-06-30",
              expires_on: null,
              status: "declared",
              expired: false,
              verified_on: null,
              evidence_ref: null,
            },
          ],
          role: { pid: "r1", job_title: "Tester" },
          gap: [
            {
              title: "Degree",
              kind: "academic",
              framework: "RQF",
              min_level: 6,
              standing: "declared_only",
            },
            {
              title: "First aid",
              kind: null,
              framework: null,
              min_level: null,
              standing: "missing",
            },
          ],
        }),
      }),
    );
    await page.goto("/me");
    const panel = page.getByTestId("my-qualifications");
    await expect(panel.getByTestId("qualification-declared")).toContainText(
      "Declared",
    );
    await expect(panel.getByTestId("gap-declared_only")).toContainText(
      "Declared, not yet verified",
    );
    await expect(panel.getByTestId("gap-missing")).toContainText("Not held");
    await expect(panel.getByTestId("gap-declared_only")).not.toContainText(
      "Met",
    );
  });

  test("with no role recorded the gap says there is nothing to compare", async ({
    page,
  }) => {
    await page.route(URL, (route) => route.fulfill({ json: list({}) }));
    await page.goto("/me");
    await expect(page.getByTestId("qualification-no-role")).toContainText(
      "nothing to compare",
    );
  });

  test("adding a qualification posts it and reads the list again", async ({
    page,
  }) => {
    let posted: Record<string, unknown> | null = null;
    let reads = 0;
    await page.route(URL, async (route) => {
      if (route.request().method() === "POST") {
        posted = route.request().postDataJSON() as Record<string, unknown>;
        return route.fulfill({ json: { pid: "q2" } });
      }
      reads += 1;
      return route.fulfill({ json: list({}) });
    });
    await page.goto("/me");
    await expect(page.getByTestId("qualification-no-role")).toBeVisible();
    const before = reads;
    await page.getByTestId("qualification-title").fill("First aid at work");
    await page.getByTestId("qualification-kind").selectOption("licence");
    await page.getByTestId("qualification-save").click();
    await expect.poll(() => posted?.title).toBe("First aid at work");
    expect(posted).toMatchObject({ kind: "licence" });
    await expect.poll(() => reads).toBeGreaterThan(before);
  });
});
