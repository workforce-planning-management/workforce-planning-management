// WPM-R247 criterion 2: a reader who has not chosen a theme gets light or dark, matching the browser's colour scheme;
// a theme the reader chose wins over the browser.

import { expect, test } from "@playwright/test";
import { base } from "./fixtures";

const THEME_KEY = "mxi.wpm.theme";

for (const scheme of ["light", "dark"] as const)
  test(`with no saved theme, the page follows a ${scheme} browser`, async ({
    page,
  }) => {
    await page.emulateMedia({ colorScheme: scheme });
    await base(page);
    await page.goto("/help");
    await expect(page.locator("html")).toHaveAttribute("data-theme", scheme);
  });

test("a theme the reader chose wins over the browser's colour scheme", async ({
  page,
}) => {
  await page.emulateMedia({ colorScheme: "dark" });
  await page.addInitScript((key) => {
    try {
      localStorage.setItem(key, "nord");
    } catch {
      /* a blocked store only leaves the default theme */
    }
  }, THEME_KEY);
  await base(page);
  await page.goto("/help");
  await expect(page.locator("html")).toHaveAttribute("data-theme", "nord");
});
