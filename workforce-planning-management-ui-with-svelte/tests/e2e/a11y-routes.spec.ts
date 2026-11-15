// Every page route passes axe's WCAG 2 A and AA rules in the light and the dark theme (automated rules only; they find
// some of the problems, not all). WPM-R247: light and dark are the themes WPM vouches for; another theme is the reader's
// choice and is not held to this (see `spec/lily-design-system-svelte-with-picker-bar/index.md`).
// The routes are listed in `axe-routes.ts`, and `tests/unit/axe-covers-every-route.test.ts` fails if a page is missing from it.
// The API is stubbed as in the other specs, so a page shows what it shows for a signed-in worker whose calls are not stubbed:
// the loading, empty or refused state. Pages with data on screen are checked by the specs written for them (for example
// `sfia-a11y.spec.ts`, `bargaining-a11y.spec.ts`).

import AxeBuilder from "@axe-core/playwright";
import { expect, test } from "@playwright/test";
import { base } from "./fixtures";
import { ROUTES } from "./axe-routes";

for (const theme of ["light", "dark"] as const)
  for (const [route, url] of Object.entries(ROUTES)) {
    test(`axe: ${route} (${theme})`, async ({ page }) => {
      // The app's theme is the Lily theme the picker loads from this saved choice, not the browser's colour scheme
      // alone; set both, and wait until the theme's stylesheet is the one in force.
      await page.emulateMedia({ colorScheme: theme });
      await page.addInitScript((t) => {
        try {
          localStorage.setItem("mxi.wpm.theme", t);
        } catch {
          /* a blocked store only leaves the default theme */
        }
      }, theme);
      await base(page);
      await page.goto(url);
      await page.waitForLoadState("networkidle");
      await page.waitForFunction(
        (t) =>
          document.documentElement.dataset.theme === t &&
          [...document.styleSheets].some((s) =>
            (s.href ?? "").endsWith(`/assets/themes/${t}.css`),
          ),
        theme,
      );
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
      expect(found, `${route} breaks WCAG 2 A/AA rules (${theme})`).toEqual([]);
    });
  }
