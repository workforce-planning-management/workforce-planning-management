// Every page route passes axe's WCAG 2 A and AA rules (automated rules only; they find some of the problems, not all).
// The routes are listed in `axe-routes.ts`, and `tests/unit/axe-covers-every-route.test.ts` fails if a page is missing from it.
// The API is stubbed as in the other specs, so a page shows what it shows for a signed-in worker whose calls are not stubbed:
// the loading, empty or refused state. Pages with data on screen are checked by the specs written for them (for example
// `sfia-a11y.spec.ts`, `bargaining-a11y.spec.ts`).

import AxeBuilder from "@axe-core/playwright";
import { expect, test } from "@playwright/test";
import { base } from "./fixtures";
import { ROUTES } from "./axe-routes";

for (const [route, url] of Object.entries(ROUTES)) {
  test(`axe: ${route}`, async ({ page }) => {
    await base(page);
    await page.goto(url);
    await page.waitForLoadState("networkidle");
    const result = await new AxeBuilder({ page })
      .withTags(["wcag2a", "wcag2aa"])
      .analyze();
    const found = result.violations.map(
      (v) =>
        `${v.id} (${v.impact}): ${v.nodes
          .slice(0, 3)
          .map((n) => `${n.target.join(" ")} [${n.any[0]?.message ?? ""}]`)
          .join(" | ")}`,
    );
    expect(found, `${route} breaks WCAG 2 A/AA rules`).toEqual([]);
  });
}
