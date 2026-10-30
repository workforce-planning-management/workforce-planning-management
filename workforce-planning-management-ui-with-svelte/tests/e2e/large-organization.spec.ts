// A very large organization (WPM-R142): the org chart draws part of it and says so, and the mentorship overview
// names only the newest unmatched people but counts all of them. The API is stubbed per test.

import { expect, test, type Page } from "@playwright/test";

async function signIn(page: Page) {
  await page.context().addCookies([
    {
      name: "__Host-mxi_session",
      value: "large-org-spec-session",
      domain: "localhost",
      path: "/",
      httpOnly: true,
      secure: true,
      sameSite: "Lax",
    },
  ]);
}

const ORG = "organization:00000000-0000-4000-8000-0000000000aa";

const node = (pid: string, reports: unknown[] = []) => ({
  pid,
  display_name: `Person ${pid}`,
  job_title: "Engineer",
  department: "engineering",
  tenure: "under_1y",
  location: null,
  reports,
});

test("the org chart says when it draws only part of the organization", async ({
  page,
}) => {
  await signIn(page);
  await page.route("**/api/proxy/**", (route) =>
    route.fulfill({ status: 404, body: "unstubbed" }),
  );
  await page.route("**/api/proxy/me/organizations/scope", (route) =>
    route.fulfill({ json: [ORG] }),
  );
  await page.route("**/api/proxy/me/organizations", (route) =>
    route.fulfill({ json: [] }),
  );
  await page.route("**/api/proxy/org-chart?**", (route) =>
    route.fulfill({ json: [node("a", [node("b")]), node("c")] }),
  );
  await page.route("**/api/proxy/org-chart/size?**", (route) =>
    route.fulfill({ json: { people: 200000 } }),
  );
  await page.goto("/en-001/org-chart");
  await expect(page.getByTestId("org-chart-truncated")).toHaveText(
    "Showing 3 of 200,000 people; the rest are not drawn.",
  );
});

test("the org chart says nothing when it draws everyone", async ({ page }) => {
  await signIn(page);
  await page.route("**/api/proxy/**", (route) =>
    route.fulfill({ status: 404, body: "unstubbed" }),
  );
  await page.route("**/api/proxy/me/organizations/scope", (route) =>
    route.fulfill({ json: [ORG] }),
  );
  await page.route("**/api/proxy/me/organizations", (route) =>
    route.fulfill({ json: [] }),
  );
  await page.route("**/api/proxy/org-chart?**", (route) =>
    route.fulfill({ json: [node("a", [node("b")])] }),
  );
  await page.route("**/api/proxy/org-chart/size?**", (route) =>
    route.fulfill({ json: { people: 2 } }),
  );
  await page.goto("/en-001/org-chart");
  await expect(page.getByTestId("org-chart")).toBeVisible();
  await expect(page.getByTestId("org-chart-truncated")).toHaveCount(0);
});

test("the mentorship overview counts everyone unmatched and names only some", async ({
  page,
}) => {
  await signIn(page);
  await page.route("**/api/proxy/**", (route) =>
    route.fulfill({ status: 404, body: "unstubbed" }),
  );
  await page.route("**/api/proxy/learning/mentorship-overview**", (route) =>
    route.fulfill({
      json: {
        as_of: "2026-10-21",
        active_pairings: 0,
        mentor_load: [],
        unmatched_total: 199000,
        unmatched_workers: [
          { pid: "w1", display_name: "Newest Hire", department: "engineering" },
        ],
        stale_days: 30,
        stale_mentorships: [],
      },
    }),
  );
  await page.goto("/en-001/mentorship");
  await expect(page.getByTestId("mentorship-tiles")).toContainText("199,000");
  await expect(page.getByTestId("unmatched-shown")).toHaveText(
    "Showing the newest 1 of 199,000.",
  );
  await expect(page.getByTestId("unmatched")).toContainText("Newest Hire");
});
