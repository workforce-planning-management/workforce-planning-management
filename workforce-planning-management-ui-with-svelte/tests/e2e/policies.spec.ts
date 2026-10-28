// Policies for HR (`/policies`) and a manager's team counts on `/me`. The API is stubbed per test; an unstubbed /api/proxy call 404s loudly.

import { expect, test } from "@playwright/test";
import { WORKER, stubSignedIn } from "./support";

const VERSION = (pid: string, version: number) => ({
  pid,
  version,
  title: "Acceptable use",
  document_ref: "https://intranet.example/aup",
  effective_on: "2026-09-01",
  audience: "all",
  audience_value: null,
  acknowledge_within_days: 14,
});

test.describe("Policies", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
  });

  test("HR names a policy, adds a version aimed at a department, and opens the named record", async ({
    page,
  }) => {
    const policies: Array<Record<string, unknown>> = [
      { pid: "p1", name: "Acceptable use", versions: [VERSION("v1", 1)] },
    ];
    let versionBody: unknown = null;
    await page.route("**/api/proxy/policies?*", (route) =>
      route.fulfill({ json: { policies } }),
    );
    let renamed: unknown = null;
    await page.route("**/api/proxy/policies/p1", (route) => {
      renamed = route.request().postDataJSON();
      policies[0] = {
        ...policies[0],
        name: (renamed as { name: string }).name,
      };
      return route.fulfill({
        json: { pid: "p1", name: (renamed as { name: string }).name },
      });
    });
    await page.route("**/api/proxy/policies", (route) => {
      const body = route.request().postDataJSON();
      policies.push({ pid: "p2", name: body.name, versions: [] });
      return route.fulfill({ json: { pid: "p2" } });
    });
    await page.route("**/api/proxy/policies/p1/versions", (route) => {
      versionBody = route.request().postDataJSON();
      return route.fulfill({ json: { pid: "v2", version: 2 } });
    });
    await page.route(
      "**/api/proxy/policy-versions/v1/acknowledgements",
      (route) =>
        route.fulfill({
          json: {
            version: VERSION("v1", 1),
            people: [
              {
                worker_pid: "w1",
                worker_number: "E-001",
                acknowledged_on: "2026-09-03",
              },
              {
                worker_pid: "w2",
                worker_number: "E-002",
                acknowledged_on: null,
              },
            ],
          },
        }),
    );

    await page.goto("/policies");
    await expect(page.getByTestId("policy-admin-row")).toContainText(
      "Acceptable use",
    );
    await page.getByTestId("policy-name").fill("Travel");
    await page.getByTestId("policy-add").click();
    await expect(page.getByTestId("policy-admin-row")).toHaveCount(2);

    await page.getByTestId("policy-rename").first().click();
    await page.getByTestId("policy-rename-input").fill("Code of conduct");
    await page.getByTestId("policy-rename-save").click();
    await expect(page.getByTestId("policy-admin-row").first()).toContainText(
      "Code of conduct",
    );
    expect(renamed).toEqual({ name: "Code of conduct" });

    await page.getByTestId("policy-target").selectOption("p1");
    await page.getByTestId("policy-title").fill("Acceptable use v2");
    await page
      .getByTestId("policy-document")
      .fill("https://intranet.example/aup2");
    await page.getByTestId("policy-audience").selectOption("department");
    await page.getByTestId("policy-audience-value").fill("Support");
    await page.getByTestId("policy-version-add").click();
    await expect
      .poll(() => versionBody)
      .toMatchObject({
        title: "Acceptable use v2",
        audience: "department",
        audience_value: "Support",
        acknowledge_within_days: 14,
      });

    await page.getByTestId("policy-record").first().click();
    const rows = page.getByTestId("policy-record-row");
    await expect(rows).toHaveCount(2);
    await expect(rows.nth(0)).toContainText("E-001");
    await expect(rows.nth(0)).toContainText("Acknowledged on 2026-09-03");
    await expect(rows.nth(1)).toContainText("Not yet");
  });

  test("a manager sees counts from the floor and nothing under it, and a non-manager sees no panel", async ({
    page,
  }) => {
    await page.route("**/api/proxy/me/policies", (route) =>
      route.fulfill({
        json: {
          worker_pid: WORKER.pid,
          waiting: 0,
          policies: [
            {
              policy: "Acceptable use",
              version: VERSION("v1", 1),
              standing: { kind: "done", on: "2026-09-02" },
            },
            {
              policy: "Travel",
              version: VERSION("v2", 1),
              standing: { kind: "done", on: "2026-09-02" },
            },
            {
              policy: "Safety",
              version: VERSION("v3", 1),
              standing: { kind: "done", on: "2026-09-02" },
            },
          ],
        },
      }),
    );
    const counts: Record<string, unknown> = {
      v1: { kind: "shown", acknowledged: 4, waiting: 6 },
      v2: { kind: "withheld" },
      v3: { kind: "no_one" },
    };
    await page.route(
      `**/api/proxy/workers/${WORKER.pid}/team-policy-counts?*`,
      (route) => {
        const v =
          new URL(route.request().url()).searchParams.get("version_pid") ?? "";
        return route.fulfill({
          json: { version_pid: v, floor: 10, count: counts[v] },
        });
      },
    );
    await page.goto("/me");
    const rows = page.getByTestId("team-policy-row");
    await expect(rows).toHaveCount(2); // the team of nobody is not shown
    await expect(rows.nth(0)).toContainText("4 acknowledged, 6 waiting");
    await expect(rows.nth(1)).toContainText(
      "Fewer than 10 people: no count is shown",
    );
  });
});
