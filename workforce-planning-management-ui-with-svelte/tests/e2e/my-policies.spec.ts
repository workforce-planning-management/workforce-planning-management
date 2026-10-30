// My policies on /me. The API is stubbed per test; an unstubbed /api/proxy call 404s loudly.

import { expect, test } from "@playwright/test";
import { stubSignedIn } from "./support";

test.describe("My policies", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
  });

  test("a worker sees what waits, how late it is, and acknowledges it themself", async ({
    page,
  }) => {
    let acknowledged = false;
    const version = (pid: string, name: string) => ({
      pid,
      version: 2,
      title: name,
      document_ref: `https://intranet.example/${pid}`,
      effective_on: "2026-09-01",
      audience: "all",
      audience_value: null,
      acknowledge_within_days: 14,
    });
    const sent: unknown[] = [];
    await page.route("**/api/proxy/me/policies", (route) =>
      route.fulfill({
        json: {
          worker_pid: "w",
          waiting: acknowledged ? 0 : 2,
          policies: [
            {
              policy: "Acceptable use",
              version: version("v1", "Acceptable use"),
              standing: acknowledged
                ? { kind: "done", on: "2026-10-25" }
                : {
                    kind: "overdue",
                    due_on: "2026-09-15",
                    calendar_days_late: 40,
                  },
            },
            {
              policy: "Travel",
              version: version("v2", "Travel"),
              standing: { kind: "waiting", due_on: "2026-11-01" },
            },
          ],
        },
      }),
    );
    await page.route("**/api/proxy/me/policy-acknowledgements", (route) => {
      sent.push(route.request().postDataJSON());
      acknowledged = true;
      return route.fulfill({ json: { acknowledged_on: "2026-10-25" } });
    });

    await page.goto("/me");
    const panel = page.getByTestId("my-policies");
    await expect(panel).toContainText("Acceptable use");
    await expect(panel).toContainText(
      "Overdue: due on 15 Sept 2026, 40 calendar days ago",
    );
    await expect(panel).toContainText("Please acknowledge by 1 Nov 2026");
    await panel.getByTestId("policy-acknowledge").first().click();
    await expect(panel).toContainText("Acknowledged on 25 Oct 2026");
    expect(sent).toEqual([{ version_pid: "v1" }]);
  });
});
