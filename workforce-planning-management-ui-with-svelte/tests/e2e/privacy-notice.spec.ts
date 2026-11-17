// Every page's footer links to the privacy notice on the website, in the page's language (WPM-R253).

import { expect, test } from "@playwright/test";

const SITE = "https://workforce-planning-management.github.io";

test.describe("The privacy notice link", () => {
  for (const [route, locale, label] of [
    ["/en-001/tour", "en-001", "Privacy notice"],
    ["/cy-001/help", "cy-001", "Hysbysiad preifatrwydd"],
    ["/ar-001/tour", "ar-001", "إشعار الخصوصية"],
    ["/de-de/help", "de-de", "Datenschutzhinweis"],
  ] as const)
    test(`the footer of ${route} links to the ${locale} notice`, async ({
      page,
    }) => {
      await page.goto(route);
      const link = page.getByRole("contentinfo").getByTestId("privacy-notice");
      await expect(link).toHaveText(label);
      await expect(link).toHaveAttribute("href", `${SITE}/${locale}/privacy/`);
    });
});
