// A person's allocations on /me (WPM-R275, WPM-T599): the months with their signals, unknown where untracked, never a
// score, never the kind of leave; hidden for someone who may not read them. The API is stubbed per test.

import AxeBuilder from "@axe-core/playwright";
import { expect, test, type Page } from "@playwright/test";
import { WORKER, stubSignedIn } from "./support";

const ALLOCATION = {
  pid: "a1",
  assignment_pid: "as1",
  source_ref: "thing:programme-a",
  pool_pid: null,
  from_month: "2027-01-01",
  to_month: "2027-02-01",
  fte_centi: 60,
  status: "confirmed",
  note: null,
  runs_past_end: "2027-01-31",
};

async function stub(page: Page, body: unknown, status = 200) {
  await page.route(
    (url) => url.pathname === `/api/proxy/workers/${WORKER.pid}/allocations`,
    (route) => route.fulfill({ status, json: body }),
  );
}

test.describe("Allocations on /me", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
  });

  test("a tracked person sees each month's standing and its signals", async ({
    page,
  }) => {
    await stub(page, {
      worker_pid: WORKER.pid,
      display_name: "Test Worker",
      tracked: true,
      allocations: [ALLOCATION],
      months: [
        {
          month: "2027-01-01",
          standing: "known",
          contracted_centi: 100,
          available_centi: 100,
          allocated_centi: 110,
          tentative_centi: 0,
          signals: [
            { kind: "over_allocated", by_centi: 10 },
            {
              kind: "assignment_over_allocated",
              assignment_pid: "as1",
              by_centi: 10,
            },
          ],
        },
        {
          month: "2027-02-01",
          standing: "known",
          contracted_centi: 100,
          available_centi: 50,
          allocated_centi: 60,
          tentative_centi: 0,
          signals: [{ kind: "leave_clash", short_centi: 10 }],
        },
        {
          month: "2027-03-01",
          standing: "known",
          contracted_centi: 100,
          available_centi: 100,
          allocated_centi: 0,
          tentative_centi: 0,
          signals: [{ kind: "available", free_centi: 100 }],
        },
      ],
    });
    await page.goto("/me");
    const panel = page.getByTestId("allocations");
    await expect(panel.getByTestId("allocations-list")).toContainText(
      "thing:programme-a",
    );
    await expect(panel.getByTestId("allocations-list")).toContainText("0.60");
    await expect(panel.getByTestId("allocations-list")).toContainText(
      "Confirmed",
    );
    await expect(panel.getByTestId("allocations-past-end")).toContainText(
      "Runs past the end",
    );
    const months = panel.getByTestId("allocations-months");
    await expect(months).toContainText("1.10 of 1.00 FTE allocated");
    await expect(months).toContainText("▲ Over-allocated by 0.10 FTE");
    await expect(months).toContainText(
      "▲ Approved leave leaves it 0.10 FTE short",
    );
    await expect(months).toContainText("● 1.00 FTE free");
    await expect(panel.getByTestId("allocations-untracked")).toHaveCount(0);
    await expect(panel).not.toContainText(/sick|utili[sz]ation|%/i);
  });

  test("an untracked organization shows a dash, never zero", async ({
    page,
  }) => {
    await stub(page, {
      worker_pid: WORKER.pid,
      display_name: "Test Worker",
      tracked: false,
      allocations: [],
      months: [{ month: "2027-01-01", standing: "unknown" }],
    });
    await page.goto("/me");
    const panel = page.getByTestId("allocations");
    await expect(panel.getByTestId("allocations-untracked")).toBeVisible();
    await expect(panel.getByTestId("allocations-none")).toBeVisible();
    await expect(panel.getByTestId("allocations-months")).toContainText("—");
    await expect(panel.getByTestId("allocations-months")).not.toContainText(
      "0.00",
    );
  });

  test("someone who may not read them sees no panel", async ({ page }) => {
    await stub(page, { title: "Forbidden" }, 403);
    await page.goto("/me");
    await expect(page.getByTestId("allocations")).toHaveCount(0);
  });

  for (const theme of ["light", "dark"]) {
    test(`the panel passes axe in the ${theme} theme`, async ({ page }) => {
      await page.addInitScript(
        (t) => localStorage.setItem("mxi.wpm.theme", t),
        theme,
      );
      await stub(page, {
        worker_pid: WORKER.pid,
        display_name: "Test Worker",
        tracked: true,
        allocations: [ALLOCATION],
        months: [
          {
            month: "2027-01-01",
            standing: "known",
            contracted_centi: 100,
            available_centi: 100,
            allocated_centi: 60,
            tentative_centi: 0,
            signals: [{ kind: "available", free_centi: 40 }],
          },
        ],
      });
      await page.goto("/me");
      await expect(page.getByTestId("allocations-list")).toBeVisible();
      const result = await new AxeBuilder({ page })
        .include('[data-testid="allocations"]')
        .withTags(["wcag2a", "wcag2aa"])
        .analyze();
      expect(result.violations.map((v) => v.id)).toEqual([]);
    });
  }
});

test.describe("The allocations grid", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
  });

  const person = (name: string, status: string) => ({
    worker_pid: `w-${name}`,
    display_name: name,
    allocations: [{ ...ALLOCATION, pid: `a-${name}`, status }],
    months: [
      {
        month: "2027-01-01",
        standing: "known",
        contracted_centi: 100,
        available_centi: 100,
        allocated_centi: 60,
        tentative_centi: 0,
        signals: [{ kind: "available", free_centi: 40 }],
      },
    ],
  });

  test("lists people in the order given with their months, and confirms a tentative allocation", async ({
    page,
  }) => {
    const moved: unknown[] = [];
    await page.route(
      (url) => url.pathname === "/api/proxy/allocations",
      (route) =>
        route.fulfill({
          json: [person("Ada", "tentative"), person("Bo", "confirmed")],
        }),
    );
    await page.route(
      (url) => url.pathname === "/api/proxy/allocations/a-Ada/status",
      (route) => {
        moved.push(route.request().postDataJSON());
        return route.fulfill({ json: { pid: "a-Ada", status: "confirmed" } });
      },
    );
    await page.goto("/allocations");
    const rows = page.getByTestId("allocations-row");
    await expect(rows).toHaveCount(2);
    await expect(rows.first()).toContainText("Ada");
    await expect(rows.first()).toContainText("0.60 of 1.00 FTE allocated");
    await expect(rows.first()).toContainText("● 0.40 FTE free");
    await expect(page.getByTestId("allocation-confirm")).toHaveCount(1);
    await page.getByTestId("allocation-confirm").click();
    await expect.poll(() => moved).toEqual([{ to: "confirmed" }]);
  });

  test("a resource manager adds an allocation to one of a person's current jobs", async ({
    page,
  }) => {
    const posted: unknown[] = [];
    await page.route(
      (url) => url.pathname === "/api/proxy/allocations",
      (route) => route.fulfill({ json: [person("Ada", "confirmed")] }),
    );
    await page.route(
      (url) => url.pathname === "/api/proxy/workers/w-Ada/assignments",
      (route) =>
        route.fulfill({
          json: {
            worker_pid: "w-Ada",
            fte_total: 100,
            assignments: [
              {
                pid: "as-now",
                job_title: "Engineer",
                department: "eng",
                fte_percent: 80,
                current: true,
              },
              {
                pid: "as-old",
                job_title: "Intern",
                department: "eng",
                fte_percent: 50,
                current: false,
              },
            ],
          },
        }),
    );
    await page.route(
      (url) => url.pathname === "/api/proxy/workers/w-Ada/allocations",
      (route) => {
        posted.push(route.request().postDataJSON());
        return route.fulfill({ json: { pid: "a-new" } });
      },
    );
    await page.goto("/allocations");
    const form = page.getByTestId("allocation-form");
    await form.getByTestId("allocation-person").selectOption("w-Ada");
    await expect(
      form.getByTestId("allocation-job").locator("option"),
    ).toHaveCount(1);
    await form.getByTestId("allocation-source").fill("thing:programme-b");
    await form.getByTestId("allocation-from").fill("2027-03");
    await form.getByTestId("allocation-to").fill("2027-05");
    await form.getByTestId("allocation-share").fill("40");
    await form.getByTestId("allocation-confirmed").check();
    await form.getByTestId("allocation-add").click();
    await expect
      .poll(() => posted)
      .toEqual([
        {
          assignment_pid: "as-now",
          source_ref: "thing:programme-b",
          from_month: "2027-03-01",
          to_month: "2027-05-01",
          fte_centi: 40,
          status: "confirmed",
        },
      ]);
  });

  test("someone who is not HR or a resource manager is told so", async ({
    page,
  }) => {
    await page.route(
      (url) => url.pathname === "/api/proxy/allocations",
      (route) => route.fulfill({ status: 403, json: { title: "Forbidden" } }),
    );
    await page.goto("/allocations");
    await expect(page.getByTestId("allocations-not-allowed")).toBeVisible();
    await expect(page.getByTestId("allocations-grid")).toHaveCount(0);
  });

  for (const theme of ["light", "dark"]) {
    test(`the grid passes axe in the ${theme} theme`, async ({ page }) => {
      await page.addInitScript(
        (t) => localStorage.setItem("mxi.wpm.theme", t),
        theme,
      );
      await page.route(
        (url) => url.pathname === "/api/proxy/allocations",
        (route) => route.fulfill({ json: [person("Ada", "tentative")] }),
      );
      await page.goto("/allocations");
      await expect(page.getByTestId("allocations-grid")).toBeVisible();
      const result = await new AxeBuilder({ page })
        .include('[data-testid="allocations-grid"]')
        .include('[data-testid="allocation-form"]')
        .withTags(["wcag2a", "wcag2aa"])
        .analyze();
      expect(result.violations.map((v) => v.id)).toEqual([]);
    });
  }
});
