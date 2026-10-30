// Dashboard (home page). The API is stubbed per test; an unstubbed /api/proxy call 404s loudly.

import { expect, test } from "@playwright/test";
import { WORKER, signIn, stubSignedIn } from "./support";

test.describe("Dashboard (home page)", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
  });

  test("dashboard renders live tiles from the stubbed API", async ({
    page,
  }) => {
    await page.goto("/");
    await expect(page.getByTestId("tile-active")).toContainText("2");
    await expect(page.getByTestId("tile-open")).toContainText("1");
    await expect(page.getByTestId("tile-gaps")).toContainText("0");
  });

  test("dashboard lists every organization membership at once, no switcher", async ({
    page,
  }) => {
    await page.goto("/");
    const section = page.getByTestId("my-organizations");
    await expect(section).toContainText(WORKER.organization_ref);
    await expect(section).toContainText("member");
    await expect(section).toContainText("Employed here");
    await expect(section).toContainText(
      "organization:44444444-4444-4444-8444-444444444444",
    );
    await expect(section).toContainText("hr_admin");
    await expect(section).toContainText("Staff access");
  });

  // The CEO dashboard must fit an iPad (9th gen): 2160 × 1620 device pixels,
  // i.e. 1080 × 810 CSS pixels at 2×, with no scrolling — and a literal
  // 2160 × 1620 viewport too. Worst-case text is used so a tile that would
  // overflow actually does; tiles clip silently (`overflow: hidden`), so the
  // check is on each tile's own content, not just the page.
  for (const screen of [
    {
      name: "iPad 9th gen @2x (1080×810 CSS)",
      width: 1080,
      height: 810,
      scale: 2,
    },
    { name: "2160×1620 @1x", width: 2160, height: 1620, scale: 1 },
  ]) {
    test(`CEO dashboard fits ${screen.name} without scrolling or clipping`, async ({
      browser,
    }) => {
      const context = await browser.newContext({
        viewport: { width: screen.width, height: screen.height },
        deviceScaleFactor: screen.scale,
        locale: "en",
      });
      const page = await context.newPage();
      await signIn(page);
      await page.route("**/api/proxy/**", (route) =>
        route.fulfill({ status: 404, body: "unstubbed" }),
      );
      await page.route("**/api/proxy/me/organizations**", (route) =>
        route.fulfill({ json: [] }),
      );
      await page.route("**/api/proxy/me/organizations/scope", (route) =>
        route.fulfill({
          json: ["organization:00000000-0000-4000-8000-0000000000aa"],
        }),
      );
      await page.route("**/api/proxy/escalations?**", (route) =>
        route.fulfill({
          json: [{ days_open: 3 }, { days_open: 41 }, { days_open: null }],
        }),
      );
      let n = 0;
      const metricsFrom: string[] = [];
      await page.route(
        "**/api/proxy/workforce-intelligence/metrics**",
        (route) => {
          metricsFrom.push(
            new URL(route.request().url()).searchParams.get("from") ?? "",
          );
          return route.fulfill({
            json: {
              period: { from: "2025-10-05", to: "2026-10-05" },
              definitions: {},
              headcount: {
                opening: 1180 + (n++ % 7) * 9,
                closing: 12345,
                opening_date: "2025-10-04",
              },
              starters: 210,
              leavers: 1234,
              turnover_rate: 0.1234,
              span_of_control: { managers: 1234, mean: 12.34, max: 40 },
              time_to_fill: {
                requisitions: 99,
                mean_days: 61.2,
                median_days: 58.7,
              },
            },
          });
        },
      );

      const long =
        "Break leavers down by department and tenure, and compare with pulse-survey and exit feedback across every organization";
      await page.route(
        "**/api/proxy/workforce-intelligence/insights**",
        (route) =>
          route.fulfill({
            json: {
              period: { from: "2025-10-05", to: "2026-10-05" },
              thresholds: {},
              insights: ["a", "b", "c", "d", "e"].map((c) => ({
                code: `unknown_${c}`,
                severity: "attention",
                observation: long,
                suggestion: long,
                params: {},
              })),
            },
          }),
      );
      await page.route("**/api/proxy/succession-plans/gaps", (route) =>
        route.fulfill({ json: { gaps: [{}, {}, {}] } }),
      );
      await page.route("**/api/proxy/requisitions?status=open", (route) =>
        route.fulfill({ json: [{}, {}, {}, {}, {}, {}] }),
      );
      await page.goto("/ceo");
      await expect(page.getByTestId("kpi-headcount")).toContainText("12,345");
      await expect(page.getByTestId("trend-chart")).toBeVisible();
      await expect(page.getByTestId("ceo-insights")).toContainText("Attention");
      await expect(page.getByTestId("kpi-escalations")).toContainText("3");
      await expect(page.getByTestId("kpi-escalations")).toContainText(
        "oldest 41 calendar days",
      );

      const fit = await page.evaluate(() => {
        const doc = document.documentElement;
        const tiles = [
          ...document.querySelectorAll<HTMLElement>(".ceo .cx-tile"),
        ].map((el) => {
          const box = el.getBoundingClientRect();
          return {
            name: el.dataset.testid ?? el.className,
            inView:
              box.left >= 0 &&
              box.top >= 0 &&
              box.right <= window.innerWidth + 0.5 &&
              box.bottom <= window.innerHeight + 0.5,
            clipsY: el.scrollHeight - el.clientHeight,
            clipsX: el.scrollWidth - el.clientWidth,
          };
        });
        return {
          pageY: doc.scrollHeight - window.innerHeight,
          pageX: doc.scrollWidth - window.innerWidth,
          bodyY: document.body.scrollHeight - window.innerHeight,
          tiles,
        };
      });
      expect(fit.pageY, "page scrolls vertically").toBeLessThanOrEqual(0);
      expect(fit.pageX, "page scrolls horizontally").toBeLessThanOrEqual(0);
      expect(fit.bodyY, "body scrolls").toBeLessThanOrEqual(0);
      expect(fit.tiles).toHaveLength(7);
      for (const tile of fit.tiles) {
        expect(tile.inView, `${tile.name} is inside the screen`).toBe(true);
        // Tiles clip silently; a tile whose content is taller or wider than
        // its box is clipping something the CEO needs to read.
        expect(
          tile.clipsY,
          `${tile.name} clips vertically`,
        ).toBeLessThanOrEqual(1);
        expect(
          tile.clipsX,
          `${tile.name} clips horizontally`,
        ).toBeLessThanOrEqual(1);
      }
      await page.screenshot({
        path: `test-results/ceo-${screen.width}x${screen.height}.png`,
      });

      // Each tile drills down to the page behind it.
      const hrefOf = (id: string) => page.getByTestId(id).getAttribute("href");
      expect(await hrefOf("kpi-headcount")).toMatch(/\/en-001\/metrics$/);
      expect(await hrefOf("kpi-open")).toMatch(/\/en-001\/requisitions$/);
      expect(await hrefOf("kpi-gaps")).toMatch(/\/en-001\/development$/);
      expect(await hrefOf("kpi-escalations")).toMatch(/\/en-001\/planning$/);

      // The period control reloads the period-based figures (and is in the URL).
      await expect(page.getByTestId("range-12m")).toHaveAttribute(
        "aria-pressed",
        "true",
      );
      await page.getByTestId("range-90d").click();
      await expect(page.getByTestId("range-90d")).toHaveAttribute(
        "aria-pressed",
        "true",
      );
      await expect(page).toHaveURL(/range=90d/);
      const ninetyDaysBack = new Date(Date.now() - 89 * 86_400_000)
        .toISOString()
        .slice(0, 10);
      await expect.poll(() => metricsFrom.includes(ninetyDaysBack)).toBe(true);
      await context.close();
    });
  }
});
