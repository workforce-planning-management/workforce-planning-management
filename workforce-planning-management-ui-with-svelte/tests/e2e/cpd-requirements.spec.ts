// A CPD requirement can be corrected and withdrawn from the page (WPM-R139). The API is stubbed per test;
// the spec checks what the page sends (the unit unchanged, the new terms) and that a withdraw needs a second click.

import { expect, test, type Page } from "@playwright/test";

async function signIn(page: Page) {
  await page.context().addCookies([
    {
      name: "__Host-mxi_session",
      value: "cpd-requirements-spec-session",
      domain: "localhost",
      path: "/",
      httpOnly: true,
      secure: true,
      sameSite: "Lax",
    },
  ]);
}

test("a requirement is corrected with its unit unchanged, and withdrawn only after a confirmation", async ({
  page,
}) => {
  await signIn(page);
  let requirements = [
    {
      pid: "r1",
      name: "Annual hours",
      unit: "hours",
      required: 30,
      period_start: "2026-01-01",
      period_end: "2026-12-31",
      job_title: null,
    },
  ];
  const sent: { method: string; body: unknown }[] = [];
  await page.route("**/api/proxy/**", (route) =>
    route.fulfill({ status: 404, body: "unstubbed: " + route.request().url() }),
  );
  await page.route("**/api/proxy/workers", (route) =>
    route.fulfill({ json: [] }),
  );
  await page.route("**/api/proxy/cpd/overview", (route) =>
    route.fulfill({
      json: {
        derivation: "",
        headcount: 0,
        requirements: [],
        registrations: { expiring: 0, expired: 0 },
      },
    }),
  );
  await page.route("**/api/proxy/cpd-requirements", (route) =>
    route.fulfill({ json: requirements }),
  );
  await page.route("**/api/proxy/cpd-requirements/r1", (route) => {
    const request = route.request();
    sent.push({ method: request.method(), body: request.postDataJSON() });
    if (request.method() === "PUT") {
      const body = request.postDataJSON();
      requirements = [
        { ...requirements[0], name: body.name, required: body.required },
      ];
    } else {
      requirements = [];
    }
    return route.fulfill({ json: { pid: "r1" } });
  });

  await page.goto("/en-001/cpd");
  const row = page.getByTestId("cpd-requirement-r1");
  await expect(row).toContainText("Annual hours");

  await row.getByTestId("cpd-correct").click();
  await row.getByTestId("cpd-edit-name").fill("Annual hours (revised)");
  await row.getByTestId("cpd-edit-required").fill("40");
  await row.getByTestId("cpd-edit-save").click();
  await expect(row).toContainText("Annual hours (revised)");
  expect(sent[0]).toEqual({
    method: "PUT",
    body: {
      name: "Annual hours (revised)",
      unit: "hours",
      required: 40,
      period_start: "2026-01-01",
      period_end: "2026-12-31",
      job_title: null,
    },
  });

  await row.getByTestId("cpd-withdraw").click();
  expect(sent).toHaveLength(1); // the first click only asks
  await row.getByTestId("cpd-withdraw-confirm").click();
  await expect(page.getByTestId("cpd-requirements")).toHaveCount(0);
  expect(sent[1].method).toBe("DELETE");
});

// A count chooses its plural form in the page's language (WPM-T374): English "1 requirement", Russian
// "3 требования" and "5 требований", Arabic its own word for two.
test.describe("the requirement count's plural form", () => {
  const requirement = (n: number) => ({
    pid: `r${n}`,
    name: `Requirement ${n}`,
    unit: "hours",
    required: 30,
    period_start: "2026-01-01",
    period_end: "2026-12-31",
    job_title: null,
  });
  async function stubCount(page: Page, count: number) {
    await signIn(page);
    await page.route("**/api/proxy/**", (route) =>
      route.fulfill({
        status: 404,
        body: "unstubbed: " + route.request().url(),
      }),
    );
    await page.route("**/api/proxy/workers", (route) =>
      route.fulfill({ json: [] }),
    );
    await page.route("**/api/proxy/cpd/overview", (route) =>
      route.fulfill({
        json: {
          derivation: "",
          headcount: 0,
          requirements: [],
          registrations: { expiring: 0, expired: 0 },
        },
      }),
    );
    await page.route("**/api/proxy/cpd-requirements", (route) =>
      route.fulfill({
        json: Array.from({ length: count }, (_, i) => requirement(i + 1)),
      }),
    );
  }

  for (const [locale, count, text] of [
    ["en-001", 1, "1 requirement defined."],
    ["en-001", 3, "3 requirements defined."],
    ["ru-001", 3, "Задано 3 требования."],
    ["ru-001", 5, "Задано 5 требований."],
    ["ar-001", 2, "تم تحديد متطلبَين"],
  ] as const) {
    test(`${count} in ${locale}`, async ({ page }) => {
      await stubCount(page, count);
      await page.goto(`/${locale}/cpd`);
      await expect(page.getByText(text)).toBeVisible();
    });
  }
});
