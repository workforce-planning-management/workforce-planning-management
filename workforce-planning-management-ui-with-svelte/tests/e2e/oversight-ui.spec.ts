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
    },
    hypercare_cover: {
      status: cover,
      reasons:
        cover === "ready"
          ? []
          : ["no on-call rota is linked for the cutover window"],
    },
    key_person_risk: {
      status: risk,
      reasons:
        risk === "ready"
          ? []
          : ["Database administration: one holder at the site"],
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
  await page.route("**/api/proxy/workforce-intelligence/key-person", (route) =>
    route.fulfill({ json: KEY_PERSON }),
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
    await expect(detail).toContainText("Withdrawn 2026-10-12: wrong month");
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
      "no on-call rota",
    );
    await expect(page.getByTestId("check-key_person_risk")).toContainText(
      "one holder at the site",
    );
    await expect(page.getByTestId("check-trained_staff")).toContainText(
      "No concerns found.",
    );
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
