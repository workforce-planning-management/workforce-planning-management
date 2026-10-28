// The signed-in home page: the headline "My Home Page", six tiles (an unknown one shows a dash, never 0),
// and a header that says "Sign out" when signed in and "Sign in" when not.

import { expect, test, type Page } from "@playwright/test";

const ORG_A = "organization:11111111-1111-4111-8111-111111111111";
const ORG_B = "organization:22222222-2222-4222-8222-222222222222";

const membership = (org: string, n: number) => ({
  pid: `0000000${n}-0000-4000-8000-000000000000`,
  person_ref: "person:33333333-3333-4333-8333-333333333333",
  organization_ref: org,
  worker_pid: null,
  employed: false,
  role: "hr_admin",
  starts_on: "2026-01-01",
  ends_on: null,
});

async function signIn(page: Page, value = "home-spec-session") {
  await page.context().addCookies([
    {
      name: "__Host-mxi_session",
      value,
      domain: "localhost",
      path: "/",
      httpOnly: true,
      secure: true,
      sameSite: "Lax",
    },
  ]);
}

async function stubHome(page: Page, over: { warnings?: "fail" } = {}) {
  await page.route("**/api/proxy/**", (route) =>
    route.fulfill({ status: 404, body: "unstubbed: " + route.request().url() }),
  );
  await page.route("**/api/proxy/me/organizations", (route) =>
    route.fulfill({ json: [membership(ORG_A, 1), membership(ORG_B, 2)] }),
  );
  await page.route("**/api/proxy/me/organizations/scope", (route) =>
    route.fulfill({ json: [ORG_A, ORG_B] }),
  );
  await page.route("**/api/proxy/workers**", (route) =>
    route.fulfill({ json: [{ pid: "w1" }, { pid: "w2" }, { pid: "w3" }] }),
  );
  await page.route("**/api/proxy/requisitions**", (route) =>
    route.fulfill({ json: [{ pid: "r1" }] }),
  );
  await page.route("**/api/proxy/succession-plans/gaps", (route) =>
    route.fulfill({ json: { gaps: [{}, {}] } }),
  );
  await page.route("**/api/proxy/announcements**", (route) =>
    route.fulfill({ json: [] }),
  );
  await page.route(
    "**/api/proxy/workforce-intelligence/key-person**",
    (route) =>
      route.fulfill({
        json: { single_points_of_failure: 4, skills: [], basis: "" },
      }),
  );
  await page.route("**/api/proxy/early-warnings?**", (route) =>
    over.warnings === "fail"
      ? route.fulfill({ status: 500, json: { description: "boom" } })
      : route.fulfill({
          json: {
            organization_ref: "",
            warnings: [],
            unacknowledged: 1,
            time_to_report_days: null,
            basis: "",
          },
        }),
  );
  await page.route("**/api/proxy/capacity?**", (route) => {
    const org = new URL(route.request().url()).searchParams.get("organization");
    // One pool over in the first organization, none in the second.
    const over = org === ORG_A ? 2 : 0;
    return route.fulfill({
      json: {
        organization_ref: org,
        from: null,
        months: [],
        constraint_pool: null,
        wip: null,
        derivation: "",
        pools: [
          {
            pid: "p1",
            name: "Pool",
            operations_reservation_bp: 0,
            members: 1,
            over_months: over,
            unknown_months: 0,
            over_total_centi: 0,
            cells: [],
          },
        ],
      },
    });
  });
}

test("signed in: the headline, six tiles with their figures, and Sign out (not Sign in) in the header", async ({
  page,
}) => {
  await signIn(page);
  await stubHome(page);
  await page.goto("/");
  await expect(page.locator("h1")).toHaveText("My Home Page");
  const tiles = page.locator(".tiles .tile");
  await expect(tiles).toHaveCount(6);
  await expect(page.getByTestId("tile-active")).toContainText("3");
  await expect(page.getByTestId("tile-open")).toContainText("1");
  await expect(page.getByTestId("tile-gaps")).toContainText("2");
  await expect(page.getByTestId("tile-single-points")).toContainText("4");
  // One unacknowledged warning in each of the two organizations.
  await expect(page.getByTestId("tile-warnings")).toContainText("2");
  // A pool over capacity in the first organization only.
  await expect(page.getByTestId("tile-pools-over")).toContainText("1");
  await expect(page.getByTestId("tile-single-points")).toHaveAttribute(
    "href",
    /\/resilience$/,
  );
  await expect(page.getByTestId("tile-warnings")).toHaveAttribute(
    "href",
    /\/warnings$/,
  );
  await expect(page.getByTestId("tile-pools-over")).toHaveAttribute(
    "href",
    /\/capacity$/,
  );
  // The header offers Sign out, and no Sign in.
  await expect(page.getByTestId("signout")).toHaveText("Sign out");
  await expect(page.getByTestId("signin-link")).toHaveCount(0);
  await expect(
    page.locator("nav.top").getByText("Sign in", { exact: true }),
  ).toHaveCount(0);
});

test("a tile that cannot be read shows a dash, never 0", async ({ page }) => {
  await signIn(page);
  await stubHome(page, { warnings: "fail" });
  await page.goto("/");
  await expect(page.getByTestId("tile-single-points")).toContainText("4");
  await expect(page.getByTestId("tile-warnings")).toContainText("—");
  await expect(page.getByTestId("tile-warnings")).not.toContainText("0");
});

test("signed out: the header offers Sign in, and no Sign out", async ({
  page,
}) => {
  await page.goto("/");
  await expect(page.getByTestId("splash")).toBeVisible();
  await expect(page.getByTestId("signin-link")).toHaveText("Sign in");
  await expect(page.getByTestId("signout")).toHaveCount(0);
});

test("Sign out ends the session: back to the splash with Sign in, and the pages ask for a sign-in again", async ({
  page,
}) => {
  await signIn(page, "demo:home-spec");
  await stubHome(page);
  await page.goto("/");
  await page.getByTestId("signout").click();
  await expect(page.getByTestId("splash")).toBeVisible();
  await expect(page.getByTestId("signin-link")).toBeVisible();
  await expect(page.getByTestId("signout")).toHaveCount(0);
  expect(
    (await page.context().cookies()).some(
      (c) => c.name === "__Host-mxi_session",
    ),
  ).toBe(false);
  await page.goto("/workers");
  await expect(page).toHaveURL(/\/signin$/);
});

test("the headline and the button are in the reader's language", async ({
  page,
}) => {
  await signIn(page);
  await stubHome(page);
  await page.goto("/cy-001");
  await expect(page.locator("h1")).toHaveText("Fy Nhudalen Hafan");
  await expect(page.getByTestId("signout")).toHaveText("Allgofnodi");
});
