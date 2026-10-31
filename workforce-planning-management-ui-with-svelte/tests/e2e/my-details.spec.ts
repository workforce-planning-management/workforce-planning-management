// "My details" (WPM-R124–R130, WPM-T215): contact details, time off and a leave request, flexible working, a
// resignation, equality monitoring and workplace health requirements on one screen. The API is stubbed per test;
// the spec checks what the page sends and what it refuses to show. `MY_DETAILS_SHOTS=<dir>` also writes screenshots.

import { expect, test, type Page } from "@playwright/test";

async function signIn(page: Page) {
  await page.context().addCookies([
    {
      name: "__Host-mxi_session",
      value: "my-details-spec-session",
      domain: "localhost",
      path: "/",
      httpOnly: true,
      secure: true,
      sameSite: "Lax",
    },
  ]);
}

type Sent = { method: string; url: string; body: unknown };

async function stub(
  page: Page,
  options: { equality?: boolean; health?: boolean; resignation?: unknown } = {},
) {
  const sent: Sent[] = [];
  let contact: Record<string, unknown> | null = null;
  let leave = [
    {
      pid: "l1",
      kind: "annual",
      start_on: "2999-01-10",
      end_on: "2999-01-12",
      days: 3,
      status: "requested",
      reason: null,
    },
    {
      pid: "l2",
      kind: "sick",
      start_on: "2026-02-02",
      end_on: "2026-02-03",
      days: 2,
      status: "approved",
      reason: null,
    },
  ];
  let flexible = [
    {
      pid: "f1",
      kind: "hours",
      requested_on: "2026-10-01",
      proposed_start: "2026-12-01",
      proposed_fte_percent: 80,
      status: "counter_proposed",
      decide_by: "2026-10-29",
      overdue: false,
      decided_on: null,
      decision_reason: null,
      decision_note: null,
      counter_note: "Four days a week from January",
      counter_fte_percent: 80,
      trial_until: null,
      can_appeal: false,
      hr_to_apply_fte_percent: null,
    },
    {
      pid: "f2",
      kind: "place_of_work",
      requested_on: "2026-08-01",
      proposed_start: "2026-09-01",
      proposed_fte_percent: null,
      status: "refused",
      decide_by: "2026-09-26",
      overdue: false,
      decided_on: "2026-09-20",
      decision_reason: "other",
      decision_note: null,
      counter_note: null,
      counter_fte_percent: null,
      trial_until: null,
      can_appeal: true,
      hr_to_apply_fte_percent: null,
    },
  ];
  let resignation: unknown = options.resignation ?? null;
  let declarations: Record<string, string> = {
    disability: "prefer_not_to_say",
  };
  const record = (route: import("@playwright/test").Route) => {
    const request = route.request();
    sent.push({
      method: request.method(),
      url: new URL(request.url()).pathname.replace("/api/proxy", ""),
      body: request.method() === "GET" ? null : request.postDataJSON(),
    });
  };
  await page.route("**/api/proxy/**", (route) =>
    route.fulfill({
      status: 404,
      body: JSON.stringify({
        description: "unstubbed: " + route.request().url(),
      }),
    }),
  );
  await page.route("**/api/proxy/me/organizations", (route) =>
    route.fulfill({
      json: [
        {
          pid: "m1",
          organization_ref: "organization:acme",
          worker_pid: "w1",
          employed: true,
        },
      ],
    }),
  );
  await page.route("**/api/proxy/me/organizations/scope", (route) =>
    route.fulfill({ json: ["organization:acme"] }),
  );
  await page.route("**/api/proxy/workers/w1/emergency-contacts", (route) =>
    route.fulfill({ json: [] }),
  );
  await page.route("**/api/proxy/me/contact-details", (route) => {
    record(route);
    const method = route.request().method();
    if (method === "PUT")
      contact = { ...route.request().postDataJSON(), on_behalf: false };
    if (method === "DELETE") {
      contact = null;
      return route.fulfill({ status: 204, body: "" });
    }
    return route.fulfill({ json: { contact_details: contact } });
  });
  await page.route("**/api/proxy/me/time-off**", (route) =>
    route.fulfill({
      json: {
        as_of: "2026-10-21",
        year: 2026,
        balances: [
          {
            kind: "annual",
            year: 2026,
            entitled: 25,
            used: 8,
            taken: 8,
            booked: 3,
            requested: 3,
            remaining: 17,
          },
        ],
        requests: leave,
        history: [
          { year: 2025, absences: [{ kind: "sick", absences: 2, days: 4 }] },
        ],
      },
    }),
  );
  await page.route("**/api/proxy/me/leave-requests", (route) => {
    record(route);
    return route.fulfill({ json: { pid: "l3" } });
  });
  await page.route("**/api/proxy/me/leave-requests/l1/cancel", (route) => {
    record(route);
    leave = leave.filter((r) => r.pid !== "l1");
    return route.fulfill({ json: {} });
  });
  await page.route("**/api/proxy/me/flexible-working", (route) => {
    if (route.request().method() === "POST") {
      record(route);
      return route.fulfill({ json: { pid: "f3" } });
    }
    return route.fulfill({ json: flexible });
  });
  await page.route("**/api/proxy/me/flexible-working/*/*", (route) => {
    record(route);
    flexible = flexible.filter(
      (r) => !route.request().url().includes(`/${r.pid}/`),
    );
    return route.fulfill({ json: {} });
  });
  await page.route("**/api/proxy/me/resignation", (route) => {
    if (route.request().method() === "POST") {
      record(route);
      resignation = {
        pid: "r1",
        status: "logged",
        logged_on: "2026-10-21",
        proposed_last_day: "2026-12-31",
        agreed_last_day: null,
        reason: "new_role",
        notice_calendar_days: 30,
      };
      return route.fulfill({ json: resignation as object });
    }
    return route.fulfill({ json: { resignation } });
  });
  await page.route("**/api/proxy/me/resignation/withdraw", (route) => {
    record(route);
    resignation = null;
    return route.fulfill({ json: {} });
  });
  await page.route("**/api/proxy/me/equality-monitoring**", (route) => {
    const request = route.request();
    if (request.method() === "GET") {
      return route.fulfill({
        json: options.equality
          ? {
              enabled: true,
              lawful_basis: "legitimate interests",
              small_group_floor: 10,
              categories: {
                disability: ["yes", "no", "prefer_not_to_say"],
                age_band: [
                  "under_30",
                  "30_to_49",
                  "50_plus",
                  "prefer_not_to_say",
                ],
              },
              declarations,
              notice: "This is voluntary. Only you can see your answers.",
            }
          : { enabled: false },
      });
    }
    record(route);
    if (request.method() === "PUT")
      declarations = {
        ...declarations,
        ...request.postDataJSON().declarations,
      };
    else if (request.url().endsWith("/disability"))
      delete declarations.disability;
    else declarations = {};
    return route.fulfill({ json: {} });
  });
  await page.route("**/api/proxy/me/health-requirements", (route) =>
    route.fulfill({
      json: options.health
        ? {
            enabled: true,
            requirements: [
              {
                requirement: {
                  pid: "h1",
                  name: "Tetanus",
                  description: "Booster every ten years",
                },
                standing: "due_soon",
                cleared: true,
                recorded_on: "2016-11-01",
                next_due: "2026-11-01",
                reminder: true,
              },
              {
                requirement: {
                  pid: "h2",
                  name: "Hepatitis B",
                  description: null,
                },
                standing: "not_recorded",
                cleared: false,
                recorded_on: null,
                next_due: null,
                reminder: false,
              },
            ],
            notice: "Only occupational health records these.",
          }
        : { enabled: false },
    }),
  );
  return sent;
}

test("contact details are replaced as a whole and can be removed", async ({
  page,
}) => {
  await signIn(page);
  const sent = await stub(page);
  await page.goto("/en-001/me/details");
  const panel = page.getByTestId("my-contact-details");
  await expect(panel).toBeVisible();
  await expect(page.getByTestId("contact-clear")).toHaveCount(0);
  await page.getByTestId("contact-city").fill("Leeds");
  await page.getByTestId("contact-phone_mobile").fill("+44 7700 900123");
  await page.getByTestId("contact-save").click();
  await expect(page.getByTestId("contact-saved")).toBeVisible();
  const put = sent.find((s) => s.method === "PUT");
  expect(put?.body).toMatchObject({
    city: "Leeds",
    phone_mobile: "+44 7700 900123",
    address_line1: "",
  });
  await page.getByTestId("contact-clear").click();
  await expect(page.getByTestId("contact-clear")).toHaveCount(0);
  await expect(page.getByTestId("contact-city")).toHaveValue("");
});

test("the address follows the country's order and labels, and a country the table lacks gets a free-text block", async ({
  page,
}) => {
  await signIn(page);
  const sent = await stub(page);
  await page.goto("/en-001/me/details");
  const address = page.getByTestId("contact-address");
  const order = async () =>
    address
      .locator("input")
      .evaluateAll((inputs) =>
        inputs.map((i) => i.getAttribute("data-testid")),
      );
  // No country: the neutral order and labels.
  await expect(address).toHaveAttribute("data-mode", "neutral");
  await expect(address.getByText("Postal code")).toBeVisible();
  // Japan: large to small, the postal code first, "Prefecture".
  await page.getByTestId("contact-country").fill("JP");
  await expect(address).toHaveAttribute("data-mode", "table");
  expect(await order()).toEqual([
    "contact-postcode",
    "contact-region",
    "contact-city",
    "contact-address_line1",
    "contact-address_line2",
  ]);
  await expect(address.getByText("Prefecture")).toBeVisible();
  // The United States: the city, the state and the ZIP code on one line, last.
  await page.getByTestId("contact-country").fill("us");
  expect(await order()).toEqual([
    "contact-address_line1",
    "contact-address_line2",
    "contact-city",
    "contact-region",
    "contact-postcode",
  ]);
  await expect(address.getByText("ZIP code")).toBeVisible();
  await expect(address.getByText("State", { exact: true })).toBeVisible();
  await page.getByTestId("contact-city").fill("Springfield");
  // A country the table lacks: the block, and the city already typed stays in view.
  await page.getByTestId("contact-country").fill("NP");
  await expect(address).toHaveAttribute("data-mode", "block");
  await expect(page.getByTestId("contact-address-order")).toContainText(
    "no address order for this country",
  );
  await expect(page.getByTestId("contact-city")).toHaveValue("Springfield");
  await expect(page.getByTestId("contact-postcode")).toHaveCount(0);
  await page
    .getByTestId("contact-address_text")
    .fill("Ward 4\nThamel\nKathmandu 44600");
  await page.getByTestId("contact-save").click();
  await expect(page.getByTestId("contact-saved")).toBeVisible();
  expect(sent.find((s) => s.method === "PUT")?.body).toMatchObject({
    country: "NP",
    city: "Springfield",
    address_text: "Ward 4\nThamel\nKathmandu 44600",
  });
  if (process.env.MY_DETAILS_SHOTS)
    await page.getByTestId("my-contact-details").screenshot({
      path: `${process.env.MY_DETAILS_SHOTS}/address-block.png`,
    });
});

test("time off: the balance, a cancellable request, and a request without a reason for sick leave", async ({
  page,
}) => {
  await signIn(page);
  const sent = await stub(page);
  await page.goto("/en-001/me/details");
  await expect(page.getByTestId("time-balances")).toContainText("17");
  await expect(page.getByTestId("time-requests")).toContainText("annual");
  await page.getByTestId("leave-cancel").click();
  await expect(page.getByTestId("leave-cancel")).toHaveCount(0);
  expect(sent.some((s) => s.url === "/me/leave-requests/l1/cancel")).toBe(true);
  await page.getByTestId("leave-kind").selectOption("sick");
  await expect(page.getByTestId("leave-reason")).toHaveCount(0);
  await page.getByTestId("leave-from").fill("2026-11-02");
  await page.getByTestId("leave-to").fill("2026-11-03");
  await page.getByTestId("leave-submit").click();
  await expect
    .poll(() => sent.some((s) => s.url === "/me/leave-requests"))
    .toBe(true);
  expect(sent.find((s) => s.url === "/me/leave-requests")?.body).toEqual({
    kind: "sick",
    start_on: "2026-11-02",
    end_on: "2026-11-03",
  });
});

test("flexible working: accept a counter-proposal, appeal a refusal, send a new request", async ({
  page,
}) => {
  await signIn(page);
  const sent = await stub(page);
  await page.goto("/en-001/me/details");
  await expect(page.getByTestId("flex-counter")).toContainText(
    "Four days a week",
  );
  await page.getByTestId("flex-accept").click();
  await expect(page.getByTestId("flex-counter")).toHaveCount(0);
  expect(
    sent.find((s) => s.url === "/me/flexible-working/f1/respond")?.body,
  ).toEqual({ response: "accept" });
  await page.getByTestId("flex-appeal").click();
  await page.getByTestId("flex-appeal-note").fill("New evidence");
  await page.getByTestId("flex-appeal-send").click();
  await expect
    .poll(() => sent.some((s) => s.url === "/me/flexible-working/f2/appeal"))
    .toBe(true);
  expect(
    sent.find((s) => s.url === "/me/flexible-working/f2/appeal")?.body,
  ).toEqual({ note: "New evidence" });
  await page.getByTestId("flex-kind").selectOption("days");
  await page.getByTestId("flex-start").fill("2027-01-04");
  await page.getByTestId("flex-fte").fill("60");
  await page.getByTestId("flex-submit").click();
  await expect
    .poll(() =>
      sent.some((s) => s.url === "/me/flexible-working" && s.method === "POST"),
    )
    .toBe(true);
  expect(
    sent.find((s) => s.url === "/me/flexible-working" && s.method === "POST")
      ?.body,
  ).toEqual({
    kind: "days",
    proposed_start: "2027-01-04",
    proposed_fte_percent: 60,
  });
});

test("a resignation needs a second click, and can be taken back", async ({
  page,
}) => {
  await signIn(page);
  const sent = await stub(page);
  await page.goto("/en-001/me/details");
  await page.getByTestId("resign-last-day").fill("2026-12-31");
  await page.getByTestId("resign-reason").selectOption("new_role");
  await page.getByTestId("resign-submit").click();
  await expect(page.getByTestId("resign-confirm-text")).toBeVisible();
  expect(
    sent.some((s) => s.url === "/me/resignation" && s.method === "POST"),
  ).toBe(false);
  await page.getByTestId("resign-submit").click();
  await expect(page.getByTestId("resign-status")).toContainText("31 Dec 2026");
  expect(sent.find((s) => s.url === "/me/resignation")?.body).toEqual({
    proposed_last_day: "2026-12-31",
    reason: "new_role",
  });
  await page.getByTestId("resign-withdraw").click();
  await expect(page.getByTestId("resign-status")).toHaveCount(0);
});

test("equality monitoring and health requirements say so when they are off", async ({
  page,
}) => {
  await signIn(page);
  await stub(page);
  await page.goto("/en-001/me/details");
  await expect(page.getByTestId("equality-off")).toBeVisible();
  await expect(page.getByTestId("health-off")).toBeVisible();
});

test("equality answers are voluntary, changed and taken back one at a time; health shows my own standing", async ({
  page,
}) => {
  await signIn(page);
  const sent = await stub(page, { equality: true, health: true });
  await page.goto("/en-001/me/details");
  await expect(page.getByTestId("equality-notice")).toContainText("voluntary");
  await page.getByTestId("equality-age_band").selectOption("50_plus");
  await page.getByTestId("equality-save").click();
  await expect.poll(() => sent.some((s) => s.method === "PUT")).toBe(true);
  expect(sent.find((s) => s.method === "PUT")?.body).toEqual({
    declarations: { disability: "prefer_not_to_say", age_band: "50_plus" },
  });
  await page.getByTestId("equality-take-back-disability").click();
  await expect
    .poll(() =>
      sent.some((s) => s.url === "/me/equality-monitoring/disability"),
    )
    .toBe(true);
  await expect(page.getByTestId("health-h1")).toContainText("Tetanus");
  await expect(page.getByTestId("health-reminder")).toBeVisible();
  await expect(page.getByTestId("health-h2")).toContainText("not cleared");
});

test("a person with no worker record is told there is nothing to show", async ({
  page,
}) => {
  await signIn(page);
  await stub(page);
  await page.route("**/api/proxy/me/organizations", (route) =>
    route.fulfill({ json: [] }),
  );
  await page.goto("/en-001/me/details");
  await expect(page.getByTestId("no-worker")).toBeVisible();
});

const shots = process.env.MY_DETAILS_SHOTS;
for (const [scheme, size] of [
  ["light", { width: 1280, height: 900 }],
  ["dark", { width: 1280, height: 900 }],
  ["light", { width: 390, height: 844 }],
  ["dark", { width: 390, height: 844 }],
] as const) {
  test(`screenshot ${scheme} ${size.width}`, async ({ page }) => {
    test.skip(!shots, "set MY_DETAILS_SHOTS=<dir> to write screenshots");
    await signIn(page);
    await page.setViewportSize(size);
    await page.emulateMedia({ colorScheme: scheme });
    await stub(page, { equality: true, health: true });
    await page.goto("/en-001/me/details");
    await expect(page.getByTestId("health-h1")).toBeVisible();
    await page.screenshot({
      path: `${shots}/my-details-${scheme}-${size.width}.png`,
      fullPage: true,
    });
  });
}
