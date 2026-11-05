// Injury and illness records (WPM-T421): proof of each report and a pointer to the statutory record, with no box for a description. The
// API is stubbed per test; an unstubbed /api/proxy call 404s loudly.

import { expect, test } from "@playwright/test";
import { WORKER, stubSignedIn } from "./support";

const report = (over: Record<string, unknown>) => ({
  pid: "r1",
  recipient: "insurer",
  due_at: "2026-03-09T10:30:00Z",
  sent_at: null,
  method: null,
  reference: null,
  standing: "past_due",
  ...over,
});

const incident = (over: Record<string, unknown>) => ({
  pid: "i1",
  organization_ref: WORKER.organization_ref,
  jurisdiction: "XA",
  kind: "accident",
  worker_pid: WORKER.pid,
  worker_name: WORKER.display_name,
  occurred_at: "2026-03-04T09:00:00Z",
  learned_at: "2026-03-04T10:30:00Z",
  class: "absence",
  absence_days: 4,
  people_affected: 1,
  statutory_record_ref: "doc:301/77",
  keep_until: "2032-01-01",
  reports: [],
  ...over,
});

test.describe("Injury records", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
    await page.route("**/api/proxy/workers*", (route) =>
      route.fulfill({ json: [WORKER] }),
    );
  });

  test("each report stands on its own times, and unknowns are said", async ({
    page,
  }) => {
    await page.route("**/api/proxy/injury-records?organization=*", (route) =>
      route.fulfill({
        json: [
          incident({
            reports: [
              report({}),
              report({
                pid: "r2",
                recipient: "safety_authority",
                due_at: null,
                standing: "no_due",
              }),
              report({
                pid: "r3",
                recipient: "labour_inspectorate",
                sent_at: "2026-03-04T17:00:00Z",
                method: "phone",
                reference: "CAL-991",
                standing: "sent_by_due",
              }),
            ],
          }),
          incident({
            pid: "i2",
            kind: "dangerous_occurrence",
            class: "no_injury",
            worker_pid: null,
            worker_name: null,
            statutory_record_ref: null,
            keep_until: null,
          }),
        ],
      }),
    );
    await page.goto("/en-001/injury-records");
    await page
      .getByTestId("injury-organization")
      .selectOption(WORKER.organization_ref);
    const standings = page.getByTestId("injury-standing");
    await expect(standings.nth(0)).toHaveText(
      "Not sent, past the time entered",
    );
    await expect(standings.nth(1)).toHaveText("Not sent, no due time entered");
    await expect(standings.nth(2)).toHaveText("Sent by the time entered");
    await expect(page.getByTestId("injury-report").nth(2)).toContainText(
      "CAL-991",
    );
    const second = page.getByTestId("injury-row").nth(1);
    await expect(second).toContainText("Dangerous occurrence");
    await expect(second.getByTestId("injury-ref")).toHaveText("Not recorded");
    await expect(second).toContainText("Not known");
  });

  test("recording an incident sends facts and offers no box for a description", async ({
    page,
  }) => {
    let sent: Record<string, unknown> | null = null;
    await page.route("**/api/proxy/injury-records?organization=*", (route) =>
      route.fulfill({ json: [] }),
    );
    await page.route("**/api/proxy/injury-records", async (route) => {
      sent = route.request().postDataJSON() as Record<string, unknown>;
      await route.fulfill({ json: { pid: "new" } });
    });
    await page.goto("/en-001/injury-records");
    await page
      .getByTestId("injury-organization")
      .selectOption(WORKER.organization_ref);
    await expect(page.getByTestId("injury-none")).toBeVisible();
    await expect(
      page.getByTestId("injury-admin").locator("textarea"),
    ).toHaveCount(0);
    await page.getByTestId("inj-jurisdiction").fill("XA");
    await page.getByTestId("inj-occurred").fill("2026-03-04T09:00");
    await page.getByTestId("inj-class").selectOption("serious");
    await page.getByTestId("inj-worker").selectOption(WORKER.pid);
    await page.getByTestId("inj-days").fill("9");
    await page.getByTestId("inj-ref").fill("fell from a ladder");
    await page.getByTestId("inj-save").click();
    expect(sent, "a sentence is not a reference").toBeNull();
    await page.getByTestId("inj-ref").fill("doc:301/77");
    await page.getByTestId("inj-save").click();
    await expect(page.getByTestId("inj-saved")).toBeVisible();
    expect(sent).toMatchObject({
      jurisdiction: "XA",
      class: "serious",
      worker_pid: WORKER.pid,
      absence_days: 9,
      statutory_record_ref: "doc:301/77",
    });
  });

  test("a report is added, then said to have gone", async ({ page }) => {
    const calls: string[] = [];
    await page.route("**/api/proxy/injury-records?organization=*", (route) =>
      route.fulfill({ json: [incident({ reports: [report({})] })] }),
    );
    await page.route(
      "**/api/proxy/injury-records/i1/reports",
      async (route) => {
        calls.push(`add ${JSON.stringify(route.request().postDataJSON())}`);
        await route.fulfill({ json: { pid: "r9" } });
      },
    );
    await page.route("**/api/proxy/injury-reports/r1", async (route) => {
      calls.push(`sent ${JSON.stringify(route.request().postDataJSON())}`);
      await route.fulfill({ json: { pid: "r1" } });
    });
    await page.goto("/en-001/injury-records");
    await page
      .getByTestId("injury-organization")
      .selectOption(WORKER.organization_ref);
    await page.getByTestId("injury-recipient").selectOption("health_authority");
    await page.getByTestId("injury-add-report").click();
    await expect.poll(() => calls.length).toBe(1);
    expect(calls[0]).toContain("health_authority");
    await page.getByTestId("injury-sent-at").fill("2026-03-10T09:00");
    await page.getByTestId("injury-method").selectOption("form");
    await page.getByTestId("injury-reference").fill("CLM-1");
    await page.getByTestId("injury-mark-sent").click();
    await expect.poll(() => calls.length).toBe(2);
    expect(calls[1]).toContain('"method":"form"');
    expect(calls[1]).toContain("CLM-1");
  });

  test("a refusal is said, and draws no empty list", async ({ page }) => {
    await page.route("**/api/proxy/injury-records?organization=*", (route) =>
      route.fulfill({
        status: 403,
        json: { description: "this is done by HR" },
      }),
    );
    await page.goto("/en-001/injury-records");
    await page
      .getByTestId("injury-organization")
      .selectOption(WORKER.organization_ref);
    await expect(page.getByTestId("error")).toContainText(
      "You are not allowed to do this.",
    );
    await expect(page.getByTestId("injury-none")).toHaveCount(0);
  });
});
