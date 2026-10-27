// The consultation register (`/consultations`). The API is stubbed per test; an unstubbed /api/proxy call 404s loudly.

import { expect, test } from "@playwright/test";
import { stubSignedIn } from "./support";

const body = { pid: "b1", organization_ref: "o", name: "Care Union", kind: "union", basis: "other", recognised_on: "2020-03-01", contact_role: "HR lead", units: 0 };
const record = (extra: Record<string, unknown> = {}) => ({
  pid: "c1", organization_ref: "o", body_pid: "b1", plan_pid: null, establishment: "Leeds site", profile: "great_britain",
  establishment_size: 200, dismissals: 25, earliest_termination_on: "2027-03-01", topics: ["redundancy"], document_ref: null,
  started_on: null, ended_on: null, outcome: "open", agreement_pid: null, ...extra,
});
const timeline = (c: unknown, timeliness: unknown) => ({
  consultation: c,
  assessment: {
    profile: "great_britain", source: "TULRCA 1992 s.188", may_apply: true, evidence: { counted: 25, needed: 20, window_days: 90 },
    consultation_days: 30, latest_consultation_start: "2027-01-30", latest_authority_notice: "2027-01-30", days_remaining: 100,
    already_late: false, checklist: ["notify_the_authority"], agreed_period_days: null, deployer_to_confirm: "x",
  },
  timeliness,
  derivation: "d",
});

test.describe("Consultations", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
  });

  test("opens a record, shows the timeline as a suggestion with its evidence, then records the start", async ({ page }) => {
    let rows: unknown[] = [];
    let opened: Record<string, unknown> | null = null;
    let begun: Record<string, unknown> | null = null;
    await page.route("**/api/proxy/recognised-bodies?*", (route) => route.fulfill({ json: { bodies: [body] } }));
    await page.route("**/api/proxy/consultations?*", (route) => route.fulfill({ json: { consultations: rows } }));
    await page.route("**/api/proxy/consultations", (route) => {
      opened = route.request().postDataJSON();
      rows = [record()];
      return route.fulfill({ json: { pid: "c1" } });
    });
    await page.route("**/api/proxy/consultations/c1/timeline", (route) =>
      route.fulfill({ json: timeline(rows[0], begun ? { kind: "begun_in_time" } : { kind: "not_begun", days_left: 100 }) }),
    );
    await page.route("**/api/proxy/consultations/c1/begin", (route) => {
      begun = route.request().postDataJSON();
      rows = [record({ started_on: "2026-10-01" })];
      return route.fulfill({ json: { pid: "c1" } });
    });
    await page.goto("/consultations");
    await expect(page.getByTestId("cs-none")).toBeVisible();
    await page.getByTestId("cs-establishment").fill("Leeds site");
    await page.getByTestId("cs-size").fill("200");
    await page.getByTestId("cs-dismissals").fill("25");
    await page.getByTestId("cs-termination").fill("2027-03-01");
    await page.getByTestId("cs-topic-redundancy").check();
    await page.getByTestId("cs-open").click();
    await expect(page.getByTestId("cs-row")).toContainText("Leeds site");
    expect(opened).toEqual({
      body_pid: "b1", establishment: "Leeds site", profile: "great_britain", establishment_size: 200, dismissals: 25,
      earliest_termination_on: "2027-03-01", topics: ["redundancy"],
    });
    await page.getByTestId("cs-timeline").click();
    await expect(page.getByTestId("cs-assessment")).toContainText("A collective duty may apply");
    await expect(page.getByTestId("cs-assessment")).toContainText("25 counted, 20 needed within 90 days");
    await expect(page.getByTestId("cs-assessment")).toContainText("not a verdict");
    await expect(page.getByTestId("cs-timeliness")).toContainText("100 days left");
    await page.getByTestId("cs-began").fill("2026-10-01");
    await page.getByTestId("cs-begin").click();
    await expect(page.getByTestId("cs-timeliness")).toContainText("Consultation began in time");
    expect(begun).toEqual({ started_on: "2026-10-01" });
  });

  test("a refusal is shown in the service's words", async ({ page }) => {
    await page.route("**/api/proxy/recognised-bodies?*", (route) => route.fulfill({ json: { bodies: [body] } }));
    await page.route("**/api/proxy/consultations?*", (route) => route.fulfill({ status: 403, json: { error: "forbidden" } }));
    await page.goto("/consultations");
    await expect(page.getByTestId("error")).toContainText("You are not allowed to do this.");
  });
});
