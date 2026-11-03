// Coverage evidence (WPM-T420): HR's list with each record's standing, the add form and retire. The API is stubbed per test; an
// unstubbed /api/proxy call 404s loudly.

import { expect, test } from "@playwright/test";
import { WORKER, stubSignedIn } from "./support";

const record = (over: Record<string, unknown>) => ({
  pid: "c1",
  organization_ref: WORKER.organization_ref,
  jurisdiction: "XA",
  kind: "private_insurance",
  insurer: "Acme",
  policy_reference: "P-1",
  minimum_cover_minor: 500000,
  currency: "EUR",
  valid_from: "2026-01-01",
  valid_to: "2026-12-01",
  certificate_ref: null,
  standing: "in_force",
  days_left: null,
  retired_on: null,
  ...over,
});

test.describe("Coverage evidence", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
  });

  test("each record shows its standing, and no end date is never valid", async ({
    page,
  }) => {
    await page.route("**/api/proxy/coverage-evidence?organization=*", (route) =>
      route.fulfill({
        json: [
          record({ standing: "ending_soon", days_left: 12 }),
          record({
            pid: "c2",
            jurisdiction: "XB",
            kind: "statutory_scheme",
            insurer: null,
            policy_reference: null,
            minimum_cover_minor: null,
            currency: null,
            valid_to: null,
            standing: "no_end_date",
          }),
        ],
      }),
    );
    await page.goto("/en-001/coverage-evidence");
    await page
      .getByTestId("coverage-organization")
      .selectOption(WORKER.organization_ref);
    const rows = page.getByTestId("coverage-row");
    await expect(rows.nth(0)).toContainText("Ending soon (12)");
    await expect(rows.nth(1)).toContainText("No end date");
    await expect(rows.nth(1)).not.toContainText("In force");
  });

  test("HR adds a private policy, which names its insurer, and retires a record", async ({
    page,
  }) => {
    const org = WORKER.organization_ref;
    const sent: { url: string; body: unknown }[] = [];
    let rows: unknown[] = [];
    await page.route("**/api/proxy/coverage-evidence?organization=*", (route) =>
      route.fulfill({ json: rows }),
    );
    await page.route("**/api/proxy/coverage-evidence", (route) => {
      sent.push({ url: "add", body: route.request().postDataJSON() });
      rows = [record({})];
      return route.fulfill({ json: { pid: "c1" } });
    });
    await page.route("**/api/proxy/coverage-evidence/c1/retire", (route) => {
      sent.push({ url: "retire", body: null });
      rows = [];
      return route.fulfill({ json: { pid: "c1" } });
    });
    await page.goto("/en-001/coverage-evidence");
    await page.getByTestId("coverage-organization").selectOption(org);
    await expect(page.getByTestId("cov-insurer")).toHaveCount(0);
    await page.getByTestId("cov-jurisdiction").fill("XA");
    await page.getByTestId("cov-kind").selectOption("private_insurance");
    await page.getByTestId("cov-insurer").fill("Acme");
    await page.getByTestId("cov-policy").fill("P-1");
    await page.getByTestId("cov-cover").fill("500000");
    await page.getByTestId("cov-currency").fill("EUR");
    await page.getByTestId("cov-to").fill("2026-12-01");
    await page.getByTestId("cov-save").click();
    await expect(page.getByTestId("coverage-row")).toContainText("In force");
    expect(sent[0]?.body).toMatchObject({
      organization: org,
      jurisdiction: "XA",
      kind: "private_insurance",
      insurer: "Acme",
      policy_reference: "P-1",
      minimum_cover_minor: 500000,
      currency: "EUR",
      valid_to: "2026-12-01",
    });
    await page.getByRole("button", { name: "Retire" }).click();
    await expect(page.getByTestId("coverage-row")).toHaveCount(0);
    expect(sent.map((s) => s.url)).toEqual(["add", "retire"]);
  });
});
