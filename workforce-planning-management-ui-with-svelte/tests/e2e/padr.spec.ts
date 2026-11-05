// The personal development review (WPM-R172, WPM-R225, WPM-R232): a due date that stays unknown, a manager opening one, writing it,
// objectives as goals, signing, what a signed version shows, a correction, and what waits on /me. The API is stubbed per test.

import { expect, test, type Page } from "@playwright/test";
import { stubSignedIn, WORKER as PERSON } from "./support";

const ME = "33333333-3333-4333-8333-333333333333";
const REPORT = "44444444-4444-4444-8444-444444444444";
const PADR = "55555555-5555-4555-8555-555555555555";

const version = (
  over: Record<string, unknown> = {},
  state: Record<string, boolean> = {},
) => ({
  frozen: false,
  complete: false,
  ...state,
  version: {
    pid: "v1",
    version: 1,
    period_start: "2027-01-01",
    period_end: "2027-12-31",
    values_summary: null,
    wellbeing_offered: null,
    development_needs: [],
    reviewer_signed_on: null,
    reviewee_signed_on: null,
    reviewee_comment: null,
    objectives: null,
    ...over,
  },
});

const padr = (over: Record<string, unknown> = {}) => ({
  pid: PADR,
  worker_pid: REPORT,
  display_name: "Ann Report",
  role: "reviewer",
  status: "draft",
  completed_on: null,
  versions: [version()],
  objectives: [],
  ...over,
});

async function stub(page: Page, view: unknown, list?: unknown) {
  await stubSignedIn(page);
  await page.route("**/api/proxy/me/organizations", (route) =>
    route.fulfill({
      json: [
        {
          organization_ref: PERSON.organization_ref,
          role: "employee",
          worker_pid: ME,
        },
      ],
    }),
  );
  await page.route(`**/api/proxy/me/padrs/${PADR}`, (route) =>
    route.request().method() === "GET"
      ? route.fulfill({ json: view })
      : route.fallback(),
  );
  await page.route("**/api/proxy/me/padrs", (route) =>
    route.fulfill({
      json: list ?? {
        worker_pid: ME,
        due: { standing: "unknown", due_on: null },
        padrs: [],
        team: [],
      },
    }),
  );
  await page.route("**/api/proxy/skills", (route) =>
    route.fulfill({ json: [] }),
  );
  await page.route("**/api/proxy/workers/*/development-plans", (route) =>
    route.fulfill({ json: { worker_pid: REPORT, plans: [] } }),
  );
}

test.describe("Personal development review", () => {
  test("an unknown due date stays unknown", async ({ page }) => {
    await stub(page, padr());
    await page.goto("/me/padr");
    await expect(
      page.getByTestId("padr-mine").getByTestId("padr-standing"),
    ).toHaveText("No date on record");
    await expect(page.getByTestId("padr-mine")).not.toContainText("Overdue");
  });

  test("a manager sees a report by name and opens a review", async ({
    page,
  }) => {
    let sent: unknown = null;
    await stub(page, padr(), {
      worker_pid: ME,
      due: { standing: "current", due_on: "2027-03-01" },
      padrs: [],
      team: [
        {
          worker_pid: REPORT,
          display_name: "Ann Report",
          due: { standing: "overdue", due_on: "2026-02-01" },
          padrs: [],
        },
      ],
    });
    await page.route(`**/api/proxy/me/reports/${REPORT}/padrs`, (route) => {
      sent = route.request().postDataJSON();
      return route.fulfill({ json: { pid: PADR } });
    });
    await page.goto("/me/padr");
    const person = page.getByTestId("padr-person");
    await expect(person).toContainText("Ann Report");
    await expect(person.getByTestId("padr-standing")).toHaveText("Overdue");
    await person.getByTestId("padr-open-start").fill("2027-01-01");
    await person.getByTestId("padr-open-end").fill("2027-12-31");
    await person.getByTestId("padr-open").click();
    await expect(page).toHaveURL(new RegExp(`/me/padr/${PADR}$`));
    expect(sent).toEqual({
      period_start: "2027-01-01",
      period_end: "2027-12-31",
    });
  });

  test("an open review can be written, and a blank wellbeing answer is sent as unknown", async ({
    page,
  }) => {
    let sent: Record<string, unknown> | null = null;
    await stub(page, padr());
    await page.route(`**/api/proxy/me/padrs/${PADR}`, (route) => {
      if (route.request().method() !== "PUT") return route.fallback();
      sent = route.request().postDataJSON();
      return route.fulfill({ json: { pid: PADR } });
    });
    await page.goto(`/me/padr/${PADR}`);
    await page.getByTestId("padr-values").fill("How the team works");
    await page.getByTestId("padr-add-need").click();
    await page.getByTestId("padr-need-text").fill("Coaching");
    await page.getByTestId("padr-save").click();
    await expect.poll(() => sent).not.toBeNull();
    expect(sent).toMatchObject({
      values_summary: "How the team works",
      wellbeing_offered: null,
      development_needs: [{ need: "Coaching", plan_item_pid: null }],
    });
  });

  test("an objective is a goal in the review's period", async ({ page }) => {
    let sent: Record<string, unknown> | null = null;
    await stub(page, padr());
    await page.route(`**/api/proxy/me/reports/${REPORT}/goals`, (route) => {
      sent = route.request().postDataJSON();
      return route.fulfill({ json: { pid: "g9" } });
    });
    await page.goto(`/me/padr/${PADR}`);
    await page.getByText("Add an objective").click();
    const form = page.getByTestId("padr-add-objective");
    await expect(form.getByTestId("goal-start")).toHaveValue("2027-01-01");
    await form.getByTestId("goal-title").fill("Run the rota");
    await form.getByTestId("goal-propose").click();
    await expect.poll(() => sent).not.toBeNull();
    expect(sent).toMatchObject({
      title: "Run the rota",
      period_start: "2027-01-01",
      period_end: "2027-12-31",
    });
  });

  test("a signed version is read only and shows what it froze", async ({
    page,
  }) => {
    await stub(
      page,
      padr({
        role: "reviewee",
        versions: [
          version(
            {
              reviewer_signed_on: "2026-11-01",
              wellbeing_offered: false,
              objectives: [
                {
                  pid: "g1",
                  title: "Run the rota",
                  status: "agreed",
                  period_start: "2027-01-01",
                  period_end: "2027-06-30",
                  measures: [],
                },
              ],
            },
            { frozen: true },
          ),
        ],
      }),
    );
    let signed = false;
    await page.route(`**/api/proxy/me/padrs/${PADR}/sign`, (route) => {
      signed = true;
      return route.fulfill({ json: { pid: PADR, complete: true } });
    });
    await page.goto(`/me/padr/${PADR}`);
    await expect(page.getByTestId("padr-form")).toHaveCount(0);
    await expect(page.getByTestId("padr-frozen-objective")).toContainText(
      "Run the rota",
    );
    await expect(page.getByTestId("padr-wellbeing-read")).toHaveText("No");
    await expect(page.getByTestId("padr-reviewee-signed")).toContainText(
      "Not signed",
    );
    await page.getByTestId("padr-sign").click();
    await expect.poll(() => signed).toBe(true);
  });

  test("a wellbeing answer never given reads as a dash, not a no", async ({
    page,
  }) => {
    await stub(page, padr({ role: "manager" }));
    await page.goto(`/me/padr/${PADR}`);
    await expect(page.getByTestId("padr-wellbeing-read")).toHaveText("—");
    await expect(page.getByTestId("padr-sign")).toHaveCount(0);
    await expect(page.getByTestId("padr-correct")).toHaveCount(0);
  });

  test("a correction keeps the signed version", async ({ page }) => {
    await stub(
      page,
      padr({
        status: "shared",
        completed_on: "2026-11-01",
        versions: [
          version(
            { values_summary: "As signed", objectives: [] },
            { frozen: true, complete: true },
          ),
          version({ pid: "v2", version: 2, values_summary: "Corrected" }),
        ],
      }),
    );
    await page.goto(`/me/padr/${PADR}`);
    await expect(page.getByTestId("padr-version")).toHaveText("Version 2");
    await expect(page.getByTestId("padr-values")).toHaveValue("Corrected");
    await page.getByTestId("padr-earlier").locator("summary").click();
    await expect(
      page.getByTestId("padr-earlier").getByTestId("padr-values-read"),
    ).toHaveText("As signed");
  });

  test("what is waiting shows on my profile and links to where it is done", async ({
    page,
  }) => {
    await stub(page, padr());
    await page.route(`**/api/proxy/workers/${ME}/notifications`, (route) =>
      route.fulfill({
        json: [
          {
            pid: "n1",
            kind: "goal_waiting",
            body: "",
            data: {},
            created_at: "2026-11-05T00:00:00Z",
            read_at: null,
          },
          {
            pid: "n2",
            kind: "padr_to_sign",
            body: "",
            data: { padr_pid: PADR },
            created_at: "2026-11-05T00:00:00Z",
            read_at: null,
          },
          {
            pid: "n3",
            kind: "goal_waiting",
            body: "",
            data: {},
            created_at: "2026-11-05T00:00:00Z",
            read_at: "2026-11-05T00:00:00Z",
          },
          {
            pid: "n4",
            kind: "policy_due",
            body: "",
            data: {},
            created_at: "2026-11-05T00:00:00Z",
            read_at: null,
          },
        ],
      }),
    );
    await page.goto("/me");
    const items = page.getByTestId("waiting-item");
    await expect(items).toHaveCount(2);
    await expect(items.nth(0).getByRole("link")).toHaveAttribute(
      "href",
      /\/me\/goals$/,
    );
    await expect(items.nth(1).getByRole("link")).toHaveAttribute(
      "href",
      new RegExp(`/me/padr/${PADR}$`),
    );
  });
});
