// CPD entries from the learning system link to their course there (WPM-R138). The API is stubbed per
// test; an unstubbed /api/proxy call 404s loudly. The page only shows what the API gave it: the link is
// built by the service from the deployer's template, so here it is just data.

import { expect, test, type Page } from "@playwright/test";

const WORKER = "11111111-1111-4111-8111-111111111111";

async function signIn(page: Page) {
  await page.context().addCookies([
    {
      name: "__Host-mxi_session",
      value: "cpd-lms-spec-session",
      domain: "localhost",
      path: "/",
      httpOnly: true,
      secure: true,
      sameSite: "Lax",
    },
  ]);
}

const entry = (pid: string, activity: string, source: string, lms_url: string | null) => ({
  pid,
  entry_date: "2026-09-01",
  activity,
  category: "course",
  unit: "hours",
  amount: 2,
  amount_hundredths: 200,
  evidence_note: null,
  evidence_url: null,
  source,
  lms_url,
  verified_on: null,
  verified_by: null,
});

async function stub(page: Page, entries: unknown[]) {
  await page.route("**/api/proxy/**", (route) =>
    route.fulfill({ status: 404, body: "unstubbed: " + route.request().url() }),
  );
  await page.route("**/api/proxy/workers", (route) =>
    route.fulfill({ json: [{ pid: WORKER, display_name: "Test Worker 001" }] }),
  );
  await page.route("**/api/proxy/cpd/overview", (route) =>
    route.fulfill({
      json: { derivation: "", headcount: 1, requirements: [], registrations: { expiring: 0, expired: 0 } },
    }),
  );
  await page.route("**/api/proxy/cpd-requirements", (route) => route.fulfill({ json: [] }));
  await page.route(`**/api/proxy/workers/${WORKER}/cpd-progress`, (route) =>
    route.fulfill({ json: { derivation: "", worker_pid: WORKER, requirements: [], registrations: [] } }),
  );
  await page.route(`**/api/proxy/workers/${WORKER}/cpd-entries`, (route) => route.fulfill({ json: entries }));
  await page.route("**/api/proxy/calendar**", (route) => route.fulfill({ json: { events: [] } }));
}

test.describe("CPD entries and the learning system", () => {
  test("a completion from the learning system opens its course, in a new tab, without the opener", async ({ page }) => {
    await signIn(page);
    await stub(page, [
      entry("a1", "LMS completion: course:33333333-3333-4333-8333-333333333333", "lms", "https://lms.example.org/course/33333333-3333-4333-8333-333333333333"),
      entry("a2", "LMS completion: course:44444444-4444-4444-8444-444444444444", "lms", null),
      entry("a3", "Read a standard", "manual", null),
    ]);
    await page.goto("/en-001/cpd");
    await page.getByTestId("cpd-worker").selectOption(WORKER);

    const link = page.getByRole("link", { name: /LMS/ });
    await expect(link).toHaveCount(1);
    await expect(link).toHaveAttribute("href", "https://lms.example.org/course/33333333-3333-4333-8333-333333333333");
    await expect(link).toHaveAttribute("target", "_blank");
    await expect(link).toHaveAttribute("rel", "noopener noreferrer");
    await expect(link).toHaveAttribute("title", "Open in the learning system");
    // without a template the entry still says where it came from, as plain text
    await expect(page.getByText("LMS", { exact: true })).toHaveCount(1);
  });
});
