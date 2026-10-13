// Capacity, oversight, cutover, early warnings and resilience against the real service (WPM-T118; see
// README.md). Each run uses its own organization, so nothing depends on what is in the database, and it
// asserts rules and shapes, never exact figures.

import { expect, test, type Page } from "@playwright/test";

test.skip(
  !process.env.WPM_LIVE_API,
  "needs a running service: set WPM_LIVE_API=1",
);

const ORG = `organization:${crypto.randomUUID()}`;
const tag = crypto.randomUUID().slice(0, 8);

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

/** With enforcement off the caller has no organizations to choose from, so the reference is typed. */
async function chooseOrg(page: Page) {
  await page.getByTestId("org-input").fill(ORG);
}

test.beforeEach(async ({ page }) => {
  await signIn(page);
});

test("a pool made in the UI appears in the heatmap, every month a known result and never blank", async ({
  page,
}) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));
  await page.goto("/capacity");
  await chooseOrg(page);
  await expect(page.getByTestId("capacity-none")).toBeVisible();
  await page.getByTestId("pool-name").fill(`Live pool ${tag}`);
  await page.getByTestId("create-pool").click();
  await expect(page.getByTestId("heatmap")).toBeVisible();
  await expect(page.getByTestId("heat-row")).toContainText(`Live pool ${tag}`);
  const statuses = await page
    .getByTestId("heat-cell")
    .evaluateAll((cells) => cells.map((c) => c.getAttribute("data-status")));
  expect(statuses.length).toBeGreaterThan(0);
  for (const status of statuses) {
    expect(["fits", "fits_with_partners", "over", "unknown"]).toContain(status);
  }
  expect(errors).toEqual([]);
});

test("a level and a position recorded in the UI show as the current level; a pack freezes and opens", async ({
  page,
}) => {
  await page.goto("/oversight");
  await chooseOrg(page);
  await expect(page.getByTestId("no-position")).toBeVisible();
  await page.getByTestId("level-ordinal").fill("2");
  await page.getByTestId("level-name").fill(`Live level ${tag}`);
  await page.getByTestId("add-level").click();
  await page
    .getByTestId("pos-level")
    .selectOption({ label: `2. Live level ${tag}` });
  await page.getByTestId("pos-on").fill("2026-01-01");
  await page.getByTestId("pos-body").fill("Synthetic committee");
  await page.getByTestId("pos-reason").fill("Synthetic reason");
  await page.getByTestId("record-position").click();
  await expect(page.getByTestId("level")).toHaveText(`Live level ${tag}`);
  await page.getByTestId("freeze").click();
  await expect(page.getByTestId("pack")).toHaveCount(1);
  await page.getByTestId("pack").getByRole("button").click();
  const detail = page.getByTestId("pack-detail");
  await expect(detail).toContainText(`Live level ${tag}`);
  // The deciding body is in the status, not in a frozen pack.
  await expect(detail).not.toContainText("Synthetic committee");
});

test("a cutover planned in the UI is never read as ready without roles, a rota and pools", async ({
  page,
}) => {
  await page.goto("/cutover");
  await chooseOrg(page);
  await expect(page.getByTestId("no-cutovers")).toBeVisible();
  await page.getByTestId("cut-programme").fill(`thing:${crypto.randomUUID()}`);
  await page.getByTestId("cut-name").fill(`Live cutover ${tag}`);
  await page.getByTestId("cut-site").fill("Synthetic Site");
  await page.getByTestId("cut-start").fill("2099-01-04");
  await page.getByTestId("cut-end").fill("2099-01-17");
  await page.getByTestId("plan-create").click();
  const cutover = page.getByTestId("cutover");
  await expect(cutover).toContainText(`Live cutover ${tag}`);
  await expect(page.getByTestId("overall")).toHaveAttribute(
    "data-status",
    "unknown",
  );
  for (const check of ["trained_staff", "hypercare_cover", "key_person_risk"]) {
    await expect(page.getByTestId(`check-${check}`)).toHaveAttribute(
      "data-status",
      "unknown",
    );
  }
  // The reasons are said in words (the code was known), not left as a raw token.
  await expect(page.getByTestId("check-trained_staff")).toContainText(
    "No required role is listed",
  );
  page.once("dialog", (d) => d.accept());
  await page.getByTestId("remove-cutover").click();
  await expect(page.getByTestId("no-cutovers")).toBeVisible();
});

test("early warnings look now and list; resilience reads for the organization", async ({
  page,
}) => {
  await page.goto("/warnings");
  await chooseOrg(page);
  await page.getByTestId("sweep").click();
  await expect(page.getByTestId("swept")).toHaveText(
    /^Opened \d+, cleared \d+, told \d+$/,
  );
  await expect(page.getByTestId("time-to-report")).toBeVisible();
  await page.goto("/resilience");
  await chooseOrg(page);
  await expect(page.getByTestId("spof-count")).toContainText(/\d+/);
  await expect(page.getByTestId("governance")).toBeVisible();
});
