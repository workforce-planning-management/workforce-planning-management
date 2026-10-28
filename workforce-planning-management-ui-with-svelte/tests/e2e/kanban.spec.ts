// My board (`/<locale>/kanban`): the signed-in person's development steps on a kanban board
// (`@lilydesignsystem/svelte-kanban-board`). The API is stubbed; screenshots go to test-results/.

import { expect, test, type Page } from "@playwright/test";

const WORKER = "44444444-4444-4444-8444-444444444444";
const ORG = "organization:33333333-3333-4333-8333-333333333333";
const SKILL_A = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";
const SKILL_B = "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb";
const SKILL_C = "cccccccc-cccc-4ccc-8ccc-cccccccccccc";

const item = (
  pid: string,
  skill: string,
  status: string,
  from: number,
  to: number,
) => ({
  pid,
  plan_pid: "plan1",
  skill_pid: skill,
  current_level: from,
  target_level: to,
  method: "course",
  course_ref: null,
  due_on: null,
  status,
});

const PLANS = (statuses: Record<string, string> = {}) => ({
  worker_pid: WORKER,
  plans: [
    {
      pid: "plan1",
      kind: "upskill",
      status: "active",
      target_job_title: null,
      items: [
        item("i1", SKILL_A, statuses.i1 ?? "planned", 2, 4),
        item("i2", SKILL_B, statuses.i2 ?? "in_progress", 1, 3),
        item("i3", SKILL_C, statuses.i3 ?? "achieved", 3, 4),
      ],
    },
  ],
});

async function signIn(page: Page, worker: string | null = WORKER) {
  await page.context().addCookies([
    {
      name: "__Host-mxi_session",
      value: "kanban-spec-session",
      domain: "localhost",
      path: "/",
      httpOnly: true,
      secure: true,
      sameSite: "Lax",
    },
  ]);
  await page.route("**/api/proxy/**", (route) =>
    route.fulfill({ status: 404, body: "unstubbed: " + route.request().url() }),
  );
  await page.route("**/api/proxy/me/organizations", (route) =>
    route.fulfill({
      json: [
        {
          pid: "m1",
          person_ref: "person:55555555-5555-4555-8555-555555555555",
          organization_ref: ORG,
          worker_pid: worker,
          employed: worker !== null,
          role: "member",
          starts_on: "2026-01-01",
          ends_on: null,
        },
      ],
    }),
  );
  await page.route("**/api/proxy/me/organizations/scope", (route) =>
    route.fulfill({ json: [ORG] }),
  );
  await page.route("**/api/proxy/skills", (route) =>
    route.fulfill({
      json: [
        {
          pid: SKILL_A,
          name: "Database administration",
          category: "technical",
          external_refs: [],
        },
        {
          pid: SKILL_B,
          name: "Incident command",
          category: "leadership",
          external_refs: [],
        },
        {
          pid: SKILL_C,
          name: "Stakeholder management",
          category: "leadership",
          external_refs: [],
        },
      ],
    }),
  );
}

test("shows each development step as a card in the column of its status, with its skill and levels", async ({
  page,
}) => {
  await signIn(page);
  await page.route(
    `**/api/proxy/workers/${WORKER}/development-plans`,
    (route) => route.fulfill({ json: PLANS() }),
  );
  await page.goto("/en-001/kanban");
  await expect(page.locator("h1")).toHaveText("My board");
  await expect(page.getByTestId("kanban-count")).toContainText("3");
  const board = page.getByTestId("kanban-board");
  await expect(board.getByRole("columnheader")).toHaveCount(4);
  await expect(board).toContainText("Database administration: 2 → 4");
  await expect(board).toContainText("Incident command: 1 → 3");
  await expect(board).toContainText("Stakeholder management: 3 → 4");
  // Columns carry the status words, in order.
  const headers = await board.getByRole("columnheader").allInnerTexts();
  expect(headers.map((h) => h.toLowerCase())).toEqual([
    expect.stringContaining("planned"),
    expect.stringContaining("in progress"),
    expect.stringContaining("achieved"),
    expect.stringContaining("abandoned"),
  ]);
});

test("moving a card by its menu saves the new status, and says that the declared level is unchanged", async ({
  page,
}) => {
  await signIn(page);
  let statuses: Record<string, string> = {};
  const saved: unknown[] = [];
  await page.route(
    `**/api/proxy/workers/${WORKER}/development-plans`,
    (route) => route.fulfill({ json: PLANS(statuses) }),
  );
  await page.route("**/api/proxy/development-plan-items/i1", (route) => {
    saved.push(route.request().postDataJSON());
    statuses = { i1: "achieved" };
    return route.fulfill({ json: {} });
  });
  await page.goto("/en-001/kanban");
  await page
    .getByRole("button", { name: "Move Database administration: 2 → 4" })
    .click();
  await page.getByRole("option", { name: /achieved/i }).click();
  await expect.poll(() => saved.length).toBe(1);
  expect(saved[0]).toEqual({ status: "achieved" });
  await expect(page.getByTestId("kanban-notice")).toContainText(
    "declared level of the skill is unchanged",
  );
});

test("a move the service refuses is put back, and the refusal is said in words", async ({
  page,
}) => {
  await signIn(page);
  await page.route(
    `**/api/proxy/workers/${WORKER}/development-plans`,
    (route) => route.fulfill({ json: PLANS() }),
  );
  await page.route("**/api/proxy/development-plan-items/i1", (route) =>
    route.fulfill({
      status: 422,
      json: { description: "the plan is not active" },
    }),
  );
  await page.goto("/en-001/kanban");
  await page
    .getByRole("button", { name: "Move Database administration: 2 → 4" })
    .click();
  await page.getByRole("option", { name: /achieved/i }).click();
  // The catalogue's words, not the service's.
  await expect(page.getByTestId("move-error")).toContainText(
    "That was not accepted",
  );
  await expect(page.getByTestId("move-error")).not.toContainText(
    "the plan is not active",
  );
  // The card is back where it was.
  await expect(page.getByTestId("kanban-board")).toContainText(
    "Database administration: 2 → 4",
  );
});

test("a person with no worker record, and a person with no plan, are told so", async ({
  page,
}) => {
  await signIn(page, null);
  await page.goto("/en-001/kanban");
  await expect(page.getByTestId("kanban-no-worker")).toBeVisible();
  const second = await page.context().newPage();
  await signIn(second);
  await second.route(
    `**/api/proxy/workers/${WORKER}/development-plans`,
    (route) => route.fulfill({ json: { worker_pid: WORKER, plans: [] } }),
  );
  await second.goto("/en-001/kanban");
  await expect(second.getByTestId("kanban-empty")).toBeVisible();
});

test("the board is in the reader's language and under the locale prefix", async ({
  page,
}) => {
  await signIn(page);
  await page.route(
    `**/api/proxy/workers/${WORKER}/development-plans`,
    (route) => route.fulfill({ json: PLANS() }),
  );
  await page.goto("/cy-001/kanban");
  await expect(page.locator("h1")).toHaveText("Fy mwrdd");
  await expect(page.getByTestId("kanban-count")).toContainText("Camau: 3");
});

for (const [scheme, size, label] of [
  ["light", { width: 1280, height: 900 }, "desktop"],
  ["dark", { width: 1280, height: 900 }, "desktop"],
  ["light", { width: 390, height: 844 }, "phone"],
  ["dark", { width: 390, height: 844 }, "phone"],
] as const) {
  test.describe(`${scheme} ${label}`, () => {
    test.use({ colorScheme: scheme, viewport: size });
    test("the board fits without the page scrolling sideways", async ({
      page,
    }) => {
      await page.addInitScript((theme) => {
        try {
          localStorage.setItem("mxi.wpm.theme", theme);
        } catch {
          /* default theme */
        }
      }, scheme);
      await signIn(page);
      await page.route(
        `**/api/proxy/workers/${WORKER}/development-plans`,
        (route) => route.fulfill({ json: PLANS() }),
      );
      await page.goto("/en-001/kanban");
      await expect(page.getByTestId("kanban-board")).toBeVisible();
      const overflow = await page.evaluate(
        () =>
          document.documentElement.scrollWidth -
          document.documentElement.clientWidth,
      );
      expect(overflow).toBeLessThanOrEqual(0);
      await page.screenshot({
        path: `test-results/kanban-${scheme}-${label}.png`,
        fullPage: true,
      });
    });
  });
}
