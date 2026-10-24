// Shared by the Playwright specs: the fake session, two workers and the stubs every signed-in page needs.
// The API is stubbed with page.route; any unstubbed /api/proxy call 404s loudly, so contract drift fails the suite.

import { expect, type Page } from "@playwright/test";

/**
 * Every page but /signin and /verify is gated on a session (WPM-T38),
 * so the smoke suite injects a fake `__Host-mxi_session` cookie
 * directly into the browser context before each test — the server
 * only checks the cookie's *presence*, never its validity
 * (`locals.sessionId` is set straight from `cookies.get`), so a
 * fabricated value is enough to pass the gate without a real
 * authentication-service round trip.
 */
export async function signIn(page: Page) {
  await page.context().addCookies([
    {
      name: "__Host-mxi_session",
      value: "smoke-test-session",
      domain: "localhost",
      path: "/",
      httpOnly: true,
      secure: true,
      sameSite: "Lax",
    },
  ]);
}

export const WORKER = {
  pid: "11111111-1111-4111-8111-111111111111",
  person_ref: "person:22222222-2222-4222-8222-222222222222",
  upstream_worker_ref: null,
  organization_ref: "organization:33333333-3333-4333-8333-333333333333",
  worker_number: "E-0001",
  display_name: "Test Worker 001",
  status: "active",
  employment_type: "permanent",
  fte_percent: 100,
  department: "engineering",
  job_title: "Engineer",
  manager_pid: null,
  salary_minor: 3600000,
  salary_currency: "GBP",
  hired_on: "2026-01-05",
  terminated_on: null,
};

export const MASKED_WORKER = {
  ...WORKER,
  pid: "44444444-4444-4444-8444-444444444444",
  worker_number: "E-0002",
  display_name: "Masked Person",
  salary_minor: null,
  salary_currency: null,
};

/**
 * Choose a locale from the Lily locale picker.
 *
 * Three things make this more than a `selectOption`:
 *
 * 1. The picker renders a button plus a `ul` listbox, not a `<select>`.
 * 2. The theme picker on the same page also renders
 *    `li[role="option"]`, so the list has to be scoped — unscoped, the
 *    selector matches 58 elements here.
 * 3. The listbox is **still expanded after a pointer selection**
 *    (verified in a browser against Lily as of 2026-07-31), so
 *    clicking the button unconditionally would close it rather than
 *    open it. Open only when collapsed, which is correct either way.
 */
export async function chooseLocale(page: Page, label: string) {
  const button = page.locator("nav.top .locale-picker-button");
  const list = page.locator("ul.locale-picker-list");
  if ((await button.getAttribute("aria-expanded")) !== "true") {
    await button.click();
  }
  await expect(list).toBeVisible();
  await list
    .locator('li[role="option"]')
    .filter({ hasText: label })
    .first()
    .click();
}

/** Sign in, fail every unstubbed call loudly, and stub the people and memberships every page reads. */
export async function stubSignedIn(page: Page) {
  await signIn(page);
  // Unstubbed API calls fail loudly.
  await page.route("**/api/proxy/**", (route) =>
    route.fulfill({
      status: 404,
      body: "unstubbed: " + route.request().url(),
    }),
  );
  await page.route("**/api/proxy/workers", (route) =>
    route.fulfill({ json: [WORKER, MASKED_WORKER] }),
  );
  await page.route("**/api/proxy/workers?status=active", (route) =>
    route.fulfill({ json: [WORKER, MASKED_WORKER] }),
  );
  await page.route("**/api/proxy/requisitions?status=open", (route) =>
    route.fulfill({
      json: [
        {
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
        },
      ],
    }),
  );
  await page.route("**/api/proxy/succession-plans/gaps", (route) =>
    route.fulfill({ json: { gaps: [] } }),
  );
  await page.route("**/api/proxy/me/organizations", (route) =>
    route.fulfill({
      json: [
        {
          pid: "88888888-8888-4888-8888-888888888888",
          person_ref: WORKER.person_ref,
          organization_ref: WORKER.organization_ref,
          worker_pid: WORKER.pid,
          employed: true,
          role: "member",
          starts_on: "2026-01-05",
          ends_on: null,
        },
        {
          pid: "99999999-9999-4999-9999-999999999999",
          person_ref: WORKER.person_ref,
          organization_ref:
            "organization:44444444-4444-4444-8444-444444444444",
          worker_pid: null,
          employed: false,
          role: "hr_admin",
          starts_on: "2026-02-01",
          ends_on: null,
        },
      ],
    }),
  );
  // Read scope, expanded through org confederation server-side — in
  // this fixture it equals the two direct memberships above (no
  // confederation edges), so `/org-chart` and `/benchmarks` render
  // the same two sections the dashboard's membership list does.
  await page.route("**/api/proxy/me/organizations/scope", (route) =>
    route.fulfill({
      json: [
        WORKER.organization_ref,
        "organization:44444444-4444-4444-8444-444444444444",
      ],
    }),
  );
}
