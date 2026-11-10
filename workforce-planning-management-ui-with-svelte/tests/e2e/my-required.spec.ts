// What is required of me (WPM-R245): from /me, one page listing mandatory training and the certifications the person's role asks
// for, most urgent first, with expiry days and a link to act on each. The API is stubbed per test; an unstubbed call 404s loudly.

import { expect, test, type Page } from "@playwright/test";
import { stubSignedIn, WORKER } from "./support";

const W = `**/api/proxy/workers/${WORKER.pid}`;

const learningRow = (
  subject: string,
  status: string,
  expires_on: string | null,
  last: string | null = null,
) => ({
  requirement_pid: `req-${subject}`,
  subject,
  basis: "mandatory",
  source: null,
  status,
  last_completed_on: last,
  expires_on,
});

const qualification = (over: Record<string, unknown>) => ({
  pid: `q-${String(over.title)}`,
  kind: "professional",
  title: "First aid at work",
  awarding_body: null,
  framework: null,
  level: null,
  awarded_on: null,
  expires_on: null,
  status: "verified",
  expired: false,
  verified_on: null,
  evidence_ref: null,
  ...over,
});

async function stubRequired(
  page: Page,
  opts: { allClear?: boolean; noRole?: boolean } = {},
) {
  await stubSignedIn(page);
  await page.route(`${W}/mandatory-learning`, (route) =>
    route.fulfill({
      json: {
        worker_pid: WORKER.pid,
        as_of: "2026-11-10",
        requirements: opts.allClear
          ? [
              learningRow(
                "Fire safety",
                "compliant",
                "2027-05-01",
                "2026-05-01",
              ),
            ]
          : [
              learningRow(
                "Fire safety",
                "compliant",
                "2027-05-01",
                "2026-05-01",
              ),
              learningRow(
                "Information governance",
                "expired",
                "2026-09-30",
                "2025-09-30",
              ),
              learningRow(
                "Manual handling",
                "due_soon",
                "2026-12-01",
                "2025-12-01",
              ),
              learningRow("Moving people", "not_applicable", null),
              learningRow("Safeguarding", "never_completed", null),
              learningRow("Conflict resolution", "unknown", null),
            ],
      },
    }),
  );
  await page.route(`${W}/qualifications`, (route) =>
    route.fulfill({
      json: {
        worker_pid: WORKER.pid,
        as_of: "2026-11-10",
        role: opts.noRole ? null : { pid: "role-1", job_title: "Engineer" },
        qualifications: opts.allClear
          ? [qualification({ expires_on: "2028-03-31" })]
          : [
              qualification({ expires_on: "2028-03-31" }),
              qualification({
                title: "Forklift licence",
                kind: "licence",
                expires_on: "2026-01-31",
                expired: true,
              }),
              qualification({ title: "Electrical safety", status: "declared" }),
            ],
        gap: opts.allClear
          ? [
              {
                title: "First aid at work",
                kind: null,
                framework: null,
                min_level: null,
                standing: "met",
              },
            ]
          : [
              {
                title: "First aid at work",
                kind: null,
                framework: null,
                min_level: null,
                standing: "met",
              },
              {
                title: "Forklift licence",
                kind: "licence",
                framework: null,
                min_level: null,
                standing: "expired",
              },
              {
                title: "Electrical safety",
                kind: null,
                framework: null,
                min_level: null,
                standing: "declared_only",
              },
              {
                title: "Working at height",
                kind: null,
                framework: null,
                min_level: null,
                standing: "missing",
              },
            ],
      },
    }),
  );
}

test.describe("My mandatory training and certifications", () => {
  test("is the first thing on My profile", async ({ page }) => {
    await stubSignedIn(page);
    await page.goto("/me");
    const link = page.getByTestId("my-required-link");
    await expect(link).toHaveText("My mandatory training and certifications");
    await expect(link).toHaveAttribute("href", "/en-001/me/required");
  });

  test("lists training most urgent first, leaves out what does not apply, and links to act", async ({
    page,
  }) => {
    await stubRequired(page);
    await page.goto("/me/required");
    const items = page.getByTestId("required-learning").locator("li");
    await expect(items).toHaveCount(5);
    await expect(items.nth(0)).toContainText("Information governance");
    await expect(items.nth(0)).toContainText("▲ Expired");
    await expect(items.nth(1)).toContainText("Safeguarding");
    await expect(items.nth(2)).toContainText("Manual handling");
    await expect(items.nth(3)).toContainText("Conflict resolution");
    await expect(items.nth(3)).toContainText(
      "does not say which group you are in",
    );
    await expect(items.nth(4)).toContainText("✓ In date");
    await expect(page.getByTestId("required-learning")).not.toContainText(
      "Moving people",
    );
    await expect(page.getByTestId("required-learning-act")).toHaveCount(3);
    await expect(
      page.getByTestId("required-learning-act").first(),
    ).toHaveAttribute("href", `/en-001/workers/${WORKER.pid}#training`);
    await expect(page.getByTestId("required-all-clear")).toHaveCount(0);
    await page.screenshot({
      path: "test-results/my-required-desktop.png",
      fullPage: true,
    });
  });

  test("lists what the role asks for, with the day the held record expires and what to do", async ({
    page,
  }) => {
    await stubRequired(page);
    await page.goto("/me/required");
    const certs = page.getByTestId("required-certs").locator("li");
    await expect(certs).toHaveCount(4);
    await expect(certs.nth(0)).toContainText("Forklift licence");
    await expect(certs.nth(0)).toContainText("Expires on");
    await expect(certs.nth(0)).toContainText("Add the renewed one");
    await expect(certs.nth(1)).toContainText("Working at height");
    await expect(certs.nth(1)).toContainText("If you hold it, add it");
    await expect(certs.nth(2)).toContainText("Electrical safety");
    await expect(certs.nth(2)).toContainText("Does not expire");
    await expect(certs.nth(2)).toContainText("Tell HR where the proof is held");
    await expect(certs.nth(3)).toContainText("✓ Met");
    await expect(certs.nth(3).getByTestId("required-cert-act")).toHaveCount(0);
    await expect(page.getByTestId("required-cert-act").first()).toHaveAttribute(
      "href",
      "/en-001/me#my-qualifications",
    );
  });

  test("says so when nothing needs doing", async ({ page }) => {
    await stubRequired(page, { allClear: true });
    await page.goto("/me/required");
    await expect(page.getByTestId("required-all-clear")).toHaveText(
      "✓ Nothing needs doing now.",
    );
  });

  test("with no role recorded, says there is nothing to compare, never all met", async ({
    page,
  }) => {
    await stubRequired(page, { noRole: true });
    await page.goto("/me/required");
    await expect(page.getByTestId("required-no-role")).toBeVisible();
    await expect(page.getByTestId("required-all-clear")).toHaveCount(0);
  });

  test("a failed read says what to try and links to help", async ({ page }) => {
    await stubSignedIn(page);
    await page.route(`${W}/mandatory-learning`, (route) =>
      route.fulfill({ status: 500, body: "{}" }),
    );
    await page.route(`${W}/qualifications`, (route) =>
      route.fulfill({ status: 403, body: "{}" }),
    );
    await page.goto("/me/required");
    await expect(
      page.getByTestId("required-learning-error-next"),
    ).toContainText("Try again");
    await expect(page.getByTestId("required-certs-error-next")).toContainText(
      "ask your manager or HR",
    );
  });

  test("fits a phone, and reads right to left in Urdu", async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 812 });
    await stubRequired(page);
    await page.goto("/ur-001/me/required");
    await expect(page.locator("html")).toHaveAttribute("dir", "rtl");
    await expect(page.getByTestId("required-certs").locator("li")).toHaveCount(
      4,
    );
    const overflow = await page.evaluate(
      () => document.documentElement.scrollWidth - window.innerWidth,
    );
    expect(overflow).toBeLessThanOrEqual(0);
    await page.screenshot({
      path: "test-results/my-required-ur-phone.png",
      fullPage: true,
    });
  });
});
