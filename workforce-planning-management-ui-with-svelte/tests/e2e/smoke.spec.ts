// Playwright smoke over a page.route-stubbed API (family pattern):
// the stubs mirror the service contract; any unstubbed /api/proxy
// call 404s loudly, so contract drift fails the suite.

import { expect, test, type Page } from "@playwright/test";

/**
 * Every page but /signin and /verify is gated on a session (WPM-T38),
 * so the smoke suite injects a fake `__Host-mxi_session` cookie
 * directly into the browser context before each test — the server
 * only checks the cookie's *presence*, never its validity
 * (`locals.sessionId` is set straight from `cookies.get`), so a
 * fabricated value is enough to pass the gate without a real
 * authentication-service round trip.
 */
async function signIn(page: Page) {
  await page.context().addCookies([
    {
      name: "__Host-mxi_session",
      value: "smoke-test-session",
      domain: "localhost",
      path: "/",
      httpOnly: true,
      secure: true,
      sameSite: "Lax",
    },
  ]);
}

const WORKER = {
  pid: "11111111-1111-4111-8111-111111111111",
  person_ref: "person:22222222-2222-4222-8222-222222222222",
  upstream_worker_ref: null,
  organization_ref: "organization:33333333-3333-4333-8333-333333333333",
  worker_number: "E-0001",
  display_name: "Test Worker 001",
  status: "active",
  employment_type: "permanent",
  fte_percent: 100,
  department: "engineering",
  job_title: "Engineer",
  manager_pid: null,
  salary_minor: 3600000,
  salary_currency: "GBP",
  hired_on: "2026-01-05",
  terminated_on: null,
};

const MASKED_WORKER = {
  ...WORKER,
  pid: "44444444-4444-4444-8444-444444444444",
  worker_number: "E-0002",
  display_name: "Masked Person",
  salary_minor: null,
  salary_currency: null,
};

/**
 * Choose a locale from the Lily locale picker.
 *
 * Three things make this more than a `selectOption`:
 *
 * 1. The picker renders a button plus a `ul` listbox, not a `<select>`.
 * 2. The theme picker on the same page also renders
 *    `li[role="option"]`, so the list has to be scoped — unscoped, the
 *    selector matches 58 elements here.
 * 3. The listbox is **still expanded after a pointer selection**
 *    (verified in a browser against Lily as of 2026-07-31), so
 *    clicking the button unconditionally would close it rather than
 *    open it. Open only when collapsed, which is correct either way.
 */
async function chooseLocale(page: Page, label: string) {
  const button = page.locator("nav.top .locale-picker-button");
  const list = page.locator("ul.locale-picker-list");
  if ((await button.getAttribute("aria-expanded")) !== "true") {
    await button.click();
  }
  await expect(list).toBeVisible();
  await list
    .locator('li[role="option"]')
    .filter({ hasText: label })
    .first()
    .click();
}

test.describe("sign-in gate (WPM-T38)", () => {
  // These tests run with NO session cookie — the opposite of every
  // other test in this file — so they get their own describe block
  // rather than relying on the file-level beforeEach.
  test("a signed-out visitor is redirected from a protected page to /signin", async ({
    page,
  }) => {
    await page.goto("/workers");
    await expect(page).toHaveURL(/\/signin$/);
  });

  test("a signed-out visitor sees the marketing splash at / instead of a redirect", async ({
    page,
  }) => {
    const response = await page.goto("/");
    expect(response?.status()).toBe(200);
    await expect(page).toHaveURL(/\/en-001$/);
    await expect(page.getByTestId("splash")).toBeVisible();
    await expect(page.getByTestId("splash")).toContainText(
      "People operations, all in one place",
    );
    await page.getByTestId("hero-cta").click();
    await expect(page).toHaveURL(/\/signin$/);
  });

  test("/tour stays reachable with no session and covers the app in depth", async ({
    page,
  }) => {
    const response = await page.goto("/tour");
    expect(response?.status()).toBe(200);
    await expect(page).toHaveURL(/\/tour$/);
    await expect(page.locator("h1")).toHaveText(
      "See how it all fits together",
    );
    await expect(page.locator("body")).toContainText(
      "Hiring & onboarding, start to finish",
    );
    await expect(page.locator("body")).toContainText(
      "Belong to more than one organization",
    );
    await page.locator('nav.top a[href="/en-001/signin"]').click();
    await expect(page).toHaveURL(/\/signin$/);
  });

  test("/signin itself stays reachable with no session", async ({ page }) => {
    const response = await page.goto("/signin");
    expect(response?.status()).toBe(200);
    await expect(page).toHaveURL(/\/signin$/);
  });
});

// Every test below exercises a gated page, so it needs a session
// already present — the file-level beforeEach lived here before
// WPM-T38's gate existed; it is now scoped to this describe so the
// "sign-in gate" tests above (deliberately signed OUT) don't inherit it.
test.describe("signed-in smoke coverage", () => {
  test.beforeEach(async ({ page }) => {
    await signIn(page);
    // Unstubbed API calls fail loudly.
    await page.route("**/api/proxy/**", (route) =>
      route.fulfill({
        status: 404,
        body: "unstubbed: " + route.request().url(),
      }),
    );
    await page.route("**/api/proxy/workers", (route) =>
      route.fulfill({ json: [WORKER, MASKED_WORKER] }),
    );
    await page.route("**/api/proxy/workers?status=active", (route) =>
      route.fulfill({ json: [WORKER, MASKED_WORKER] }),
    );
    await page.route("**/api/proxy/requisitions?status=open", (route) =>
      route.fulfill({
        json: [
          {
            pid: "55555555-5555-4555-8555-555555555555",
            organization_ref: WORKER.organization_ref,
            department: "engineering",
            job_title: "Platform Engineer",
            headcount: 1,
            salary_min_minor: null,
            salary_max_minor: null,
            salary_currency: null,
            status: "open",
            opened_on: "2026-06-01",
          },
        ],
      }),
    );
    await page.route("**/api/proxy/succession-plans/gaps", (route) =>
      route.fulfill({ json: { gaps: [] } }),
    );
    await page.route("**/api/proxy/me/organizations", (route) =>
      route.fulfill({
        json: [
          {
            pid: "88888888-8888-4888-8888-888888888888",
            person_ref: WORKER.person_ref,
            organization_ref: WORKER.organization_ref,
            worker_pid: WORKER.pid,
            employed: true,
            role: "member",
            starts_on: "2026-01-05",
            ends_on: null,
          },
          {
            pid: "99999999-9999-4999-9999-999999999999",
            person_ref: WORKER.person_ref,
            organization_ref: "organization:44444444-4444-4444-8444-444444444444",
            worker_pid: null,
            employed: false,
            role: "hr_admin",
            starts_on: "2026-02-01",
            ends_on: null,
          },
        ],
      }),
    );
    // Read scope, expanded through org confederation server-side — in
    // this fixture it equals the two direct memberships above (no
    // confederation edges), so `/org-chart` and `/benchmarks` render
    // the same two sections the dashboard's membership list does.
    await page.route("**/api/proxy/me/organizations/scope", (route) =>
      route.fulfill({
        json: [
          WORKER.organization_ref,
          "organization:44444444-4444-4444-8444-444444444444",
        ],
      }),
    );
  });

  test("dashboard renders live tiles from the stubbed API", async ({
    page,
  }) => {
    await page.goto("/");
    await expect(page.getByTestId("tile-active")).toContainText("2");
    await expect(page.getByTestId("tile-open")).toContainText("1");
    await expect(page.getByTestId("tile-gaps")).toContainText("0");
  });

  test("dashboard lists every organization membership at once, no switcher", async ({
    page,
  }) => {
    await page.goto("/");
    const section = page.getByTestId("my-organizations");
    await expect(section).toContainText(WORKER.organization_ref);
    await expect(section).toContainText("member");
    await expect(section).toContainText("Employed here");
    await expect(section).toContainText(
      "organization:44444444-4444-4444-8444-444444444444",
    );
    await expect(section).toContainText("hr_admin");
    await expect(section).toContainText("Staff access");
  });

  // The CEO dashboard must fit an iPad (9th gen): 2160 × 1620 device pixels,
  // i.e. 1080 × 810 CSS pixels at 2×, with no scrolling — and a literal
  // 2160 × 1620 viewport too. Worst-case text is used so a tile that would
  // overflow actually does; tiles clip silently (`overflow: hidden`), so the
  // check is on each tile's own content, not just the page.
  for (const screen of [
    { name: "iPad 9th gen @2x (1080×810 CSS)", width: 1080, height: 810, scale: 2 },
    { name: "2160×1620 @1x", width: 2160, height: 1620, scale: 1 },
  ]) {
    test(`CEO dashboard fits ${screen.name} without scrolling or clipping`, async ({
      browser,
    }) => {
      const context = await browser.newContext({
        viewport: { width: screen.width, height: screen.height },
        deviceScaleFactor: screen.scale,
        locale: "en",
      });
      const page = await context.newPage();
      await signIn(page);
      await page.route("**/api/proxy/**", (route) =>
        route.fulfill({ status: 404, body: "unstubbed" }),
      );
      await page.route("**/api/proxy/me/organizations**", (route) =>
        route.fulfill({ json: [] }),
      );
      let n = 0;
      const metricsFrom: string[] = [];
      await page.route("**/api/proxy/workforce-intelligence/metrics**", (route) => {
        metricsFrom.push(new URL(route.request().url()).searchParams.get("from") ?? "");
        return route.fulfill({
          json: {
            period: { from: "2025-10-05", to: "2026-10-05" },
            definitions: {},
            headcount: { opening: 1180 + (n++ % 7) * 9, closing: 12345, opening_date: "2025-10-04" },
            starters: 210,
            leavers: 1234,
            turnover_rate: 0.1234,
            span_of_control: { managers: 1234, mean: 12.34, max: 40 },
            time_to_fill: { requisitions: 99, mean_days: 61.2, median_days: 58.7 },
          },
        });
      });

      const long =
        "Break leavers down by department and tenure, and compare with pulse-survey and exit feedback across every organization";
      await page.route("**/api/proxy/workforce-intelligence/insights**", (route) =>
        route.fulfill({
          json: {
            period: { from: "2025-10-05", to: "2026-10-05" },
            thresholds: {},
            insights: ["a", "b", "c", "d", "e"].map((c) => ({
              code: `unknown_${c}`,
              severity: "attention",
              observation: long,
              suggestion: long,
              params: {},
            })),
          },
        }),
      );
      await page.route("**/api/proxy/succession-plans/gaps", (route) =>
        route.fulfill({ json: { gaps: [{}, {}, {}] } }),
      );
      await page.route("**/api/proxy/requisitions?status=open", (route) =>
        route.fulfill({ json: [{}, {}, {}, {}, {}, {}] }),
      );
      await page.goto("/ceo");
      await expect(page.getByTestId("kpi-headcount")).toContainText("12345");
      await expect(page.getByTestId("trend-chart")).toBeVisible();
      await expect(page.getByTestId("ceo-insights")).toContainText("Attention");

      const fit = await page.evaluate(() => {
        const doc = document.documentElement;
        const tiles = [...document.querySelectorAll<HTMLElement>(".ceo .cx-tile")].map((el) => {
          const box = el.getBoundingClientRect();
          return {
            name: el.dataset.testid ?? el.className,
            inView:
              box.left >= 0 && box.top >= 0 &&
              box.right <= window.innerWidth + 0.5 && box.bottom <= window.innerHeight + 0.5,
            clipsY: el.scrollHeight - el.clientHeight,
            clipsX: el.scrollWidth - el.clientWidth,
          };
        });
        return {
          pageY: doc.scrollHeight - window.innerHeight,
          pageX: doc.scrollWidth - window.innerWidth,
          bodyY: document.body.scrollHeight - window.innerHeight,
          tiles,
        };
      });
      expect(fit.pageY, "page scrolls vertically").toBeLessThanOrEqual(0);
      expect(fit.pageX, "page scrolls horizontally").toBeLessThanOrEqual(0);
      expect(fit.bodyY, "body scrolls").toBeLessThanOrEqual(0);
      expect(fit.tiles).toHaveLength(6);
      for (const tile of fit.tiles) {
        expect(tile.inView, `${tile.name} is inside the screen`).toBe(true);
        // Tiles clip silently; a tile whose content is taller or wider than
        // its box is clipping something the CEO needs to read.
        expect(tile.clipsY, `${tile.name} clips vertically`).toBeLessThanOrEqual(1);
        expect(tile.clipsX, `${tile.name} clips horizontally`).toBeLessThanOrEqual(1);
      }
      await page.screenshot({
        path: `test-results/ceo-${screen.width}x${screen.height}.png`,
      });

      // Each tile drills down to the page behind it.
      const hrefOf = (id: string) => page.getByTestId(id).getAttribute("href");
      expect(await hrefOf("kpi-headcount")).toMatch(/\/en-001\/metrics$/);
      expect(await hrefOf("kpi-open")).toMatch(/\/en-001\/requisitions$/);
      expect(await hrefOf("kpi-gaps")).toMatch(/\/en-001\/development$/);

      // The period control reloads the period-based figures (and is in the URL).
      await expect(page.getByTestId("range-12m")).toHaveAttribute("aria-pressed", "true");
      await page.getByTestId("range-90d").click();
      await expect(page.getByTestId("range-90d")).toHaveAttribute("aria-pressed", "true");
      await expect(page).toHaveURL(/range=90d/);
      const ninetyDaysBack = new Date(Date.now() - 89 * 86_400_000).toISOString().slice(0, 10);
      await expect.poll(() => metricsFrom.includes(ninetyDaysBack)).toBe(true);
      await context.close();
    });
  }

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
        status: "live",
        author: null,
      },
    ];
    await page.route("**/api/proxy/announcements**", (route) => {
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

    await page.getByText("Post an announcement").first().click();
    await page.getByTestId("announcement-title").fill("Town hall Friday");
    await page.getByTestId("announcement-body").fill("All hands at 3pm.");
    await page.getByTestId("announcement-post").click();
    await expect(list).toContainText("Town hall Friday");
  });

  test("on-call rota shows who is on call, why, and takes a swap", async ({
    page,
  }) => {
    const rotaPid = "aaaaaaaa-0000-4000-8000-00000000a001";
    const swaps: Array<Record<string, unknown>> = [];
    const view = () => ({
      pid: rotaPid,
      organization_ref: WORKER.organization_ref,
      name: "Platform on-call",
      description: null,
      period_days: 7,
      starts_on: "2026-10-05",
      members: [
        { position: 1, worker_pid: WORKER.pid, name: WORKER.display_name, job_title: "Engineer" },
        { position: 2, worker_pid: MASKED_WORKER.pid, name: MASKED_WORKER.display_name, job_title: "Engineer" },
      ],
      overrides: swaps,
      window: { from: "2026-10-05", to: "2026-11-01" },
      runs: [
        { from: "2026-10-05", to: "2026-10-11", worker_pid: WORKER.pid, worker_name: WORKER.display_name, source: "rotation" },
        { from: "2026-10-12", to: "2026-10-18", worker_pid: WORKER.pid, worker_name: WORKER.display_name, source: "skipped" },
        { from: "2026-10-19", to: "2026-10-25", worker_pid: null, worker_name: null, source: null },
      ],
      load: [{ worker_pid: WORKER.pid, name: WORKER.display_name, days: 14 }],
      on_call_today: { worker_pid: WORKER.pid, name: WORKER.display_name },
    });
    await page.route("**/api/proxy/rotas", (route) =>
      route.fulfill({
        json: [
          {
            pid: rotaPid,
            organization_ref: WORKER.organization_ref,
            name: "Platform on-call",
            description: null,
            period_days: 7,
            starts_on: "2026-10-05",
            members: 2,
            on_call_today: { worker_pid: WORKER.pid, name: WORKER.display_name },
          },
        ],
      }),
    );
    await page.route(`**/api/proxy/rotas/${rotaPid}`, (route) =>
      route.fulfill({ json: view() }),
    );
    await page.route(`**/api/proxy/rotas/${rotaPid}/overrides`, (route) => {
      swaps.push({
        pid: "s1",
        worker_name: MASKED_WORKER.display_name,
        note: null,
        ...route.request().postDataJSON(),
      });
      return route.fulfill({ json: { pid: "s1" } });
    });

    const asked: Array<Record<string, unknown>> = [];
    await page.route(`**/api/proxy/rotas/${rotaPid}/swap-requests`, (route) => {
      if (route.request().method() === "POST") {
        asked.push({
          pid: "q1",
          rota_pid: rotaPid,
          rota_name: "Platform on-call",
          requester_name: WORKER.display_name,
          taker_name: MASKED_WORKER.display_name,
          note: null,
          status: "requested",
          ...route.request().postDataJSON(),
        });
        return route.fulfill({ json: { pid: "q1" } });
      }
      return route.fulfill({ json: asked });
    });

    await page.goto("/rota");
    await expect(page.getByTestId("on-call-now")).toContainText(
      WORKER.display_name,
    );
    const runs = page.getByTestId("rota-runs");
    await expect(runs).toContainText("Rotation");
    await expect(runs).toContainText("Covering for someone away");
    // A stretch nobody can take is shown plainly, not guessed.
    await expect(runs).toContainText("Nobody is available");

    await page.getByTestId("swap-worker").selectOption(MASKED_WORKER.pid);
    const swapForm = page.getByTestId("swap-form");
    await swapForm.getByLabel("From").fill("2026-10-05");
    await swapForm.getByLabel("To").fill("2026-10-06");
    await page.getByTestId("swap-add").click();
    await expect(page.getByTestId("rota-swaps")).toContainText(
      MASKED_WORKER.display_name,
    );

    // Ask a colleague to take your on-call days.
    await expect(page.getByTestId("rota-requests")).toContainText(
      "No open swap requests.",
    );
    await page.getByTestId("ask-requester").selectOption(WORKER.pid);
    await page.getByTestId("ask-taker").selectOption(MASKED_WORKER.pid);
    const form = page.getByTestId("swap-request-form");
    await form.getByLabel("From").fill("2026-10-05");
    await form.getByLabel("To").fill("2026-10-11");
    await page.getByTestId("swap-request-add").click();
    await expect(page.getByTestId("rota-requests")).toContainText("requested");
    await expect(page.getByTestId("rota-requests")).toContainText(
      MASKED_WORKER.display_name,
    );
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

  test("directory searches employed workers and shows no pay", async ({
    page,
  }) => {
    const rows = [
      {
        pid: WORKER.pid,
        display_name: WORKER.display_name,
        job_title: WORKER.job_title,
        department: WORKER.department,
        location: "Leeds",
        organization_ref: WORKER.organization_ref,
        manager_name: null,
        away_today: true,
        covered_by: "Kim Lee",
        on_call: ["Platform on-call"],
      },
    ];
    const seen: string[] = [];
    await page.route(
      (url) => url.pathname === "/api/proxy/directory",
      (route) => {
        seen.push(new URL(route.request().url()).searchParams.get("q") ?? "");
        return route.fulfill({
          json: seen.at(-1) === "nobody" ? [] : rows,
        });
      },
    );
    await page.goto("/directory");
    const table = page.getByTestId("directory-table");
    await expect(table).toContainText(WORKER.display_name);
    await expect(table).toContainText("Leeds");
    await expect(table).not.toContainText("£");
    await expect(page.getByTestId("on-call")).toHaveText(
      "On call: Platform on-call",
    );
    await expect(page.getByTestId("away")).toHaveText("Away today");
    await expect(page.getByTestId("covered-by")).toContainText("Covered by Kim Lee");
    await page.getByTestId("directory-search").fill("nobody");
    await expect(page.getByTestId("directory-none")).toBeVisible();
    expect(seen).toContain("nobody");
  });

  test("org chart renders one section per organization membership, no switcher", async ({
    page,
  }) => {
    const secondOrg = "organization:44444444-4444-4444-8444-444444444444";
    await page.route(
      (url) =>
        url.pathname === "/api/proxy/org-chart" &&
        url.searchParams.get("organization") === WORKER.organization_ref,
      (route) =>
        route.fulfill({
          json: [
            {
              pid: WORKER.pid,
              display_name: WORKER.display_name,
              job_title: WORKER.job_title,
              department: WORKER.department,
              reports: [],
            },
          ],
        }),
    );
    await page.route(
      (url) =>
        url.pathname === "/api/proxy/org-chart" &&
        url.searchParams.get("organization") === secondOrg,
      (route) => route.fulfill({ json: [] }),
    );
    await page.goto("/org-chart");
    const sections = page.getByTestId("org-chart");
    await expect(sections).toHaveCount(2);
    await expect(sections.nth(0)).toContainText(WORKER.organization_ref);
    await expect(sections.nth(0)).toContainText(WORKER.display_name);
    await expect(sections.nth(1)).toContainText(secondOrg);
  });

  test("org confederation expands org chart to a descendant with no direct membership", async ({
    page,
  }) => {
    // A membership in a parent confederation reads as membership in
    // every descendant too — the backend's `scope` already includes
    // this third org even though it never appears in `/me/organizations`
    // (the dashboard's literal-grants list, unaffected by confederation).
    const confederatedChild =
      "organization:55555555-5555-4555-8555-555555555555";
    await page.route("**/api/proxy/me/organizations/scope", (route) =>
      route.fulfill({
        json: [
          WORKER.organization_ref,
          "organization:44444444-4444-4444-8444-444444444444",
          confederatedChild,
        ],
      }),
    );
    await page.route(
      (url) => url.pathname === "/api/proxy/org-chart",
      (route) => route.fulfill({ json: [] }),
    );
    await page.goto("/org-chart");
    await expect(page.getByTestId("org-chart")).toHaveCount(3);
    await expect(page.locator("body")).toContainText(confederatedChild);

    // The dashboard's membership list is unaffected: still exactly the
    // two literal grants, no confederation-only descendant.
    await page.goto("/");
    const membershipSection = page.getByTestId("my-organizations");
    await expect(membershipSection).not.toContainText(confederatedChild);
  });

  test("benchmark comparison renders one section per organization membership", async ({
    page,
  }) => {
    const secondOrg = "organization:44444444-4444-4444-8444-444444444444";
    await page.route("**/api/proxy/benchmarks", (route) =>
      route.fulfill({ json: [] }),
    );
    await page.route(
      (url) =>
        url.pathname === "/api/proxy/benchmarks/comparison" &&
        url.searchParams.get("organization") === WORKER.organization_ref,
      (route) =>
        route.fulfill({
          json: {
            organization: WORKER.organization_ref,
            rows: [
              {
                worker_pid: WORKER.pid,
                job_title: WORKER.job_title,
                department: WORKER.department,
                benchmark_pid: null,
                flag: null,
              },
            ],
          },
        }),
    );
    await page.route(
      (url) =>
        url.pathname === "/api/proxy/benchmarks/comparison" &&
        url.searchParams.get("organization") === secondOrg,
      (route) => route.fulfill({ json: { organization: secondOrg, rows: [] } }),
    );
    await page.goto("/benchmarks");
    const tables = page.getByTestId("comparison");
    await expect(tables).toHaveCount(2);
    await expect(page.locator("body")).toContainText(WORKER.organization_ref);
    await expect(page.locator("body")).toContainText(secondOrg);
    await expect(tables.nth(0)).toContainText(WORKER.pid.slice(0, 8));
  });

  test("worker list shows money for visible salaries and Hidden for masked", async ({
    page,
  }) => {
    await page.goto("/workers");
    const table = page.getByTestId("worker-table");
    await expect(table).toContainText("E-0001");
    await expect(table).toContainText("£36,000.00");
    await expect(table).toContainText("Hidden");
  });

  test("payroll run detail drives the lifecycle actions", async ({ page }) => {
    const run = {
      pid: "66666666-6666-4666-8666-666666666666",
      organization_ref: WORKER.organization_ref,
      period_start: "2026-07-01",
      period_end: "2026-07-31",
      status: "calculated",
    };
    await page.route(`**/api/proxy/payroll-runs/${run.pid}`, (route) =>
      route.fulfill({ json: run }),
    );
    await page.route(`**/api/proxy/payroll-runs/${run.pid}/payslips`, (route) =>
      route.fulfill({
        json: [
          {
            pid: "77777777-7777-4777-8777-777777777777",
            run_pid: run.pid,
            worker_pid: WORKER.pid,
            currency: "GBP",
            gross_minor: 300000,
            deductions: [{ label: "tax", amount_minor: 39050 }],
            net_minor: 260950,
          },
        ],
      }),
    );
    await page.goto(`/payroll/${run.pid}`);
    await expect(page.getByTestId("run-status")).toHaveText("calculated");
    await expect(page.getByTestId("action-approve")).toBeVisible();
    const payslips = page.getByTestId("payslips");
    await expect(payslips).toContainText("£3,000.00");
    await expect(payslips).toContainText("£2,609.50");
    await expect(payslips).toContainText("tax");
  });

  test("locale switcher retranslates the chrome (and ar flips direction)", async ({
    page,
  }) => {
    await page.goto("/workers");
    // An unprefixed visit lands on the default locale's content route.
    await expect(page).toHaveURL(/\/en-001\/workers$/);
    await expect(page.locator("h1")).toHaveText("Workers");
    await chooseLocale(page, "Deutsch");
    await expect(page).toHaveURL(/\/de-001\/workers$/);
    await expect(page.locator("h1")).toHaveText("Arbeitskräfte");
    await chooseLocale(page, "العربية");
    await expect(page).toHaveURL(/\/ar-001\/workers$/);
    await expect(page.locator("html")).toHaveAttribute("dir", "rtl");
  });

  test("the CMS shell is served per locale, in that locale's CMS language", async ({
    page,
  }) => {
    await page.route("https://unpkg.com/**", (route) =>
      route.fulfill({ contentType: "text/javascript", body: "" }),
    );
    await page.goto("/de/admin/");
    await expect(page).toHaveURL(/\/de-001\/admin\/?$/);
    await expect(page.locator("html")).toHaveAttribute("lang", "de-001");
    expect(
      await page.evaluate(
        () => JSON.parse(localStorage.getItem("sveltia-cms.prefs") ?? "{}").locale,
      ),
    ).toBe("de");
    const config = await page.request.get("/admin/config.yml");
    expect(config.status()).toBe(200);
  });

  test("a bare language alias redirects to its -001 content route", async ({
    page,
  }) => {
    await page.goto("/cy/workers");
    await expect(page).toHaveURL(/\/cy-001\/workers$/);
    await expect(page.locator("html")).toHaveAttribute("lang", "cy-001");
  });

  test("hamburger menu opens the left nav drawer with every section link", async ({
    page,
  }) => {
    await page.goto("/");
    const drawer = page.locator(".drawer");
    await expect(drawer).toBeHidden();
    await page.getByRole("button", { name: "Menu" }).click();
    await expect(drawer).toBeVisible();
    await expect(drawer).toContainText("Workers");
    await expect(drawer).toContainText("Org chart");
    await expect(drawer).toContainText("Benchmarks");
    await drawer.getByRole("link", { name: "Workers" }).click();
    await expect(page).toHaveURL(/\/workers$/);
    await expect(drawer).toBeHidden();
  });

  test("requisition board renders SVAR Kanban columns and cards", async ({
    page,
  }) => {
    await page.route("**/api/proxy/requisitions", (route) =>
      route.fulfill({
        json: [
          {
            pid: "99999999-9999-4999-8999-999999999999",
            organization_ref: WORKER.organization_ref,
            department: "engineering",
            job_title: "Platform Engineer",
            headcount: 2,
            salary_min_minor: 3000000,
            salary_max_minor: 5000000,
            salary_currency: "GBP",
            status: "interviewing",
            opened_on: "2026-06-01",
          },
        ],
      }),
    );
    await page.goto("/requisitions");
    const board = page.getByTestId("requisition-board");
    await expect(board).toContainText("interviewing");
    await expect(board).toContainText("Platform Engineer");
    await expect(board).toContainText("£30,000.00");
  });

  test("learning area renders skills matrix, analytics, and path progress", async ({
    page,
  }) => {
    await page.route("**/api/proxy/learning/skills-matrix", (route) =>
      route.fulfill({
        json: {
          as_of: "2026-07-20T00:00:00Z",
          note: "coverage over declared proficiencies only",
          matrix: [
            {
              department: "engineering",
              skill: "Rust",
              workers: 2,
              average_proficiency: 3.5,
              below_target: 1,
            },
          ],
          gaps: [
            {
              worker_pid: "e2",
              department: "engineering",
              skill: "Rust",
              proficiency: 2,
              target: 4,
            },
          ],
        },
      }),
    );
    await page.route("**/api/proxy/learning/training-analytics", (route) =>
      route.fulfill({
        json: {
          as_of: "2026-07-20",
          horizon: "2026-10-18",
          note: "completion rate = completed / non-failed",
          departments: [
            {
              department: "engineering",
              by_status: { completed: 1 },
              completion_rate: { numerator: 1, denominator: 1, value: 1 },
              certs_expiring: 0,
            },
          ],
        },
      }),
    );
    await page.route("**/api/proxy/learning-paths", (route) =>
      route.fulfill({
        json: [
          { pid: "path1", name: "Backend basics", summary: null, steps: 2 },
        ],
      }),
    );
    await page.route("**/api/proxy/learning-paths/path1/progress", (route) =>
      route.fulfill({
        json: {
          as_of: "2026-07-20T00:00:00Z",
          path: { pid: "path1", name: "Backend basics" },
          steps: [{ course_ref: "course:a", title: "Intro", position: 0 }],
          derivation:
            "a step is complete iff a completed training enrolment matches",
          members: [
            {
              worker_pid: "e2",
              display_name: "Sam Mentee",
              completed_steps: 1,
              total_steps: 2,
            },
          ],
        },
      }),
    );
    await page.goto("/learning");
    await expect(
      page.getByTestId("skills-matrix").getByText("Rust"),
    ).toBeVisible();
    await expect(
      page.getByTestId("skills-gaps").getByText(/Rust in engineering/),
    ).toBeVisible();
    await expect(
      page.getByTestId("training-analytics").getByText("100% (1/1)"),
    ).toBeVisible();
    await expect(
      page.getByTestId("path-progress").getByText("Sam Mentee"),
    ).toBeVisible();
  });

  test("mentorship area renders load, unmatched, and stale", async ({
    page,
  }) => {
    await page.route("**/api/proxy/learning/mentorship-overview**", (route) =>
      route.fulfill({
        json: {
          as_of: "2026-07-20",
          active_pairings: 1,
          mentor_load: [
            { mentor_pid: "e1", mentor: "Ada Mentor", active_mentees: 1 },
          ],
          unmatched_workers: [
            { pid: "e3", display_name: "Solo Dev", department: "engineering" },
          ],
          stale_days: 30,
          stale_mentorships: [
            {
              pid: "m1",
              mentor: "Ada Mentor",
              mentee: "Sam Mentee",
              last_session: "2026-05-01",
            },
          ],
        },
      }),
    );
    await page.goto("/mentorship");
    await expect(
      page.getByTestId("mentor-load").getByText("Ada Mentor"),
    ).toBeVisible();
    await expect(
      page.getByTestId("unmatched").getByText("Solo Dev"),
    ).toBeVisible();
    await expect(
      page.getByTestId("stale-mentorships").getByText("2026-05-01"),
    ).toBeVisible();
  });

  test("wellbeing area renders rules and aggregate-only uptake", async ({
    page,
  }) => {
    await page.route("**/api/proxy/wellbeing-entitlements", (route) =>
      route.fulfill({
        json: [
          {
            pid: "w1",
            name: "Seasonal flu vaccination",
            kind: "health",
            benefit_plan_pid: null,
            description: "Free NHS flu jab for frontline staff.",
            info_url: null,
            min_age: null,
            max_age: null,
            departments: ["engineering"],
            job_titles: [],
            doses: 2,
            active_from: null,
            active_until: null,
          },
          {
            pid: "w2",
            name: "Cycle-to-work scheme",
            kind: "benefit",
            benefit_plan_pid: "bp1",
            description: "Save on a bike through salary sacrifice.",
            info_url: null,
            min_age: null,
            max_age: null,
            departments: [],
            job_titles: [],
            doses: 1,
            active_from: null,
            active_until: null,
          },
        ],
      }),
    );
    await page.route("**/api/proxy/wellbeing/uptake", (route) =>
      route.fulfill({
        json: {
          as_of: "2026-07-24T00:00:00Z",
          derivation:
            "uptake = (booked + done) / all acknowledgements; counts only",
          entitlements: [
            {
              entitlement_pid: "w1",
              name: "Seasonal flu vaccination",
              kind: "health",
              by_response: { booked: 3, done: 1, declined: 1, dismissed: 0 },
              uptake_rate: { numerator: 4, denominator: 5, value: 0.8 },
              enrolment_conversion: null,
            },
            {
              entitlement_pid: "w2",
              name: "Cycle-to-work scheme",
              kind: "benefit",
              by_response: { booked: 0, done: 2, declined: 0, dismissed: 2 },
              uptake_rate: { numerator: 2, denominator: 4, value: 0.5 },
              enrolment_conversion: {
                numerator: 2,
                denominator: 4,
                value: 0.5,
              },
            },
          ],
        },
      }),
    );
    await page.route("**/api/proxy/pulse-surveys", (route) =>
      route.fulfill({
        json: [
          {
            pid: "s1",
            name: "July pulse",
            question: "How are you doing this week?",
            active_from: null,
            active_until: null,
            open: true,
          },
        ],
      }),
    );
    await page.route("**/api/proxy/pulse-surveys/s1/results", (route) =>
      route.fulfill({
        json: {
          as_of: "2026-07-25T00:00:00Z",
          survey: {
            pid: "s1",
            name: "July pulse",
            question: "How are you doing this week?",
          },
          overall: {
            suppressed: false,
            count: 6,
            distribution: [1, 1, 1, 1, 2],
            mean: 3.5,
          },
          departments: [
            {
              department: "engineering",
              suppressed: false,
              count: 5,
              distribution: [1, 1, 1, 1, 1],
              mean: 3.0,
            },
            { department: "finance", suppressed: true },
          ],
          derivation:
            "anonymous by construction; cells under 5 responses are suppressed",
        },
      }),
    );
    await page.goto("/wellbeing");
    await expect(
      page.getByTestId("wellbeing-rules").getByText("Seasonal flu vaccination"),
    ).toBeVisible();
    await expect(
      page.getByTestId("wellbeing-rules").getByText("engineering"),
    ).toBeVisible();
    await expect(
      page.getByTestId("wellbeing-rules").getByText("Cycle-to-work scheme"),
    ).toBeVisible();
    await expect(
      page.getByTestId("wellbeing-rules").getByText("Benefit"),
    ).toBeVisible();
    await expect(
      page.getByTestId("wellbeing-uptake").getByText("80% (4/5)"),
    ).toBeVisible();
    await expect(
      page.getByTestId("wellbeing-uptake").getByText(/Enrolled after prompt/),
    ).toBeVisible();
    await expect(
      page
        .getByTestId("wellbeing-uptake")
        .getByText(/50% \(2\/4\)/)
        .first(),
    ).toBeVisible();
    await expect(page.getByTestId("pulse-overall")).toContainText("3.5");
    await expect(
      page.getByTestId("pulse-results").getByText("Hidden below 5 responses"),
    ).toBeVisible();
  });

  test("privacy area renders the retention report and runs the sweep", async ({
    page,
  }) => {
    await page.route("**/api/proxy/retention", (route) =>
      route.fulfill({
        json: {
          as_of: "2026-07-25T00:00:00Z",
          horizon_days: 365,
          soft_deleted_past_horizon: { workers: 2, time_entries: 14 },
          expired_consent_candidates: 3,
          derivation:
            "soft-deleted rows older than the horizon are hard-deleted by the sweep",
        },
      }),
    );
    await page.route("**/api/proxy/retention/sweep", (route) =>
      route.fulfill({
        json: {
          horizon_days: 365,
          deleted: { workers: 2, time_entries: 14 },
          rows_deleted: 16,
          candidates_scrubbed: 3,
        },
      }),
    );
    await page.goto("/privacy");
    const report = page.getByTestId("retention-report");
    await expect(report.getByText("365")).toBeVisible();
    await expect(report.getByText("time_entries")).toBeVisible();
    await page.getByTestId("run-sweep").click();
    await expect(page.getByTestId("sweep-result")).toContainText(
      "16 rows deleted",
    );
  });
});
