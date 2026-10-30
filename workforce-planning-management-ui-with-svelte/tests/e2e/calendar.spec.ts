// The calendar (Lily svelte-calendar) on the CPD page: ledger entries land on their day, the
// requirement period is marked, the view switches, and Welsh names the weekdays and months.

import { expect, test, type Page } from "@playwright/test";

const WORKER = {
  pid: "aaaaaaaa-0000-4000-8000-0000000000c1",
  display_name: "Ana Lyst",
  job_title: "Analyst",
  department: "Data",
  organization_ref: "urn:org:1",
};

async function stub(page: Page, now = "2026-10-14T09:00:00Z") {
  // The calendar opens on today; pin it so the entry is on screen.
  await page.clock.setFixedTime(new Date(now));
  await page.context().addCookies([
    {
      name: "__Host-mxi_session",
      value: "calendar-spec-session",
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
  await page.route("**/api/proxy/me/organizations**", (route) =>
    route.fulfill({ json: [] }),
  );
  await page.route("**/api/proxy/workers", (route) =>
    route.fulfill({ json: [WORKER] }),
  );
  await page.route("**/api/proxy/cpd/overview", (route) =>
    route.fulfill({
      json: {
        derivation: "d",
        headcount: 1,
        requirements: [
          {
            requirement_pid: "r1",
            name: "Annual CPD",
            unit: "hours",
            period_start: "2026-10-01",
            period_end: "2026-10-31",
            applicable_workers: 8,
            met_recorded: { numerator: 6, denominator: 8, value: 0.75 },
            met_verified: null,
          },
        ],
        registrations: { expiring: 0, expired: 0 },
      },
    }),
  );
  await page.route("**/api/proxy/cpd-requirements", (route) =>
    route.fulfill({ json: [] }),
  );
  await page.route(`**/api/proxy/workers/${WORKER.pid}/cpd-progress`, (route) =>
    route.fulfill({
      json: {
        derivation: "d",
        requirements: [
          {
            requirement_pid: "r1",
            name: "Annual CPD",
            unit: "hours",
            period_start: "2026-10-01",
            period_end: "2026-10-31",
            required: 30,
            recorded: 2,
            verified: 0,
            remaining: 28,
            met: false,
            met_verified: false,
          },
        ],
        registrations: [
          {
            body: "Data Council",
            expires_on: "2026-10-20",
            status: "expiring",
          },
        ],
      },
    }),
  );
  await page.route(`**/api/proxy/workers/${WORKER.pid}/cpd-entries`, (route) =>
    route.fulfill({
      json: [
        {
          pid: "e1",
          entry_date: "2026-10-14",
          activity: "Data ethics course",
          category: "course",
          unit: "hours",
          amount: 2,
          evidence_note: null,
          evidence_url: null,
          source: "self",
          verified_on: null,
          verified_by: null,
        },
      ],
    }),
  );
}

test("the CPD calendar shows an entry on its day and the requirement period, and switches view", async ({
  page,
}) => {
  await stub(page);
  await page.goto("/cpd");
  await page.getByTestId("cpd-worker").selectOption(WORKER.pid);
  const calendar = page.getByTestId("cpd-calendar");
  await expect(calendar).toContainText("Data ethics course");
  await expect(calendar).toContainText("Annual CPD");
  await expect(calendar).toContainText("Data Council");
  await expect(
    calendar.getByRole("button", { name: "Week", exact: true }),
  ).toBeVisible();
  await calendar.getByRole("button", { name: "Week", exact: true }).click();
  await expect(calendar.locator("tbody tr")).toHaveCount(1);
  await calendar.screenshot({ path: "test-results/cpd-calendar.png" });
});

test("the calendar names weekdays and months in Welsh", async ({ page }) => {
  await stub(page);
  await page.goto("/cy-001/cpd");
  await page.getByTestId("cpd-worker").selectOption(WORKER.pid);
  const calendar = page.getByTestId("cpd-calendar");
  await expect(calendar).toContainText("Mis");
  await expect(calendar).toContainText("Heddiw");
  await expect(calendar).not.toContainText("Today");
  await expect(calendar).not.toContainText("Monday");
});

for (const [name, viewport, colorScheme] of [
  ["phone-light", { width: 390, height: 800 }, "light"],
  ["phone-dark", { width: 390, height: 800 }, "dark"],
  ["desktop-dark", { width: 1280, height: 900 }, "dark"],
] as const) {
  test(`the calendar fits the page without sideways page scroll (${name})`, async ({
    page,
  }) => {
    await page.setViewportSize(viewport);
    await page.emulateMedia({ colorScheme });
    await page.addInitScript((theme) => {
      try {
        localStorage.setItem("mxi.wpm.theme", theme);
      } catch {
        /* a blocked store only leaves the default theme */
      }
    }, colorScheme);
    await stub(page);
    await page.goto("/cpd");
    await page.getByTestId("cpd-worker").selectOption(WORKER.pid);
    const calendar = page.getByTestId("cpd-calendar");
    await expect(calendar).toContainText("Data ethics course");
    const overflow = await page.evaluate(
      () =>
        document.documentElement.scrollWidth -
        document.documentElement.clientWidth,
    );
    expect(overflow).toBeLessThanOrEqual(0);
    await calendar.screenshot({
      path: `test-results/cpd-calendar-${name}.png`,
    });
  });
}

test("an aggregate table exports what it shows, and an unknown stays a dash, not 0", async ({
  page,
}) => {
  await stub(page);
  await page.goto("/cpd");
  const table = page.getByTestId("cpd-overview");
  await expect(table).toContainText("Annual CPD");
  const download = page.waitForEvent("download");
  await page.getByRole("button", { name: "CSV" }).click();
  const file = await download;
  expect(file.suggestedFilename()).toBe("cpd-overview.csv");
  const path = await file.path();
  const { readFileSync } = await import("node:fs");
  const text = readFileSync(path, "utf8");
  expect(text).toContain("Annual CPD");
  expect(text).toContain("75%");
  expect(text).not.toMatch(/(^|,)"?0"?(,|$)/m);
});

test("a table exports as TSV, as Excel, and copies to the clipboard (WPM-R133)", async ({
  page,
  context,
}) => {
  await context.grantPermissions(["clipboard-read", "clipboard-write"]);
  await stub(page);
  await page.goto("/cpd");
  await expect(page.getByTestId("cpd-overview")).toContainText("Annual CPD");
  const { readFileSync } = await import("node:fs");

  const tsv = page.waitForEvent("download");
  await page.getByRole("button", { name: "TSV" }).click();
  const tsvFile = await tsv;
  expect(tsvFile.suggestedFilename()).toBe("cpd-overview.tsv");
  const tsvText = readFileSync(await tsvFile.path(), "utf8");
  expect(tsvText).toContain("Annual CPD");
  expect(tsvText).toContain("\t");

  const xlsx = page.waitForEvent("download");
  await page.getByRole("button", { name: "Excel" }).click();
  expect((await xlsx).suggestedFilename()).toBe("cpd-overview.xlsx");

  await page.getByRole("button", { name: "Copy" }).click();
  const clip = await page.evaluate(() => navigator.clipboard.readText());
  expect(clip).toContain("Annual CPD");
  expect(clip).toContain("\t");
});

// Today, week start and weekend are the reader's (WPM-T378, WPM-D149).

/** The day number of the cell marked today, and which header columns are weekend days. */
async function todayAndWeekend(page: Page) {
  await page.getByTestId("cpd-worker").selectOption(WORKER.pid);
  const calendar = page.getByTestId("cpd-calendar");
  await expect(calendar).toContainText("Data ethics course");
  const today = await calendar
    .locator('td[aria-current="date"] .cx-calendar-date')
    .textContent();
  const weekend = await calendar
    .locator("thead th")
    .evaluateAll((ths) => ths.map((th) => th.hasAttribute("data-weekend")));
  return { today: today?.trim(), weekend };
}

test.describe("today in a time zone where the UTC date differs (UTC+14)", () => {
  test.use({ timezoneId: "Pacific/Kiritimati" });
  test("noon UTC on the 14th is the 15th here, and the calendar marks the 15th", async ({
    page,
  }) => {
    await stub(page, "2026-10-14T12:00:00Z");
    await page.goto("/cpd");
    expect((await todayAndWeekend(page)).today).toBe("15");
  });
});

test.describe("today in UTC", () => {
  test.use({ timezoneId: "UTC" });
  test("the same instant is the 14th", async ({ page }) => {
    await stub(page, "2026-10-14T12:00:00Z");
    await page.goto("/cpd");
    expect((await todayAndWeekend(page)).today).toBe("14");
  });
});

test("a world-English reader with no country: Monday first, Saturday and Sunday the weekend", async ({
  page,
}) => {
  await stub(page);
  await page.goto("/en-001/cpd");
  const { weekend } = await todayAndWeekend(page);
  expect(weekend).toEqual([false, false, false, false, false, true, true]);
  await expect(
    page.getByTestId("cpd-calendar").locator("thead th").first(),
  ).toContainText("Mon");
});

test.describe("a reader in Egypt on the Arabic page", () => {
  test.use({ locale: "ar-EG" });
  test("the week starts on Saturday and Friday and Saturday are the weekend", async ({
    page,
  }) => {
    await stub(page);
    await page.goto("/ar-001/cpd");
    const { weekend } = await todayAndWeekend(page);
    expect(weekend).toEqual([true, false, false, false, false, false, true]);
    const saturday = await page.evaluate(() =>
      new Intl.DateTimeFormat("ar-001", {
        weekday: "short",
        timeZone: "UTC",
      }).format(new Date(Date.UTC(2023, 0, 7))),
    );
    await expect(
      page.getByTestId("cpd-calendar").locator("thead th").first(),
    ).toContainText(saturday);
    // Weekend days are marked in the grid too, two to a week.
    const rows = await page
      .getByTestId("cpd-calendar")
      .locator("tbody tr")
      .count();
    await expect(
      page.getByTestId("cpd-calendar").locator("tbody td[data-weekend]"),
    ).toHaveCount(rows * 2);
    await page
      .getByTestId("cpd-calendar")
      .screenshot({ path: "test-results/cpd-calendar-ar-eg.png" });
  });
});

test.describe("a reader on the Arabic page with no country", () => {
  test.use({ locale: "ar" });
  test("gets CLDR's world week, not a country guessed from the language", async ({
    page,
  }) => {
    await stub(page);
    await page.goto("/ar-001/cpd");
    const { weekend } = await todayAndWeekend(page);
    expect(weekend).toEqual([false, false, false, false, false, true, true]);
  });
});
