// Tolerances on a workforce plan (WPM-R140): red, amber, green shown as a colour, a shape and a word, with
// an unknown shown as unknown. The API is stubbed per test; an unstubbed /api/proxy call 404s loudly.

import { expect, test, type Page } from "@playwright/test";

const PLAN = "11111111-1111-4111-8111-111111111111";

async function signIn(page: Page) {
  await page.context().addCookies([
    {
      name: "__Host-mxi_session",
      value: "tolerance-spec-session",
      domain: "localhost",
      path: "/",
      httpOnly: true,
      secure: true,
      sameSite: "Lax",
    },
  ]);
}

const dimension = (
  name: string,
  unit: string,
  over: Record<string, unknown>,
) => ({
  dimension: name,
  unit,
  set: true,
  limit: 100,
  amber_percent: 75,
  deviation: 0,
  rag: "green",
  basis: `basis of ${name}`,
  reason: null,
  escalation: null,
  ...over,
});

const TOLERANCES = {
  derivation: "a tolerance is the largest deviation the plan may have",
  as_of: "2026-10-20",
  overall: "red",
  breaches_without_escalation: 1,
  dimensions: [
    dimension("time", "calendar_days", {
      deviation: 112,
      limit: 120,
      rag: "amber",
    }),
    dimension("cost", "basis_points", {
      deviation: 10000,
      limit: 20000,
      rag: "green",
    }),
    dimension("resources", "basis_points", {
      deviation: 4000,
      limit: 3000,
      rag: "red",
    }),
    dimension("risk", "roles", {
      set: true,
      limit: 0,
      deviation: null,
      rag: "unknown",
      reason:
        "there is no succession plan in the plan's departments to look at",
    }),
  ],
};

async function stub(page: Page) {
  await page.route("**/api/proxy/**", (route) =>
    route.fulfill({ status: 404, body: "unstubbed: " + route.request().url() }),
  );
  const json = (pattern: string, body: unknown) =>
    page.route(pattern, (route) => route.fulfill({ json: body }));
  await json("**/api/proxy/role-profiles**", []);
  await json("**/api/proxy/workforce-plans", [
    {
      pid: PLAN,
      name: "FY27",
      organization_ref: "organization:33333333-3333-4333-8333-333333333333",
      horizon_start: "2026-01-01",
      horizon_end: "2027-12-31",
      status: "active",
      attrition_bp: 1000,
      demand_lines: 0,
    },
  ]);
  await json(`**/api/proxy/workforce-plans/${PLAN}`, {
    pid: PLAN,
    name: "FY27",
    organization_ref: "organization:33333333-3333-4333-8333-333333333333",
    horizon_start: "2026-01-01",
    horizon_end: "2027-12-31",
    rationale: null,
    attrition_bp: 1000,
    budget_minor: 10000000,
    budget_currency: "GBP",
    on_cost_bp: 0,
    status: "active",
    demand_lines: [],
    objectives: [],
  });
  await json(`**/api/proxy/workforce-plans/${PLAN}/forecast`, {
    derivation: "",
    as_of: "2026-10-20",
    plan: { pid: PLAN, name: "FY27", status: "active" },
    assumptions: {
      attrition_bp: 1000,
      attrition_source: "plan_assumption",
      hires_assumed: 0,
    },
    departments: [],
    programmes: [],
  });
  await json(`**/api/proxy/workforce-plans/${PLAN}/alignment`, {
    derivation: "",
    planned_headcount: 0,
    aligned_share: null,
    objectives: 0,
    unresourced_objectives: [],
    unaligned_demand_lines: [],
    critical_roles_without_bench: {
      in_plan_departments: 0,
      all_departments: 0,
    },
  });
  await json(`**/api/proxy/workforce-plans/${PLAN}/escalations`, []);
  await json(`**/api/proxy/workforce-plans/${PLAN}/cost`, {
    derivation: "",
    currency: "GBP",
    salary_visible: true,
    assumptions: { on_cost_bp: 0 },
    groups: [],
    total_annual_cost_minor: 0,
    uncosted_groups: 0,
    affordability: null,
  });
}

test.describe("tolerances", () => {
  test("each dimension shows a colour, a shape and a word, and an unknown is not green", async ({
    page,
  }) => {
    await signIn(page);
    await stub(page);
    await page.route(
      `**/api/proxy/workforce-plans/${PLAN}/tolerances`,
      (route) => route.fulfill({ json: TOLERANCES }),
    );
    await page.goto("/en-001/planning");

    const table = page.getByTestId("plan-tolerances");
    await expect(table).toBeVisible();
    for (const [name, rag, word] of [
      ["time", "amber", "Amber"],
      ["cost", "green", "Green"],
      ["resources", "red", "Red"],
      ["risk", "unknown", "Unknown"],
    ]) {
      const chip = page.getByTestId(`tolerance-${name}`).locator(".chip.rag");
      await expect(chip).toHaveAttribute("data-rag", rag);
      await expect(chip).toContainText(word);
      await expect(chip.locator(".shape")).toHaveAttribute(
        "aria-hidden",
        "true",
      );
    }
    // look at it: light and dark, in test-results/
    await table.screenshot({ path: "test-results/tolerance-light.png" });
    await page.emulateMedia({ colorScheme: "dark" });
    await table.screenshot({ path: "test-results/tolerance-dark.png" });
    await page.emulateMedia({ colorScheme: "light" });
    // the overall chip is next to the heading
    await expect(
      page.getByRole("heading", { name: /Tolerances/ }).locator(".chip.rag"),
    ).toHaveAttribute("data-rag", "red");
    // a deviation that cannot be worked out is shown as unknown, never as a number
    await expect(page.getByTestId("tolerance-risk")).toContainText("Unknown");
    await expect(page.getByTestId("tolerance-time")).toContainText("112");
  });

  test("setting a tolerance sends the limit and the amber share for that dimension", async ({
    page,
  }) => {
    await signIn(page);
    await stub(page);
    const sent: Array<{ method: string; url: string; body: string | null }> =
      [];
    await page.route(
      `**/api/proxy/workforce-plans/${PLAN}/tolerances**`,
      (route) => {
        const request = route.request();
        if (request.method() !== "GET") {
          sent.push({
            method: request.method(),
            url: new URL(request.url()).pathname,
            body: request.postData(),
          });
          return route.fulfill({ json: {} });
        }
        return route.fulfill({ json: TOLERANCES });
      },
    );
    await page.goto("/en-001/planning");
    const row = page.getByTestId("tolerance-time");
    await row.getByLabel(/Tolerance \(largest deviation allowed\)/).fill("30");
    await row.getByLabel(/Amber from/).fill("50");
    await row.getByRole("button", { name: "Set tolerance" }).click();
    await expect.poll(() => sent.length).toBe(1);
    expect(sent[0]?.method).toBe("PUT");
    expect(sent[0]?.url).toMatch(/\/tolerances\/time$/);
    expect(JSON.parse(sent[0]?.body ?? "{}")).toEqual({
      limit: 30,
      amber_percent: 50,
    });
    await row.getByRole("button", { name: "Remove" }).click();
    await expect.poll(() => sent.length).toBe(2);
    expect(sent[1]?.method).toBe("DELETE");
  });

  test("a red breach with no escalation is flagged, can be escalated, raised and decided", async ({
    page,
  }) => {
    await signIn(page);
    await stub(page);
    // the plan's tolerances change as the escalation does: first unanswered, then open at level 1
    let open: Record<string, unknown> | null = null;
    const view = () => ({
      ...TOLERANCES,
      breaches_without_escalation: open ? 0 : 1,
      dimensions: TOLERANCES.dimensions.map((d) =>
        d.dimension === "resources" ? { ...d, escalation: open } : d,
      ),
    });
    const sent: Array<{ method: string; path: string; body: string | null }> =
      [];
    await page.route(
      `**/api/proxy/workforce-plans/${PLAN}/tolerances`,
      (route) => route.fulfill({ json: view() }),
    );
    await page.route(
      `**/api/proxy/workforce-plans/${PLAN}/escalations`,
      (route) => {
        const request = route.request();
        if (request.method() === "GET") return route.fulfill({ json: [] });
        sent.push({
          method: "POST",
          path: new URL(request.url()).pathname,
          body: request.postData(),
        });
        open = {
          pid: "e1",
          plan_pid: PLAN,
          plan_name: null,
          dimension: "resources",
          status: "open",
          level: "plan_owner",
          level_number: 1,
          raised_by: null,
          raised_on: "2026-10-20",
          level_raised_on: null,
          days_open: 3,
          note: null,
          at_raise: { rag: "red", deviation: 4000, limit: 3000 },
          decision: null,
          decided_by: null,
          decided_on: null,
          decision_note: null,
          at_decision: null,
        };
        return route.fulfill({ json: open });
      },
    );
    await page.route("**/api/proxy/escalations/e1/**", (route) => {
      const request = route.request();
      sent.push({
        method: request.method(),
        path: new URL(request.url()).pathname,
        body: request.postData(),
      });
      return route.fulfill({ json: {} });
    });
    await page.goto("/en-001/planning");

    // the breach is flagged, and only a red or amber dimension offers to escalate
    await expect(page.getByTestId("breaches-without-escalation")).toContainText(
      "1 red with no escalation",
    );
    await expect(
      page.getByTestId("escalation-cost").getByRole("button"),
    ).toHaveCount(0); // green
    await expect(
      page.getByTestId("escalation-risk").getByRole("button"),
    ).toHaveCount(0); // unknown
    const cell = page.getByTestId("escalation-resources");
    await cell.getByRole("button", { name: "Escalate" }).click();
    await cell
      .getByLabel(/Note \(no personal details\)/)
      .fill("the gap is wider than agreed");
    await cell.getByRole("button", { name: "Escalate" }).last().click();
    await expect.poll(() => sent.length).toBe(1);
    expect(sent[0]?.path).toMatch(/\/escalations$/);
    expect(JSON.parse(sent[0]?.body ?? "{}")).toEqual({
      dimension: "resources",
      note: "the gap is wider than agreed",
    });

    // now it is open at level 1: the banner goes, the level and the days open show, and it can go up or be decided
    await expect(page.getByTestId("breaches-without-escalation")).toHaveCount(
      0,
    );
    await expect(cell).toContainText("Level 1: Plan owner");
    await expect(cell).toContainText("3 calendar days open");
    await cell.getByRole("button", { name: "Raise level" }).click();
    await expect.poll(() => sent.length).toBe(2);
    expect(sent[1]?.path).toMatch(/\/escalations\/e1\/raise-level$/);

    await cell.getByRole("button", { name: "Decide" }).click();
    await page
      .getByTestId("plan-tolerances")
      .screenshot({ path: "test-results/escalation-open.png" });
    const submit = cell.getByRole("button", { name: "Decide" }).last();
    await expect(submit).toBeDisabled(); // a reason is required
    await cell.getByLabel("Decision").selectOption("change_plan");
    await cell.getByLabel(/Why \(required/).fill("bring forward the intake");
    await submit.click();
    await expect.poll(() => sent.length).toBe(3);
    expect(sent[2]?.path).toMatch(/\/escalations\/e1\/decide$/);
    expect(JSON.parse(sent[2]?.body ?? "{}")).toEqual({
      decision: "change_plan",
      note: "bring forward the intake",
    });
  });
});
