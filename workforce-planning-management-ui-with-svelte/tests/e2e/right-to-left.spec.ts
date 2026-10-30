// Right-to-left (WPM-T377; spec/worldwide.md rows 28 to 30). On an Arabic and an Urdu page: `<html dir="rtl">`, a
// table's cells start on the right (`text-align: start`, not `left`), a period is the locale's own range with no
// arrow, the menu drawer opens from the start (right) edge and its links navigate, and axe (WCAG 2 A and AA) passes.
// The API is stubbed; an unstubbed /api/proxy call 404s loudly.

import AxeBuilder from "@axe-core/playwright";
import { expect, test } from "@playwright/test";
import { WORKER, stubSignedIn } from "./support";

const RUNS = [
  {
    pid: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
    organization_ref: WORKER.organization_ref,
    period_start: "2026-09-01",
    period_end: "2026-09-30",
    status: "paid",
  },
  {
    pid: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
    organization_ref: WORKER.organization_ref,
    period_start: "2026-10-01",
    period_end: "2026-10-31",
    status: "draft",
  },
];

for (const locale of ["ar-001", "ur-001"]) {
  test.describe(`right to left in ${locale}`, () => {
    test.beforeEach(async ({ page }) => {
      await stubSignedIn(page);
      await page.route("**/api/proxy/payroll-runs?limit=500", (route) =>
        route.fulfill({ json: RUNS }),
      );
    });

    test("a table starts on the right and a period is the locale's range, with no arrow", async ({
      page,
    }) => {
      await page.goto(`/${locale}/payroll`);
      await expect(page.locator("html")).toHaveAttribute("dir", "rtl");
      await expect(page.locator("html")).toHaveAttribute("lang", locale);
      const table = page.getByTestId("runs");
      const expected = await page.evaluate(
        (loc) =>
          new Intl.DateTimeFormat(loc, {
            dateStyle: "medium",
            timeZone: "UTC",
          }).formatRange(
            new Date("2026-10-01T00:00:00Z"),
            new Date("2026-10-31T00:00:00Z"),
          ),
        locale,
      );
      const link = table.getByRole("link", { name: expected });
      await expect(link).toBeVisible();
      await expect(table).not.toContainText("→");
      await expect(table).not.toContainText("2026-10-01");

      // Every header and data cell is aligned to the start of the line, which is the right here.
      const aligns = await table.locator("th, td").evaluateAll((cells) =>
        cells.map((c) => {
          const s = getComputedStyle(c);
          return `${s.direction} ${s.textAlign}`;
        }),
      );
      expect(aligns.length).toBeGreaterThan(0);
      for (const a of aligns) expect(["rtl start", "rtl right"]).toContain(a);
      expect(aligns.filter((a) => a.endsWith("left"))).toEqual([]);

      // …and the text sits at the cell's right edge, not its left.
      const cell = link.locator("xpath=ancestor::*[self::td or self::th][1]");
      const [cellBox, linkBox] = await Promise.all([
        cell.boundingBox(),
        link.boundingBox(),
      ]);
      expect(cellBox && linkBox).toBeTruthy();
      if (cellBox && linkBox) {
        const rightGap =
          cellBox.x + cellBox.width - (linkBox.x + linkBox.width);
        const leftGap = linkBox.x - cellBox.x;
        expect(rightGap).toBeLessThan(leftGap);
      }

      const result = await new AxeBuilder({ page })
        .withTags(["wcag2a", "wcag2aa"])
        .analyze();
      expect(
        result.violations.map(
          (v) => `${v.id}: ${v.nodes.map((n) => n.html).join(" | ")}`,
        ),
      ).toEqual([]);
      await page.screenshot({
        path: `test-results/right-to-left-${locale}-payroll.png`,
        fullPage: true,
      });
    });

    test("the menu opens from the right edge and its links navigate", async ({
      page,
    }) => {
      await page.goto(`/${locale}/payroll`);
      await expect(page.getByTestId("runs")).toBeVisible();
      // The share button's forward arrow points left here (mirrored), and right on a left-to-right page.
      await expect(page.locator(".share-picker-icon")).toHaveCSS(
        "transform",
        "matrix(-1, 0, 0, 1, 0, 0)",
      );
      await page.locator("nav.top .hamburger-menu-button").click();
      const drawer = page.locator(".drawer");
      await expect(drawer).toBeVisible();
      const box = await drawer.boundingBox();
      const width = page.viewportSize()?.width ?? 0;
      expect(box).toBeTruthy();
      if (box) {
        // Flush with the right edge (the inline start), not the left.
        expect(Math.abs(box.x + box.width - width)).toBeLessThanOrEqual(2);
        expect(box.x).toBeGreaterThan(0);
      }
      const result = await new AxeBuilder({ page })
        .include(".drawer")
        .withTags(["wcag2a", "wcag2aa"])
        .analyze();
      expect(
        result.violations.map(
          (v) => `${v.id}: ${v.nodes.map((n) => n.html).join(" | ")}`,
        ),
      ).toEqual([]);
      await page.screenshot({
        path: `test-results/right-to-left-${locale}-menu.png`,
      });
      await drawer.locator(`a[href="/${locale}/workers"]`).click();
      await expect(page).toHaveURL(new RegExp(`/${locale}/workers$`));
      await expect(drawer).toBeHidden();
      await expect(page.locator("html")).toHaveAttribute("dir", "rtl");
    });
  });
}

test("the control: an English page stays left to right, its arrow unmirrored", async ({
  page,
}) => {
  await stubSignedIn(page);
  await page.route("**/api/proxy/payroll-runs?limit=500", (route) =>
    route.fulfill({ json: RUNS }),
  );
  await page.goto("/en-001/payroll");
  await expect(page.locator("html")).toHaveAttribute("dir", "ltr");
  await expect(page.getByTestId("runs")).toContainText("1–31 Oct 2026");
  await expect(page.locator(".share-picker-icon")).toHaveCSS(
    "transform",
    "none",
  );
  const align = await page
    .getByTestId("runs")
    .locator("td")
    .first()
    .evaluate((c) => getComputedStyle(c).textAlign);
  expect(align).toBe("start");
});
