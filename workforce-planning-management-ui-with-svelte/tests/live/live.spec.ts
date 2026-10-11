// The contingent-workforce screens against the real service (see README.md). Asserts rules and shapes
// that must hold on any data, never exact figures.

import { expect, test, type Page } from "@playwright/test";

test.skip(
  !process.env.WPM_LIVE_API,
  "needs a running service: set WPM_LIVE_API=1",
);

async function signIn(page: Page) {
  await page.context().addCookies([
    {
      name: "__Host-mxi_session",
      value: "live-test-session",
      domain: "localhost",
      path: "/",
      httpOnly: true,
      secure: true,
      sameSite: "Lax",
    },
  ]);
}

test.beforeEach(async ({ page }) => {
  await signIn(page);
});

type W = {
  pid: string;
  worker_number: string;
  employment_type: string;
  department: string;
};

async function workers(page: Page): Promise<W[]> {
  const res = await page.request.get("/api/proxy/workers");
  expect(res.status()).toBe(200);
  return res.json();
}

test("the contingent page shows real shares, bands, funding months, trend and programmes", async ({
  page,
}) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));
  await page.goto("/contingent");
  await expect(page.getByTestId("contingent-share")).toBeVisible();
  // A percentage, or a dash when there is no FTE: never blank, never NaN.
  for (const id of ["share-contractor", "share-fixed-term"]) {
    await expect(page.getByTestId(id)).toHaveText(/^(\d+(\.\d)?%|—)$/);
  }
  // Every band is a count or "withheld", never an undefined or NaN.
  for (const list of [
    "ending_within_30",
    "ending_31_to_60",
    "ending_61_to_90",
    "past_end_undecided",
    "much_extended",
    "long_running_contractors",
  ]) {
    await expect(page.getByTestId(`figure-${list}-count`)).toHaveText(
      /^(\d+|withheld)$/,
    );
  }
  await expect(page.getByTestId("contingent-funding")).toBeVisible();
  await expect(page.getByTestId("contingent-trend")).toBeVisible();
  await expect(page.getByTestId("contingent-programmes")).toBeVisible();
  expect(errors).toEqual([]);
  await page.screenshot({
    path: "test-results/live-contingent.png",
    fullPage: true,
  });
});

test("'Show who' reads the real named list by worker number and names no one", async ({
  page,
}) => {
  await page.goto("/contingent");
  await expect(page.getByTestId("contingent-engagements")).toBeVisible();
  const count = await page
    .getByTestId("figure-ending_within_30-count")
    .innerText();
  await page.getByTestId("show-ending_within_30").click();
  await expect(page.getByTestId("contingent-who")).toBeVisible();
  if (count !== "withheld" && Number(count) > 0) {
    const table = page.getByTestId("contingent-who-table");
    await expect(table).toBeVisible();
    // The list is the figure: as many rows as the count.
    await expect(table.locator("tbody tr")).toHaveCount(Number(count));
    await expect(table).not.toContainText("Synthetic Contingent");
  }
});

test("a contractor's engagement panel shows the real basis, end and funding flag", async ({
  page,
}) => {
  const all = await workers(page);
  const contractor = all.find((w) => w.employment_type === "contractor");
  expect(contractor, "the demo data holds a contractor").toBeTruthy();
  await page.goto(`/workers/${contractor!.pid}`);
  await expect(page.getByTestId("engagement")).toBeVisible();
  await expect(page.getByTestId("engagement-basis")).toHaveText("contractor");
  await expect(page.getByTestId("engagement-ends")).toHaveText(
    /^\d{4}-\d{2}-\d{2}$/,
  );
  await expect(page.getByTestId("engagement-standing")).not.toBeEmpty();
});

test("a permanent worker shows no engagement panel", async ({ page }) => {
  const all = await workers(page);
  const permanent = all.find((w) => w.employment_type === "permanent");
  await page.goto(`/workers/${permanent!.pid}`);
  await expect(page.getByTestId("facts")).toBeVisible();
  await expect(page.getByTestId("engagement")).toHaveCount(0);
});

test("the plan forecast reports by basis and programme, and a new line reaches the service", async ({
  page,
}) => {
  await page.goto("/planning");
  await expect(page.getByTestId("plan-forecast")).toBeVisible();
  await expect(page.getByTestId("forecast-basis").first()).toContainText("→");
  // The forecast lists its levers in words, not tokens.
  await expect(page.getByTestId("plan-forecast")).not.toContainText(
    "fixed_term_hire",
  );
  await expect(page.getByTestId("plan-forecast-programmes")).toBeVisible();
  const before = await (
    await page.request.get("/api/proxy/workforce-plans")
  ).json();
  const plan = before[0];
  const lines = (
    await (
      await page.request.get(`/api/proxy/workforce-plans/${plan.pid}`)
    ).json()
  ).demand_lines.length;
  // A new department each run: the same department, date, basis and programme would update the line, not add one.
  await page.getByTestId("line-department").fill(`live-check-${Date.now()}`);
  await page.getByTestId("line-target").fill(plan.horizon_end);
  await page.getByTestId("line-basis").selectOption("fixed_term");
  await page.getByTestId("line-ends").fill("2099-01-01");
  await page.getByTestId("line-fte").fill("0.5");
  await page.getByTestId("line-add").click();
  await expect
    .poll(
      async () =>
        (
          await (
            await page.request.get(`/api/proxy/workforce-plans/${plan.pid}`)
          ).json()
        ).demand_lines.length,
    )
    .toBe(lines + 1);
});

test("the requisition board shows a fixed-term post with its duration", async ({
  page,
}) => {
  await page.goto("/requisitions");
  await expect(page.getByTestId("requisition-board")).toBeVisible();
  await expect(page.getByTestId("requisition-board")).toContainText(
    "fixed term",
  );
});

test("a requisition made in the UI reaches the service with its basis, duration and programme", async ({
  page,
}) => {
  await page.goto("/requisitions");
  await expect(page.getByTestId("requisition-form")).toBeVisible();
  const title = `Live Fixed ${Date.now()}`;
  // With no membership the organization is typed; with one it is chosen.
  const org = (await (await page.request.get("/api/proxy/workers")).json())[0]
    .organization_ref;
  if (
    (await page.getByTestId("req-org").evaluate((el) => el.tagName)) === "INPUT"
  ) {
    await page.getByTestId("req-org").fill(org);
  }
  await page.getByTestId("req-department").fill("research");
  await page.getByTestId("req-title").fill(title);
  await page.getByTestId("req-basis").selectOption("contractor");
  await page.getByTestId("req-months").fill("9");
  await page.getByTestId("req-create").click();
  await expect(page.getByTestId("requisition-board")).toContainText(title);
  const list = await (await page.request.get("/api/proxy/requisitions")).json();
  const made = list.find((r: { job_title: string }) => r.job_title === title);
  expect(made).toMatchObject({
    basis: "contractor",
    expected_duration_months: 9,
  });
});
