// Emergency contacts and backups. The API is stubbed per test; an unstubbed /api/proxy call 404s loudly.

import { expect, test } from "@playwright/test";
import { MASKED_WORKER, WORKER, stubSignedIn } from "./support";

test.describe("Emergency contacts and backups", () => {
  test.beforeEach(async ({ page }) => {
    await stubSignedIn(page);
  });

  test("a person provides emergency contacts and names a backup", async ({
    page,
  }) => {
    const base = `**/api/proxy/workers/${WORKER.pid}`;
    const contacts: Array<Record<string, unknown>> = [];
    const backups: Array<Record<string, unknown>> = [];
    const colleague = MASKED_WORKER;
    await page.route(`${base}/emergency-contacts`, async (route) => {
      if (route.request().method() === "POST") {
        const body = route.request().postDataJSON();
        contacts.push({
          pid: `c${contacts.length + 1}`,
          priority: contacts.length + 1,
          alt_phone: null,
          email: null,
          note: null,
          on_behalf: false,
          ...body,
        });
        return route.fulfill({ json: { pid: `c${contacts.length}` } });
      }
      return route.fulfill({ json: contacts });
    });
    await page.route(`${base}/backups`, async (route) => {
      if (route.request().method() === "POST") {
        const body = route.request().postDataJSON();
        backups.push({
          pid: `b${backups.length + 1}`,
          priority: backups.length + 1,
          backup_name: colleague.display_name,
          backup_title: colleague.job_title,
          starts_on: null,
          ends_on: null,
          note: null,
          on_behalf: false,
          ...body,
        });
        return route.fulfill({ json: { pid: `b${backups.length}` } });
      }
      return route.fulfill({ json: backups });
    });
    await page.route(`${base}/cover*`, (route) =>
      route.fulfill({
        json: {
          on: "2026-10-05",
          worker_pid: WORKER.pid,
          covered_by: backups.length ? colleague.pid : null,
          covered_by_name: backups.length ? colleague.display_name : null,
          covered_by_title: null,
          backups_named: backups.length,
        },
      }),
    );

    await page.goto("/me");
    const panel = page.getByTestId("emergency-contacts");
    await expect(panel).toContainText("Emergency contacts");
    await expect(panel).toContainText("No emergency contacts yet.");
    await page.getByTestId("contact-name").fill("Sam Lee");
    await page.getByTestId("contact-relationship").fill("Partner");
    await page.getByTestId("contact-phone").fill("+44 7700 900123");
    await page.getByTestId("contact-add").click();
    await expect(page.getByTestId("contact-list")).toContainText("Sam Lee");
    await expect(page.getByTestId("contact-list")).toContainText("Partner");

    const backupsPanel = page.getByTestId("backups");
    await expect(backupsPanel).toContainText("No backup named yet.");
    await page.getByTestId("backup-choice").selectOption(colleague.pid);
    await page.getByTestId("backup-add").click();
    await expect(page.getByTestId("backup-list")).toContainText(
      colleague.display_name,
    );
    await expect(page.getByTestId("cover-today")).toContainText(
      colleague.display_name,
    );
  });
});
