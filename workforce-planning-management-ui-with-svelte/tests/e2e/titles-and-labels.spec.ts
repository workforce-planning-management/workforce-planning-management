// Titles and labels (WPM-T262): every page names itself in the tab, in the locale the address carries, and the
// forms on the pages that had unlabelled controls pass axe's naming rules. The API is stubbed per test; an
// unstubbed /api/proxy call 404s loudly.

import AxeBuilder from "@axe-core/playwright";
import { expect, test, type Page } from "@playwright/test";

async function signIn(page: Page) {
  await page.context().addCookies([
    {
      name: "__Host-mxi_session",
      value: "titles-spec-session",
      domain: "localhost",
      path: "/",
      httpOnly: true,
      secure: true,
      sameSite: "Lax",
    },
  ]);
}

const PID = "11111111-1111-4111-8111-111111111111";
const WORKER = {
  pid: PID,
  person_ref: "person:22222222-2222-4222-8222-222222222222",
  upstream_worker_ref: null,
  organization_ref: "organization:33333333-3333-4333-8333-333333333333",
  worker_number: "E-0001",
  display_name: "Test Worker 001",
  status: "active",
  employment_type: "employee",
  fte_percent: 100,
  department: "engineering",
  job_title: "Engineer",
  location: null,
  manager_pid: null,
  salary_minor: null,
  salary_currency: null,
  hired_on: "2026-01-05",
  terminated_on: null,
  engagement_ends_on: null,
};
const REQUISITION = {
  pid: "55555555-5555-4555-8555-555555555555",
  organization_ref: WORKER.organization_ref,
  department: "engineering",
  job_title: "Platform Engineer",
  headcount: 1,
  salary_min_minor: null,
  salary_max_minor: null,
  salary_currency: null,
  status: "open",
  opened_on: "2026-06-01",
};

/**
 * The controls on the page that have no name of their own: no label around or for them, no `aria-label`,
 * `aria-labelledby` or `title`. A placeholder is not a name (axe accepts one; screen readers drop it once typed in).
 */
async function unnamedControls(page: Page): Promise<string[]> {
  return page.evaluate(() =>
    [...document.querySelectorAll<HTMLElement>("input, select, textarea")]
      .filter((el) => {
        if (
          el instanceof HTMLInputElement &&
          ["hidden", "submit", "button"].includes(el.type)
        )
          return false;
        if (el.closest("label")) return false;
        if (el.id && document.querySelector(`label[for="${el.id}"]`))
          return false;
        return !(
          el.getAttribute("aria-label") ||
          el.getAttribute("aria-labelledby") ||
          el.getAttribute("title")
        );
      })
      .map((el) => el.outerHTML.slice(0, 120)),
  );
}

test.beforeEach(async ({ page }) => {
  await signIn(page);
  // Everything is an empty list unless a test says otherwise; the titles come from the route, not the data.
  await page.route("**/api/proxy/**", (route) => route.fulfill({ json: [] }));
});

const TITLED: Array<[string, string]> = [
  ["/benchmarks", "Benchmarks"],
  ["/development", "Development"],
  ["/org-chart", "Org chart"],
  ["/payroll", "Payroll"],
  [`/payroll/${PID}`, "Payroll"],
  ["/privacy", "Privacy"],
  ["/requisitions", "Vacancies"],
  [`/requisitions/${PID}`, "Vacancies"],
  ["/wellbeing", "Wellbeing"],
  ["/workers", "Workers"],
  [`/workers/${PID}`, "Workers"],
  ["/workforce", "Workforce"],
];

test.describe("every page names itself (WPM-T262)", () => {
  for (const [path, name] of TITLED) {
    test(`${path} is titled "${name} — WPM"`, async ({ page }) => {
      await page.goto(path);
      await expect(page).toHaveTitle(`${name} — WPM`);
    });
  }

  test("the title follows the locale in the address", async ({ page }) => {
    await page.goto("/de-001/workers");
    await expect(page).toHaveTitle("Arbeitskräfte — WPM");
  });
});

test.describe("controls have names (WPM-T262)", () => {
  const rules = ["label", "select-name", "document-title"];

  test("the wellbeing form", async ({ page }) => {
    await page.route("**/api/proxy/wellbeing/uptake", (route) =>
      route.fulfill({
        json: { as_of: "2026-10-01", derivation: "x", entitlements: [] },
      }),
    );
    await page.goto("/wellbeing");
    await expect(page.getByTestId("wellbeing-form")).toBeVisible();
    const result = await new AxeBuilder({ page }).withRules(rules).analyze();
    expect(
      result.violations.map(
        (v) => `${v.id}: ${v.nodes.map((n) => n.html).join(" | ")}`,
      ),
    ).toEqual([]);
    expect(await unnamedControls(page)).toEqual([]);
  });

  test("the hire-number input on an offer", async ({ page }) => {
    await page.route(`**/api/proxy/requisitions/${REQUISITION.pid}`, (route) =>
      route.fulfill({ json: REQUISITION }),
    );
    await page.route(
      `**/api/proxy/requisitions/${REQUISITION.pid}/applications`,
      (route) =>
        route.fulfill({
          json: [
            {
              pid: "66666666-6666-4666-8666-666666666666",
              requisition_pid: REQUISITION.pid,
              candidate_pid: "77777777-7777-4777-8777-777777777777",
              stage: "offer",
              notes: null,
            },
          ],
        }),
    );
    await page.goto(`/requisitions/${REQUISITION.pid}`);
    await expect(page.getByTestId("applications")).toContainText("offer");
    const result = await new AxeBuilder({ page }).withRules(rules).analyze();
    expect(
      result.violations.map(
        (v) => `${v.id}: ${v.nodes.map((n) => n.html).join(" | ")}`,
      ),
    ).toEqual([]);
    expect(await unnamedControls(page)).toEqual([]);
  });

  test("a worker's adjustment, ergonomic and appraisal forms", async ({
    page,
  }) => {
    // As in the other worker-page specs: what is not stubbed 404s and that panel shows its own state.
    await page.route("**/api/proxy/**", (route) =>
      route.fulfill({ status: 404, body: "unstubbed" }),
    );
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
      await page.route(`**/api/proxy/workers/${PID}/${list}`, (route) =>
        route.fulfill({ json: [] }),
      );
    }
    await page.route(`**/api/proxy/workers/${PID}`, (route) =>
      route.fulfill({ json: WORKER }),
    );
    await page.route(`**/api/proxy/workers/${PID}/wellbeing-prompts`, (route) =>
      route.fulfill({ json: { prompts: [] } }),
    );
    await page.goto(`/workers/${PID}`);
    await expect(page.getByTestId("adjustments")).toBeVisible();
    const result = await new AxeBuilder({ page }).withRules(rules).analyze();
    expect(
      result.violations.map(
        (v) => `${v.id}: ${v.nodes.map((n) => n.html).join(" | ")}`,
      ),
    ).toEqual([]);
    expect(await unnamedControls(page)).toEqual([]);
  });
});
