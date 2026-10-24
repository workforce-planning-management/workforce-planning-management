// Announcements. The API is stubbed per test; an unstubbed /api/proxy call 404s loudly.

import { expect, test } from "@playwright/test";
import { WORKER, stubSignedIn } from "./support";

test.describe("Announcements", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
  });

  test("announcements show pinned first as plain text, and editors post", async ({
    page,
  }) => {
    const posts: Array<Record<string, unknown>> = [
      {
        pid: "n1",
        organization_ref: WORKER.organization_ref,
        title: "Office closed Monday",
        body: "Line one\nLine two with <b>no markup</b>",
        pinned: true,
        publish_on: "2026-10-01",
        expires_on: null,
        department: "Finance",
        links: [
          { label: "Handbook", url: "https://intranet.example.org/hb" },
          // Never rendered as a link, whatever the service sent.
          { label: "Sneaky", url: "javascript:alert(1)" },
        ],
        read_count: 4,
        status: "live",
        author: null,
      },
    ];
    const reads: string[] = [];
    await page.route(
      `**/api/proxy/workers/${WORKER.pid}/announcement-reads`,
      (route) => route.fulfill({ json: reads }),
    );
    await page.route("**/api/proxy/announcements/n1/read", (route) => {
      reads.push("n1");
      return route.fulfill({ json: { read: true } });
    });
    await page.route("**/api/proxy/announcements**", (route) => {
      if (route.request().url().includes("/read")) return route.fallback();
      if (route.request().method() === "POST") {
        posts.push({
          pid: `n${posts.length + 1}`,
          expires_on: null,
          status: "live",
          author: null,
          publish_on: "2026-10-05",
          ...route.request().postDataJSON(),
        });
        return route.fulfill({ json: { pid: "n2" } });
      }
      return route.fulfill({ json: posts });
    });

    await page.goto("/announcements");
    const list = page.getByTestId("announcement-list");
    await expect(list).toContainText("Office closed Monday");
    await expect(list).toContainText("Pinned");
    // Markup in a post is shown as text, never interpreted.
    await expect(list).toContainText("<b>no markup</b>");
    await expect(list.locator("b")).toHaveCount(0);

    // Department audience, https links only, and read receipts.
    await expect(list).toContainText("Finance");
    const links = page.getByTestId("announcement-links").locator("a");
    await expect(links).toHaveCount(1);
    await expect(links.first()).toHaveAttribute(
      "href",
      "https://intranet.example.org/hb",
    );
    await expect(links.first()).toHaveAttribute("rel", /noopener/);
    await expect(page.getByTestId("read-count")).toHaveText("4 read");
    await expect(page.getByTestId("unread")).toBeVisible();
    await page.getByTestId("mark-read").click();
    await expect(page.getByTestId("unread")).toHaveCount(0);

    await page.getByText("Post an announcement").first().click();
    await page.getByTestId("announcement-title").fill("Town hall Friday");
    await page.getByTestId("announcement-body").fill("All hands at 3pm.");
    await page.getByTestId("announcement-post").click();
    await expect(list).toContainText("Town hall Friday");
  });
});
