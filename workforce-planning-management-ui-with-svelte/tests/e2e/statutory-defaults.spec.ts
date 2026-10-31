// The statutory defaults page (`/statutory-defaults`, WPM-T403): a read-only view of whose each default is. The API is stubbed per
// test; an unstubbed /api/proxy call 404s loudly.

import { expect, test } from "@playwright/test";
import { stubSignedIn } from "./support";

const origin = (regime: string | null, status: string, reference: string) => ({
  regime,
  status,
  reference,
});

const SETTINGS = [
  {
    setting: "WPM_AI_ACT_APPLIES_FROM",
    area: "ai_register",
    value: "2027-12-02",
    set: false,
    default: "2027-12-02",
    default_from: origin(
      "EU",
      "unverified",
      "European Union: AI Act, Article 113 as amended",
    ),
  },
  {
    setting: "WPM_FLEXIBLE_DECISION_MONTHS",
    area: "flexible_working",
    value: 3,
    set: true,
    default: "2",
    default_from: origin(
      "GB",
      "unverified",
      "Great Britain: Employment Rights Act 1996, Part 8A",
    ),
  },
  {
    setting: "WPM_PAY_GAP_TIERS",
    area: "pay_transparency",
    value: [
      { from_workers: 250, every_years: 1, first_report_on: "2027-06-07" },
    ],
    set: true,
    default: "250:1:2027-06-07,150:3:2027-06-07,100:3:2031-06-07",
    default_from: origin(
      "EU",
      "unverified",
      "European Union: Pay Transparency Directive (EU) 2023/970, Article 9; from search excerpts",
    ),
  },
  {
    setting: "WPM_REQUIRE_PAY_RANGE",
    area: "pay_transparency",
    value: true,
    set: false,
    default: "on",
    default_from: origin(
      "EU",
      "unverified",
      "European Union: Pay Transparency Directive (EU) 2023/970, Article 5; from search excerpts",
    ),
  },
  {
    setting: "WPM_RETENTION_PAY_DAYS",
    area: "retention",
    value: 2190,
    set: false,
    default: "2190",
    default_from: origin(
      null,
      "starting_point",
      "a cautious starting point, no regime's",
    ),
  },
  {
    setting: "WPM_PAYROLL_STUB_CURRENCY",
    area: "payroll",
    value: "GBP",
    set: false,
    default: "GBP",
    default_from: origin(
      "GB",
      "example",
      "an example in the shape of the UK's income tax for 2025-26",
    ),
  },
];

test.describe("Statutory defaults", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
  });

  test("shows each setting's value in force, whether it was set, and whose its default is", async ({
    page,
  }) => {
    let writes = 0;
    await page.route("**/api/proxy/statutory-defaults", (route) => {
      if (route.request().method() !== "GET") writes += 1;
      return route.fulfill({
        json: {
          settings: SETTINGS,
          derivation:
            "Each setting's value in force is read as its feature reads it.",
        },
      });
    });
    await page.goto("/en-001/statutory-defaults");
    await expect(
      page.getByRole("heading", { level: 1, name: "Statutory defaults" }),
    ).toBeVisible();
    await expect(page).toHaveTitle("Statutory defaults — WPM");

    const ai = page.getByTestId("default-WPM_AI_ACT_APPLIES_FROM");
    await expect(ai).toContainText("AI register");
    await expect(ai).toContainText("2 Dec 2027");
    await expect(ai).toContainText("EU");
    await expect(ai).toContainText("Not checked against the text");
    await expect(ai).toContainText("Not set: the default");

    const months = page.getByTestId("default-WPM_FLEXIBLE_DECISION_MONTHS");
    await expect(months).toContainText("Set by this deployment");
    await expect(months).toContainText("GB");

    const tiers = page.getByTestId("default-WPM_PAY_GAP_TIERS");
    await expect(tiers).toContainText(
      "workers, at least: 250; years between reports: 1; first report: 7 Jun 2027",
    );
    // The Directive was read only as search excerpts, so its defaults are not checked (WPM-T404).
    await expect(tiers).toContainText("Not checked against the text");
    await expect(
      page.getByTestId("default-WPM_REQUIRE_PAY_RANGE"),
    ).toContainText("On");

    // A starting point names no regime, and a number is in the page's format.
    const retention = page.getByTestId("default-WPM_RETENTION_PAY_DAYS");
    await expect(retention).toContainText("No regime's");
    await expect(retention).toContainText("A starting point");
    await expect(retention).toContainText("2,190");
    await expect(
      page.getByTestId("default-WPM_PAYROLL_STUB_CURRENCY"),
    ).toContainText("An example in a regime's shape");

    // Read only: no control writes a setting.
    await expect(page.getByRole("main").getByRole("textbox")).toHaveCount(0);
    expect(writes).toBe(0);
    await page.screenshot({
      path: "test-results/statutory-defaults.png",
      fullPage: true,
    });
  });

  test("is in the page's language, with the service's own text kept as sent", async ({
    page,
  }) => {
    await page.route("**/api/proxy/statutory-defaults", (route) =>
      route.fulfill({ json: { settings: SETTINGS, derivation: "" } }),
    );
    await page.goto("/de-001/statutory-defaults");
    await expect(
      page.getByRole("heading", {
        level: 1,
        name: "Gesetzliche Voreinstellungen",
      }),
    ).toBeVisible();
    const retention = page.getByTestId("default-WPM_RETENTION_PAY_DAYS");
    await expect(retention).toContainText("Keiner Rechtsordnung");
    await expect(retention).toContainText("2.190");
    await expect(retention).toContainText(
      "a cautious starting point, no regime's",
    );
  });

  test("for one organization, each value says whether it came from the profile, the setting or the default (WPM-D153)", async ({
    page,
  }) => {
    const org = "organization:44444444-4444-4444-8444-444444444444";
    const profile = (code: string) => ({
      kind: "profile",
      status: "example",
      from: [
        {
          code,
          layer: "national",
          status: "example",
          source: "synthetic",
          effective_from: "2000-01-01",
        },
      ],
    });
    const asked: string[] = [];
    await page.route(/\/api\/proxy\/statutory-defaults(\?.*)?$/, (route) => {
      const url = new URL(route.request().url());
      asked.push(url.search);
      const forOrg = url.searchParams.get("organization") === org;
      return route.fulfill({
        json: {
          organization_ref: forOrg ? org : null,
          home_jurisdiction: forOrg ? "XB" : null,
          derivation: "",
          settings: [
            {
              ...SETTINGS[2],
              key: "pay_transparency.reporting_tiers",
              reader: "organization",
              came_from: forOrg
                ? profile("XB")
                : { kind: "default", setting: "WPM_PAY_GAP_TIERS" },
            },
            {
              ...SETTINGS[1],
              key: "flexible_working.decision_months",
              reader: "worker",
              came_from: {
                kind: "setting",
                setting: "WPM_FLEXIBLE_DECISION_MONTHS",
              },
              ...(forOrg
                ? {
                    by_place: [
                      { code: "XA", value: 1, came_from: profile("XA") },
                    ],
                  }
                : {}),
            },
            {
              ...SETTINGS[5],
              key: null,
              reader: null,
              came_from: { kind: "default" },
            },
          ],
        },
      });
    });
    await page.goto("/en-001/statutory-defaults");
    const tiers = page.getByTestId("default-WPM_PAY_GAP_TIERS");
    await expect(tiers).toContainText("The default");
    await expect(tiers).toContainText("The organization's home jurisdiction");
    await expect(page.getByTestId("defaults-home")).toHaveCount(0);

    await page.getByTestId("defaults-organization").selectOption(org);
    await expect(page.getByTestId("defaults-home")).toContainText(
      "Home jurisdiction: XB",
    );
    await expect(tiers).toContainText(
      "Profile: XB (An example in a regime's shape)",
    );
    const months = page.getByTestId("default-WPM_FLEXIBLE_DECISION_MONTHS");
    await expect(months).toContainText("This deployment's setting");
    await expect(months).toContainText("Each worker's place of work");
    await expect(months).toContainText(
      "XA: 1 (Profile: XA (An example in a regime's shape))",
    );
    await expect(
      page.getByTestId("default-WPM_PAYROLL_STUB_CURRENCY"),
    ).toContainText("The whole deployment");
    expect(asked).toEqual(["", `?organization=${encodeURIComponent(org)}`]);
    await page.screenshot({
      path: "test-results/statutory-defaults-organization.png",
      fullPage: true,
    });
  });

  test("a failed read says so in the page's words and shows no table", async ({
    page,
  }) => {
    await page.route("**/api/proxy/statutory-defaults", (route) =>
      route.fulfill({
        status: 500,
        body: "internal: table statutory_defaults",
      }),
    );
    await page.goto("/en-001/statutory-defaults");
    await expect(page.getByTestId("error")).toBeVisible();
    await expect(page.getByTestId("error")).not.toContainText(
      "statutory_defaults",
    );
    await expect(page.getByTestId("defaults-table")).toHaveCount(0);
  });
});
