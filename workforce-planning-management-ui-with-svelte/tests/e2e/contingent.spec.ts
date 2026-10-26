// Contingent workforce (WPM-T143): the HR aggregate view at /contingent, and the engagement panel on a
// worker. The API is stubbed per test; an unstubbed /api/proxy call 404s loudly. Screenshots go to
// test-results/ (light and dark, desktop and phone) and should be looked at.

import { expect, test, type Page } from "@playwright/test";

async function signIn(page: Page) {
  await page.context().addCookies([
    {
      name: "__Host-mxi_session",
      value: "contingent-spec-session",
      domain: "localhost",
      path: "/",
      httpOnly: true,
      secure: true,
      sameSite: "Lax",
    },
  ]);
}

const WORKER = {
  pid: "11111111-1111-4111-8111-111111111111",
  person_ref: "person:22222222-2222-4222-8222-222222222222",
  upstream_worker_ref: null,
  organization_ref: "organization:33333333-3333-4333-8333-333333333333",
  worker_number: "C-0042",
  display_name: "Test Worker 042",
  status: "active",
  employment_type: "contractor",
  fte_percent: 100,
  department: "engineering",
  job_title: "Engineer",
  location: null,
  manager_pid: null,
  salary_minor: null,
  salary_currency: null,
  hired_on: "2026-01-05",
  terminated_on: null,
  engagement_ends_on: "2026-12-31",
};

const VIEW = {
  small_group_floor: 10,
  headcount: 120,
  fte: 111.5,
  contractor_percent_of_fte: 8.3,
  fixed_term_percent_of_fte: 12.5,
  departments_shown: false,
  by_department: [],
  funding_ends: [
    { month: "already_ended", people: 0, fte: 0 },
    { month: "2026-10", people: null, fte: null },
    { month: "2026-11", people: 14, fte: 12.5 },
  ],
  trend: {
    status: "insufficient_history",
    points: [],
    dates_left_out_without_split: 2,
    dates_left_out_below_floor: 0,
  },
  engagements: {
    ending_within_30_calendar_days: 12,
    ending_31_to_60_calendar_days: null,
    ending_61_to_90_calendar_days: 0,
    past_end_with_no_decision: null,
    extended_more_than: { times: 2, people: 0 },
    contractors_engaged_longer_than: { calendar_months: 24, people: 11 },
    note: "",
  },
  derivation: "",
};

const ROLLUP = {
  small_group_floor: 10,
  withheld_programmes: 2,
  programmes: [
    {
      programme: "thing:aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
      months: [
        {
          month: "2026-10",
          funded_fte: 14,
          contractor_percent_of_fte: 21.4,
          fixed_term_percent_of_fte: 14.3,
          engagements_ending: null,
          funding_ending: 0,
        },
        {
          month: "2026-11",
          funded_fte: null,
          contractor_percent_of_fte: null,
          fixed_term_percent_of_fte: null,
          engagements_ending: 10,
          funding_ending: 10,
        },
      ],
    },
  ],
  derivation: "",
};

// One worker for this file: it loads the worker page (about forty stubbed requests) several times, and
// running those beside the rest of the suite starved the dashboard specs of time.
test.describe.configure({ mode: "serial" });

test.describe("contingent workforce", () => {
  test.beforeEach(async ({ page }) => {
    await signIn(page);
    await page.route("**/api/proxy/**", (route) =>
      route.fulfill({
        status: 404,
        body: "unstubbed: " + route.request().url(),
      }),
    );
  });

  async function stubView(page: Page) {
    await page.route(
      "**/api/proxy/workforce-intelligence/contingent",
      (route) => route.fulfill({ json: VIEW }),
    );
    await page.route(
      "**/api/proxy/workforce-intelligence/programmes",
      (route) => route.fulfill({ json: ROLLUP }),
    );
  }

  test("shows the shares, withholds small counts rather than showing 0, and names no one until asked", async ({
    page,
  }) => {
    await stubView(page);
    const asked: string[] = [];
    await page.route("**/api/proxy/engagements/contingent-list**", (route) => {
      asked.push(new URL(route.request().url()).searchParams.get("list") ?? "");
      return route.fulfill({
        json: {
          list: "ending_within_30",
          people: [
            {
              worker_pid: WORKER.pid,
              worker_number: "C-0042",
              department: "engineering",
              employment_type: "contractor",
              hired_on: "2026-01-05",
              engagement_ends_on: "2026-10-20",
              extensions: 0,
              decision_recorded: false,
            },
          ],
        },
      });
    });
    await page.goto("/contingent");
    await expect(page.locator("h1")).toHaveText("Contingent workforce");
    await expect(page.getByTestId("share-contractor")).toHaveText("8.3%");
    await expect(page.getByTestId("share-fixed-term")).toHaveText("12.5%");
    // A count of 1–9 is withheld — shown as such, never as 0 — and a real zero is shown as 0.
    await expect(page.getByTestId("figure-ending_within_30-count")).toHaveText(
      "12",
    );
    await expect(page.getByTestId("figure-ending_31_to_60-count")).toHaveText(
      "withheld",
    );
    await expect(page.getByTestId("figure-ending_61_to_90-count")).toHaveText(
      "0",
    );
    await expect(
      page.getByTestId("figure-past_end_undecided-count"),
    ).toHaveText("withheld");
    // Departments are withheld, and the trend says it has too little history.
    await expect(page.getByTestId("departments-withheld")).toBeVisible();
    await expect(page.getByTestId("trend-insufficient")).toBeVisible();
    // Nobody is named until a person asks, and the ask is for one list.
    await expect(page.getByTestId("contingent-who")).toHaveCount(0);
    await page.getByTestId("show-ending_within_30").click();
    await expect(page.getByTestId("contingent-who-table")).toContainText(
      "C-0042",
    );
    await expect(page.getByTestId("contingent-who-table")).not.toContainText(
      "Test Worker",
    );
    expect(asked).toEqual(["ending_within_30"]);
    await page.getByTestId("show-ending_within_30").click();
    await expect(page.getByTestId("contingent-who")).toHaveCount(0);
    // Programmes: a small month is withheld, and smaller programmes are counted, not shown.
    await expect(page.getByTestId("programmes-withheld")).toContainText("2");
    const programme = page.getByTestId("contingent-programmes");
    await expect(programme).toContainText("withheld");
    await expect(programme).toContainText("14.00");
  });

  test("a trend is drawn from the points when there is enough history", async ({
    page,
  }) => {
    await page.route(
      "**/api/proxy/workforce-intelligence/contingent",
      (route) =>
        route.fulfill({
          json: {
            ...VIEW,
            departments_shown: true,
            by_department: [
              {
                department: "engineering",
                headcount: 60,
                contractor_percent_of_fte: 10,
                fixed_term_percent_of_fte: 5,
              },
            ],
            trend: {
              status: "ok",
              points: [
                {
                  as_of: "2026-01-01",
                  contractor_percent_of_fte: 7.5,
                  fixed_term_percent_of_fte: 12,
                },
                {
                  as_of: "2026-03-01",
                  contractor_percent_of_fte: 8,
                  fixed_term_percent_of_fte: 12.2,
                },
                {
                  as_of: "2026-05-01",
                  contractor_percent_of_fte: 8.3,
                  fixed_term_percent_of_fte: 12.5,
                },
              ],
              dates_left_out_without_split: 0,
              dates_left_out_below_floor: 0,
            },
          },
        }),
    );
    await page.route(
      "**/api/proxy/workforce-intelligence/programmes",
      (route) =>
        route.fulfill({
          json: { ...ROLLUP, withheld_programmes: 0, programmes: [] },
        }),
    );
    await page.goto("/contingent");
    await expect(page.getByTestId("trend-table")).toContainText("2026-05-01");
    await expect(page.getByTestId("contingent-departments")).toContainText(
      "engineering",
    );
    await expect(page.getByTestId("programmes-none")).toBeVisible();
    await expect(page.getByTestId("programmes-withheld")).toHaveCount(0);
  });

  test("anyone but HR is told it is not theirs to see", async ({ page }) => {
    await page.route(
      "**/api/proxy/workforce-intelligence/contingent",
      (route) => route.fulfill({ status: 403, json: { error: "forbidden" } }),
    );
    await page.route(
      "**/api/proxy/workforce-intelligence/programmes",
      (route) => route.fulfill({ status: 403, json: { error: "forbidden" } }),
    );
    await page.goto("/contingent");
    await expect(page.getByTestId("contingent-forbidden")).toBeVisible();
    await expect(page.getByTestId("contingent-share")).toHaveCount(0);
  });

  test("the page is translated and keeps the locale in its links", async ({
    page,
  }) => {
    await stubView(page);
    await page.goto("/de-001/contingent");
    await expect(page.locator("h1")).toHaveText("Externe Beschäftigte");
    await expect(page.getByTestId("figure-ending_31_to_60-count")).toHaveText(
      "zurückgehalten",
    );
  });

  async function stubWorker(page: Page, extra: (p: Page) => Promise<void>) {
    // The worker page itself reads these lists; every other panel on it is left unstubbed (404) and
    // shows its own state, so this spec depends only on the panel under test.
    for (const list of [
      "onboarding",
      "leave-entitlements",
      "leave-requests",
      "payslips",
      "reviews",
      "training-enrollments",
      "appraisals",
      "appraisal-requests",
      "notifications",
      "ergonomic-assessments",
      "adjustment-requests",
    ]) {
      await page.route(`**/api/proxy/workers/${WORKER.pid}/${list}`, (route) =>
        route.fulfill({ json: [] }),
      );
    }
    await page.route(
      `**/api/proxy/workers/${WORKER.pid}/wellbeing-prompts`,
      (route) => route.fulfill({ json: { prompts: [] } }),
    );
    await page.route("**/api/proxy/pulse-surveys", (route) =>
      route.fulfill({ json: [] }),
    );
    await page.route(`**/api/proxy/workers/${WORKER.pid}`, (route) =>
      route.fulfill({ json: WORKER }),
    );
    await extra(page);
  }

  test("a contractor's engagement shows the end, the funding mismatch and a masked rate", async ({
    page,
  }, info) => {
    await stubWorker(page, async (p) => {
      await p.route(`**/api/proxy/workers/${WORKER.pid}/engagement`, (route) =>
        route.fulfill({
          json: {
            employment_type: "contractor",
            hired_on: "2026-01-05",
            engagement_ends_on: "2026-12-31",
            standing: "ending_soon",
            window_calendar_days: 90,
            extension_count: 1,
            extensions: [
              {
                pid: "e1",
                previous_end: "2026-06-30",
                new_end: "2026-12-31",
                reason: "project slipped",
                decided_by: null,
              },
            ],
            decision: null,
            decisions: [],
          },
        }),
      );
      await p.route(`**/api/proxy/workers/${WORKER.pid}/funding`, (route) =>
        route.fulfill({
          json: {
            funding: {
              kind: "time_limited",
              programme_ref: "thing:aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
              ends_on: "2026-09-30",
            },
            contract_ends_on: "2026-12-31",
            alignment: "contract_outlasts_funding",
            needs_attention: true,
          },
        }),
      );
      await p.route(
        `**/api/proxy/workers/${WORKER.pid}/contractor-details`,
        (route) =>
          route.fulfill({
            json: {
              contractor_details: {
                supplier_ref:
                  "organization:44444444-4444-4444-8444-444444444444",
                route: "statement_of_work",
                rate_minor: null,
                rate_currency: null,
                rate_basis: "day",
                rate_recorded: true,
                rate_masked: true,
              },
            },
          }),
      );
    });
    await page.goto(`/workers/${WORKER.pid}`);
    const panel = page.getByTestId("engagement");
    await expect(panel).toBeVisible();
    await expect(page.getByTestId("engagement-basis")).toHaveText("contractor");
    await expect(page.getByTestId("engagement-ends")).toHaveText("2026-12-31");
    await expect(page.getByTestId("engagement-standing")).toContainText(
      "ending soon",
    );
    await expect(page.getByTestId("engagement-extensions")).toContainText(
      "project slipped",
    );
    // The funding ends before the contract: flagged with an icon and words, not colour alone.
    await expect(page.getByTestId("engagement-alignment")).toContainText(
      "the contract outlasts its funding",
    );
    await expect(page.getByTestId("engagement-alignment")).toContainText("⚠");
    // The rate is masked: a state, never a zero.
    await expect(page.getByTestId("engagement-rate")).toHaveText(
      "Hidden from you",
    );
    await expect(panel).not.toContainText("0.00");
    await panel.screenshot({
      path: `test-results/engagement-${info.project.name}-light.png`,
    });
  });

  test("a permanent worker with nothing to watch has no engagement panel, and so does anyone refused", async ({
    page,
  }) => {
    await stubWorker(page, async (p) => {
      await p.route(`**/api/proxy/workers/${WORKER.pid}/engagement`, (route) =>
        route.fulfill({
          json: {
            employment_type: "permanent",
            hired_on: "2020-01-05",
            engagement_ends_on: null,
            standing: "open_ended",
            window_calendar_days: 90,
            extension_count: 0,
            extensions: [],
            decision: null,
            decisions: [],
          },
        }),
      );
      await p.route(`**/api/proxy/workers/${WORKER.pid}/funding`, (route) =>
        route.fulfill({ status: 403, json: { error: "forbidden" } }),
      );
    });
    await page.goto(`/workers/${WORKER.pid}`);
    await expect(page.getByTestId("facts")).toBeVisible();
    await expect(page.getByTestId("engagement")).toHaveCount(0);
  });

  test("an engagement the caller may not see renders nothing and no error", async ({
    page,
  }) => {
    await stubWorker(page, async (p) => {
      await p.route(`**/api/proxy/workers/${WORKER.pid}/engagement`, (route) =>
        route.fulfill({ status: 403, json: { error: "forbidden" } }),
      );
    });
    await page.goto(`/workers/${WORKER.pid}`);
    await expect(page.getByTestId("facts")).toBeVisible();
    await expect(page.getByTestId("engagement")).toHaveCount(0);
    await expect(page.getByTestId("engagement-error")).toHaveCount(0);
  });
});

for (const [scheme, size, label] of [
  ["light", { width: 1280, height: 900 }, "desktop"],
  ["dark", { width: 1280, height: 900 }, "desktop"],
  ["light", { width: 390, height: 844 }, "phone"],
  ["dark", { width: 390, height: 844 }, "phone"],
] as const) {
  test.describe(`${scheme} ${label}`, () => {
    test.use({ colorScheme: scheme, viewport: size });

    test("the view and the engagement panel fit without the page scrolling sideways", async ({
      page,
    }) => {
      await signIn(page);
      await page.route("**/api/proxy/**", (route) =>
        route.fulfill({
          status: 404,
          body: "unstubbed: " + route.request().url(),
        }),
      );
      await page.route(
        "**/api/proxy/workforce-intelligence/contingent",
        (route) => route.fulfill({ json: VIEW }),
      );
      await page.route(
        "**/api/proxy/workforce-intelligence/programmes",
        (route) => route.fulfill({ json: ROLLUP }),
      );
      await page.route("**/api/proxy/engagements/contingent-list**", (route) =>
        route.fulfill({
          json: {
            list: "ending_within_30",
            people: [
              {
                worker_pid: WORKER.pid,
                worker_number: "C-0042",
                department: "engineering",
                employment_type: "contractor",
                hired_on: "2026-01-05",
                engagement_ends_on: "2026-10-20",
                extensions: 0,
                decision_recorded: false,
              },
            ],
          },
        }),
      );
      await page.goto("/contingent");
      await expect(page.getByTestId("contingent-share")).toBeVisible();
      await page.getByTestId("show-ending_within_30").click();
      await expect(page.getByTestId("contingent-who-table")).toBeVisible();
      const overflow = await page.evaluate(
        () =>
          document.documentElement.scrollWidth -
          document.documentElement.clientWidth,
      );
      expect(overflow).toBeLessThanOrEqual(0);
      await page.screenshot({
        path: `test-results/contingent-${scheme}-${label}.png`,
        fullPage: true,
      });
    });
  });
}

test.describe("planning by basis and programme", () => {
  const PLAN = "99999999-9999-4999-8999-999999999999";
  const PROGRAMME = "thing:bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb";
  const LINE = {
    pid: "88888888-8888-4888-8888-888888888888",
    department: "engineering",
    role_profile_pid: null,
    job_title: null,
    target_on: "2027-06-30",
    target_headcount: 3,
    target_fte: 2.5,
    basis: "fixed_term",
    expected_ends_on: "2027-12-31",
    programme_ref: PROGRAMME,
    note: null,
    objective_pids: [],
  };

  test("a demand line carries basis, FTE and programme to the API, and the forecast shows them", async ({
    page,
  }) => {
    await signIn(page);
    await page.route("**/api/proxy/**", (route) =>
      route.fulfill({
        status: 404,
        body: "unstubbed: " + route.request().url(),
      }),
    );
    const plan = {
      pid: PLAN,
      name: "Basis plan",
      organization_ref: "organization:33333333-3333-4333-8333-333333333333",
      horizon_start: "2026-01-01",
      horizon_end: "2028-12-31",
      rationale: null,
      attrition_bp: 0,
      budget_minor: null,
      budget_currency: null,
      on_cost_bp: null,
      status: "draft",
      demand_lines: [LINE],
      objectives: [],
    };
    await page.route("**/api/proxy/role-profiles", (route) =>
      route.fulfill({ json: [] }),
    );
    await page.route("**/api/proxy/workforce-plans", (route) =>
      route.fulfill({
        json: [{ ...plan, demand_lines: 1 }],
      }),
    );
    await page.route(`**/api/proxy/workforce-plans/${PLAN}`, (route) =>
      route.fulfill({ json: plan }),
    );
    await page.route(
      `**/api/proxy/workforce-plans/${PLAN}/alignment`,
      (route) =>
        route.fulfill({
          json: {
            aligned_share: null,
            unresourced_objectives: [],
            unaligned_demand_lines: [],
            objectives: 0,
            critical_roles_without_bench: {
              in_plan_departments: 0,
              all_departments: 0,
            },
          },
        }),
    );
    await page.route(`**/api/proxy/workforce-plans/${PLAN}/tolerances`, (route) =>
      route.fulfill({
        json: { derivation: "", as_of: "2026-10-20", overall: "not_set", breaches_without_escalation: 0, dimensions: [] },
      }),
    );
    await page.route(`**/api/proxy/workforce-plans/${PLAN}/escalations`, (route) =>
      route.fulfill({ json: [] }),
    );
    await page.route(`**/api/proxy/workforce-plans/${PLAN}/cost`, (route) =>
      route.fulfill({
        json: {
          derivation: "",
          currency: "GBP",
          salary_visible: false,
          assumptions: { attrition_bp: 0, on_cost_bp: 0 },
          groups: [],
          uncosted_groups: 0,
        },
      }),
    );
    await page.route(`**/api/proxy/workforce-plans/${PLAN}/forecast`, (route) =>
      route.fulfill({
        json: {
          derivation: "",
          as_of: "2026-10-11",
          assumptions: {
            attrition_bp: 0,
            attrition_source: "plan_assumption",
            hires_assumed: 0,
          },
          programmes: [
            {
              target_on: "2027-06-30",
              programme_ref: PROGRAMME,
              headcount: 3,
              fte: 2.5,
            },
            {
              target_on: "2027-06-30",
              programme_ref: null,
              headcount: 1,
              fte: null,
            },
          ],
          departments: [
            {
              department: "engineering",
              target_on: "2027-06-30",
              days_ahead: 262,
              opening_headcount: 3,
              planned_demand: 4,
              planned_demand_fte: null,
              demand_by_basis: [
                { basis: "permanent", headcount: 1, fte: null },
                { basis: "fixed_term", headcount: 3, fte: 2.5 },
              ],
              demand_by_programme: [],
              projected_supply: 2,
              projected_supply_fte: 2,
              supply_by_basis: {
                permanent: {
                  headcount: 1,
                  fte: 1,
                  no_end_date_counted_as_continuing: 0,
                  leaving_before_the_date: 0,
                },
                fixed_term: {
                  headcount: 0,
                  fte: 0,
                  no_end_date_counted_as_continuing: 0,
                  leaving_before_the_date: 1,
                },
                contractor: {
                  headcount: 1,
                  fte: 1,
                  no_end_date_counted_as_continuing: 0,
                  leaving_before_the_date: 0,
                },
                intern: {
                  headcount: 0,
                  fte: 0,
                  no_end_date_counted_as_continuing: 0,
                  leaving_before_the_date: 0,
                },
              },
              supply_assumptions: [
                "No hires are assumed.",
                "Fixed-term engagements leave the day after their end date.",
              ],
              headcount_gap: 2,
              fte_gap: null,
              levers: [
                "hire",
                "convert_contingent",
                "fixed_term_hire",
                "engage_contractor",
              ],
              conversions: {
                agreed: { headcount: 1, fte: 1 },
                hoped: { headcount: 2, fte: null },
              },
              competency_gaps: [],
            },
          ],
        },
      }),
    );
    const sent: Array<Record<string, unknown>> = [];
    await page.route(
      `**/api/proxy/workforce-plans/${PLAN}/demand-lines`,
      (route) => {
        sent.push(route.request().postDataJSON());
        return route.fulfill({ json: { pid: LINE.pid } });
      },
    );
    await page.goto("/planning");
    // The forecast: by basis (planned → projected), the shortfall levers in words, and demand by programme.
    await expect(page.getByTestId("forecast-basis")).toContainText(
      "fixed term: 3 → 0",
    );
    await expect(page.getByTestId("plan-forecast")).toContainText(
      "fixed-term hire",
    );
    await expect(page.getByTestId("plan-forecast")).toContainText(
      "engage a contractor",
    );
    await expect(page.getByTestId("plan-forecast")).toContainText(
      "convert contingent to permanent",
    );
    // Conversion plans (WPM-T164): agreed and hoped apart, and a hoped FTE that is unknown is not shown as 0.
    await expect(page.getByTestId("conversions-agreed")).toContainText(
      "agreed: 1 (1 FTE)",
    );
    await expect(page.getByTestId("conversions-hoped")).toHaveText(
      /hoped: 2\s*$/,
    );
    await expect(page.getByTestId("forecast-conversions")).toContainText(
      "Neither changes the projected supply",
    );
    await expect(page.getByTestId("plan-forecast-programmes")).toContainText(
      PROGRAMME,
    );
    await expect(page.getByTestId("plan-forecast-programmes")).toContainText(
      "No programme",
    );
    // The unknown FTE is a dash, not a zero.
    await expect(page.getByTestId("forecast-fte-gap")).toHaveCount(0);
    // Adding a line sends the basis, the FTE in percent, the end and the programme; a permanent line sends no end.
    await page.getByTestId("line-department").fill("operations");
    await page.getByTestId("line-target").fill("2027-03-31");
    await page.getByTestId("line-fte").fill("1.5");
    await page.getByTestId("line-basis").selectOption("contractor");
    await page.getByTestId("line-ends").fill("2027-09-30");
    await page.getByTestId("line-programme").fill(PROGRAMME);
    await page.getByTestId("line-add").click();
    await expect.poll(() => sent.length).toBe(1);
    expect(sent[0]).toMatchObject({
      department: "operations",
      basis: "contractor",
      target_fte_percent: 150,
      expected_ends_on: "2027-09-30",
      programme_ref: PROGRAMME,
    });
    await page.getByTestId("line-basis").selectOption("permanent");
    await expect(page.getByTestId("line-ends")).toHaveCount(0);
    // the page reloads after the first add and can redraw the form over a fill: clear until it holds
    await expect(async () => {
      await page.getByTestId("line-fte").fill("");
      await page.getByTestId("line-programme").fill("");
      await expect(page.getByTestId("line-fte")).toHaveValue("", { timeout: 500 });
      await expect(page.getByTestId("line-programme")).toHaveValue("", { timeout: 500 });
    }).toPass();
    await page.getByTestId("line-add").click();
    await expect.poll(() => sent.length).toBe(2);
    expect(sent[1]).toMatchObject({ basis: "permanent" });
    expect(sent[1]).not.toHaveProperty("expected_ends_on");
    expect(sent[1]).not.toHaveProperty("target_fte_percent");
    expect(sent[1]).not.toHaveProperty("programme_ref");
  });
});

test.describe("requisitions by basis and programme", () => {
  const ORG = "organization:33333333-3333-4333-8333-333333333333";
  const PROGRAMME = "thing:cccccccc-cccc-4ccc-8ccc-cccccccccccc";

  test("a fixed-term requisition needs a duration, and the board shows basis, duration and programme", async ({
    page,
  }) => {
    await signIn(page);
    await page.route("**/api/proxy/**", (route) =>
      route.fulfill({
        status: 404,
        body: "unstubbed: " + route.request().url(),
      }),
    );
    await page.route("**/api/proxy/me/organizations", (route) =>
      route.fulfill({ json: [] }),
    );
    await page.route("**/api/proxy/me/organizations/scope", (route) =>
      route.fulfill({ json: [ORG] }),
    );
    const made: Array<Record<string, unknown>> = [];
    await page.route(/\/api\/proxy\/requisitions(\?limit=500)?$/, (route) => {
      if (route.request().method() === "POST") {
        const body = route.request().postDataJSON();
        made.push(body);
        return route.fulfill({ json: { pid: "r-new" } });
      }
      return route.fulfill({
        json: made.map((b, i) => ({
          pid: `r${i}`,
          organization_ref: ORG,
          department: b.department,
          job_title: b.job_title,
          headcount: b.headcount,
          salary_min_minor: null,
          salary_max_minor: null,
          salary_currency: null,
          status: "draft",
          opened_on: null,
          basis: b.basis,
          expected_duration_months: b.expected_duration_months ?? null,
          programme_ref: b.programme_ref ?? null,
        })),
      });
    });
    await page.goto("/requisitions");
    await page.getByTestId("req-department").fill("research");
    await page.getByTestId("req-title").fill("Research Fellow");
    // A permanent post has no duration field; an ending one has, and it is required.
    await expect(page.getByTestId("req-months")).toHaveCount(0);
    await page.getByTestId("req-basis").selectOption("fixed_term");
    await expect(page.getByTestId("req-months")).toBeVisible();
    await page.getByTestId("req-create").click();
    await page.waitForTimeout(300);
    expect(
      made,
      "the browser refuses a fixed-term post with no duration",
    ).toHaveLength(0);
    await page.getByTestId("req-months").fill("18");
    await page.getByTestId("req-programme").fill(PROGRAMME);
    await page.getByTestId("req-create").click();
    await expect.poll(() => made.length).toBe(1);
    expect(made[0]).toMatchObject({
      organization_ref: ORG,
      department: "research",
      job_title: "Research Fellow",
      basis: "fixed_term",
      expected_duration_months: 18,
      programme_ref: PROGRAMME,
    });
    // The board reloads and the card says what it is.
    await expect(page.getByTestId("requisition-board")).toContainText(
      "Research Fellow",
    );
    await expect(page.getByTestId("requisition-board")).toContainText(
      "fixed term (18 months)",
    );
    await expect(page.getByTestId("requisition-board")).toContainText(
      PROGRAMME,
    );
    // A permanent post sends no duration.
    await page.getByTestId("req-department").fill("ops");
    await page.getByTestId("req-title").fill("Scheduler");
    // Even after a duration was typed and the basis changed back: it is not sent.
    await page.getByTestId("req-basis").selectOption("fixed_term");
    await page.getByTestId("req-months").fill("12");
    await page.getByTestId("req-basis").selectOption("permanent");
    await page.getByTestId("req-create").click();
    await expect.poll(() => made.length).toBe(2);
    expect(made[1]).toMatchObject({ basis: "permanent" });
    expect(made[1]).not.toHaveProperty("expected_duration_months");
    expect(made[1]).not.toHaveProperty("programme_ref");
  });
});

test.describe("conversion plans (WPM-T164)", () => {
  const PLAN = (over: Record<string, unknown> = {}) => ({
    pid: "cccccccc-0000-4000-8000-000000000001",
    worker_pid: WORKER.pid,
    intent: "convert",
    target_on: "2026-12-01",
    department: null,
    role_profile_ref: null,
    post_funding_kind: "core",
    post_funding_ends_on: null,
    reason: "Funded from April",
    proposed_by: "user:manager",
    approved_by: null,
    status: "proposed",
    review_on: null,
    settled_on: null,
    flags: [],
    ...over,
  });

  test.beforeEach(async ({ page }) => {
    await signIn(page);
    await page.route("**/api/proxy/**", (route) =>
      route.fulfill({
        status: 404,
        body: "unstubbed: " + route.request().url(),
      }),
    );
  });

  async function stubWorkerPage(page: Page) {
    for (const list of [
      "onboarding",
      "leave-entitlements",
      "leave-requests",
      "payslips",
      "reviews",
      "training-enrollments",
      "appraisals",
      "appraisal-requests",
      "notifications",
      "ergonomic-assessments",
      "adjustment-requests",
    ]) {
      await page.route(`**/api/proxy/workers/${WORKER.pid}/${list}`, (route) =>
        route.fulfill({ json: [] }),
      );
    }
    await page.route(
      `**/api/proxy/workers/${WORKER.pid}/wellbeing-prompts`,
      (route) => route.fulfill({ json: { prompts: [] } }),
    );
    await page.route("**/api/proxy/pulse-surveys", (route) =>
      route.fulfill({ json: [] }),
    );
    await page.route(`**/api/proxy/workers/${WORKER.pid}`, (route) =>
      route.fulfill({ json: WORKER }),
    );
  }

  test("HR proposes a plan, approves it, and carries it out; the buttons follow its status", async ({
    page,
  }) => {
    await stubWorkerPage(page);
    let plans: ReturnType<typeof PLAN>[] = [];
    const sent: unknown[] = [];
    const moves: string[] = [];
    await page.route(
      `**/api/proxy/workers/${WORKER.pid}/conversion-plans`,
      (route) => {
        if (route.request().method() === "POST") {
          sent.push(route.request().postDataJSON());
          plans = [PLAN()];
          return route.fulfill({ json: plans[0] });
        }
        return route.fulfill({ json: plans });
      },
    );
    await page.route("**/api/proxy/conversion-plans/*/*", (route) => {
      const verb = route.request().url().split("/").pop()!;
      moves.push(verb);
      plans = [
        PLAN({
          status: verb === "approve" ? "approved" : "done",
          approved_by: "user:hr",
          settled_on: verb === "done" ? "2026-10-18" : null,
        }),
      ];
      return route.fulfill({ json: plans[0] });
    });
    await page.goto(`/workers/${WORKER.pid}`);
    const panel = page.getByTestId("conversion");
    await expect(panel).toBeVisible();
    await expect(page.getByTestId("conversion-none")).toBeVisible();
    await expect(page.getByTestId("conversion-approve")).toHaveCount(0);
    await panel.screenshot({ path: "test-results/conversion-panel-empty.png" });

    await page.getByTestId("conversion-target-input").fill("2026-12-01");
    await page.getByTestId("conversion-funding-input").selectOption("core");
    await page.getByTestId("conversion-propose").click();
    await expect(page.getByTestId("conversion-status")).toHaveText("proposed");
    expect(sent).toEqual([
      { intent: "convert", target_on: "2026-12-01", post_funding_kind: "core" },
    ]);
    // One plan is open at a time: no form while it is.
    await expect(page.getByTestId("conversion-form")).toHaveCount(0);
    await expect(page.getByTestId("conversion-done")).toHaveCount(0);

    await page.getByTestId("conversion-approve").click();
    await expect(page.getByTestId("conversion-status")).toHaveText("approved");
    await expect(page.getByTestId("conversion-approve")).toHaveCount(0);
    await panel.screenshot({
      path: "test-results/conversion-panel-approved.png",
    });

    await page.getByTestId("conversion-done").click();
    expect(moves).toEqual(["approve", "done"]);
    // A done plan is history; the form is back only for a worker still on a contract (this stub's worker is).
    await expect(page.getByTestId("conversion-history")).toContainText(
      "convert to permanent",
    );
    await expect(page.getByTestId("conversion-open")).toHaveCount(0);
  });

  test("the service's refusal is shown, and a caller who may not read plans sees no panel", async ({
    page,
  }) => {
    await stubWorkerPage(page);
    await page.route(
      `**/api/proxy/workers/${WORKER.pid}/conversion-plans`,
      (route) => {
        if (route.request().method() === "POST") {
          return route.fulfill({
            status: 422,
            json: {
              description:
                "the worker already has an open plan; approve, carry out or abandon it first",
            },
          });
        }
        return route.fulfill({ json: [] });
      },
    );
    await page.goto(`/workers/${WORKER.pid}`);
    await page.getByTestId("conversion-target-input").fill("2026-12-01");
    await page.getByTestId("conversion-propose").click();
    await expect(page.getByTestId("conversion-action-error")).toContainText(
      "already has an open plan",
    );

    await page.unroute(`**/api/proxy/workers/${WORKER.pid}/conversion-plans`);
    await page.route(
      `**/api/proxy/workers/${WORKER.pid}/conversion-plans`,
      (route) =>
        route.fulfill({ status: 403, json: { description: "forbidden" } }),
    );
    await page.goto(`/workers/${WORKER.pid}`);
    await expect(
      page
        .getByTestId("engagement-basis")
        .or(page.getByRole("heading", { level: 1 })),
    ).toBeVisible();
    await expect(page.getByTestId("conversion")).toHaveCount(0);
    await expect(page.getByTestId("conversion-error")).toHaveCount(0);
  });

  test("a caller who may not read plans can still propose one, and never sees the plan back", async ({
    page,
  }) => {
    await stubWorkerPage(page);
    const sent: unknown[] = [];
    await page.route(
      `**/api/proxy/workers/${WORKER.pid}/conversion-plans`,
      (route) => {
        if (route.request().method() === "POST") {
          sent.push(route.request().postDataJSON());
          return route.fulfill({ json: PLAN() });
        }
        return route.fulfill({
          status: 403,
          json: { description: "forbidden" },
        });
      },
    );
    await page.goto(`/workers/${WORKER.pid}`);
    const panel = page.getByTestId("conversion-propose-only");
    await expect(panel).toBeVisible();
    await expect(page.getByTestId("conversion")).toHaveCount(0);
    await expect(page.getByTestId("conversion-error")).toHaveCount(0);
    await page.getByTestId("conversion-intent-input").selectOption("extend");
    await page.getByTestId("conversion-target-input").fill("2027-01-31");
    await page.getByTestId("conversion-propose").click();
    await expect(page.getByTestId("conversion-sent")).toBeVisible();
    expect(sent).toEqual([{ intent: "extend", target_on: "2027-01-31" }]);
    // Nothing of the plan is shown: this caller cannot read it.
    await expect(panel).not.toContainText("Funded from April");
    await panel.screenshot({
      path: "test-results/conversion-propose-only.png",
    });
    // A refusal from the service (not the line manager) is shown as it is.
    await page.unroute(`**/api/proxy/workers/${WORKER.pid}/conversion-plans`);
    await page.route(
      `**/api/proxy/workers/${WORKER.pid}/conversion-plans`,
      (route) =>
        route.request().method() === "POST"
          ? route.fulfill({
              status: 403,
              json: {
                description: "only the worker's line manager or HR decides",
              },
            })
          : route.fulfill({ status: 403, json: { description: "forbidden" } }),
    );
    await page.getByTestId("conversion-target-input").fill("2027-01-31");
    await page.getByTestId("conversion-propose").click();
    await expect(page.getByTestId("conversion-action-error")).toContainText(
      "line manager or HR",
    );
    await expect(page.getByTestId("conversion-sent")).toHaveCount(0);
  });

  test("the worker themselves is offered no way to propose their own plan", async ({
    page,
  }) => {
    await stubWorkerPage(page);
    await page.route("**/api/proxy/me/organizations", (route) =>
      route.fulfill({
        json: [{ worker_pid: WORKER.pid }],
      }),
    );
    await page.route("**/api/proxy/me/organizations/scope", (route) =>
      route.fulfill({ json: [] }),
    );
    await page.route(
      `**/api/proxy/workers/${WORKER.pid}/conversion-plans`,
      (route) =>
        route.fulfill({ status: 403, json: { description: "forbidden" } }),
    );
    await page.goto(`/workers/${WORKER.pid}`);
    await expect(page.getByRole("heading", { level: 1 })).toBeVisible();
    await expect(page.getByTestId("conversion-propose-only")).toHaveCount(0);
    await expect(page.getByTestId("conversion-form")).toHaveCount(0);
  });

  test("the contingent page lists open plans and what is past its end, only when asked", async ({
    page,
  }) => {
    await page.route(
      "**/api/proxy/workforce-intelligence/contingent",
      (route) => route.fulfill({ json: VIEW }),
    );
    await page.route(
      "**/api/proxy/workforce-intelligence/programmes",
      (route) => route.fulfill({ json: ROLLUP }),
    );
    let asked = 0;
    await page.route("**/api/proxy/conversion-plans?limit=500", (route) => {
      asked += 1;
      return route.fulfill({
        json: [
          {
            worker_pid: WORKER.pid,
            worker_number: "C-0042",
            department: "engineering",
            employment_type: "contractor",
            engagement_ends_on: "2026-10-01",
            plan: null,
            flags: ["past_end_without_approved_plan"],
          },
          {
            worker_pid: "11111111-1111-4111-8111-111111111112",
            worker_number: "C-0043",
            department: "design",
            employment_type: "fixed_term",
            engagement_ends_on: "2026-12-31",
            plan: PLAN({ worker_pid: "11111111-1111-4111-8111-111111111112" }),
            flags: [],
          },
        ],
      });
    });
    await page.goto("/contingent");
    await expect(page.getByTestId("contingent-conversions")).toBeVisible();
    expect(asked).toBe(0);
    await page.getByTestId("show-conversions").click();
    const table = page.getByTestId("conversion-table");
    await expect(table).toContainText("C-0042");
    await expect(table).toContainText("Past its end with no approved plan");
    await expect(table).toContainText("convert to permanent · proposed");
    // The reason is not in the list.
    await expect(table).not.toContainText("Funded from April");
    expect(asked).toBe(1);
    await page.getByTestId("show-conversions").click();
    await expect(table).toHaveCount(0);
  });
});
