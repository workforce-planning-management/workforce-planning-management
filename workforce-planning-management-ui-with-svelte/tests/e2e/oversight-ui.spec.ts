// Capacity, resilience, oversight (with the monthly pack) and cutover readiness (WPM-T118). The API is
// stubbed per test; an unstubbed /api/proxy call 404s loudly. Screenshots go to test-results/ (light and
// dark, desktop and phone) and should be looked at.

import { expect, test, type Page } from "@playwright/test";

const ORG = "organization:33333333-3333-4333-8333-333333333333";
const POOL = "77777777-7777-4777-8777-777777777777";
const PACK = "55555555-5555-4555-8555-555555555555";
const CUTOVER = "66666666-6666-4666-8666-666666666666";

async function signIn(page: Page) {
  await page.context().addCookies([
    {
      name: "__Host-mxi_session",
      value: "oversight-ui-spec-session",
      domain: "localhost",
      path: "/",
      httpOnly: true,
      secure: true,
      sameSite: "Lax",
    },
  ]);
  await page.route("**/api/proxy/**", (route) =>
    route.fulfill({ status: 404, body: "unstubbed: " + route.request().url() }),
  );
  await page.route("**/api/proxy/me/organizations", (route) =>
    route.fulfill({ json: [] }),
  );
  await page.route("**/api/proxy/me/organizations/scope", (route) =>
    route.fulfill({ json: [ORG] }),
  );
}

const cell = (
  month: string,
  status: string,
  supply: number,
  demand: number,
) => ({
  month,
  supply_centi: supply,
  demand_centi: demand,
  proposed_centi: 0,
  own_balance_centi: supply - demand,
  firm_partner_centi: 0,
  remaining_shortfall_centi: Math.max(demand - supply, 0),
  status,
  partners: [],
});

const CAPACITY = {
  organization_ref: ORG,
  from: "2026-10-01",
  months: ["2026-10-01", "2026-11-01", "2026-12-01", "2027-01-01"],
  pools: [
    {
      pid: POOL,
      name: "Platform engineers",
      operations_reservation_bp: 2000,
      members: 12,
      over_months: 1,
      unknown_months: 1,
      over_total_centi: 150,
      cells: [
        cell("2026-10-01", "fits", 800, 600),
        cell("2026-11-01", "fits_with_partners", 800, 900),
        cell("2026-12-01", "over", 800, 950),
        cell("2027-01-01", "unknown", 800, 900),
      ],
    },
  ],
  constraint_pool: POOL,
  wip: { pool_pid: POOL, active: 3, limit: 2 },
  derivation:
    "Supply is the FTE of the pool's members less the operations reservation.",
};

const KEY_PERSON = {
  single_points_of_failure: 2,
  skills: [
    {
      skill_pid: "s1",
      skill: "Database administration",
      minimum_proficiency: 4,
      department: null,
      holders: 1,
      backups: 0,
      ready_successors: 0,
      status: "single_point_of_failure",
    },
    {
      skill_pid: "s2",
      skill: "Incident command",
      minimum_proficiency: 3,
      department: null,
      holders: 5,
      backups: 3,
      ready_successors: 1,
      status: "shared",
    },
  ],
  basis:
    "A holder is an employed person whose declared proficiency meets the highest critical minimum.",
};

const GOVERNANCE = {
  organization_ref: ORG,
  commitments: 3,
  senior_hours_per_month: 18.5,
  all_hours_per_month: 31,
  roles: [
    { role: "Director", senior: true, commitments: 2, hours_per_month: 18.5 },
    { role: "Analyst", senior: false, commitments: 1, hours_per_month: 12.5 },
  ],
  basis: "An estimate from the entered commitments.",
};

const CRITERIA = [
  {
    criterion_pid: "c1",
    criterion: "Turnover is under control",
    status: "not_met",
    measures: [
      {
        measure_pid: "m1",
        name: "Turnover rate",
        origin: "derived",
        derivation: "turnover_rate",
        unit: "basis points",
        derived_values: null,
        threshold: 1200,
        direction: "at_most",
        status: "not_met",
        run: 2,
        required: 3,
        stopped_by: { reason: "null_reading", month: "2026-08" },
        reported_periods: 4,
      },
    ],
  },
];

const STATUS = {
  organization_ref: ORG,
  as_of_month: "2026-09",
  scale: [{ pid: "l1", ordinal: 1, name: "Level 2", synthetic: true }],
  current: {
    pid: "p1",
    level: "Level 2",
    ordinal: 1,
    effective_on: "2026-06-01",
    deciding_body: "Board committee",
    reason: "Exit criteria for level 1 were met.",
  },
  positions: [],
  criteria: CRITERIA,
  basis: "A missing or null reading is never a pass.",
};

const PACK_BODY = {
  pid: PACK,
  organization_ref: ORG,
  as_of: "2026-10-12",
  month: "2026-09-01",
  withdrawn: null,
  pack: {
    oversight: {
      ...STATUS,
      current: { ...STATUS.current, deciding_body: undefined },
    },
    capacity: {
      months: CAPACITY.months,
      pools: CAPACITY.pools.map((p) => ({
        name: p.name,
        over_months: p.over_months,
        unknown_months: p.unknown_months,
        cells: p.cells,
      })),
      constraint_pool: "Platform engineers",
      derivation: CAPACITY.derivation,
    },
    key_person: { ...KEY_PERSON, skills: [] },
    governance: {
      commitments: 3,
      senior_hours_per_month: 18.5,
      all_hours_per_month: 31,
      basis: "",
    },
    basis: "Frozen as of the date shown and never changed.",
  },
};

const READINESS = (
  readiness: string,
  trained: string,
  cover: string,
  risk: string,
) => ({
  cutover: {
    pid: CUTOVER,
    organization_ref: ORG,
    programme_ref: "thing:aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
    name: "Finance system go-live",
    site: "St Mary's Hospital",
    window_start: "2026-11-02",
    window_end: "2026-11-15",
    rota_pid: null,
    site_aliases: [],
  },
  readiness,
  checks: {
    trained_staff: {
      status: trained,
      reasons:
        trained === "ready"
          ? []
          : ["2 of 5 required people in the role are trained"],
      details:
        trained === "ready"
          ? []
          : [
              {
                code: "trained_short",
                params: {
                  name: "Operator",
                  trained: "2",
                  required: "5",
                  short: "3",
                },
              },
            ],
    },
    hypercare_cover: {
      status: cover,
      reasons:
        cover === "ready"
          ? []
          : ["no on-call rota is linked for the cutover window"],
      details: cover === "ready" ? [] : [{ code: "no_rota", params: {} }],
    },
    key_person_risk: {
      status: risk,
      reasons:
        risk === "ready"
          ? []
          : ["Database administration: one holder at the site"],
      details:
        risk === "ready"
          ? []
          : [
              {
                code: "key_single_holder",
                params: { name: "Database administration" },
              },
            ],
    },
  },
  basis: "The people side only.",
});

async function stubCapacity(page: Page) {
  await page.route("**/api/proxy/capacity?**", (route) =>
    route.fulfill({ json: CAPACITY }),
  );
}
async function stubResilience(page: Page) {
  await page.route(
    "**/api/proxy/workforce-intelligence/key-person**",
    (route) => route.fulfill({ json: KEY_PERSON }),
  );
  await page.route("**/api/proxy/governance-commitments/load?**", (route) =>
    route.fulfill({ json: GOVERNANCE }),
  );
}
async function stubOversight(page: Page, packs: unknown[] = []) {
  await page.route("**/api/proxy/oversight/status?**", (route) =>
    route.fulfill({ json: STATUS }),
  );
  await page.route("**/api/proxy/oversight/packs?**", (route) =>
    route.fulfill({ json: { organization_ref: ORG, packs } }),
  );
}
async function stubCutover(page: Page) {
  await page.route("**/api/proxy/cutovers?**", (route) =>
    route.fulfill({
      json: [READINESS("unknown", "ready", "unknown", "at_risk").cutover],
    }),
  );
  await page.route(`**/api/proxy/cutovers/${CUTOVER}/readiness`, (route) =>
    route.fulfill({
      json: READINESS("at_risk", "ready", "unknown", "at_risk"),
    }),
  );
}

test.describe("capacity", () => {
  test("marks an unknown month as unknown, not as a zero or a pass, with a word and a mark", async ({
    page,
  }) => {
    await signIn(page);
    await stubCapacity(page);
    await page.goto("/capacity");
    await expect(page.getByTestId("heatmap")).toBeVisible();
    const cells = page.getByTestId("heat-cell");
    await expect(cells).toHaveCount(4);
    await expect(cells.nth(3)).toHaveAttribute("data-status", "unknown");
    await expect(cells.nth(3)).toContainText("unknown");
    await expect(cells.nth(3)).toContainText("?");
    // Drawn as an unknown, not tinted as a pass.
    await expect(cells.nth(3)).toHaveClass(/unknown/);
    await expect(cells.nth(3)).not.toHaveClass(/fits/);
    await expect(cells.nth(2)).toContainText("over");
    await expect(cells.nth(0)).toContainText("8.00 / 6.00");
    await expect(page.getByTestId("constraint")).toContainText(
      "Platform engineers",
    );
    await expect(page.getByTestId("constraint")).toContainText("3 of 2");
  });
});

test.describe("resilience", () => {
  test("shows counts and hours, and names no one", async ({ page }) => {
    await signIn(page);
    await stubResilience(page);
    await page.goto("/resilience");
    await expect(page.getByTestId("spof-count")).toContainText("2");
    await expect(page.getByTestId("key-person-table")).toContainText(
      "Database administration",
    );
    await expect(page.getByTestId("key-person-table")).toContainText(
      "single point of failure",
    );
    await expect(page.getByTestId("senior-hours")).toContainText(
      "18.5 senior hours a month",
    );
    await expect(page.getByTestId("governance")).toContainText("Director");
    await expect(page.getByTestId("governance")).not.toContainText(
      "worker_number",
    );
  });
});

test.describe("oversight and the monthly pack", () => {
  test("shows the level and why a measure's run stopped; a missing reading is not a pass", async ({
    page,
  }) => {
    await signIn(page);
    await stubOversight(page);
    await page.goto("/oversight");
    await expect(page.getByTestId("level")).toHaveText("Level 2");
    await expect(page.getByTestId("measure")).toContainText("2 of 3 months");
    await expect(page.getByTestId("measure")).toContainText(
      "The reading for 2026-08 is missing",
    );
    await expect(page.getByTestId("measure")).toContainText("not met");
    await expect(page.getByTestId("no-packs")).toBeVisible();
  });

  test("a refusal to freeze is said in words; a frozen pack opens, and a withdrawal shows the mark", async ({
    page,
  }) => {
    await signIn(page);
    let frozen = false;
    let withdrawn: { on: string; reason: string } | null = null;
    await page.route("**/api/proxy/oversight/status?**", (route) =>
      route.fulfill({ json: STATUS }),
    );
    await page.route("**/api/proxy/oversight/packs?**", (route) =>
      route.fulfill({
        json: {
          organization_ref: ORG,
          packs: frozen
            ? [
                {
                  pid: PACK,
                  as_of: "2026-10-12",
                  month: "2026-09-01",
                  withdrawn,
                },
              ]
            : [],
        },
      }),
    );
    await page.route("**/api/proxy/oversight/packs", (route) =>
      route.fulfill({
        status: 403,
        json: { description: "freezing is for an administrator" },
      }),
    );
    await page.goto("/oversight");
    await page.getByTestId("freeze").click();
    await expect(page.getByTestId("not-allowed")).toBeVisible();
    await expect(page.getByTestId("no-packs")).toBeVisible();

    // The caller is now allowed.
    await page.unroute("**/api/proxy/oversight/packs");
    await page.route("**/api/proxy/oversight/packs", (route) => {
      frozen = true;
      return route.fulfill({ json: PACK_BODY });
    });
    await page.route(`**/api/proxy/oversight/packs/${PACK}`, (route) =>
      route.fulfill({ json: { ...PACK_BODY, withdrawn } }),
    );
    await page.route(
      `**/api/proxy/oversight/packs/${PACK}/withdraw`,
      (route) => {
        withdrawn = { on: "2026-10-12", reason: "wrong month" };
        return route.fulfill({ json: {} });
      },
    );
    await page.getByTestId("freeze").click();
    await expect(page.getByTestId("pack")).toHaveCount(1);
    await expect(page.getByTestId("not-allowed")).toHaveCount(0);
    await page.getByTestId("pack").getByRole("button").click();
    const detail = page.getByTestId("pack-detail");
    await expect(detail).toContainText("Turnover is under control");
    await expect(detail.getByTestId("heatmap")).toBeVisible();
    // The deciding body is left out of a frozen pack.
    await expect(detail).not.toContainText("Board committee");
    await detail.screenshot({ path: "test-results/oversight-pack-open.png" });
    await page.getByTestId("withdraw-reason").fill("wrong month");
    await page.getByTestId("withdraw").click();
    await expect(page.getByTestId("withdrawn")).toContainText("wrong month");
    await expect(detail).toContainText("Withdrawn 12 Oct 2026: wrong month");
    await expect(detail.getByTestId("withdraw")).toHaveCount(0);
  });
});

test.describe("cutover readiness", () => {
  test("shows each check and the worst of them, with the reasons, and never reads unknown as ready", async ({
    page,
  }) => {
    await signIn(page);
    await stubCutover(page);
    await page.goto("/cutover");
    await expect(page.getByTestId("overall")).toHaveAttribute(
      "data-status",
      "at_risk",
    );
    await expect(page.getByTestId("check-trained_staff")).toHaveAttribute(
      "data-status",
      "ready",
    );
    await expect(page.getByTestId("check-hypercare_cover")).toHaveAttribute(
      "data-status",
      "unknown",
    );
    await expect(page.getByTestId("check-hypercare_cover")).toContainText(
      "unknown",
    );
    await expect(page.getByTestId("check-hypercare_cover")).toContainText("?");
    await expect(page.getByTestId("check-hypercare_cover")).not.toContainText(
      "✓",
    );
    await expect(
      page.getByTestId("check-hypercare_cover").locator(".state"),
    ).toHaveClass(/unknown/);
    await expect(page.getByTestId("check-hypercare_cover")).toContainText(
      "No on-call rota",
    );
    await expect(page.getByTestId("check-key_person_risk")).toContainText(
      "one holder at the site",
    );
    await expect(page.getByTestId("check-trained_staff")).toContainText(
      "No concerns found.",
    );
  });
});

/** Catch the writes (not the reads) on a path; answer them with `reply`. Returns what was sent. */
async function record(
  page: Page,
  glob: string,
  reply: { status?: number; json?: unknown } = {},
) {
  const sent: Array<{ method: string; body: unknown }> = [];
  await page.route(glob, (route) => {
    const request = route.request();
    if (request.method() === "GET") return route.fallback();
    let body: unknown;
    try {
      body = request.postDataJSON();
    } catch {
      body = null;
    }
    sent.push({ method: request.method(), body });
    return route.fulfill({
      status: reply.status ?? 200,
      json: reply.json ?? {},
    });
  });
  return sent;
}

const ROLE_PROFILES = [{ pid: "rp1", job_title: "Operator" }];
const SKILL_POOLS = [
  {
    pid: POOL,
    organization_ref: ORG,
    name: "Platform engineers",
    operations_reservation_bp: 0,
    members: [],
  },
];

test.describe("planning a cutover", () => {
  test("sends what was typed, splits the aliases, and says a refusal in words", async ({
    page,
  }) => {
    await signIn(page);
    await page.route("**/api/proxy/cutovers?**", (route) =>
      route.fulfill({ json: [] }),
    );
    await page.route("**/api/proxy/role-profiles", (route) =>
      route.fulfill({ json: ROLE_PROFILES }),
    );
    await page.route("**/api/proxy/skill-pools?**", (route) =>
      route.fulfill({ json: SKILL_POOLS }),
    );
    const refused = await record(page, "**/api/proxy/cutovers", {
      status: 403,
      json: { description: "planning is for an administrator" },
    });
    await page.goto("/cutover");
    await expect(page.getByTestId("no-cutovers")).toBeVisible();
    await page
      .getByTestId("cut-programme")
      .fill("thing:aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa");
    await page.getByTestId("cut-name").fill("Finance go-live");
    await page.getByTestId("cut-site").fill("St Mary's Hospital");
    await page.getByTestId("cut-start").fill("2026-11-02");
    await page.getByTestId("cut-end").fill("2026-11-15");
    await page.getByTestId("cut-aliases").fill("St. Mary's,  Mary's ,");
    await page.getByTestId("plan-create").click();
    await expect(page.getByTestId("write-error")).toContainText(
      "You do not have the right to make this change.",
    );
    expect(refused).toHaveLength(1);
    expect(refused[0]!.body).toEqual({
      organization_ref: ORG,
      programme_ref: "thing:aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
      name: "Finance go-live",
      site: "St Mary's Hospital",
      window_start: "2026-11-02",
      window_end: "2026-11-15",
      rota_pid: null,
      site_aliases: ["St. Mary's", "Mary's"],
    });
  });

  test("adds a role and a pool, and removes the cutover after a confirmation", async ({
    page,
  }) => {
    await signIn(page);
    await stubCutover(page);
    await page.route("**/api/proxy/role-profiles", (route) =>
      route.fulfill({ json: ROLE_PROFILES }),
    );
    await page.route("**/api/proxy/skill-pools?**", (route) =>
      route.fulfill({ json: SKILL_POOLS }),
    );
    const roles = await record(page, `**/api/proxy/cutovers/${CUTOVER}/roles`);
    const pools = await record(page, `**/api/proxy/cutovers/${CUTOVER}/pools`);
    const removed = await record(page, `**/api/proxy/cutovers/${CUTOVER}`);
    await page.goto("/cutover");
    await expect(page.getByTestId("cutover")).toBeVisible();
    await page.getByText("Add a required role").click();
    await page.getByTestId("role-profile").selectOption("rp1");
    await page.getByTestId("role-course").fill("SYN-COURSE");
    await page.getByTestId("add-role").click();
    await expect.poll(() => roles.length).toBe(1);
    expect(roles[0]!.body).toEqual({
      role_profile_pid: "rp1",
      course_ref: "SYN-COURSE",
      required_headcount: 1,
    });
    await page.getByText("Add a pool it depends on").click();
    await page.getByTestId("pool-choice").selectOption(POOL);
    await page.getByTestId("add-pool").click();
    await expect.poll(() => pools.length).toBe(1);
    expect(pools[0]!.body).toEqual({ pool_pid: POOL });
    page.once("dialog", (dialog) => dialog.dismiss());
    await page.getByTestId("remove-cutover").click();
    expect(removed).toHaveLength(0);
    page.once("dialog", (dialog) => dialog.accept());
    await page.getByTestId("remove-cutover").click();
    await expect.poll(() => removed.length).toBe(1);
    expect(removed[0]!.method).toBe("DELETE");
  });

  test("says the reasons in the reader's language from the service's data, and in English when a code is new", async ({
    page,
  }) => {
    await signIn(page);
    await page.route("**/api/proxy/cutovers?**", (route) =>
      route.fulfill({
        json: [READINESS("at_risk", "ready", "unknown", "at_risk").cutover],
      }),
    );
    const body = READINESS("at_risk", "ready", "unknown", "at_risk");
    body.checks.key_person_risk.details = [
      { code: "a_code_the_page_does_not_know", params: {} },
    ];
    await page.route(`**/api/proxy/cutovers/${CUTOVER}/readiness`, (route) =>
      route.fulfill({ json: body }),
    );
    await page.goto("/cy-001/cutover");
    await expect(page.getByTestId("check-hypercare_cover")).toContainText(
      "Nid oes rota ar alwad wedi'i chysylltu",
    );
    // `unknown` has its own word here, not the pay comparison's "not enough to compare".
    await expect(page.getByTestId("check-hypercare_cover")).toContainText(
      "anhysbys",
    );
    // A code the page does not know shows the service's own text, and the page says that it is English.
    await expect(page.getByTestId("check-key_person_risk")).toContainText(
      "Database administration: one holder at the site",
    );
    await expect(page.getByText("Saesneg").first()).toBeVisible();
  });
});

test.describe("entering capacity", () => {
  test("converts percent and FTE to the service's units and sends them", async ({
    page,
  }) => {
    await signIn(page);
    await stubCapacity(page);
    await page.route("**/api/proxy/role-profiles", (route) =>
      route.fulfill({ json: ROLE_PROFILES }),
    );
    const pools = await record(page, "**/api/proxy/skill-pools");
    const members = await record(
      page,
      `**/api/proxy/skill-pools/${POOL}/members`,
    );
    const claims = await record(page, "**/api/proxy/programme-demands");
    await page.goto("/capacity");
    await expect(page.getByTestId("heatmap")).toBeVisible();
    await page.getByTestId("pool-name").fill("Analysts");
    await page.getByLabel("Operations reservation").fill("12.5");
    await page.getByTestId("create-pool").click();
    await expect.poll(() => pools.length).toBe(1);
    expect(pools[0]!.body).toEqual({
      organization_ref: ORG,
      name: "Analysts",
      operations_reservation_bp: 1250,
    });
    await page.getByTestId("member-pool").selectOption(POOL);
    await page.getByTestId("member-role").selectOption("rp1");
    await page.getByTestId("add-member").click();
    await expect.poll(() => members.length).toBe(1);
    expect(members[0]!.body).toEqual({ role_profile_pid: "rp1" });
    await page
      .getByTestId("claim-programme")
      .fill("thing:bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb");
    await page.getByTestId("claim-pool").selectOption(POOL);
    await page.getByTestId("claim-month").fill("2026-12-01");
    await page.getByTestId("claim-fte").fill("1.5");
    await page.getByTestId("save-claim").click();
    await expect.poll(() => claims.length).toBe(1);
    expect(claims[0]!.method).toBe("PUT");
    expect(claims[0]!.body).toEqual({
      programme_ref: "thing:bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
      pool_pid: POOL,
      month: "2026-12-01",
      fte_centi: 150,
    });
  });

  test("says so in the catalogue's words when a write is refused as invalid", async ({
    page,
  }) => {
    await signIn(page);
    await stubCapacity(page);
    await record(page, "**/api/proxy/skill-pools", {
      status: 422,
      json: { description: "name must not be blank" },
    });
    await page.goto("/capacity");
    await page.getByTestId("pool-name").fill("x");
    await page.getByTestId("create-pool").click();
    // The catalogue's words for a refusal, then the service's own message naming what was refused.
    // The catalogue's words, not the service's.
    await expect(page.getByTestId("write-error")).toContainText(
      "That was not accepted",
    );
    await expect(page.getByTestId("write-error")).not.toContainText(
      "name must not be blank",
    );
    await expect(page.getByTestId("write-error")).toContainText(
      "That was not accepted",
    );
  });

  test("the heatmap is a table a screen reader can read: headers, row names, a word in each cell", async ({
    page,
  }) => {
    await signIn(page);
    await stubCapacity(page);
    await page.goto("/capacity");
    const table = page.getByTestId("heatmap").getByRole("table");
    await expect(
      table.getByRole("columnheader", { name: "Dec 2026" }),
    ).toBeVisible();
    await expect(
      table.getByRole("rowheader", { name: /Platform engineers/ }),
    ).toBeVisible();
    await expect(table.getByRole("cell", { name: /unknown/ })).toHaveCount(1);
    await expect(
      table.getByRole("cell", { name: /fits with partners/ }),
    ).toHaveCount(1);
    // The mark is decoration; the word is what is read.
    await expect(
      table.getByRole("cell", { name: /unknown/ }),
    ).not.toContainText("✓");
  });
});

test.describe("entering oversight evidence", () => {
  const REPORTED = {
    ...STATUS,
    criteria: [
      {
        ...CRITERIA[0]!,
        measures: [
          {
            ...CRITERIA[0]!.measures[0]!,
            origin: "reported",
            derivation: null,
          },
        ],
      },
    ],
  };

  test("sends the scale, position, criterion, measure and reading as typed; a missing value carries no evidence", async ({
    page,
  }) => {
    await signIn(page);
    await page.route("**/api/proxy/oversight/status?**", (route) =>
      route.fulfill({ json: REPORTED }),
    );
    await page.route("**/api/proxy/oversight/packs?**", (route) =>
      route.fulfill({ json: { organization_ref: ORG, packs: [] } }),
    );
    await page.route("**/api/proxy/oversight/derivations", (route) =>
      route.fulfill({ json: { derivations: [] } }),
    );
    const levels = await record(page, "**/api/proxy/oversight/levels");
    const positions = await record(page, "**/api/proxy/oversight/positions");
    const criteria = await record(page, "**/api/proxy/oversight/criteria");
    const measures = await record(page, "**/api/proxy/oversight/measures");
    const readings = await record(
      page,
      "**/api/proxy/oversight/measures/m1/readings",
    );
    await page.goto("/oversight");
    await expect(page.getByTestId("manage-oversight")).toBeVisible();

    await page.getByTestId("level-ordinal").fill("3");
    await page.getByTestId("level-name").fill("Level 4");
    await page.getByTestId("add-level").click();
    await expect.poll(() => levels.length).toBe(1);
    expect(levels[0]!.body).toEqual({
      organization_ref: ORG,
      ordinal: 3,
      name: "Level 4",
    });

    await page.getByTestId("pos-level").selectOption("l1");
    await page.getByTestId("pos-on").fill("2026-10-01");
    await page.getByTestId("pos-body").fill("Board committee");
    await page.getByTestId("pos-reason").fill("Criteria met");
    await page.getByTestId("record-position").click();
    await expect.poll(() => positions.length).toBe(1);
    expect(positions[0]!.body).toEqual({
      organization_ref: ORG,
      level_pid: "l1",
      effective_on: "2026-10-01",
      deciding_body: "Board committee",
      reason: "Criteria met",
    });

    await page.getByTestId("crit-level").selectOption("l1");
    await page.getByTestId("crit-name").fill("Cover is adequate");
    await page.getByTestId("add-criterion").click();
    await expect.poll(() => criteria.length).toBe(1);
    expect(criteria[0]!.body).toEqual({
      level_pid: "l1",
      name: "Cover is adequate",
    });

    await page.getByTestId("meas-criterion").selectOption("c1");
    await page.getByTestId("meas-name").fill("Cover ratio");
    await page.getByTestId("meas-unit").fill("percent");
    await page.getByTestId("meas-threshold").fill("90");
    await page.getByTestId("add-measure").click();
    await expect.poll(() => measures.length).toBe(1);
    expect(measures[0]!.body).toEqual({
      criterion_pid: "c1",
      name: "Cover ratio",
      origin: "reported",
      unit: "percent",
      threshold: 90,
      direction: "at_most",
      sustain: 3,
    });

    // The data is missing for the month: a null value, with no evidence sent.
    await page.getByTestId("read-measure").selectOption("m1");
    await page.getByTestId("read-month").fill("2026-09-01");
    await page.getByTestId("save-reading").click();
    await expect.poll(() => readings.length).toBe(1);
    expect(readings[0]!.method).toBe("PUT");
    expect(readings[0]!.body).toEqual({ period_on: "2026-09-01", value: null });

    // A value carries its evidence note and source document.
    await page.getByTestId("read-value").fill("84");
    await page.getByTestId("read-evidence").fill("Rota audit");
    await page.getByTestId("read-source").fill("rota-audit-2026-09.pdf");
    await page.getByTestId("save-reading").click();
    await expect.poll(() => readings.length).toBe(2);
    expect(readings[1]!.body).toEqual({
      period_on: "2026-09-01",
      value: 84,
      evidence: "Rota audit",
      source_document: "rota-audit-2026-09.pdf",
    });
  });
});

test.describe("early warnings", () => {
  const WARNINGS = {
    organization_ref: ORG,
    warnings: [
      {
        pid: "w1",
        kind: "pool_over_committed",
        subject_pid: POOL,
        subject_name: "Platform engineers",
        first_true_on: "2026-10-01",
        cleared_on: null,
        acknowledged_on: null,
        days_open: 12,
        days_to_acknowledge: null,
      },
      {
        pid: "w2",
        kind: "cutover_at_risk",
        subject_pid: CUTOVER,
        subject_name: "Finance system go-live",
        first_true_on: "2026-09-20",
        cleared_on: "2026-09-25",
        acknowledged_on: "2026-09-22",
        days_open: 5,
        days_to_acknowledge: 2,
      },
    ],
    unacknowledged: 1,
    time_to_report_days: null,
    basis:
      "The date a condition was first true is the date of the first sweep that saw it.",
  };

  test("lists the warnings, says the time to report is not known while one is unacknowledged, acknowledges and sweeps", async ({
    page,
  }) => {
    await signIn(page);
    await page.route("**/api/proxy/early-warnings?**", (route) =>
      route.fulfill({ json: WARNINGS }),
    );
    const acked = await record(
      page,
      "**/api/proxy/early-warnings/w1/acknowledge",
    );
    const swept = await record(page, "**/api/proxy/early-warnings/sweep?**", {
      json: { opened: 1, cleared: 0, told: 2 },
    });
    await page.goto("/warnings");
    await expect(page.getByTestId("warning")).toHaveCount(2);
    await expect(page.getByTestId("time-to-report")).toContainText(
      "not known while any warning is unacknowledged",
    );
    await expect(page.getByTestId("time-to-report")).not.toContainText(
      "0 calendar days",
    );
    await expect(page.getByTestId("warning").first()).toContainText(
      "pool over-committed",
    );
    await expect(page.getByTestId("warning").first()).toContainText(
      "Not acknowledged",
    );
    await expect(page.getByTestId("warning").nth(1)).toContainText(
      "Cleared 25 Sept 2026",
    );
    // Only the unacknowledged one can be acknowledged.
    await expect(page.getByTestId("acknowledge")).toHaveCount(1);
    await page.getByTestId("acknowledge").click();
    await expect.poll(() => acked.length).toBe(1);
    await page.getByTestId("sweep").click();
    await expect(page.getByTestId("swept")).toHaveText(
      "Opened 1, cleared 0, told 2",
    );
    expect(swept).toHaveLength(1);
  });
});

test.describe("resilience for one organization", () => {
  test("asks for the chosen organization", async ({ page }) => {
    await signIn(page);
    const asked: string[] = [];
    await page.route(
      "**/api/proxy/workforce-intelligence/key-person**",
      (route) => {
        asked.push(
          new URL(route.request().url()).searchParams.get("organization") ?? "",
        );
        return route.fulfill({ json: KEY_PERSON });
      },
    );
    await page.route("**/api/proxy/governance-commitments/load?**", (route) =>
      route.fulfill({ json: GOVERNANCE }),
    );
    await page.goto("/resilience");
    await expect(page.getByTestId("spof-count")).toBeVisible();
    expect(asked).toEqual([ORG]);
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

    const pages: Array<[string, string, (p: Page) => Promise<void>, string]> = [
      ["capacity", "/capacity", stubCapacity, "heatmap"],
      ["resilience", "/resilience", stubResilience, "key-person-table"],
      [
        "oversight",
        "/oversight",
        (p) =>
          stubOversight(p, [
            {
              pid: PACK,
              as_of: "2026-10-12",
              month: "2026-09-01",
              withdrawn: { on: "2026-10-13", reason: "wrong month" },
            },
          ]),
        "packs",
      ],
      ["cutover", "/cutover", stubCutover, "overall"],
      [
        "warnings",
        "/warnings",
        async (p) =>
          p.route("**/api/proxy/early-warnings?**", (route) =>
            route.fulfill({
              json: {
                organization_ref: ORG,
                warnings: [
                  {
                    pid: "w1",
                    kind: "pool_over_committed",
                    subject_pid: POOL,
                    subject_name: "Platform engineers",
                    first_true_on: "2026-10-01",
                    cleared_on: null,
                    acknowledged_on: null,
                    days_open: 12,
                    days_to_acknowledge: null,
                  },
                ],
                unacknowledged: 1,
                time_to_report_days: null,
                basis: "",
              },
            }),
          ),
        "warning",
      ],
    ];
    for (const [name, path, stub, ready] of pages) {
      test(`${name} fits without the page scrolling sideways`, async ({
        page,
      }) => {
        // The app's dark mode is a Lily theme the picker loads from this saved choice, not the
        // browser's colour scheme alone; without it a "dark" screenshot would be light.
        await page.addInitScript((theme) => {
          try {
            localStorage.setItem("mxi.wpm.theme", theme);
          } catch {
            /* a blocked store only leaves the default theme */
          }
        }, scheme);
        await signIn(page);
        await stub(page);
        await page.goto(path);
        await expect(page.getByTestId(ready).first()).toBeVisible();
        const overflow = await page.evaluate(
          () =>
            document.documentElement.scrollWidth -
            document.documentElement.clientWidth,
        );
        expect(overflow).toBeLessThanOrEqual(0);
        await page.screenshot({
          path: `test-results/${name}-${scheme}-${label}.png`,
          fullPage: true,
        });
      });
    }
  });
}
